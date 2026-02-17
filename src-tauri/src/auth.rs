//! Local authentication: password hashing (Argon2id) and validation.
//! No network; credentials are stored only in the local DB.

use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;

const MIN_PASSWORD_LEN: usize = 6;
const MAX_PASSWORD_LEN: usize = 512;
const MIN_USERNAME_LEN: usize = 1;
const MAX_USERNAME_LEN: usize = 64;
const MIN_DISPLAY_NAME_LEN: usize = 1;
const MAX_DISPLAY_NAME_LEN: usize = 128;

/// Username: alphanumeric, underscore, hyphen. No spaces.
fn is_valid_username_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-'
}

/// Returns Ok(()) if valid, Err(message) otherwise.
pub fn validate_username(username: &str) -> Result<(), String> {
    let s = username.trim();
    if s.len() < MIN_USERNAME_LEN {
        return Err("Имя пользователя не может быть пустым".to_string());
    }
    if s.len() > MAX_USERNAME_LEN {
        return Err("Имя пользователя слишком длинное".to_string());
    }
    if !s.chars().all(is_valid_username_char) {
        return Err("Имя пользователя может содержать только буквы, цифры, _ и -".to_string());
    }
    Ok(())
}

/// Returns Ok(()) if valid, Err(message) otherwise.
pub fn validate_password(password: &str) -> Result<(), String> {
    if password.len() < MIN_PASSWORD_LEN {
        return Err("Пароль должен быть не короче 6 символов".to_string());
    }
    if password.len() > MAX_PASSWORD_LEN {
        return Err("Пароль слишком длинный".to_string());
    }
    Ok(())
}

/// Returns Ok(()) if valid, Err(message) otherwise.
pub fn validate_display_name(display_name: &str) -> Result<(), String> {
    let s = display_name.trim();
    if s.len() < MIN_DISPLAY_NAME_LEN {
        return Err("Отображаемое имя не может быть пустым".to_string());
    }
    if s.len() > MAX_DISPLAY_NAME_LEN {
        return Err("Отображаемое имя слишком длинное".to_string());
    }
    Ok(())
}

/// Hash password with Argon2id; returns PHC string.
pub fn hash_password(password: &str) -> Result<String, String> {
    validate_password(password)?;
    let salt = SaltString::generate(&mut rand::rngs::OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| format!("Ошибка хеширования пароля: {}", e))
}

/// Verify password against a PHC hash string.
pub fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
    let parsed =
        PasswordHash::new(hash).map_err(|e| format!("Неверный формат хеша: {}", e))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_username() {
        assert!(validate_username("user").is_ok());
        assert!(validate_username("user_1").is_ok());
        assert!(validate_username("user-1").is_ok());
        assert!(validate_username("").is_err());
        assert!(validate_username("  ").is_err());
        assert!(validate_username("user name").is_err());
    }

    #[test]
    fn test_validate_password() {
        assert!(validate_password("short").is_err());
        assert!(validate_password("123456").is_ok());
        assert!(validate_password("long_enough").is_ok());
    }

    #[test]
    fn test_hash_and_verify() {
        let password = "test_password_123";
        let hash = hash_password(password).unwrap();
        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrong", &hash).unwrap());
    }
}
