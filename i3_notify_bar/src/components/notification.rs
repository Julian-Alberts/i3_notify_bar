use std::sync::{Arc, Mutex};

use i3_bar_components::{
    components::{prelude::*, BaseComponent, Button, Label, Padding, ProgressBar},
    protocol::ClickEvent,
    string::{AnimatedString, PartiallyAnimatedString},
    ManageComponents,
};
use log::debug;
use notify_server::{notification::Action, CloseReason};

use crate::{
    icons,
    notification_bar::{NotificationData, NotificationManager},
};

use super::{action_bar::ActionBar, close_type::CloseType};

pub struct NotificationComponent {
    close_type: CloseType,
    label: Label,
    id: u32,
    name: String,
    notification_manager: Arc<Mutex<NotificationManager>>,
    actions: Vec<Action>,
    max_width: usize,
    move_chars_per_sec: usize,
    right_padding: Padding,
}

impl NotificationComponent {
    pub fn new(
        nd: &NotificationData,
        max_width: usize,
        move_chars_per_sec: usize,
        notification_manager: Arc<Mutex<NotificationManager>>,
    ) -> NotificationComponent {
        let close_type = match nd.expire_timeout {
            -1 => {
                let mut b = Button::new(format!(" {} ", icons::X_ICON).into());
                b.set_seperator(false);
                b.set_separator_block_width(0);
                nd.style.iter().for_each(|s| {
                    s.apply(b.get_base_component_mut());
                });
                CloseType::Button(b)
            }
            _ => {
                let mut t = ProgressBar::new(nd.expire_timeout as u64);
                t.set_seperator(false);
                t.set_separator_block_width(0);
                nd.style.iter().for_each(|s| {
                    s.apply(t.get_base_component_mut());
                });
                CloseType::Timer(Box::new(t))
            }
        };

        let animated_notification_text =
            notification_data_to_animated_text(nd, max_width, move_chars_per_sec);
        let mut label = Label::new(animated_notification_text.into());

        label.set_seperator(false);
        label.set_separator_block_width(0);

        let mut right_padding = Padding::new(1);

        nd.style.iter().for_each(|s| {
            s.apply(label.get_base_component_mut());
            s.apply(right_padding.get_base_component_mut());
        });

        Self {
            close_type,
            label,
            id: nd.id,
            notification_manager,
            actions: nd.actions.clone(),
            name: notification_id_to_notification_compnent_name(nd.id),
            max_width,
            move_chars_per_sec,
            right_padding,
        }
    }

    pub fn update_notification(&mut self, nd: &NotificationData) {
        self.label.set_text(
            notification_data_to_animated_text(nd, self.max_width, self.move_chars_per_sec).into(),
        );
        self.label.update(0.);
        let close_type = match nd.expire_timeout {
            -1 => {
                let mut b = Button::new(String::from(" X ").into());
                b.set_seperator(false);
                b.set_separator_block_width(0);
                nd.style.iter().for_each(|s| {
                    s.apply(b.get_base_component_mut());
                });
                CloseType::Button(b)
            }
            _ => {
                let mut t = ProgressBar::new(nd.expire_timeout as u64);
                t.set_seperator(false);
                t.set_separator_block_width(0);
                nd.style.iter().for_each(|s| {
                    s.apply(t.get_base_component_mut());
                });
                CloseType::Timer(Box::new(t))
            }
        };
        self.close_type = close_type;
    }

    fn on_close_button_click(&self) {
        match self.notification_manager.lock() {
            Ok(nm) => nm,
            Err(_) => {
                debug!("Could not lock notification manager");
                return;
            }
        }
        .remove(self.id, &CloseReason::Closed);
    }

    fn on_notification_right_click(&mut self, mc: &mut dyn ManageComponents) {
        mc.new_layer();
        mc.add_component(Box::new(ActionBar::new(
            &self.actions,
            self.id,
            Arc::clone(&self.notification_manager),
        )))
    }

    fn on_notification_click(&mut self) {
        let action = self.actions.iter().find(|action| action.key == "default");

        if let Some(action) = action {
            self.notification_manager
                .lock()
                .expect("Could not lock notification manager")
                .action_invoked(self.id, &action.key)
        }
    }
}

impl Component for NotificationComponent {
    fn collect_base_components<'a>(&'a self, base_components: &mut Vec<&'a BaseComponent>) {
        self.label.collect_base_components(base_components);
        self.close_type.collect_base_components(base_components);
        self.right_padding.collect_base_components(base_components)
    }

    fn collect_base_components_mut<'a>(
        &'a mut self,
        base_components: &mut Vec<&'a mut BaseComponent>,
    ) {
        self.label.collect_base_components_mut(base_components);
        self.close_type.collect_base_components_mut(base_components);
        self.right_padding
            .collect_base_components_mut(base_components)
    }

    fn event(&mut self, mc: &mut dyn ManageComponents, ce: &ClickEvent) {
        if self.close_type.is_button()
            && ce.get_button() == 1
            && ce.get_instance()
                == self
                    .close_type
                    .get_base_component()
                    .get_properties()
                    .instance
        {
            self.on_close_button_click()
        } else if ce.get_button() == 3 {
            self.on_notification_right_click(mc)
        } else if ce.get_button() == 1 {
            self.on_notification_click()
        }
    }

    fn update(&mut self, dt: f64) {
        if self.close_type.is_timer() && self.close_type.is_finished() {
            match self.notification_manager.lock() {
                Ok(nm) => nm,
                Err(_) => {
                    debug!("Could not lock notification manager");
                    return;
                }
            }
            .remove(self.id, &CloseReason::Expired);
        }

        self.label.update(dt);
        self.close_type.update(dt);
    }

    fn name(&self) -> Option<&str> {
        Some(&self.name[..])
    }
}

pub fn notification_id_to_notification_compnent_name(id: u32) -> String {
    format!("i3_notify_bar_notification_component:{}", id)
}

pub fn notification_data_to_animated_text(
    nd: &NotificationData,
    max_width: usize,
    move_chars_per_sec: usize,
) -> PartiallyAnimatedString {
    let icon = if nd.icon != ' ' {
        Some(nd.icon.to_string())
    } else {
        None
    };
    PartiallyAnimatedString::new(
        icon,
        AnimatedString::new(nd.text.clone())
            .with_max_width(max_width)
            .with_move_chars_per_sec(move_chars_per_sec),
        String::from(" ").into(),
    )
}
