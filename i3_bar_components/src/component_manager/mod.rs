mod component_manager_messenger;
mod component_manager;

pub use component_manager::ComponentManagerBuilder;
pub use component_manager::ComponentManager;
pub use component_manager_messenger::ComponentManagerMassenger;

use crate::components::prelude::Component;

pub trait ManageComponents {
    fn add_component(&mut self, comp: Box<dyn Component>);
    fn remove_by_name(&mut self, name: &str);
    fn new_layer(&mut self);
    fn pop_layer(&mut self);
}