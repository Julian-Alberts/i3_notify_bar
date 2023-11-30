use clap::Parser;
use notify_server::notification::Urgency;
use log::LevelFilter;
use emoji::EmojiMode;

mod cli;

pub fn load() -> Args {
    cli::Args::parse().into()
}

pub struct Args {
    pub emoji_mode: EmojiMode,
    pub log_level: LevelFilter,
    pub log_file: Option<String>,
    pub emoji_file: Option<String>,
    pub refresh_rate: u64,
    pub max_text_length: usize,
    pub animation_chars_per_second: usize,
    pub config_file: Option<String>,
    pub command: Command,
}

#[derive(Default)]
pub enum Command {
    DebugConfig(DebugConfig),
    #[default]
    Run,
}

pub struct DebugConfig {
    pub app_name: String,
    pub id: u32,
    pub app_icon: String,
    pub summary: String,
    pub body: String,
    pub urgency: Urgency,
    pub expire_timeout: i32,
}

impl From<cli::Args> for Args {

    fn from(cli_args: cli::Args) -> Self {
        Args {
            emoji_mode: cli_args.emoji_mode,
            log_level: cli_args.log_level,
            log_file: cli_args.log_file,
            emoji_file: cli_args.emoji_file,
            refresh_rate: cli_args.refresh_rate,
            max_text_length: cli_args.max_text_length,
            animation_chars_per_second: cli_args.animation_chars_per_second,
            config_file: cli_args.config_file,
            command: cli_args.command.into()
        }
    }
    
}

impl From<Option<cli::Command>> for Command {
    fn from(c: Option<cli::Command>) -> Self {
        c.map(|c| match c {
            cli::Command::DebugConfig(dc) => Command::DebugConfig(dc.into()),
            cli::Command::Run => Command::Run
        }).unwrap_or_default()
    }
}

impl From<cli::DebugConfig> for DebugConfig {
    fn from(dc: cli::DebugConfig) -> Self {
        DebugConfig {
            app_name: dc.app_name,
            id: dc.id,
            app_icon: dc.app_icon,
            summary: dc.summary,
            body: dc.body,
            urgency: dc.urgency,
            expire_timeout: dc.expire_timeout,
        }
    }
}

