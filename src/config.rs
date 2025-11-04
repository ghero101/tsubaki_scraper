use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub download_dir: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            download_dir: "downloads".to_string(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let path = Path::new("config.toml");
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(cfg) = toml::from_str::<Config>(&content) {
                    return cfg;
                }
            }
        }
        Self::default()
    }
}
