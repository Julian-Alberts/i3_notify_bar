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

impl Component for ProgressBar {
    fn collect_base_components<'a>(&'a self, base_components: &mut Vec<&'a BaseComponent>) {
        base_components.push(&self.base_component)
    }

    fn collect_base_components_mut<'a>(
        &'a mut self,
        base_components: &mut Vec<&'a mut BaseComponent>,
    ) {
        base_components.push(&mut self.base_component);
    }

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

        self.base_component.get_properties_mut().text.full = icon.to_string()
    }

    fn name(&self) -> &str {
        match self.base_component.get_name() {
            Some(name) => name,
            None => "",
        }
    }

    fn get_id(&self) -> &str {
        self.get_base_component().get_id()
    }
}

impl Widget for ProgressBar {
    fn get_base_component(&self) -> &BaseComponent {
        &self.base_component
    }

    fn get_base_component_mut(&mut self) -> &mut BaseComponent {
        &mut self.base_component
    }
}

impl Seperator for ProgressBar {}

impl SeperatorWidth for ProgressBar {}
