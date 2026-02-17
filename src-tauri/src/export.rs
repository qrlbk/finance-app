//! Export module for CSV, JSON, and Excel export functionality

use crate::db::{self, TransactionWithDetails, Account, Category};
use calamine::{open_workbook, Data, DataType, Reader, Xlsx};
use chrono::{NaiveDate, Duration};
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
    pub account_id: Option<i64>,
    pub category_id: Option<i64>,
}

#[derive(Serialize, Deserialize)]
pub struct ExportData {
    pub transactions: Vec<TransactionWithDetails>,
    pub accounts: Option<Vec<Account>>,
    pub categories: Option<Vec<Category>>,
    pub exported_at: String,
}

/// Результат превью импорта (для UI: показать заголовки и первые строки или количество транзакций).
#[derive(Serialize)]
pub struct ImportPreview {
    pub headers: Option<Vec<String>>,
    pub rows: Option<Vec<Vec<String>>>,
    pub transaction_count: Option<usize>,
}

/// Опции импорта: счёт по умолчанию и пропуск дубликатов.
#[derive(Clone, Default)]
pub struct ImportOptions {
    /// Счёт для строк, где счёт не указан или не найден по имени.
    pub default_account_id: Option<i64>,
    /// Не вставлять транзакции с совпадающими (счёт, дата, сумма, заметка).
    pub skip_duplicates: bool,
}

#[derive(Serialize, Deserialize)]
pub struct ImportResult {
    pub transactions_imported: usize,
    pub duplicates_skipped: usize,
    pub accounts_imported: usize,
    pub categories_imported: usize,
    pub errors: Vec<String>,
    /// Всего обработано строк (для контекста в отчёте).
    #[serde(default)]
    pub total_parsed: usize,
}

/// Export data to the specified format for the given user.
/// Экспортирует все транзакции по фильтрам (до 50_000 за один вызов).
pub fn export_data(conn: &Connection, user_id: i64, options: &ExportOptions, output_path: &Path) -> Result<(), String> {
    let filters = db::TransactionFilters {
        limit: 50_000,
        offset: 0,
        date_from: options.date_from.clone(),
        date_to: options.date_to.clone(),
        account_id: options.account_id,
        category_id: options.category_id,
        uncategorized_only: false,
        transaction_type: None,
        search_note: None,
    };

    let transactions = db::get_transactions_filtered(conn, user_id, filters)?;

    let accounts = if options.include_accounts {
        Some(db::get_accounts(conn, user_id)?)
    } else {
        None
    };

    let categories = if options.include_categories {
        Some(db::get_categories(conn, user_id)?)
    } else {
        None
    };

    match options.format.as_str() {
        "csv" => export_csv(&transactions, output_path),
        "json" => export_json(&transactions, &accounts, &categories, output_path),
        "xlsx" => export_xlsx(&transactions, output_path),
        _ => Err(format!("{}{}", crate::messages::ERR_UNSUPPORTED_FORMAT_WITH, options.format)),
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

const PREVIEW_MAX_ROWS: usize = 15;

/// Превью файла перед импортом: заголовки + первые строки (CSV/XLSX) или количество транзакций (JSON).
pub fn import_preview(file_path: &Path, format: &str) -> Result<ImportPreview, String> {
    match format.to_lowercase().as_str() {
        "csv" => import_preview_csv(file_path),
        "json" => import_preview_json(file_path),
        "xlsx" => import_preview_xlsx(file_path),
        _ => Err(crate::messages::ERR_UNSUPPORTED_FORMAT.to_string()),
    }
}

fn import_preview_csv(file_path: &Path) -> Result<ImportPreview, String> {
    let file = File::open(file_path).map_err(|e| e.to_string())?;
    let mut rdr = csv::Reader::from_reader(file);
    let headers: Vec<String> = rdr
        .headers()
        .map_err(|e| e.to_string())?
        .iter()
        .map(|s| s.to_string())
        .collect();
    let mut rows = Vec::with_capacity(PREVIEW_MAX_ROWS);
    for (i, record) in rdr.records().enumerate() {
        if i >= PREVIEW_MAX_ROWS {
            break;
        }
        let record = record.map_err(|e| e.to_string())?;
        rows.push(record.iter().map(|s| s.to_string()).collect());
    }
    Ok(ImportPreview {
        headers: Some(headers),
        rows: Some(rows),
        transaction_count: None,
    })
}

fn import_preview_json(file_path: &Path) -> Result<ImportPreview, String> {
    let file = File::open(file_path).map_err(|e| e.to_string())?;
    let data: ExportData = serde_json::from_reader(file).map_err(|e| e.to_string())?;
    Ok(ImportPreview {
        headers: None,
        rows: None,
        transaction_count: Some(data.transactions.len()),
    })
}

fn import_preview_xlsx(file_path: &Path) -> Result<ImportPreview, String> {
    let mut workbook: Xlsx<_> = open_workbook(file_path).map_err(|e: calamine::XlsxError| e.to_string())?;
    let sheet_names = workbook.sheet_names().to_vec();
    let first_sheet = sheet_names.first().ok_or(crate::messages::ERR_NO_SHEETS)?;
    let range = workbook
        .worksheet_range(first_sheet)
        .map_err(|e| e.to_string())?;
    let (total_rows, total_cols) = range.get_size();
    let mut headers = Vec::with_capacity(total_cols);
    for c in 0..total_cols {
        headers.push(cell_to_string(range.get((0, c))));
    }
    let mut rows = Vec::new();
    for row_idx in 1..total_rows.min(1 + PREVIEW_MAX_ROWS) {
        let mut row = Vec::with_capacity(total_cols);
        for col in 0..total_cols {
            row.push(cell_to_string(range.get((row_idx, col))));
        }
        rows.push(row);
    }
    Ok(ImportPreview {
        headers: Some(headers),
        rows: Some(rows),
        transaction_count: None,
    })
}

/// Синонимы заголовков CSV (EN / RU).
fn csv_header_index(headers: &[String], names: &[&str]) -> Option<usize> {
    for (i, h) in headers.iter().enumerate() {
        let h_lower = h.trim().to_lowercase();
        for n in names {
            if h_lower == n.trim().to_lowercase() {
                return Some(i);
            }
        }
    }
    None
}

fn csv_cell(row: &[String], idx: Option<usize>) -> Option<String> {
    idx.and_then(|i| row.get(i).map(|s| s.trim().to_string()))
}

/// Import data from CSV for the given user. Supports flexible headers (Date/Дата, Amount/Сумма, Account/Счёт, etc.).
pub fn import_csv(
    conn: &Connection,
    user_id: i64,
    file_path: &Path,
    options: &ImportOptions,
) -> Result<ImportResult, String> {
    let file = File::open(file_path).map_err(|e| e.to_string())?;
    let mut rdr = csv::Reader::from_reader(file);

    let mut result = ImportResult {
        transactions_imported: 0,
        duplicates_skipped: 0,
        accounts_imported: 0,
        categories_imported: 0,
        errors: Vec::new(),
        total_parsed: 0,
    };

    let accounts = db::get_accounts(conn, user_id)?;
    let categories = db::get_categories(conn, user_id)?;

    let headers: Vec<String> = rdr
        .headers()
        .map_err(|e| e.to_string())?
        .iter()
        .map(|s| s.to_string())
        .collect();

    let col_date = csv_header_index(&headers, &["date", "дата", "date"]);
    let col_type = csv_header_index(&headers, &["type", "тип"]);
    let col_amount = csv_header_index(&headers, &["amount", "сумма"]);
    let col_account = csv_header_index(&headers, &["account", "счёт", "account"]);
    let col_category = csv_header_index(&headers, &["category", "категория"]);
    let col_note = csv_header_index(&headers, &["note", "заметка", "описание", "description"]);

    let date_idx = col_date.ok_or("В CSV не найдена колонка даты (Date / Дата)")?;
    let amount_idx = col_amount.ok_or("В CSV не найдена колонка суммы (Amount / Сумма)")?;
    let account_idx = col_account;

    for (i, record) in rdr.records().enumerate() {
        result.total_parsed += 1;
        let row_num = i + 2;

        let record = match record {
            Ok(r) => r,
            Err(e) => {
                result.errors.push(crate::messages::row_error(row_num, &e.to_string()));
                continue;
            }
        };

        let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();
        let date_str = csv_cell(&row, Some(date_idx)).unwrap_or_default();
        let amount_str = csv_cell(&row, Some(amount_idx)).unwrap_or_default();
        let account_name = csv_cell(&row, account_idx).unwrap_or_default();
        let category_name = csv_cell(&row, col_category);
        let note = csv_cell(&row, col_note).filter(|s| !s.is_empty());
        let type_str = csv_cell(&row, col_type).unwrap_or_default();

        if date_str.is_empty() {
            result.errors.push(format!("Строка {}: пустая дата", row_num));
            continue;
        }

        let amount: f64 = amount_str.replace(',', ".").parse().unwrap_or(0.0);
        if amount.abs() < 1e-9 {
            result.errors.push(crate::messages::row_invalid_amount(row_num));
            continue;
        }

        let account_id = if account_name.is_empty() {
            options.default_account_id
        } else {
            accounts.iter().find(|a| a.name == account_name).map(|a| a.id)
        }
        .or(options.default_account_id);

        let account_id = match account_id {
            Some(id) => id,
            None => {
                result.errors.push(crate::messages::row_account_not_found(row_num, &account_name));
                continue;
            }
        };

        let category_id = category_name
            .as_ref()
            .filter(|n| !n.is_empty())
            .and_then(|name| categories.iter().find(|c| c.name == name.as_str()))
            .map(|c| c.id);

        let tx_type = if type_str.is_empty() {
            if amount < 0.0 { "expense" } else { "income" }
        } else {
            let t = type_str.to_lowercase();
            if t.contains("доход") || t == "income" { "income" } else { "expense" }
        };

        let amount_abs = amount.abs();
        if options.skip_duplicates {
            if db::transaction_duplicate_exists(
                conn,
                user_id,
                account_id,
                &date_str,
                amount_abs,
                note.as_deref(),
            )? {
                result.duplicates_skipped += 1;
                continue;
            }
        }

        match db::create_transaction(
            conn,
            user_id,
            account_id,
            category_id,
            amount_abs,
            tx_type,
            note.as_deref(),
            &date_str,
        ) {
            Ok(_) => result.transactions_imported += 1,
            Err(e) => result.errors.push(crate::messages::row_error(row_num, &e.to_string())),
        }
    }

    Ok(result)
}

/// Import data from JSON for the given user (format export of this app).
pub fn import_json(
    conn: &Connection,
    user_id: i64,
    file_path: &Path,
    options: &ImportOptions,
) -> Result<ImportResult, String> {
    let file = File::open(file_path).map_err(|e| e.to_string())?;
    let data: ExportData = serde_json::from_reader(file).map_err(|e| e.to_string())?;

    let mut result = ImportResult {
        transactions_imported: 0,
        duplicates_skipped: 0,
        accounts_imported: 0,
        categories_imported: 0,
        errors: Vec::new(),
        total_parsed: data.transactions.len(),
    };

    let accounts = db::get_accounts(conn, user_id)?;
    let categories = db::get_categories(conn, user_id)?;

    for tx in data.transactions {
        let account_id = accounts
            .iter()
            .find(|a| a.name == tx.account_name)
            .map(|a| a.id)
            .or(options.default_account_id);

        let account_id = match account_id {
            Some(id) => id,
            None => {
                result.errors.push(crate::messages::tx_account_not_found(tx.id, &tx.account_name));
                continue;
            }
        };

        let category_id = tx.category_name.as_ref().and_then(|name| {
            categories.iter().find(|c| &c.name == name).map(|c| c.id)
        });

        let amount_abs = tx.amount.abs();
        if options.skip_duplicates {
            if db::transaction_duplicate_exists(
                conn,
                user_id,
                account_id,
                &tx.date,
                amount_abs,
                tx.note.as_deref(),
            )? {
                result.duplicates_skipped += 1;
                continue;
            }
        }

        match db::create_transaction(
            conn,
            user_id,
            account_id,
            category_id,
            amount_abs,
            &tx.transaction_type,
            tx.note.as_deref(),
            &tx.date,
        ) {
            Ok(_) => result.transactions_imported += 1,
            Err(e) => result.errors.push(crate::messages::tx_error(tx.id, &e.to_string())),
        }
    }

    Ok(result)
}

/// Import data from XLSX for the given user. First sheet: columns Дата, Тип, Сумма, Счёт, Категория, Заметка (B–G).
pub fn import_xlsx(
    conn: &Connection,
    user_id: i64,
    file_path: &Path,
    options: &ImportOptions,
) -> Result<ImportResult, String> {
    let mut workbook: Xlsx<_> = open_workbook(file_path).map_err(|e: calamine::XlsxError| e.to_string())?;
    let sheet_names = workbook.sheet_names().to_vec();
    let first_sheet = sheet_names.first().ok_or(crate::messages::ERR_NO_SHEETS)?;
    let range = workbook
        .worksheet_range(first_sheet)
        .map_err(|e| e.to_string())?;

    let mut result = ImportResult {
        transactions_imported: 0,
        duplicates_skipped: 0,
        accounts_imported: 0,
        categories_imported: 0,
        errors: Vec::new(),
        total_parsed: 0,
    };

    let accounts = db::get_accounts(conn, user_id)?;
    let categories = db::get_categories(conn, user_id)?;
    let (total_rows, total_cols) = range.get_size();
    if total_cols < 7 {
        return Ok(result);
    }

    for row_idx in 1..total_rows {
        result.total_parsed += 1;
        let date_str = cell_to_date_string(range.get((row_idx, 1)));
        let type_str = cell_to_string(range.get((row_idx, 2)));
        let amount_val = cell_to_f64(range.get((row_idx, 3)));
        let account_name = cell_to_string(range.get((row_idx, 4)));
        let category_name = cell_to_string(range.get((row_idx, 5)));
        let note_str = cell_to_string(range.get((row_idx, 6)));

        if date_str.is_empty() {
            continue;
        }
        let amount = amount_val.abs();
        if amount < 1e-9 {
            result.errors.push(format!("Строка {}: некорректная сумма", row_idx + 1));
            continue;
        }

        let transaction_type = if type_str.to_lowercase().contains("доход") || type_str.eq_ignore_ascii_case("income") {
            "income"
        } else {
            "expense"
        };

        let account_id = accounts
            .iter()
            .find(|a| a.name == account_name)
            .map(|a| a.id)
            .or(options.default_account_id);

        let account_id = match account_id {
            Some(id) => id,
            None => {
                result.errors.push(format!("Строка {}: счёт «{}» не найден", row_idx + 1, account_name));
                continue;
            }
        };

        let category_id = if category_name.is_empty() {
            None
        } else {
            categories.iter().find(|c| c.name == category_name).map(|c| c.id)
        };

        let note_opt = if note_str.is_empty() { None } else { Some(note_str.as_str()) };

        if options.skip_duplicates {
            if db::transaction_duplicate_exists(
                conn,
                user_id,
                account_id,
                &date_str,
                amount,
                note_opt,
            )? {
                result.duplicates_skipped += 1;
                continue;
            }
        }

        match db::create_transaction(
            conn,
            user_id,
            account_id,
            category_id,
            amount,
            transaction_type,
            note_opt,
            &date_str,
        ) {
            Ok(_) => result.transactions_imported += 1,
            Err(e) => result.errors.push(format!("Строка {}: {}", row_idx + 1, e)),
        }
    }

    Ok(result)
}

fn cell_to_string(cell: Option<&Data>) -> String {
    cell.and_then(|c| c.as_string()).unwrap_or_default()
}

/// Convert cell to date string YYYY-MM-DD. Handles string or Excel serial (float/int).
fn cell_to_date_string(cell: Option<&Data>) -> String {
    let Some(c) = cell else { return String::new() };
    if let Some(s) = c.get_string() {
        return s.trim().to_string();
    }
    if let Some(f) = c.get_float() {
        let days = f.trunc() as i64;
        if let Some(d) = NaiveDate::from_ymd_opt(1899, 12, 30).and_then(|d| d.checked_add_signed(Duration::days(days))) {
            return d.format("%Y-%m-%d").to_string();
        }
    }
    if let Some(i) = c.get_int() {
        if let Some(d) = NaiveDate::from_ymd_opt(1899, 12, 30).and_then(|d| d.checked_add_signed(Duration::days(i))) {
            return d.format("%Y-%m-%d").to_string();
        }
    }
    String::new()
}

fn cell_to_f64(cell: Option<&Data>) -> f64 {
    cell.and_then(|c| c.as_f64()).unwrap_or(0.0)
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
        crate::db::schema::seed_categories(&conn, 1).unwrap();
        conn
    }

    fn setup_db_with_data() -> Connection {
        let conn = setup_test_db();

        // Create account
        let account_id = db::create_account(&conn, 1, "Тестовый счёт", "card", "KZT", 0.0).unwrap();

        // Get a category for transactions
        let categories = db::get_categories(&conn, 1).unwrap();
        let expense_cat = categories.iter().find(|c| c.category_type == "expense").unwrap();

        // Create some transactions
        db::create_transaction(&conn, 1, account_id, Some(expense_cat.id), 1000.0, "expense", Some("Покупка 1"), "2024-01-15").unwrap();
        db::create_transaction(&conn, 1, account_id, Some(expense_cat.id), 500.0, "expense", Some("Покупка 2"), "2024-01-16").unwrap();
        db::create_transaction(&conn, 1, account_id, None, 5000.0, "income", Some("Зарплата"), "2024-01-10").unwrap();

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
            account_id: None,
            category_id: None,
        };

        export_data(&conn, 1, &options, &output_path).unwrap();

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
            account_id: None,
            category_id: None,
        };

        export_data(&conn, 1, &options, &output_path).unwrap();

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
            account_id: None,
            category_id: None,
        };

        export_data(&conn, 1, &options, &output_path).unwrap();

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
            account_id: None,
            category_id: None,
        };

        export_data(&conn, 1, &options, &output_path).unwrap();

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
            account_id: None,
            category_id: None,
        };

        export_data(&conn, 1, &options, &output_path).unwrap();

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
            account_id: None,
            category_id: None,
        };

        export_data(&conn, 1, &options, &output_path).unwrap();

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
            account_id: None,
            category_id: None,
        };

        export_data(&conn, 1, &options, &output_path).unwrap();

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
            account_id: None,
            category_id: None,
        };

        export_data(&conn, 1, &options, &output_path).unwrap();

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
        db::create_account(&conn, 1, "Тестовый счёт", "card", "KZT", 0.0).unwrap();

        let dir = tempdir().unwrap();
        let csv_path = dir.path().join("import.csv");

        // Create a valid CSV file
        let csv_content = "ID,Date,Type,Amount,Account,Category,Note
1,2024-02-01,expense,100.00,Тестовый счёт,Еда,Обед
2,2024-02-02,income,5000.00,Тестовый счёт,,Зарплата";

        std::fs::write(&csv_path, csv_content).unwrap();

        let result = import_csv(&conn, 1, &csv_path, &ImportOptions::default()).unwrap();

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

        let result = import_csv(&conn, 1, &csv_path, &ImportOptions::default()).unwrap();

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
        db::create_account(&conn, 1, "Тестовый счёт", "card", "KZT", 0.0).unwrap();

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

        let result = import_json(&conn, 1, &json_path, &ImportOptions::default()).unwrap();

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
            account_id: None,
            category_id: None,
        };

        let result = export_data(&conn, 1, &options, &output_path);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains(crate::messages::ERR_UNSUPPORTED_FORMAT_WITH));
    }
}
