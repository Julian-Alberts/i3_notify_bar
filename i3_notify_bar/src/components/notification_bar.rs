use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

use i3_bar_components::{
    components::{prelude::*, Button},
    protocol::ClickEvent,
    ManageComponents,
};

use crate::notification_bar::{
    CloseAllNotifications as _, NotificationEvent, NotificationManagerCommands, SharedConfig,
};
use crate::SystemCommand;
use crate::{icons, notification_bar::NotificationData};

use super::{min_urgency_selector, NotificationComponent, NotificationGroup};

pub struct NotificationBar<'a> {
    notifications: Vec<NotificationComponent>,
    groups: BTreeMap<String, NotificationGroup>,
    menu_btn: Button,
    notification_manager_cmd: NotificationManagerCommands,
    max_width: usize,
    animation_chars_per_second: usize,
    notification_event_channel: std::sync::mpsc::Receiver<NotificationEvent>,
    config: &'a crate::rule::Config,
}

impl<'a> NotificationBar<'a> {
    pub fn new(
        shared_config: SharedConfig,
        notification_manager_cmd: NotificationManagerCommands,
        notification_event_channel: std::sync::mpsc::Receiver<NotificationEvent>,
        max_width: usize,
        animation_chars_per_second: usize,
        system_command_tx: std::sync::mpsc::Sender<SystemCommand>,
        config: &'a crate::rule::Config,
    ) -> Self {
        let icon = icons::get_icon("menu").map_or(String::from(" menu "), |c| format!(" {} ", c));
        let mut menu_btn = Button::new(Box::new(icon));

        let menu_btn_instance = menu_btn.instance();
        let nm_cmd = notification_manager_cmd.clone();
        menu_btn.set_on_click(move |_, mc, ce| {
            let Some(instance) = ce.get_instance() else {
                return;
            };
            if menu_btn_instance != instance {
                return;
            }
            let _ = system_command_tx.send(SystemCommand::ForceUpdate);
            open_menu(mc, ce, shared_config.clone(), nm_cmd.clone());
        });

        Self {
            notifications: Vec::default(),
            groups: BTreeMap::default(),
            menu_btn,
            notification_manager_cmd,
            notification_event_channel,
            max_width,
            animation_chars_per_second,
            config,
        }
    }
}

impl<'a> Component for NotificationBar<'a> {
    fn all_properties<'b>(
        &'b self,
    ) -> Box<dyn Iterator<Item = &i3_bar_components::property::Properties> + 'b> {
        Box::new(
            self.notifications
                .iter()
                .map(Component::all_properties)
                .chain(self.groups.values().map(Component::all_properties))
                .flatten()
                .chain(self.menu_btn.all_properties()),
        )
    }

    fn update(&mut self, dt: f64) {
        self.notification_event_channel
            .try_iter()
            .for_each(|event| {
                use crate::NotificationEvent::*;
                match event {
                    Add(n) => add_notification(
                        n,
                        &mut self.groups,
                        &self.config,
                        &mut self.notifications,
                        &self.notification_manager_cmd,
                        self.max_width,
                        self.animation_chars_per_second,
                    ),
                    Remove(n) => remove_notification(n, &mut self.groups, &mut self.notifications),
                }
            });

        self.notifications
            .iter_mut()
            .map::<&mut dyn Component, _>(|n| n)
            .chain(
                self.groups
                    .iter_mut()
                    .map::<&mut dyn Component, _>(|(_, g)| g),
            )
            .for_each(|c| c.update(dt));
        self.groups.retain(|_, g| !g.is_empty());
        self.menu_btn.update(dt);
    }

    fn event_targets<'b>(
        &'b self,
    ) -> Box<
        dyn Iterator<
                Item = (
                    i3_bar_components::property::Instance,
                    *const dyn EventTarget,
                ),
            > + 'b,
    > {
        Box::new(
            self.menu_btn
                .event_targets()
                .chain(
                    self.notifications
                        .iter()
                        .map(Component::event_targets)
                        .flatten(),
                )
                .chain(self.groups.values().map(Component::event_targets).flatten()),
        )
    }
}

fn add_notification(
    n: Arc<RwLock<NotificationData>>,
    groups: &mut BTreeMap<String, NotificationGroup>,
    config: &crate::rule::Config,
    notifications: &mut Vec<NotificationComponent>,
    notification_manager_cmd: &NotificationManagerCommands,
    max_width: usize,
    move_chars_per_sec: usize,
) {
    let Ok(n_l) = n.read() else { return };
    if let Some(group) = &n_l.group {
        let group_name = group.to_string();
        drop(n_l);

        let group = groups.entry(group_name.clone()).or_insert_with(|| {
            let mut group = NotificationGroup::new(
                group_name.clone(),
                notification_manager_cmd.clone(),
                max_width,
                move_chars_per_sec,
                vec![],
            );
            config
                .default_group
                .as_ref()
                .map(|g| g.style.iter().for_each(|s| s.apply(&mut group)));
            config
                .groups
                .get(&group_name)
                .as_ref()
                .map(|g| g.style.iter().for_each(|s| s.apply(&mut group)));
            group
        });
        group.add(n);
    } else {
        drop(n_l);
        notifications.push(NotificationComponent::new(
            n,
            max_width,
            move_chars_per_sec,
            notification_manager_cmd.clone(),
        ))
    }
}

fn remove_notification(
    n: Arc<RwLock<NotificationData>>,
    groups: &mut BTreeMap<String, NotificationGroup>,
    notifications: &mut Vec<NotificationComponent>,
) {
    let Ok(n_l) = n.read() else { return };
    if let Some(group) = &n_l.group {
        if let Some(group) = groups.get_mut(group) {
            group.remove(n_l.id);
        }
    } else {
        notifications.retain(|nc| nc.id() != n_l.id)
    }
}

fn open_menu(
    mc: &mut dyn ManageComponents,
    ce: &ClickEvent,
    shared_config: SharedConfig,
    notification_manager_cmd: NotificationManagerCommands,
) {
    if ce.get_button() != 1 {
        return;
    };
    mc.new_layer();
    let mut close_all = Button::new(Box::new(" close all ".to_owned()));
    close_all.set_on_click(move |_, _, ce| {
        log::debug!("close button clicked");
        if ce.get_button() != 1 {
            return;
        };
        notification_manager_cmd.close_all_notifications(notify_server::CloseReason::Dismissed);
    });

    #[cfg(feature = "audio")]
    let toggle_audio = {
        let toggle_audio_icon = if shared_config.is_audio_enabled() {
            icons::AUDIO_ACTIVE
        } else {
            icons::AUDIO_MUTED
        };
        let shared_config_i = shared_config.clone();
        let mut toggle_audio = Button::new(Box::new(toggle_audio_icon.to_string()));
        toggle_audio.set_on_click(move |this, _, ce| {
            if ce.get_button() != 1 {
                return;
            }
            if shared_config_i.is_audio_enabled() {
                shared_config_i.audio_enabled(false);
                this.set_text(icons::AUDIO_MUTED.to_string());
            } else {
                shared_config_i.audio_enabled(true);
                this.set_text(icons::AUDIO_ACTIVE.to_string());
            }
        });
        toggle_audio
    };

    let group = min_urgency_selector::init(shared_config.minimum_urgency);
    mc.add_component(Box::new(close_all));
    mc.add_component(Box::new(group));
    #[cfg(feature = "audio")]
    mc.add_component(Box::new(toggle_audio));
    mc.add_component(Box::new(menu_button_close()));
}

fn close_menu(_: &mut Button, mc: &mut dyn ManageComponents, ce: &ClickEvent) {
    if ce.get_button() != 1 {
        return;
    };
    mc.pop_layer()
}

pub fn menu_button_close() -> Button {
    let icon = icons::get_icon("close").map_or(String::from(" close "), |c| format!(" {} ", c));
    let mut button = Button::new(Box::new(icon));
    button.set_on_click(close_menu);
    button
}
