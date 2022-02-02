use serde::ser::SerializeStruct;

macro_rules! serialize_field {
    ($field_name: expr => $name: literal, $state: ident) => {
        if let Some(value) = &$field_name {
            $state.serialize_field($name, value)?;
        }
    };
    ($field_name: expr => $name: literal?, $state: ident) => {
        $state.serialize_field($name, &$field_name)?;
    };
}

#[derive(Default, Debug, PartialEq)]
pub struct Properties {
    pub text: Text,
    pub color: Color,
    pub border: Border,
    pub separator: Separator,
    pub min_width: Option<usize>,
    pub align: Align,
    pub name: Option<String>,
    pub instance: Option<u32>,
    pub urgent: bool,
    pub markup: Markup,
}

impl serde::Serialize for Properties {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Properties", 17)?;

        state.serialize_field("full_text", &self.text.full)?;
        serialize_field!(self.text.short => "short_text", state);

        serialize_field!(self.color.text => "color", state);
        serialize_field!(self.color.background => "background", state);

        serialize_field!(self.border.color => "border", state);
        serialize_field!(self.border.top => "border_top", state);
        serialize_field!(self.border.right => "border_right", state);
        serialize_field!(self.border.bottom => "border_bottom", state);
        serialize_field!(self.border.left => "border_left", state);

        serialize_field!(self.separator.show => "separator"?, state);
        serialize_field!(self.separator.block_width => "separator_block_width", state);

        serialize_field!(self.min_width => "min_width", state);

        match self.align {
            Align::Left => {}
            Align::Center => state.serialize_field("align", "center")?,
            Align::Right => state.serialize_field("align", "right")?,
        }

        serialize_field!(self.name => "name", state);
        serialize_field!(self.instance => "instance", state);

        serialize_field!(self.urgent => "urgent"? ,state);

        match self.markup {
            Markup::Pango => state.serialize_field("markup", "pango")?,
            Markup::None => {}
        }

        state.end()
    }
}

#[derive(Default, Debug, PartialEq)]
pub struct Text {
    pub full: String,
    pub short: Option<String>,
}

impl From<String> for Text {
    fn from(full: String) -> Self {
        Self { full, short: None }
    }
}

#[derive(Default, Debug, PartialEq)]
pub struct Color {
    pub text: Option<String>,
    pub background: Option<String>,
}

#[derive(Default, Debug, PartialEq)]
pub struct Border {
    pub color: Option<String>,
    pub top: Option<usize>,
    pub right: Option<usize>,
    pub bottom: Option<usize>,
    pub left: Option<usize>,
}

impl From<(String, usize)> for Border {
    fn from((color, widht): (String, usize)) -> Self {
        Self {
            color: Some(color),
            bottom: Some(widht),
            left: Some(widht),
            right: Some(widht),
            top: Some(widht),
        }
    }
}

impl From<usize> for Border {
    fn from(widht: usize) -> Self {
        Self {
            color: None,
            bottom: Some(widht),
            left: Some(widht),
            right: Some(widht),
            top: Some(widht),
        }
    }
}

#[derive(Default, Debug, PartialEq)]
pub struct Separator {
    pub show: bool,
    pub block_width: Option<usize>,
}

#[derive(Debug, PartialEq)]
pub enum Align {
    Left,
    Center,
    Right,
}

impl Default for Align {
    fn default() -> Self {
        Self::Left
    }
}

#[derive(Debug, PartialEq)]
pub enum Markup {
    Pango,
    None,
}

impl Default for Markup {
    fn default() -> Self {
        Self::None
    }
}
