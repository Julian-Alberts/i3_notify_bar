use std::sync::{Arc, Mutex};

use i3_bar_components::{
    components::{prelude::*, Button, Label, ProgressBar},
    protocol::ClickEvent,
    string::{AnimatedString, PartiallyAnimatedString},
    ManageComponents,
};
use log::debug;
use notify_server::{notification::Action, CloseReason, NotificationId};

use crate::{
    icons,
    notification_bar::{NotificationData, NotificationManager},
    rule::Style,
};

use super::action_bar::ActionBar;

pub struct NotificationComponent {
    label: Label,
    id: NotificationId,
    close_button: Button,
    close_timer: Option<ProgressBar>,
    name: String,
    notification_manager: Arc<Mutex<NotificationManager>>,
    actions: Vec<Action>,
    max_width: usize,
    move_chars_per_sec: usize,
}

impl NotificationComponent {
    pub fn new(
        nd: &NotificationData,
        max_width: usize,
        move_chars_per_sec: usize,
        notification_manager: Arc<Mutex<NotificationManager>>,
    ) -> NotificationComponent {
        let close_timer = match nd.expire_timeout {
            -1 => None,
            _ => Some(create_timer(nd.style.as_slice(), nd.expire_timeout as u64)),
        };

        let animated_notification_text =
            notification_data_to_animated_text(nd, max_width, move_chars_per_sec);
        let mut label = Label::new(animated_notification_text.into());

        label.set_show(false);
        label.set_block_width(Some(0));

        nd.style.iter().for_each(|s| {
            s.apply(&mut label);
        });

        Self {
            label,
            close_button: create_button(nd.style.as_slice()),
            close_timer,
            id: nd.id,
            notification_manager,
            actions: nd.actions.clone(),
            name: notification_id_to_notification_compnent_name(nd.id),
            max_width,
            move_chars_per_sec,
        }
    }

    pub fn update_notification(&mut self, nd: &NotificationData) {
        self.label.set_text(
            notification_data_to_animated_text(nd, self.max_width, self.move_chars_per_sec).into(),
        );
        self.label.update(0.);
        let close_timer = match nd.expire_timeout {
            -1 => None,
            _ => Some(create_timer(nd.style.as_slice(), nd.expire_timeout as u64)),
        };
        self.close_timer = close_timer;
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
    fn all_properties<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = &i3_bar_components::property::Properties> + 'a> {
        Box::new(
            [
                Some(self.label.all_properties()),
                self.close_timer.as_ref().map(Component::all_properties),
                Some(self.close_button.all_properties()),
            ]
            .into_iter()
            .flatten()
            .flatten(),
        )
    }

    fn event(&mut self, mc: &mut dyn ManageComponents, ce: &ClickEvent) {
        let Some(instance) = ce.get_instance() else {
            return;
        };
        match ce.get_button() {
            // Button clicked
            1 if self.close_button.instance() == instance => self.on_close_button_click(),
            // Notification clicked
            1 => self.on_notification_click(),
            // Notification right click
            3 => self.on_notification_right_click(mc),
            _ => {}
        }
    }

    fn update(&mut self, dt: f64) {
        if self
            .close_timer
            .as_ref()
            .map(|t| t.is_finished())
            .unwrap_or(false)
        {
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
        self.close_button.update(dt);
        if let Some(t) = self.close_timer.as_mut() {
            t.update(dt);
        }
    }

    fn name(&self) -> Option<&str> {
        Some(&self.name[..])
    }
}

fn create_button(style: &[Style]) -> Button {
    let mut b = Button::new(format!(" {} ", icons::X_ICON).into());
    b.set_show(false);
    b.set_block_width(Some(0));
    style.iter().for_each(|s| {
        s.apply(&mut b);
    });
    b
}

fn create_timer(style: &[Style], expire: u64) -> ProgressBar {
    let mut t = ProgressBar::new(expire);
    t.set_show(false);
    t.set_block_width(Some(0));
    style.iter().for_each(|s| {
        s.apply(&mut t);
    });
    t
}

pub fn notification_id_to_notification_compnent_name(id: NotificationId) -> String {
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
