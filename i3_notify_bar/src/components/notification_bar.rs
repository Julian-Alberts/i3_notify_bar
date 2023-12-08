use i3_bar_components::components::{prelude::*, BaseComponent};

use super::{NotificationComponent, NotificationGroup};

pub struct NotificationBar {
    notifications: Vec<NotificationComponent>,
    groups: Vec<NotificationGroup>,
    menu_btn: BaseComponent,
}

impl NotificationBar {
    pub fn new() -> Self {
        Self {
            notifications: Vec::default(),
            groups: Vec::default(),
            menu_btn: todo!(),
        }
    }
}

impl Component for NotificationBar {
    fn all_properties<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = &i3_bar_components::property::Properties> + 'a> {
        Box::new(
            self.notifications
                .iter()
                .map(Component::all_properties)
                .chain(self.groups.iter().map(Component::all_properties))
                .flatten()
                .chain(self.menu_btn.all_properties()),
        )
    }

    fn update(&mut self, dt: f64) {
        self.notifications
            .iter_mut()
            .map::<&mut dyn Component, _>(|n| n)
            .chain(self.groups.iter_mut().map::<&mut dyn Component, _>(|g| g))
            .for_each(|c| c.update(dt));
        self.menu_btn.update(dt);
    }

    fn event(
        &mut self,
        cm: &mut dyn i3_bar_components::ManageComponents,
        event: &i3_bar_components::protocol::ClickEvent,
    ) {
        self.notifications
            .iter_mut()
            .map::<&mut dyn Component, _>(|n| n)
            .chain(self.groups.iter_mut().map::<&mut dyn Component, _>(|g| g))
            .for_each(|c| c.event(cm, event));
        self.menu_btn.event(cm, event);
    }
}
