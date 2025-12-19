use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub provider: String,
    pub api_key: String,
    pub model: String,
    pub base_url: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            provider: "anthropic".to_string(),
            api_key: "".to_string(),
            model: "claude-3-sonnet-20240229".to_string(),
            base_url: None,
        }
    }
}

pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    pub fn new(app_handle: &AppHandle) -> Self {
        let mut path = app_handle.path().app_config_dir().unwrap_or_else(|_| PathBuf::from("."));
        // Ensure directory exists
        let _ = fs::create_dir_all(&path);
        path.push("config.json");
        Self { config_path: path }
    }

    pub fn load(&self) -> Config {
        if let Ok(content) = fs::read_to_string(&self.config_path) {
            if let Ok(config) = serde_json::from_str(&content) {
                return config;
            }
        }
        Config::default()
    }

    pub fn save(&self, config: &Config) -> Result<(), String> {
        let content = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
        fs::write(&self.config_path, content).map_err(|e| e.to_string())?;
        Ok(())
    }
}
