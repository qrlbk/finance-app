use crate::bank_import::{self, ImportResult as BankImportResult, ImportTransaction, ParsedStatement};
use crate::db::{self, ensure_encrypted, get_db_path, open_connection};
use crate::export;
use crate::ml::{self, model::ModelStatus};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;
use std::time::{Duration, Instant};
use tauri::State;

const VALID_ACCOUNT_TYPES: &[&str] = &["cash", "card", "savings"];
const VALID_TRANSACTION_TYPES: &[&str] = &["income", "expense"];
const MAX_CURRENCY_LEN: usize = 10;

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
}

pub fn init_db_on_startup(app_handle: &tauri::AppHandle, state: &AppState) -> Result<(), String> {
    let db_path = get_db_path(app_handle)?;

    // Автоматически мигрируем незашифрованную БД если нужно
    ensure_encrypted(&db_path)?;

    let conn = open_connection(&db_path)?;
    db::init_db(&conn)?;
    *state.db_path.lock().unwrap() = Some(db_path);
    Ok(())
}

fn with_connection<F, T>(
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

#[tauri::command]
pub fn get_accounts(app_handle: tauri::AppHandle, state: State<AppState>) -> Result<Vec<db::Account>, String> {
    // Try cache first
    if let Some(cached) = state.cache.get_accounts() {
        return Ok(cached);
    }
    
    let accounts = with_connection(&app_handle, &state, db::get_accounts)?;
    state.cache.set_accounts(accounts.clone());
    Ok(accounts)
}

#[derive(Deserialize)]
pub struct CreateAccountInput {
    name: String,
    account_type: String,
    currency: Option<String>,
}

fn validate_account_input(
    name: &str,
    account_type: &str,
    currency: &str,
) -> Result<(), String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Название счёта не может быть пустым".to_string());
    }
    if !VALID_ACCOUNT_TYPES.contains(&account_type) {
        return Err("Недопустимый тип счёта".to_string());
    }
    let currency = currency.trim();
    if currency.is_empty() {
        return Err("Валюта не может быть пустой".to_string());
    }
    if currency.len() > MAX_CURRENCY_LEN {
        return Err("Слишком длинный код валюты".to_string());
    }
    Ok(())
}

#[tauri::command]
pub fn create_account(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    input: CreateAccountInput,
) -> Result<i64, String> {
    let currency = input.currency.unwrap_or_else(|| "KZT".to_string());
    validate_account_input(&input.name, &input.account_type, &currency)?;
    let name = input.name.trim().to_string();
    let result = with_connection(&app_handle, &state, |conn| {
        db::create_account(conn, &name, &input.account_type, &currency)
    })?;
    state.cache.invalidate_accounts();
    Ok(result)
}

#[derive(Deserialize)]
pub struct UpdateAccountInput {
    id: i64,
    name: String,
    account_type: String,
    currency: Option<String>,
}

#[tauri::command]
pub fn update_account(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    input: UpdateAccountInput,
) -> Result<(), String> {
    let currency = input.currency.unwrap_or_else(|| "KZT".to_string());
    validate_account_input(&input.name, &input.account_type, &currency)?;
    let name = input.name.trim().to_string();
    let result = with_connection(&app_handle, &state, |conn| {
        db::update_account(conn, input.id, &name, &input.account_type, &currency)
    })?;
    state.cache.invalidate_accounts();
    Ok(result)
}

#[tauri::command]
pub fn delete_account(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    id: i64,
) -> Result<(), String> {
    let result = with_connection(&app_handle, &state, |conn| db::delete_account(conn, id))?;
    state.cache.invalidate_accounts();
    Ok(result)
}

#[tauri::command]
pub fn get_categories(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
) -> Result<Vec<db::Category>, String> {
    // Try cache first
    if let Some(cached) = state.cache.get_categories() {
        return Ok(cached);
    }
    
    let categories = with_connection(&app_handle, &state, db::get_categories)?;
    state.cache.set_categories(categories.clone());
    Ok(categories)
}

#[derive(Deserialize, Default)]
pub struct GetTransactionsInput {
    limit: Option<i64>,
    account_id: Option<i64>,
    date_from: Option<String>,
    date_to: Option<String>,
    category_id: Option<i64>,
    transaction_type: Option<String>,
    search_note: Option<String>,
}

#[tauri::command]
pub fn get_transactions(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    input: Option<GetTransactionsInput>,
) -> Result<Vec<db::TransactionWithDetails>, String> {
    with_connection(&app_handle, &state, |conn| {
        let i = input.unwrap_or_default();
        let filters = db::TransactionFilters {
            limit: i.limit.unwrap_or(100),
            date_from: i.date_from,
            date_to: i.date_to,
            account_id: i.account_id,
            category_id: i.category_id,
            transaction_type: i.transaction_type,
            search_note: i.search_note,
        };
        db::get_transactions_filtered(conn, filters)
    })
}

#[derive(Deserialize)]
pub struct CreateTransactionInput {
    account_id: i64,
    category_id: Option<i64>,
    amount: f64,
    transaction_type: String,
    note: Option<String>,
    date: String,
}

fn validate_transaction_input(
    conn: &rusqlite::Connection,
    account_id: i64,
    category_id: Option<i64>,
    amount: f64,
    transaction_type: &str,
    date: &str,
) -> Result<(), String> {
    if amount <= 0.0 || amount.is_nan() {
        return Err("Сумма должна быть больше нуля".to_string());
    }
    if !VALID_TRANSACTION_TYPES.contains(&transaction_type) {
        return Err("Недопустимый тип транзакции".to_string());
    }
    NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map_err(|_| "Некорректная дата".to_string())?;
    if !db::account_exists(conn, account_id)? {
        return Err("Счёт не найден".to_string());
    }
    if let Some(cid) = category_id {
        if !db::category_exists_and_type(conn, cid, transaction_type)? {
            return Err("Категория не найдена или не совпадает с типом транзакции".to_string());
        }
    }
    Ok(())
}

#[tauri::command]
pub fn create_transaction(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    input: CreateTransactionInput,
) -> Result<i64, String> {
    with_connection(&app_handle, &state, |conn| {
        validate_transaction_input(
            conn,
            input.account_id,
            input.category_id,
            input.amount,
            &input.transaction_type,
            &input.date,
        )?;
        db::create_transaction(
            conn,
            input.account_id,
            input.category_id,
            input.amount,
            &input.transaction_type,
            input.note.as_deref(),
            &input.date,
        )
    })
}

#[derive(Deserialize)]
pub struct UpdateTransactionInput {
    id: i64,
    account_id: i64,
    category_id: Option<i64>,
    amount: f64,
    transaction_type: String,
    note: Option<String>,
    date: String,
}

#[tauri::command]
pub fn update_transaction(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    input: UpdateTransactionInput,
) -> Result<(), String> {
    with_connection(&app_handle, &state, |conn| {
        validate_transaction_input(
            conn,
            input.account_id,
            input.category_id,
            input.amount,
            &input.transaction_type,
            &input.date,
        )?;
        db::update_transaction(
            conn,
            input.id,
            input.account_id,
            input.category_id,
            input.amount,
            &input.transaction_type,
            input.note.as_deref(),
            &input.date,
        )
    })
}

#[tauri::command]
pub fn delete_transaction(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    id: i64,
) -> Result<(), String> {
    with_connection(&app_handle, &state, |conn| db::delete_transaction(conn, id))
}

#[tauri::command]
pub fn get_summary(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
) -> Result<db::Summary, String> {
    with_connection(&app_handle, &state, db::get_summary)
}

#[derive(Deserialize)]
pub struct GetExpenseByCategoryInput {
    year: i32,
    month: u32,
}

#[tauri::command]
pub fn get_expense_by_category(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    input: GetExpenseByCategoryInput,
) -> Result<Vec<db::CategoryTotal>, String> {
    with_connection(&app_handle, &state, |conn| {
        db::get_expense_by_category(conn, input.year, input.month)
    })
}

#[derive(Deserialize)]
pub struct GetMonthlyTotalsInput {
    months: Option<i32>,
}

#[tauri::command]
pub fn get_monthly_totals(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    input: Option<GetMonthlyTotalsInput>,
) -> Result<Vec<db::MonthlyTotal>, String> {
    let months = input.and_then(|i| i.months).unwrap_or(6);
    with_connection(&app_handle, &state, |conn| db::get_monthly_totals(conn, months))
}

#[tauri::command]
pub fn export_backup(app_handle: tauri::AppHandle) -> Result<String, String> {
    use std::fs;

    let db_path = get_db_path(&app_handle)?;
    if !db_path.exists() {
        return Err("База данных не инициализирована".to_string());
    }
    let backup_path = db_path
        .parent()
        .ok_or("Invalid path")?
        .join(format!("finance_backup_{}.db", chrono::Local::now().format("%Y%m%d_%H%M%S")));
    fs::copy(&db_path, &backup_path).map_err(|e| e.to_string())?;
    Ok(backup_path.to_string_lossy().to_string())
}

#[derive(Deserialize)]
pub struct CreateTransferInput {
    from_account_id: i64,
    to_account_id: i64,
    amount: f64,
    date: String,
    note: Option<String>,
}

#[tauri::command]
pub fn create_transfer(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    input: CreateTransferInput,
) -> Result<(), String> {
    with_connection(&app_handle, &state, |conn| {
        db::create_transfer(
            conn,
            input.from_account_id,
            input.to_account_id,
            input.amount,
            &input.date,
            input.note.as_deref(),
        )
    })
}

#[tauri::command]
pub fn restore_backup(app_handle: tauri::AppHandle, path: String) -> Result<(), String> {
    use std::fs;
    use std::io::Read;

    let backup_path = std::path::Path::new(&path);
    if !backup_path.exists() {
        return Err("Файл не найден".to_string());
    }
    if !backup_path.is_file() {
        return Err("Указанный путь не является файлом".to_string());
    }
    let mut header = [0u8; 16];
    let mut f = fs::File::open(backup_path).map_err(|e| e.to_string())?;
    f.read_exact(&mut header).map_err(|e| e.to_string())?;
    if !header.starts_with(b"SQLite format 3\0") {
        return Err("Файл не похож на базу SQLite".to_string());
    }

    let db_path = get_db_path(&app_handle)?;
    let old_path = db_path.parent().ok_or("Invalid path")?.join("finance.db.old");
    if db_path.exists() {
        fs::rename(&db_path, &old_path).map_err(|e| e.to_string())?;
    }
    fs::copy(backup_path, &db_path).map_err(|e| e.to_string())?;
    Ok(())
}

// ============================================================================
// ML Commands
// ============================================================================

/// Category prediction result
#[derive(Serialize)]
pub struct CategoryPrediction {
    pub category_id: i64,
    pub category_name: String,
    pub confidence: f64,
}

/// Model training result
#[derive(Serialize)]
pub struct TrainResult {
    pub success: bool,
    pub sample_count: usize,
    pub accuracy: Option<f64>,
    pub message: String,
}

/// Insights containing anomalies and forecast
#[derive(Serialize)]
pub struct Insights {
    pub anomalies: Vec<ml::Anomaly>,
    pub forecast: Option<ml::Forecast>,
    /// Number of months of expense data available (needed 3 for forecasting)
    pub months_of_data: i32,
}

/// Predict category for a transaction note
#[tauri::command]
pub fn predict_category(
    _app_handle: tauri::AppHandle,
    note: String,
    amount: Option<f64>,
    date: Option<String>,
) -> Result<Option<CategoryPrediction>, String> {
    let note = note.trim();
    if note.is_empty() || note.len() < 3 {
        return Ok(None);
    }

    let model_path = ml::model::get_model_path()?;
    
    if !model_path.exists() {
        return Ok(None);
    }

    let model = ml::CategoryModel::load(&model_path)?;
    
    // Use enhanced prediction with amount and date if available
    let prediction = model.predict_with_context(note, amount, date.as_deref());
    
    match prediction {
        Some((category_id, category_name, confidence)) => {
            // Only return prediction if confidence is above threshold
            if confidence >= 0.3 {
                Ok(Some(CategoryPrediction {
                    category_id,
                    category_name,
                    confidence,
                }))
            } else {
                Ok(None)
            }
        }
        None => Ok(None),
    }
}

/// Train the category prediction model from database
#[tauri::command]
pub fn train_model(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
) -> Result<TrainResult, String> {
    with_connection(&app_handle, &state, |conn| {
        match ml::ModelTrainer::train_from_db(conn) {
            Ok(model) => {
                let model_path = ml::model::get_model_path()?;
                model.save(&model_path)?;
                
                Ok(TrainResult {
                    success: true,
                    sample_count: model.sample_count,
                    accuracy: model.accuracy,
                    message: format!(
                        "Модель успешно обучена на {} транзакциях. Точность: {:.0}%",
                        model.sample_count,
                        model.accuracy.unwrap_or(0.0) * 100.0
                    ),
                })
            }
            Err(e) => {
                Ok(TrainResult {
                    success: false,
                    sample_count: 0,
                    accuracy: None,
                    message: e,
                })
            }
        }
    })
}

/// Get current model status
#[tauri::command]
pub fn get_model_status(
    _app_handle: tauri::AppHandle,
) -> Result<ModelStatus, String> {
    let model_path = ml::model::get_model_path()?;
    
    if !model_path.exists() {
        return Ok(ModelStatus::default());
    }

    match ml::CategoryModel::load(&model_path) {
        Ok(model) => Ok(model.status()),
        Err(_) => Ok(ModelStatus::default()),
    }
}

/// Get insights (anomalies and forecast)
#[tauri::command]
pub fn get_insights(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
) -> Result<Insights, String> {
    with_connection(&app_handle, &state, |conn| {
        // Count months of expense data
        let months_of_data: i32 = conn
            .query_row(
                "SELECT COUNT(DISTINCT strftime('%Y-%m', date)) 
                 FROM transactions 
                 WHERE type = 'expense' 
                   AND date >= date('now', '-12 months')",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        // Get anomalies for last 30 days
        let anomalies = ml::AnomalyDetector::detect_anomalies(conn, 30)
            .unwrap_or_default();

        // Get expense forecast
        let forecast = ml::ExpenseForecaster::forecast_next_month(conn).ok();

        Ok(Insights {
            anomalies,
            forecast,
            months_of_data,
        })
    })
}

/// Get smart insights (patterns, suggestions)
#[tauri::command]
pub fn get_smart_insights(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
) -> Result<ml::SmartInsights, String> {
    with_connection(&app_handle, &state, |conn| {
        ml::insights::analyze_spending_patterns(conn)
    })
}

/// Category forecast result
#[derive(Serialize)]
pub struct CategoryForecastResult {
    pub overall: ml::Forecast,
    pub by_category: Vec<ml::forecast::CategoryForecast>,
}

/// Get detailed forecast with category breakdown
#[tauri::command]
pub fn get_forecast_details(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
) -> Result<CategoryForecastResult, String> {
    with_connection(&app_handle, &state, |conn| {
        let (overall, by_category) = ml::ExpenseForecaster::forecast_with_categories(conn)?;
        Ok(CategoryForecastResult {
            overall,
            by_category,
        })
    })
}

// ============================================================================
// Recurring Payments Commands
// ============================================================================

#[tauri::command]
pub fn get_recurring_payments(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
) -> Result<Vec<db::RecurringPayment>, String> {
    with_connection(&app_handle, &state, db::get_recurring_payments)
}

#[derive(Deserialize)]
pub struct CreateRecurringInput {
    account_id: i64,
    category_id: Option<i64>,
    amount: f64,
    payment_type: String,
    frequency: String,
    next_date: String,
    end_date: Option<String>,
    note: Option<String>,
}

const VALID_FREQUENCIES: &[&str] = &["daily", "weekly", "monthly", "yearly"];

fn validate_recurring_input(
    conn: &rusqlite::Connection,
    input: &CreateRecurringInput,
) -> Result<(), String> {
    if input.amount <= 0.0 || input.amount.is_nan() {
        return Err("Сумма должна быть больше нуля".to_string());
    }
    if !VALID_TRANSACTION_TYPES.contains(&input.payment_type.as_str()) {
        return Err("Недопустимый тип платежа".to_string());
    }
    if !VALID_FREQUENCIES.contains(&input.frequency.as_str()) {
        return Err("Недопустимая частота".to_string());
    }
    NaiveDate::parse_from_str(&input.next_date, "%Y-%m-%d")
        .map_err(|_| "Некорректная дата следующего платежа".to_string())?;
    if let Some(end) = &input.end_date {
        NaiveDate::parse_from_str(end, "%Y-%m-%d")
            .map_err(|_| "Некорректная дата окончания".to_string())?;
    }
    if !db::account_exists(conn, input.account_id)? {
        return Err("Счёт не найден".to_string());
    }
    if let Some(cid) = input.category_id {
        if !db::category_exists_and_type(conn, cid, &input.payment_type)? {
            return Err("Категория не найдена или не совпадает с типом платежа".to_string());
        }
    }
    Ok(())
}

#[tauri::command]
pub fn create_recurring(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    input: CreateRecurringInput,
) -> Result<i64, String> {
    with_connection(&app_handle, &state, |conn| {
        validate_recurring_input(conn, &input)?;
        db::create_recurring(
            conn,
            input.account_id,
            input.category_id,
            input.amount,
            &input.payment_type,
            &input.frequency,
            &input.next_date,
            input.end_date.as_deref(),
            input.note.as_deref(),
        )
    })
}

#[derive(Deserialize)]
pub struct UpdateRecurringInput {
    id: i64,
    account_id: i64,
    category_id: Option<i64>,
    amount: f64,
    payment_type: String,
    frequency: String,
    next_date: String,
    end_date: Option<String>,
    note: Option<String>,
    is_active: bool,
}

#[tauri::command]
pub fn update_recurring(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    input: UpdateRecurringInput,
) -> Result<(), String> {
    with_connection(&app_handle, &state, |conn| {
        let create_input = CreateRecurringInput {
            account_id: input.account_id,
            category_id: input.category_id,
            amount: input.amount,
            payment_type: input.payment_type.clone(),
            frequency: input.frequency.clone(),
            next_date: input.next_date.clone(),
            end_date: input.end_date.clone(),
            note: input.note.clone(),
        };
        validate_recurring_input(conn, &create_input)?;
        db::update_recurring(
            conn,
            input.id,
            input.account_id,
            input.category_id,
            input.amount,
            &input.payment_type,
            &input.frequency,
            &input.next_date,
            input.end_date.as_deref(),
            input.note.as_deref(),
            input.is_active,
        )
    })
}

#[tauri::command]
pub fn delete_recurring(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    id: i64,
) -> Result<(), String> {
    with_connection(&app_handle, &state, |conn| db::delete_recurring(conn, id))
}

#[tauri::command]
pub fn process_recurring_payments(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
) -> Result<Vec<i64>, String> {
    with_connection(&app_handle, &state, db::process_due_recurring)
}

// ============================================================================
// Budget Commands
// ============================================================================

#[tauri::command]
pub fn get_budgets(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
) -> Result<Vec<db::Budget>, String> {
    with_connection(&app_handle, &state, db::get_budgets_with_spending)
}

#[derive(Deserialize)]
pub struct CreateBudgetInput {
    category_id: i64,
    amount: f64,
    period: String,
}

const VALID_PERIODS: &[&str] = &["weekly", "monthly", "yearly"];

#[tauri::command]
pub fn create_budget(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    input: CreateBudgetInput,
) -> Result<i64, String> {
    if input.amount <= 0.0 || input.amount.is_nan() {
        return Err("Сумма бюджета должна быть больше нуля".to_string());
    }
    if !VALID_PERIODS.contains(&input.period.as_str()) {
        return Err("Недопустимый период".to_string());
    }
    with_connection(&app_handle, &state, |conn| {
        db::create_budget(conn, input.category_id, input.amount, &input.period)
    })
}

#[derive(Deserialize)]
pub struct UpdateBudgetInput {
    id: i64,
    amount: f64,
}

#[tauri::command]
pub fn update_budget(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    input: UpdateBudgetInput,
) -> Result<(), String> {
    if input.amount <= 0.0 || input.amount.is_nan() {
        return Err("Сумма бюджета должна быть больше нуля".to_string());
    }
    with_connection(&app_handle, &state, |conn| {
        db::update_budget(conn, input.id, input.amount)
    })
}

#[tauri::command]
pub fn delete_budget(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    id: i64,
) -> Result<(), String> {
    with_connection(&app_handle, &state, |conn| db::delete_budget(conn, id))
}

#[tauri::command]
pub fn get_budget_alerts(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
) -> Result<Vec<db::BudgetAlert>, String> {
    with_connection(&app_handle, &state, db::check_budget_alerts)
}

// ============================================================================
// Category Management Commands
// ============================================================================

#[derive(Deserialize)]
pub struct CreateCategoryInput {
    name: String,
    category_type: String,
    icon: Option<String>,
    color: Option<String>,
    parent_id: Option<i64>,
}

#[tauri::command]
pub fn create_category(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    input: CreateCategoryInput,
) -> Result<i64, String> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err("Название категории не может быть пустым".to_string());
    }
    if !VALID_TRANSACTION_TYPES.contains(&input.category_type.as_str()) {
        return Err("Недопустимый тип категории".to_string());
    }
    let result = with_connection(&app_handle, &state, |conn| {
        db::create_category(
            conn,
            name,
            &input.category_type,
            input.icon.as_deref(),
            input.color.as_deref(),
            input.parent_id,
        )
    })?;
    state.cache.invalidate_categories();
    Ok(result)
}

#[derive(Deserialize)]
pub struct UpdateCategoryInput {
    id: i64,
    name: String,
    icon: Option<String>,
    color: Option<String>,
}

#[tauri::command]
pub fn update_category(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    input: UpdateCategoryInput,
) -> Result<(), String> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err("Название категории не может быть пустым".to_string());
    }
    let result = with_connection(&app_handle, &state, |conn| {
        db::update_category(conn, input.id, name, input.icon.as_deref(), input.color.as_deref())
    })?;
    state.cache.invalidate_categories();
    Ok(result)
}

#[tauri::command]
pub fn delete_category(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    id: i64,
) -> Result<(), String> {
    let result = with_connection(&app_handle, &state, |conn| db::delete_category(conn, id))?;
    state.cache.invalidate_categories();
    Ok(result)
}

// ============================================================================
// Export/Import Commands
// ============================================================================

#[derive(Deserialize)]
pub struct ExportDataInput {
    format: String,
    date_from: Option<String>,
    date_to: Option<String>,
    include_accounts: bool,
    include_categories: bool,
}

#[tauri::command]
pub fn export_data(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    input: ExportDataInput,
) -> Result<String, String> {
    let db_path = get_db_path(&app_handle)?;
    let export_dir = db_path.parent().ok_or("Invalid path")?;
    
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let extension = match input.format.as_str() {
        "csv" => "csv",
        "json" => "json",
        "xlsx" => "xlsx",
        _ => return Err("Unsupported format".to_string()),
    };
    let filename = format!("finance_export_{}.{}", timestamp, extension);
    let output_path = export_dir.join(&filename);
    
    let options = export::ExportOptions {
        format: input.format,
        date_from: input.date_from,
        date_to: input.date_to,
        include_accounts: input.include_accounts,
        include_categories: input.include_categories,
    };
    
    with_connection(&app_handle, &state, |conn| {
        export::export_data(conn, &options, &output_path)?;
        Ok(output_path.to_string_lossy().to_string())
    })
}

#[derive(Deserialize)]
pub struct ImportDataInput {
    path: String,
    format: String,
}

#[tauri::command]
pub fn import_data(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    input: ImportDataInput,
) -> Result<export::ImportResult, String> {
    let file_path = PathBuf::from(&input.path);
    if !file_path.exists() {
        return Err("File not found".to_string());
    }
    
    with_connection(&app_handle, &state, |conn| {
        match input.format.as_str() {
            "csv" => export::import_csv(conn, &file_path),
            "json" => export::import_json(conn, &file_path),
            _ => Err("Unsupported format".to_string()),
        }
    })
}

/// Open a file with the default system application
#[tauri::command]
pub fn open_file(path: String) -> Result<(), String> {
    let path = PathBuf::from(&path);
    if !path.exists() {
        return Err("Файл не найден".to_string());
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Не удалось открыть файл: {}", e))?;
    }
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", path.to_str().unwrap_or("")])
            .spawn()
            .map_err(|e| format!("Не удалось открыть файл: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Не удалось открыть файл: {}", e))?;
    }
    
    Ok(())
}

// ============================================================================
// Bank Statement Import Commands
// ============================================================================

/// Parse a bank statement PDF file
#[tauri::command]
pub fn parse_bank_statement(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    path: String,
) -> Result<ParsedStatement, String> {
    // Validate file exists
    let file_path = PathBuf::from(&path);
    if !file_path.exists() {
        return Err("Файл не найден".to_string());
    }
    
    // Check file extension
    let extension = file_path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());
    
    if extension != Some("pdf".to_string()) {
        return Err("Поддерживаются только PDF файлы".to_string());
    }
    
    // Parse the statement
    let mut statement = bank_import::parse_statement(&path)?;
    
    // Enhance with ML category predictions and duplicate detection
    with_connection(&app_handle, &state, |conn| {
        for tx in &mut statement.transactions {
            // ML category prediction
            if let Ok(Some(prediction)) = predict_category_internal(&tx.description) {
                tx.suggested_category_id = Some(prediction.category_id);
                tx.confidence = Some(prediction.confidence);
            }
            
            // Duplicate detection
            tx.is_duplicate = check_duplicate(conn, &tx.date, tx.amount, &tx.description)?;
        }
        Ok(())
    })?;
    
    Ok(statement)
}

/// Internal helper for category prediction without Tauri state
fn predict_category_internal(note: &str) -> Result<Option<CategoryPrediction>, String> {
    let note = note.trim();
    if note.is_empty() || note.len() < 3 {
        return Ok(None);
    }

    let model_path = ml::model::get_model_path()?;
    
    if !model_path.exists() {
        return Ok(None);
    }

    let model = ml::CategoryModel::load(&model_path)?;
    let prediction = model.predict_with_context(note, None, None);
    
    match prediction {
        Some((category_id, category_name, confidence)) => {
            if confidence >= 0.3 {
                Ok(Some(CategoryPrediction {
                    category_id,
                    category_name,
                    confidence,
                }))
            } else {
                Ok(None)
            }
        }
        None => Ok(None),
    }
}

/// Check if a transaction is a duplicate
fn check_duplicate(
    conn: &rusqlite::Connection,
    date: &str,
    amount: f64,
    description: &str,
) -> Result<bool, String> {
    // Get first N characters safely (UTF-8 aware)
    let desc_prefix: String = description.chars().take(15).collect();

    // Compare absolute values of amounts since expenses are stored as negative
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM transactions 
         WHERE date = ? AND ABS(ABS(amount) - ?) < 0.01 
         AND (note = ? OR note LIKE ?)",
        rusqlite::params![date, amount.abs(), description, format!("%{}%", desc_prefix)],
        |row| row.get(0),
    ).unwrap_or(0);
    
    Ok(count > 0)
}

/// Input for importing bank transactions
#[derive(Deserialize)]
pub struct ImportBankTransactionsInput {
    pub transactions: Vec<ImportTransaction>,
    pub account_id: i64,
    pub skip_duplicates: bool,
}

/// Import parsed bank transactions into the database
#[tauri::command]
pub fn import_bank_transactions(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    input: ImportBankTransactionsInput,
) -> Result<BankImportResult, String> {
    with_connection(&app_handle, &state, |conn| {
        // Validate account exists
        if !db::account_exists(conn, input.account_id)? {
            return Err("Счёт не найден".to_string());
        }
        
        let mut imported = 0;
        let mut skipped_duplicates = 0;
        let mut failed = 0;
        let mut errors = Vec::new();
        
        for tx in input.transactions {
            // Check for duplicate if requested
            if input.skip_duplicates && tx.skip_if_duplicate {
                let is_dup = check_duplicate(conn, &tx.date, tx.amount, &tx.description)?;
                if is_dup {
                    skipped_duplicates += 1;
                    continue;
                }
            }
            
            // Validate category if provided
            if let Some(cat_id) = tx.category_id {
                if !db::category_exists_and_type(conn, cat_id, &tx.transaction_type)? {
                    errors.push(format!(
                        "Категория {} не найдена для транзакции {}",
                        cat_id, tx.description
                    ));
                    failed += 1;
                    continue;
                }
            }
            
            // Create the transaction
            match db::create_transaction(
                conn,
                input.account_id,
                tx.category_id,
                tx.amount,
                &tx.transaction_type,
                Some(&tx.description),
                &tx.date,
            ) {
                Ok(_) => imported += 1,
                Err(e) => {
                    errors.push(format!("Ошибка импорта {}: {}", tx.description, e));
                    failed += 1;
                }
            }
        }
        
        Ok(BankImportResult {
            imported,
            skipped_duplicates,
            failed,
            errors,
        })
    })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use std::thread;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::schema::create_tables(&conn).unwrap();
        crate::db::schema::seed_categories(&conn).unwrap();
        conn
    }

    // =====================================================================
    // Account Validation Tests
    // =====================================================================

    #[test]
    fn test_validate_account_input_empty_name() {
        let result = validate_account_input("", "card", "KZT");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("пустым"));
    }

    #[test]
    fn test_validate_account_input_whitespace_name() {
        let result = validate_account_input("   ", "card", "KZT");
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

    #[test]
    fn test_validate_account_input_empty_currency() {
        let result = validate_account_input("Карта", "card", "");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Валюта"));
    }

    #[test]
    fn test_validate_account_input_currency_too_long() {
        let result = validate_account_input("Карта", "card", "VERY_LONG_CURRENCY_CODE");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("длинный"));
    }

    #[test]
    fn test_validate_account_input_all_types() {
        for account_type in VALID_ACCOUNT_TYPES {
            let result = validate_account_input("Тест", account_type, "KZT");
            assert!(result.is_ok(), "Type {} should be valid", account_type);
        }
    }

    // =====================================================================
    // Transaction Validation Tests
    // =====================================================================

    #[test]
    fn test_validate_transaction_input_zero_amount() {
        let conn = setup_test_db();
        crate::db::create_account(&conn, "Тест", "card", "KZT").unwrap();
        
        let result = validate_transaction_input(&conn, 1, None, 0.0, "income", "2024-01-01");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("больше нуля"));
    }

    #[test]
    fn test_validate_transaction_input_negative_amount() {
        let conn = setup_test_db();
        crate::db::create_account(&conn, "Тест", "card", "KZT").unwrap();
        
        let result = validate_transaction_input(&conn, 1, None, -100.0, "income", "2024-01-01");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("больше нуля"));
    }

    #[test]
    fn test_validate_transaction_input_nan_amount() {
        let conn = setup_test_db();
        crate::db::create_account(&conn, "Тест", "card", "KZT").unwrap();
        
        let result = validate_transaction_input(&conn, 1, None, f64::NAN, "income", "2024-01-01");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_transaction_input_invalid_type() {
        let conn = setup_test_db();
        crate::db::create_account(&conn, "Тест", "card", "KZT").unwrap();
        
        let result = validate_transaction_input(&conn, 1, None, 100.0, "invalid", "2024-01-01");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("тип"));
    }

    #[test]
    fn test_validate_transaction_input_invalid_date() {
        let conn = setup_test_db();
        crate::db::create_account(&conn, "Тест", "card", "KZT").unwrap();
        
        let result = validate_transaction_input(&conn, 1, None, 100.0, "income", "invalid-date");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("дата"));
    }

    #[test]
    fn test_validate_transaction_input_wrong_date_format() {
        let conn = setup_test_db();
        crate::db::create_account(&conn, "Тест", "card", "KZT").unwrap();
        
        let result = validate_transaction_input(&conn, 1, None, 100.0, "income", "01-01-2024");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_transaction_input_nonexistent_account() {
        let conn = setup_test_db();
        
        let result = validate_transaction_input(&conn, 9999, None, 100.0, "income", "2024-01-01");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Счёт не найден"));
    }

    #[test]
    fn test_validate_transaction_input_category_type_mismatch() {
        let conn = setup_test_db();
        crate::db::create_account(&conn, "Тест", "card", "KZT").unwrap();
        
        // Get an expense category
        let categories = crate::db::get_categories(&conn).unwrap();
        let expense_cat = categories.iter().find(|c| c.category_type == "expense").unwrap();
        
        // Try to use expense category for income transaction
        let result = validate_transaction_input(&conn, 1, Some(expense_cat.id), 100.0, "income", "2024-01-01");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Категория"));
    }

    #[test]
    fn test_validate_transaction_input_valid_with_category() {
        let conn = setup_test_db();
        crate::db::create_account(&conn, "Тест", "card", "KZT").unwrap();
        
        // Get an expense category
        let categories = crate::db::get_categories(&conn).unwrap();
        let expense_cat = categories.iter().find(|c| c.category_type == "expense").unwrap();
        
        let result = validate_transaction_input(&conn, 1, Some(expense_cat.id), 100.0, "expense", "2024-01-01");
        assert!(result.is_ok());
    }

    // =====================================================================
    // Recurring Validation Tests
    // =====================================================================

    #[test]
    fn test_validate_recurring_input_invalid_frequency() {
        let conn = setup_test_db();
        let account_id = crate::db::create_account(&conn, "Тест", "card", "KZT").unwrap();
        
        let input = CreateRecurringInput {
            account_id,
            category_id: None,
            amount: 100.0,
            payment_type: "expense".to_string(),
            frequency: "invalid_frequency".to_string(),
            next_date: "2024-02-01".to_string(),
            end_date: None,
            note: None,
        };
        
        let result = validate_recurring_input(&conn, &input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("частота"));
    }

    #[test]
    fn test_validate_recurring_input_all_frequencies() {
        let conn = setup_test_db();
        let account_id = crate::db::create_account(&conn, "Тест", "card", "KZT").unwrap();
        
        for frequency in VALID_FREQUENCIES {
            let input = CreateRecurringInput {
                account_id,
                category_id: None,
                amount: 100.0,
                payment_type: "expense".to_string(),
                frequency: frequency.to_string(),
                next_date: "2024-02-01".to_string(),
                end_date: None,
                note: None,
            };
            
            let result = validate_recurring_input(&conn, &input);
            assert!(result.is_ok(), "Frequency {} should be valid", frequency);
        }
    }

    #[test]
    fn test_validate_recurring_input_zero_amount() {
        let conn = setup_test_db();
        let account_id = crate::db::create_account(&conn, "Тест", "card", "KZT").unwrap();
        
        let input = CreateRecurringInput {
            account_id,
            category_id: None,
            amount: 0.0,
            payment_type: "expense".to_string(),
            frequency: "monthly".to_string(),
            next_date: "2024-02-01".to_string(),
            end_date: None,
            note: None,
        };
        
        let result = validate_recurring_input(&conn, &input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("больше нуля"));
    }

    #[test]
    fn test_validate_recurring_input_invalid_end_date() {
        let conn = setup_test_db();
        let account_id = crate::db::create_account(&conn, "Тест", "card", "KZT").unwrap();
        
        let input = CreateRecurringInput {
            account_id,
            category_id: None,
            amount: 100.0,
            payment_type: "expense".to_string(),
            frequency: "monthly".to_string(),
            next_date: "2024-02-01".to_string(),
            end_date: Some("invalid-date".to_string()),
            note: None,
        };
        
        let result = validate_recurring_input(&conn, &input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("дата окончания"));
    }

    // =====================================================================
    // Cache Tests
    // =====================================================================

    #[test]
    fn test_cache_entry_creation() {
        let entry = CacheEntry::new(vec![1, 2, 3], Duration::from_secs(60));
        assert!(entry.is_valid());
        assert_eq!(entry.get(), Some(vec![1, 2, 3]));
    }

    #[test]
    fn test_cache_entry_expiration() {
        let entry = CacheEntry::new(vec![1, 2, 3], Duration::from_millis(50));
        assert!(entry.is_valid());
        
        thread::sleep(Duration::from_millis(60));
        
        assert!(!entry.is_valid());
        assert!(entry.get().is_none());
    }

    #[test]
    fn test_query_cache_categories_empty() {
        let cache = QueryCache::new();
        assert!(cache.get_categories().is_none());
    }

    #[test]
    fn test_query_cache_categories_set_get() {
        let cache = QueryCache::new();
        let categories = vec![db::Category {
            id: 1,
            name: "Тест".to_string(),
            category_type: "expense".to_string(),
            icon: None,
            color: None,
        }];
        
        cache.set_categories(categories.clone());
        
        let cached = cache.get_categories();
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().len(), 1);
    }

    #[test]
    fn test_query_cache_invalidation() {
        let cache = QueryCache::new();
        let categories = vec![db::Category {
            id: 1,
            name: "Тест".to_string(),
            category_type: "expense".to_string(),
            icon: None,
            color: None,
        }];
        
        cache.set_categories(categories);
        assert!(cache.get_categories().is_some());
        
        cache.invalidate_categories();
        assert!(cache.get_categories().is_none());
    }

    #[test]
    fn test_query_cache_accounts() {
        let cache = QueryCache::new();
        let accounts = vec![db::Account {
            id: 1,
            name: "Карта".to_string(),
            account_type: "card".to_string(),
            balance: 1000.0,
            currency: "KZT".to_string(),
        }];
        
        cache.set_accounts(accounts.clone());
        
        let cached = cache.get_accounts();
        assert!(cached.is_some());
        assert_eq!(cached.unwrap()[0].balance, 1000.0);
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

    // =====================================================================
    // Duplicate Detection Tests
    // =====================================================================

    #[test]
    fn test_check_duplicate_no_match() {
        let conn = setup_test_db();
        let account_id = crate::db::create_account(&conn, "Тест", "card", "KZT").unwrap();
        
        // Create a transaction
        crate::db::create_transaction(&conn, account_id, None, 100.0, "expense", Some("Покупка"), "2024-01-15").unwrap();
        
        // Check for non-matching transaction
        let is_dup = check_duplicate(&conn, "2024-01-20", 200.0, "Другая покупка").unwrap();
        assert!(!is_dup);
    }

    #[test]
    fn test_check_duplicate_exact_match() {
        let conn = setup_test_db();
        let account_id = crate::db::create_account(&conn, "Тест", "card", "KZT").unwrap();
        
        // Create a transaction
        crate::db::create_transaction(&conn, account_id, None, 100.0, "expense", Some("Покупка в магазине"), "2024-01-15").unwrap();
        
        // Check for exact match (same date, amount, and similar note)
        let is_dup = check_duplicate(&conn, "2024-01-15", 100.0, "Покупка в магазине").unwrap();
        assert!(is_dup);
    }

    #[test]
    fn test_check_duplicate_different_date() {
        let conn = setup_test_db();
        let account_id = crate::db::create_account(&conn, "Тест", "card", "KZT").unwrap();
        
        crate::db::create_transaction(&conn, account_id, None, 100.0, "expense", Some("Покупка"), "2024-01-15").unwrap();
        
        // Different date - should not be a duplicate
        let is_dup = check_duplicate(&conn, "2024-01-16", 100.0, "Покупка").unwrap();
        assert!(!is_dup);
    }

    #[test]
    fn test_check_duplicate_different_amount() {
        let conn = setup_test_db();
        let account_id = crate::db::create_account(&conn, "Тест", "card", "KZT").unwrap();
        
        crate::db::create_transaction(&conn, account_id, None, 100.0, "expense", Some("Покупка"), "2024-01-15").unwrap();
        
        // Different amount - should not be a duplicate
        let is_dup = check_duplicate(&conn, "2024-01-15", 150.0, "Покупка").unwrap();
        assert!(!is_dup);
    }

    // =====================================================================
    // Valid Types Tests
    // =====================================================================

    #[test]
    fn test_valid_account_types() {
        assert!(VALID_ACCOUNT_TYPES.contains(&"cash"));
        assert!(VALID_ACCOUNT_TYPES.contains(&"card"));
        assert!(VALID_ACCOUNT_TYPES.contains(&"savings"));
        assert!(!VALID_ACCOUNT_TYPES.contains(&"invalid"));
    }

    #[test]
    fn test_valid_transaction_types() {
        assert!(VALID_TRANSACTION_TYPES.contains(&"income"));
        assert!(VALID_TRANSACTION_TYPES.contains(&"expense"));
        assert!(!VALID_TRANSACTION_TYPES.contains(&"transfer"));
    }

    #[test]
    fn test_valid_frequencies() {
        assert!(VALID_FREQUENCIES.contains(&"daily"));
        assert!(VALID_FREQUENCIES.contains(&"weekly"));
        assert!(VALID_FREQUENCIES.contains(&"monthly"));
        assert!(VALID_FREQUENCIES.contains(&"yearly"));
        assert!(!VALID_FREQUENCIES.contains(&"hourly"));
    }
}
