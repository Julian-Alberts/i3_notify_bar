#![deny(clippy::unwrap_used)]
mod args;
mod components;
mod config_parser;
// Currently disabled
//mod debug_config;
mod icons;
mod notification_bar;
mod path_manager;
mod rule;
mod template;

use args::Args;
use components::NotificationBar;
use emoji::EmojiMode;
use i3_bar_components::{
    component_manager::{ComponentManagerBuilder, ManageComponents},
    components::{prelude::Urgent, Label},
    string::AnimatedString,
};
use log::{debug, error};
use notification_bar::{MinimalUrgency, NotificationEvent, NotificationManager};
use path_manager::PathManager;
use rule::Definition;
use std::{
    io::BufReader,
    path::Path,
    sync::{Arc, RwLock, Mutex},
    time::Duration,
};

#[macro_use]
extern crate pest_derive;

#[async_std::main]
async fn main() {
    let mut path_manager = PathManager::default();
    let Args {
        emoji_mode,
        emoji_file,
        log_level,
        log_file,
        refresh_rate,
        max_text_length,
        animation_chars_per_second,
        config_file,
        command,
    } = args::load();

    if let Some(file) = config_file {
        path_manager.set_config_file(file)
    }

    if let Some(file) = log_file {
        path_manager.set_log_file(file);
    }

    if let Some(file) = emoji_file {
        path_manager.set_emoji_file(file);
    }

    logger::init(log_level, path_manager.log_file());

    let config = read_config(path_manager.config_file());
    emoji::init(path_manager.emoji_file().as_ref().map(Path::new));

    drop(path_manager);

    match command {
        args::Command::Run => run(
            config,
            emoji_mode,
            max_text_length,
            animation_chars_per_second,
            refresh_rate,
        ),
        args::Command::DebugConfig(_) => eprintln!("Currently disabled"),
        // args::Command::DebugConfig(dc) => debug_config::debug_config(&config, emoji_mode, dc),
    }
}

fn run(
    config: Vec<Definition>,
    emoji_mode: EmojiMode,
    max_text_length: usize,
    animation_chars_per_second: usize,
    refresh_rate: u64,
) {
    let minimal_urgency = Arc::new(RwLock::new(MinimalUrgency::All));

    let mut component_manager = ComponentManagerBuilder::new()
        .with_click_events(true)
        .build();

    component_manager.set_global_event_listener(|_, ce| {
        debug!("{}", ce.get_button().to_string());
    });

    let notify_server =
        notify_server::NotifyServer::start().expect("Error starting notification server.");
    let notification_manager = Arc::new(Mutex::new(NotificationManager::new(
        config,
        emoji_mode,
        Arc::clone(&minimal_urgency),
        notify_server,
    )));
    
    let nm_ref: &Arc<_> = &notification_manager;
    component_manager.add_component(Box::new(NotificationBar::new(
        minimal_urgency,
        Arc::clone(nm_ref),
        max_text_length,
        animation_chars_per_second,
    )));

    let mut last_update = std::time::SystemTime::now();

    loop {
        let mut nm_lock = notification_manager.lock();
        let nm = match nm_lock.as_mut() {
            Ok(nm) => nm,
            Err(_) => {
                error!("Could not lock notification manager");
                break;
            }
        };
        nm.update(
            last_update
                .elapsed()
                .map(|e| e.as_secs_f64())
                .unwrap_or_default(),
        );
        drop(nm_lock);
        last_update = std::time::SystemTime::now();

        component_manager.update();
        std::thread::sleep(Duration::from_millis(refresh_rate));
    }
}

fn read_config(config_file: Option<&Path>) -> Vec<crate::rule::Definition> {
    match config_file {
        Some(path) => {
            let config_file = match std::fs::File::open(path) {
                Ok(f) => f,
                Err(e) => {
                    error!("Could not open file {:#?} error: {:#?}", path, e);
                    return Vec::new();
                }
            };
            let mut config_file = BufReader::new(config_file);
            match rule::parse_config(&mut config_file) {
                Ok(r) => r,
                Err(e) => {
                    error!("{}", e.to_string());
                    print_error(e.to_string());
                }
            }
        }
        None => Vec::new(),
    }
}

fn print_error(data: String) -> ! {
    let mut cm = ComponentManagerBuilder::new()
        .with_click_events(false)
        .build();

    let data = data.replace('\n', "");
    let mut animated_data = AnimatedString::new(data);
    animated_data.set_max_width(50);
    let mut label = Label::new(animated_data);
    label.set_urgent(true);
    cm.add_component(Box::new(label));
    cm.update();
    loop {
        std::thread::sleep(Duration::new(10, 0));
    }
}

mod logger {
    use std::{fs::OpenOptions, path::Path};

    use log::{error, LevelFilter};
    use simplelog::{ColorChoice, CombinedLogger, Config, SharedLogger, TermLogger, WriteLogger};

    pub fn init(level_filter: LevelFilter, log_file: Option<&Path>) {
        let logger: Box<dyn SharedLogger> = if let Some(path) = &log_file {
            if let Some(parent) = path.parent() {
                create_folder(parent)
            }
            let file = OpenOptions::new().create(true).append(true).open(path);
            match file {
                Ok(file) => WriteLogger::new(level_filter, Config::default(), file),
                Err(_) => TermLogger::new(
                    level_filter,
                    Config::default(),
                    simplelog::TerminalMode::Stderr,
                    ColorChoice::Auto,
                ),
            }
        } else {
            TermLogger::new(
                level_filter,
                Config::default(),
                simplelog::TerminalMode::Stderr,
                ColorChoice::Auto,
            )
        };

        if CombinedLogger::init(vec![logger]).is_err() {
            error!("Could not init logger")
        }
    }

    fn create_folder(parent: &Path) {
        if let Err(e) = std::fs::create_dir_all(parent) {
            log::error!(
                "Failed to create folder {} Error: {}",
                parent
                    .to_str()
                    .unwrap_or(r#""File name is not valid UTF-8""#),
                e
            );
            super::print_error(format!(
                "Error trying to create config folder at {} Error: {}",
                parent
                    .to_str()
                    .unwrap_or(r#""File name is not valid UTF-8""#),
                e
            ))
        }
    }
}
