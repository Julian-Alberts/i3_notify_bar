use crate::{ComponentManagerMessenger, protocol::{Block, ClickEvent}};
use super::prelude::*;

pub struct Label {
    block: Block,
    component_manager: Option<ComponentManagerMessenger>
}

impl Label {

    pub fn new(text: String) -> Self {
        let block = Block::new().with_full_text(text);
        Self {
            block,
            component_manager: None
        }
    }

    pub fn set_text(&mut self, s: String) {
        self.block.set_full_text(s)
    }

}

impl Component for Label {

    fn update(&mut self) {}
    fn event(&mut self, _: &ClickEvent) {}

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

impl Widget for Label {

    fn get_block(&self) -> &Block {
        &self.block
    }

    fn get_block_mut(&mut self) -> &mut Block {
        &mut self.block
    }

}

impl Seperator for Label {}

impl SeperatorWidth for Label {}