use i3_bar_components::component_manager::ManageComponents;
use i3_bar_components::{
    components::{
        prelude::{Component, Widget},
        BaseComponent, Button, ProgressBar,
    },
    protocol::ClickEvent,
};

pub enum CloseType {
    Button(Button),
    Timer(ProgressBar),
}

impl CloseType {
    pub fn is_button(&self) -> bool {
        matches!(self, Self::Button(_))
    }

    pub fn is_timer(&self) -> bool {
        matches!(self, Self::Timer(_))
    }

    pub fn is_finished(&self) -> bool {
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

    fn event(&mut self, mc: &mut dyn ManageComponents, event: &ClickEvent) {
        match self {
            Self::Button(b) => b.event(mc, event),
            Self::Timer(t) => t.event(mc, event),
        }
    }

    fn name(&self) -> Option<&str> {
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
