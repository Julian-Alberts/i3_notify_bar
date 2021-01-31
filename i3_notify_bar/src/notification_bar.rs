use std::{collections::BTreeMap, time::SystemTime};

use log::{debug, info};
use notify_server::{Event, Observer, notification::Notification};
use serde::Serialize;
use std::sync::Arc;
use crate::{icons, rule::Action};

use crate::rule::{Definition as RuleDefinition, Style};

pub struct NotificationManager {
    changed: Vec<Arc<NotificationData>>,
    notifications: BTreeMap<u32, Arc<NotificationData>>,
    rules: Vec<RuleDefinition>
}

impl NotificationManager {

    pub fn new(rules: Vec<RuleDefinition>) -> Self {
        Self {
            changed: Vec::new(),
            notifications: BTreeMap::new(),
            rules
        }
    }

    fn notify(&mut self, n: Notification) {
        info!(r#"Got new notification app_name "{}" summary "{}" body "{}""#, n.app_name, n.summary, n.body);
        debug!("Notification: {:#?}", n);

        let mut notification_data = NotificationData {
            expire_timeout: n.expire_timeout,
            icon: icons::get_icon(&n.app_name).unwrap_or(' ').to_string(),
            id: n.id,
            style: Vec::new(),
            text: n.summary.clone()
        };
        debug!("Notification Data: {:#?}", notification_data);

        let notification_template_data = NotificationTemplateData {
            app_name: n.app_name.clone(),
            icon: n.app_icon.clone(),
            summary: n.summary.clone(),
            body: n.body.clone(),
            expire_timeout: n.expire_timeout,
            time: SystemTime::now()
        };
        debug!("Notification Tempalate Data: {:#?}", notification_template_data);

        let mut last_rule_id = 0;
        let mut read_next_rule = true;

        while read_next_rule {
            let rule = self.rules[last_rule_id..]
                .iter()
                .enumerate()
                .find(|(_, r)| r.matches(&n));

            match rule { 
                Some((index, rule)) => { 
                    debug!("Matched rule {} {:#?}", last_rule_id + index, rule.rules);
                    last_rule_id += index + 1;

                    for action in &rule.actions {
                        match action {
                            Action::Ignore => {
                                debug!("Ignore Message");
                                return
                            },
                            Action::Set(set_property) => set_property.set(&mut notification_data, &notification_template_data),
                            Action::Stop => read_next_rule = false
                        }

                    }
                    
                    notification_data.style.extend(rule.style.clone());
                },
                None => read_next_rule = false
            }
        }

        debug!("Finished rules");
        debug!("Final notification_data {:#?}", notification_data);

        let notification = Arc::new(notification_data);
        self.changed.push(Arc::clone(&notification));
        self.notifications.insert(notification.id, notification);
        
    }

    pub fn get_changed(&mut self) -> Vec<Arc<NotificationData>> {
        let mut changed = Vec::new();
        std::mem::swap(&mut changed, &mut self.changed);
        changed
    }

}

impl Observer<Event> for NotificationManager {

    fn on_notify(&mut self, event: Event) {
        match event {
            Event::Notify(n) => self.notify(n)
        }
    }

}

#[derive(Debug)]
pub struct NotificationData {
    pub id: u32,
    pub expire_timeout: i32,
    pub icon: String,
    pub text: String,
    pub style: Vec<Style>
}

#[derive(Debug, Serialize)]
pub struct NotificationTemplateData {
    app_name: String, 
    icon: String, 
    summary: String, 
    body: String, 
    expire_timeout: i32,
    time: SystemTime
}