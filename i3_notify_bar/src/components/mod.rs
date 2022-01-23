mod notification;
mod close_type;
mod label;
mod min_urgency_selector;

use i3_bar_components::{components::Button, ManageComponents, protocol::ClickEvent};
pub use notification::NotificationComponent;
pub use min_urgency_selector::init;

use crate::icons;

pub fn menu_button(is_menu_open: bool) -> Button {
    let icon = match is_menu_open {
        true => icons::get_icon("close").map_or(String::from("menu"), |c| c.to_string()),
        false => icons::get_icon("menu").map_or(String::from("menu"), |c| c.to_string())
    };
    let mut button = Button::new(icon);
    
    if is_menu_open {
        button.set_on_click(close_menu);
    } else {
        button.set_on_click(open_menu);
    }

    button
}

fn close_menu(_: &mut Button, mc: &mut dyn ManageComponents, ce: &ClickEvent) {
    mc.pop_layer()
}

fn open_menu(_: &mut Button, mc: &mut dyn ManageComponents, ce: &ClickEvent) {
    mc.new_layer();
    mc.add_component(Box::new(min_urgency_selector::init()));
    mc.add_component(Box::new(menu_button(true)));
}