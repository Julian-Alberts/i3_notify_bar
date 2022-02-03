mod args;
mod components;
mod config_parser;
mod debug_config;
mod icons;
mod notification_bar;
mod path_manager;
mod rule;
mod template;

use args::Args;
use clap::Parser;
use components::{NotificationComponent, notification_id_to_notification_compnent_name};
use emoji::EmojiMode;
use i3_bar_components::{
    component_manager::{ComponentManagerBuilder, ManageComponents},
    components::Label, string::AnimatedString,
};
use log::{debug, error};
use notification_bar::{MinimalUrgency, NotificationEvent, NotificationManager};
use path_manager::PathManager;
use rule::Definition;
use std::{
    io::BufReader,
    path::Path,
    sync::{Arc, Mutex},
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
    } = Args::parse();

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
        Some(args::Command::Run) | None => run(
            config,
            emoji_mode,
            max_text_length,
            animation_chars_per_second,
            refresh_rate,
        ),
        Some(args::Command::DebugConfig(dc)) => debug_config::debug_config(&config, emoji_mode, dc),
    }
}

fn run(
    config: Vec<Definition>,
    emoji_mode: EmojiMode,
    max_text_length: usize,
    animation_chars_per_second: usize,
    refresh_rate: u64,
) {
    let minimal_urgency = Arc::new(Mutex::new(MinimalUrgency::All));

    let mut component_manager = ComponentManagerBuilder::new()
        .with_click_events(true)
        .build();
    component_manager.add_component(Box::new(components::menu_button_open(Arc::clone(
        &minimal_urgency,
    ))));
    component_manager.set_global_event_listener(|_, ce| {
        debug!("{}", ce.get_button().to_string());
    });

    let notify_server = notify_server::NotifyServer::start().unwrap();
    let notification_manager =
        NotificationManager::new(config, emoji_mode, minimal_urgency, notify_server);

    loop {
        let mut nm_lock = notification_manager.lock();
        let nm = match nm_lock.as_mut() {
            Ok(nm) => nm,
            Err(_) => {
                error!("Could not lock notification manager");
                break;
            }
        };
        let events = nm.get_events();
        drop(nm_lock);

        events.iter().for_each(|event| match &event {
            &NotificationEvent::Add(n) | &NotificationEvent::Update(n) => {
                let n = match n.read() {
                    Ok(n) => n,
                    Err(_) => {
                        error!("Could not lock notification data");
                        return;
                    }
                };
                match component_manager.get_component_mut::<NotificationComponent>(&notification_id_to_notification_compnent_name(n.id)) {
                    Some(c) => c.update_notification(&n),
                    None => component_manager.add_component_at_on_layer(
                        Box::new(NotificationComponent::new(
                            &n,
                            max_text_length,
                            animation_chars_per_second,
                            Arc::clone(&notification_manager),
                        )),
                        -1,
                        0,
                    ),
                }
            }
            &NotificationEvent::Remove(id) => component_manager.remove_by_name(&notification_id_to_notification_compnent_name(*id)),
        });

        component_manager.update();
        std::thread::sleep(Duration::from_millis(refresh_rate));
    }
}

fn read_config(config_file: &Option<String>) -> Vec<crate::rule::Definition> {
    match config_file {
        Some(path) => {
            let config_file = match std::fs::File::open(path) {
                Ok(f) => f,
                Err(e) => {
                    error!("Could not open file {} error: {:#?}", path, e);
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
    use i3_bar_components::components::prelude::Component;
    let mut cm = ComponentManagerBuilder::new()
        .with_click_events(false)
        .build();

    let data = data.replace("\n", "");
    let mut animated_data = AnimatedString::new(data);
    animated_data.set_max_width(50);
    let mut label = Label::new(animated_data.into());
    let mut base_components = Vec::new();
    label.collect_base_components_mut(&mut base_components);
    base_components[0].get_properties_mut().urgent = true;
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

    pub fn init(level_filter: LevelFilter, log_file: &Option<String>) {
        let logger: Box<dyn SharedLogger> = match &log_file {
            Some(path) => {
                let path = Path::new(path);
                if let Some(parent) = path.parent() {
                    match std::fs::create_dir_all(parent) {
                        Ok(()) => {}
                        Err(e) => {
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
            }
            None => TermLogger::new(
                level_filter,
                Config::default(),
                simplelog::TerminalMode::Stderr,
                ColorChoice::Auto,
            ),
        };

        if let Err(_) = CombinedLogger::init(vec![logger]) {
            error!("Could not init logger")
        }
    }
}
