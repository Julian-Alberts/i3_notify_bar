use crate::{protocol::ClickEvent, ComponentManagerMessenger};

use super::BaseComponent;

pub trait Component: std::any::Any {
    fn update(&mut self, dt: f64);
    fn event(&mut self, event: &ClickEvent);
    fn collect_base_components<'a>(&'a self, base_components: &mut Vec<&'a BaseComponent>);
    fn collect_base_components_mut<'a>(
        &'a mut self,
        base_components: &mut Vec<&'a mut BaseComponent>,
    );
    fn name(&self) -> &str;
    fn add_component_manager_messenger(
        &mut self,
        component_manager_messanger: ComponentManagerMessenger,
    );
    fn get_id(&self) -> &str;
}

pub trait Widget: Component {
    fn get_base_component(&self) -> &BaseComponent;
    fn get_base_component_mut(&mut self) -> &mut BaseComponent;
}

pub trait Seperator: Widget {
    fn set_seperator(&mut self, s: bool) {
        self.get_base_component_mut().set_separator(s)
    }
}

pub trait SeperatorWidth: Widget {
    fn set_separator_block_width(&mut self, sbw: usize) {
        self.get_base_component_mut().set_separator_block_width(sbw);
    }
}
