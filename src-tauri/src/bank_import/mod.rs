//! Bank statement import module
//!
//! Provides parsing functionality for bank PDF statements from various Kazakhstan banks.
//! Currently supports:
//! - Kaspi Bank (Kaspi Gold statements)

pub mod kaspi;

use serde::{Deserialize, Serialize};

/// Trait for bank-specific PDF parsers
pub trait BankParser: Send + Sync {
    /// Returns the bank name for display
    fn bank_name(&self) -> &'static str;

    /// Checks if this parser can handle the given PDF text
    fn can_parse(&self, text: &str) -> bool;

    /// Parses the PDF text and extracts transactions
    fn parse(&self, text: &str) -> Result<ParsedStatement, String>;
}

/// Represents a parsed bank statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedStatement {
    /// Bank name
    pub bank: String,
    /// Statement period start date (YYYY-MM-DD)
    pub period_start: String,
    /// Statement period end date (YYYY-MM-DD)
    pub period_end: String,
    /// Account number (if available)
    pub account: Option<String>,
    /// Card number (masked, if available)
    pub card: Option<String>,
    /// List of parsed transactions
    pub transactions: Vec<ParsedTransaction>,
}

/// Represents a single parsed transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedTransaction {
    /// Transaction date (YYYY-MM-DD)
    pub date: String,
    /// Transaction amount (always positive)
    pub amount: f64,
    /// Transaction type: "income" or "expense"
    pub transaction_type: String,
    /// Description (merchant name, person name, etc.)
    pub description: String,
    /// Original operation type from bank (e.g., "Покупка", "Пополнение")
    pub original_type: String,
    /// Suggested category ID (from ML prediction)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_category_id: Option<i64>,
    /// Suggestion confidence (0.0 - 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
    /// Whether this transaction might be a duplicate
    #[serde(default)]
    pub is_duplicate: bool,
}

/// Result of importing transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    /// Number of successfully imported transactions
    pub imported: i32,
    /// Number of skipped duplicates
    pub skipped_duplicates: i32,
    /// Number of failed imports
    pub failed: i32,
    /// Error messages for failed imports
    pub errors: Vec<String>,
}

/// Input for importing transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportTransaction {
    /// Transaction date (YYYY-MM-DD)
    pub date: String,
    /// Transaction amount (always positive)
    pub amount: f64,
    /// Transaction type: "income" or "expense"
    pub transaction_type: String,
    /// Description/note
    pub description: String,
    /// Category ID to assign
    pub category_id: Option<i64>,
    /// Whether to skip if duplicate
    pub skip_if_duplicate: bool,
}

/// Extract text from a PDF file
pub fn extract_pdf_text(path: &str) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("Не удалось прочитать файл: {}", e))?;
    pdf_extract::extract_text_from_mem(&bytes)
        .map_err(|e| format!("Не удалось извлечь текст из PDF: {}", e))
}

/// Get all available bank parsers
pub fn get_parsers() -> Vec<Box<dyn BankParser>> {
    vec![Box::new(kaspi::KaspiParser)]
}

/// Parse a bank statement PDF
pub fn parse_statement(path: &str) -> Result<ParsedStatement, String> {
    let text = extract_pdf_text(path)?;

    for parser in get_parsers() {
        if parser.can_parse(&text) {
            return parser.parse(&text);
        }
    }

    Err("Формат выписки не поддерживается. Поддерживаемые банки: Kaspi".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsed_transaction_serialization() {
        let tx = ParsedTransaction {
            date: "2026-02-12".to_string(),
            amount: 1000.0,
            transaction_type: "expense".to_string(),
            description: "Test".to_string(),
            original_type: "Покупка".to_string(),
            suggested_category_id: None,
            confidence: None,
            is_duplicate: false,
        };

        let json = serde_json::to_string(&tx).unwrap();
        assert!(json.contains("2026-02-12"));
    }
}
