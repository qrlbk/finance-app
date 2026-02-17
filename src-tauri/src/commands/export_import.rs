//! Export data, import data, import preview, open file, reset database.

use super::common::{with_connection_and_user, AppState};
use crate::db;
use crate::export;
use crate::messages;
use serde::Deserialize;
use std::path::PathBuf;
use tauri::State;

#[derive(Deserialize)]
pub struct ExportDataInput {
    format: String,
    date_from: Option<String>,
    date_to: Option<String>,
    include_accounts: bool,
    include_categories: bool,
    account_id: Option<i64>,
    category_id: Option<i64>,
}

#[tauri::command]
pub fn export_data(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    input: ExportDataInput,
) -> Result<String, String> {
    let db_path = db::get_db_path(&app_handle)?;
    let export_dir = db_path.parent().ok_or(messages::ERR_INVALID_PATH)?;
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let extension = match input.format.as_str() {
        "csv" => "csv",
        "json" => "json",
        "xlsx" => "xlsx",
        _ => return Err(messages::ERR_UNSUPPORTED_FORMAT.to_string()),
    };
    let filename = format!("finance_export_{}.{}", timestamp, extension);
    let output_path = export_dir.join(&filename);
    let options = export::ExportOptions {
        format: input.format,
        date_from: input.date_from,
        date_to: input.date_to,
        include_accounts: input.include_accounts,
        include_categories: input.include_categories,
        account_id: input.account_id,
        category_id: input.category_id,
    };
    with_connection_and_user(&app_handle, &state, |conn, user_id| {
        export::export_data(conn, user_id, &options, &output_path)?;
        Ok(output_path.to_string_lossy().to_string())
    })
}

#[derive(Deserialize, Default)]
pub struct ImportDataInput {
    path: String,
    format: String,
    /// Счёт по умолчанию, если в файле счёт не указан или не найден.
    pub default_account_id: Option<i64>,
    /// Пропускать дубликаты (счёт, дата, сумма, заметка).
    #[serde(default)]
    pub skip_duplicates: bool,
}

#[tauri::command]
pub fn import_data(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    input: ImportDataInput,
) -> Result<export::ImportResult, String> {
    let file_path = PathBuf::from(&input.path);
    if !file_path.exists() {
        return Err(messages::ERR_FILE_NOT_FOUND.to_string());
    }
    let options = export::ImportOptions {
        default_account_id: input.default_account_id,
        skip_duplicates: input.skip_duplicates,
    };
    with_connection_and_user(&app_handle, &state, |conn, user_id| {
        match input.format.as_str() {
            "csv" => export::import_csv(conn, user_id, &file_path, &options),
            "json" => export::import_json(conn, user_id, &file_path, &options),
            "xlsx" => export::import_xlsx(conn, user_id, &file_path, &options),
            _ => Err(messages::ERR_UNSUPPORTED_FORMAT.to_string()),
        }
    })
}

#[tauri::command]
pub fn import_preview(path: String, format: String) -> Result<export::ImportPreview, String> {
    let file_path = PathBuf::from(&path);
    if !file_path.exists() {
        return Err(messages::ERR_FILE_NOT_FOUND.to_string());
    }
    export::import_preview(&file_path, &format)
}

#[tauri::command]
pub fn open_file(path: String) -> Result<(), String> {
    let path = PathBuf::from(&path);
    if !path.exists() {
        return Err("Файл не найден".to_string());
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(&path).spawn().map_err(|e| format!("Не удалось открыть файл: {}", e))?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", path.to_str().unwrap_or("")])
            .spawn()
            .map_err(|e| format!("Не удалось открыть файл: {}", e))?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(&path).spawn().map_err(|e| format!("Не удалось открыть файл: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub fn reset_database(app_handle: tauri::AppHandle, state: State<AppState>) -> Result<(), String> {
    with_connection_and_user(&app_handle, &state, |conn, user_id| {
        db::clear_all_data(conn, user_id)?;
        db::seed_categories(conn, user_id)?;
        Ok(())
    })?;
    state.cache.invalidate_all();
    Ok(())
}
