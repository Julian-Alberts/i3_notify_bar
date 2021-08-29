mod args;
mod components;
mod config_parser;
mod debug_config;
mod emoji;
mod icons;
mod notification_bar;
mod path_manager;
mod rule;
mod template;

use args::Args;
use clap::Clap;
use components::NotificationComponent;
use emoji::EmojiMode;
use i3_bar_components::{components::Label, ComponentManagerBuilder};
use log::error;
use notification_bar::{NotificationEvent, NotificationManager};
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

fn main() {
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
    crate::emoji::init(path_manager.emoji_file().as_ref().map(Path::new));

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
) -> ! {
    let mut notify_server = notify_server::NotifyServer::start();
    let mut manager = ComponentManagerBuilder::new()
        .with_click_events(true)
        .build();
    let notification_manager = Arc::new(Mutex::new(NotificationManager::new(config, emoji_mode)));
    notify_server.add_observer(notification_manager.clone());

    loop {
        let mut nm_lock = notification_manager.lock();
        let nm = nm_lock.as_mut().unwrap();
        let events = nm.get_events();
        drop(nm_lock);

        events.iter().for_each(|event| match &event {
            &NotificationEvent::Add(n) | &NotificationEvent::Update(n) => {
                let n = n.read().unwrap();
                match manager.get_component_mut::<NotificationComponent>(&n.id) {
                    Some(c) => c.update_notification(&n),
                    None => manager.add_component(Box::new(NotificationComponent::new(
                        &n,
                        max_text_length,
                        animation_chars_per_second,
                        Arc::clone(&notification_manager),
                    ))),
                }
            }
            &NotificationEvent::Remove(id) => manager.remove_by_name(id),
        });

        manager.update();
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
    let mut label = Label::new(data);
    let mut base_components = Vec::new();
    label.collect_base_components_mut(&mut base_components);
    base_components[0].set_urgent(true);
    cm.add_component(Box::new(label));
    cm.update();
    loop {
        std::thread::sleep(Duration::new(10, 0));
    }
}

mod logger {
    use std::{fs::OpenOptions, path::Path};

    use log::LevelFilter;
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
                                parent.to_str().unwrap(),
                                e
                            );
                            super::print_error(format!(
                                "Error trying to create config folder at {} Error: {}",
                                parent.to_str().unwrap(),
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

        CombinedLogger::init(vec![logger]).unwrap();
    }
}
