use log::LevelFilter;
use notify_server::notification::Urgency;

use emoji::EmojiMode;

#[derive(clap::Parser)]
#[clap(version = include_str!("../version.txt"), author = "Julian Alberts")]
pub struct Args {
    /// "Allowed values: "ignore", "remove", "replace"
    #[cfg(emoji_mode_replace)]
    #[clap(
        long,
        default_value = "ignore",
    )]
    pub emoji_mode: EmojiMode,
    
    /// Allowed values: "ignore", "remove"
    #[cfg(not(emoji_mode_replace))]
    #[clap(
        long,
        default_value = "ignore",
    )]
    pub emoji_mode: EmojiMode,
    
    ///Allowed values: "off", "Error", "Warn", "Info", "Debug", "Trace"
    #[clap(
        long,
        default_value = "off",
    )]
    pub log_level: LevelFilter,

    /// log file location
    #[clap(long)]
    pub log_file: Option<String>,
    
    /// override default emoji file
    #[clap(long)]
    pub emoji_file: Option<String>,
    
    ///Time between refresh in ms
    #[clap(
        short,
        long,
        default_value = "250",
    )]
    pub refresh_rate: u64,

    /// Maximum length a single notification can use in chars
    #[clap(
        short,
        long,
        default_value = "30",
    )]
    pub max_text_length: usize,

    /// How fast the text is animated
    #[clap(
        short,
        long,
        default_value = "5",
    )]
    pub animation_chars_per_second: usize,
    pub config_file: Option<String>,
    #[clap(subcommand)]
    pub command: Option<Command>,
}

#[derive(clap::Parser, Debug)]
pub enum Command {
    DebugConfig(DebugConfig),
    Run,
}

#[derive(clap::Parser, Debug)]
pub struct DebugConfig {
    #[clap(long, default_value = "")]
    pub app_name: String,
    #[clap(long, default_value = "0")]
    pub id: u32,
    #[clap(long, default_value = "")]
    pub app_icon: String,
    #[clap(long, default_value = "")]
    pub summary: String,
    #[clap(long, default_value = "")]
    pub body: String,
    #[clap(long, default_value = "normal")]
    pub urgency: Urgency,
    #[clap(long, default_value = "0")]
    pub expire_timeout: i32,
}
