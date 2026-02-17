//! LLM-based category suggestion (Ollama). Used when rules and ML model don't return a result.

use crate::db::Category;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::time::Duration;

const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Ollama generate API request
#[derive(serde::Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

/// Ollama generate API response (non-streaming)
#[derive(Deserialize)]
struct OllamaResponse {
    response: Option<String>,
}

/// Ask Ollama for a completion. Returns the model's text response or an error.
pub fn ask_ollama(
    base_url: &str,
    model: &str,
    prompt: &str,
) -> Result<String, String> {
    let url = format!("{}/api/generate", base_url.trim_end_matches('/'));
    let client = Client::builder()
        .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
        .build()
        .map_err(|e| format!("HTTP client: {}", e))?;

    let body = OllamaRequest {
        model: model.to_string(),
        prompt: prompt.to_string(),
        stream: false,
    };

    let response = client
        .post(&url)
        .json(&body)
        .send()
        .map_err(|e| format!("Ollama запрос не удался: {}. Убедитесь, что Ollama запущен (ollama serve) и модель загружена (ollama pull {}).", e, model))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().unwrap_or_default();
        return Err(format!("Ollama вернул {}: {}", status, text));
    }

    let parsed: OllamaResponse = response
        .json()
        .map_err(|e| format!("Ответ Ollama: {}", e))?;

    Ok(parsed.response.unwrap_or_default().trim().to_string())
}

/// Build prompt for category suggestion: list categories and ask to pick one for the transaction.
fn build_prompt(categories: &[Category], transaction_type: &str, description: &str) -> String {
    let type_label = if transaction_type == "income" {
        "доходов"
    } else {
        "расходов"
    };
    let category_names: Vec<&str> = categories.iter().map(|c| c.name.as_str()).collect();
    let list = category_names.join(", ");
    format!(
        r#"Ты помощник по категоризации операций. Список категорий {} (выбери ровно одну): {}.
Описание операции: "{}"
Ответь только одним словом или короткой фразой — названием категории из списка. Без кавычек, без пояснений."#,
        type_label, list, description
    )
}

/// Find best matching category by name (case-insensitive, trim). Returns (id, name) if found.
fn match_category_name(categories: &[Category], response: &str) -> Option<(i64, String)> {
    let normalized = response.trim().to_lowercase();
    if normalized.is_empty() {
        return None;
    }
    for cat in categories {
        if cat.name.to_lowercase() == normalized {
            return Some((cat.id, cat.name.clone()));
        }
    }
    // Allow partial match: response contains category name or vice versa
    for cat in categories {
        let cat_lower = cat.name.to_lowercase();
        if normalized.contains(&cat_lower) || cat_lower.contains(&normalized) {
            return Some((cat.id, cat.name.clone()));
        }
    }
    None
}

/// Suggest category for a transaction using Ollama. Returns (category_id, category_name) or None.
pub fn suggest_category_llm(
    categories: &[Category],
    transaction_type: &str,
    description: &str,
    ollama_url: &str,
    ollama_model: &str,
) -> Result<Option<(i64, String)>, String> {
    if description.trim().len() < 2 {
        return Ok(None);
    }
    let prompt = build_prompt(categories, transaction_type, description);
    let response = ask_ollama(ollama_url, ollama_model, &prompt)?;
    Ok(match_category_name(categories, &response))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_expense_categories() -> Vec<Category> {
        vec![
            Category {
                id: 1,
                name: "Еда".to_string(),
                category_type: "expense".to_string(),
                icon: None,
                color: None,
                parent_id: None,
            },
            Category {
                id: 2,
                name: "Транспорт".to_string(),
                category_type: "expense".to_string(),
                icon: None,
                color: None,
                parent_id: None,
            },
        ]
    }

    fn sample_income_categories() -> Vec<Category> {
        vec![
            Category {
                id: 10,
                name: "Зарплата".to_string(),
                category_type: "income".to_string(),
                icon: None,
                color: None,
                parent_id: None,
            },
        ]
    }

    #[test]
    fn test_build_prompt_expense() {
        let categories = sample_expense_categories();
        let prompt = build_prompt(&categories, "expense", "Покупка в магазине");
        assert!(prompt.contains("категорий расходов"));
        assert!(prompt.contains("Еда"));
        assert!(prompt.contains("Транспорт"));
        assert!(prompt.contains("Покупка в магазине"));
        assert!(prompt.contains("выбери ровно одну"));
    }

    #[test]
    fn test_build_prompt_income() {
        let categories = sample_income_categories();
        let prompt = build_prompt(&categories, "income", "Зарплата за январь");
        assert!(prompt.contains("категорий доходов"));
        assert!(prompt.contains("Зарплата"));
        assert!(prompt.contains("Зарплата за январь"));
    }

    #[test]
    fn test_match_category_name_exact() {
        let categories = sample_expense_categories();
        assert_eq!(
            match_category_name(&categories, "Еда"),
            Some((1, "Еда".to_string()))
        );
        assert_eq!(
            match_category_name(&categories, "  еда  "),
            Some((1, "Еда".to_string()))
        );
        assert_eq!(match_category_name(&categories, "Другое"), None);
    }

    #[test]
    fn test_match_category_name_partial() {
        let categories = sample_expense_categories();
        assert_eq!(
            match_category_name(&categories, "Транспорт"),
            Some((2, "Транспорт".to_string()))
        );
        assert_eq!(
            match_category_name(&categories, "Ответ: Еда"),
            Some((1, "Еда".to_string()))
        );
    }

    /// Integration test: real Ollama request. Run with `cargo test test_ollama_integration -- --ignored`
    /// Requires Ollama running at 127.0.0.1:11434 with finance-embedded or similar model.
    #[test]
    #[ignore]
    fn test_ollama_integration() {
        let categories = sample_expense_categories();
        let url = "http://127.0.0.1:11434";
        let model = "finance-embedded";
        let prompt = build_prompt(&categories, "expense", "Оплата в супермаркете");
        let response = match ask_ollama(url, model, &prompt) {
            Ok(r) => r,
            Err(_) => return, // Ollama not running or model not available — skip
        };
        assert!(!response.trim().is_empty(), "Ollama response should not be empty");
        let matched = match_category_name(&categories, &response);
        assert!(
            matched.is_some(),
            "Ollama response '{}' should match one of the categories",
            response
        );
    }
}
