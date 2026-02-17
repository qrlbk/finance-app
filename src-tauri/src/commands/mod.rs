//! Tauri commands module — split into submodules by domain.

mod common;
mod auth;
mod accounts;
mod transactions;
mod categories;
mod summary_transfers;
mod recurring;
mod budgets;
mod ml_cmd;
mod export_import;
mod bank_import_cmd;

pub use common::{AppState, QueryCache, init_db_on_startup};
pub use auth::*;
pub use accounts::*;
pub use transactions::*;
pub use categories::*;
pub use summary_transfers::*;
pub use recurring::*;
pub use budgets::*;
pub use ml_cmd::*;
pub use export_import::*;
pub use bank_import_cmd::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    #[test]
    fn test_validate_account_input_empty_name() {
        let result = validate_account_input("", "card", "KZT");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("пустым"));
    }

    #[test]
    fn test_validate_account_input_valid() {
        let result = validate_account_input("Карта", "card", "KZT");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_account_input_invalid_type() {
        let result = validate_account_input("Карта", "invalid_type", "KZT");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("тип"));
    }

    fn setup_test_db() -> rusqlite::Connection {
        common::setup_test_db()
    }

    #[test]
    fn test_validate_transaction_input_zero_amount() {
        let conn = setup_test_db();
        let _ = db::create_account(&conn, 1, "Тест", "card", "KZT", 0.0);
        let result = validate_transaction_input(&conn, 1, 1, None, 0.0, "income", "2024-01-01");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("больше нуля"));
    }

    #[test]
    fn test_valid_account_types() {
        assert!(VALID_ACCOUNT_TYPES.contains(&"cash"));
        assert!(VALID_ACCOUNT_TYPES.contains(&"card"));
        assert!(VALID_ACCOUNT_TYPES.contains(&"savings"));
    }

    #[test]
    fn test_valid_transaction_types() {
        assert!(transactions::VALID_TRANSACTION_TYPES.contains(&"income"));
        assert!(transactions::VALID_TRANSACTION_TYPES.contains(&"expense"));
    }

    #[test]
    fn test_query_cache_categories_empty() {
        let cache = QueryCache::new();
        assert!(cache.get_categories().is_none());
    }

    #[test]
    fn test_query_cache_invalidate_all() {
        let cache = QueryCache::new();
        cache.set_categories(vec![db::Category {
            id: 1,
            name: "Тест".to_string(),
            category_type: "expense".to_string(),
            icon: None,
            color: None,
            parent_id: None,
        }]);
        cache.set_accounts(vec![db::Account {
            id: 1,
            name: "Карта".to_string(),
            account_type: "card".to_string(),
            balance: 0.0,
            currency: "KZT".to_string(),
        }]);
        cache.invalidate_all();
        assert!(cache.get_categories().is_none());
        assert!(cache.get_accounts().is_none());
    }

    #[test]
    fn test_check_duplicate_no_match() {
        let conn = setup_test_db();
        let account_id = db::create_account(&conn, 1, "Тест", "card", "KZT", 0.0).unwrap();
        let _ = db::create_transaction(&conn, 1, account_id, None, 100.0, "expense", Some("Покупка"), "2024-01-15");
        let is_dup = bank_import_cmd::check_duplicate(&conn, "2024-01-20", 200.0, "Другая покупка").unwrap();
        assert!(!is_dup);
    }

    #[test]
    fn test_check_duplicate_exact_match() {
        let conn = setup_test_db();
        let account_id = db::create_account(&conn, 1, "Тест", "card", "KZT", 0.0).unwrap();
        let _ = db::create_transaction(&conn, 1, account_id, None, 100.0, "expense", Some("Покупка в магазине"), "2024-01-15");
        let is_dup = bank_import_cmd::check_duplicate(&conn, "2024-01-15", 100.0, "Покупка в магазине").unwrap();
        assert!(is_dup);
    }

    #[test]
    fn test_check_duplicate_normalized_spaces_match() {
        let conn = setup_test_db();
        let account_id = db::create_account(&conn, 1, "Тест", "card", "KZT", 0.0).unwrap();
        let _ = db::create_transaction(&conn, 1, account_id, None, 50.0, "expense", Some("Кофе магазин"), "2024-01-10");
        let is_dup = bank_import_cmd::check_duplicate(&conn, "2024-01-10", 50.0, "  Кофе   магазин  ").unwrap();
        assert!(is_dup);
    }

    #[test]
    fn test_check_duplicate_different_note_no_match() {
        let conn = setup_test_db();
        let account_id = db::create_account(&conn, 1, "Тест", "card", "KZT", 0.0).unwrap();
        let _ = db::create_transaction(&conn, 1, account_id, None, 100.0, "expense", Some("Покупка в магазине"), "2024-01-15");
        let is_dup = bank_import_cmd::check_duplicate(&conn, "2024-01-15", 100.0, "Покупка в кафе").unwrap();
        assert!(!is_dup);
    }
}
