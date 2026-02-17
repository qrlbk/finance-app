use rusqlite::Connection;
use std::sync::atomic::{AtomicU64, Ordering};

pub fn create_tables(conn: &Connection) -> Result<(), String> {
    // Create basic tables
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS accounts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            type TEXT NOT NULL,
            balance REAL DEFAULT 0,
            currency TEXT DEFAULT 'KZT',
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS categories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            type TEXT NOT NULL,
            icon TEXT,
            color TEXT,
            parent_id INTEGER REFERENCES categories(id)
        );

        CREATE TABLE IF NOT EXISTS transactions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            account_id INTEGER NOT NULL REFERENCES accounts(id),
            category_id INTEGER REFERENCES categories(id),
            amount REAL NOT NULL,
            type TEXT NOT NULL,
            note TEXT,
            date TEXT NOT NULL,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS recurring (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            account_id INTEGER NOT NULL REFERENCES accounts(id),
            category_id INTEGER REFERENCES categories(id),
            amount REAL NOT NULL,
            type TEXT NOT NULL,
            frequency TEXT NOT NULL,
            next_date TEXT NOT NULL,
            end_date TEXT,
            note TEXT,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS budgets (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            category_id INTEGER NOT NULL REFERENCES categories(id),
            amount REAL NOT NULL,
            period TEXT NOT NULL,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(category_id, period)
        );
        ",
    )
    .map_err(|e| e.to_string())?;

    // Run migrations for existing databases
    run_migrations(conn)?;

    // Create indexes (safe to run always)
    conn.execute_batch(
        "
        -- Performance indexes for transactions
        CREATE INDEX IF NOT EXISTS idx_transactions_date ON transactions(date);
        CREATE INDEX IF NOT EXISTS idx_transactions_account ON transactions(account_id);
        CREATE INDEX IF NOT EXISTS idx_transactions_category ON transactions(category_id);
        CREATE INDEX IF NOT EXISTS idx_transactions_type ON transactions(type);
        CREATE INDEX IF NOT EXISTS idx_transactions_date_type ON transactions(date, type);
        CREATE INDEX IF NOT EXISTS idx_transactions_account_date ON transactions(account_id, date);
        CREATE INDEX IF NOT EXISTS idx_transactions_category_date ON transactions(category_id, date);
        
        -- Indexes for recurring payments
        CREATE INDEX IF NOT EXISTS idx_recurring_next_date ON recurring(next_date);
        CREATE INDEX IF NOT EXISTS idx_recurring_active ON recurring(is_active);
        
        -- Indexes for budgets
        CREATE INDEX IF NOT EXISTS idx_budgets_category ON budgets(category_id);
        CREATE INDEX IF NOT EXISTS idx_budgets_period ON budgets(period);
        
        -- Index for categories lookup
        CREATE INDEX IF NOT EXISTS idx_categories_type ON categories(type);
        ",
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// Run database migrations for schema updates
fn run_migrations(conn: &Connection) -> Result<(), String> {
    // Helper function to check if column exists
    let has_column = |table: &str, column: &str| -> bool {
        conn.query_row(
            &format!("SELECT COUNT(*) FROM pragma_table_info('{}') WHERE name = '{}'", table, column),
            [],
            |r| r.get::<_, i32>(0),
        )
        .unwrap_or(0) > 0
    };

    // Migration: Add note column to recurring table
    if !has_column("recurring", "note") {
        conn.execute(
            "ALTER TABLE recurring ADD COLUMN note TEXT",
            [],
        )
        .map_err(|e| format!("Failed to add note column: {}", e))?;
    }

    // Migration: Add end_date column to recurring table
    if !has_column("recurring", "end_date") {
        conn.execute(
            "ALTER TABLE recurring ADD COLUMN end_date TEXT",
            [],
        )
        .map_err(|e| format!("Failed to add end_date column: {}", e))?;
    }

    // Migration: Add is_active column to recurring table
    if !has_column("recurring", "is_active") {
        conn.execute(
            "ALTER TABLE recurring ADD COLUMN is_active INTEGER DEFAULT 1",
            [],
        )
        .map_err(|e| format!("Failed to add is_active column: {}", e))?;
    }

    // Migration: Add last_processed column to recurring table
    if !has_column("recurring", "last_processed") {
        conn.execute(
            "ALTER TABLE recurring ADD COLUMN last_processed TEXT",
            [],
        )
        .map_err(|e| format!("Failed to add last_processed column: {}", e))?;
    }

    // Migration: Create transfers table for transfer history
    let has_table = |name: &str| -> bool {
        conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?",
            [name],
            |r| r.get::<_, i32>(0),
        )
        .unwrap_or(0) > 0
    };
    if !has_table("transfers") {
        conn.execute_batch(
            "CREATE TABLE transfers (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                from_account_id INTEGER NOT NULL REFERENCES accounts(id),
                to_account_id INTEGER NOT NULL REFERENCES accounts(id),
                amount REAL NOT NULL,
                date TEXT NOT NULL,
                note TEXT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            );
            CREATE INDEX IF NOT EXISTS idx_transfers_date ON transfers(date);
            ",
        )
        .map_err(|e| format!("Failed to create transfers table: {}", e))?;
    }

    // Migration: category_rules for ML hybrid (note_normalized -> category_id)
    if !has_table("category_rules") {
        conn.execute_batch(
            "CREATE TABLE category_rules (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                note_normalized TEXT NOT NULL UNIQUE,
                category_id INTEGER NOT NULL REFERENCES categories(id),
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            );
            CREATE INDEX IF NOT EXISTS idx_category_rules_note ON category_rules(note_normalized);
            ",
        )
        .map_err(|e| format!("Failed to create category_rules table: {}", e))?;
    }

    // Migration: users table for multi-user support
    if !has_table("users") {
        conn.execute_batch(
            "CREATE TABLE users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                display_name TEXT NOT NULL,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            );
            CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
            ",
        )
        .map_err(|e| format!("Failed to create users table: {}", e))?;
    }

    // Migration: add user_id to data tables
    for (table, _) in [
        ("accounts", ()),
        ("categories", ()),
        ("transactions", ()),
        ("recurring", ()),
        ("budgets", ()),
        ("transfers", ()),
    ] {
        if !has_column(table, "user_id") {
            conn.execute(
                &format!("ALTER TABLE {} ADD COLUMN user_id INTEGER REFERENCES users(id)", table),
                [],
            )
            .map_err(|e| format!("Failed to add user_id to {}: {}", table, e))?;
        }
    }
    if !has_column("category_rules", "user_id") {
        conn.execute(
            "ALTER TABLE category_rules ADD COLUMN user_id INTEGER REFERENCES users(id)",
            [],
        )
        .map_err(|e| format!("Failed to add user_id to category_rules: {}", e))?;
    }

    // Backfill: создаём пользователя "default" только при миграции существующих данных (есть счета/транзакции/категории).
    // В новых установках (пустая БД) пользователя по умолчанию не создаём — вход только через регистрацию.
    let user_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM users", [], |r| r.get(0))
        .unwrap_or(0);
    let has_data: bool = user_count == 0
        && conn
            .query_row(
                "SELECT (SELECT COUNT(*) FROM accounts) + (SELECT COUNT(*) FROM transactions) + (SELECT COUNT(*) FROM categories)",
                [],
                |r| r.get::<_, i64>(0),
            )
            .map(|n| n > 0)
            .unwrap_or(false);
    if user_count == 0 && has_data {
        let impossible_password = format!("no_login_{}", AtomicU64::new(0).fetch_add(1, Ordering::Relaxed));
        let hash = default_user_password_hash(&impossible_password);
        conn.execute(
            "INSERT INTO users (id, username, password_hash, display_name) VALUES (1, 'default', ?1, 'Пользователь')",
            rusqlite::params![hash],
        )
        .map_err(|e| format!("Failed to insert default user: {}", e))?;
        for table in ["accounts", "categories", "transactions", "recurring", "budgets", "transfers", "category_rules"] {
            if has_column(table, "user_id") {
                conn.execute(
                    &format!("UPDATE {} SET user_id = 1 WHERE user_id IS NULL", table),
                    [],
                )
                .map_err(|e| format!("Failed to backfill user_id in {}: {}", table, e))?;
            }
        }
    }

    // Indexes for user_id
    for (table, idx_name) in [
        ("accounts", "idx_accounts_user_id"),
        ("categories", "idx_categories_user_id"),
        ("transactions", "idx_transactions_user_id"),
        ("recurring", "idx_recurring_user_id"),
        ("budgets", "idx_budgets_user_id"),
        ("transfers", "idx_transfers_user_id"),
        ("category_rules", "idx_category_rules_user_id"),
    ] {
        let exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name=?",
                [idx_name],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if exists == 0 {
            conn.execute(
                &format!("CREATE INDEX IF NOT EXISTS {} ON {}(user_id)", idx_name, table),
                [],
            )
            .map_err(|e| format!("Failed to create index {}: {}", idx_name, e))?;
        }
    }

    Ok(())
}

/// Generates a PHC hash for the default user (so no-one can login without setting password).
fn default_user_password_hash(password: &str) -> String {
    use argon2::password_hash::{rand_core::OsRng, PasswordHasher, SaltString};
    use argon2::Argon2;
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .unwrap_or_else(|_| "$argon2id$v=19$m=19456,t=2,p=1$dummy$dummy".to_string())
}

/// Очищает все данные пользователя (в порядке, учитывающем внешние ключи).
/// После вызова можно вызвать `seed_categories(conn, user_id)` для восстановления категорий по умолчанию.
pub fn clear_all_data(conn: &Connection, user_id: i64) -> Result<(), String> {
    conn.execute("DELETE FROM category_rules WHERE user_id = ?1", [user_id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM budgets WHERE user_id = ?1", [user_id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM recurring WHERE user_id = ?1", [user_id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM transfers WHERE user_id = ?1", [user_id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM transactions WHERE user_id = ?1", [user_id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM accounts WHERE user_id = ?1", [user_id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM categories WHERE user_id = ?1", [user_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Создаёт категории по умолчанию для пользователя, если у него ещё нет категорий.
pub fn seed_categories(conn: &Connection, user_id: i64) -> Result<(), String> {
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM categories WHERE user_id = ?1", [user_id], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    if count > 0 {
        return Ok(());
    }

    let default_categories = [
        ("Зарплата", "income", "#22c55e"),
        ("Подработка", "income", "#3b82f6"),
        ("Еда", "expense", "#ef4444"),
        ("Транспорт", "expense", "#f97316"),
        ("Коммунальные", "expense", "#eab308"),
        ("Здоровье", "expense", "#ec4899"),
        ("Развлечения", "expense", "#8b5cf6"),
        ("Одежда", "expense", "#06b6d4"),
        ("Прочее", "expense", "#64748b"),
    ];

    let mut stmt = conn
        .prepare("INSERT INTO categories (user_id, name, type, color) VALUES (?1, ?2, ?3, ?4)")
        .map_err(|e| e.to_string())?;

    for (name, cat_type, color) in default_categories {
        stmt.execute(rusqlite::params![user_id, name, cat_type, color])
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn get_tables(conn: &Connection) -> Vec<String> {
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap();
        stmt.query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }

    fn has_column(conn: &Connection, table: &str, column: &str) -> bool {
        conn.query_row(
            &format!(
                "SELECT COUNT(*) FROM pragma_table_info('{}') WHERE name = '{}'",
                table, column
            ),
            [],
            |r| r.get::<_, i32>(0),
        )
        .unwrap_or(0)
            > 0
    }

    fn has_index(conn: &Connection, index_name: &str) -> bool {
        conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name=?",
            [index_name],
            |r| r.get::<_, i32>(0),
        )
        .unwrap_or(0)
            > 0
    }

    // =====================================================================
    // Table Creation Tests
    // =====================================================================

    #[test]
    fn test_create_tables_creates_all_tables() {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();

        let tables = get_tables(&conn);

        assert!(tables.contains(&"accounts".to_string()));
        assert!(tables.contains(&"categories".to_string()));
        assert!(tables.contains(&"transactions".to_string()));
        assert!(tables.contains(&"recurring".to_string()));
        assert!(tables.contains(&"budgets".to_string()));
    }

    #[test]
    fn test_create_tables_idempotent() {
        let conn = Connection::open_in_memory().unwrap();

        // Call create_tables multiple times
        create_tables(&conn).unwrap();
        create_tables(&conn).unwrap();
        create_tables(&conn).unwrap();

        // Should still work without errors
        let tables = get_tables(&conn);
        assert!(tables.contains(&"accounts".to_string()));
    }

    #[test]
    fn test_accounts_table_columns() {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();

        assert!(has_column(&conn, "accounts", "id"));
        assert!(has_column(&conn, "accounts", "name"));
        assert!(has_column(&conn, "accounts", "type"));
        assert!(has_column(&conn, "accounts", "balance"));
        assert!(has_column(&conn, "accounts", "currency"));
        assert!(has_column(&conn, "accounts", "created_at"));
    }

    #[test]
    fn test_transactions_table_columns() {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();

        assert!(has_column(&conn, "transactions", "id"));
        assert!(has_column(&conn, "transactions", "account_id"));
        assert!(has_column(&conn, "transactions", "category_id"));
        assert!(has_column(&conn, "transactions", "amount"));
        assert!(has_column(&conn, "transactions", "type"));
        assert!(has_column(&conn, "transactions", "note"));
        assert!(has_column(&conn, "transactions", "date"));
    }

    #[test]
    fn test_recurring_table_has_migration_columns() {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();

        // These columns are added by migrations
        assert!(has_column(&conn, "recurring", "is_active"));
        assert!(has_column(&conn, "recurring", "last_processed"));
        assert!(has_column(&conn, "recurring", "note"));
        assert!(has_column(&conn, "recurring", "end_date"));
    }

    // =====================================================================
    // Index Tests
    // =====================================================================

    #[test]
    fn test_create_tables_creates_indexes() {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();

        // Transaction indexes
        assert!(has_index(&conn, "idx_transactions_date"));
        assert!(has_index(&conn, "idx_transactions_account"));
        assert!(has_index(&conn, "idx_transactions_category"));
        assert!(has_index(&conn, "idx_transactions_type"));

        // Recurring indexes
        assert!(has_index(&conn, "idx_recurring_next_date"));

        // Budget indexes
        assert!(has_index(&conn, "idx_budgets_category"));
    }

    // =====================================================================
    // Seed Categories Tests
    // =====================================================================

    #[test]
    fn test_seed_categories_creates_defaults() {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();
        seed_categories(&conn, 1).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM categories WHERE user_id = 1", [], |r| r.get(0))
            .unwrap();

        assert_eq!(count, 9); // 9 default categories
    }

    #[test]
    fn test_seed_categories_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();

        // Call seed_categories multiple times
        seed_categories(&conn, 1).unwrap();
        seed_categories(&conn, 1).unwrap();
        seed_categories(&conn, 1).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM categories WHERE user_id = 1", [], |r| r.get(0))
            .unwrap();

        // Should still have only 9 categories
        assert_eq!(count, 9);
    }

    #[test]
    fn test_seed_categories_has_income_and_expense() {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();
        seed_categories(&conn, 1).unwrap();

        let income_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM categories WHERE user_id = 1 AND type = 'income'",
                [],
                |r| r.get(0),
            )
            .unwrap();

        let expense_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM categories WHERE user_id = 1 AND type = 'expense'",
                [],
                |r| r.get(0),
            )
            .unwrap();

        assert!(income_count >= 2); // At least Зарплата and Подработка
        assert!(expense_count >= 7); // At least 7 expense categories
    }

    #[test]
    fn test_seed_categories_have_colors() {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();
        seed_categories(&conn, 1).unwrap();

        let null_color_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM categories WHERE user_id = 1 AND color IS NULL",
                [],
                |r| r.get(0),
            )
            .unwrap();

        // All default categories should have colors
        assert_eq!(null_color_count, 0);
    }

    // =====================================================================
    // Default Values Tests
    // =====================================================================

    #[test]
    fn test_accounts_default_balance() {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();

        conn.execute(
            "INSERT INTO accounts (name, type) VALUES ('Test', 'card')",
            [],
        )
        .unwrap();

        let balance: f64 = conn
            .query_row("SELECT balance FROM accounts WHERE name = 'Test'", [], |r| {
                r.get(0)
            })
            .unwrap();

        assert_eq!(balance, 0.0);
    }

    #[test]
    fn test_accounts_default_currency() {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();

        conn.execute(
            "INSERT INTO accounts (name, type) VALUES ('Test', 'card')",
            [],
        )
        .unwrap();

        let currency: String = conn
            .query_row(
                "SELECT currency FROM accounts WHERE name = 'Test'",
                [],
                |r| r.get(0),
            )
            .unwrap();

        assert_eq!(currency, "KZT");
    }
}
