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
    pub emoji_mode: EmojiMode,
    #[cfg(not(emoji_mode_replace))]
    #[clap(
        long,
        default_value = "ignore",
        about = r#"Allowed values: "ignore", "remove""#
    )]
    pub emoji_mode: EmojiMode,
    #[clap(
        long,
        default_value = "off",
        about = r#"Allowed values: "off", "Error", "Warn", "Info", "Debug", "Trace""#
    )]
    pub log_level: LevelFilter,
    #[clap(long, about = "log file location")]
    pub log_file: Option<String>,
    #[clap(
        short,
        long,
        default_value = "250",
        about = "Time between refresh in ms"
    )]
    pub refresh_rate: u64,
    #[clap(
        short,
        long,
        default_value = "30",
        about = "Maximum length a single notification can use in chars"
    )]
    pub max_text_length: usize,
    #[clap(
        short,
        long,
        default_value = "5",
        about = "How fast the text is animated"
    )]
    pub animation_chars_per_second: usize,
    pub config_file: Option<String>,
}
