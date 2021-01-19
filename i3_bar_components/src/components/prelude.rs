use crate::{ComponentManagerMessenger, protocol::{Block, ClickEvent}};

pub trait Component: std::any::Any {

    fn update(&mut self);
    fn event(&mut self, event: &ClickEvent);
    fn collect_blocks<'a>(&'a self, blocks: &mut Vec<&'a Block>);
    fn name(&self) -> &str;
    fn add_component_manager_messenger(&mut self, component_manager_messanger: ComponentManagerMessenger);
    fn get_id(&self) -> &str;

}

pub trait Widget: Component {
    fn get_block(&self) -> &Block;
    fn get_block_mut(&mut self) -> &mut Block;
}

pub trait Seperator: Widget {
    fn set_seperator(&mut self, s: bool) {
        self.get_block_mut().set_separator(s)
    }
}

pub trait SeperatorWidth: Widget {
    fn set_separator_block_width(&mut self, sbw: usize) {
        self.get_block_mut().set_separator_block_width(sbw);
    }
}