use clap::Clap;
use log::LevelFilter;

use crate::emoji::EmojiMode;

#[derive(Clap, Debug)]
#[clap(version = include_str!("../version.txt"), author = "Julian Alberts")]
pub struct Args {
    #[cfg(emoji_mode_replace)]
    #[clap(
        long,
        default_value = "ignore",
        about = r#"Allowed values: "ignore", "remove", "replace""#
    )]
    emoji_mode: EmojiMode,
    #[cfg(not(emoji_mode_replace))]
    #[clap(
        long,
        default_value = "ignore",
        about = r#"Allowed values: "ignore", "remove""#
    )]
    emoji_mode: EmojiMode,
    #[clap(
        long,
        default_value = "off",
        about = r#"Allowed values: "off", "Error", "Warn", "Info", "Debug", "Trace""#
    )]
    log_level: LevelFilter,
    #[clap(long, about = "log file location")]
    log_file: Option<String>,
    #[clap(
        short,
        long,
        default_value = "250",
        about = "Time between refresh in ms"
    )]
    refresh_rate: u64,
    #[clap(
        short,
        long,
        default_value = "30",
        about = "Maximum length a single notification can use in chars"
    )]
    max_text_length: usize,
    #[clap(
        short,
        long,
        default_value = "5",
        about = "How fast the text is animated"
    )]
    animation_chars_per_second: usize,
    config_file: Option<String>,
}

impl Args {
    pub fn log_level(&self) -> &LevelFilter {
        &self.log_level
    }

    pub fn log_file(&self) -> &Option<String> {
        &self.log_file
    }

    pub fn config_file(&self) -> &Option<String> {
        &self.config_file
    }

    pub fn refresh_rate(&self) -> u64 {
        self.refresh_rate
    }

    pub fn max_text_length(&self) -> usize {
        self.max_text_length
    }

    pub fn animation_chars_per_second(&self) -> usize {
        self.animation_chars_per_second
    }
    pub fn emoji_mode(&self) -> EmojiMode {
        self.emoji_mode.clone()
    }
}
