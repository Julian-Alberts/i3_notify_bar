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

#[derive(Serialize, Debug)]
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

#[derive(Deserialize, Debug)]
pub struct ClickEvent {
    name: Option<String>,
    #[serde(deserialize_with = "string_to_opt_usize")]
    instance: Option<usize>,
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

    pub fn get_instance(&self) -> Option<usize> {
        self.instance
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

pub enum Align {
    Left,
    Center,
    Right,
}

pub enum Markup {
    Pango,
    None,
}

fn string_to_opt_usize<'de, D>(d: D) -> Result<Option<usize>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let Some(value) = Option::<String>::deserialize(d)? else {
        return Ok(None);
    };
    value
        .as_str()
        .parse::<usize>()
        .map(Some)
        .map_err(serde::de::Error::custom)
}
