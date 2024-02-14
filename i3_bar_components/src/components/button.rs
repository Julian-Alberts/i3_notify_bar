use crate::{
    component_manager::ManageComponents, property::Properties, protocol::ClickEvent,
    string::ComponentString,
};

use super::{prelude::*, BaseComponent};

type ClickHandler = dyn Fn(&mut Button, &mut dyn ManageComponents, &ClickEvent) + 'static;

pub struct Button {
    base_component: BaseComponent,
    text: Box<dyn ComponentString>,
    on_click: Box<ClickHandler>,
}

impl Button {
    pub fn new(text: Box<dyn ComponentString>) -> Button {
        Button {
            base_component: BaseComponent::from(Properties {
                border: crate::property::Border {
                    color: Some(String::from("#FFFFFF")),
                    ..Default::default()
                },
                ..Default::default()
            }),
            on_click: Box::new(|_, _, _| {}),
            text,
        }
    }

    pub fn set_on_click<F: Fn(&mut Self, &mut dyn ManageComponents, &ClickEvent) + 'static>(
        &mut self,
        on_click: F,
    ) {
        self.on_click = Box::new(on_click);
    }
}

impl SimpleComponent for Button {
    fn properties(&self) -> &crate::property::Properties {
        self.base_component.properties()
    }
    fn properties_mut(&mut self) -> &mut crate::property::Properties {
        self.base_component.properties_mut()
    }
}

impl Component for Button {
    fn update(&mut self, dt: f64) {
        self.text.update(dt);
        self.set_full(self.text.to_component_text());
    }
    fn all_properties<'a>(&'a self) -> Box<dyn Iterator<Item = &Properties> + 'a> {
        Box::new([self.properties()].into_iter())
    }
    fn event_targets<'a>(
        &'a self,
    ) -> Box<
        (dyn Iterator<
            Item = (
                crate::property::Instance,
                *const (dyn EventTarget + 'static),
            ),
        > + 'a),
    > {
        Box::new(std::iter::once((
            self.properties().instance,
            self as *const _,
        )))
    }
}

impl EventTarget for Button {
    fn event(&mut self, mc: &mut dyn ManageComponents, ce: &ClickEvent) {
        let self_ptr: *mut _ = self;
        let self_ref = unsafe { self_ptr.as_mut().unwrap() };
        (self.on_click)(self_ref, mc, ce);
    }
}

#[cfg(test)]
mod tests {

    use crate::component_manager::ComponentManagerBuilder;

    use super::*;

    #[test]
    fn on_button_click() {
        let mut button = Button::new(Box::new(String::from("test")));
        button.set_on_click(|btn, _, _| {
            btn.properties_mut().name = Some(String::from("clicked"));
        });

        let ce: ClickEvent = serde_json::from_str(
            r#"
        {
            "button": 0,
            "x": 0,
            "y": 0,
            "relative_x": 0,
            "relative_y": 0,
            "output_x": 0,
            "output_y": 0,
            "width": 0,
            "height": 0,
            "instance": "1"
        }
        "#,
        )
        .unwrap();

        button.event(&mut ComponentManagerBuilder::new().build(), &ce);
        assert_eq!(button.properties_mut().name, Some(String::from("clicked")))
    }
}
