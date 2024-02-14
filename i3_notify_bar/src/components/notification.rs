use std::{
    sync::{Arc, RwLock},
    usize,
};

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
    notification_bar::{
        CloseNotification, InvokeAction as _, NotificationData, NotificationManagerCommands,
    },
    rule::Style,
};

use super::action_bar::ActionBar;

pub struct NotificationComponent {
    notification: Arc<RwLock<NotificationData>>,
    label: Label<PartiallyAnimatedString>,
    close_button: Button,
    close_timer: Option<ProgressBar>,
    name: String,
    notification_manager_cmd: NotificationManagerCommands,
    actions: Vec<Action>,
    max_width: usize,
    move_chars_per_sec: usize,
    notification_state_id: usize,
}

impl NotificationComponent {
    pub fn new(
        nd: Arc<RwLock<NotificationData>>,
        max_width: usize,
        move_chars_per_sec: usize,
        notification_manager_cmd: NotificationManagerCommands,
    ) -> NotificationComponent {
        let nd_l = nd
            .write()
            .expect("Unable to lock notification while creating component");
        let close_timer = match nd_l.expire_timeout {
            -1 => None,
            _ => Some(create_timer(
                nd_l.style.as_slice(),
                nd_l.expire_timeout as f64,
            )),
        };

        let animated_notification_text =
            notification_data_to_animated_text(&nd_l, max_width, move_chars_per_sec);
        let mut label = Label::new(animated_notification_text);

        label.set_show(true);
        label.set_block_width(Some(0));

        nd_l.style.iter().for_each(|s| {
            s.apply(&mut label);
        });

        let name = notification_id_to_notification_compnent_name(nd_l.id);
        let close_button = create_button(
            nd_l.style.as_slice(),
            nd_l.id,
            name.clone(),
            notification_manager_cmd.clone(),
        );
        let actions = nd_l.actions.clone();
        let notification_state_id = nd_l.notification_update_id;
        drop(nd_l);
        Self {
            notification: nd,
            close_button,
            name,
            label,
            close_timer,
            notification_manager_cmd,
            actions,
            max_width,
            move_chars_per_sec,
            notification_state_id,
        }
    }

    fn reinit(&mut self) {
        let new = Self::new(
            Arc::clone(&self.notification),
            self.max_width,
            self.move_chars_per_sec,
            self.notification_manager_cmd.clone(),
        );
        *self = new;
    }

    fn on_notification_right_click(&mut self, mc: &mut dyn ManageComponents) {
        mc.new_layer();
        mc.add_component(Box::new(ActionBar::new(
            &self.actions,
            self.id(),
            self.notification_manager_cmd.clone(),
        )))
    }

    fn on_notification_click(&mut self) {
        let action = self.actions.iter().find(|action| action.key == "default");

        if let Some(action) = action {
            self.notification_manager_cmd
                .action_invoked(self.id(), &action.key)
        }
    }

    pub fn id(&self) -> NotificationId {
        self.notification
            .read()
            .expect("Unable to create read lock")
            .id
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

    fn update(&mut self, dt: f64) {
        let notification_lock = self.notification.read();
        if let Ok(notification) = &notification_lock {
            if notification.notification_update_id != self.notification_state_id {
                drop(notification_lock);
                self.reinit();
            }
        };

        self.label.update(dt);
        self.close_button.update(dt);
        if let Some(t) = self.close_timer.as_mut() {
            let n = self
                .notification
                .read()
                .expect("Unable to read notification");
            t.set_current(
                n.remove_in_secs
                    .map(|rmis| n.expire_timeout as f64 - rmis)
                    .unwrap_or_default(),
            );
            t.update(dt);
        }
    }

    fn name(&self) -> Option<&str> {
        Some(&self.name)
    }

    fn event_targets<'a>(
        &'a self,
    ) -> Box<
        dyn Iterator<
                Item = (
                    i3_bar_components::property::Instance,
                    *const dyn EventTarget,
                ),
            > + 'a,
    > {
        Box::new(
            self.close_button
                .event_targets()
                .chain(std::iter::once((self.label.instance(), self as *const _))),
        )
    }
}

impl EventTarget for NotificationComponent {
    fn event(&mut self, mc: &mut dyn ManageComponents, event: &ClickEvent) {
        match event.get_button() {
            // Notification clicked
            1 => self.on_notification_click(),
            // Notification right click
            3 => self.on_notification_right_click(mc),
            _ => {}
        }
    }
}

fn create_button(
    style: &[Style],
    id: NotificationId,
    name: String,
    notification_manager_cmd: NotificationManagerCommands,
) -> Button {
    let mut b = Button::new(Box::new(format!(" {} ", icons::X_ICON)));
    b.set_show(true);
    b.set_block_width(Some(10));
    style.iter().for_each(|s| {
        s.apply(&mut b);
    });
    b.set_on_click(move |_, cm, _| {
        debug!("Notification({id}) close button clicked");
        notification_manager_cmd.notification_closed(id, CloseReason::Closed);
        cm.remove_by_name(&name);
    });
    b
}

fn create_timer(style: &[Style], expire: f64) -> ProgressBar {
    let mut t = ProgressBar::new(expire);
    t.set_show(false);
    t.set_block_width(Some(10));
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
        Some(format!("{} ", nd.icon))
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
