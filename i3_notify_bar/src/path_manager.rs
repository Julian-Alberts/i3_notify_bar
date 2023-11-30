use std::{env, path::{PathBuf, Path}};

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
    log_file: Option<PathBuf>,
    config_file: Option<PathBuf>,
    emoji_file: Option<PathBuf>,
}

impl Default for PathManager {
    fn default() -> Self {
        let home_dir = match home_dir() {
            Some(h) => PathBuf::from(h),
            None => {
                return PathManager {
                    config_file: None,
                    log_file: None,
                    emoji_file: None,
                }
            }
        };

        let log_file = home_dir.join(".config/i3_notify_bar/log");
        let config_file = home_dir.join(".config/i3_notify_bar/config");
        let emoji_file = home_dir.join(".config/i3_notify_bar/emojis");

        PathManager {
            log_file: Some(log_file),
            config_file: Some(config_file),
            emoji_file: Some(emoji_file),
        }
    }
}

impl PathManager {
    pub fn set_log_file(&mut self, file: PathBuf) {
        self.log_file = Some(file);
    }

    pub fn log_file(&self) -> Option<&Path> {
        self.log_file.as_deref()
    }

    pub fn set_config_file(&mut self, file: PathBuf) {
        self.config_file = Some(file);
    }

    pub fn config_file(&self) -> Option<&Path> {
        self.config_file.as_deref()
    }

    pub fn set_emoji_file(&mut self, file: PathBuf) {
        self.emoji_file = Some(file)
    }

    pub fn emoji_file(&self) -> Option<&Path> {
        self.emoji_file.as_deref()
    }
}
