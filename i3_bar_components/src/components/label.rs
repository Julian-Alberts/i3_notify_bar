use super::{prelude::*, BaseComponent};
use crate::{
    property::{Properties, Text},
    protocol::ClickEvent,
};

pub struct Label {
    base_component: BaseComponent,
}

impl Label {
    pub fn new(text: String) -> Self {
        Self {
            base_component: BaseComponent::from(Properties {
                text: Text {
                    full: text,
                    ..Default::default()
                },
                ..Default::default()
            }),
        }
    }

    pub fn set_text(&mut self, s: String) {
        self.base_component.get_properties_mut().text.full = s;
    }
}

impl Component for Label {
    fn update(&mut self, _: f64) {}
    fn event(&mut self, _: &ClickEvent) {}

    fn collect_base_components<'a>(&'a self, base_components: &mut Vec<&'a BaseComponent>) {
        base_components.push(&self.base_component)
    }

    fn collect_base_components_mut<'a>(
        &'a mut self,
        base_components: &mut Vec<&'a mut BaseComponent>,
    ) {
        base_components.push(&mut self.base_component)
    }

    fn name(&self) -> &str {
        match self.base_component.get_name() {
            Some(name) => name,
            None => "",
        }
    }

    fn get_id(&self) -> &str {
        self.base_component.get_id()
    }
}

impl Widget for Label {
    fn get_base_component(&self) -> &BaseComponent {
        &self.base_component
    }

    fn get_base_component_mut(&mut self) -> &mut BaseComponent {
        &mut self.base_component
    }
}

impl Seperator for Label {}

impl SeperatorWidth for Label {}
