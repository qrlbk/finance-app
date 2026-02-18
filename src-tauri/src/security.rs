//! Security utilities for input sanitization and validation

use crate::error::{AppError, ValidationError};
use crate::messages;

/// Maximum allowed string length for various fields
#[allow(dead_code)]
pub mod limits {
    pub const NAME_MAX_LEN: usize = 100;
    pub const NOTE_MAX_LEN: usize = 500;
    pub const CURRENCY_MAX_LEN: usize = 10;
    pub const PATH_MAX_LEN: usize = 1000;
    pub const AMOUNT_MAX: f64 = 1_000_000_000.0; // 1 billion
    pub const AMOUNT_MIN: f64 = 0.0;
}

/// Sanitize a text input by trimming and limiting length
pub fn sanitize_text(input: &str, max_len: usize) -> String {
    let trimmed = input.trim();
    if trimmed.len() > max_len {
        trimmed.chars().take(max_len).collect()
    } else {
        trimmed.to_string()
    }
}

/// Sanitize a name field (removes control characters)
pub fn sanitize_name(input: &str) -> String {
    let cleaned: String = input
        .chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\r')
        .collect();
    sanitize_text(&cleaned, limits::NAME_MAX_LEN)
}

/// Sanitize a note field
pub fn sanitize_note(input: &str) -> String {
    let cleaned: String = input
        .chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\r')
        .collect();
    sanitize_text(&cleaned, limits::NOTE_MAX_LEN)
}

/// Validate an amount is within acceptable range
pub fn validate_amount(amount: f64) -> Result<f64, ValidationError> {
    if amount < limits::AMOUNT_MIN {
        return Err(ValidationError::BelowMinimum {
            field: "amount".to_string(),
            min: limits::AMOUNT_MIN,
        });
    }
    if amount > limits::AMOUNT_MAX {
        return Err(ValidationError::AboveMaximum {
            field: "amount".to_string(),
            max: limits::AMOUNT_MAX,
        });
    }
    if amount.is_nan() || amount.is_infinite() {
        return Err(ValidationError::InvalidValue {
            field: "amount".to_string(),
            reason: "must be a valid number".to_string(),
        });
    }
    Ok(amount)
}

/// Validate a date string format (YYYY-MM-DD)
pub fn validate_date(date_str: &str) -> Result<(), ValidationError> {
    if date_str.is_empty() {
        return Err(ValidationError::EmptyField {
            field: "date".to_string(),
        });
    }
    
    // Check format with regex-like validation
    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() != 3 {
        return Err(ValidationError::InvalidDate(date_str.to_string()));
    }
    
    let year: i32 = parts[0].parse().map_err(|_| ValidationError::InvalidDate(date_str.to_string()))?;
    let month: u32 = parts[1].parse().map_err(|_| ValidationError::InvalidDate(date_str.to_string()))?;
    let day: u32 = parts[2].parse().map_err(|_| ValidationError::InvalidDate(date_str.to_string()))?;
    
    // Basic range checks
    if year < 1900 || year > 2100 {
        return Err(ValidationError::InvalidDate(date_str.to_string()));
    }
    if month < 1 || month > 12 {
        return Err(ValidationError::InvalidDate(date_str.to_string()));
    }
    if day < 1 || day > 31 {
        return Err(ValidationError::InvalidDate(date_str.to_string()));
    }
    
    Ok(())
}

/// Validate a non-empty string field
pub fn validate_required(value: &str, field_name: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        return Err(ValidationError::EmptyField {
            field: field_name.to_string(),
        });
    }
    Ok(())
}

/// Validate a transaction type
pub fn validate_transaction_type(type_str: &str) -> Result<(), ValidationError> {
    match type_str {
        "income" | "expense" => Ok(()),
        _ => Err(ValidationError::InvalidType(format!(
            "transaction type must be 'income' or 'expense', got '{}'",
            type_str
        ))),
    }
}

/// Validate an account type
pub fn validate_account_type(type_str: &str) -> Result<(), ValidationError> {
    match type_str {
        "cash" | "card" | "savings" | "checking" => Ok(()),
        _ => Err(ValidationError::InvalidType(format!(
            "account type must be 'cash', 'card', or 'savings', got '{}'",
            type_str
        ))),
    }
}

/// Validate a recurring payment frequency
pub fn validate_frequency(freq: &str) -> Result<(), ValidationError> {
    match freq {
        "daily" | "weekly" | "biweekly" | "monthly" | "yearly" => Ok(()),
        _ => Err(ValidationError::InvalidType(format!(
            "invalid frequency: '{}'",
            freq
        ))),
    }
}

/// Validate a budget period
pub fn validate_period(period: &str) -> Result<(), ValidationError> {
    match period {
        "weekly" | "monthly" | "yearly" => Ok(()),
        _ => Err(ValidationError::InvalidType(format!(
            "invalid period: '{}'",
            period
        ))),
    }
}

/// Sanitize a file path (prevent directory traversal)
pub fn sanitize_path(path: &str) -> Result<String, AppError> {
    let path = path.trim();
    
    // Check for directory traversal attempts
    if path.contains("..") {
        return Err(AppError::Validation(messages::ERR_INVALID_PATH_TRAVERSAL.to_string()));
    }
    
    // Check length
    if path.len() > limits::PATH_MAX_LEN {
        return Err(AppError::Validation(messages::ERR_PATH_TOO_LONG.to_string()));
    }
    
    Ok(path.to_string())
}

/// Simple rate limiter for operations
pub struct RateLimiter {
    operations: std::sync::Mutex<std::collections::HashMap<String, (std::time::Instant, u32)>>,
    max_ops_per_second: u32,
}

impl RateLimiter {
    pub fn new(max_ops_per_second: u32) -> Self {
        RateLimiter {
            operations: std::sync::Mutex::new(std::collections::HashMap::new()),
            max_ops_per_second,
        }
    }

    /// Check if an operation is allowed
    pub fn check(&self, operation: &str) -> Result<(), AppError> {
        let mut ops = self.operations.lock().map_err(|_| AppError::Internal("Lock error".to_string()))?;
        let now = std::time::Instant::now();
        
        if let Some((last_time, count)) = ops.get_mut(operation) {
            let elapsed = now.duration_since(*last_time);
            
            if elapsed.as_secs() >= 1 {
                // Reset counter after 1 second
                *last_time = now;
                *count = 1;
            } else if *count >= self.max_ops_per_second {
                return Err(AppError::Validation("Too many requests. Please try again.".to_string()));
            } else {
                *count += 1;
            }
        } else {
            ops.insert(operation.to_string(), (now, 1));
        }
        
        Ok(())
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(100) // 100 operations per second default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_text() {
        assert_eq!(sanitize_text("  hello  ", 10), "hello");
        assert_eq!(sanitize_text("hello world", 5), "hello");
    }

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_name("Test\x00Name"), "TestName");
        assert_eq!(sanitize_name("  Normal Name  "), "Normal Name");
    }

    #[test]
    fn test_validate_amount() {
        assert!(validate_amount(100.0).is_ok());
        assert!(validate_amount(-1.0).is_err());
        assert!(validate_amount(f64::NAN).is_err());
        assert!(validate_amount(f64::INFINITY).is_err());
    }

    #[test]
    fn test_validate_date() {
        assert!(validate_date("2024-01-15").is_ok());
        assert!(validate_date("invalid").is_err());
        assert!(validate_date("2024-13-01").is_err()); // invalid month
        assert!(validate_date("").is_err());
    }

    #[test]
    fn test_validate_required() {
        assert!(validate_required("test", "field").is_ok());
        assert!(validate_required("", "field").is_err());
        assert!(validate_required("   ", "field").is_err());
    }

    #[test]
    fn test_sanitize_path() {
        assert!(sanitize_path("/normal/path").is_ok());
        assert!(sanitize_path("../etc/passwd").is_err());
        assert!(sanitize_path("path/../secret").is_err());
    }

    #[test]
    fn test_rate_limiter() {
        let limiter = RateLimiter::new(3);
        
        assert!(limiter.check("test_op").is_ok());
        assert!(limiter.check("test_op").is_ok());
        assert!(limiter.check("test_op").is_ok());
        assert!(limiter.check("test_op").is_err()); // Should be rate limited
    }
}
