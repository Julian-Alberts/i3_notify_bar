use crate::{ComponentManagerMessenger, protocol::{Block, ClickEvent}};

use super::{BaseComponent, prelude::*};

pub struct Button {
    base_component: BaseComponent,
    component_manager: Option<ComponentManagerMessenger>
}

impl Button {

    pub fn new(text: String) -> Button {

        let block = Block::new()
            .with_border(String::from("#FFFFFF"))
            .with_full_text(text);
        Button {
            base_component: BaseComponent::from(block),
            component_manager: None
        }
    }

}

impl Component for Button {

    fn update(&mut self, _: f64) {}
    fn event(&mut self, _: &ClickEvent) {
        self.base_component.set_background(String::from("#00FF00"));
    }

    fn collect_base_components<'a>(&'a self, base_components: &mut Vec<&'a BaseComponent>) {
        base_components.push(&self.base_component)
    }

    fn collect_base_components_mut<'a>(&'a mut self, base_components: &mut Vec<&'a mut BaseComponent>) {
        base_components.push(&mut self.base_component);
    }

    fn name(&self) -> &str {
        match self.base_component.get_name() {
            Some(name) => name,
            None => ""
        }
    }

    fn add_component_manager_messenger(&mut self, component_manager_messanger: ComponentManagerMessenger) {
        self.component_manager = Some(component_manager_messanger);
    }

    fn get_id(&self) -> &str {
        self.base_component.get_id()
    }
}

impl Widget for Button {

    fn get_base_component(&self) -> &BaseComponent {
        &self.base_component
    }

    fn get_base_component_mut(&mut self) -> &mut BaseComponent {
        &mut self.base_component
    }

}

impl Seperator for Button {}
impl SeperatorWidth for Button {}