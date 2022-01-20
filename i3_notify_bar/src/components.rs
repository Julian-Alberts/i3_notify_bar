use std::sync::{Arc, Mutex};

use i3_bar_components::{
    components::{prelude::*, BaseComponent, Button, Label, ProgressBar},
    protocol::ClickEvent,
    ComponentManagerMessenger,
};

use log::debug;

use crate::icons;
use crate::notification_bar::{NotificationData, NotificationManager};

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

    fn event(&mut self, ce: &ClickEvent) {
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

    fn add_component_manager_messenger(&mut self, _: ComponentManagerMessenger) {}

    fn get_id(&self) -> &str {
        &self.id
    }
}

enum CloseType {
    Button(Button),
    Timer(ProgressBar),
}

impl CloseType {
    fn is_button(&self) -> bool {
        matches!(self, Self::Button(_))
    }

    fn is_timer(&self) -> bool {
        matches!(self, Self::Timer(_))
    }

    fn is_finished(&self) -> bool {
        match self {
            Self::Timer(t) => t.is_finished(),
            _ => false,
        }
    }
}

impl Component for CloseType {
    fn update(&mut self, dt: f64) {
        match self {
            Self::Button(b) => b.update(dt),
            Self::Timer(t) => t.update(dt),
        }
    }

    fn add_component_manager_messenger(
        &mut self,
        component_manager_messanger: ComponentManagerMessenger,
    ) {
        match self {
            Self::Button(b) => b.add_component_manager_messenger(component_manager_messanger),
            Self::Timer(t) => t.add_component_manager_messenger(component_manager_messanger),
        }
    }

    fn collect_base_components<'a>(&'a self, base_components: &mut Vec<&'a BaseComponent>) {
        match self {
            Self::Button(b) => b.collect_base_components(base_components),
            Self::Timer(t) => t.collect_base_components(base_components),
        }
    }

    fn collect_base_components_mut<'a>(
        &'a mut self,
        base_components: &mut Vec<&'a mut BaseComponent>,
    ) {
        match self {
            Self::Button(b) => b.collect_base_components_mut(base_components),
            Self::Timer(t) => t.collect_base_components_mut(base_components),
        }
    }

    fn event(&mut self, event: &ClickEvent) {
        match self {
            Self::Button(b) => b.event(event),
            Self::Timer(t) => t.event(event),
        }
    }

    fn get_id(&self) -> &str {
        match self {
            Self::Button(b) => b.get_id(),
            Self::Timer(t) => t.get_id(),
        }
    }

    fn name(&self) -> &str {
        match self {
            Self::Button(b) => b.name(),
            Self::Timer(t) => t.name(),
        }
    }
}

impl Widget for CloseType {
    fn get_base_component(&self) -> &BaseComponent {
        match self {
            Self::Button(b) => b.get_base_component(),
            Self::Timer(t) => t.get_base_component(),
        }
    }

    fn get_base_component_mut(&mut self) -> &mut BaseComponent {
        match self {
            Self::Button(b) => b.get_base_component_mut(),
            Self::Timer(t) => t.get_base_component_mut(),
        }
    }
}

struct AnimatedLabel {
    start_offset: f64,
    max_width: usize,
    move_chars_per_sec: usize,
    text: String,
    stop_animation_for_secs: f64,
    label: Label,
    icon: char
}

impl AnimatedLabel {
    fn update_offset(&mut self, dt: f64) {
        let text_len = self.text.chars().count();

        if text_len <= self.max_width {
            return;
        }

        if self.stop_animation_for_secs > 0.0 {
            self.stop_animation_for_secs -= dt;
            return;
        }

        let move_chars = self.move_chars_per_sec as f64 * dt;
        self.start_offset += move_chars;
        if self.start_offset as usize >= text_len {
            self.start_offset = 0.0;
            self.stop_animation_for_secs = 1.0;
        }
    }
}

impl Component for AnimatedLabel {

    fn add_component_manager_messenger(&mut self, component_manager_messanger: ComponentManagerMessenger) {
        self.label.add_component_manager_messenger(component_manager_messanger);
    }
    fn event(&mut self, event: &ClickEvent) {
        self.label.event(event)
    }
    fn collect_base_components<'a>(&'a self, base_components: &mut Vec<&'a BaseComponent>) {
        self.label.collect_base_components(base_components)
    }
    fn collect_base_components_mut<'a>(&'a mut self, base_components: &mut Vec<&'a mut BaseComponent>) {
        self.label.collect_base_components_mut(base_components)
    }
    fn get_id(&self) -> &str {
        self.label.get_id()
    }
    fn name(&self) -> &str {
        self.label.name()
    }
    fn update(&mut self, dt: f64) {
        self.update_offset(dt);
        self.label.set_text(self.to_string());
    }

}

impl Widget for AnimatedLabel {

    fn get_base_component(&self) -> &BaseComponent {
        self.label.get_base_component()
    }

    fn get_base_component_mut(&mut self) -> &mut BaseComponent {
        self.label.get_base_component_mut()
    }

}

impl ToString for AnimatedLabel {
    fn to_string(&self) -> String {
        let text_len = self.text.chars().count();

        if text_len <= self.max_width {
            return self.text.to_owned();
        }
        let end;
        if self.start_offset as usize + self.max_width < text_len {
            end = self.start_offset as usize + self.max_width;
        } else {
            end = text_len;
        }

        let chars = self.text.chars().collect::<Vec<char>>();
        let chars = &chars[self.start_offset as usize..end];

        let main_text = format!(
            "{text: <width$}",
            text = chars.iter().collect::<String>(),
            width = self.max_width
        );

        if self.icon == ' ' {
            format!(" {} ", main_text)
        } else {
            format!(" {} {} ", self.icon, main_text)
        }
    }
}

impl SeperatorWidth for AnimatedLabel {}
impl Seperator for AnimatedLabel {}