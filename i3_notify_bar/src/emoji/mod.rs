use std::{path::Path, str::FromStr};

mod ignore;
mod remove;
#[cfg(feature = "emoji_mode_replace")]
mod replace;

#[derive(Debug, Clone)]
pub enum EmojiMode {
    Ignore,
    Remove,
    #[cfg(feature = "emoji_mode_replace")]
    Replace,
}

impl FromStr for EmojiMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "remove" => EmojiMode::Remove,
            #[cfg(feature = "emoji_mode_replace")]
            "replace" => EmojiMode::Replace,
            "ignore" => EmojiMode::Ignore,
            _ => return Err(format!("Unknown emoji mode {}", s)),
        })
    }
}

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
