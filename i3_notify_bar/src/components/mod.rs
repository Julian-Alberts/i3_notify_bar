mod action_bar;
mod close_type;
mod min_urgency_selector;
mod notification;

use std::sync::{Arc, Mutex};

use i3_bar_components::{
    components::{Button, Padding},
    protocol::ClickEvent,
    ManageComponents,
};
use log::debug;
pub use min_urgency_selector::init;
pub use notification::{notification_id_to_notification_compnent_name, NotificationComponent};
use notify_server::CloseReason;

use crate::{
    icons,
    notification_bar::{MinimalUrgency, NotificationManager},
};

pub fn menu_button_open(
    selected: Arc<Mutex<MinimalUrgency>>,
    notification_manager: Arc<Mutex<NotificationManager>>,
) -> Button {
    let icon = icons::get_icon("menu").map_or(String::from(" menu "), |c| format!(" {} ", c));
    let mut button = Button::new(icon.into());

    button.set_on_click(move |_, mc, ce| {
        open_menu(mc, ce, selected.clone(), notification_manager.clone());
    });

    button
}

pub fn menu_button_close() -> Button {
    let icon = icons::get_icon("close").map_or(String::from(" close "), |c| format!(" {} ", c));
    let mut button = Button::new(icon.into());
    button.set_on_click(close_menu);
    button
}

fn close_menu(_: &mut Button, mc: &mut dyn ManageComponents, ce: &ClickEvent) {
    if ce.get_button() != 1 {
        return;
    };
    mc.pop_layer()
}

fn open_menu(
    mc: &mut dyn ManageComponents,
    ce: &ClickEvent,
    selected: Arc<Mutex<MinimalUrgency>>,
    notification_manager: Arc<Mutex<NotificationManager>>,
) {
    if ce.get_button() != 1 {
        return;
    };
    mc.new_layer();
    let mut close_all = Button::new(" close all ".to_owned().into());
    close_all.set_on_click(move |_, _, ce| {
        debug!("close button clicked");
        if ce.get_button() != 1 {
            return;
        };
        notification_manager
            .lock()
            .expect("Could not lock notification manager")
            .close_all_notifications(CloseReason::Dismissed);
    });
    let group = min_urgency_selector::init(selected);
    mc.add_component(Box::new(close_all));
    mc.add_component(Box::new(Padding::new(2)));
    mc.add_component(Box::new(group));
    mc.add_component(Box::new(menu_button_close()));
}
