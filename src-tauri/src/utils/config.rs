use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};
use toml_edit::{value, DocumentMut, Value};

use crate::errors::{AppError, AppResult};

#[derive(PartialEq, Deserialize, Serialize, Clone, Debug)]
pub struct Config {
    pub ip_address: String,
    pub db_name: String,
    pub db_username: String,
    pub db_password: String,
    pub mt_username: String,
    pub mt_password: String,
    pub is_initialized: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ip_address: "".to_string(),
            db_name: "mydb".to_string(),
            db_username: "user".to_string(),
            db_password: "password".to_string(),
            mt_username: "user@gmail.com".to_string(),
            mt_password: "qwerty1234!".to_string(),
            is_initialized: false,
        }
    }
}

pub async fn load_or_create_config() -> AppResult<Config> {
    let mut path = dirs::config_dir().unwrap_or(PathBuf::from("."));
    path.push("config.toml");
    if !path.exists() {
        let default = Config::default();
        let toml_str = toml::to_string_pretty(&default)
            .map_err(|_| AppError::Config("Невірний формат конфігураційного файлу"))?;
        fs::write(&path, toml_str)
            .map_err(|_| AppError::Config("Невдалося записати конфігураційний файл"))?;
        return Ok(default);
    }

    let content = fs::read_to_string(&path)
        .map_err(|_| AppError::Config("Невдалося прочитати конфігураційний файл"))?;
    Ok(toml::from_str::<Config>(&content)
        .map_err(|_| AppError::Config("Невірний формат конфігураційного файлу"))?)
}

pub async fn update_config_field(key: &str, new_value: Value) -> AppResult<()> {
    let mut path = dirs::config_dir().unwrap_or(PathBuf::from("."));
    path.push("config.toml");
    let content = fs::read_to_string(&path)
        .map_err(|_| AppError::Config("Невдалося прочитати конфігураційний файл"))?;
    let mut doc = content
        .parse::<DocumentMut>()
        .map_err(|_| AppError::Config("Невірний формат кофігураційного файлу"))?;
    doc[key] = value(new_value);
    fs::write(&path, doc.to_string())
        .map_err(|_| AppError::Config("Невдалося записати конфігураційний файл"))?;
    Ok(())
}
