//! Recurring payments commands.

use super::common::{with_connection_and_user, AppState};
use super::transactions::VALID_TRANSACTION_TYPES;
use crate::db;
use chrono::NaiveDate;
use serde::Deserialize;
use tauri::State;

const VALID_FREQUENCIES: &[&str] = &["daily", "weekly", "monthly", "yearly"];

#[tauri::command]
pub fn get_recurring_payments(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
) -> Result<Vec<db::RecurringPayment>, String> {
    with_connection_and_user(&app_handle, &state, db::get_recurring_payments)
}

#[derive(Deserialize)]
pub struct CreateRecurringInput {
    account_id: i64,
    category_id: Option<i64>,
    amount: f64,
    payment_type: String,
    frequency: String,
    next_date: String,
    end_date: Option<String>,
    note: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateRecurringInput {
    id: i64,
    account_id: i64,
    category_id: Option<i64>,
    amount: f64,
    payment_type: String,
    frequency: String,
    next_date: String,
    end_date: Option<String>,
    note: Option<String>,
    is_active: bool,
}

fn validate_recurring_input(
    conn: &rusqlite::Connection,
    user_id: i64,
    input: &CreateRecurringInput,
) -> Result<(), String> {
    if input.amount <= 0.0 || input.amount.is_nan() {
        return Err("Сумма должна быть больше нуля".to_string());
    }
    if !VALID_TRANSACTION_TYPES.contains(&input.payment_type.as_str()) {
        return Err("Недопустимый тип платежа".to_string());
    }
    if !VALID_FREQUENCIES.contains(&input.frequency.as_str()) {
        return Err("Недопустимая частота".to_string());
    }
    NaiveDate::parse_from_str(&input.next_date, "%Y-%m-%d")
        .map_err(|_| "Некорректная дата следующего платежа".to_string())?;
    if let Some(end) = &input.end_date {
        NaiveDate::parse_from_str(end, "%Y-%m-%d")
            .map_err(|_| "Некорректная дата окончания".to_string())?;
    }
    if !db::account_exists(conn, user_id, input.account_id)? {
        return Err("Счёт не найден".to_string());
    }
    if let Some(cid) = input.category_id {
        if !db::category_exists_and_type(conn, user_id, cid, &input.payment_type)? {
            return Err("Категория не найдена или не совпадает с типом платежа".to_string());
        }
    }
    Ok(())
}

#[tauri::command]
pub fn create_recurring(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    input: CreateRecurringInput,
) -> Result<i64, String> {
    with_connection_and_user(&app_handle, &state, |conn, user_id| {
        validate_recurring_input(conn, user_id, &input)?;
        db::create_recurring(
            conn,
            user_id,
            input.account_id,
            input.category_id,
            input.amount,
            &input.payment_type,
            &input.frequency,
            &input.next_date,
            input.end_date.as_deref(),
            input.note.as_deref(),
        )
    })
}

#[tauri::command]
pub fn update_recurring(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    input: UpdateRecurringInput,
) -> Result<(), String> {
    with_connection_and_user(&app_handle, &state, |conn, user_id| {
        let create_input = CreateRecurringInput {
            account_id: input.account_id,
            category_id: input.category_id,
            amount: input.amount,
            payment_type: input.payment_type.clone(),
            frequency: input.frequency.clone(),
            next_date: input.next_date.clone(),
            end_date: input.end_date.clone(),
            note: input.note.clone(),
        };
        validate_recurring_input(conn, user_id, &create_input)?;
        db::update_recurring(
            conn,
            user_id,
            input.id,
            input.account_id,
            input.category_id,
            input.amount,
            &input.payment_type,
            &input.frequency,
            &input.next_date,
            input.end_date.as_deref(),
            input.note.as_deref(),
            input.is_active,
        )
    })
}

#[tauri::command]
pub fn delete_recurring(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    id: i64,
) -> Result<(), String> {
    with_connection_and_user(&app_handle, &state, |conn, user_id| db::delete_recurring(conn, user_id, id))
}

#[tauri::command]
pub fn process_recurring_payments(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
) -> Result<Vec<i64>, String> {
    with_connection_and_user(&app_handle, &state, db::process_due_recurring)
}
