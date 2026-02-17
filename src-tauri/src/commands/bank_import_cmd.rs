//! Bank statement parse and import commands.

use super::common::{with_connection, with_connection_and_user, AppState};
use super::ml_cmd::{CategoryPrediction, LlmConfig};
use crate::bank_import::{self, ImportResult as BankImportResult, ImportTransaction, ParsedStatement};
use crate::db;
use crate::embedded_llm;
use crate::llm;
use crate::ml;
use serde::Deserialize;
use std::path::PathBuf;
use tauri::State;

#[tauri::command]
pub fn parse_bank_statement(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    path: String,
    use_llm: Option<bool>,
    ollama_url: Option<String>,
    ollama_model: Option<String>,
    use_embedded: Option<bool>,
) -> Result<ParsedStatement, String> {
    let file_path = PathBuf::from(&path);
    if !file_path.exists() {
        return Err("Файл не найден".to_string());
    }
    let extension = file_path.extension().and_then(|e| e.to_str()).map(|e| e.to_lowercase());
    if extension != Some("pdf".to_string()) {
        return Err("Поддерживаются только PDF файлы".to_string());
    }
    let llm_config = if use_embedded.unwrap_or(false) {
        LlmConfig {
            enabled: true,
            url: "http://127.0.0.1:11434".to_string(),
            model: embedded_llm::EMBEDDED_MODEL_NAME.to_string(),
        }
    } else {
        LlmConfig {
            enabled: use_llm.unwrap_or(false),
            url: ollama_url.unwrap_or_else(|| "http://127.0.0.1:11434".to_string()),
            model: ollama_model.unwrap_or_else(|| "llama3.2".to_string()),
        }
    };
    let mut statement = bank_import::parse_statement(&path)?;
    with_connection(&app_handle, &state, |conn| {
        for tx in &mut statement.transactions {
            let tx_type = tx.transaction_type.as_str();
            if let Ok(Some(prediction)) =
                predict_category_internal(conn, &tx.description, tx_type, &llm_config)
            {
                tx.suggested_category_id = Some(prediction.category_id);
                tx.confidence = Some(prediction.confidence);
            }
            tx.is_duplicate = check_duplicate(conn, &tx.date, tx.amount, &tx.description)?;
        }
        Ok(())
    })?;
    Ok(statement)
}

pub fn predict_category_internal(
    conn: &rusqlite::Connection,
    note: &str,
    transaction_type: &str,
    llm_config: &LlmConfig,
) -> Result<Option<CategoryPrediction>, String> {
    let note = note.trim();
    if note.is_empty() || note.len() < 3 {
        return Ok(None);
    }
    let user_id = 1_i64; // used for rules lookup during import (no session yet in context)
    if let Some((category_id, category_name)) = ml::rules::lookup(conn, user_id, note) {
        return Ok(Some(CategoryPrediction { category_id, category_name, confidence: 1.0 }));
    }
    let model_path = ml::model::get_model_path()?;
    if model_path.exists() {
        let model = ml::CategoryModel::load(&model_path)?;
        let prediction = model.predict_with_context(note, None, None);
        if let Some((category_id, category_name, confidence)) = prediction {
            if confidence >= 0.3 {
                return Ok(Some(CategoryPrediction { category_id, category_name, confidence }));
            }
        }
    }
    if llm_config.enabled {
        let categories = db::get_categories(conn, user_id)?;
        let filtered: Vec<db::Category> =
            categories.into_iter().filter(|c| c.category_type == transaction_type).collect();
        if let Ok(Some((category_id, category_name))) = llm::suggest_category_llm(
            &filtered,
            transaction_type,
            note,
            &llm_config.url,
            &llm_config.model,
        ) {
            return Ok(Some(CategoryPrediction { category_id, category_name, confidence: 0.85 }));
        }
    }
    Ok(None)
}

fn normalize_note(s: &str) -> String {
    s.trim().split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn check_duplicate(
    conn: &rusqlite::Connection,
    date: &str,
    amount: f64,
    description: &str,
) -> Result<bool, String> {
    let normalized = normalize_note(description);
    let mut stmt = conn
        .prepare(
            "SELECT note FROM transactions WHERE date = ?1 AND ABS(ABS(amount) - ?2) < 0.01",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params![date, amount.abs()], |row| row.get::<_, Option<String>>(0))
        .map_err(|e| e.to_string())?;
    for note in rows {
        let note = note.map_err(|e| e.to_string())?;
        if normalized == normalize_note(note.as_deref().unwrap_or("")) {
            return Ok(true);
        }
    }
    Ok(false)
}

#[derive(Deserialize)]
pub struct ImportBankTransactionsInput {
    pub transactions: Vec<ImportTransaction>,
    pub account_id: i64,
    pub skip_duplicates: bool,
}

#[tauri::command]
pub fn import_bank_transactions(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    input: ImportBankTransactionsInput,
) -> Result<BankImportResult, String> {
    with_connection_and_user(&app_handle, &state, |conn, user_id| {
        if !db::account_exists(conn, user_id, input.account_id)? {
            return Err("Счёт не найден".to_string());
        }
        let mut imported = 0;
        let mut skipped_duplicates = 0;
        let mut failed = 0;
        let mut errors = Vec::new();
        for tx in input.transactions {
            if input.skip_duplicates && tx.skip_if_duplicate {
                let is_dup = check_duplicate(conn, &tx.date, tx.amount, &tx.description)?;
                if is_dup {
                    skipped_duplicates += 1;
                    continue;
                }
            }
            if let Some(cat_id) = tx.category_id {
                if !db::category_exists_and_type(conn, user_id, cat_id, &tx.transaction_type)? {
                    errors.push(format!("Категория {} не найдена для транзакции {}", cat_id, tx.description));
                    failed += 1;
                    continue;
                }
            }
            match db::create_transaction(
                conn,
                user_id,
                input.account_id,
                tx.category_id,
                tx.amount,
                &tx.transaction_type,
                Some(tx.description.as_str()),
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
