mod button;
mod label;
pub mod prelude;
mod progress_bar;
pub use button::Button;
pub use label::Label;
use log::debug;
pub use progress_bar::ProgressBar;

use crate::protocol::Block;

macro_rules! block_pass_through {
    ($fn_name: ident ($var_name:ident: $type:ty)) => {
        pub fn $fn_name(&mut self, $var_name: $type) {
            self.get_block_mut().$fn_name($var_name);
        }
    };
}
#[derive(Debug, PartialEq)]
pub struct BaseComponent {
    block: Block,
    is_dirty: bool,
    serialized: Vec<u8>,
}

impl Default for BaseComponent {
    fn default() -> Self {
        Self {
            block: Block::new(),
            is_dirty: true,
            serialized: Vec::new(),
        }
    }
}

impl BaseComponent {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn serialize_cache(&mut self) -> &[u8] {
        if self.is_dirty {
            self.serialized = match serde_json::to_string(&self.block) {
                Ok(b) => b,
                Err(_) => {
                    debug!("Could not serialize block {:#?}", self.block);
                    todo!("return error")
                }
            }
                .as_bytes()
                .to_vec();
            self.is_dirty = false;
        }

        &self.serialized
    }

    /// Returns a mutable reference to the inner block
    /// Try to limit calls to this method. Calling this method marks the block as dirty and forces serialization even if no value has been changed.
    pub fn get_block_mut(&mut self) -> &mut Block {
        self.is_dirty = true;
        &mut self.block
    }

    pub fn get_name(&self) -> &Option<String> {
        self.block.name()
    }

    pub fn get_id(&self) -> &str {
        self.block.get_id()
    }
}

impl BaseComponent {
    block_pass_through!(set_full_text(full_text: String));
    block_pass_through!(set_separator(s: bool));
    block_pass_through!(set_separator_block_width(sbw: usize));
    block_pass_through!(set_background(color: String));
    block_pass_through!(set_color(color: String));
    block_pass_through!(set_urgent(urgent: bool));
}

impl From<Block> for BaseComponent {
    fn from(block: Block) -> Self {
        Self {
            block,
            is_dirty: true,
            serialized: Vec::new(),
        }
    }
}
