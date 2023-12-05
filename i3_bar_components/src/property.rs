#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub struct Instance(usize);

#[derive(Default, Debug, PartialEq, serde::Serialize)]
pub struct Properties {
    #[serde(flatten)]
    pub text: Text,
    #[serde(flatten)]
    pub color: Color,
    #[serde(flatten)]
    pub border: Border,
    #[serde(flatten)]
    pub separator: Separator,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_width: Option<usize>,
    pub align: Align,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub instance: Instance,
    pub urgent: bool,
    pub markup: Markup,
}

#[derive(Default, Debug, PartialEq, serde::Serialize)]
pub struct Text {
    #[serde(rename = "full_text")]
    pub full: String,
    #[serde(rename = "short_text", skip_serializing_if = "Option::is_none")]
    pub short: Option<String>,
}

impl From<String> for Text {
    fn from(full: String) -> Self {
        Self { full, short: None }
    }
}

#[derive(Default, Debug, PartialEq, serde::Serialize)]
pub struct Color {
    #[serde(rename = "color", skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(rename = "background", skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
}

#[derive(Default, Debug, PartialEq, serde::Serialize)]
pub struct Border {
    #[serde(rename = "border", skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(rename = "border_top", skip_serializing_if = "Option::is_none")]
    pub top: Option<usize>,
    #[serde(rename = "border_right", skip_serializing_if = "Option::is_none")]
    pub right: Option<usize>,
    #[serde(rename = "border_bottom", skip_serializing_if = "Option::is_none")]
    pub bottom: Option<usize>,
    #[serde(rename = "border_left", skip_serializing_if = "Option::is_none")]
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

#[derive(Default, Debug, PartialEq, serde::Serialize)]
pub struct Separator {
    #[serde(rename = "separator")]
    pub show: bool,
    #[serde(
        rename = "separator_block_width",
        skip_serializing_if = "Option::is_none"
    )]
    pub block_width: Option<usize>,
}

#[derive(Debug, PartialEq, Clone, Copy, Default, serde::Serialize)]
pub enum Align {
    #[default]
    #[serde(rename = "left")]
    Left,
    #[serde(rename = "center")]
    Center,
    #[serde(rename = "right")]
    Right,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, serde::Serialize)]
pub enum Markup {
    #[serde(rename = "pango")]
    Pango,
    #[default]
    #[serde(rename = "none")]
    None,
}

impl Default for Instance {
    fn default() -> Self {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static INSTANCE: AtomicUsize = AtomicUsize::new(0);
        Self(INSTANCE.fetch_add(1, Ordering::Relaxed))
    }
}

impl PartialEq<usize> for Instance {
    fn eq(&self, other: &usize) -> bool {
        self.0 == *other
    }
}
