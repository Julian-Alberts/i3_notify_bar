use std::sync::{Arc, Mutex};

use crate::{protocol::ClickEvent, component_manager::ManageComponents, property::Border};

use super::{prelude::{Component, Widget}, Button};

pub struct ButtonGroup<K: Copy + PartialEq + 'static> {
    buttons: Vec<GroupButton<K>>,
    selected: Arc<Mutex<K>>
}

impl <K: Copy + PartialEq + 'static> ButtonGroup<K> {

    pub fn new(mut buttons: Vec<GroupButton<K>>, selected: Arc<Mutex<K>>) -> Self {
        buttons.sort_by(|a, b| a.pos.cmp(&b.pos));
        let mut button_group = Self {
            buttons,
            selected
        };
        let selected = *button_group.selected.lock().unwrap();
        button_group.select(selected);
        button_group
    }

    pub fn select(&mut self, selected_key: K) {
        self.buttons.iter_mut().for_each(|button| {
            let button_key = button.key;
            let border = &mut button.get_base_component_mut().get_properties_mut().border;
                
            if button_key == selected_key {
                *border = Border {
                    top: Some(2),
                    left: Some(2),
                    bottom: Some(2),
                    right: Some(2),
                    color: border.color.clone(),
                }
            } else {
                *border = Border {
                    top: Some(1),
                    left: Some(1),
                    bottom: Some(1),
                    right: Some(1),
                    color: border.color.clone(),
                }
            }
        });
        match self.selected.lock() {
            Ok(mut s) => *s = selected_key,
            Err(_) => {}
        }
    }

}

impl <K: Copy + PartialEq + 'static> Component for ButtonGroup<K> {

    fn collect_base_components<'a>(&'a self, base_components: &mut Vec<&'a super::BaseComponent>) {
        self.buttons.iter().for_each(|b| b.collect_base_components(base_components))
    }

    fn collect_base_components_mut<'a>(&'a mut self, base_components: &mut Vec<&'a mut super::BaseComponent>) {
        self.buttons.iter_mut().for_each(|b| b.collect_base_components_mut(base_components))
    }

    fn event(&mut self, _: &mut dyn ManageComponents, event: &ClickEvent) {
        let buttons = &mut self.buttons;
        let selected = buttons.iter_mut().find_map(|button| {
            if &button.get_base_component().get_properties().instance == event.get_instance() {
                return Some(button.key.clone())
            }
            return None
        });

        if let Some(selected) = selected {
            self.select(selected);
        }
    }

    fn get_id(&self) -> &str {
        ""
    }

    fn name(&self) -> &str {
        ""
    }

    fn update(&mut self, _: f64) {}

}

pub struct GroupButton<K: Copy + PartialEq + 'static> {
    pos: isize,
    key: K,
    button: Button
}

impl <K: Copy + PartialEq + 'static> GroupButton<K> {

    pub fn new(pos: isize, key: K, button: Button) -> Self {
        Self {
            pos,
            key,
            button
        }
    }

}

impl <K: Copy + PartialEq + 'static> Component for GroupButton<K> {

    fn collect_base_components<'a>(&'a self, base_components: &mut Vec<&'a super::BaseComponent>) {
        self.button.collect_base_components(base_components)
    }

    fn collect_base_components_mut<'a>(&'a mut self, base_components: &mut Vec<&'a mut super::BaseComponent>) {
        self.button.collect_base_components_mut(base_components)
    }

    fn event(&mut self, cm: &mut dyn ManageComponents, event: &ClickEvent) {
        self.button.event(cm, event)
    }

    fn get_id(&self) -> &str {
        self.button.get_id()
    }

    fn name(&self) -> &str {
        self.button.name()
    }

    fn update(&mut self, dt: f64) {
        self.button.update(dt)
    }

}

impl <K: Copy + PartialEq + 'static> Widget for GroupButton<K> {
    
    fn get_base_component(&self) -> &super::BaseComponent {
        self.button.get_base_component()
    }

    fn get_base_component_mut(&mut self) -> &mut super::BaseComponent {
        self.button.get_base_component_mut()
    }

}