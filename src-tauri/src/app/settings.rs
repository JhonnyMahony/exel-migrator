use std::fs;
use std::path::PathBuf;

use tauri;

use crate::errors::{AppError, AppResult};
use crate::utils::config::Config;

#[tauri::command]
pub async fn update_config(config: Config) -> AppResult<()> {
    let mut path = dirs::config_dir().unwrap_or(PathBuf::from("."));
    path.push("config.toml");
    let toml_str = toml::to_string_pretty(&config).unwrap();
    fs::write(&path, toml_str)
        .map_err(|_| AppError::Config("Невдалося оновити кофігураційний файл"))?;
    Ok(())
}
