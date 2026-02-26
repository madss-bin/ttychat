use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerProfile {
    pub server: String,
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub profiles: Vec<ServerProfile>,
    pub last_server: Option<String>,
    pub last_username: Option<String>,
}

impl Config {
    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("ttychat")
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.json")
    }

    pub fn key_path(username: &str) -> PathBuf {
        let safe_name = username.replace(|c: char| !c.is_alphanumeric(), "_");
        let filename = if safe_name.is_empty() {
            "identity.key".to_string()
        } else {
            format!("identity_{}.key", safe_name)
        };
        let specific_path = Self::config_dir().join(&filename);
        let legacy_path = Self::config_dir().join("identity.key");

        if !specific_path.exists() && legacy_path.exists() && filename != "identity.key" {
            let _ = fs::rename(&legacy_path, &specific_path);
        }

        specific_path
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(data) = fs::read_to_string(&path) {
                if let Ok(cfg) = serde_json::from_str(&data) {
                    return cfg;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<()> {
        let dir = Self::config_dir();
        fs::create_dir_all(&dir)?;
        let json = serde_json::to_string_pretty(self)?;
        fs::write(Self::config_path(), json)?;
        Ok(())
    }
}
