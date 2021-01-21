mod components;
mod notification_bar;
mod rule;
mod icons;

use std::{io::BufReader, sync::{Arc, Mutex}, time::Duration};
use components::NotificationComponent;
use i3_bar_components::ComponentManagerBuilder;
use notification_bar::NotificationManager;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    let path = args.get(1);
    
    let rules;
    match path {
        Some(path) => {
            let config_file = std::fs::File::open(path).unwrap();
            let mut config_file = BufReader::new(config_file);
            rules = rule::parser::parse_config(&mut config_file).unwrap();
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
            match manager.get_component_mut::<NotificationComponent>(&format!("{}", n.0.id)) {
                Some(c) => c.update_notification(&n.0, &n.1),
                None => {
                    manager.add_component(Box::new(NotificationComponent::new(&n.0, &n.1)))
                },
            }
        });
        drop(nm_lock);
        
        manager.update();
        std::thread::sleep(Duration::new(1, 0));
    }
}