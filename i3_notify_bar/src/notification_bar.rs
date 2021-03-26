use std::time::SystemTime;

use log::{debug, info};
use notify_server::{Event, Observer, notification::Notification};
use serde::Serialize;
use crate::{icons, rule::Action};

use crate::rule::{Definition as RuleDefinition, Style};

pub struct NotificationManager {
    new_notifications: Vec<NotificationData>,
    rules: Vec<RuleDefinition>
}

impl NotificationManager {

    pub fn new(rules: Vec<RuleDefinition>) -> Self {
        Self {
            new_notifications: Vec::new(),
            rules
        }
    }

    fn notify(&mut self, n: Notification) {
        info!(r#"Got new notification app_name "{}" summary "{}" body "{}""#, n.app_name, n.summary, n.body);
        debug!("Notification: {:#?}", n);

        let mut notification_data = NotificationData {
            expire_timeout: n.expire_timeout,
            icon: icons::get_icon(&n.app_name).unwrap_or(' '),
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

        self.new_notifications.push(notification_data);        
    }

    pub fn get_new(&mut self) -> Vec<NotificationData> {
        let mut new_notifications = Vec::new();
        std::mem::swap(&mut new_notifications, &mut self.new_notifications);
        new_notifications
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
    pub icon: char,
    pub text: String,
    pub style: Vec<Style>
}

#[derive(Debug, Serialize)]
pub struct NotificationTemplateData {
    pub app_name: String, 
    pub icon: String, 
    pub summary: String, 
    pub body: String, 
    pub expire_timeout: i32,
    pub time: SystemTime
}