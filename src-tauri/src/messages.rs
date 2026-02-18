//! User-facing error and UI messages (Russian).
//! Centralized for consistency and future i18n.

// --- Commands / paths ---
pub const ERR_INVALID_PATH: &str = "Недопустимый путь к файлу";
pub const ERR_FILE_NOT_FOUND: &str = "Файл не найден";
pub const ERR_UNSUPPORTED_FORMAT: &str = "Неподдерживаемый формат";

// --- Export ---
pub const ERR_UNSUPPORTED_FORMAT_WITH: &str = "Неподдерживаемый формат: ";
pub const ERR_NO_SHEETS: &str = "В книге нет листов";

// --- Import (CSV/JSON row errors) ---
pub fn row_account_not_found(row: usize, account: &str) -> String {
    format!("Строка {}: счёт «{}» не найден", row, account)
}
pub fn row_invalid_amount(row: usize) -> String {
    format!("Строка {}: некорректная сумма", row)
}
pub fn row_error(row: usize, detail: &str) -> String {
    format!("Строка {}: {}", row, detail)
}
pub fn tx_account_not_found(tx_id: i64, account: &str) -> String {
    format!("Транзакция {}: счёт «{}» не найден", tx_id, account)
}
pub fn tx_error(tx_id: i64, detail: &str) -> String {
    format!("Транзакция {}: {}", tx_id, detail)
}

// --- DB / date calculation (user-facing) ---
pub const ERR_INVALID_DB_PATH: &str = "Недопустимый путь к базе данных";
pub const ERR_INVALID_CURRENT_DATE: &str = "Некорректная текущая дата";
pub const ERR_DATE_OVERFLOW: &str = "Переполнение даты";
pub const ERR_NEXT_MONTH_CALC: &str = "Не удалось вычислить следующий месяц";
pub const ERR_NEXT_YEAR_CALC: &str = "Не удалось вычислить следующий год";
pub const ERR_DATE_CALCULATION_FAILED: &str = "Ошибка расчёта даты";

pub fn err_invalid_date(detail: &str) -> String {
    format!("Некорректная дата: {}", detail)
}

// --- SQLCipher / DB connection (user-facing) ---
pub const ERR_SQLCIPHER_CONFIGURE: &str = "Не удалось настроить SQLCipher";
pub const ERR_INVALID_KEY_OR_CORRUPTED: &str = "Неверный ключ шифрования или повреждённая база данных";
pub const ERR_SOURCE_DB_NOT_VALID: &str = "Исходная база данных недействительна";
pub const ERR_MIGRATE_DATABASE: &str = "Не удалось перенести базу данных";
pub const ERR_BACKUP_OLD_DATABASE: &str = "Не удалось создать резервную копию старой базы данных";
pub const ERR_RENAME_ENCRYPTED_DATABASE: &str = "Не удалось переименовать зашифрованную базу данных";

// --- Crypto (user-facing) ---
pub const ERR_READ_ENCRYPTION_KEY: &str = "Не удалось прочитать ключ шифрования";
pub const ERR_WRITE_ENCRYPTION_KEY: &str = "Не удалось записать ключ шифрования";
pub const ERR_CREATE_KEY_DIRECTORY: &str = "Не удалось создать директорию для ключа";
pub const ERR_DELETE_KEY: &str = "Не удалось удалить файл ключа";

// --- Security (user-facing) ---
pub const ERR_INVALID_PATH_TRAVERSAL: &str = "Недопустимый путь: обход директорий запрещён";
pub const ERR_PATH_TOO_LONG: &str = "Путь слишком длинный";

// --- Restore backup (user-facing) ---
pub const ERR_RESTORE_BACKUP_OPEN: &str = "Не удалось открыть бэкап. Убедитесь, что он создан этим приложением и пароль не менялся.";
