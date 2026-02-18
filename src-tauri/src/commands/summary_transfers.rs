//! Summary, export backup, restore, transfers.

use super::common::{with_connection_and_user, AppState};
use crate::db;
use crate::messages;
use serde::Deserialize;
use std::fs;
use std::io::Read;
use tauri::State;

#[tauri::command]
pub fn get_summary(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
) -> Result<db::Summary, String> {
    with_connection_and_user(&app_handle, &state, db::get_summary)
}

#[tauri::command]
pub fn get_base_currency(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
) -> Result<String, String> {
    with_connection_and_user(&app_handle, &state, |conn, user_id| {
        let cur = db::get_setting(conn, user_id, "base_currency")?
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "KZT".to_string());
        Ok(cur)
    })
}

#[derive(serde::Deserialize)]
pub struct SetBaseCurrencyInput {
    base_currency: String,
}

#[tauri::command]
pub fn set_base_currency(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    input: SetBaseCurrencyInput,
) -> Result<(), String> {
    let cur = input.base_currency.trim();
    if cur.is_empty() {
        return Err("Базовая валюта не может быть пустой".to_string());
    }
    with_connection_and_user(&app_handle, &state, |conn, user_id| {
        db::set_setting(conn, user_id, "base_currency", cur)
    })
}

#[tauri::command]
pub fn get_exchange_rates(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
) -> Result<Vec<db::ExchangeRateRow>, String> {
    with_connection_and_user(&app_handle, &state, db::get_exchange_rates)
}

#[derive(serde::Deserialize)]
pub struct AddExchangeRateInput {
    from_currency: String,
    to_currency: String,
    rate: f64,
    date: String,
}

#[tauri::command]
pub fn add_exchange_rate(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    input: AddExchangeRateInput,
) -> Result<(), String> {
    with_connection_and_user(&app_handle, &state, |conn, user_id| {
        db::add_exchange_rate(
            conn,
            user_id,
            input.from_currency.trim(),
            input.to_currency.trim(),
            input.rate,
            input.date.trim(),
        )
    })
}

#[derive(Deserialize)]
pub struct GetExpenseByCategoryInput {
    year: i32,
    month: u32,
    #[serde(default)]
    include_children: bool,
}

#[tauri::command]
pub fn get_expense_by_category(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    input: GetExpenseByCategoryInput,
) -> Result<Vec<db::CategoryTotal>, String> {
    with_connection_and_user(&app_handle, &state, |conn, user_id| {
        db::get_expense_by_category(conn, user_id, input.year, input.month, input.include_children)
    })
}

#[derive(Deserialize)]
pub struct GetMonthlyTotalsInput {
    months: Option<i32>,
}

#[tauri::command]
pub fn get_monthly_totals(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    input: Option<GetMonthlyTotalsInput>,
) -> Result<Vec<db::MonthlyTotal>, String> {
    let months = input.and_then(|i| i.months).unwrap_or(6);
    with_connection_and_user(&app_handle, &state, |conn, user_id| db::get_monthly_totals(conn, user_id, months))
}

#[tauri::command]
pub fn export_backup(app_handle: tauri::AppHandle) -> Result<String, String> {
    let db_path = db::get_db_path(&app_handle)?;
    if !db_path.exists() {
        return Err("База данных не инициализирована".to_string());
    }
    let backup_path = db_path
        .parent()
        .ok_or(messages::ERR_INVALID_PATH)?
        .join(format!("finance_backup_{}.db", chrono::Local::now().format("%Y%m%d_%H%M%S")));
    fs::copy(&db_path, &backup_path).map_err(|e| e.to_string())?;
    Ok(backup_path.to_string_lossy().to_string())
}

#[derive(Deserialize)]
pub struct CreateTransferInput {
    from_account_id: i64,
    to_account_id: i64,
    amount: f64,
    date: String,
    note: Option<String>,
}

#[tauri::command]
pub fn create_transfer(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    input: CreateTransferInput,
) -> Result<(), String> {
    with_connection_and_user(&app_handle, &state, |conn, user_id| {
        db::create_transfer(
            conn,
            user_id,
            input.from_account_id,
            input.to_account_id,
            input.amount,
            &input.date,
            input.note.as_deref(),
        )
    })
}

#[derive(Deserialize)]
pub struct GetTransfersInput {
    limit: Option<i64>,
}

#[tauri::command]
pub fn get_transfers(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    input: Option<GetTransfersInput>,
) -> Result<Vec<db::TransferWithDetails>, String> {
    let limit = input.and_then(|i| i.limit).unwrap_or(50);
    with_connection_and_user(&app_handle, &state, |conn, user_id| db::get_transfers(conn, user_id, limit))
}

#[tauri::command]
pub fn restore_backup(app_handle: tauri::AppHandle, path: String) -> Result<(), String> {
    let backup_path = std::path::Path::new(&path);
    if !backup_path.exists() {
        return Err("Файл не найден".to_string());
    }
    if !backup_path.is_file() {
        return Err("Указанный путь не является файлом".to_string());
    }
    let mut header = [0u8; 16];
    let mut f = fs::File::open(backup_path).map_err(|e| e.to_string())?;
    f.read_exact(&mut header).map_err(|e| e.to_string())?;
    if !header.starts_with(b"SQLite format 3\0") {
        return Err("Файл не похож на базу SQLite".to_string());
    }

    let db_path = db::get_db_path(&app_handle)?;
    let old_path = db_path.parent().ok_or(messages::ERR_INVALID_PATH)?.join("finance.db.old");
    if db_path.exists() {
        fs::rename(&db_path, &old_path).map_err(|e| e.to_string())?;
    }
    fs::copy(backup_path, &db_path).map_err(|e| e.to_string())?;

    // Проверяем, что восстановленную БД можно открыть с текущим ключом (SQLCipher)
    if let Err(e) = db::open_connection(&db_path) {
        // Откат: восстанавливаем старую БД
        let _ = std::fs::remove_file(&db_path);
        if old_path.exists() {
            let _ = fs::rename(&old_path, &db_path);
        }
        return Err(format!(
            "{} {}",
            messages::ERR_RESTORE_BACKUP_OPEN,
            e
        ));
    }

    Ok(())
}
