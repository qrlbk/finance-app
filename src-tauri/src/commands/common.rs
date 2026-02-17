//! Shared state, cache, and DB connection helpers for Tauri commands.

use crate::db::{self, ensure_encrypted, get_db_path, open_connection};
use crate::messages;
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;
use std::time::{Duration, Instant};
use tauri::State;

/// Cache entry with expiration
struct CacheEntry<T> {
    data: T,
    expires_at: Instant,
}

impl<T: Clone> CacheEntry<T> {
    fn new(data: T, ttl: Duration) -> Self {
        CacheEntry {
            data,
            expires_at: Instant::now() + ttl,
        }
    }

    fn is_valid(&self) -> bool {
        Instant::now() < self.expires_at
    }

    fn get(&self) -> Option<T> {
        if self.is_valid() {
            Some(self.data.clone())
        } else {
            None
        }
    }
}

/// Simple in-memory cache for frequently accessed data
pub struct QueryCache {
    categories: RwLock<Option<CacheEntry<Vec<db::Category>>>>,
    accounts: RwLock<Option<CacheEntry<Vec<db::Account>>>>,
}

impl QueryCache {
    pub fn new() -> Self {
        QueryCache {
            categories: RwLock::new(None),
            accounts: RwLock::new(None),
        }
    }

    pub fn get_categories(&self) -> Option<Vec<db::Category>> {
        self.categories.read().ok()?.as_ref()?.get()
    }

    pub fn set_categories(&self, data: Vec<db::Category>) {
        if let Ok(mut guard) = self.categories.write() {
            *guard = Some(CacheEntry::new(data, Duration::from_secs(60)));
        }
    }

    pub fn invalidate_categories(&self) {
        if let Ok(mut guard) = self.categories.write() {
            *guard = None;
        }
    }

    pub fn get_accounts(&self) -> Option<Vec<db::Account>> {
        self.accounts.read().ok()?.as_ref()?.get()
    }

    pub fn set_accounts(&self, data: Vec<db::Account>) {
        if let Ok(mut guard) = self.accounts.write() {
            *guard = Some(CacheEntry::new(data, Duration::from_secs(30)));
        }
    }

    pub fn invalidate_accounts(&self) {
        if let Ok(mut guard) = self.accounts.write() {
            *guard = None;
        }
    }

    pub fn invalidate_all(&self) {
        self.invalidate_categories();
        self.invalidate_accounts();
    }
}

impl Default for QueryCache {
    fn default() -> Self {
        Self::new()
    }
}

pub struct AppState {
    pub db_path: std::sync::Mutex<Option<std::path::PathBuf>>,
    pub cache: QueryCache,
    pub current_user_id: std::sync::Mutex<Option<i64>>,
}

pub fn get_session_path(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
    let db_path = get_db_path(app_handle)?;
    Ok(db_path
        .parent()
        .ok_or_else(|| messages::ERR_INVALID_DB_PATH.to_string())?
        .join("current_user.json"))
}

pub fn read_stored_user_id(app_handle: &tauri::AppHandle) -> Option<i64> {
    let path = get_session_path(app_handle).ok()?;
    let s = fs::read_to_string(&path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&s).ok()?;
    v.get("user_id")?.as_i64()
}

pub fn write_stored_user_id(app_handle: &tauri::AppHandle, user_id: i64) -> Result<(), String> {
    let path = get_session_path(app_handle)?;
    let s = format!("{{\"user_id\":{}}}", user_id);
    fs::write(&path, s).map_err(|e| e.to_string())
}

pub fn clear_stored_user_id(app_handle: &tauri::AppHandle) {
    if let Ok(path) = get_session_path(app_handle) {
        let _ = fs::remove_file(path);
    }
}

pub fn init_db_on_startup(app_handle: &tauri::AppHandle, state: &AppState) -> Result<(), String> {
    let db_path = get_db_path(app_handle)?;

    ensure_encrypted(&db_path)?;

    let conn = open_connection(&db_path)?;
    db::init_db(&conn)?;
    *state.db_path.lock().unwrap() = Some(db_path.clone());

    if let Some(uid) = read_stored_user_id(app_handle) {
        let exists: bool = conn
            .query_row("SELECT 1 FROM users WHERE id = ?1", [uid], |_| Ok(()))
            .is_ok();
        if exists {
            *state.current_user_id.lock().unwrap() = Some(uid);
            // Run due recurring payments at startup so they are applied even before the user opens the app
            if let Ok(created) = db::process_due_recurring(&conn, uid) {
                if !created.is_empty() {
                    tracing::info!("At startup: processed {} recurring payment(s)", created.len());
                }
            }
        } else {
            clear_stored_user_id(app_handle);
        }
    }

    Ok(())
}

pub fn with_connection<F, T>(
    app_handle: &tauri::AppHandle,
    state: &State<AppState>,
    f: F,
) -> Result<T, String>
where
    F: FnOnce(&rusqlite::Connection) -> Result<T, String>,
{
    let db_path = {
        let guard = state.db_path.lock().unwrap();
        guard.clone()
    };
    let db_path = db_path.or_else(|| get_db_path(app_handle).ok());
    let db_path = db_path.ok_or_else(|| "Не удалось определить путь к базе данных".to_string())?;
    let conn = open_connection(&db_path)?;
    f(&conn)
}

pub const ERR_LOGIN_REQUIRED: &str = "Необходимо войти в систему";

pub fn with_connection_and_user<F, T>(
    app_handle: &tauri::AppHandle,
    state: &State<AppState>,
    f: F,
) -> Result<T, String>
where
    F: FnOnce(&rusqlite::Connection, i64) -> Result<T, String>,
{
    let user_id = *state.current_user_id.lock().unwrap();
    let uid = user_id.ok_or_else(|| ERR_LOGIN_REQUIRED.to_string())?;
    let db_path = {
        let guard = state.db_path.lock().unwrap();
        guard.clone()
    };
    let db_path = db_path.or_else(|| get_db_path(app_handle).ok());
    let db_path = db_path.ok_or_else(|| "Не удалось определить путь к базе данных".to_string())?;
    let conn = open_connection(&db_path)?;
    f(&conn, uid)
}

#[cfg(test)]
pub fn setup_test_db() -> rusqlite::Connection {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    crate::db::schema::create_tables(&conn).unwrap();
    crate::db::schema::seed_categories(&conn, 1).unwrap();
    conn
}
