use std::path::PathBuf;
use anyhow::{anyhow, Result};

pub struct Config {
    pub store_dir: PathBuf,
    pub clip_time: u64,
    pub gpg_opts: Vec<String>,
    pub generated_length: usize,
    pub character_set: Option<String>,
    pub character_set_no_symbols: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let store_dir = if let Ok(dir) = std::env::var("PASSWORD_STORE_DIR") {
            PathBuf::from(dir)
        } else {
            dirs::home_dir()
                .ok_or_else(|| anyhow!("Cannot find home directory"))?
                .join(".password-store")
        };

        let clip_time = std::env::var("PASSWORD_STORE_CLIP_TIME")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(45u64);

        let gpg_opts = std::env::var("PASSWORD_STORE_GPG_OPTS")
            .ok()
            .map(|s| s.split_whitespace().map(String::from).collect())
            .unwrap_or_default();

        let generated_length = std::env::var("PASSWORD_STORE_GENERATED_LENGTH")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(25usize);

        let character_set = std::env::var("PASSWORD_STORE_CHARACTER_SET").ok();
        let character_set_no_symbols =
            std::env::var("PASSWORD_STORE_CHARACTER_SET_NO_SYMBOLS").ok();

        Ok(Config {
            store_dir,
            clip_time,
            gpg_opts,
            generated_length,
            character_set,
            character_set_no_symbols,
        })
    }
}
