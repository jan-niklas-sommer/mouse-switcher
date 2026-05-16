use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub speed: u32,
    pub enhance_precision: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub toggle: String,
    pub speed_up: String,
    pub speed_down: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub normal: Profile,
    pub gaming: Profile,
    pub hotkey: HotkeyConfig,
}

impl Config {
    pub fn default_values() -> Self {
        Config {
            normal: Profile {
                speed: 10,
                enhance_precision: true,
            },
            gaming: Profile {
                speed: 4,
                enhance_precision: false,
            },
            hotkey: HotkeyConfig {
                toggle: "Ctrl+Alt+KeyM".to_string(),
                speed_up: "Ctrl+Alt+ArrowUp".to_string(),
                speed_down: "Ctrl+Alt+ArrowDown".to_string(),
            },
        }
    }

    pub fn config_path() -> PathBuf {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()));

        if let Some(dir) = exe_dir {
            let path = dir.join("settings.toml");
            if path.exists() {
                return path;
            }
        }

        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("mouse-switcher")
            .join("settings.toml")
    }

    pub fn load() -> Result<Self, String> {
        let path = Self::config_path();
        if !path.exists() {
            let config = Self::default_values();
            config.save()?;
            return Ok(config);
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read config: {e}"))?;
        toml::from_str(&content).map_err(|e| format!("Failed to parse config: {e}"))
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config dir: {e}"))?;
        }
        let content =
            toml::to_string_pretty(self).map_err(|e| format!("Failed to serialize config: {e}"))?;
        fs::write(&path, content).map_err(|e| format!("Failed to write config: {e}"))
    }
}
