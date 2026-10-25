use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct Config {
    pub theme: String,
    pub follow_palette: bool,
    pub palette_file: Option<PathBuf>,
    pub colors: BTreeMap<String, String>,
    pub font: Option<String>,
    pub lyrics_font: Option<String>,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            theme: "dark".into(),
            follow_palette: true,
            palette_file: None,
            colors: BTreeMap::new(),
            font: None,
            lyrics_font: None,
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> Config {
        let Ok(text) = fs::read_to_string(path) else {
            return Config::default();
        };
        match toml::from_str(&text) {
            Ok(config) => config,
            Err(err) => {
                tracing::info!("ignoring {}: {}", path.display(), err.message());
                Config::default()
            }
        }
    }
}

pub fn palette_path(config: &Config, env: Option<&OsStr>, home: &Path) -> PathBuf {
    if let Some(env) = env.filter(|e| !e.is_empty()) {
        return PathBuf::from(env);
    }
    if let Some(path) = &config.palette_file {
        return path.clone();
    }
    home.join(".config/rice/palette.json")
}
