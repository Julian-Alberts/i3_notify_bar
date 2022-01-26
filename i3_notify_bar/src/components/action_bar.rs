use i3_bar_components::components::{
    prelude::{Component, Widget},
    Button,
};
use notify_server::notification::Action;

use crate::icons;
pub struct ActionBar {
    buttons: Vec<ActionButton>,
    close_btn: Button,
    notification_id: u32,
}

impl ActionBar {
    pub fn new(
        actions: &[Action],
        notification_id: u32,
    ) -> Self {
        let buttons = actions
            .iter()
            .map(|a| ActionButton {
                button: Button::new(format!(" {} ", a.text.clone())),
                key: a.key.clone(),
            })
            .collect::<Vec<_>>();

        let btn_text = format!(
            " {} ",
            icons::get_icon("close").map_or("close".to_owned(), |i| i.to_string())
        );
        let close_btn = Button::new(btn_text);

        Self {
            buttons,
            notification_id,
            close_btn,
        }
    }
}

impl Component for ActionBar {
    fn collect_base_components<'a>(
        &'a self,
        base_components: &mut Vec<&'a i3_bar_components::components::BaseComponent>,
    ) {
        self.buttons
            .iter()
            .for_each(|b| b.collect_base_components(base_components));
        self.close_btn.collect_base_components(base_components);
    }

    fn collect_base_components_mut<'a>(
        &'a mut self,
        base_components: &mut Vec<&'a mut i3_bar_components::components::BaseComponent>,
    ) {
        self.buttons
            .iter_mut()
            .for_each(|b| b.collect_base_components_mut(base_components));
        self.close_btn.collect_base_components_mut(base_components);
    }

    fn event(
        &mut self,
        mc: &mut dyn i3_bar_components::ManageComponents,
        event: &i3_bar_components::protocol::ClickEvent,
    ) {
        if event.get_button() != 1 {
            return;
        }

        if event.get_instance()
            == &self
                .close_btn
                .get_base_component()
                .get_properties()
                .instance
        {
            mc.pop_layer()
        }

        let button = self
            .buttons
            .iter()
            .find(|b| &b.get_base_component().get_properties().instance == event.get_instance());

        if let Some(button) = button {
            /*self.notification_tx
                .send(NotificationMessage::ActionInvoked(
                    self.notification_id,
                    button.key.clone(),
                ))
                .unwrap()*/
        }
    }

    fn get_id(&self) -> &str {
        ""
    }

    fn name(&self) -> &str {
        ""
    }

    fn update(&mut self, _: f64) {}
}

struct ActionButton {
    button: Button,
    key: String,
}

impl Component for ActionButton {
    fn collect_base_components<'a>(
        &'a self,
        base_components: &mut Vec<&'a i3_bar_components::components::BaseComponent>,
    ) {
        self.button.collect_base_components(base_components)
    }

    fn collect_base_components_mut<'a>(
        &'a mut self,
        base_components: &mut Vec<&'a mut i3_bar_components::components::BaseComponent>,
    ) {
        self.button.collect_base_components_mut(base_components)
    }

    fn event(
        &mut self,
        _: &mut dyn i3_bar_components::ManageComponents,
        _: &i3_bar_components::protocol::ClickEvent,
    ) {
    }

    fn get_id(&self) -> &str {
        self.button.get_id()
    }

    fn name(&self) -> &str {
        self.button.name()
    }

    fn update(&mut self, _: f64) {}
}

impl Widget for ActionButton {
    fn get_base_component(&self) -> &i3_bar_components::components::BaseComponent {
        self.button.get_base_component()
    }

    fn get_base_component_mut(&mut self) -> &mut i3_bar_components::components::BaseComponent {
        self.button.get_base_component_mut()
    }
}
