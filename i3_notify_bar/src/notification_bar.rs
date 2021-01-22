use std::collections::BTreeMap;

use notify_server::{Event, Observer, notification::Notification};
use std::sync::Arc;
use crate::rule::rule::Action;

use crate::rule::rule::{Definition as RuleDefinition, Style};

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

        let rule = self.rules.iter().find(|r| r.matches(&n));

        let mut notification_data = NotificationData {
            expire_timeout: n.expire_timeout,
            icon: n.app_icon.clone(),
            id: n.id,
            style: Vec::new(),
            text: n.summary.clone()
        };

        if let Some(rule) = rule {
            for action in &rule.actions {
                match action {
                    Action::Ignore => {
                        return;
                    },
                    Action::Set(set_property) => set_property.set(&mut notification_data, &n),
                }

            }
            
            notification_data.style = rule.style.clone();
        }

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

pub struct NotificationData {
    pub id: u32,
    pub expire_timeout: i32,
    pub icon: String,
    pub text: String,
    pub style: Vec<Style>
}