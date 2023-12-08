use super::{prelude::*, BaseComponent};
use crate::{
    component_manager::ManageComponents, property::Properties, protocol::ClickEvent,
    string::ComponentString,
};

pub struct Label<Text: ComponentString = Box<dyn ComponentString>> {
    base_component: BaseComponent,
    text: Text,
}

impl Label {
    pub fn new<Text: ComponentString>(text: Text) -> Label<Text> {
        Label::<Text> {
            base_component: BaseComponent::from(Properties::default()),
            text,
        }
    }
}

impl<Text: ComponentString> Label<Text> {
    pub fn set_text(&mut self, s: Text) {
        self.text = s;
    }

    pub fn text(&self) -> &Text {
        &self.text
    }

    pub fn text_mut(&mut self) -> &mut Text {
        &mut self.text
    }
}

impl<Text: ComponentString> SimpleComponent for Label<Text> {
    fn properties_mut(&mut self) -> &mut crate::property::Properties {
        self.base_component.properties_mut()
    }
    fn properties(&self) -> &crate::property::Properties {
        self.base_component.properties()
    }
}

impl<Text: ComponentString> Component for Label<Text> {
    fn update(&mut self, dt: f64) {
        self.text.update(dt);
        self.set_full(self.text.to_component_text())
    }
    fn event(&mut self, _: &mut dyn ManageComponents, _: &ClickEvent) {}
    fn all_properties<'a>(&'a self) -> Box<dyn Iterator<Item = &Properties> + 'a> {
        Box::new([self.properties()].into_iter())
    }
}
