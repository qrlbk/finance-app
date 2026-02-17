//! Smart Insights module
//! Analyzes spending patterns and provides recommendations

use std::collections::HashMap;
use rusqlite::Connection;
use serde::Serialize;
use chrono::{NaiveDate, Datelike, Months};

use crate::messages;

/// Spending pattern for a category
#[derive(Debug, Serialize, Clone)]
pub struct SpendingPattern {
    pub category: String,
    pub category_color: Option<String>,
    pub avg_amount: f64,
    pub total_transactions: usize,
    pub typical_frequency: String,
    pub trend: String,           // "increasing" | "decreasing" | "stable"
    pub trend_percent: f64,
}

/// Savings suggestion
#[derive(Debug, Serialize, Clone)]
pub struct SavingsSuggestion {
    pub category: String,
    pub current_spending: f64,
    pub suggested_limit: f64,
    pub potential_savings: f64,
    pub suggestion: String,
    pub confidence: f64,
}

/// Monthly comparison data
#[derive(Debug, Serialize, Clone)]
pub struct MonthlyComparison {
    pub current_month_total: f64,
    pub previous_month_total: f64,
    pub change_percent: f64,
    pub top_increase_category: Option<String>,
    pub top_decrease_category: Option<String>,
}

/// Comprehensive smart insights
#[derive(Debug, Serialize, Clone)]
pub struct SmartInsights {
    pub patterns: Vec<SpendingPattern>,
    pub suggestions: Vec<SavingsSuggestion>,
    pub monthly_comparison: MonthlyComparison,
    pub high_spending_days: Vec<String>,
}

/// Analyze spending patterns from transaction data for the given user.
pub fn analyze_spending_patterns(conn: &Connection, user_id: i64) -> Result<SmartInsights, String> {
    let patterns = get_category_patterns(conn, user_id)?;
    let suggestions = generate_savings_suggestions(conn, user_id, &patterns)?;
    let monthly_comparison = get_monthly_comparison(conn, user_id)?;
    let high_spending_days = get_high_spending_days(conn, user_id)?;

    Ok(SmartInsights {
        patterns,
        suggestions,
        monthly_comparison,
        high_spending_days,
    })
}

fn get_category_patterns(conn: &Connection, user_id: i64) -> Result<Vec<SpendingPattern>, String> {
    let now = chrono::Local::now();
    let three_months_ago = now.date_naive()
        .checked_sub_months(Months::new(3))
        .ok_or_else(|| messages::ERR_DATE_CALCULATION_FAILED.to_string())?;
    let one_month_ago = now.date_naive()
        .checked_sub_months(Months::new(1))
        .ok_or_else(|| messages::ERR_DATE_CALCULATION_FAILED.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT c.name, c.color, 
                    COUNT(*) as tx_count,
                    AVG(ABS(t.amount)) as avg_amount,
                    SUM(CASE WHEN t.date >= ?2 THEN ABS(t.amount) ELSE 0 END) as recent_total,
                    SUM(CASE WHEN t.date < ?2 AND t.date >= ?1 THEN ABS(t.amount) ELSE 0 END) as older_total
             FROM transactions t
             JOIN categories c ON t.category_id = c.id AND c.user_id = t.user_id
             WHERE t.user_id = ?3 AND t.type = 'expense' AND t.date >= ?1
             GROUP BY c.id
             HAVING tx_count >= 3
             ORDER BY avg_amount DESC"
        )
        .map_err(|e| e.to_string())?;

    let three_months_str = three_months_ago.format("%Y-%m-%d").to_string();
    let one_month_str = one_month_ago.format("%Y-%m-%d").to_string();

    let rows = stmt
        .query_map(rusqlite::params![&three_months_str, &one_month_str, user_id], |row| {
            let name: String = row.get(0)?;
            let color: Option<String> = row.get(1)?;
            let tx_count: i64 = row.get(2)?;
            let avg_amount: f64 = row.get(3)?;
            let recent_total: f64 = row.get(4)?;
            let older_total: f64 = row.get(5)?;
            Ok((name, color, tx_count, avg_amount, recent_total, older_total))
        })
        .map_err(|e| e.to_string())?;

    let mut patterns = Vec::new();
    
    for row in rows {
        let (name, color, tx_count, avg_amount, recent_total, older_total) = row.map_err(|e| e.to_string())?;
        
        // Calculate trend
        let (trend, trend_percent) = if older_total > 0.0 {
            let change = ((recent_total - older_total) / older_total) * 100.0;
            if change > 10.0 {
                ("increasing".to_string(), change)
            } else if change < -10.0 {
                ("decreasing".to_string(), change)
            } else {
                ("stable".to_string(), change)
            }
        } else {
            ("stable".to_string(), 0.0)
        };

        // Calculate typical frequency
        let freq_per_month = tx_count as f64 / 3.0;
        let typical_frequency = if freq_per_month >= 20.0 {
            "почти каждый день".to_string()
        } else if freq_per_month >= 8.0 {
            "несколько раз в неделю".to_string()
        } else if freq_per_month >= 4.0 {
            "раз в неделю".to_string()
        } else if freq_per_month >= 2.0 {
            "пару раз в месяц".to_string()
        } else {
            "редко".to_string()
        };

        patterns.push(SpendingPattern {
            category: name,
            category_color: color,
            avg_amount,
            total_transactions: tx_count as usize,
            typical_frequency,
            trend,
            trend_percent,
        });
    }

    Ok(patterns)
}

fn generate_savings_suggestions(
    _conn: &Connection,
    _user_id: i64,
    patterns: &[SpendingPattern],
) -> Result<Vec<SavingsSuggestion>, String> {
    let mut suggestions = Vec::new();

    // Find categories with high spending and increasing trend
    for pattern in patterns {
        // Skip if already decreasing
        if pattern.trend == "decreasing" {
            continue;
        }

        // High frequency categories (could be optimized)
        if pattern.total_transactions >= 20 && pattern.avg_amount < 5000.0 {
            let potential_savings = pattern.avg_amount * 0.2 * (pattern.total_transactions as f64 / 3.0);
            suggestions.push(SavingsSuggestion {
                category: pattern.category.clone(),
                current_spending: pattern.avg_amount * (pattern.total_transactions as f64 / 3.0),
                suggested_limit: pattern.avg_amount * 0.8 * (pattern.total_transactions as f64 / 3.0),
                potential_savings,
                suggestion: format!(
                    "Частые траты на \"{}\". Попробуйте сократить на 20%",
                    pattern.category
                ),
                confidence: 0.7,
            });
        }

        // Increasing trend categories
        if pattern.trend == "increasing" && pattern.trend_percent > 20.0 {
            let potential_savings = pattern.avg_amount * (pattern.trend_percent / 100.0);
            suggestions.push(SavingsSuggestion {
                category: pattern.category.clone(),
                current_spending: pattern.avg_amount * (pattern.total_transactions as f64 / 3.0),
                suggested_limit: pattern.avg_amount * 0.9 * (pattern.total_transactions as f64 / 3.0),
                potential_savings,
                suggestion: format!(
                    "Расходы на \"{}\" выросли на {:.0}%",
                    pattern.category, pattern.trend_percent
                ),
                confidence: 0.8,
            });
        }
    }

    // Sort by potential savings
    suggestions.sort_by(|a, b| b.potential_savings.partial_cmp(&a.potential_savings).unwrap_or(std::cmp::Ordering::Equal));
    
    // Limit to top 5
    suggestions.truncate(5);

    Ok(suggestions)
}

fn get_monthly_comparison(conn: &Connection, user_id: i64) -> Result<MonthlyComparison, String> {
    let now = chrono::Local::now();
    let current_month_start = format!("{}-{:02}-01", now.year(), now.month());
    let prev_month_start = now.date_naive()
        .checked_sub_months(Months::new(1))
        .map(|d| format!("{}-{:02}-01", d.year(), d.month()))
        .ok_or_else(|| messages::ERR_DATE_CALCULATION_FAILED.to_string())?;

    let current_total: f64 = conn
        .query_row(
            "SELECT COALESCE(ABS(SUM(amount)), 0) FROM transactions 
             WHERE user_id = ?1 AND type = 'expense' AND date >= ?2",
            rusqlite::params![user_id, &current_month_start],
            |r| r.get(0),
        )
        .unwrap_or(0.0);

    let prev_total: f64 = conn
        .query_row(
            "SELECT COALESCE(ABS(SUM(amount)), 0) FROM transactions 
             WHERE user_id = ?1 AND type = 'expense' AND date >= ?2 AND date < ?3",
            rusqlite::params![user_id, &prev_month_start, &current_month_start],
            |r| r.get(0),
        )
        .unwrap_or(0.0);

    let change_percent = if prev_total > 0.0 {
        ((current_total - prev_total) / prev_total) * 100.0
    } else {
        0.0
    };

    let top_increase: Option<String> = conn
        .query_row(
            "SELECT c.name FROM transactions t
             JOIN categories c ON t.category_id = c.id AND c.user_id = t.user_id
             WHERE t.user_id = ?1 AND t.type = 'expense' AND t.date >= ?2
             GROUP BY c.id
             ORDER BY SUM(ABS(t.amount)) DESC
             LIMIT 1",
            rusqlite::params![user_id, &current_month_start],
            |r| r.get(0),
        )
        .ok();

    Ok(MonthlyComparison {
        current_month_total: current_total,
        previous_month_total: prev_total,
        change_percent,
        top_increase_category: top_increase.clone(),
        top_decrease_category: None, // Could implement later
    })
}

fn get_high_spending_days(conn: &Connection, user_id: i64) -> Result<Vec<String>, String> {
    let now = chrono::Local::now();
    let month_start = format!("{}-{:02}-01", now.year(), now.month());

    let mut stmt = conn
        .prepare(
            "SELECT date, SUM(ABS(amount)) as total
             FROM transactions
             WHERE user_id = ?1 AND type = 'expense' AND date >= ?2
             GROUP BY date
             ORDER BY total DESC
             LIMIT 3"
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(rusqlite::params![user_id, &month_start], |row| {
            let date: String = row.get(0)?;
            let total: f64 = row.get(1)?;
            Ok(format!("{}: {:.0} ₸", date, total))
        })
        .map_err(|e| e.to_string())?;

    let days: Vec<String> = rows.filter_map(|r| r.ok()).collect();
    Ok(days)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spending_pattern_creation() {
        let pattern = SpendingPattern {
            category: "Еда".to_string(),
            category_color: Some("#ef4444".to_string()),
            avg_amount: 2500.0,
            total_transactions: 30,
            typical_frequency: "несколько раз в неделю".to_string(),
            trend: "stable".to_string(),
            trend_percent: 5.0,
        };
        assert_eq!(pattern.category, "Еда");
        assert_eq!(pattern.trend, "stable");
    }
}
