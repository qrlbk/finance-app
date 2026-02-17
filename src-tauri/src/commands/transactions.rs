//! Transaction commands: get, create, update, delete.

use super::common::{with_connection_and_user, AppState};
use crate::db;
use crate::ml;
use chrono::NaiveDate;
use serde::Deserialize;
use tauri::State;

pub const VALID_TRANSACTION_TYPES: &[&str] = &["income", "expense"];

#[derive(Deserialize, Default)]
pub struct GetTransactionsInput {
    limit: Option<i64>,
    offset: Option<i64>,
    account_id: Option<i64>,
    date_from: Option<String>,
    date_to: Option<String>,
    category_id: Option<i64>,
    uncategorized_only: Option<bool>,
    transaction_type: Option<String>,
    search_note: Option<String>,
}

#[tauri::command]
pub fn get_transactions(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    input: Option<GetTransactionsInput>,
) -> Result<Vec<db::TransactionWithDetails>, String> {
    with_connection_and_user(&app_handle, &state, |conn, user_id| {
        let i = input.unwrap_or_default();
        let filters = db::TransactionFilters {
            limit: i.limit.unwrap_or(100),
            offset: i.offset.unwrap_or(0),
            date_from: i.date_from,
            date_to: i.date_to,
            account_id: i.account_id,
            category_id: i.category_id,
            uncategorized_only: i.uncategorized_only.unwrap_or(false),
            transaction_type: i.transaction_type,
            search_note: i.search_note,
        };
        db::get_transactions_filtered(conn, user_id, filters)
    })
}

#[derive(Deserialize)]
pub struct CreateTransactionInput {
    account_id: i64,
    category_id: Option<i64>,
    amount: f64,
    transaction_type: String,
    note: Option<String>,
    date: String,
}

#[derive(Deserialize)]
pub struct UpdateTransactionInput {
    id: i64,
    account_id: i64,
    category_id: Option<i64>,
    amount: f64,
    transaction_type: String,
    note: Option<String>,
    date: String,
}

pub fn validate_transaction_input(
    conn: &rusqlite::Connection,
    user_id: i64,
    account_id: i64,
    category_id: Option<i64>,
    amount: f64,
    transaction_type: &str,
    date: &str,
) -> Result<(), String> {
    if amount <= 0.0 || amount.is_nan() {
        return Err("Сумма должна быть больше нуля".to_string());
    }
    if !VALID_TRANSACTION_TYPES.contains(&transaction_type) {
        return Err("Недопустимый тип транзакции".to_string());
    }
    NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map_err(|_| "Некорректная дата".to_string())?;
    if !db::account_exists(conn, user_id, account_id)? {
        return Err("Счёт не найден".to_string());
    }
    if let Some(cid) = category_id {
        if !db::category_exists_and_type(conn, user_id, cid, transaction_type)? {
            return Err("Категория не найдена или не совпадает с типом транзакции".to_string());
        }
    }
    Ok(())
}

#[tauri::command]
pub fn create_transaction(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    input: CreateTransactionInput,
) -> Result<i64, String> {
    with_connection_and_user(&app_handle, &state, |conn, user_id| {
        validate_transaction_input(
            conn,
            user_id,
            input.account_id,
            input.category_id,
            input.amount,
            &input.transaction_type,
            &input.date,
        )?;
        let id = db::create_transaction(
            conn,
            user_id,
            input.account_id,
            input.category_id,
            input.amount,
            &input.transaction_type,
            input.note.as_deref(),
            &input.date,
        )?;
        if let (Some(ref note), Some(cat_id)) = (input.note.as_ref(), input.category_id) {
            let note = note.trim();
            if !note.is_empty() && ml::rules::is_note_length_ok(note) {
                let _ = ml::rules::upsert_rule(conn, user_id, note, cat_id);
            }
        }
        Ok(id)
    })
}

#[tauri::command]
pub fn update_transaction(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    input: UpdateTransactionInput,
) -> Result<(), String> {
    with_connection_and_user(&app_handle, &state, |conn, user_id| {
        validate_transaction_input(
            conn,
            user_id,
            input.account_id,
            input.category_id,
            input.amount,
            &input.transaction_type,
            &input.date,
        )?;
        db::update_transaction(
            conn,
            user_id,
            input.id,
            input.account_id,
            input.category_id,
            input.amount,
            &input.transaction_type,
            input.note.as_deref(),
            &input.date,
        )?;
        if let (Some(ref note), Some(cat_id)) = (input.note.as_ref(), input.category_id) {
            let note = note.trim();
            if !note.is_empty() && ml::rules::is_note_length_ok(note) {
                let _ = ml::rules::upsert_rule(conn, user_id, note, cat_id);
            }
        }
        Ok(())
    })
}

#[tauri::command]
pub fn delete_transaction(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    id: i64,
) -> Result<(), String> {
    with_connection_and_user(&app_handle, &state, |conn, user_id| db::delete_transaction(conn, user_id, id))
}
