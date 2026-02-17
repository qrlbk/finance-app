//! Category commands: get, create, update, delete.

use super::common::{with_connection_and_user, AppState};
use super::transactions::VALID_TRANSACTION_TYPES;
use crate::db;
use serde::Deserialize;
use tauri::State;

#[tauri::command]
pub fn get_categories(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
) -> Result<Vec<db::Category>, String> {
    if let Some(cached) = state.cache.get_categories() {
        return Ok(cached);
    }
    let categories = with_connection_and_user(&app_handle, &state, db::get_categories)?;
    state.cache.set_categories(categories.clone());
    Ok(categories)
}

#[derive(Deserialize)]
pub struct CreateCategoryInput {
    name: String,
    category_type: String,
    icon: Option<String>,
    color: Option<String>,
    parent_id: Option<i64>,
}

#[derive(Deserialize)]
pub struct UpdateCategoryInput {
    id: i64,
    name: String,
    icon: Option<String>,
    color: Option<String>,
}

#[tauri::command]
pub fn create_category(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    input: CreateCategoryInput,
) -> Result<i64, String> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err("Название категории не может быть пустым".to_string());
    }
    if !VALID_TRANSACTION_TYPES.contains(&input.category_type.as_str()) {
        return Err("Недопустимый тип категории".to_string());
    }
    let result = with_connection_and_user(&app_handle, &state, |conn, user_id| {
        db::create_category(
            conn,
            user_id,
            name,
            &input.category_type,
            input.icon.as_deref(),
            input.color.as_deref(),
            input.parent_id,
        )
    })?;
    state.cache.invalidate_categories();
    Ok(result)
}

#[tauri::command]
pub fn update_category(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    input: UpdateCategoryInput,
) -> Result<(), String> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err("Название категории не может быть пустым".to_string());
    }
    let result = with_connection_and_user(&app_handle, &state, |conn, user_id| {
        db::update_category(conn, user_id, input.id, name, input.icon.as_deref(), input.color.as_deref())
    })?;
    state.cache.invalidate_categories();
    Ok(result)
}

#[tauri::command]
pub fn delete_category(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    id: i64,
) -> Result<(), String> {
    let result = with_connection_and_user(&app_handle, &state, |conn, user_id| db::delete_category(conn, user_id, id))?;
    state.cache.invalidate_categories();
    Ok(result)
}
