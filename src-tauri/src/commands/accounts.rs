//! Account commands: CRUD and reassign transactions.

use super::common::{with_connection_and_user, AppState};
use crate::db;
use serde::Deserialize;
use tauri::State;

pub const VALID_ACCOUNT_TYPES: &[&str] = &["cash", "card", "savings"];
const MAX_CURRENCY_LEN: usize = 10;

#[tauri::command]
pub fn get_accounts(app_handle: tauri::AppHandle, state: State<AppState>) -> Result<Vec<db::Account>, String> {
    if let Some(cached) = state.cache.get_accounts() {
        return Ok(cached);
    }
    let accounts = with_connection_and_user(&app_handle, &state, db::get_accounts)?;
    state.cache.set_accounts(accounts.clone());
    Ok(accounts)
}

#[derive(Deserialize)]
pub struct CreateAccountInput {
    name: String,
    account_type: String,
    currency: Option<String>,
    initial_balance: Option<f64>,
}

#[derive(Deserialize)]
pub struct UpdateAccountInput {
    id: i64,
    name: String,
    account_type: String,
    currency: Option<String>,
}

pub fn validate_account_input(
    name: &str,
    account_type: &str,
    currency: &str,
) -> Result<(), String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Название счёта не может быть пустым".to_string());
    }
    if !VALID_ACCOUNT_TYPES.contains(&account_type) {
        return Err("Недопустимый тип счёта".to_string());
    }
    let currency = currency.trim();
    if currency.is_empty() {
        return Err("Валюта не может быть пустой".to_string());
    }
    if currency.len() > MAX_CURRENCY_LEN {
        return Err("Слишком длинный код валюты".to_string());
    }
    Ok(())
}

#[tauri::command]
pub fn create_account(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    input: CreateAccountInput,
) -> Result<i64, String> {
    let currency = input.currency.unwrap_or_else(|| "KZT".to_string());
    validate_account_input(&input.name, &input.account_type, &currency)?;
    let initial_balance = input.initial_balance.unwrap_or(0.0);
    let name = input.name.trim().to_string();
    let result = with_connection_and_user(&app_handle, &state, |conn, user_id| {
        db::create_account(conn, user_id, &name, &input.account_type, &currency, initial_balance)
    })?;
    state.cache.invalidate_accounts();
    Ok(result)
}

#[tauri::command]
pub fn update_account(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    input: UpdateAccountInput,
) -> Result<(), String> {
    let currency = input.currency.unwrap_or_else(|| "KZT".to_string());
    validate_account_input(&input.name, &input.account_type, &currency)?;
    let name = input.name.trim().to_string();
    let result = with_connection_and_user(&app_handle, &state, |conn, user_id| {
        db::update_account(conn, user_id, input.id, &name, &input.account_type, &currency)
    })?;
    state.cache.invalidate_accounts();
    Ok(result)
}

#[tauri::command]
pub fn delete_account(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    id: i64,
) -> Result<(), String> {
    let result = with_connection_and_user(&app_handle, &state, |conn, user_id| db::delete_account(conn, user_id, id))?;
    state.cache.invalidate_accounts();
    Ok(result)
}

#[tauri::command]
pub fn reassign_transactions_to_account(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    from_account_id: i64,
    to_account_id: i64,
) -> Result<(), String> {
    let result = with_connection_and_user(&app_handle, &state, |conn, user_id| {
        db::reassign_transactions_to_account(conn, user_id, from_account_id, to_account_id)
    })?;
    state.cache.invalidate_accounts();
    Ok(result)
}
