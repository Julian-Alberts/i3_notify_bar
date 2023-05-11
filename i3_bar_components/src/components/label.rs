use super::{prelude::*, BaseComponent};
use crate::{
    component_manager::ManageComponents, property::Properties, protocol::ClickEvent,
    string::ComponentString,
};

pub struct Label {
    base_component: BaseComponent,
    text: Box<dyn ComponentString>,
}

impl Label {
    pub fn new(text: Box<dyn ComponentString>) -> Self {
        Self {
            base_component: BaseComponent::from(Properties::default()),
            text,
        }
    }

    pub fn set_text(&mut self, s: Box<dyn ComponentString>) {
        self.text = s;
    }
}

impl Component for Label {
    fn update(&mut self, dt: f64) {
        self.text.update(dt);
        self.base_component.get_properties_mut().text.full = self.text.to_component_text()
    }
    fn event(&mut self, _: &mut dyn ManageComponents, _: &ClickEvent) {}

    fn collect_base_components<'a>(&'a self, base_components: &mut Vec<&'a BaseComponent>) {
        base_components.push(&self.base_component)
    }

    fn collect_base_components_mut<'a>(
        &'a mut self,
        base_components: &mut Vec<&'a mut BaseComponent>,
    ) {
        base_components.push(&mut self.base_component)
    }

    fn name(&self) -> Option<&str> {
        self.base_component.get_name()
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
