use crate::{ComponentManagerMessenger, protocol::{Block, ClickEvent}};

use super::prelude::*;

pub struct Button {
    block: Block,
    component_manager: Option<ComponentManagerMessenger>
}

impl Button {

    pub fn new(text: String) -> Button {

        let block = Block::new()
            .with_border(String::from("#FFFFFF"))
            .with_full_text(text);
        Button {
            block,
            component_manager: None
        }
    }

}

impl Component for Button {

    fn update(&mut self) {}
    fn event(&mut self, _: &ClickEvent) {
        self.block.set_background(String::from("#00FF00"));
    }

    fn collect_blocks<'a>(&'a self, blocks: &mut Vec<&'a Block>) {
        blocks.push(&self.block);    
    }

    fn name(&self) -> &str {
        match self.block.name() {
            Some(name) => name,
            None => ""
        }
    }

    fn add_component_manager_messenger(&mut self, component_manager_messanger: ComponentManagerMessenger) {
        self.component_manager = Some(component_manager_messanger);
    }

    fn get_id(&self) -> &str {
        self.block.get_id()
    }
}

impl Widget for Button {

    fn get_block(&self) -> &Block {
        &self.block
    }

    fn get_block_mut(&mut self) -> &mut Block {
        &mut self.block
    }

}

impl Seperator for Button {}
impl SeperatorWidth for Button {}