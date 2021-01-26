mod components;
mod notification_bar;
mod rule;
mod icons;

use std::{io::BufReader, sync::{Arc, Mutex}, time::Duration};
use components::NotificationComponent;
use i3_bar_components::ComponentManagerBuilder;
use log::{error, info};
use notification_bar::NotificationManager;

fn main() {
    logger::init();
    let args = std::env::args().collect::<Vec<String>>();

    let path = args.get(1);
    
    let rules;
    match path {
        Some(path) => {
            let config_file = std::fs::File::open(path).unwrap();
            let mut config_file = BufReader::new(config_file);
            rules = match rule::parser::parse_config(&mut config_file) {
                Ok(r) => r,
                Err(e) => {
                    error!("{}", e.to_string());
                    return
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
        std::thread::sleep(Duration::from_millis(250));
    }
}

mod logger {
    use std::fs::OpenOptions;

    use log::LevelFilter;
    use simplelog::{CombinedLogger, Config, TermLogger, WriteLogger, SharedLogger};

    pub fn init() {
        
        let path = std::env::current_exe();
        let logger: Box<dyn SharedLogger> = match path {
            Ok(mut path) => {
                path.pop();
                path.push("notify.log");
                let file = OpenOptions::new().create(true).append(true).open(path);
                match file {
                    Ok(file) => {
                        WriteLogger::new(LevelFilter::Info, Config::default(), file)
                    },
                    Err(_) => {
                        TermLogger::new(LevelFilter::Info, Config::default(), simplelog::TerminalMode::Stderr)
                    }
                }
            },
            Err(_) => {
                TermLogger::new(LevelFilter::Info, Config::default(), simplelog::TerminalMode::Stderr)
            }
        };

        CombinedLogger::init(
            vec![
                logger
            ]
        ).unwrap();
    }

}