mod components;
mod notification_bar;
mod rule;
mod icons;
mod args;
mod template;
mod mini_template;

use std::{io::BufReader, sync::{Arc, Mutex}, time::Duration};
use components::NotificationComponent;
use i3_bar_components::{ComponentManagerBuilder, components::Label};
use log::error;
use notification_bar::NotificationManager;
use args::Args;
use clap::Clap;

fn main() {
    let args: Args = Args::parse();
    logger::init(args.log_level(), args.log_file());
    
    let rules;
    match args.rule_file() {
        Some(path) => {
            let config_file = std::fs::File::open(path).unwrap();
            let mut config_file = BufReader::new(config_file);
            rules = match rule::parse_config(&mut config_file) {
                Ok(r) => r,
                Err(e) => {
                    error!("{}", e.to_string());
                    print_error(e.to_string());
                }
            };
        },
        None => {
            rules = Vec::new()
        }
    }

    
    let mut notify_server = notify_server::NotifyServer::start();
    let mut manager = ComponentManagerBuilder::new().with_click_events(true).build();
    let notification_manager = Arc::new(Mutex::new(NotificationManager::new(rules)));
    notify_server.add_observer(notification_manager.clone());

    loop {

        let mut nm_lock = notification_manager.lock();
        let nm = nm_lock.as_mut().unwrap();
        let changed = nm.get_changed();
        
        changed.iter().for_each(|n| {
            match manager.get_component_mut::<NotificationComponent>(&format!("{}", n.id)) {
                Some(c) => c.update_notification(&n),
                None => {
                    manager.add_component(Box::new(NotificationComponent::new(&n)))
                },
            }
        });
        drop(nm_lock);
        
        manager.update();
        std::thread::sleep(Duration::from_millis(args.refresh_rate()));
    }
}

fn print_error(data: String) -> ! {
    use i3_bar_components::components::prelude::Component;
    let mut cm = ComponentManagerBuilder::new().with_click_events(false).build();
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
    use simplelog::{CombinedLogger, Config, TermLogger, WriteLogger, SharedLogger};

    pub fn init(level_filer: &LevelFilter, log_file: &Option<String>) {
        
        let logger: Box<dyn SharedLogger> = match &log_file {
            Some(path) => {
                let file = OpenOptions::new().create(true).append(true).open(path);
                match file {
                    Ok(file) => {
                        WriteLogger::new(level_filer.to_owned(), Config::default(), file)
                    },
                    Err(_) => {
                        TermLogger::new(level_filer.to_owned(), Config::default(), simplelog::TerminalMode::Stderr)
                    }
                }
            },
            None => {
                TermLogger::new(level_filer.to_owned(), Config::default(), simplelog::TerminalMode::Stderr)
            }
        };

        CombinedLogger::init(
            vec![
                logger
            ]
        ).unwrap();
    }

}