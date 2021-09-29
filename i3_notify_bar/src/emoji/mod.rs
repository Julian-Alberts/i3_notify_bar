use std::{
    error::Error,
    fmt::{Display, Formatter},
    path::Path,
    str::FromStr,
};

mod ignore;
mod remove;
#[cfg(feature = "emoji_mode_replace")]
mod replace;

#[derive(Debug, Clone, PartialEq)]
pub enum EmojiMode {
    Ignore,
    Remove,
    #[cfg(feature = "emoji_mode_replace")]
    Replace,
}

impl FromStr for EmojiMode {
    type Err = EmojiModeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "remove" => EmojiMode::Remove,
            #[cfg(feature = "emoji_mode_replace")]
            "replace" => EmojiMode::Replace,
            "ignore" => EmojiMode::Ignore,
            _ => {
                return Err(EmojiModeError {
                    given_value: s.to_owned(),
                })
            }
        })
    }
}

#[derive(Debug)]
pub struct EmojiModeError {
    given_value: String,
}

impl Display for EmojiModeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, r#"Unknown Emoji mode "{}""#, self.given_value)
    }
}

impl Error for EmojiModeError {}

pub fn handle(text: String, mode: &EmojiMode) -> String {
    match mode {
        EmojiMode::Ignore => ignore::handle(text),
        EmojiMode::Remove => remove::handle(text),
        #[cfg(feature = "emoji_mode_replace")]
        EmojiMode::Replace => replace::handle(text),
    }
}

pub fn init(emoji_file: Option<&Path>) {
    if let Some(ef) = emoji_file {
        replace::load_emoji_file(ef)
    }
}
