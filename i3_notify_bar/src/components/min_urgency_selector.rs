use std::sync::{Arc, Mutex};

use i3_bar_components::components::{Button, prelude::Widget, ButtonGroup, GroupButton};
use notify_server::notification::Urgency;

const BUTTON_CONFIG: [ButtonConfig; 3] = [
    ButtonConfig {
        color: "#00FF00",
        text: "low",
        key: Urgency::Low
    },
    ButtonConfig {
        color: "#F5E642",
        text: "normal",
        key: Urgency::Normal
    },
    ButtonConfig {
        color: "#F5424B",
        text: "critical",
        key: Urgency::Critical
    }
];


pub fn init(selected: Arc<Mutex<Urgency>>) -> ButtonGroup<Urgency> {

    let buttons = BUTTON_CONFIG.iter().fold(Vec::with_capacity(BUTTON_CONFIG.len()), |mut vec, config| {
        let button = Button::from(config);
        vec.push(GroupButton::new(config.key as isize, *&config.key, button));
        vec
    });

    ButtonGroup::new(buttons, selected)
}

struct ButtonConfig<'a> {
    text: &'a str,
    color: &'a str,
    key: Urgency
}

impl <'a> From<&'a ButtonConfig<'a>> for Button {

    fn from(config: &'a ButtonConfig) -> Self {
        let mut button = Button::new(config.text.to_owned());
        let properties = button.get_base_component_mut().get_properties_mut();
        properties.color.text = Some(config.color.to_owned());
        properties.border.color = Some(config.color.to_owned());
        button
    }

}