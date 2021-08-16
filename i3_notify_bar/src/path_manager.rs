use std::{env, path::PathBuf};

use log::info;

fn home_dir() -> Option<String> {
    match env::var("HOME") {
        Ok(home) => return Some(home),
        Err(env::VarError::NotPresent) => info!("Enviroment variable HOME not set"),
        Err(env::VarError::NotUnicode(_)) => info!("Enviroment variable HOME has invalid value"),
    }

    None
}

pub struct PathManager {
    log_file: Option<String>,
    config_file: Option<String>,
}

impl Default for PathManager {
    fn default() -> Self {
        let home_dir = match home_dir() {
            Some(h) => h,
            None => {
                return PathManager {
                    config_file: None,
                    log_file: None,
                }
            }
        };
        let mut log_file = PathBuf::from(home_dir.clone());
        log_file.push(".config/i3_notify_bar/log");

        let mut config_file = PathBuf::from(home_dir);
        config_file.push(".config/i3_notify_bar/config");

        PathManager {
            log_file: log_file.to_str().map(str::to_owned),
            config_file: config_file.to_str().map(str::to_owned),
        }
    }
}

impl PathManager {
    pub fn set_log_file(&mut self, file: String) {
        self.log_file = Some(file);
    }

    pub fn log_file(&self) -> &Option<String> {
        &self.log_file
    }

    pub fn set_config_file(&mut self, file: String) {
        self.config_file = Some(file);
    }

    pub fn config_file(&self) -> &Option<String> {
        &self.config_file
    }
}
