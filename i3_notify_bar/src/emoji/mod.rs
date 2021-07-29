use std::str::FromStr;

mod ignore;
#[cfg(feature = "emoji_mode_remove")]
mod remove;
#[cfg(feature = "emoji_mode_replace")]
mod replace;

#[derive(Debug, Clone)]
pub enum EmojiMode {
    Ignore,
    Remove,
    #[allow(dead_code)]
    Replace,
}

impl FromStr for EmojiMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "remove" => EmojiMode::Remove,
            #[cfg(emoji_mode_replace)]
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
        EmojiMode::Replace => replace::handle(text),
    }
}
