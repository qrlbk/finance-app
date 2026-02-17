//! Auth commands: register, login, logout, session.

use super::common::{clear_stored_user_id, write_stored_user_id, with_connection, AppState};
use crate::auth;
use crate::db;
use serde::Deserialize;
use tauri::State;

#[derive(Deserialize)]
pub struct RegisterArgs {
    username: String,
    password: String,
    #[serde(rename = "displayName")]
    display_name: String,
}

#[tauri::command]
pub fn register(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    args: RegisterArgs,
) -> Result<db::UserInfo, String> {
    let RegisterArgs { username, password, display_name } = args;
    auth::validate_username(username.trim())?;
    auth::validate_password(&password)?;
    auth::validate_display_name(display_name.trim())?;
    let username = username.trim().to_string();
    let display_name = display_name.trim().to_string();

    let user_info = with_connection(&app_handle, &state, |conn| {
        if db::get_user_by_username(conn, &username)?.is_some() {
            return Err("Пользователь с таким именем уже существует".to_string());
        }
        let password_hash = auth::hash_password(&password)?;
        let user_id = db::create_user(conn, &username, &password_hash, &display_name)?;
        db::seed_categories(conn, user_id)?;
        Ok(db::UserInfo {
            id: user_id,
            username: username.clone(),
            display_name: display_name.clone(),
        })
    })?;

    write_stored_user_id(&app_handle, user_info.id)?;
    *state.current_user_id.lock().unwrap() = Some(user_info.id);
    state.cache.invalidate_all();
    Ok(user_info)
}

#[tauri::command]
pub fn login(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    username: String,
    password: String,
) -> Result<db::UserInfo, String> {
    let username = username.trim();
    if username.is_empty() {
        return Err("Введите имя пользователя".to_string());
    }

    let user_info = with_connection(&app_handle, &state, |conn| {
        let Some((id, hash, display_name)) = db::get_user_by_username(conn, username)? else {
            return Err("Неверное имя пользователя или пароль".to_string());
        };
        if !auth::verify_password(&password, &hash)? {
            return Err("Неверное имя пользователя или пароль".to_string());
        }
        Ok(db::UserInfo {
            id,
            username: username.to_string(),
            display_name,
        })
    })?;

    write_stored_user_id(&app_handle, user_info.id)?;
    *state.current_user_id.lock().unwrap() = Some(user_info.id);
    state.cache.invalidate_all();
    Ok(user_info)
}

#[tauri::command]
pub fn logout(app_handle: tauri::AppHandle, state: State<AppState>) -> Result<(), String> {
    clear_stored_user_id(&app_handle);
    *state.current_user_id.lock().unwrap() = None;
    state.cache.invalidate_all();
    Ok(())
}

#[tauri::command]
pub fn get_current_session(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
) -> Result<Option<db::UserInfo>, String> {
    let user_id = *state.current_user_id.lock().unwrap();
    let Some(uid) = user_id else {
        return Ok(None);
    };
    with_connection(&app_handle, &state, |conn| db::get_user_by_id(conn, uid))
}

#[tauri::command]
pub fn list_users(app_handle: tauri::AppHandle, state: State<AppState>) -> Result<Vec<db::UserInfo>, String> {
    with_connection(&app_handle, &state, db::list_users)
}
