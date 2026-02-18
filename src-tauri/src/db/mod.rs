mod queries;
pub mod schema;

use rusqlite::Connection;
use std::path::PathBuf;

use crate::messages;

pub use queries::*;
pub use schema::{clear_all_data, create_tables, seed_categories};

/// Gets the database path. Uses directories crate for reliable cross-platform paths
/// (avoids issues with Tauri path resolution in dev mode).
pub fn get_db_path(_app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
    let app_data = directories::ProjectDirs::from("com", "kuralbekadilet475", "finance-app")
        .ok_or("Не удалось определить директорию данных приложения")?
        .data_dir()
        .to_path_buf();
    std::fs::create_dir_all(&app_data).map_err(|e| e.to_string())?;
    Ok(app_data.join("finance.db"))
}

/// Открывает зашифрованное подключение к базе данных.
///
/// Использует SQLCipher для шифрования базы данных с параметрами:
/// - cipher_page_size = 4096
/// - kdf_iter = 256000 (количество итераций PBKDF2)
/// - cipher_hmac_algorithm = HMAC_SHA256
/// - cipher_kdf_algorithm = PBKDF2_HMAC_SHA256
pub fn open_connection(db_path: &PathBuf) -> Result<Connection, String> {
    let conn = match try_open_encrypted(db_path) {
        Ok(c) => return Ok(c),
        Err(e) => {
            if e.contains("file is not a database") || e.contains("not a database") {
                // Файл пустой, повреждён или не в формате SQLCipher — удаляем и создаём заново
                let _ = std::fs::remove_file(db_path);
                let _ = std::fs::remove_file(db_path.with_extension("db-wal"));
                let _ = std::fs::remove_file(db_path.with_extension("db-shm"));
                try_open_encrypted(db_path)?
            } else {
                return Err(e);
            }
        }
    };
    Ok(conn)
}

fn try_open_encrypted(db_path: &PathBuf) -> Result<Connection, String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let key = crate::crypto::get_or_create_key()?;

    {
        let pragma_sql = format!("PRAGMA key = \"x'{}'\"", key);
        let mut stmt = conn
            .prepare(&pragma_sql)
            .map_err(|e| format!("{}: {}", messages::ERR_SQLCIPHER_CONFIGURE, e))?;
        let mut rows = stmt
            .query([])
            .map_err(|e| format!("{}: {}", messages::ERR_SQLCIPHER_CONFIGURE, e))?;
        while let Some(_) = rows.next().map_err(|e| format!("{}: {}", messages::ERR_SQLCIPHER_CONFIGURE, e))? {}
    }

    conn.execute_batch(
        "PRAGMA cipher_page_size = 4096;
         PRAGMA kdf_iter = 256000;
         PRAGMA cipher_hmac_algorithm = HMAC_SHA256;
         PRAGMA cipher_kdf_algorithm = PBKDF2_HMAC_SHA256;
         PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;",
    )
    .map_err(|e| format!("{}: {}", messages::ERR_SQLCIPHER_CONFIGURE, e))?;

    conn.query_row("SELECT count(*) FROM sqlite_master", [], |_| Ok(()))
        .map_err(|_| messages::ERR_INVALID_KEY_OR_CORRUPTED.to_string())?;

    Ok(conn)
}

/// Открывает незашифрованное подключение (для миграции).
pub fn open_unencrypted_connection(db_path: &PathBuf) -> Result<Connection, String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
        .map_err(|e| e.to_string())?;
    Ok(conn)
}

/// Проверяет, является ли база данных зашифрованной.
pub fn is_database_encrypted(db_path: &PathBuf) -> bool {
    if !db_path.exists() {
        return false;
    }

    // Пытаемся открыть без ключа и выполнить запрос
    if let Ok(conn) = Connection::open(db_path) {
        // Если получится выполнить запрос - БД не зашифрована
        conn.query_row("SELECT count(*) FROM sqlite_master", [], |_| Ok(()))
            .is_err()
    } else {
        false
    }
}

pub fn init_db(conn: &Connection) -> Result<(), String> {
    schema::create_tables(conn)?;
    // Seed default categories for every user that has none (e.g. new default user after migration)
    let mut stmt = conn
        .prepare("SELECT id FROM users")
        .map_err(|e| e.to_string())?;
    let user_ids: Vec<i64> = stmt
        .query_map([], |r| r.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .collect();
    for user_id in user_ids {
        let _ = schema::seed_categories(conn, user_id);
    }
    Ok(())
}

/// Мигрирует незашифрованную базу данных в зашифрованную.
///
/// # Arguments
/// * `old_path` - путь к незашифрованной базе данных
/// * `new_path` - путь для новой зашифрованной базы данных
/// * `key` - ключ шифрования (hex строка)
///
/// # Returns
/// - `Ok(())` - миграция успешна
/// - `Err(String)` - описание ошибки
pub fn migrate_to_encrypted(
    old_path: &PathBuf,
    new_path: &PathBuf,
    key: &str,
) -> Result<(), String> {
    // Открываем незашифрованную БД
    let old_conn = Connection::open(old_path).map_err(|e| e.to_string())?;

    // Проверяем, что старая БД читается
    old_conn
        .query_row("SELECT count(*) FROM sqlite_master", [], |_| Ok(()))
        .map_err(|e| format!("{}: {}", messages::ERR_SOURCE_DB_NOT_VALID, e))?;

    // Экспортируем в зашифрованную БД (ключ в формате x'hex')
    old_conn
        .execute_batch(&format!(
            "ATTACH DATABASE '{}' AS encrypted KEY \"x'{}'\";
             SELECT sqlcipher_export('encrypted');
             DETACH DATABASE encrypted;",
            new_path.display(),
            key
        ))
        .map_err(|e| format!("{}: {}", messages::ERR_MIGRATE_DATABASE, e))?;

    Ok(())
}

/// Автоматически мигрирует БД если она не зашифрована.
///
/// Эта функция проверяет, является ли БД зашифрованной, и если нет -
/// создает зашифрованную копию и заменяет оригинал.
pub fn ensure_encrypted(db_path: &PathBuf) -> Result<(), String> {
    if !db_path.exists() {
        // БД не существует, будет создана зашифрованной
        return Ok(());
    }

    // Проверяем, зашифрована ли БД
    if is_database_encrypted(db_path) {
        // Уже зашифрована
        return Ok(());
    }

    // БД не зашифрована, нужна миграция
    let key = crate::crypto::get_or_create_key()?;

    // Создаем временный путь для новой БД
    let temp_path = db_path.with_extension("db.encrypted");

    // Мигрируем
    migrate_to_encrypted(db_path, &temp_path, &key)?;

    // Создаем бэкап старой БД
    let backup_path = db_path.with_extension("db.unencrypted.bak");
    std::fs::rename(db_path, &backup_path)
        .map_err(|e| format!("{}: {}", messages::ERR_BACKUP_OLD_DATABASE, e))?;

    // Переименовываем новую БД
    std::fs::rename(&temp_path, db_path)
        .map_err(|e| format!("{}: {}", messages::ERR_RENAME_ENCRYPTED_DATABASE, e))?;

    // Удаляем WAL и SHM файлы от старой БД если есть
    let wal_path = backup_path.with_extension("db.unencrypted.bak-wal");
    let shm_path = backup_path.with_extension("db.unencrypted.bak-shm");
    let _ = std::fs::remove_file(wal_path);
    let _ = std::fs::remove_file(shm_path);

    Ok(())
}
