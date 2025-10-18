use std::sync::{Arc, Mutex};

use calamine::{open_workbook_auto, Reader};
use mysql::{prelude::Queryable, Pool};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use tauri_plugin_dialog::DialogExt;
use toml_edit::Formatted;

use crate::{
    app::settings::update_config,
    errors::{AppError, AppResult},
    utils::{
        config::{load_or_create_config, update_config_field, Config},
        metabase::{add_db, create_first_user, sync_database},
        mysql::{add_exel_table, delete_table},
    },
};

mod app;
mod errors;
mod utils;

#[derive(Serialize)]
struct FileData {
    file_path: String,
    sheets: Vec<String>,
}

#[tauri::command]
async fn choose_file(app_handle: AppHandle) -> AppResult<FileData> {
    let file_path = app_handle
        .dialog()
        .file()
        .blocking_pick_file()
        .ok_or(AppError::FileDialog("Невдалося обрати файл"))?
        .to_string();

    let workbook = open_workbook_auto(&file_path)
        .map_err(|_| AppError::Exel("Невдалося відкрити обраний файл"))?;
    let sheets = workbook.sheet_names();

    Ok(FileData { file_path, sheets })
}

#[derive(Deserialize)]
struct ActionData {
    action: Action,
    file_path: String,
    sheet: String,
    range: String,
    table_name: String,
}
#[derive(Deserialize)]
enum Action {
    Create,
    Rewrite,
    Append,
    Delete,
}

#[tauri::command]
async fn execute_action(action_data: ActionData, app_handle: AppHandle) -> AppResult<()> {
    let pool = app_handle
        .try_state::<Arc<Mutex<Pool>>>()
        .ok_or(AppError::DbConnErr)?
        .inner()
        .clone();

    match action_data.action {
        Action::Create => {
            add_exel_table(
                &action_data.file_path,
                &action_data.sheet,
                &action_data.range,
                &action_data.table_name,
                pool,
            )
            .await?;
        }
        Action::Rewrite => {}
        Action::Append => {}
        Action::Delete => delete_table(&action_data.table_name, pool).await?,
    }

    let config = load_or_create_config().await?;
    sync_database(config.ip_address, &config.db_name).await?;
    Ok(())
}

#[tauri::command]
async fn connect_to_db(server_ip: String, app_handle: AppHandle) -> AppResult<Vec<String>> {
    let config = load_or_create_config().await?;
    let db_url = format!(
        "mysql://{}:{}@{}/{}",
        config.db_username, config.db_password, server_ip, config.db_name
    );
    let pool = Pool::new(db_url.as_str()).map_err(|_| AppError::DbConnErr)?;
    let mut conn = pool.get_conn().map_err(|_| AppError::DbConnErr)?;
    let tables: Vec<String> = conn.query("SHOW TABLES").unwrap();
    if !config.is_initialized {
        create_first_user(&server_ip).await?;
        add_db(&server_ip).await?;
        update_config_field(
            "is_initialized",
            toml_edit::Value::Boolean(Formatted::new(true)),
        )
        .await?;
    }
    update_config_field(
        "ip_address",
        toml_edit::Value::String(Formatted::new(server_ip)),
    )
    .await?;
    app_handle.manage(Arc::new(Mutex::new(pool)));
    Ok(tables)
}

#[tauri::command]
async fn get_tables(app_handle: AppHandle) -> AppResult<Vec<String>> {
    let pool = app_handle.state::<Arc<Mutex<Pool>>>().inner().clone();
    let mut conn = pool
        .lock()
        .unwrap()
        .get_conn()
        .map_err(|_| AppError::DbConnErr)?;
    let tables: Vec<String> = conn
        .query("SHOW TABLES")
        .map_err(|_| AppError::Db("Невдалося прочитати таблиці"))?;
    Ok(tables)
}

#[tauri::command]
async fn get_config() -> AppResult<Config> {
    let config = load_or_create_config().await?;
    Ok(config)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            choose_file,
            execute_action,
            connect_to_db,
            get_tables,
            get_config,
            update_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
