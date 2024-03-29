use i3_bar_components::components::{
    prelude::{Component, EventTarget, SimpleComponent},
    Button,
};
use notify_server::notification::Action;

use crate::{
    icons,
    notification_bar::{InvokeAction as _, NotificationManagerCommands},
};
pub struct ActionBar {
    buttons: Vec<ActionButton>,
    close_btn: Button,
    notification_id: notify_server::NotificationId,
    notification_manager_cmd: NotificationManagerCommands,
}

impl ActionBar {
    pub fn new(
        actions: &[Action],
        notification_id: notify_server::NotificationId,
        notification_manager_cmd: NotificationManagerCommands,
    ) -> Self {
        let buttons = actions
            .iter()
            .map(|a| ActionButton {
                button: Button::new(Box::new(format!(" {} ", a.text.clone()))),
                key: a.key.clone(),
            })
            .collect::<Vec<_>>();

        let btn_text = format!(
            " {} ",
            icons::get_icon("close").map_or("close".to_owned(), |i| i.to_string())
        );
        let close_btn = Button::new(Box::new(btn_text));

        Self {
            buttons,
            notification_id,
            close_btn,
            notification_manager_cmd,
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
                .flat_map(Component::all_properties)
                .chain(self.close_btn.all_properties()),
        )
    }

    fn update(&mut self, dt: f64) {
        self.close_btn.update(dt);
        self.buttons.iter_mut().for_each(|b| b.update(dt))
    }

    fn event_targets<'a>(
        &'a self,
    ) -> Box<
        dyn Iterator<
                Item = (
                    i3_bar_components::property::Instance,
                    *const dyn i3_bar_components::components::prelude::EventTarget,
                ),
            > + 'a,
    > {
        Box::new(
            self.buttons
                .iter()
                .map(|b| (b.instance(), self as *const _))
                .chain(std::iter::once((
                    self.close_btn.instance(),
                    self as *const _,
                ))),
        )
    }
}

impl EventTarget for ActionBar {
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
            self.notification_manager_cmd
                .action_invoked(self.notification_id, &button.key)
        }
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
        Box::new([self.button.properties()].into_iter())
    }

    fn update(&mut self, dt: f64) {
        self.button.update(dt)
    }
}
