use std::sync::{Arc, Mutex};

use i3_bar_components::components::{
    prelude::{Component, SimpleComponent},
    Button,
};
use notify_server::notification::Action;

use crate::{icons, notification_bar::NotificationManager};
pub struct ActionBar {
    buttons: Vec<ActionButton>,
    close_btn: Button,
    notification_id: notify_server::NotificationId,
    notification_manager: Arc<Mutex<NotificationManager>>,
}

impl ActionBar {
    pub fn new(
        actions: &[Action],
        notification_id: notify_server::NotificationId,
        notification_manager: Arc<Mutex<NotificationManager>>,
    ) -> Self {
        let buttons = actions
            .iter()
            .map(|a| ActionButton {
                button: Button::new(format!(" {} ", a.text.clone()).into()),
                key: a.key.clone(),
            })
            .collect::<Vec<_>>();

        let btn_text = format!(
            " {} ",
            icons::get_icon("close").map_or("close".to_owned(), |i| i.to_string())
        );
        let close_btn = Button::new(btn_text.into());

        Self {
            buttons,
            notification_id,
            close_btn,
            notification_manager,
        }
    }
}

impl Component for ActionBar {
    fn all_properties<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = &i3_bar_components::property::Properties> + 'a> {
        Box::new(
            self.buttons
                .iter()
                .map(Component::all_properties)
                .flatten()
                .chain(self.close_btn.all_properties()),
        )
    }

    fn event(
        &mut self,
        mc: &mut dyn i3_bar_components::ManageComponents,
        event: &i3_bar_components::protocol::ClickEvent,
    ) {
        let Some(event_element) = event.get_instance() else {
            return;
        };
        if event.get_button() != 1 {
            return;
        }

        if self.close_btn.instance() == event_element {
            mc.pop_layer()
        }

        let button = self.buttons.iter().find(|b| b.instance() == event_element);

        if let Some(button) = button {
            self.notification_manager
                .lock()
                .expect("Could not lock notification bar")
                .action_invoked(self.notification_id, &button.key)
        }
    }

    fn update(&mut self, dt: f64) {
        self.close_btn.update(dt);
        self.buttons.iter_mut().for_each(|b| b.update(dt))
    }
}

struct ActionButton {
    button: Button,
    key: String,
}

impl SimpleComponent for ActionButton {
    fn properties(&self) -> &i3_bar_components::property::Properties {
        self.button.properties()
    }
    fn properties_mut(&mut self) -> &mut i3_bar_components::property::Properties {
        self.button.properties_mut()
    }
}

impl Component for ActionButton {
    fn all_properties<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = &i3_bar_components::property::Properties> + 'a> {
        Box::new([self.properties()].into_iter())
    }

    fn event(
        &mut self,
        _: &mut dyn i3_bar_components::ManageComponents,
        _: &i3_bar_components::protocol::ClickEvent,
    ) {
    }

    fn update(&mut self, _: f64) {}
}
