mod args;
mod components;
mod config_parser;
mod emoji;
mod icons;
mod notification_bar;
mod rule;
mod template;

use args::Args;
use clap::Clap;
use components::NotificationComponent;
use i3_bar_components::{components::Label, ComponentManagerBuilder};
use log::error;
use notification_bar::{NotificationEvent, NotificationManager};
use std::{
    io::BufReader,
    sync::{Arc, Mutex},
    time::Duration,
};

#[macro_use]
extern crate pest_derive;

fn main() {
    let args: Args = Args::parse();
    logger::init(args.log_level(), args.log_file());

    let config;
    match args.config_file() {
        Some(path) => {
            let config_file = std::fs::File::open(path).unwrap();
            let mut config_file = BufReader::new(config_file);
            config = match rule::parse_config(&mut config_file) {
                Ok(r) => r,
                Err(e) => {
                    error!("{}", e.to_string());
                    print_error(e.to_string());
                }
            };
        }
        None => config = Vec::new(),
    }

    let mut notify_server = notify_server::NotifyServer::start();
    let mut manager = ComponentManagerBuilder::new()
        .with_click_events(true)
        .build();
    let notification_manager = Arc::new(Mutex::new(NotificationManager::new(
        config,
        args.emoji_mode(),
    )));
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
                        args.max_text_length(),
                        args.animation_chars_per_second(),
                        Arc::clone(&notification_manager),
                    ))),
                }
            }
            &NotificationEvent::Remove(id) => manager.remove_by_name(id),
        });

        manager.update();
        std::thread::sleep(Duration::from_millis(args.refresh_rate()));
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
    use std::fs::OpenOptions;

    use log::LevelFilter;
    use simplelog::{ColorChoice, CombinedLogger, Config, SharedLogger, TermLogger, WriteLogger};

    pub fn init(level_filer: &LevelFilter, log_file: &Option<String>) {
        let logger: Box<dyn SharedLogger> = match &log_file {
            Some(path) => {
                let file = OpenOptions::new().create(true).append(true).open(path);
                match file {
                    Ok(file) => WriteLogger::new(level_filer.to_owned(), Config::default(), file),
                    Err(_) => TermLogger::new(
                        level_filer.to_owned(),
                        Config::default(),
                        simplelog::TerminalMode::Stderr,
                        ColorChoice::Auto,
                    ),
                }
            }
            None => TermLogger::new(
                level_filer.to_owned(),
                Config::default(),
                simplelog::TerminalMode::Stderr,
                ColorChoice::Auto,
            ),
        };

        CombinedLogger::init(vec![logger]).unwrap();
    }
}
