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

impl SimpleComponent for Label {
    fn properties_mut(&mut self) -> &mut crate::property::Properties {
        self.base_component.properties_mut()
    }
    fn properties(&self) -> &crate::property::Properties {
        self.base_component.properties()
    }
}

impl Component for Label {
    fn update(&mut self, dt: f64) {
        self.text.update(dt);
        self.set_full(self.text.to_component_text())
    }
    fn event(&mut self, _: &mut dyn ManageComponents, _: &ClickEvent) {}
    fn all_properties<'a>(&'a self) -> Box<dyn Iterator<Item = &Properties> + 'a> {
        Box::new([self.properties()].into_iter())
    }
}
