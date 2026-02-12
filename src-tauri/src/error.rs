//! Error types for the finance app using thiserror

use thiserror::Error;

/// Application error types
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Ошибка базы данных: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Ошибка сериализации: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Ошибка ввода-вывода: {0}")]
    Io(#[from] std::io::Error),

    #[error("Ошибка валидации: {0}")]
    Validation(String),

    #[error("Ресурс не найден: {0}")]
    NotFound(String),

    #[error("Недостаточно данных: {0}")]
    InsufficientData(String),

    #[error("Внутренняя ошибка: {0}")]
    Internal(String),

    #[error("Ошибка конфигурации: {0}")]
    Config(String),

    #[error("Неподдерживаемый формат: {0}")]
    UnsupportedFormat(String),

    #[error("Ошибка CSV: {0}")]
    Csv(#[from] csv::Error),
}

/// Validation error with field information
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Поле '{field}' не может быть пустым")]
    EmptyField { field: String },

    #[error("Недопустимое значение для поля '{field}': {reason}")]
    InvalidValue { field: String, reason: String },

    #[error("Значение '{field}' должно быть больше {min}")]
    BelowMinimum { field: String, min: f64 },

    #[error("Значение '{field}' не должно превышать {max}")]
    AboveMaximum { field: String, max: f64 },

    #[error("Недопустимый формат даты: {0}")]
    InvalidDate(String),

    #[error("Недопустимый тип: {0}")]
    InvalidType(String),
}

impl From<ValidationError> for AppError {
    fn from(err: ValidationError) -> Self {
        AppError::Validation(err.to_string())
    }
}

/// Result type alias for our app
pub type AppResult<T> = Result<T, AppError>;

/// Convert AppError to String for Tauri commands
impl From<AppError> for String {
    fn from(err: AppError) -> Self {
        err.to_string()
    }
}

/// Convert a String error to AppError::Internal
impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::Internal(s)
    }
}

/// Helper trait for adding context to errors
pub trait ResultExt<T> {
    /// Add context to an error
    fn with_context(self, context: &str) -> AppResult<T>;
}

impl<T, E: std::error::Error> ResultExt<T> for Result<T, E> {
    fn with_context(self, context: &str) -> AppResult<T> {
        self.map_err(|e| AppError::Internal(format!("{}: {}", context, e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = AppError::Validation("Invalid amount".to_string());
        assert!(err.to_string().contains("Invalid amount"));
    }

    #[test]
    fn test_validation_error_conversion() {
        let validation_err = ValidationError::EmptyField { field: "name".to_string() };
        let app_err: AppError = validation_err.into();
        assert!(app_err.to_string().contains("name"));
    }

    #[test]
    fn test_error_to_string() {
        let err = AppError::NotFound("Account".to_string());
        let s: String = err.into();
        assert!(s.contains("не найден"));
    }
}
