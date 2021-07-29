mod ignore;
#[cfg(feature = "emoji_mode_remove")]
mod remove;
#[cfg(feature = "emoji_mode_replace")]
mod replace;

#[derive(Debug)]
pub enum EmojiMode {
    Ignore,
    #[cfg(feature = "emoji_mode_remove")]
    Remove,
    #[cfg(feature = "emoji_mode_replace")]
    Replace
}

impl Default for EmojiMode {

    fn default() -> Self {
        EmojiMode::Ignore
    }

}

pub fn handle(text: String, mode: EmojiMode) -> String {
    match mode {
        EmojiMode::Ignore => ignore::handle(text),
        #[cfg(feature = "emoji_mode_remove")]
        EmojiMode::Remove => remove::handle(text),
        #[cfg(feature = "emoji_mode_replace")]
        EmojiMode::Replace => replace::handle(text)
    }
}