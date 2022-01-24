mod action_bar;
mod close_type;
mod label;
mod min_urgency_selector;
mod notification;

use std::sync::{Arc, Mutex};

use i3_bar_components::{components::Button, protocol::ClickEvent, ManageComponents};
pub use min_urgency_selector::init;
pub use notification::NotificationComponent;

use crate::{icons, notification_bar::MinimalUrgency};

pub fn menu_button_open(selected: Arc<Mutex<MinimalUrgency>>) -> Button {
    let icon = icons::get_icon("menu").map_or(String::from(" menu "), |c| format!(" {} ", c));
    let mut button = Button::new(icon);

    button.set_on_click(move |_, mc, ce| {
        open_menu(mc, ce, selected.clone());
    });

    button
}

pub fn menu_button_close() -> Button {
    let icon = icons::get_icon("close").map_or(String::from(" close "), |c| format!(" {} ", c));
    let mut button = Button::new(icon);
    button.set_on_click(&close_menu);
    button
}

fn close_menu(_: &mut Button, mc: &mut dyn ManageComponents, ce: &ClickEvent) {
    if ce.get_button() != 1 {
        return;
    };
    mc.pop_layer()
}

fn open_menu(mc: &mut dyn ManageComponents, ce: &ClickEvent, selected: Arc<Mutex<MinimalUrgency>>) {
    if ce.get_button() != 1 {
        return;
    };
    mc.new_layer();
    let group = min_urgency_selector::init(selected);
    mc.add_component(Box::new(group));
    mc.add_component(Box::new(menu_button_close()));
}
