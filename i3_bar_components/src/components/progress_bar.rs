use super::{prelude::*, BaseComponent};

pub struct ProgressBar {
    base_component: BaseComponent,
    current: f64,
    max: f64,
}

impl ProgressBar {
    pub fn new(max: f64) -> Self {
        Self {
            base_component: BaseComponent::new(),
            current: 0.0,
            max,
        }
    }
    pub fn set_current(&mut self, current: f64) {
        self.current = current;
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
    fn update(&mut self, _: f64) {
        let step = (self.current / self.max * 8_f64).floor() as u8;

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

    fn event_targets<'a>(
        &'a self,
    ) -> Box<
        (dyn Iterator<
            Item = (
                crate::property::Instance,
                *const (dyn crate::components::prelude::EventTarget + 'static),
            ),
        > + 'a),
    > {
        Box::new(std::iter::empty())
    }
}
