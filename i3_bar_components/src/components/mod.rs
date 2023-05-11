mod button;
mod button_group;
mod label;
mod padding;
pub mod prelude;
mod progress_bar;

pub use button::Button;
pub use button_group::{ButtonGroup, GroupButton};
pub use label::Label;
use log::debug;
pub use padding::Padding;
pub use progress_bar::ProgressBar;

use crate::property::Properties;

#[derive(Debug, PartialEq, Default)]
pub struct BaseComponent {
    properties: Properties,
    serialized: Option<Vec<u8>>,
}

impl BaseComponent {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn serialize_cache(&mut self) -> &[u8] {
        let properties = &self.properties;
        self.serialized.get_or_insert_with(|| {
            match serde_json::to_string(properties) {
                Ok(b) => b,
                Err(_) => {
                    debug!("Could not serialize block {:#?}", properties);
                    todo!("return error")
                }
            }
            .as_bytes()
            .to_vec()
        })
    }

    /// Returns a mutable reference to the properties
    /// Try to limit calls to this method. Calling this method marks the block as dirty and forces serialization even if no value has been changed.
    pub fn get_properties_mut(&mut self) -> &mut Properties {
        self.serialized = None;
        &mut self.properties
    }

    pub fn get_properties(&self) -> &Properties {
        &self.properties
    }

    pub fn get_name(&self) -> Option<&str> {
        self.properties.name.as_ref().map(|s| &s[..])
    }

    pub fn get_id(&self) -> u32 {
        self.properties.instance.unwrap()
    }
}

impl BaseComponent {
    #[deprecated]
    pub fn set_full_text(&mut self, full_text: String) {
        self.get_properties_mut().text.full = full_text;
    }

    #[deprecated]
    pub fn set_separator(&mut self, s: bool) {
        self.get_properties_mut().separator.show = s;
    }

    #[deprecated]
    pub fn set_separator_block_width(&mut self, sbw: usize) {
        self.get_properties_mut().separator.block_width = Some(sbw);
    }

    #[deprecated]
    pub fn set_background(&mut self, color: String) {
        self.get_properties_mut().color.background = Some(color);
    }

    #[deprecated]
    pub fn set_color(&mut self, color: String) {
        self.get_properties_mut().color.text = Some(color)
    }

    #[deprecated]
    pub fn set_urgent(&mut self, urgent: bool) {
        self.get_properties_mut().urgent = urgent
    }
}

impl From<Properties> for BaseComponent {
    fn from(block: Properties) -> Self {
        Self {
            properties: block,
            serialized: None,
        }
    }
}
