use std::sync::{Arc, Mutex};

use i3_bar_components::{components::{Label, Button, ProgressBar, prelude::*, BaseComponent}, protocol::ClickEvent, ManageComponents, };
use log::debug;

use crate::{notification_bar::{NotificationManager, NotificationData}, icons};

use super::{close_type::CloseType, label::AnimatedLabel};

pub struct NotificationComponent {
    close_type: CloseType,
    label: AnimatedLabel,
    padding_r: Label,
    id: String,
    notification_manager: Arc<Mutex<NotificationManager>>,
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
                let mut b = Button::new(format!(" {} ", icons::X_ICON));
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
                CloseType::Timer(t)
            }
        };

        
        let mut label = AnimatedLabel {
            max_width,
            move_chars_per_sec,
            start_offset: 0.0,
            text: nd.text.clone(),
            stop_animation_for_secs: 0.0,
            label: Label::new(String::new()),
            icon: nd.icon
        };

        
        label.set_seperator(false);
        label.set_separator_block_width(0);

        let mut padding_r = Label::new(String::from(" "));

        nd.style.iter().for_each(|s| {
            s.apply(label.get_base_component_mut());
            s.apply(padding_r.get_base_component_mut());
        });

        Self {
            close_type,
            label,
            id: nd.id.to_owned(),
            notification_manager,
            padding_r,
        }
    }

    pub fn update_notification(&mut self, nd: &NotificationData) {
        self.label.icon = nd.icon;
        self.label.text = nd.text.to_string();
        self.label.update(0.);
        let close_type = match nd.expire_timeout {
            -1 => {
                let mut b = Button::new(String::from(" X "));
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
                CloseType::Timer(t)
            }
        };
        self.close_type = close_type;
    }

}

impl Component for NotificationComponent {
    fn collect_base_components<'a>(&'a self, base_components: &mut Vec<&'a BaseComponent>) {
        self.label.collect_base_components(base_components);
        self.close_type.collect_base_components(base_components);
        self.padding_r.collect_base_components(base_components);
    }

    fn collect_base_components_mut<'a>(
        &'a mut self,
        base_components: &mut Vec<&'a mut BaseComponent>,
    ) {
        self.label.collect_base_components_mut(base_components);
        self.close_type.collect_base_components_mut(base_components);
        self.padding_r.collect_base_components_mut(base_components);
    }

    fn event(&mut self, _: &mut dyn ManageComponents, ce: &ClickEvent) {
        if self.close_type.is_button()
            && ce.get_button() == 1
            && ce.get_id() == self.close_type.get_id()
        {
            match self.notification_manager.lock() {
                Ok(nm) => nm,
                Err(_) => {
                    debug!("Could not lock notification manager");
                    return;
                }
            }
            .remove(&self.id);
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
            .remove(&self.id);
        }

        self.label.update(dt);
        self.close_type.update(dt);
    }

    fn name(&self) -> &str {
        &self.id
    }

    fn get_id(&self) -> &str {
        &self.id
    }
}