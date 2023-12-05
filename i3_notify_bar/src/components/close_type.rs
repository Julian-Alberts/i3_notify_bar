use i3_bar_components::component_manager::ManageComponents;
use i3_bar_components::{
    components::{prelude::Component, Button, ProgressBar},
    protocol::ClickEvent,
};

pub struct CloseType {
    button: Button,
    timer: Option<ProgressBar>,
}

impl Component for CloseType {
    fn update(&mut self, dt: f64) {
        self.button.update(dt);
        self.timer.as_mut().map(|t| t.update(dt));
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
            .filter_map(|a| a)
            .flatten(),
        )
    }

    fn event(&mut self, mc: &mut dyn ManageComponents, event: &ClickEvent) {
        self.timer.as_mut().map(|t| t.event(mc, event));
        self.button.event(mc, event);
    }
}
