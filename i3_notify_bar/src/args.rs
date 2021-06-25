use clap::Clap;
use log::LevelFilter;

#[derive(Clap, Debug)]
#[clap(version = include_str!("../version.txt"), author = "Julian Alberts")]
pub struct Args {
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
    rule_file: Option<String>,
}

impl Args {
    pub fn log_level(&self) -> &LevelFilter {
        &self.log_level
    }

    pub fn log_file(&self) -> &Option<String> {
        &self.log_file
    }

    pub fn rule_file(&self) -> &Option<String> {
        &self.rule_file
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
}
