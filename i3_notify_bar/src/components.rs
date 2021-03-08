use i3_bar_components::{ComponentManagerMessenger, components::{BaseComponent, Button, Label, ProgressBar, prelude::*}, protocol::ClickEvent};

use crate::notification_bar::NotificationData;
use crate::icons;

pub struct NotificationComponent {
    close_type: CloseType,
    label: Label,
    padding_r: Label,
    id: String,
    component_manager: Option<ComponentManagerMessenger>,
    text: AnimatedText,
    icon: char
}

impl NotificationComponent {

    pub fn new(nd: &NotificationData) -> NotificationComponent {
        let close_type = match nd.expire_timeout {
            -1 => {
                let mut b = Button::new(format!(" {} ", icons::X_ICON));
                b.set_seperator(false);
                b.set_separator_block_width(0);
                nd.style.iter().for_each(|s| {
                    s.apply(b.get_base_component_mut());
                });
                CloseType::Button(b)
            },
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

        let text = AnimatedText {
            max_with: 20,
            move_chars_per_sec: 5,
            start_offset: 0.0,
            text: nd.text.clone(),
            stop_animation_for_secs: 0.0
        };

        let mut label;
        if nd.icon == ' ' {
            label = Label::new(format!(" {} ", text.to_string()));
        } else {
            label = Label::new(format!(" {} {} ",nd.icon, text.to_string()));
        }
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
            id: format!("{}", nd.id),
            component_manager: None,
            padding_r,
            icon: nd.icon,
            text
        }
    }

    pub fn update_notification(&mut self, nd: &NotificationData) {
        self.icon = nd.icon;
        self.text.text = nd.text.to_string();

        self.update_label_text();

        let close_type = match nd.expire_timeout {
            -1 => {
                let mut b = Button::new(String::from(" X "));
                b.set_seperator(false);
                b.set_separator_block_width(0);
                nd.style.iter().for_each(|s| {
                    s.apply(b.get_base_component_mut());
                });
                CloseType::Button(b)
            },
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

    fn update_label_text(&mut self) {
        if self.icon == ' ' {
            self.label.set_text(format!(" {} ", self.text.to_string()));
        } else {
            self.label.set_text(format!(" {} {} ",self.icon, self.text.to_string()));
        }
    }

}

impl Component for NotificationComponent {

    fn collect_base_components<'a>(&'a self, base_components: &mut Vec<&'a BaseComponent>) {
        self.label.collect_base_components(base_components);
        self.close_type.collect_base_components(base_components);
        self.padding_r.collect_base_components(base_components);
    }

    fn collect_base_components_mut<'a>(&'a mut self, base_components: &mut Vec<&'a mut BaseComponent>) {
        self.label.collect_base_components_mut(base_components);
        self.close_type.collect_base_components_mut(base_components);
        self.padding_r.collect_base_components_mut(base_components);
    }

    fn event(&mut self, ce: &ClickEvent) {
        if self.close_type.is_button() && ce.get_id() == self.close_type.get_id() {
            let cm = self.component_manager.as_ref().unwrap();
            cm.remove();
        }
    }

    fn update(&mut self, dt: f64) {
        let cm = match &self.component_manager {
            Some(cm) => cm,
            None => panic!("ComponentManagerMassenger not set")
        };

        if self.close_type.is_timer() && self.close_type.is_finished() {
            cm.remove();
        }

        self.text.update(dt);
        self.update_label_text();
        self.label.update(dt);
        self.close_type.update(dt);
    }

    fn name(&self) -> &str {
        &self.id
    }

    fn add_component_manager_messenger(&mut self, component_manager_messanger: ComponentManagerMessenger) {
        self.component_manager = Some(component_manager_messanger);
    }

    fn get_id(&self) -> &str {
        &self.id
    }

}

enum CloseType {
    Button(Button),
    Timer(ProgressBar)
}

impl CloseType {

    fn is_button(&self) -> bool {
        match self {
            Self::Button(_) => true,
            _ => false
        }
    }

    fn is_timer(&self) -> bool {
        match self {
            Self::Timer(_) => true,
            _ => false
        }
    }

    fn is_finished(&self) -> bool {
        match self {
            Self::Timer(t) => t.is_finished(),
            _ => false
        }
    }

}

impl Component for CloseType {

    fn update(&mut self, dt: f64) {
        match self {
            Self::Button(b) => b.update(dt),
            Self::Timer(t) => t.update(dt)
        }
    }

    fn add_component_manager_messenger(&mut self, component_manager_messanger: ComponentManagerMessenger) {
        match self {
            Self::Button(b) => b.add_component_manager_messenger(component_manager_messanger),
            Self::Timer(t) => t.add_component_manager_messenger(component_manager_messanger)
        }
    }

    fn collect_base_components<'a>(&'a self, base_components: &mut Vec<&'a BaseComponent>) {
        match self {
            Self::Button(b) => b.collect_base_components(base_components),
            Self::Timer(t) => t.collect_base_components(base_components)
        }
    }

    fn collect_base_components_mut<'a>(&'a mut self, base_components: &mut Vec<&'a mut BaseComponent>) {
        match self {
            Self::Button(b) => b.collect_base_components_mut(base_components),
            Self::Timer(t) => t.collect_base_components_mut(base_components)
        }
    }

    fn event(&mut self, event: &ClickEvent) {
        match self {
            Self::Button(b) => b.event(event),
            Self::Timer(t) => t.event(event)
        }
    }

    fn get_id(&self) -> &str {
        match self {
            Self::Button(b) => b.get_id(),
            Self::Timer(t) => t.get_id()
        }
    }

    fn name(&self) -> &str {
        match self {
            Self::Button(b) => b.name(),
            Self::Timer(t) => t.name()
        }
    }

}

impl Widget for CloseType {

    fn get_base_component(&self) -> &BaseComponent {
        match self {
            Self::Button(b) => b.get_base_component(),
            Self::Timer(t) => t.get_base_component()
        }
    }

    fn get_base_component_mut(&mut self) -> &mut BaseComponent {
        match self {
            Self::Button(b) => b.get_base_component_mut(),
            Self::Timer(t) => t.get_base_component_mut()
        }
    }

}

struct AnimatedText {
    start_offset: f64,
    max_with: usize,
    move_chars_per_sec: usize,
    text: String,
    stop_animation_for_secs: f64
}

impl AnimatedText {

    fn update(&mut self, dt: f64) {
        let text_len = self.text.chars().count();

        if text_len <= self.max_with {
            return
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

impl ToString for AnimatedText {

    fn to_string(&self) -> String {

        let text_len = self.text.chars().count();

        if text_len <= self.max_with {
            return self.text.to_owned()
        }
        let end;
        if self.start_offset as usize + self.max_with < text_len {
            end = self.start_offset as usize + self.max_with;
        } else {
            end = text_len;
        }

        let chars = self.text.chars().collect::<Vec<char>>();
        let chars = &chars[self.start_offset as usize..end];

        format!("{text: <width$}", text=chars.into_iter().collect::<String>(), width=self.max_with)
    }

}