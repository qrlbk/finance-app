//! Budget commands.

use super::common::{with_connection_and_user, AppState};
use crate::db;
use serde::Deserialize;
use tauri::State;

const VALID_PERIODS: &[&str] = &["weekly", "monthly", "yearly"];

#[tauri::command]
pub fn get_budgets(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
) -> Result<Vec<db::Budget>, String> {
    with_connection_and_user(&app_handle, &state, db::get_budgets_with_spending)
}

#[derive(Deserialize)]
pub struct CreateBudgetInput {
    category_id: i64,
    amount: f64,
    period: String,
}

#[derive(Deserialize)]
pub struct UpdateBudgetInput {
    id: i64,
    amount: f64,
}

#[tauri::command]
pub fn create_budget(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    input: CreateBudgetInput,
) -> Result<i64, String> {
    if input.amount <= 0.0 || input.amount.is_nan() {
        return Err("Сумма бюджета должна быть больше нуля".to_string());
    }
    if !VALID_PERIODS.contains(&input.period.as_str()) {
        return Err("Недопустимый период".to_string());
    }
    with_connection_and_user(&app_handle, &state, |conn, user_id| {
        db::create_budget(conn, user_id, input.category_id, input.amount, &input.period)
    })
}

#[tauri::command]
pub fn update_budget(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    input: UpdateBudgetInput,
) -> Result<(), String> {
    if input.amount <= 0.0 || input.amount.is_nan() {
        return Err("Сумма бюджета должна быть больше нуля".to_string());
    }
    with_connection_and_user(&app_handle, &state, |conn, user_id| {
        db::update_budget(conn, user_id, input.id, input.amount)
    })
}

#[tauri::command]
pub fn delete_budget(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    id: i64,
) -> Result<(), String> {
    with_connection_and_user(&app_handle, &state, |conn, user_id| db::delete_budget(conn, user_id, id))
}

#[tauri::command]
pub fn get_budget_alerts(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
) -> Result<Vec<db::BudgetAlert>, String> {
    with_connection_and_user(&app_handle, &state, db::check_budget_alerts)
}
