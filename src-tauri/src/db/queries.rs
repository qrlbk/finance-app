use chrono::Datelike;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Account {
    pub id: i64,
    pub name: String,
    pub account_type: String,
    pub balance: f64,
    pub currency: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub category_type: String,
    pub icon: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Transaction {
    pub id: i64,
    pub account_id: i64,
    pub category_id: Option<i64>,
    pub amount: f64,
    pub transaction_type: String,
    pub note: Option<String>,
    pub date: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransactionWithDetails {
    pub id: i64,
    pub account_id: i64,
    pub account_name: String,
    pub category_id: Option<i64>,
    pub category_name: Option<String>,
    pub amount: f64,
    pub transaction_type: String,
    pub note: Option<String>,
    pub date: String,
}

#[derive(Debug, Serialize)]
pub struct Summary {
    pub total_balance: f64,
    pub income_month: f64,
    pub expense_month: f64,
}

pub fn get_accounts(conn: &Connection) -> Result<Vec<Account>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, type as account_type, balance, currency FROM accounts ORDER BY name",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Account {
                id: row.get(0)?,
                name: row.get(1)?,
                account_type: row.get(2)?,
                balance: row.get(3)?,
                currency: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

pub fn create_account(
    conn: &Connection,
    name: &str,
    account_type: &str,
    currency: &str,
) -> Result<i64, String> {
    conn.execute(
        "INSERT INTO accounts (name, type, currency) VALUES (?1, ?2, ?3)",
        [name, account_type, currency],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

pub fn update_account(
    conn: &Connection,
    id: i64,
    name: &str,
    account_type: &str,
    currency: &str,
) -> Result<(), String> {
    conn.execute(
        "UPDATE accounts SET name = ?1, type = ?2, currency = ?3 WHERE id = ?4",
        rusqlite::params![name, account_type, currency, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn delete_account(conn: &Connection, id: i64) -> Result<(), String> {
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM transactions WHERE account_id = ?1", [id], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    if count > 0 {
        return Err("Нельзя удалить счёт с транзакциями. Сначала удалите или переместите транзакции.".to_string());
    }
    conn.execute("DELETE FROM accounts WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Returns true if an account with the given id exists.
pub fn account_exists(conn: &Connection, id: i64) -> Result<bool, String> {
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM accounts WHERE id = ?1", [id], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    Ok(count > 0)
}

/// Returns true if category exists and its type matches expected_type (income/expense).
pub fn category_exists_and_type(
    conn: &Connection,
    category_id: i64,
    expected_type: &str,
) -> Result<bool, String> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM categories WHERE id = ?1 AND type = ?2",
            rusqlite::params![category_id, expected_type],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    Ok(count > 0)
}

pub fn get_categories(conn: &Connection) -> Result<Vec<Category>, String> {
    let mut stmt = conn
        .prepare("SELECT id, name, type as category_type, icon, color FROM categories ORDER BY type, name")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Category {
                id: row.get(0)?,
                name: row.get(1)?,
                category_type: row.get(2)?,
                icon: row.get(3)?,
                color: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

pub struct TransactionFilters {
    pub limit: i64,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub account_id: Option<i64>,
    pub category_id: Option<i64>,
    pub transaction_type: Option<String>,
    pub search_note: Option<String>,
}

#[allow(dead_code)]
pub fn get_transactions(
    conn: &Connection,
    limit: Option<i64>,
    account_id: Option<i64>,
) -> Result<Vec<TransactionWithDetails>, String> {
    get_transactions_filtered(
        conn,
        TransactionFilters {
            limit: limit.unwrap_or(100),
            date_from: None,
            date_to: None,
            account_id,
            category_id: None,
            transaction_type: None,
            search_note: None,
        },
    )
}

pub fn get_transactions_filtered(
    conn: &Connection,
    filters: TransactionFilters,
) -> Result<Vec<TransactionWithDetails>, String> {
    let limit = filters.limit.min(500);
    let note_pattern = filters
        .search_note
        .as_ref()
        .and_then(|s| {
            let t = s.trim();
            if t.is_empty() {
                None
            } else {
                Some(format!("%{}%", t.replace('%', "\\%").replace('_', "\\_")))
            }
        });

    let sql = "SELECT t.id, t.account_id, a.name as account_name, t.category_id, c.name as category_name, 
         t.amount, t.type as transaction_type, t.note, t.date 
         FROM transactions t 
         LEFT JOIN accounts a ON t.account_id = a.id 
         LEFT JOIN categories c ON t.category_id = c.id 
         WHERE (t.date >= ?1 OR ?1 IS NULL) 
         AND (t.date <= ?2 OR ?2 IS NULL) 
         AND (t.account_id = ?3 OR ?3 IS NULL) 
         AND (t.category_id = ?4 OR ?4 IS NULL) 
         AND (t.type = ?5 OR ?5 IS NULL) 
         AND (t.note LIKE ?6 ESCAPE '\\' OR ?6 IS NULL) 
         ORDER BY t.date DESC, t.id DESC LIMIT ?7";

    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(
        rusqlite::params![
            filters.date_from,
            filters.date_to,
            filters.account_id,
            filters.category_id,
            filters.transaction_type,
            note_pattern,
            limit,
        ],
        map_transaction_row,
    );
    let rows = rows.map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

fn map_transaction_row(row: &rusqlite::Row) -> rusqlite::Result<TransactionWithDetails> {
    Ok(TransactionWithDetails {
        id: row.get(0)?,
        account_id: row.get(1)?,
        account_name: row.get(2)?,
        category_id: row.get(3)?,
        category_name: row.get(4)?,
        amount: row.get(5)?,
        transaction_type: row.get(6)?,
        note: row.get(7)?,
        date: row.get(8)?,
    })
}

pub fn create_transaction(
    conn: &Connection,
    account_id: i64,
    category_id: Option<i64>,
    amount: f64,
    transaction_type: &str,
    note: Option<&str>,
    date: &str,
) -> Result<i64, String> {
    let mut actual_amount = amount;
    if transaction_type == "expense" {
        actual_amount = -amount.abs();
    } else if transaction_type == "income" {
        actual_amount = amount.abs();
    }

    conn.execute(
        "INSERT INTO transactions (account_id, category_id, amount, type, note, date) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![account_id, category_id, actual_amount, transaction_type, note, date],
    )
    .map_err(|e| e.to_string())?;

    let tx_id = conn.last_insert_rowid();

    // Update account balance
    conn.execute(
        "UPDATE accounts SET balance = balance + ?1 WHERE id = ?2",
        rusqlite::params![actual_amount, account_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(tx_id)
}

pub fn update_transaction(
    conn: &Connection,
    id: i64,
    account_id: i64,
    category_id: Option<i64>,
    amount: f64,
    transaction_type: &str,
    note: Option<&str>,
    date: &str,
) -> Result<(), String> {
    let (old_amount, old_account_id): (f64, i64) = conn
        .query_row(
            "SELECT amount, account_id FROM transactions WHERE id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|e| e.to_string())?;

    // Revert old balance
    conn.execute(
        "UPDATE accounts SET balance = balance - ?1 WHERE id = ?2",
        rusqlite::params![old_amount, old_account_id],
    )
    .map_err(|e| e.to_string())?;

    let mut actual_amount = amount;
    if transaction_type == "expense" {
        actual_amount = -amount.abs();
    } else if transaction_type == "income" {
        actual_amount = amount.abs();
    }

    conn.execute(
        "UPDATE transactions SET account_id = ?1, category_id = ?2, amount = ?3, type = ?4, note = ?5, date = ?6 WHERE id = ?7",
        rusqlite::params![account_id, category_id, actual_amount, transaction_type, note, date, id],
    )
    .map_err(|e| e.to_string())?;

    // Apply new balance
    conn.execute(
        "UPDATE accounts SET balance = balance + ?1 WHERE id = ?2",
        rusqlite::params![actual_amount, account_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn create_transfer(
    conn: &Connection,
    from_account_id: i64,
    to_account_id: i64,
    amount: f64,
    date: &str,
    note: Option<&str>,
) -> Result<(), String> {
    if from_account_id == to_account_id {
        return Err("Счёт списания и счёт зачисления не должны совпадать".to_string());
    }
    if amount <= 0.0 || amount.is_nan() {
        return Err("Сумма должна быть больше нуля".to_string());
    }
    if !account_exists(conn, from_account_id)? {
        return Err("Счёт списания не найден".to_string());
    }
    if !account_exists(conn, to_account_id)? {
        return Err("Счёт зачисления не найден".to_string());
    }
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    create_transaction(
        &tx,
        from_account_id,
        None,
        amount,
        "expense",
        note,
        date,
    )?;
    create_transaction(&tx, to_account_id, None, amount, "income", note, date)?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn delete_transaction(conn: &Connection, id: i64) -> Result<(), String> {
    let (amount, account_id): (f64, i64) = conn
        .query_row(
            "SELECT amount, account_id FROM transactions WHERE id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM transactions WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE accounts SET balance = balance - ?1 WHERE id = ?2",
        rusqlite::params![amount, account_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn get_summary(conn: &Connection) -> Result<Summary, String> {
    let total_balance: f64 = conn
        .query_row("SELECT COALESCE(SUM(balance), 0) FROM accounts", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;

    let now = chrono::Local::now();
    let month_start = format!("{}-{:02}-01", now.year(), now.month());

    let income_month: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM transactions WHERE type = 'income' AND date >= ?1",
            [&month_start],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;

    let expense_month: f64 = conn
        .query_row(
            "SELECT COALESCE(ABS(SUM(amount)), 0) FROM transactions WHERE type = 'expense' AND date >= ?1",
            [&month_start],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;

    Ok(Summary {
        total_balance,
        income_month,
        expense_month,
    })
}

#[derive(Debug, Serialize)]
pub struct CategoryTotal {
    pub category_name: String,
    pub total: f64,
}

#[derive(Debug, Serialize)]
pub struct MonthlyTotal {
    pub month: String,
    pub income: f64,
    pub expense: f64,
}

pub fn get_expense_by_category(
    conn: &Connection,
    year: i32,
    month: u32,
) -> Result<Vec<CategoryTotal>, String> {
    let month_start = format!("{}-{:02}-01", year, month);
    let month_end = if month == 12 {
        format!("{}-01-01", year + 1)
    } else {
        format!("{}-{:02}-01", year, month + 1)
    };

    let mut stmt = conn
        .prepare(
            "SELECT COALESCE(c.name, 'Без категории') as name, ABS(SUM(t.amount)) as total
             FROM transactions t
             LEFT JOIN categories c ON t.category_id = c.id
             WHERE t.type = 'expense' AND t.date >= ?1 AND t.date < ?2
             GROUP BY t.category_id
             ORDER BY total DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(rusqlite::params![&month_start, &month_end], |row| {
            Ok(CategoryTotal {
                category_name: row.get(0)?,
                total: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

pub fn get_monthly_totals(
    conn: &Connection,
    months: i32,
) -> Result<Vec<MonthlyTotal>, String> {
    use chrono::Months;
    let now = chrono::Local::now();
    let mut results = Vec::new();

    for i in 0..months {
        let Some(d) = now.date_naive().checked_sub_months(Months::new(i as u32)) else {
            continue;
        };
        let y = d.year();
        let m = d.month();
        let month_start = format!("{}-{:02}-01", y, m);
        let month_end = if m == 12 {
            format!("{}-12-31", y)
        } else {
            format!("{}-{:02}-01", y, m + 1)
        };

        let income: f64 = conn
            .query_row(
                "SELECT COALESCE(SUM(amount), 0) FROM transactions WHERE type = 'income' AND date >= ?1 AND date < ?2",
                rusqlite::params![&month_start, &month_end],
                |r| r.get(0),
            )
            .unwrap_or(0.0);

        let expense: f64 = conn
            .query_row(
                "SELECT COALESCE(ABS(SUM(amount)), 0) FROM transactions WHERE type = 'expense' AND date >= ?1 AND date < ?2",
                rusqlite::params![&month_start, &month_end],
                |r| r.get(0),
            )
            .unwrap_or(0.0);

        results.push(MonthlyTotal {
            month: format!("{:02}.{}", m, y),
            income,
            expense,
        });
    }

    results.reverse();
    Ok(results)
}

// ============================================================================
// Recurring Payments
// ============================================================================

#[derive(Debug, Serialize)]
pub struct RecurringPayment {
    pub id: i64,
    pub account_id: i64,
    pub account_name: String,
    pub category_id: Option<i64>,
    pub category_name: Option<String>,
    pub amount: f64,
    pub payment_type: String,
    pub frequency: String,
    pub next_date: String,
    pub end_date: Option<String>,
    pub note: Option<String>,
    pub is_active: bool,
}

pub fn get_recurring_payments(conn: &Connection) -> Result<Vec<RecurringPayment>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT r.id, r.account_id, a.name as account_name, r.category_id, c.name as category_name,
                    r.amount, r.type as payment_type, r.frequency, r.next_date, r.end_date, r.note, r.is_active
             FROM recurring r
             LEFT JOIN accounts a ON r.account_id = a.id
             LEFT JOIN categories c ON r.category_id = c.id
             ORDER BY r.next_date ASC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(RecurringPayment {
                id: row.get(0)?,
                account_id: row.get(1)?,
                account_name: row.get(2)?,
                category_id: row.get(3)?,
                category_name: row.get(4)?,
                amount: row.get(5)?,
                payment_type: row.get(6)?,
                frequency: row.get(7)?,
                next_date: row.get(8)?,
                end_date: row.get(9)?,
                note: row.get(10)?,
                is_active: row.get::<_, i64>(11)? == 1,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

pub fn create_recurring(
    conn: &Connection,
    account_id: i64,
    category_id: Option<i64>,
    amount: f64,
    payment_type: &str,
    frequency: &str,
    next_date: &str,
    end_date: Option<&str>,
    note: Option<&str>,
) -> Result<i64, String> {
    conn.execute(
        "INSERT INTO recurring (account_id, category_id, amount, type, frequency, next_date, end_date, note, is_active)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1)",
        rusqlite::params![account_id, category_id, amount, payment_type, frequency, next_date, end_date, note],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

pub fn update_recurring(
    conn: &Connection,
    id: i64,
    account_id: i64,
    category_id: Option<i64>,
    amount: f64,
    payment_type: &str,
    frequency: &str,
    next_date: &str,
    end_date: Option<&str>,
    note: Option<&str>,
    is_active: bool,
) -> Result<(), String> {
    conn.execute(
        "UPDATE recurring SET account_id = ?1, category_id = ?2, amount = ?3, type = ?4, 
         frequency = ?5, next_date = ?6, end_date = ?7, note = ?8, is_active = ?9 WHERE id = ?10",
        rusqlite::params![account_id, category_id, amount, payment_type, frequency, next_date, end_date, note, is_active as i64, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn delete_recurring(conn: &Connection, id: i64) -> Result<(), String> {
    conn.execute("DELETE FROM recurring WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Calculate the next date based on frequency
fn calculate_next_date(current_date: &str, frequency: &str) -> Result<String, String> {
    use chrono::{NaiveDate, Duration, Months};
    
    let date = NaiveDate::parse_from_str(current_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid date: {}", e))?;
    
    let next = match frequency {
        "daily" => date + Duration::days(1),
        "weekly" => date + Duration::weeks(1),
        "monthly" => date.checked_add_months(Months::new(1))
            .ok_or_else(|| "Failed to calculate next month".to_string())?,
        "yearly" => date.checked_add_months(Months::new(12))
            .ok_or_else(|| "Failed to calculate next year".to_string())?,
        _ => return Err(format!("Unknown frequency: {}", frequency)),
    };
    
    Ok(next.format("%Y-%m-%d").to_string())
}

/// Process all due recurring payments and create transactions
pub fn process_due_recurring(conn: &Connection) -> Result<Vec<i64>, String> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let mut created_ids = Vec::new();

    // Get all active recurring payments that are due
    let mut stmt = conn
        .prepare(
            "SELECT id, account_id, category_id, amount, type, frequency, next_date, end_date, note
             FROM recurring
             WHERE is_active = 1 AND next_date <= ?1
             AND (end_date IS NULL OR end_date >= ?1)",
        )
        .map_err(|e| e.to_string())?;

    struct DuePayment {
        id: i64,
        account_id: i64,
        category_id: Option<i64>,
        amount: f64,
        payment_type: String,
        frequency: String,
        next_date: String,
        note: Option<String>,
    }

    let due_payments: Vec<DuePayment> = stmt
        .query_map([&today], |row| {
            Ok(DuePayment {
                id: row.get(0)?,
                account_id: row.get(1)?,
                category_id: row.get(2)?,
                amount: row.get(3)?,
                payment_type: row.get(4)?,
                frequency: row.get(5)?,
                next_date: row.get(6)?,
                note: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    for payment in due_payments {
        // Create the transaction
        let note = payment.note.as_deref().unwrap_or("Автоплатёж");
        let tx_id = create_transaction(
            conn,
            payment.account_id,
            payment.category_id,
            payment.amount,
            &payment.payment_type,
            Some(note),
            &payment.next_date,
        )?;
        created_ids.push(tx_id);

        // Calculate and update the next date
        let new_next_date = calculate_next_date(&payment.next_date, &payment.frequency)?;
        conn.execute(
            "UPDATE recurring SET next_date = ?1, last_processed = ?2 WHERE id = ?3",
            rusqlite::params![new_next_date, today, payment.id],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(created_ids)
}

// ============================================================================
// Budgets
// ============================================================================

#[derive(Debug, Serialize)]
pub struct Budget {
    pub id: i64,
    pub category_id: i64,
    pub category_name: String,
    pub category_color: Option<String>,
    pub amount: f64,
    pub spent: f64,
    pub remaining: f64,
    pub percent_used: f64,
    pub period: String,
}

#[derive(Debug, Serialize)]
pub struct BudgetAlert {
    pub category_name: String,
    pub percent_used: f64,
    pub severity: String, // "warning" | "exceeded"
}

pub fn get_budgets_with_spending(conn: &Connection) -> Result<Vec<Budget>, String> {
    let now = chrono::Local::now();
    let month_start = format!("{}-{:02}-01", now.year(), now.month());
    let month_end = if now.month() == 12 {
        format!("{}-01-01", now.year() + 1)
    } else {
        format!("{}-{:02}-01", now.year(), now.month() + 1)
    };

    let mut stmt = conn
        .prepare(
            "SELECT b.id, b.category_id, c.name, c.color, b.amount, b.period,
                    COALESCE(ABS(SUM(t.amount)), 0) as spent
             FROM budgets b
             JOIN categories c ON b.category_id = c.id
             LEFT JOIN transactions t ON t.category_id = b.category_id 
                 AND t.type = 'expense' 
                 AND t.date >= ?1 AND t.date < ?2
             GROUP BY b.id
             ORDER BY c.name",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(rusqlite::params![&month_start, &month_end], |row| {
            let amount: f64 = row.get(4)?;
            let spent: f64 = row.get(6)?;
            let remaining = (amount - spent).max(0.0);
            let percent_used = if amount > 0.0 { (spent / amount) * 100.0 } else { 0.0 };

            Ok(Budget {
                id: row.get(0)?,
                category_id: row.get(1)?,
                category_name: row.get(2)?,
                category_color: row.get(3)?,
                amount,
                spent,
                remaining,
                percent_used,
                period: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

pub fn create_budget(
    conn: &Connection,
    category_id: i64,
    amount: f64,
    period: &str,
) -> Result<i64, String> {
    conn.execute(
        "INSERT OR REPLACE INTO budgets (category_id, amount, period) VALUES (?1, ?2, ?3)",
        rusqlite::params![category_id, amount, period],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

pub fn update_budget(conn: &Connection, id: i64, amount: f64) -> Result<(), String> {
    conn.execute(
        "UPDATE budgets SET amount = ?1 WHERE id = ?2",
        rusqlite::params![amount, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn delete_budget(conn: &Connection, id: i64) -> Result<(), String> {
    conn.execute("DELETE FROM budgets WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn check_budget_alerts(conn: &Connection) -> Result<Vec<BudgetAlert>, String> {
    let budgets = get_budgets_with_spending(conn)?;
    let mut alerts = Vec::new();

    for budget in budgets {
        if budget.percent_used >= 100.0 {
            alerts.push(BudgetAlert {
                category_name: budget.category_name,
                percent_used: budget.percent_used,
                severity: "exceeded".to_string(),
            });
        } else if budget.percent_used >= 80.0 {
            alerts.push(BudgetAlert {
                category_name: budget.category_name,
                percent_used: budget.percent_used,
                severity: "warning".to_string(),
            });
        }
    }

    Ok(alerts)
}

// ============================================================================
// Category Management
// ============================================================================

pub fn create_category(
    conn: &Connection,
    name: &str,
    category_type: &str,
    icon: Option<&str>,
    color: Option<&str>,
    parent_id: Option<i64>,
) -> Result<i64, String> {
    conn.execute(
        "INSERT INTO categories (name, type, icon, color, parent_id) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![name, category_type, icon, color, parent_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

pub fn update_category(
    conn: &Connection,
    id: i64,
    name: &str,
    icon: Option<&str>,
    color: Option<&str>,
) -> Result<(), String> {
    conn.execute(
        "UPDATE categories SET name = ?1, icon = ?2, color = ?3 WHERE id = ?4",
        rusqlite::params![name, icon, color, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn delete_category(conn: &Connection, id: i64) -> Result<(), String> {
    // Check if category is used in transactions
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM transactions WHERE category_id = ?1",
            [id],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;

    if count > 0 {
        return Err("Нельзя удалить категорию с транзакциями. Сначала измените категорию у транзакций.".to_string());
    }

    // Check if category is used in budgets
    let budget_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM budgets WHERE category_id = ?1",
            [id],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;

    if budget_count > 0 {
        return Err("Нельзя удалить категорию с бюджетом. Сначала удалите бюджет.".to_string());
    }

    conn.execute("DELETE FROM categories WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::schema::create_tables(&conn).unwrap();
        crate::db::schema::seed_categories(&conn).unwrap();
        conn
    }

    // =====================================================================
    // Account Tests
    // =====================================================================

    #[test]
    fn test_create_account() {
        let conn = setup_test_db();
        let id = create_account(&conn, "Тест", "card", "KZT").unwrap();
        assert!(id > 0);
    }

    #[test]
    fn test_get_accounts_empty() {
        let conn = setup_test_db();
        let accounts = get_accounts(&conn).unwrap();
        assert!(accounts.is_empty());
    }

    #[test]
    fn test_get_accounts_returns_created() {
        let conn = setup_test_db();
        create_account(&conn, "Карта", "card", "KZT").unwrap();
        create_account(&conn, "Наличные", "cash", "USD").unwrap();

        let accounts = get_accounts(&conn).unwrap();
        assert_eq!(accounts.len(), 2);
        assert!(accounts.iter().any(|a| a.name == "Карта"));
        assert!(accounts.iter().any(|a| a.name == "Наличные"));
    }

    #[test]
    fn test_create_account_initial_balance_zero() {
        let conn = setup_test_db();
        create_account(&conn, "Тест", "card", "KZT").unwrap();

        let accounts = get_accounts(&conn).unwrap();
        assert_eq!(accounts[0].balance, 0.0);
    }

    #[test]
    fn test_update_account() {
        let conn = setup_test_db();
        let id = create_account(&conn, "Старое", "card", "KZT").unwrap();

        update_account(&conn, id, "Новое", "cash", "USD").unwrap();

        let accounts = get_accounts(&conn).unwrap();
        assert_eq!(accounts[0].name, "Новое");
        assert_eq!(accounts[0].account_type, "cash");
        assert_eq!(accounts[0].currency, "USD");
    }

    #[test]
    fn test_delete_account_empty() {
        let conn = setup_test_db();
        let id = create_account(&conn, "Удалить", "card", "KZT").unwrap();

        let result = delete_account(&conn, id);
        assert!(result.is_ok());

        let accounts = get_accounts(&conn).unwrap();
        assert!(accounts.is_empty());
    }

    #[test]
    fn test_delete_account_with_transactions_fails() {
        let conn = setup_test_db();
        let account_id = create_account(&conn, "Тест", "card", "KZT").unwrap();
        create_transaction(&conn, account_id, None, 100.0, "income", None, "2024-01-01").unwrap();

        let result = delete_account(&conn, account_id);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("транзакциями"));
    }

    #[test]
    fn test_account_exists_true() {
        let conn = setup_test_db();
        let id = create_account(&conn, "Тест", "card", "KZT").unwrap();

        assert!(account_exists(&conn, id).unwrap());
    }

    #[test]
    fn test_account_exists_false() {
        let conn = setup_test_db();
        assert!(!account_exists(&conn, 9999).unwrap());
    }

    // =====================================================================
    // Transaction Tests
    // =====================================================================

    #[test]
    fn test_create_transaction_income() {
        let conn = setup_test_db();
        let account_id = create_account(&conn, "Тест", "card", "KZT").unwrap();

        let tx_id = create_transaction(&conn, account_id, None, 1000.0, "income", Some("Зарплата"), "2024-01-15").unwrap();
        assert!(tx_id > 0);
    }

    #[test]
    fn test_create_transaction_expense() {
        let conn = setup_test_db();
        let account_id = create_account(&conn, "Тест", "card", "KZT").unwrap();

        let tx_id = create_transaction(&conn, account_id, None, 500.0, "expense", Some("Еда"), "2024-01-15").unwrap();
        assert!(tx_id > 0);
    }

    #[test]
    fn test_create_transaction_updates_balance_income() {
        let conn = setup_test_db();
        let account_id = create_account(&conn, "Тест", "card", "KZT").unwrap();

        create_transaction(&conn, account_id, None, 1000.0, "income", None, "2024-01-15").unwrap();

        let accounts = get_accounts(&conn).unwrap();
        assert_eq!(accounts[0].balance, 1000.0);
    }

    #[test]
    fn test_create_transaction_updates_balance_expense() {
        let conn = setup_test_db();
        let account_id = create_account(&conn, "Тест", "card", "KZT").unwrap();
        create_transaction(&conn, account_id, None, 1000.0, "income", None, "2024-01-15").unwrap();

        create_transaction(&conn, account_id, None, 300.0, "expense", None, "2024-01-16").unwrap();

        let accounts = get_accounts(&conn).unwrap();
        assert_eq!(accounts[0].balance, 700.0);
    }

    #[test]
    fn test_delete_transaction_reverts_balance() {
        let conn = setup_test_db();
        let account_id = create_account(&conn, "Тест", "card", "KZT").unwrap();
        create_transaction(&conn, account_id, None, 1000.0, "income", None, "2024-01-15").unwrap();
        let tx_id = create_transaction(&conn, account_id, None, 300.0, "expense", None, "2024-01-16").unwrap();

        delete_transaction(&conn, tx_id).unwrap();

        let accounts = get_accounts(&conn).unwrap();
        assert_eq!(accounts[0].balance, 1000.0);
    }

    #[test]
    fn test_update_transaction_adjusts_balance() {
        let conn = setup_test_db();
        let account_id = create_account(&conn, "Тест", "card", "KZT").unwrap();
        let tx_id = create_transaction(&conn, account_id, None, 1000.0, "income", None, "2024-01-15").unwrap();

        update_transaction(&conn, tx_id, account_id, None, 500.0, "income", None, "2024-01-15").unwrap();

        let accounts = get_accounts(&conn).unwrap();
        assert_eq!(accounts[0].balance, 500.0);
    }

    #[test]
    fn test_get_transactions_filtered_by_date() {
        let conn = setup_test_db();
        let account_id = create_account(&conn, "Тест", "card", "KZT").unwrap();
        create_transaction(&conn, account_id, None, 100.0, "income", None, "2024-01-01").unwrap();
        create_transaction(&conn, account_id, None, 200.0, "income", None, "2024-01-15").unwrap();
        create_transaction(&conn, account_id, None, 300.0, "income", None, "2024-02-01").unwrap();

        let transactions = get_transactions_filtered(&conn, TransactionFilters {
            limit: 100,
            date_from: Some("2024-01-10".to_string()),
            date_to: Some("2024-01-31".to_string()),
            account_id: None,
            category_id: None,
            transaction_type: None,
            search_note: None,
        }).unwrap();

        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].amount, 200.0);
    }

    #[test]
    fn test_get_transactions_filtered_by_account() {
        let conn = setup_test_db();
        let account1 = create_account(&conn, "Счёт 1", "card", "KZT").unwrap();
        let account2 = create_account(&conn, "Счёт 2", "cash", "KZT").unwrap();
        create_transaction(&conn, account1, None, 100.0, "income", None, "2024-01-01").unwrap();
        create_transaction(&conn, account2, None, 200.0, "income", None, "2024-01-01").unwrap();

        let transactions = get_transactions_filtered(&conn, TransactionFilters {
            limit: 100,
            date_from: None,
            date_to: None,
            account_id: Some(account1),
            category_id: None,
            transaction_type: None,
            search_note: None,
        }).unwrap();

        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].account_id, account1);
    }

    #[test]
    fn test_get_transactions_filtered_by_type() {
        let conn = setup_test_db();
        let account_id = create_account(&conn, "Тест", "card", "KZT").unwrap();
        create_transaction(&conn, account_id, None, 1000.0, "income", None, "2024-01-01").unwrap();
        create_transaction(&conn, account_id, None, 100.0, "expense", None, "2024-01-02").unwrap();
        create_transaction(&conn, account_id, None, 200.0, "expense", None, "2024-01-03").unwrap();

        let transactions = get_transactions_filtered(&conn, TransactionFilters {
            limit: 100,
            date_from: None,
            date_to: None,
            account_id: None,
            category_id: None,
            transaction_type: Some("expense".to_string()),
            search_note: None,
        }).unwrap();

        assert_eq!(transactions.len(), 2);
    }

    #[test]
    fn test_get_transactions_filtered_by_note() {
        let conn = setup_test_db();
        let account_id = create_account(&conn, "Тест", "card", "KZT").unwrap();
        create_transaction(&conn, account_id, None, 100.0, "expense", Some("Кофе в Starbucks"), "2024-01-01").unwrap();
        create_transaction(&conn, account_id, None, 200.0, "expense", Some("Обед"), "2024-01-02").unwrap();

        // Note: SQLite LIKE is case-sensitive for non-ASCII characters, so use same case
        let transactions = get_transactions_filtered(&conn, TransactionFilters {
            limit: 100,
            date_from: None,
            date_to: None,
            account_id: None,
            category_id: None,
            transaction_type: None,
            search_note: Some("Кофе".to_string()),
        }).unwrap();

        assert_eq!(transactions.len(), 1);
    }

    // =====================================================================
    // Transfer Tests
    // =====================================================================

    #[test]
    fn test_create_transfer_success() {
        let conn = setup_test_db();
        let from_account = create_account(&conn, "Карта", "card", "KZT").unwrap();
        let to_account = create_account(&conn, "Наличные", "cash", "KZT").unwrap();

        // Add initial balance
        create_transaction(&conn, from_account, None, 1000.0, "income", None, "2024-01-01").unwrap();

        let result = create_transfer(&conn, from_account, to_account, 500.0, "2024-01-15", Some("Снятие"));
        assert!(result.is_ok());

        let accounts = get_accounts(&conn).unwrap();
        let from = accounts.iter().find(|a| a.id == from_account).unwrap();
        let to = accounts.iter().find(|a| a.id == to_account).unwrap();

        assert_eq!(from.balance, 500.0);
        assert_eq!(to.balance, 500.0);
    }

    #[test]
    fn test_create_transfer_same_account_fails() {
        let conn = setup_test_db();
        let account_id = create_account(&conn, "Тест", "card", "KZT").unwrap();

        let result = create_transfer(&conn, account_id, account_id, 100.0, "2024-01-15", None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("совпадать"));
    }

    #[test]
    fn test_create_transfer_zero_amount_fails() {
        let conn = setup_test_db();
        let from_account = create_account(&conn, "Карта", "card", "KZT").unwrap();
        let to_account = create_account(&conn, "Наличные", "cash", "KZT").unwrap();

        let result = create_transfer(&conn, from_account, to_account, 0.0, "2024-01-15", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_transfer_negative_amount_fails() {
        let conn = setup_test_db();
        let from_account = create_account(&conn, "Карта", "card", "KZT").unwrap();
        let to_account = create_account(&conn, "Наличные", "cash", "KZT").unwrap();

        let result = create_transfer(&conn, from_account, to_account, -100.0, "2024-01-15", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_transfer_nonexistent_account_fails() {
        let conn = setup_test_db();
        let account_id = create_account(&conn, "Тест", "card", "KZT").unwrap();

        let result = create_transfer(&conn, account_id, 9999, 100.0, "2024-01-15", None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("не найден"));
    }

    // =====================================================================
    // Summary Tests
    // =====================================================================

    #[test]
    fn test_get_summary_empty_db() {
        let conn = setup_test_db();
        let summary = get_summary(&conn).unwrap();

        assert_eq!(summary.total_balance, 0.0);
        assert_eq!(summary.income_month, 0.0);
        assert_eq!(summary.expense_month, 0.0);
    }

    #[test]
    fn test_get_summary_with_transactions() {
        let conn = setup_test_db();
        let account_id = create_account(&conn, "Тест", "card", "KZT").unwrap();

        // Use current month for the test
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();

        create_transaction(&conn, account_id, None, 5000.0, "income", None, &today).unwrap();
        create_transaction(&conn, account_id, None, 1000.0, "expense", None, &today).unwrap();

        let summary = get_summary(&conn).unwrap();

        assert_eq!(summary.total_balance, 4000.0);
        assert_eq!(summary.income_month, 5000.0);
        assert_eq!(summary.expense_month, 1000.0);
    }

    // =====================================================================
    // Category Tests
    // =====================================================================

    #[test]
    fn test_get_categories_seeded() {
        let conn = setup_test_db();
        let categories = get_categories(&conn).unwrap();

        assert!(categories.len() >= 9); // Default seeded categories
    }

    #[test]
    fn test_category_exists_and_type_true() {
        let conn = setup_test_db();
        let categories = get_categories(&conn).unwrap();
        let expense_cat = categories.iter().find(|c| c.category_type == "expense").unwrap();

        assert!(category_exists_and_type(&conn, expense_cat.id, "expense").unwrap());
    }

    #[test]
    fn test_category_exists_and_type_wrong_type() {
        let conn = setup_test_db();
        let categories = get_categories(&conn).unwrap();
        let expense_cat = categories.iter().find(|c| c.category_type == "expense").unwrap();

        assert!(!category_exists_and_type(&conn, expense_cat.id, "income").unwrap());
    }

    #[test]
    fn test_create_category() {
        let conn = setup_test_db();
        let id = create_category(&conn, "Тест", "expense", Some("icon"), Some("#ff0000"), None).unwrap();
        assert!(id > 0);

        let categories = get_categories(&conn).unwrap();
        let created = categories.iter().find(|c| c.id == id).unwrap();
        assert_eq!(created.name, "Тест");
    }

    #[test]
    fn test_update_category() {
        let conn = setup_test_db();
        let id = create_category(&conn, "Старое", "expense", None, None, None).unwrap();

        update_category(&conn, id, "Новое", Some("new-icon"), Some("#00ff00")).unwrap();

        let categories = get_categories(&conn).unwrap();
        let updated = categories.iter().find(|c| c.id == id).unwrap();
        assert_eq!(updated.name, "Новое");
        assert_eq!(updated.icon, Some("new-icon".to_string()));
    }

    #[test]
    fn test_delete_category_empty() {
        let conn = setup_test_db();
        let id = create_category(&conn, "Удалить", "expense", None, None, None).unwrap();

        let result = delete_category(&conn, id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_delete_category_with_transactions_fails() {
        let conn = setup_test_db();
        let cat_id = create_category(&conn, "Тест", "expense", None, None, None).unwrap();
        let account_id = create_account(&conn, "Счёт", "card", "KZT").unwrap();
        create_transaction(&conn, account_id, Some(cat_id), 100.0, "expense", None, "2024-01-01").unwrap();

        let result = delete_category(&conn, cat_id);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("транзакциями"));
    }

    #[test]
    fn test_delete_category_with_budget_fails() {
        let conn = setup_test_db();
        let cat_id = create_category(&conn, "Тест", "expense", None, None, None).unwrap();
        create_budget(&conn, cat_id, 1000.0, "monthly").unwrap();

        let result = delete_category(&conn, cat_id);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("бюджетом"));
    }

    // =====================================================================
    // Budget Tests
    // =====================================================================

    #[test]
    fn test_create_budget() {
        let conn = setup_test_db();
        let categories = get_categories(&conn).unwrap();
        let expense_cat = categories.iter().find(|c| c.category_type == "expense").unwrap();

        let id = create_budget(&conn, expense_cat.id, 5000.0, "monthly").unwrap();
        assert!(id > 0);
    }

    #[test]
    fn test_get_budgets_with_spending() {
        let conn = setup_test_db();
        let categories = get_categories(&conn).unwrap();
        let expense_cat = categories.iter().find(|c| c.category_type == "expense").unwrap();
        let account_id = create_account(&conn, "Тест", "card", "KZT").unwrap();

        create_budget(&conn, expense_cat.id, 5000.0, "monthly").unwrap();

        // Add expense for current month
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        create_transaction(&conn, account_id, Some(expense_cat.id), 2000.0, "expense", None, &today).unwrap();

        let budgets = get_budgets_with_spending(&conn).unwrap();
        let budget = budgets.iter().find(|b| b.category_id == expense_cat.id).unwrap();

        assert_eq!(budget.amount, 5000.0);
        assert_eq!(budget.spent, 2000.0);
        assert_eq!(budget.remaining, 3000.0);
        assert!((budget.percent_used - 40.0).abs() < 0.01);
    }

    #[test]
    fn test_update_budget() {
        let conn = setup_test_db();
        let categories = get_categories(&conn).unwrap();
        let expense_cat = categories.iter().find(|c| c.category_type == "expense").unwrap();

        let id = create_budget(&conn, expense_cat.id, 5000.0, "monthly").unwrap();
        update_budget(&conn, id, 10000.0).unwrap();

        let budgets = get_budgets_with_spending(&conn).unwrap();
        let budget = budgets.iter().find(|b| b.id == id).unwrap();

        assert_eq!(budget.amount, 10000.0);
    }

    #[test]
    fn test_delete_budget() {
        let conn = setup_test_db();
        let categories = get_categories(&conn).unwrap();
        let expense_cat = categories.iter().find(|c| c.category_type == "expense").unwrap();

        let id = create_budget(&conn, expense_cat.id, 5000.0, "monthly").unwrap();
        delete_budget(&conn, id).unwrap();

        let budgets = get_budgets_with_spending(&conn).unwrap();
        assert!(!budgets.iter().any(|b| b.id == id));
    }

    #[test]
    fn test_check_budget_alerts_warning() {
        let conn = setup_test_db();
        let categories = get_categories(&conn).unwrap();
        let expense_cat = categories.iter().find(|c| c.category_type == "expense").unwrap();
        let account_id = create_account(&conn, "Тест", "card", "KZT").unwrap();

        create_budget(&conn, expense_cat.id, 1000.0, "monthly").unwrap();

        // Spend 85% of budget
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        create_transaction(&conn, account_id, Some(expense_cat.id), 850.0, "expense", None, &today).unwrap();

        let alerts = check_budget_alerts(&conn).unwrap();
        assert!(!alerts.is_empty());
        assert!(alerts.iter().any(|a| a.severity == "warning"));
    }

    #[test]
    fn test_check_budget_alerts_exceeded() {
        let conn = setup_test_db();
        let categories = get_categories(&conn).unwrap();
        let expense_cat = categories.iter().find(|c| c.category_type == "expense").unwrap();
        let account_id = create_account(&conn, "Тест", "card", "KZT").unwrap();

        create_budget(&conn, expense_cat.id, 1000.0, "monthly").unwrap();

        // Spend 120% of budget
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        create_transaction(&conn, account_id, Some(expense_cat.id), 1200.0, "expense", None, &today).unwrap();

        let alerts = check_budget_alerts(&conn).unwrap();
        assert!(!alerts.is_empty());
        assert!(alerts.iter().any(|a| a.severity == "exceeded"));
    }

    // =====================================================================
    // Recurring Payment Tests
    // =====================================================================

    #[test]
    fn test_create_recurring() {
        let conn = setup_test_db();
        let account_id = create_account(&conn, "Тест", "card", "KZT").unwrap();

        let id = create_recurring(
            &conn,
            account_id,
            None,
            5000.0,
            "expense",
            "monthly",
            "2024-02-01",
            None,
            Some("Аренда"),
        ).unwrap();

        assert!(id > 0);
    }

    #[test]
    fn test_get_recurring_payments() {
        let conn = setup_test_db();
        let account_id = create_account(&conn, "Тест", "card", "KZT").unwrap();

        create_recurring(&conn, account_id, None, 5000.0, "expense", "monthly", "2024-02-01", None, Some("Аренда")).unwrap();
        create_recurring(&conn, account_id, None, 1000.0, "expense", "weekly", "2024-02-01", None, Some("Интернет")).unwrap();

        let recurring = get_recurring_payments(&conn).unwrap();
        assert_eq!(recurring.len(), 2);
    }

    #[test]
    fn test_update_recurring() {
        let conn = setup_test_db();
        let account_id = create_account(&conn, "Тест", "card", "KZT").unwrap();

        let id = create_recurring(&conn, account_id, None, 5000.0, "expense", "monthly", "2024-02-01", None, Some("Аренда")).unwrap();

        update_recurring(&conn, id, account_id, None, 6000.0, "expense", "monthly", "2024-03-01", None, Some("Аренда обновлённая"), true).unwrap();

        let recurring = get_recurring_payments(&conn).unwrap();
        let updated = recurring.iter().find(|r| r.id == id).unwrap();

        assert_eq!(updated.amount, 6000.0);
        assert_eq!(updated.next_date, "2024-03-01");
    }

    #[test]
    fn test_delete_recurring() {
        let conn = setup_test_db();
        let account_id = create_account(&conn, "Тест", "card", "KZT").unwrap();

        let id = create_recurring(&conn, account_id, None, 5000.0, "expense", "monthly", "2024-02-01", None, None).unwrap();
        delete_recurring(&conn, id).unwrap();

        let recurring = get_recurring_payments(&conn).unwrap();
        assert!(!recurring.iter().any(|r| r.id == id));
    }

    // =====================================================================
    // Expense by Category Tests
    // =====================================================================

    #[test]
    fn test_get_expense_by_category_empty() {
        let conn = setup_test_db();
        let expenses = get_expense_by_category(&conn, 2024, 1).unwrap();
        assert!(expenses.is_empty());
    }

    #[test]
    fn test_get_expense_by_category_with_transactions() {
        let conn = setup_test_db();
        let account_id = create_account(&conn, "Тест", "card", "KZT").unwrap();
        let categories = get_categories(&conn).unwrap();
        let food_cat = categories.iter().find(|c| c.name == "Еда").unwrap();
        let transport_cat = categories.iter().find(|c| c.name == "Транспорт").unwrap();

        create_transaction(&conn, account_id, Some(food_cat.id), 1000.0, "expense", None, "2024-01-15").unwrap();
        create_transaction(&conn, account_id, Some(food_cat.id), 500.0, "expense", None, "2024-01-20").unwrap();
        create_transaction(&conn, account_id, Some(transport_cat.id), 300.0, "expense", None, "2024-01-10").unwrap();

        let expenses = get_expense_by_category(&conn, 2024, 1).unwrap();

        assert_eq!(expenses.len(), 2);
        let food_expense = expenses.iter().find(|e| e.category_name == "Еда").unwrap();
        assert_eq!(food_expense.total, 1500.0);
    }

    // =====================================================================
    // Monthly Totals Tests
    // =====================================================================

    #[test]
    fn test_get_monthly_totals_empty() {
        let conn = setup_test_db();
        let totals = get_monthly_totals(&conn, 6).unwrap();

        assert_eq!(totals.len(), 6);
        for total in totals {
            assert_eq!(total.income, 0.0);
            assert_eq!(total.expense, 0.0);
        }
    }

    #[test]
    fn test_get_monthly_totals_with_data() {
        let conn = setup_test_db();
        let account_id = create_account(&conn, "Тест", "card", "KZT").unwrap();

        // Use current month for the test
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();

        create_transaction(&conn, account_id, None, 5000.0, "income", None, &today).unwrap();
        create_transaction(&conn, account_id, None, 2000.0, "expense", None, &today).unwrap();

        let totals = get_monthly_totals(&conn, 3).unwrap();

        // Last month should have our data
        let current_month = totals.last().unwrap();
        assert_eq!(current_month.income, 5000.0);
        assert_eq!(current_month.expense, 2000.0);
    }
}
