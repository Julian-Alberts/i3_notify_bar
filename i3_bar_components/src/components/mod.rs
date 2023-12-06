mod button;
mod button_group;
mod label;
pub mod prelude;
mod progress_bar;

pub use button::Button;
pub use button_group::{ButtonGroup, GroupButton};
pub use label::Label;
pub use progress_bar::ProgressBar;

use crate::property::Properties;

use self::prelude::{Component, SimpleComponent};

#[derive(Debug, Default)]
pub struct BaseComponent {
    properties: Properties,
}

impl BaseComponent {
    pub fn new() -> Self {
        Self::default()
    }
}

impl SimpleComponent for BaseComponent {
    fn properties(&self) -> &crate::property::Properties {
        &self.properties
    }
    fn properties_mut(&mut self) -> &mut crate::property::Properties {
        &mut self.properties
    }
}
impl Component for BaseComponent {
    fn update(&mut self, _dt: f64) {}
    fn event(
        &mut self,
        _cm: &mut dyn crate::ManageComponents,
        _event: &crate::protocol::ClickEvent,
    ) {
    }
    fn all_properties<'a>(&'a self) -> Box<dyn Iterator<Item = &Properties> + 'a> {
        Box::new([self.properties()].into_iter())
    }
    fn name(&self) -> Option<&str> {
        self.properties.name.as_deref()
    }
}

impl From<Properties> for BaseComponent {
    fn from(block: Properties) -> Self {
        Self { properties: block }
    }
}
