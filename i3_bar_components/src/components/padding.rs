use super::{prelude::*, BaseComponent};
use crate::{component_manager::ManageComponents, property::Properties, protocol::ClickEvent};

pub struct Padding {
    base_component: BaseComponent,
}

impl Padding {
    pub fn new(width: usize) -> Self {
        Self {
            base_component: BaseComponent::from(Properties {
                text: crate::property::Text {
                    full: " ".repeat(width),
                    ..Default::default()
                },
                ..Default::default()
            }),
        }
    }

    pub fn set_width(&mut self, width: usize) {
        self.base_component.get_properties_mut().text.full = " ".repeat(width);
    }
}

impl Component for Padding {
    fn update(&mut self, _: f64) {}
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

impl Widget for Padding {
    fn get_base_component(&self) -> &BaseComponent {
        &self.base_component
    }

    fn get_base_component_mut(&mut self) -> &mut BaseComponent {
        &mut self.base_component
    }
}
