use std::time::SystemTime;

use crate::{component_manager::ManageComponents, protocol::ClickEvent};

use super::{prelude::*, BaseComponent};

pub struct ProgressBar {
    base_component: BaseComponent,
    current: SystemTime,
    max: u64,
}

impl ProgressBar {
    pub fn new(max: u64) -> Self {
        Self {
            base_component: BaseComponent::new(),
            current: SystemTime::now(),
            max,
        }
    }

    pub fn is_finished(&self) -> bool {
        match self.current.elapsed() {
            Ok(e) => e,
            Err(_) => {
                log::error!("I will mess with time!");
                return false;
            }
        }
        .as_secs()
            >= self.max
    }
}

impl SimpleComponent for ProgressBar {
    fn properties(&self) -> &crate::property::Properties {
        self.base_component.properties()
    }
    fn properties_mut(&mut self) -> &mut crate::property::Properties {
        self.base_component.properties_mut()
    }
}

impl Component for ProgressBar {
    fn event(&mut self, _: &mut dyn ManageComponents, _: &ClickEvent) {}

    fn update(&mut self, _: f64) {
        let step =
            (self.current.elapsed().unwrap().as_secs_f64() / self.max as f64 * 8_f64).floor() as u8;

        let icon = match step {
            0 => '\u{2588}',
            1 => '\u{2587}',
            2 => '\u{2586}',
            3 => '\u{2585}',
            4 => '\u{2584}',
            5 => '\u{2583}',
            6 => '\u{2582}',
            7 => '\u{2581}',
            _ => ' ',
        };

        self.set_full(icon.to_string())
    }

    fn all_properties<'a>(&'a self) -> Box<dyn Iterator<Item = &crate::property::Properties> + 'a> {
        Box::new([self.properties()].into_iter())
    }
}
