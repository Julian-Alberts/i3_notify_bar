use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, RwLock};

use i3_bar_components::{
    components::{prelude::*, Button},
    protocol::ClickEvent,
    ManageComponents,
};

use crate::{
    icons,
    notification_bar::{MinimalUrgency, NotificationData, NotificationManager},
};

use super::{min_urgency_selector, NotificationComponent, NotificationGroup};

pub struct NotificationBar {
    notifications: Vec<NotificationComponent>,
    groups: BTreeMap<String, NotificationGroup>,
    menu_btn: Button,
    notification_manager: Arc<Mutex<NotificationManager>>,
    max_width: usize,
    animation_chars_per_second: usize,
}

impl NotificationBar {
    pub fn new(
        selected_urgency: Arc<RwLock<MinimalUrgency>>,
        notification_manager: Arc<Mutex<NotificationManager>>,
        max_width: usize,
        animation_chars_per_second: usize,
    ) -> Self {
        let icon = icons::get_icon("menu").map_or(String::from(" menu "), |c| format!(" {} ", c));
        let mut menu_btn = Button::new(Box::new(icon));

        let nm_c = Arc::clone(&notification_manager);
        
        let menu_btn_instance = menu_btn.instance();
        menu_btn.set_on_click(move |_, mc, ce| {
            let Some(instance) = ce.get_instance() else {
                return
            };
            if menu_btn_instance != instance { return }
            open_menu(mc, ce, selected_urgency.clone(), Arc::clone(&nm_c));
        });

        Self {
            notifications: Vec::default(),
            groups: BTreeMap::default(),
            menu_btn,
            notification_manager,
            max_width,
            animation_chars_per_second,
        }
    }
}

impl Component for NotificationBar {
    fn all_properties<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = &i3_bar_components::property::Properties> + 'a> {
        Box::new(
            self.notifications
                .iter()
                .map(Component::all_properties)
                .chain(
                    self.groups
                        .iter()
                        .map(|(_, b)| b)
                        .map(Component::all_properties),
                )
                .flatten()
                .chain(self.menu_btn.all_properties()),
        )
    }

    fn update(&mut self, dt: f64) {
        let Ok(mut nm) = self.notification_manager.lock() else {
            return;
        };
        nm.get_events().into_iter().for_each(|event| {
            use crate::NotificationEvent::*;
            match event {
                Add(n) => add_notification(
                    n,
                    &mut self.groups,
                    &mut self.notifications,
                    &self.notification_manager,
                    self.max_width,
                    self.animation_chars_per_second
                ),
                Remove(n) => remove_notification(
                    n, 
                    &mut self.groups, 
                    &mut self.notifications,
                ),
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

    fn event(
        &mut self,
        cm: &mut dyn i3_bar_components::ManageComponents,
        event: &i3_bar_components::protocol::ClickEvent,
    ) {
        self.notifications
            .iter_mut()
            .map::<&mut dyn Component, _>(|n| n)
            .chain(
                self.groups
                    .iter_mut()
                    .map::<&mut dyn Component, _>(|(_, g)| g),
            )
            .for_each(|c| c.event(cm, event));
        self.menu_btn.event(cm, event);
    }
}

fn add_notification(
    n: Arc<RwLock<NotificationData>>,
    groups: &mut BTreeMap<String, NotificationGroup>,
    notifications: &mut Vec<NotificationComponent>,
    notification_manager: &Arc<Mutex<NotificationManager>>,
    max_width: usize,
    move_chars_per_sec: usize,
) {
    let Ok(n_l) = n.read() else { return };
    if let Some(group) = &n_l.group {
        let group_name = group.to_string();
        drop(n_l);

        let group = groups.entry(group_name.clone()).or_insert_with(|| {
            NotificationGroup::new(group_name, notification_manager.clone(), max_width, move_chars_per_sec, vec![])
        });
        group.add(n);
    } else {
        drop(n_l);
        notifications.push(NotificationComponent::new(
            n,
            max_width,
            move_chars_per_sec,
            notification_manager.clone(),
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
    selected: Arc<RwLock<MinimalUrgency>>,
    notification_manager: Arc<Mutex<NotificationManager>>,
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
        notification_manager
            .lock()
            .expect("Could not lock notification manager")
            .close_all_notifications(notify_server::CloseReason::Dismissed);
    });
    let group = min_urgency_selector::init(selected);
    mc.add_component(Box::new(close_all));
    mc.add_component(Box::new(group));
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
