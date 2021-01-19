use std::time::SystemTime;

use crate::{ComponentManagerMessenger, protocol::{Block, ClickEvent}};

use super::prelude::*;

pub struct ProgressBar {
    block: Block,
    current: SystemTime,
    max: u64,
    component_manager: Option<ComponentManagerMessenger>
}

impl ProgressBar {

    pub fn new(max: u64) -> Self {
        Self {
            block: Block::new(),
            current: SystemTime::now(),
            max,
            component_manager: None
        }
    }

    pub fn is_finished(&self) -> bool {
        self.current.elapsed().unwrap().as_secs() >= self.max
    }

}

impl Component for ProgressBar {

    fn collect_blocks<'a>(&'a self, blocks: &mut Vec<&'a Block>) {
        blocks.push(&self.block);
    }

    fn event(&mut self, _: &ClickEvent) {}

    fn update(&mut self) {
        let step = (self.current.elapsed().unwrap().as_secs_f64() / self.max as f64 * 8_f64).floor() as u8;

        let icon = match step {
            0 => '\u{2588}',
            1 => '\u{2587}',
            2 => '\u{2586}',
            3 => '\u{2585}',
            4 => '\u{2584}',
            5 => '\u{2583}',
            6 => '\u{2582}',
            7 => '\u{2581}',
            _ => ' '
        };

        self.block.set_full_text([icon].iter().collect::<String>())
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
        self.get_block().get_id()
    }

}

impl Widget for ProgressBar {

    fn get_block(&self) -> &Block {
        &self.block
    }

    fn get_block_mut(&mut self) -> &mut Block {
        &mut self.block
    }

}

impl Seperator for ProgressBar {}

impl SeperatorWidth for ProgressBar {}