use crate::{ComponentManagerMessenger, protocol::{Block, ClickEvent}};
use super::{BaseComponent, prelude::*};

pub struct Label {
    base_component: BaseComponent,
    component_manager: Option<ComponentManagerMessenger>
}

impl Label {

    pub fn new(text: String) -> Self {
        let block = Block::new().with_full_text(text);
        Self {
            base_component: BaseComponent::from(block),
            component_manager: None
        }
    }

    pub fn set_text(&mut self, s: String) {
        self.base_component.set_full_text(s);
    }

}

impl Component for Label {

    fn update(&mut self) {}
    fn event(&mut self, _: &ClickEvent) {}

    fn collect_base_components<'a>(&'a self, base_components: &mut Vec<&'a BaseComponent>) {
        base_components.push(&self.base_component)
    }

    fn collect_base_components_mut<'a>(&'a mut self, base_components: &mut Vec<&'a mut BaseComponent>) {
        base_components.push(&mut self.base_component)
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