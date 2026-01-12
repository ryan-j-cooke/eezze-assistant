use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

use dirs_next::config_dir;
use serde::{Deserialize, Serialize};

const CONFIG_DIR_NAME: &str = "eezze";
const CONFIG_FILE_NAME: &str = "config.json";

/// Simple model config that mirrors the four default constants.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EezzeConfig {
    pub expert_recursive_local: String,
    pub expert_fast_model: String,
    pub expert_reviewer_model: String,
    pub expert_embedding_default: String,
}

impl Default for EezzeConfig {
    fn default() -> Self {
        Self {
            expert_recursive_local: "qwen2.5:3b".to_string(),
            expert_fast_model: "qwen2.5:1.5b".to_string(),
            expert_reviewer_model: "qwen2.5:0.5b".to_string(),
            expert_embedding_default: "nomic-embed-text".to_string(),
        }
    }
}

pub fn config_path() -> io::Result<PathBuf> {
    let base = config_dir().ok_or_else(|| {
        io::Error::new(io::ErrorKind::Other, "Could not determine OS config directory")
    })?;

    Ok(base.join(CONFIG_DIR_NAME).join(CONFIG_FILE_NAME))
}

pub fn ensure_config_exists() -> io::Result<EezzeConfig> {
    let path = config_path()?;
    if path.exists() {
        return load_config();
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let cfg = EezzeConfig::default();
    let json = serde_json::to_string_pretty(&cfg)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    let mut file = fs::File::create(&path)?;
    file.write_all(json.as_bytes())?;

    Ok(cfg)
}

pub fn load_config() -> io::Result<EezzeConfig> {
    let path = config_path()?;
    let mut file = fs::File::open(&path)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;

    let cfg: EezzeConfig = serde_json::from_str(&buf)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    Ok(cfg)
}

pub fn save_config(cfg: &EezzeConfig) -> io::Result<()> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(cfg)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    let mut file = fs::File::create(&path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}
