use crate::{component_manager::ManageComponents, property::Properties, protocol::ClickEvent, string::ComponentString};

use super::{prelude::*, BaseComponent};

pub struct Button {
    base_component: BaseComponent,
    text: Box<dyn ComponentString>,
    on_click: Box<dyn Fn(&mut Self, &mut dyn ManageComponents, &ClickEvent) + 'static>,
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
            text
        }
    }

    pub fn set_on_click<F: Fn(&mut Self, &mut dyn ManageComponents, &ClickEvent) + 'static>(
        &mut self,
        on_click: F,
    ) {
        self.on_click = Box::new(on_click);
    }
}

impl Component for Button {
    fn update(&mut self, dt: f64) {
        self.text.update(dt);
        self.base_component.get_properties_mut().text.full = self.text.to_component_text();
    }
    fn event(&mut self, mc: &mut dyn ManageComponents, ce: &ClickEvent) {
        let self_ptr: *mut _ = self;
        let self_ref = unsafe { self_ptr.as_mut().unwrap() };
        (self.on_click)(self_ref, mc, ce);
    }

    fn collect_base_components<'a>(&'a self, base_components: &mut Vec<&'a BaseComponent>) {
        base_components.push(&self.base_component)
    }

    fn collect_base_components_mut<'a>(
        &'a mut self,
        base_components: &mut Vec<&'a mut BaseComponent>,
    ) {
        base_components.push(&mut self.base_component);
    }

    fn name(&self) -> Option<&str> {
        self.base_component.get_name()
    }

}

impl Widget for Button {
    fn get_base_component(&self) -> &BaseComponent {
        &self.base_component
    }

    fn get_base_component_mut(&mut self) -> &mut BaseComponent {
        &mut self.base_component
    }
}

impl Seperator for Button {}
impl SeperatorWidth for Button {}

#[cfg(test)]
mod tests {

    use crate::component_manager::ComponentManagerBuilder;

    use super::*;

    #[test]
    fn on_button_click() {
        let mut button = Button::new(String::from("test").into());
        button.set_on_click(|btn, _, _| {
            btn.get_base_component_mut().get_properties_mut().name =
                Some(String::from("clicked"));
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
            "height": 0
        }
        "#,
        )
        .unwrap();

        button.event(&mut ComponentManagerBuilder::new().build(), &ce);
        assert_eq!(
            button
                .get_base_component_mut()
                .get_properties_mut()
                .name,
            Some(String::from("clicked"))
        )
    }
}
