use i3_bar_components::component_manager::ManageComponents;
use i3_bar_components::{
    components::{prelude::Component, BaseComponent, Button, ProgressBar},
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

    fn collect_base_components<'a>(&'a self, base_components: &mut Vec<&'a BaseComponent>) {
        self.timer
            .as_ref()
            .map(|t| t.collect_base_components(base_components));
        self.button.collect_base_components(base_components);
    }

    fn collect_base_components_mut<'a>(
        &'a mut self,
        base_components: &mut Vec<&'a mut BaseComponent>,
    ) {
        self.timer
            .as_mut()
            .map(|t| t.collect_base_components_mut(base_components));
        self.button.collect_base_components_mut(base_components);
    }

    fn event(&mut self, mc: &mut dyn ManageComponents, event: &ClickEvent) {
        self.timer.as_mut().map(|t| t.event(mc, event));
        self.button.event(mc, event);
    }

    fn name(&self) -> Option<&str> {
        None
    }
}
