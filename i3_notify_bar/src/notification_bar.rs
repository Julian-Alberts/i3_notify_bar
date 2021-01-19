use std::collections::BTreeMap;

use notify_server::{Event, Observer, notification::Notification};
use std::sync::Arc;
use crate::rule::rule::Action;

use crate::rule::rule::{Definition as RuleDefinition, Style};

pub struct NotificationManager {
    changed: Vec<Arc<(Notification, Vec<Style>)>>,
    notifications: BTreeMap<u32, Arc<(Notification, Vec<Style>)>>,
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

    fn notify(&mut self, n: &Notification) {

        let mut styles = Vec::new();
        let mut n = n.clone();

        for rule in &self.rules {
            if rule.matches(&n) {
                for action in &rule.actions {
                    match action {
                        Action::Ignore => {
                            return;
                        },
                        Action::Set(set_property) => set_property.set(&mut n)
                    }
                }
                
                styles = rule.style.clone();

            }
        }

        let n_id = n.id;
        let n = Arc::new((n, styles));
        self.changed.push(Arc::clone(&n));
        self.notifications.insert(n_id, n);
    }

    pub fn get_changed(&mut self) -> Vec<Arc<(Notification, Vec<Style>)>> {
        let mut changed = Vec::new();
        std::mem::swap(&mut changed, &mut self.changed);
        changed
    }

}

impl Observer<Event> for NotificationManager {

    fn on_notify(&mut self, event: &Event) {
        match event {
            Event::Notify(n) => self.notify(n)
        }
    }

}