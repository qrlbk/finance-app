//! Модуль управления ключами шифрования для SQLCipher.
//!
//! Обеспечивает генерацию, хранение и получение ключа шифрования базы данных.

use crate::messages;
use directories::ProjectDirs;
use rand::Rng;
use std::fs;
use std::path::PathBuf;

/// Длина ключа шифрования в байтах (256 бит).
const KEY_LENGTH: usize = 32;

/// Получить существующий ключ или создать новый.
///
/// Ключ хранится в файле `.key` в директории данных приложения.
/// При первом запуске генерируется новый случайный ключ.
///
/// # Returns
/// - `Ok(String)` - hex-encoded ключ шифрования
/// - `Err(String)` - описание ошибки
pub fn get_or_create_key() -> Result<String, String> {
    let key_path = get_key_path()?;

    if key_path.exists() {
        // Читаем существующий ключ
        let key = fs::read_to_string(&key_path)
            .map_err(|e| format!("{}: {}", messages::ERR_READ_ENCRYPTION_KEY, e))?;

        // Валидация ключа (должен быть hex строкой правильной длины)
        if key.len() == KEY_LENGTH * 2 && key.chars().all(|c| c.is_ascii_hexdigit()) {
            Ok(key)
        } else {
            // Ключ поврежден, генерируем новый
            let new_key = generate_secure_key();
            fs::write(&key_path, &new_key)
                .map_err(|e| format!("{}: {}", messages::ERR_WRITE_ENCRYPTION_KEY, e))?;
            Ok(new_key)
        }
    } else {
        // Создаем директорию если не существует
        if let Some(parent) = key_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("{}: {}", messages::ERR_CREATE_KEY_DIRECTORY, e))?;
        }

        // Генерируем новый ключ
        let key = generate_secure_key();
        fs::write(&key_path, &key)
            .map_err(|e| format!("{}: {}", messages::ERR_WRITE_ENCRYPTION_KEY, e))?;

        // Устанавливаем ограничения доступа на Unix системах
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = fs::Permissions::from_mode(0o600);
            let _ = fs::set_permissions(&key_path, permissions);
        }

        Ok(key)
    }
}

/// Получить путь к файлу ключа.
fn get_key_path() -> Result<PathBuf, String> {
    let dirs = ProjectDirs::from("com", "kuralbekadilet475", "finance-app")
        .ok_or("Cannot determine app directory")?;

    Ok(dirs.data_dir().join(".key"))
}

/// Сгенерировать безопасный случайный ключ.
///
/// Использует криптографически стойкий генератор случайных чисел.
fn generate_secure_key() -> String {
    let mut rng = rand::thread_rng();
    let key_bytes: Vec<u8> = (0..KEY_LENGTH).map(|_| rng.gen()).collect();

    // Конвертируем в hex строку
    key_bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Проверить, существует ли файл ключа.
pub fn key_exists() -> bool {
    get_key_path().map(|p| p.exists()).unwrap_or(false)
}

/// Удалить ключ шифрования (опасная операция!).
///
/// ВНИМАНИЕ: После удаления ключа зашифрованная база данных станет недоступна!
#[allow(dead_code)]
pub fn delete_key() -> Result<(), String> {
    let key_path = get_key_path()?;
    if key_path.exists() {
        fs::remove_file(&key_path).map_err(|e| format!("{}: {}", messages::ERR_DELETE_KEY, e))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_secure_key_length() {
        let key = generate_secure_key();
        assert_eq!(key.len(), KEY_LENGTH * 2);
    }

    #[test]
    fn test_generate_secure_key_hex() {
        let key = generate_secure_key();
        assert!(key.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_generate_secure_key_unique() {
        let key1 = generate_secure_key();
        let key2 = generate_secure_key();
        assert_ne!(key1, key2);
    }
}
