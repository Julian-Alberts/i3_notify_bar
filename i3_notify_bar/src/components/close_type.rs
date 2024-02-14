use i3_bar_components::components::prelude::EventTarget;
use i3_bar_components::components::{prelude::Component, Button, ProgressBar};

pub struct CloseType {
    button: Button,
    timer: Option<ProgressBar>,
}

impl Component for CloseType {
    fn update(&mut self, dt: f64) {
        self.button.update(dt);
        if let Some(t) = self.timer.as_mut() {
            t.update(dt);
        }
    }

    fn all_properties<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = &i3_bar_components::property::Properties> + 'a> {
        Box::new(
            [
                self.timer.as_ref().map(Component::all_properties),
                Some(self.button.all_properties()),
            ]
            .into_iter()
            .flatten()
            .flatten(),
        )
    }

    fn event_targets<'a>(
        &'a self,
    ) -> Box<
        dyn Iterator<
                Item = (
                    i3_bar_components::property::Instance,
                    *const dyn EventTarget,
                ),
            > + 'a,
    > {
        self.button.event_targets()
    }
}
