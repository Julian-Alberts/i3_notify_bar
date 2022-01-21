use crate::{
    protocol::ClickEvent,
    ComponentManagerMessenger, property::Properties,
};

use super::{prelude::*, BaseComponent};

pub struct Button {
    base_component: BaseComponent,
    component_manager: Option<ComponentManagerMessenger>,
    on_click: fn(&mut Self, &ClickEvent),
}

impl Button {
    pub fn new(text: String) -> Button {
        Button {
            base_component: BaseComponent::from(Properties {
                text: crate::property::Text { 
                    full: text, 
                    short: None 
                },
                border: crate::property::Border { 
                    color: Some(String::from("#FFFFFF")),
                    ..Default::default()
                },
                ..Default::default()
            }),
            component_manager: None,
            on_click: |_, _| {},
        }
    }

    pub fn set_on_click(&mut self, on_click: fn(&mut Self, &ClickEvent)) {
        self.on_click = on_click;
    }
}

impl Component for Button {
    fn update(&mut self, _: f64) {}
    fn event(&mut self, ce: &ClickEvent) {
        (self.on_click)(self, ce);
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
        self.base_component.get_id()
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
    use super::*;

    #[test]
    fn on_button_click() {
        let mut button = Button::new(String::from("test"));
        button.set_on_click(|btn, _| {
            btn.get_base_component_mut()
                .get_properties_mut().instance = Some(String::from("clicked"));
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

        button.event(&ce);
        assert_eq!(
            button.get_base_component_mut().get_properties_mut().instance,
            Some(String::from("clicked"))
        )
    }
}
