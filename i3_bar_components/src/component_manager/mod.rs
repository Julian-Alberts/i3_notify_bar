mod component_manager;
mod component_manager_messenger;

pub use component_manager::ComponentManager;
pub use component_manager::ComponentManagerBuilder;
pub use component_manager_messenger::ComponentManagerMassenger;

pub use self::component_manager::AnyComponent;

pub trait ManageComponents {
    fn add_component(&mut self, comp: Box<dyn AnyComponent>);
    fn add_component_at(&mut self, comp: Box<dyn AnyComponent>, pos: isize);
    fn add_component_at_on_layer(&mut self, comp: Box<dyn AnyComponent>, pos: isize, layer: usize);
    fn new_layer(&mut self);
    fn pop_layer(&mut self);
    fn remove_by_name(&mut self, name: &str);
}
