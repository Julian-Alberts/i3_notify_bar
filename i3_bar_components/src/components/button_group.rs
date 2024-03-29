use std::sync::{Arc, RwLock};

use crate::{
    component_manager::ManageComponents, property::Properties, protocol::ClickEvent,
    string::ComponentString,
};

use super::{
    prelude::{Color, *},
    Button, Label,
};

pub struct ButtonGroup<
    K: Copy + PartialEq + 'static,
    LT: ComponentString = Box<dyn ComponentString>,
> {
    description: Option<Label<LT>>,
    buttons: Vec<GroupButton<K>>,
    selected: Arc<RwLock<K>>,
    name: Option<String>,
}

impl<K: Copy + PartialEq + 'static, LT: ComponentString> ButtonGroup<K, LT> {
    pub fn new(
        mut buttons: Vec<GroupButton<K>>,
        selected: Arc<RwLock<K>>,
        description: Option<Label<LT>>,
    ) -> Self {
        buttons.sort_by(|a, b| a.pos.cmp(&b.pos));
        let mut button_group = Self {
            buttons,
            selected,
            name: None,
            description,
        };
        let selected = *button_group.selected.read().unwrap();
        button_group.select(selected);
        button_group
    }

    pub fn select(&mut self, selected_key: K) {
        let current_selected = *self.selected.read().unwrap();
        self.buttons.iter_mut().for_each(|button| {
            if button.key == selected_key {
                button.select();
            } else if button.key == current_selected {
                button.deselect()
            }
        });
        if let Ok(mut s) = self.selected.write() {
            *s = selected_key
        }
    }

    pub fn set_name(&mut self, name: String) {
        self.name = Some(name)
    }
}

impl<K: Copy + PartialEq + 'static, LT: ComponentString + 'static> Component
    for ButtonGroup<K, LT>
{
    fn update(&mut self, dt: f64) {
        self.buttons.iter_mut().for_each(|btn| btn.update(dt));
        if let Some(description) = self.description.as_mut() {
            description.update(dt);
        }
    }

    fn all_properties<'a>(&'a self) -> Box<dyn Iterator<Item = &Properties> + 'a> {
        Box::new(self.buttons.iter().flat_map(|b| b.all_properties()))
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
        Box::new(
            self.buttons
                .iter()
                .map(|b| (b.properties().instance, self as *const _)),
        )
    }
}

impl<K: Copy + PartialEq + 'static, LT: ComponentString> EventTarget for ButtonGroup<K, LT> {
    fn event(&mut self, _: &mut dyn ManageComponents, event: &ClickEvent) {
        let Some(clicked_element) = event.get_instance() else {
            return;
        };
        let buttons = &mut self.buttons;
        let selected = buttons.iter_mut().find_map(|button| {
            if button.instance() == clicked_element {
                return Some(button.key);
            }
            None
        });

        if let Some(selected) = selected {
            self.select(selected);
        }
    }
}

pub struct GroupButton<K: Copy + PartialEq + 'static> {
    pos: isize,
    key: K,
    button: Button,
}

impl<K: Copy + PartialEq + 'static> GroupButton<K> {
    pub fn new(pos: isize, key: K, button: Button) -> Self {
        Self { pos, key, button }
    }

    fn deselect(&mut self) {
        let color = self.button.color_mut();
        std::mem::swap(&mut color.text, &mut color.background);
    }

    fn select(&mut self) {
        let color = self.button.color_mut();
        std::mem::swap(&mut color.text, &mut color.background);
    }
}

impl<K: Copy + PartialEq + 'static> SimpleComponent for GroupButton<K> {
    fn properties_mut(&mut self) -> &mut crate::property::Properties {
        self.button.properties_mut()
    }
    fn properties(&self) -> &crate::property::Properties {
        self.button.properties()
    }
}

impl<K: Copy + PartialEq + 'static> Component for GroupButton<K> {
    fn update(&mut self, dt: f64) {
        self.button.update(dt)
    }

    fn all_properties<'a>(&'a self) -> Box<dyn Iterator<Item = &Properties> + 'a> {
        Box::new([self.button.properties()].into_iter())
    }
}
