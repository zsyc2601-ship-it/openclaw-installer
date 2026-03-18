use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use super::env_detect;

#[derive(Debug, Serialize, Deserialize, Default)]
struct OpenClawConfig {
    #[serde(default)]
    api_keys: HashMap<String, String>,
}

fn config_path() -> Result<PathBuf, String> {
    let env = env_detect::detect()?;
    let dir = env.openclaw_config_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("Cannot create config dir: {}", e))?;
    Ok(dir.join("config.json"))
}

fn read_config() -> Result<OpenClawConfig, String> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(OpenClawConfig::default());
    }
    let content = fs::read_to_string(&path).map_err(|e| format!("Cannot read config: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("Invalid config JSON: {}", e))
}

fn write_config(config: &OpenClawConfig) -> Result<(), String> {
    let path = config_path()?;
    let content =
        serde_json::to_string_pretty(config).map_err(|e| format!("JSON serialize error: {}", e))?;
    fs::write(&path, content).map_err(|e| format!("Cannot write config: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn save_api_key(provider: String, key: String) -> Result<(), String> {
    let mut config = read_config()?;
    config.api_keys.insert(provider, key);
    write_config(&config)?;
    log::info!("API key saved");
    Ok(())
}

#[tauri::command]
pub fn load_config() -> Result<HashMap<String, String>, String> {
    let config = read_config()?;
    Ok(config.api_keys)
}
