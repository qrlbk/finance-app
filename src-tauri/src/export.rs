//! Export module for CSV, JSON, and Excel export functionality

use crate::db::{self, TransactionWithDetails, Account, Category};
use rusqlite::Connection;
use rust_xlsxwriter::{Format, Workbook};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Deserialize)]
pub struct ExportOptions {
    pub format: String,          // "csv" | "json" | "xlsx"
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub include_accounts: bool,
    pub include_categories: bool,
}

#[derive(Serialize, Deserialize)]
pub struct ExportData {
    pub transactions: Vec<TransactionWithDetails>,
    pub accounts: Option<Vec<Account>>,
    pub categories: Option<Vec<Category>>,
    pub exported_at: String,
}

#[derive(Serialize, Deserialize)]
pub struct ImportResult {
    pub transactions_imported: usize,
    pub accounts_imported: usize,
    pub categories_imported: usize,
    pub errors: Vec<String>,
}

/// Export data to the specified format
pub fn export_data(conn: &Connection, options: &ExportOptions, output_path: &Path) -> Result<(), String> {
    let filters = db::TransactionFilters {
        limit: 10000, // Large limit for export
        date_from: options.date_from.clone(),
        date_to: options.date_to.clone(),
        account_id: None,
        category_id: None,
        transaction_type: None,
        search_note: None,
    };
    
    let transactions = db::get_transactions_filtered(conn, filters)?;
    
    let accounts = if options.include_accounts {
        Some(db::get_accounts(conn)?)
    } else {
        None
    };
    
    let categories = if options.include_categories {
        Some(db::get_categories(conn)?)
    } else {
        None
    };

    match options.format.as_str() {
        "csv" => export_csv(&transactions, output_path),
        "json" => export_json(&transactions, &accounts, &categories, output_path),
        "xlsx" => export_xlsx(&transactions, output_path),
        _ => Err(format!("Unsupported format: {}", options.format)),
    }
}

fn export_csv(transactions: &[TransactionWithDetails], output_path: &Path) -> Result<(), String> {
    let file = File::create(output_path).map_err(|e| e.to_string())?;
    let mut wtr = csv::Writer::from_writer(file);
    
    // Write header
    wtr.write_record(&[
        "ID", "Date", "Type", "Amount", "Account", "Category", "Note"
    ]).map_err(|e| e.to_string())?;
    
    // Write data
    for tx in transactions {
        let amount = if tx.transaction_type == "expense" {
            format!("-{:.2}", tx.amount.abs())
        } else {
            format!("{:.2}", tx.amount.abs())
        };
        
        wtr.write_record(&[
            tx.id.to_string(),
            tx.date.clone(),
            tx.transaction_type.clone(),
            amount,
            tx.account_name.clone(),
            tx.category_name.clone().unwrap_or_default(),
            tx.note.clone().unwrap_or_default(),
        ]).map_err(|e| e.to_string())?;
    }
    
    wtr.flush().map_err(|e| e.to_string())?;
    Ok(())
}

fn export_json(
    transactions: &[TransactionWithDetails],
    accounts: &Option<Vec<Account>>,
    categories: &Option<Vec<Category>>,
    output_path: &Path,
) -> Result<(), String> {
    let data = ExportData {
        transactions: transactions.to_vec(),
        accounts: accounts.clone(),
        categories: categories.clone(),
        exported_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    };
    
    let json = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;
    
    let mut file = File::create(output_path).map_err(|e| e.to_string())?;
    file.write_all(json.as_bytes()).map_err(|e| e.to_string())?;
    
    Ok(())
}

fn export_xlsx(transactions: &[TransactionWithDetails], output_path: &Path) -> Result<(), String> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    
    // Header format
    let header_format = Format::new()
        .set_bold()
        .set_background_color(rust_xlsxwriter::Color::RGB(0x4472C4))
        .set_font_color(rust_xlsxwriter::Color::White);
    
    // Number format for currency
    let currency_format = Format::new()
        .set_num_format("#,##0.00 ₸");
    
    let income_format = Format::new()
        .set_num_format("#,##0.00 ₸")
        .set_font_color(rust_xlsxwriter::Color::RGB(0x22C55E));
    
    let expense_format = Format::new()
        .set_num_format("-#,##0.00 ₸")
        .set_font_color(rust_xlsxwriter::Color::RGB(0xEF4444));
    
    // Write headers
    let headers = ["ID", "Дата", "Тип", "Сумма", "Счёт", "Категория", "Заметка"];
    for (col, header) in headers.iter().enumerate() {
        worksheet.write_string_with_format(0, col as u16, *header, &header_format)
            .map_err(|e| e.to_string())?;
    }
    
    // Write data
    for (row, tx) in transactions.iter().enumerate() {
        let row = row as u32 + 1;
        
        worksheet.write_number(row, 0, tx.id as f64)
            .map_err(|e| e.to_string())?;
        
        worksheet.write_string(row, 1, &tx.date)
            .map_err(|e| e.to_string())?;
        
        let type_label = if tx.transaction_type == "income" { "Доход" } else { "Расход" };
        worksheet.write_string(row, 2, type_label)
            .map_err(|e| e.to_string())?;
        
        // Amount with color formatting
        let amount = tx.amount.abs();
        if tx.transaction_type == "income" {
            worksheet.write_number_with_format(row, 3, amount, &income_format)
                .map_err(|e| e.to_string())?;
        } else {
            worksheet.write_number_with_format(row, 3, -amount, &expense_format)
                .map_err(|e| e.to_string())?;
        }
        
        worksheet.write_string(row, 4, &tx.account_name)
            .map_err(|e| e.to_string())?;
        
        worksheet.write_string(row, 5, tx.category_name.as_deref().unwrap_or(""))
            .map_err(|e| e.to_string())?;
        
        worksheet.write_string(row, 6, tx.note.as_deref().unwrap_or(""))
            .map_err(|e| e.to_string())?;
    }
    
    // Auto-fit columns
    worksheet.set_column_width(0, 8).map_err(|e| e.to_string())?;   // ID
    worksheet.set_column_width(1, 12).map_err(|e| e.to_string())?;  // Date
    worksheet.set_column_width(2, 10).map_err(|e| e.to_string())?;  // Type
    worksheet.set_column_width(3, 15).map_err(|e| e.to_string())?;  // Amount
    worksheet.set_column_width(4, 15).map_err(|e| e.to_string())?;  // Account
    worksheet.set_column_width(5, 15).map_err(|e| e.to_string())?;  // Category
    worksheet.set_column_width(6, 30).map_err(|e| e.to_string())?;  // Note
    
    // Save workbook
    workbook.save(output_path).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[derive(Deserialize)]
struct CsvTransaction {
    #[serde(rename = "Date")]
    date: String,
    #[serde(rename = "Type")]
    transaction_type: String,
    #[serde(rename = "Amount")]
    amount: String,
    #[serde(rename = "Account")]
    account_name: String,
    #[serde(rename = "Category")]
    category_name: Option<String>,
    #[serde(rename = "Note")]
    note: Option<String>,
}

/// Import data from CSV
pub fn import_csv(conn: &Connection, file_path: &Path) -> Result<ImportResult, String> {
    let file = File::open(file_path).map_err(|e| e.to_string())?;
    let mut rdr = csv::Reader::from_reader(file);
    
    let mut result = ImportResult {
        transactions_imported: 0,
        accounts_imported: 0,
        categories_imported: 0,
        errors: Vec::new(),
    };
    
    // Get existing accounts and categories for mapping
    let accounts = db::get_accounts(conn)?;
    let categories = db::get_categories(conn)?;
    
    for (i, record) in rdr.deserialize::<CsvTransaction>().enumerate() {
        match record {
            Ok(tx) => {
                // Find or skip if account doesn't exist
                let account = accounts.iter().find(|a| a.name == tx.account_name);
                if account.is_none() {
                    result.errors.push(format!("Row {}: Account '{}' not found", i + 2, tx.account_name));
                    continue;
                }
                let account_id = account.unwrap().id;
                
                // Find category if specified
                let category_id = tx.category_name
                    .as_ref()
                    .filter(|n| !n.is_empty())
                    .and_then(|name| categories.iter().find(|c| &c.name == name))
                    .map(|c| c.id);
                
                // Parse amount
                let amount: f64 = tx.amount.replace(',', ".").parse().unwrap_or(0.0);
                if amount == 0.0 {
                    result.errors.push(format!("Row {}: Invalid amount", i + 2));
                    continue;
                }
                
                // Determine transaction type from amount if not specified
                let tx_type = if tx.transaction_type.is_empty() {
                    if amount < 0.0 { "expense" } else { "income" }
                } else {
                    &tx.transaction_type
                };
                
                // Create transaction
                match db::create_transaction(
                    conn,
                    account_id,
                    category_id,
                    amount.abs(),
                    tx_type,
                    tx.note.as_deref(),
                    &tx.date,
                ) {
                    Ok(_) => result.transactions_imported += 1,
                    Err(e) => result.errors.push(format!("Row {}: {}", i + 2, e)),
                }
            }
            Err(e) => {
                result.errors.push(format!("Row {}: {}", i + 2, e));
            }
        }
    }
    
    Ok(result)
}

/// Import data from JSON
pub fn import_json(conn: &Connection, file_path: &Path) -> Result<ImportResult, String> {
    let file = File::open(file_path).map_err(|e| e.to_string())?;
    let data: ExportData = serde_json::from_reader(file).map_err(|e| e.to_string())?;
    
    let mut result = ImportResult {
        transactions_imported: 0,
        accounts_imported: 0,
        categories_imported: 0,
        errors: Vec::new(),
    };
    
    // Get existing accounts and categories for mapping
    let accounts = db::get_accounts(conn)?;
    let categories = db::get_categories(conn)?;
    
    for tx in data.transactions {
        // Find account by name
        let account = accounts.iter().find(|a| a.name == tx.account_name);
        if account.is_none() {
            result.errors.push(format!("Transaction {}: Account '{}' not found", tx.id, tx.account_name));
            continue;
        }
        let account_id = account.unwrap().id;
        
        // Find category by name if specified
        let category_id = tx.category_name
            .as_ref()
            .and_then(|name| categories.iter().find(|c| &c.name == name))
            .map(|c| c.id);
        
        match db::create_transaction(
            conn,
            account_id,
            category_id,
            tx.amount.abs(),
            &tx.transaction_type,
            tx.note.as_deref(),
            &tx.date,
        ) {
            Ok(_) => result.transactions_imported += 1,
            Err(e) => result.errors.push(format!("Transaction {}: {}", tx.id, e)),
        }
    }
    
    Ok(result)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use std::io::Read;
    use tempfile::tempdir;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::schema::create_tables(&conn).unwrap();
        crate::db::schema::seed_categories(&conn).unwrap();
        conn
    }

    fn setup_db_with_data() -> Connection {
        let conn = setup_test_db();

        // Create account
        db::create_account(&conn, "Тестовый счёт", "card", "KZT").unwrap();

        // Get a category for transactions
        let categories = db::get_categories(&conn).unwrap();
        let expense_cat = categories.iter().find(|c| c.category_type == "expense").unwrap();

        // Create some transactions
        db::create_transaction(&conn, 1, Some(expense_cat.id), 1000.0, "expense", Some("Покупка 1"), "2024-01-15").unwrap();
        db::create_transaction(&conn, 1, Some(expense_cat.id), 500.0, "expense", Some("Покупка 2"), "2024-01-16").unwrap();
        db::create_transaction(&conn, 1, None, 5000.0, "income", Some("Зарплата"), "2024-01-10").unwrap();

        conn
    }

    // =====================================================================
    // CSV Export Tests
    // =====================================================================

    #[test]
    fn test_export_csv_creates_file() {
        let conn = setup_db_with_data();
        let dir = tempdir().unwrap();
        let output_path = dir.path().join("export.csv");

        let options = ExportOptions {
            format: "csv".to_string(),
            date_from: None,
            date_to: None,
            include_accounts: false,
            include_categories: false,
        };

        export_data(&conn, &options, &output_path).unwrap();

        assert!(output_path.exists());
    }

    #[test]
    fn test_export_csv_has_header() {
        let conn = setup_db_with_data();
        let dir = tempdir().unwrap();
        let output_path = dir.path().join("export.csv");

        let options = ExportOptions {
            format: "csv".to_string(),
            date_from: None,
            date_to: None,
            include_accounts: false,
            include_categories: false,
        };

        export_data(&conn, &options, &output_path).unwrap();

        let mut content = String::new();
        File::open(&output_path).unwrap().read_to_string(&mut content).unwrap();

        assert!(content.contains("ID"));
        assert!(content.contains("Date"));
        assert!(content.contains("Type"));
        assert!(content.contains("Amount"));
    }

    #[test]
    fn test_export_csv_contains_transactions() {
        let conn = setup_db_with_data();
        let dir = tempdir().unwrap();
        let output_path = dir.path().join("export.csv");

        let options = ExportOptions {
            format: "csv".to_string(),
            date_from: None,
            date_to: None,
            include_accounts: false,
            include_categories: false,
        };

        export_data(&conn, &options, &output_path).unwrap();

        let mut content = String::new();
        File::open(&output_path).unwrap().read_to_string(&mut content).unwrap();

        // Should contain our transactions
        assert!(content.contains("Покупка 1"));
        assert!(content.contains("Зарплата"));
    }

    // =====================================================================
    // JSON Export Tests
    // =====================================================================

    #[test]
    fn test_export_json_creates_file() {
        let conn = setup_db_with_data();
        let dir = tempdir().unwrap();
        let output_path = dir.path().join("export.json");

        let options = ExportOptions {
            format: "json".to_string(),
            date_from: None,
            date_to: None,
            include_accounts: true,
            include_categories: true,
        };

        export_data(&conn, &options, &output_path).unwrap();

        assert!(output_path.exists());
    }

    #[test]
    fn test_export_json_valid_format() {
        let conn = setup_db_with_data();
        let dir = tempdir().unwrap();
        let output_path = dir.path().join("export.json");

        let options = ExportOptions {
            format: "json".to_string(),
            date_from: None,
            date_to: None,
            include_accounts: true,
            include_categories: true,
        };

        export_data(&conn, &options, &output_path).unwrap();

        // Try to parse the JSON
        let file = File::open(&output_path).unwrap();
        let data: ExportData = serde_json::from_reader(file).unwrap();

        assert!(!data.transactions.is_empty());
        assert!(data.accounts.is_some());
        assert!(data.categories.is_some());
        assert!(!data.exported_at.is_empty());
    }

    #[test]
    fn test_export_json_without_accounts_categories() {
        let conn = setup_db_with_data();
        let dir = tempdir().unwrap();
        let output_path = dir.path().join("export.json");

        let options = ExportOptions {
            format: "json".to_string(),
            date_from: None,
            date_to: None,
            include_accounts: false,
            include_categories: false,
        };

        export_data(&conn, &options, &output_path).unwrap();

        let file = File::open(&output_path).unwrap();
        let data: ExportData = serde_json::from_reader(file).unwrap();

        assert!(data.accounts.is_none());
        assert!(data.categories.is_none());
    }

    // =====================================================================
    // XLSX Export Tests
    // =====================================================================

    #[test]
    fn test_export_xlsx_creates_file() {
        let conn = setup_db_with_data();
        let dir = tempdir().unwrap();
        let output_path = dir.path().join("export.xlsx");

        let options = ExportOptions {
            format: "xlsx".to_string(),
            date_from: None,
            date_to: None,
            include_accounts: false,
            include_categories: false,
        };

        export_data(&conn, &options, &output_path).unwrap();

        assert!(output_path.exists());
        // Check file has some content
        let metadata = std::fs::metadata(&output_path).unwrap();
        assert!(metadata.len() > 0);
    }

    // =====================================================================
    // Date Filtering Tests
    // =====================================================================

    #[test]
    fn test_export_with_date_filter() {
        let conn = setup_db_with_data();
        let dir = tempdir().unwrap();
        let output_path = dir.path().join("export.csv");

        let options = ExportOptions {
            format: "csv".to_string(),
            date_from: Some("2024-01-14".to_string()),
            date_to: Some("2024-01-16".to_string()),
            include_accounts: false,
            include_categories: false,
        };

        export_data(&conn, &options, &output_path).unwrap();

        let mut content = String::new();
        File::open(&output_path).unwrap().read_to_string(&mut content).unwrap();

        // Should contain transactions within date range
        assert!(content.contains("Покупка 1")); // 2024-01-15
        assert!(content.contains("Покупка 2")); // 2024-01-16
        // Should NOT contain transaction outside range
        assert!(!content.contains("Зарплата")); // 2024-01-10
    }

    // =====================================================================
    // CSV Import Tests
    // =====================================================================

    #[test]
    fn test_import_csv_valid() {
        let conn = setup_test_db();
        db::create_account(&conn, "Тестовый счёт", "card", "KZT").unwrap();

        let dir = tempdir().unwrap();
        let csv_path = dir.path().join("import.csv");

        // Create a valid CSV file
        let csv_content = "ID,Date,Type,Amount,Account,Category,Note
1,2024-02-01,expense,100.00,Тестовый счёт,Еда,Обед
2,2024-02-02,income,5000.00,Тестовый счёт,,Зарплата";

        std::fs::write(&csv_path, csv_content).unwrap();

        let result = import_csv(&conn, &csv_path).unwrap();

        assert_eq!(result.transactions_imported, 2);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_import_csv_account_not_found() {
        let conn = setup_test_db();
        // Don't create any account

        let dir = tempdir().unwrap();
        let csv_path = dir.path().join("import.csv");

        let csv_content = "ID,Date,Type,Amount,Account,Category,Note
1,2024-02-01,expense,100.00,NonExistentAccount,Еда,Обед";

        std::fs::write(&csv_path, csv_content).unwrap();

        let result = import_csv(&conn, &csv_path).unwrap();

        assert_eq!(result.transactions_imported, 0);
        assert!(!result.errors.is_empty());
        assert!(result.errors[0].contains("not found"));
    }

    // =====================================================================
    // JSON Import Tests
    // =====================================================================

    #[test]
    fn test_import_json_valid() {
        let conn = setup_test_db();
        db::create_account(&conn, "Тестовый счёт", "card", "KZT").unwrap();

        let dir = tempdir().unwrap();
        let json_path = dir.path().join("import.json");

        let json_content = r#"{
            "transactions": [
                {
                    "id": 1,
                    "account_id": 1,
                    "account_name": "Тестовый счёт",
                    "category_id": null,
                    "category_name": null,
                    "amount": 1000.0,
                    "transaction_type": "expense",
                    "note": "Тестовая транзакция",
                    "date": "2024-02-01"
                }
            ],
            "accounts": null,
            "categories": null,
            "exported_at": "2024-02-01 10:00:00"
        }"#;

        std::fs::write(&json_path, json_content).unwrap();

        let result = import_json(&conn, &json_path).unwrap();

        assert_eq!(result.transactions_imported, 1);
        assert!(result.errors.is_empty());
    }

    // =====================================================================
    // Unsupported Format Test
    // =====================================================================

    #[test]
    fn test_export_unsupported_format() {
        let conn = setup_db_with_data();
        let dir = tempdir().unwrap();
        let output_path = dir.path().join("export.txt");

        let options = ExportOptions {
            format: "txt".to_string(),
            date_from: None,
            date_to: None,
            include_accounts: false,
            include_categories: false,
        };

        let result = export_data(&conn, &options, &output_path);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unsupported format"));
    }
}
