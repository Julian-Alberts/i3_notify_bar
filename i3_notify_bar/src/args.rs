use clap::Clap;
use log::LevelFilter;

#[derive(Clap, Debug)]
#[clap(version = include_str!("../version.txt"), author = "Julian Alberts")]
pub struct Args {
    #[clap(long, default_value="off")]
    log_level: LevelFilter,
    #[clap(long, about="log file location")]
    log_file: Option<String>,
    #[clap(short, long, default_value="250")]
    refresh_rate: u64,
    rule_file: Option<String>
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
}