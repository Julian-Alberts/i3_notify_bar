use std::sync::{Arc, RwLock};

use i3_bar_components::components::{Button, ButtonGroup, GroupButton, Label};

use crate::notification_bar::MinimalUrgency;

const BUTTON_CONFIG: [ButtonConfig; 4] = [
    ButtonConfig {
        color: "#00FF00",
        text: " low ",
        key: MinimalUrgency::All,
    },
    ButtonConfig {
        color: "#F5E642",
        text: " normal ",
        key: MinimalUrgency::Normal,
    },
    ButtonConfig {
        color: "#F5424B",
        text: " critical ",
        key: MinimalUrgency::Critical,
    },
    ButtonConfig {
        color: "",
        text: " off ",
        key: MinimalUrgency::None,
    },
];

pub fn init(selected: Arc<RwLock<MinimalUrgency>>) -> ButtonGroup<MinimalUrgency, String> {
    let buttons = BUTTON_CONFIG.iter().fold(
        Vec::with_capacity(BUTTON_CONFIG.len()),
        |mut vec, config| {
            let button = Button::from(config);
            vec.push(GroupButton::new(config.key as isize, config.key, button));
            vec
        },
    );

    let description = Label::new("Minimal urgency".to_string());

    ButtonGroup::new(buttons, selected, Some(description))
}

struct ButtonConfig<'a> {
    text: &'a str,
    color: &'a str,
    key: MinimalUrgency,
}

impl<'a> From<&'a ButtonConfig<'a>> for Button {
    fn from(config: &'a ButtonConfig) -> Self {
        use i3_bar_components::components::prelude::*;
        let mut button = Button::new(Box::new(config.text.to_owned()));
        button.set_color_text(Some(config.color.to_owned()));
        button.set_border_color(Some(config.color.to_owned()));
        button
    }
}
