use std::sync::{Arc, Mutex, RwLock};

use i3_bar_components::{
    components::{prelude::*, Button, Label},
    string::{AnimatedString, PartiallyAnimatedString},
};

use crate::{
    icons,
    notification_bar::{NotificationData, NotificationManager},
};

pub struct NotificationGroup {
    label: Label<PartiallyAnimatedString>,
    notifications: Vec<Arc<RwLock<NotificationData>>>,
    notification_manager: Arc<Mutex<NotificationManager>>,
    max_width: usize,
    move_chars_per_sec: usize,
    group_name: String,
}

struct NotificationGroupCloseButton {
    button: Button,
}

impl NotificationGroup {
    pub fn new(
        group_name: String,
        notification_manager: Arc<Mutex<NotificationManager>>,
        max_width: usize,
        move_chars_per_sec: usize,
        notifications: Vec<Arc<RwLock<NotificationData>>>,
    ) -> Self {
        let string = PartiallyAnimatedString::new(
            None,
            AnimatedString::new(group_name.clone()),
            Some(format!(" {}", notifications.len())),
        );
        let mut label = Label::new(string);
        label.set_show(true);
        label.set_block_width(Some(10));
        Self {
            label,
            notifications,
            notification_manager,
            max_width,
            move_chars_per_sec,
            group_name,
        }
    }

    pub fn add(&mut self, nd: Arc<RwLock<NotificationData>>) {
        self.notifications.push(nd);
        self.label
            .text_mut()
            .set_right_static(Some(format!(" {}", self.notifications.len())));
    }
    pub fn remove(&mut self, id: notify_server::NotificationId) {
        self.notifications
            .retain(|n| n.read().map(|n| n.id != id).unwrap_or_default());
        self.label
            .text_mut()
            .set_right_static(self.notifications.len().to_string().into());
    }
    pub fn is_empty(&self) -> bool {
        self.notifications.is_empty()
    }
}

impl SimpleComponent for NotificationGroup {
    fn properties(&self) -> &i3_bar_components::property::Properties {
        self.label.properties()
    }
    fn properties_mut(&mut self) -> &mut i3_bar_components::property::Properties {
        self.label.properties_mut()
    }
}

impl Component for NotificationGroup {
    fn all_properties<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = &i3_bar_components::property::Properties> + 'a> {
        self.label.all_properties()
    }

    fn event(
        &mut self,
        cm: &mut dyn i3_bar_components::ManageComponents,
        event: &i3_bar_components::protocol::ClickEvent,
    ) {
        let Some(event_instance) = event.get_instance() else {
            return;
        };
        if self.label.instance() != event_instance {
            return;
        }
        cm.new_layer();
        self.notifications.iter().for_each(|n| {
            cm.add_component(Box::new(super::NotificationComponent::new(
                Arc::clone(n),
                self.max_width,
                self.move_chars_per_sec,
                self.notification_manager.clone(),
            )));
        });
        cm.add_component(Box::new(NotificationGroupCloseButton::new()));
    }

    fn update(&mut self, dt: f64) {
        self.label.update(dt)
    }

    fn name(&self) -> Option<&str> {
        Some(&self.group_name)
    }
}

impl NotificationGroupCloseButton {
    fn new() -> Self {
        Self {
            button: Button::new(Box::new(format!(
                " {} ",
                icons::get_icon("close").unwrap_or('X')
            ))),
        }
    }
}

impl SimpleComponent for NotificationGroupCloseButton {
    fn properties(&self) -> &i3_bar_components::property::Properties {
        self.button.properties()
    }
    fn properties_mut(&mut self) -> &mut i3_bar_components::property::Properties {
        self.button.properties_mut()
    }
}

impl Component for NotificationGroupCloseButton {
    fn all_properties<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = &i3_bar_components::property::Properties> + 'a> {
        self.button.all_properties()
    }

    fn event(
        &mut self,
        cm: &mut dyn i3_bar_components::ManageComponents,
        event: &i3_bar_components::protocol::ClickEvent,
    ) {
        let Some(event_instance) = event.get_instance() else {
            return;
        };
        if self.button.instance() != event_instance {
            return;
        }
        cm.pop_layer();
    }

    fn update(&mut self, dt: f64) {
        self.button.update(dt)
    }
}
