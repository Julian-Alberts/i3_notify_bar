use log::*;
use serde::{Deserialize, Serialize};

macro_rules! create_setter {
    ($var:ident: $type:ty => $(set: $set:ident)?  $(with: $with:ident)? $(get: $get:ident)?) => {
        $(
            pub fn $set(&mut self, $var: $type) {
                self.$var = Some($var);
            }
        )?

        $(
            pub fn $with(mut self, $var: $type) -> Self {
                self.$set($var);
                self
            }
        )?

        $(
            pub fn $get(&self) -> &Option<$type> {
                &self.$var
            }
        )?

    };
}

#[derive(Serialize)]
pub struct Header {
    version: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    click_events: Option<bool>,
}

impl Default for Header {
    fn default() -> Self {
        Self::new()
    }
}

impl Header {
    pub fn new() -> Header {
        Self {
            version: 1,
            click_events: None,
        }
    }

    create_setter!(click_events: bool => set: set_click_events with: with_click_events);
}

#[derive(Serialize, Debug, PartialEq)]
pub struct Block {
    full_text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    short_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    background: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    border: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    border_top: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    border_right: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    border_bottom: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    border_left: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    min_width: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    align: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    instance: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    urgent: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    separator: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    separator_block_width: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    markup: Option<String>,
}

impl Default for Block {
    fn default() -> Self {
        Self::new()
    }
}

impl Block {
    pub fn new() -> Self {
        let instance = format!("{}", rand::random::<u128>());

        debug!(r#"Created block with id "{}""#, instance);

        Block {
            full_text: String::new(),
            short_text: None,
            color: None,
            background: None,
            border: None,
            border_top: None,
            border_right: None,
            border_bottom: None,
            border_left: None,
            min_width: None,
            align: None,
            name: None,
            instance,
            urgent: None,
            separator: None,
            separator_block_width: None,
            markup: None,
        }
    }

    pub fn set_full_text(&mut self, full_text: String) {
        self.full_text = full_text;
    }

    pub fn with_full_text(mut self, full_text: String) -> Self {
        self.full_text = full_text;
        self
    }

    create_setter!(short_text: String => set: set_short_text with: with_short_text);
    create_setter!(color: String => set: set_color with: with_color);
    create_setter!(background: String => set: set_background with: with_background);
    create_setter!(border: String => set: set_border with: with_border);
    create_setter!(border_top: usize => set: set_border_top with: with_border_top);
    create_setter!(border_right: usize => set: set_border_right with: with_border_right);
    create_setter!(border_bottom: usize => set: set_border_bottom with: with_border_bottom);
    create_setter!(border_left: usize => set: set_border_left with: with_border_left);
    create_setter!(min_width: usize => set: set_min_width with: with_min_width);

    pub fn with_align(mut self, align: Align) -> Self {
        self.set_align(align);
        self
    }

    pub fn set_align(&mut self, align: Align) {
        self.align = Some(match align {
            Align::Left => String::from("left"),
            Align::Center => String::from("center"),
            Align::Right => String::from("right"),
        });
    }

    create_setter!(name: String => set: set_name  with: with_name get: name);
    create_setter!(urgent: bool => set: set_urgent with: with_urgent);
    create_setter!(separator: bool => set: set_separator with: with_separator);
    create_setter!(separator_block_width: usize => set: set_separator_block_width with: with_separator_block_width);

    pub fn with_markup(mut self, markup: Markup) -> Self {
        self.set_markup(markup);
        self
    }

    pub fn set_markup(&mut self, markup: Markup) {
        self.align = Some(match markup {
            Markup::Pango => String::from("pango"),
            Markup::None => String::from("none"),
        });
    }
}

impl Block {
    pub fn get_id(&self) -> &str {
        &self.instance
    }
}

#[derive(Deserialize, Debug)]
pub struct ClickEvent {
    name: Option<String>,
    instance: Option<String>,
    button: usize,
    modifiers: Option<Vec<String>>,
    x: usize,
    y: usize,
    relative_x: usize,
    relative_y: usize,
    output_x: usize,
    output_y: usize,
    width: usize,
    height: usize,
}

impl ClickEvent {
    pub fn get_name(&self) -> &Option<String> {
        &self.name
    }

    pub fn get_instance(&self) -> &Option<String> {
        &self.instance
    }

    pub fn get_button(&self) -> usize {
        self.button
    }

    pub fn get_modifiers(&self) -> &Option<Vec<String>> {
        &self.modifiers
    }

    pub fn get_x(&self) -> usize {
        self.x
    }

    pub fn get_y(&self) -> usize {
        self.y
    }

    pub fn get_relative_x(&self) -> usize {
        self.relative_x
    }

    pub fn get_relative_y(&self) -> usize {
        self.relative_y
    }

    pub fn get_output_x(&self) -> usize {
        self.output_x
    }

    pub fn get_output_y(&self) -> usize {
        self.output_y
    }

    pub fn get_width(&self) -> usize {
        self.width
    }

    pub fn get_height(&self) -> usize {
        self.height
    }
}

impl ClickEvent {
    pub fn get_id(&self) -> &str {
        match self.get_instance() {
            Some(s) => s,
            None => "",
        }
    }
}

pub enum Align {
    Left,
    Center,
    Right,
}

pub enum Markup {
    Pango,
    None,
}
