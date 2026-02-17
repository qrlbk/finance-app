//! ML and LLM commands: predict category, train, insights, chat.

use super::common::{with_connection, with_connection_and_user, AppState};
use crate::db;
use crate::embedded_llm;
use crate::llm;
use crate::ml::{self, model::ModelStatus};
use serde::Serialize;
use tauri::State;

#[derive(Clone)]
pub struct LlmConfig {
    pub enabled: bool,
    pub url: String,
    pub model: String,
}

#[derive(Serialize)]
pub struct CategoryPrediction {
    pub category_id: i64,
    pub category_name: String,
    pub confidence: f64,
}

#[derive(Serialize)]
pub struct TrainResult {
    pub success: bool,
    pub sample_count: usize,
    pub accuracy: Option<f64>,
    pub message: String,
}

#[derive(Serialize)]
pub struct Insights {
    pub anomalies: Vec<ml::Anomaly>,
    pub forecast: Option<ml::Forecast>,
    pub months_of_data: i32,
}

#[derive(Clone, Serialize)]
pub struct TestEmbeddedLlmResult {
    pub success: bool,
    pub message: String,
}

#[derive(Clone, serde::Serialize)]
pub enum EnsureOllamaResult {
    Ready,
    OpenedDownload,
}

#[derive(Serialize)]
pub struct CategoryForecastResult {
    pub overall: ml::Forecast,
    pub by_category: Vec<ml::forecast::CategoryForecast>,
}

#[tauri::command]
pub fn predict_category(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    note: String,
    amount: Option<f64>,
    date: Option<String>,
    confidence_threshold: Option<f64>,
    use_llm: Option<bool>,
    ollama_url: Option<String>,
    ollama_model: Option<String>,
    transaction_type: Option<String>,
    use_embedded: Option<bool>,
) -> Result<Option<CategoryPrediction>, String> {
    let note = note.trim();
    if note.is_empty() || note.len() < 3 {
        return Ok(None);
    }
    let threshold = confidence_threshold.map(|t| t.clamp(0.2, 0.9)).unwrap_or(0.3);
    let (enabled, url, model) = if use_embedded.unwrap_or(false) {
        (true, "http://127.0.0.1:11434".to_string(), embedded_llm::EMBEDDED_MODEL_NAME.to_string())
    } else {
        (
            use_llm.unwrap_or(false),
            ollama_url.unwrap_or_else(|| "http://127.0.0.1:11434".to_string()),
            ollama_model.unwrap_or_else(|| "llama3.2".to_string()),
        )
    };
    let llm_config = LlmConfig { enabled, url, model };
    let tx_type = transaction_type.as_deref().unwrap_or("expense");

    let result = with_connection_and_user(&app_handle, &state, |conn, user_id| {
        if let Some((category_id, category_name)) = ml::rules::lookup(conn, user_id, note) {
            return Ok(Some(CategoryPrediction { category_id, category_name, confidence: 1.0 }));
        }
        let model_path = ml::model::get_model_path()?;
        if model_path.exists() {
            let model = ml::CategoryModel::load(&model_path)?;
            let prediction = model.predict_with_context(note, amount, date.as_deref());
            if let Some((category_id, category_name, confidence)) = prediction {
                if confidence >= threshold {
                    return Ok(Some(CategoryPrediction { category_id, category_name, confidence }));
                }
            }
        }
        if llm_config.enabled {
            let categories = db::get_categories(conn, user_id)?;
            let expense_or_income: Vec<db::Category> =
                categories.into_iter().filter(|c| c.category_type == tx_type).collect();
            if let Ok(Some((category_id, category_name))) =
                llm::suggest_category_llm(&expense_or_income, tx_type, note, &llm_config.url, &llm_config.model)
            {
                return Ok(Some(CategoryPrediction { category_id, category_name, confidence: 0.85 }));
            }
        }
        Ok(None)
    })?;
    Ok(result)
}

#[tauri::command]
pub fn train_model(app_handle: tauri::AppHandle, state: State<AppState>) -> Result<TrainResult, String> {
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
            Err(e) => Ok(TrainResult {
                success: false,
                sample_count: 0,
                accuracy: None,
                message: e,
            }),
        }
    })
}

#[tauri::command]
pub fn get_model_status(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
) -> Result<ModelStatus, String> {
    let (count, count_note_no_cat): (Option<usize>, Option<usize>) =
        with_connection_and_user(&app_handle, &state, |conn, user_id| {
            let with_cat: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM transactions WHERE user_id = ?1 AND category_id IS NOT NULL AND note IS NOT NULL AND note != ''",
                    [user_id],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            let note_no_cat: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM transactions WHERE user_id = ?1 AND (note IS NOT NULL AND note != '') AND category_id IS NULL",
                    [user_id],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            Ok((with_cat as usize, note_no_cat as usize))
        })
        .map(|(a, b)| (Some(a), Some(b)))
        .unwrap_or((None, None));

    let model_path = ml::model::get_model_path()?;
    if !model_path.exists() {
        let mut status = ModelStatus::default();
        status.transactions_with_categories_count = count;
        status.transactions_with_note_no_category = count_note_no_cat;
        return Ok(status);
    }
    match ml::CategoryModel::load(&model_path) {
        Ok(model) => {
            let mut status = model.status(count);
            status.transactions_with_note_no_category = count_note_no_cat;
            Ok(status)
        }
        Err(_) => {
            let mut status = ModelStatus::default();
            status.transactions_with_categories_count = count;
            status.transactions_with_note_no_category = count_note_no_cat;
            Ok(status)
        }
    }
}

#[tauri::command]
pub fn get_embedded_llm_status() -> embedded_llm::EmbeddedLlmStatus {
    embedded_llm::get_embedded_llm_status()
}

#[tauri::command]
pub fn download_and_register_embedded_model() -> Result<(), String> {
    embedded_llm::download_and_register_embedded_model()
}

#[tauri::command]
pub fn test_embedded_llm() -> Result<TestEmbeddedLlmResult, String> {
    const TEST_PROMPT: &str = "Answer with exactly one word: OK";
    match llm::ask_ollama(
        "http://127.0.0.1:11434",
        embedded_llm::EMBEDDED_MODEL_NAME,
        TEST_PROMPT,
    ) {
        Ok(resp) if !resp.trim().is_empty() => Ok(TestEmbeddedLlmResult {
            success: true,
            message: "Подключение успешно. Модель отвечает.".to_string(),
        }),
        Ok(_) => Ok(TestEmbeddedLlmResult {
            success: false,
            message: "Модель вернула пустой ответ.".to_string(),
        }),
        Err(e) => Ok(TestEmbeddedLlmResult { success: false, message: e }),
    }
}

#[tauri::command]
pub fn start_ollama_server() {
    embedded_llm::try_start_ollama_server();
}

#[tauri::command]
pub fn chat_message(
    message: String,
    system_prompt: Option<String>,
    context: Option<String>,
    use_embedded: Option<bool>,
    ollama_url: Option<String>,
    ollama_model: Option<String>,
) -> Result<String, String> {
    let msg = message.trim();
    if msg.is_empty() {
        return Err("Введите сообщение".to_string());
    }
    let (url, model) = if use_embedded.unwrap_or(false) {
        ("http://127.0.0.1:11434".to_string(), embedded_llm::EMBEDDED_MODEL_NAME.to_string())
    } else {
        (
            ollama_url.unwrap_or_else(|| "http://127.0.0.1:11434".to_string()),
            ollama_model.unwrap_or_else(|| "llama3.2".to_string()),
        )
    };
    let sys = match (system_prompt.as_deref(), context.as_deref()) {
        (Some(s), Some(c)) if !c.trim().is_empty() => {
            format!("{}\n\nАктуальные данные пользователя (отвечай только на их основе):\n{}\n", s.trim(), c.trim())
        }
        (Some(s), _) => s.trim().to_string(),
        (None, Some(c)) if !c.trim().is_empty() => {
            format!("Ты помощник по финансам. Отвечай на русском. Актуальные данные пользователя:\n{}\nОтвечай на основе этих данных.", c.trim())
        }
        (None, _) => String::new(),
    };
    let prompt = if sys.is_empty() {
        format!("User: {}\n\nAssistant:", msg)
    } else {
        format!("{}\n\nUser: {}\n\nAssistant:", sys, msg)
    };
    llm::ask_ollama(&url, &model, &prompt)
}

fn open_url_in_browser(url: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(url).spawn().map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd").args(["/C", "start", "", url]).spawn().map_err(|e| e.to_string())?;
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        std::process::Command::new("xdg-open").arg(url).spawn().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn ensure_ollama_installed() -> Result<EnsureOllamaResult, String> {
    const OLLAMA_DOWNLOAD_URL: &str = "https://ollama.com/download";
    let check = std::process::Command::new("ollama").arg("--version").output();
    match check {
        Ok(out) if out.status.success() => {
            embedded_llm::try_start_ollama_server();
            return Ok(EnsureOllamaResult::Ready);
        }
        Ok(_) => {
            embedded_llm::try_start_ollama_server();
            return Ok(EnsureOllamaResult::Ready);
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(format!("Проверка Ollama: {}", e)),
    }
    #[cfg(target_os = "macos")]
    {
        let brew_ok = std::process::Command::new("brew").arg("--version").output();
        if let Ok(out) = brew_ok {
            if out.status.success() {
                let install = std::process::Command::new("brew").args(["install", "ollama"]).output();
                if let Ok(out) = install {
                    if out.status.success() {
                        embedded_llm::try_start_ollama_server();
                        return Ok(EnsureOllamaResult::Ready);
                    }
                }
            }
        }
        open_url_in_browser(OLLAMA_DOWNLOAD_URL)?;
        return Ok(EnsureOllamaResult::OpenedDownload);
    }
    #[cfg(target_os = "windows")]
    {
        let winget = std::process::Command::new("winget")
            .args([
                "install", "Ollama.Ollama",
                "--accept-package-agreements", "--accept-source-agreements", "--silent",
            ])
            .output();
        if let Ok(out) = winget {
            if out.status.success() {
                embedded_llm::try_start_ollama_server();
                return Ok(EnsureOllamaResult::Ready);
            }
        }
        open_url_in_browser(OLLAMA_DOWNLOAD_URL)?;
        return Ok(EnsureOllamaResult::OpenedDownload);
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        open_url_in_browser(OLLAMA_DOWNLOAD_URL)?;
        Ok(EnsureOllamaResult::OpenedDownload)
    }
}

#[tauri::command]
pub fn get_insights(app_handle: tauri::AppHandle, state: State<AppState>) -> Result<Insights, String> {
    with_connection_and_user(&app_handle, &state, |conn, user_id| {
        let months_of_data: i32 = conn
            .query_row(
                "SELECT COUNT(DISTINCT strftime('%Y-%m', date)) FROM transactions WHERE user_id = ?1 AND type = 'expense' AND date >= date('now', '-12 months')",
                [user_id],
                |row| row.get(0),
            )
            .unwrap_or(0);
        let anomalies = ml::AnomalyDetector::detect_anomalies(conn, user_id, 30).unwrap_or_default();
        let forecast = ml::ExpenseForecaster::forecast_next_month(conn, user_id).ok();
        Ok(Insights { anomalies, forecast, months_of_data })
    })
}

#[tauri::command]
pub fn get_smart_insights(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
) -> Result<ml::SmartInsights, String> {
    with_connection_and_user(&app_handle, &state, |conn, user_id| {
        ml::insights::analyze_spending_patterns(conn, user_id)
    })
}

#[tauri::command]
pub fn get_forecast_details(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
) -> Result<CategoryForecastResult, String> {
    with_connection_and_user(&app_handle, &state, |conn, user_id| {
        let (overall, by_category) = ml::ExpenseForecaster::forecast_with_categories(conn, user_id)?;
        Ok(CategoryForecastResult { overall, by_category })
    })
}
