use std::time::SystemTime;

use crate::{protocol::ClickEvent, ComponentManagerMessenger};

use super::{prelude::*, BaseComponent};

pub struct ProgressBar {
    base_component: BaseComponent,
    current: SystemTime,
    max: u64,
    component_manager: Option<ComponentManagerMessenger>,
}

impl ProgressBar {
    pub fn new(max: u64) -> Self {
        Self {
            base_component: BaseComponent::new(),
            current: SystemTime::now(),
            max,
            component_manager: None,
        }
    }

    pub fn is_finished(&self) -> bool {
        self.current.elapsed().unwrap().as_secs() >= self.max
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

    fn event(&mut self, _: &ClickEvent) {}

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

        self.base_component
            .set_full_text([icon].iter().collect::<String>())
    }

    fn name(&self) -> &str {
        match self.base_component.get_name() {
            Some(name) => name,
            None => "",
        }
    }

    fn add_component_manager_messenger(
        &mut self,
        component_manager_messanger: ComponentManagerMessenger,
    ) {
        self.component_manager = Some(component_manager_messanger);
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
