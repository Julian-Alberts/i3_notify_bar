use i3_bar_components::{ComponentManagerMessenger, components::{Button, Label, ProgressBar, prelude::*}, protocol::{Block, ClickEvent}};
use notify_server::notification::Notification;

use crate::rule::rule::Style;

pub struct NotificationComponent {
    close_type: CloseType,
    label: Label,
    padding_r: Label,
    id: String,
    component_manager: Option<ComponentManagerMessenger>
}

impl NotificationComponent {

    pub fn new(e: &Notification, styles: &Vec<Style>) -> NotificationComponent {
        let close_type = match e.expire_timeout {
            -1 => {
                let mut b = Button::new(String::from("X"));
                b.set_seperator(false);
                b.set_separator_block_width(0);
                styles.iter().for_each(|s| {
                    s.apply(b.get_block_mut());
                });
                CloseType::Button(b)
            },
            _ => {
                let mut t = ProgressBar::new(e.expire_timeout as u64);
                t.set_seperator(false);
                t.set_separator_block_width(0);
                styles.iter().for_each(|s| {
                    s.apply(t.get_block_mut());
                });
                CloseType::Timer(t)
            }
        };

        let mut label = Label::new(format!(" {}{} ", e.app_icon, e.summary.clone()));
        label.set_seperator(false);
        label.set_separator_block_width(0);

        let mut padding_r = Label::new(String::from(" "));

        styles.iter().for_each(|s| {
            s.apply(label.get_block_mut());
            s.apply(padding_r.get_block_mut());
        });
        
        
        Self {
            close_type,
            label,
            id: format!("{}", e.id),
            component_manager: None,
            padding_r
        }
    }

    pub fn update_notification(&mut self, n: &Notification, styles: &Vec<Style>) {
        self.label.set_text(format!(" {}{} ", n.app_icon, n.summary));
        let close_type = match n.expire_timeout {
            -1 => {
                let mut b = Button::new(String::from(" X "));
                b.set_seperator(false);
                b.set_separator_block_width(0);
                styles.iter().for_each(|s| {
                    s.apply(b.get_block_mut());
                });
                CloseType::Button(b)
            },
            _ => {
                let mut t = ProgressBar::new(n.expire_timeout as u64);
                t.set_seperator(false);
                t.set_separator_block_width(0);
                styles.iter().for_each(|s| {
                    s.apply(t.get_block_mut());
                });
                CloseType::Timer(t)
            }
        };
        self.close_type = close_type;
    }

}

impl Component for NotificationComponent {

    fn collect_blocks<'a>(&'a self, blocks: &mut Vec<&'a Block>) {
        self.label.collect_blocks(blocks);
        self.close_type.collect_blocks(blocks);
        self.padding_r.collect_blocks(blocks);
    }

    fn event(&mut self, ce: &ClickEvent) {
        if self.close_type.is_button() && ce.get_id() == self.close_type.get_id() {
            let cm = self.component_manager.as_ref().unwrap();
            cm.remove();
        }
    }

    fn update(&mut self) {
        let cm = match &self.component_manager {
            Some(cm) => cm,
            None => panic!("ComponentManagerMassenger not set")
        };

        if self.close_type.is_timer() && self.close_type.is_finished() {
            cm.remove();
        }

        self.label.update();
        self.close_type.update();
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

    fn update(&mut self) {
        match self {
            Self::Button(b) => b.update(),
            Self::Timer(t) => t.update()
        }
    }

    fn add_component_manager_messenger(&mut self, component_manager_messanger: ComponentManagerMessenger) {
        match self {
            Self::Button(b) => b.add_component_manager_messenger(component_manager_messanger),
            Self::Timer(t) => t.add_component_manager_messenger(component_manager_messanger)
        }
    }

    fn collect_blocks<'a>(&'a self, blocks: &mut Vec<&'a Block>) {
        match self {
            Self::Button(b) => b.collect_blocks(blocks),
            Self::Timer(t) => t.collect_blocks(blocks)
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

    fn get_block(&self) -> &Block {
        match self {
            Self::Button(b) => b.get_block(),
            Self::Timer(t) => t.get_block()
        }
    }

    fn get_block_mut(&mut self) -> &mut Block {
        match self {
            Self::Button(b) => b.get_block_mut(),
            Self::Timer(t) => t.get_block_mut()
        }
    }

}