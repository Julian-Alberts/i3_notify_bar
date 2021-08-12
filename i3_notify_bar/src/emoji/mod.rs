use std::str::FromStr;

mod ignore;
mod remove;
#[cfg(emoji_mode_replace)]
mod replace;

#[derive(Debug, Clone)]
pub enum EmojiMode {
    Ignore,
    Remove,
    #[cfg(emoji_mode_replace)]
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
        #[cfg(emoji_mode_replace)]
        EmojiMode::Replace => replace::handle(text),
    }
}
