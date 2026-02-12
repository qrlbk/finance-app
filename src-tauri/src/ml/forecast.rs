use rusqlite::Connection;
use serde::Serialize;

/// Expense forecast result
#[derive(Serialize, Clone)]
pub struct Forecast {
    /// Predicted expense for next month
    pub predicted_expense: f64,
    /// Lower bound of confidence interval (95%)
    pub confidence_low: f64,
    /// Upper bound of confidence interval (95%)
    pub confidence_high: f64,
    /// Trend direction
    pub trend: String, // "up" | "down" | "stable"
    /// Trend percentage change
    pub trend_percent: f64,
}

/// Holt's double exponential smoothing result
struct HoltResult {
    level: f64,
    trend: f64,
}

/// Expense forecaster using exponential smoothing
pub struct ExpenseForecaster;

impl ExpenseForecaster {
    /// Forecast expenses for the next month
    /// Uses Holt's double exponential smoothing for better trend handling
    pub fn forecast_next_month(conn: &Connection) -> Result<Forecast, String> {
        // Get monthly expense totals for the last 12 months
        let mut stmt = conn
            .prepare(
                "SELECT strftime('%Y-%m', date) as month, SUM(amount) as total
                 FROM transactions
                 WHERE type = 'expense'
                   AND date >= date('now', '-12 months')
                 GROUP BY month
                 ORDER BY month"
            )
            .map_err(|e| format!("Query error: {}", e))?;

        let monthly_totals: Vec<f64> = stmt
            .query_map([], |row| {
                let total: f64 = row.get(1)?;
                Ok(total)
            })
            .map_err(|e| format!("Query error: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        if monthly_totals.len() < 3 {
            return Err("Недостаточно данных для прогноза. Нужно минимум 3 месяца истории расходов.".to_string());
        }

        // Use Holt's method for better trend handling when we have enough data
        let prediction = if monthly_totals.len() >= 6 {
            Self::forecast_with_holt(&monthly_totals, 1)
        } else {
            // Fall back to simple exponential smoothing for less data
            Self::exponential_smoothing(&monthly_totals, 0.3)
        };

        // Calculate prediction error (RMSE-based)
        let alpha = 0.3;
        let errors = Self::calculate_errors(&monthly_totals, alpha);
        let rmse = Self::calculate_rmse(&errors);
        
        // 95% confidence interval (approximately 2 * RMSE)
        let confidence_low = (prediction - 2.0 * rmse).max(0.0);
        let confidence_high = prediction + 2.0 * rmse;

        // Calculate trend
        let (trend, trend_percent) = Self::calculate_trend(&monthly_totals, prediction);

        Ok(Forecast {
            predicted_expense: prediction.round(),
            confidence_low: confidence_low.round(),
            confidence_high: confidence_high.round(),
            trend,
            trend_percent,
        })
    }

    /// Simple Exponential Smoothing
    /// Returns forecast for next period
    fn exponential_smoothing(data: &[f64], alpha: f64) -> f64 {
        if data.is_empty() {
            return 0.0;
        }

        let mut smoothed = data[0];
        
        for &value in &data[1..] {
            smoothed = alpha * value + (1.0 - alpha) * smoothed;
        }

        smoothed
    }

    /// Holt's Double Exponential Smoothing (Linear trend)
    /// Better for data with trends
    fn holt_smoothing(data: &[f64], alpha: f64, beta: f64) -> HoltResult {
        if data.is_empty() {
            return HoltResult { level: 0.0, trend: 0.0 };
        }
        if data.len() == 1 {
            return HoltResult { level: data[0], trend: 0.0 };
        }

        // Initialize level and trend
        let mut level = data[0];
        let mut trend = data[1] - data[0];

        for &value in &data[1..] {
            let prev_level = level;
            level = alpha * value + (1.0 - alpha) * (prev_level + trend);
            trend = beta * (level - prev_level) + (1.0 - beta) * trend;
        }

        HoltResult { level, trend }
    }

    /// Forecast using Holt's method
    fn forecast_with_holt(data: &[f64], periods_ahead: i32) -> f64 {
        let result = Self::holt_smoothing(data, 0.3, 0.1);
        (result.level + result.trend * periods_ahead as f64).max(0.0)
    }

    /// Calculate one-step-ahead forecast errors for RMSE
    fn calculate_errors(data: &[f64], alpha: f64) -> Vec<f64> {
        if data.len() < 2 {
            return vec![];
        }

        let mut errors = Vec::new();
        let mut smoothed = data[0];

        for &actual in &data[1..] {
            let forecast = smoothed;
            errors.push(actual - forecast);
            smoothed = alpha * actual + (1.0 - alpha) * smoothed;
        }

        errors
    }

    /// Calculate Root Mean Square Error
    fn calculate_rmse(errors: &[f64]) -> f64 {
        if errors.is_empty() {
            return 0.0;
        }

        let mse: f64 = errors.iter().map(|e| e * e).sum::<f64>() / errors.len() as f64;
        mse.sqrt()
    }

    /// Calculate trend direction and percentage
    fn calculate_trend(historical: &[f64], prediction: f64) -> (String, f64) {
        if historical.is_empty() {
            return ("stable".to_string(), 0.0);
        }

        // Compare prediction to last month
        let last_month = historical.last().copied().unwrap_or(0.0);
        
        if last_month == 0.0 {
            return ("stable".to_string(), 0.0);
        }

        let percent_change = ((prediction - last_month) / last_month * 100.0).round();

        let trend = if percent_change > 5.0 {
            "up"
        } else if percent_change < -5.0 {
            "down"
        } else {
            "stable"
        };

        (trend.to_string(), percent_change)
    }

    /// Get forecast with optional breakdown by category
    pub fn forecast_with_categories(conn: &Connection) -> Result<(Forecast, Vec<CategoryForecast>), String> {
        let overall = Self::forecast_next_month(conn)?;

        // Get category-level forecasts
        let mut category_forecasts = Vec::new();

        let categories: Vec<(i64, String)> = conn
            .prepare(
                "SELECT DISTINCT c.id, c.name 
                 FROM categories c
                 JOIN transactions t ON c.id = t.category_id
                 WHERE t.type = 'expense'
                   AND t.date >= date('now', '-6 months')"
            )
            .and_then(|mut stmt| {
                stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                    .map(|rows| rows.filter_map(|r| r.ok()).collect())
            })
            .unwrap_or_default();

        for (cat_id, cat_name) in categories {
            if let Ok(forecast) = Self::forecast_category(conn, cat_id) {
                category_forecasts.push(CategoryForecast {
                    category_id: cat_id,
                    category_name: cat_name,
                    predicted_expense: forecast.round(),
                });
            }
        }

        Ok((overall, category_forecasts))
    }

    /// Forecast for a specific category
    fn forecast_category(conn: &Connection, category_id: i64) -> Result<f64, String> {
        let mut stmt = conn
            .prepare(
                "SELECT strftime('%Y-%m', date) as month, SUM(amount) as total
                 FROM transactions
                 WHERE type = 'expense'
                   AND category_id = ?1
                   AND date >= date('now', '-6 months')
                 GROUP BY month
                 ORDER BY month"
            )
            .map_err(|e| format!("Query error: {}", e))?;

        let monthly_totals: Vec<f64> = stmt
            .query_map([category_id], |row| {
                let total: f64 = row.get(1)?;
                Ok(total)
            })
            .map_err(|e| format!("Query error: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        if monthly_totals.len() < 2 {
            return Err("Not enough data".to_string());
        }

        Ok(Self::exponential_smoothing(&monthly_totals, 0.3))
    }
}

/// Category-level forecast
#[derive(Serialize, Clone)]
pub struct CategoryForecast {
    pub category_id: i64,
    pub category_name: String,
    pub predicted_expense: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exponential_smoothing() {
        let data = vec![100.0, 110.0, 120.0, 115.0, 125.0];
        let forecast = ExpenseForecaster::exponential_smoothing(&data, 0.3);
        assert!(forecast > 0.0);
        // Should be somewhere between the values
        assert!(forecast > 100.0 && forecast < 130.0);
    }

    #[test]
    fn test_empty_data() {
        let forecast = ExpenseForecaster::exponential_smoothing(&[], 0.3);
        assert_eq!(forecast, 0.0);
    }

    #[test]
    fn test_trend_calculation() {
        let data = vec![100.0, 110.0, 120.0];
        let (trend, _) = ExpenseForecaster::calculate_trend(&data, 130.0);
        assert_eq!(trend, "up");

        let (trend, _) = ExpenseForecaster::calculate_trend(&data, 100.0);
        assert_eq!(trend, "down");

        let (trend, _) = ExpenseForecaster::calculate_trend(&data, 121.0);
        assert_eq!(trend, "stable");
    }

    #[test]
    fn test_single_value() {
        let data = vec![100.0];
        let forecast = ExpenseForecaster::exponential_smoothing(&data, 0.3);
        assert_eq!(forecast, 100.0);
    }

    #[test]
    fn test_alpha_sensitivity() {
        let data = vec![100.0, 200.0];
        
        // Low alpha = more weight on history
        let low_alpha = ExpenseForecaster::exponential_smoothing(&data, 0.1);
        // High alpha = more weight on recent
        let high_alpha = ExpenseForecaster::exponential_smoothing(&data, 0.9);
        
        // With high alpha, forecast should be closer to 200
        assert!(high_alpha > low_alpha);
        assert!(high_alpha > 150.0);
        assert!(low_alpha < 150.0);
    }

    #[test]
    fn test_holt_smoothing_constant() {
        // For constant data, Holt's method should have zero trend
        let data = vec![100.0, 100.0, 100.0, 100.0];
        let result = ExpenseForecaster::holt_smoothing(&data, 0.3, 0.1);
        
        assert!((result.level - 100.0).abs() < 1.0);
        assert!(result.trend.abs() < 1.0);
    }

    #[test]
    fn test_holt_smoothing_trend() {
        // For increasing data, trend should be positive
        let data = vec![100.0, 110.0, 120.0, 130.0, 140.0];
        let result = ExpenseForecaster::holt_smoothing(&data, 0.3, 0.1);
        
        assert!(result.trend > 0.0);
    }

    #[test]
    fn test_holt_empty_data() {
        let result = ExpenseForecaster::holt_smoothing(&[], 0.3, 0.1);
        assert_eq!(result.level, 0.0);
        assert_eq!(result.trend, 0.0);
    }

    #[test]
    fn test_holt_single_value() {
        let result = ExpenseForecaster::holt_smoothing(&[100.0], 0.3, 0.1);
        assert_eq!(result.level, 100.0);
        assert_eq!(result.trend, 0.0);
    }

    #[test]
    fn test_forecast_with_holt() {
        let data = vec![100.0, 110.0, 120.0, 130.0];
        
        // Forecast 1 period ahead
        let forecast_1 = ExpenseForecaster::forecast_with_holt(&data, 1);
        // Forecast 2 periods ahead
        let forecast_2 = ExpenseForecaster::forecast_with_holt(&data, 2);
        
        // Should forecast increasing trend
        assert!(forecast_2 > forecast_1);
        assert!(forecast_1 > 130.0);
    }

    #[test]
    fn test_forecast_non_negative() {
        // Even with decreasing data, forecast should not be negative
        let data = vec![100.0, 80.0, 60.0, 40.0, 20.0];
        let forecast = ExpenseForecaster::forecast_with_holt(&data, 5);
        
        assert!(forecast >= 0.0);
    }

    #[test]
    fn test_rmse_calculation() {
        let errors = vec![1.0, -1.0, 2.0, -2.0];
        let rmse = ExpenseForecaster::calculate_rmse(&errors);
        
        // RMSE = sqrt((1+1+4+4)/4) = sqrt(2.5) ≈ 1.58
        assert!((rmse - 1.58).abs() < 0.1);
    }

    #[test]
    fn test_rmse_empty() {
        let rmse = ExpenseForecaster::calculate_rmse(&[]);
        assert_eq!(rmse, 0.0);
    }

    #[test]
    fn test_rmse_zero_errors() {
        let errors = vec![0.0, 0.0, 0.0];
        let rmse = ExpenseForecaster::calculate_rmse(&errors);
        assert_eq!(rmse, 0.0);
    }

    #[test]
    fn test_trend_empty_history() {
        let (trend, percent) = ExpenseForecaster::calculate_trend(&[], 100.0);
        assert_eq!(trend, "stable");
        assert_eq!(percent, 0.0);
    }

    #[test]
    fn test_trend_zero_last_month() {
        let data = vec![100.0, 50.0, 0.0];
        let (trend, percent) = ExpenseForecaster::calculate_trend(&data, 100.0);
        
        // When last month is 0, should be stable to avoid division by zero
        assert_eq!(trend, "stable");
        assert_eq!(percent, 0.0);
    }

    #[test]
    fn test_trend_percentage() {
        let data = vec![100.0];
        
        // 20% increase
        let (_, percent) = ExpenseForecaster::calculate_trend(&data, 120.0);
        assert!((percent - 20.0).abs() < 0.1);
        
        // 20% decrease
        let (_, percent) = ExpenseForecaster::calculate_trend(&data, 80.0);
        assert!((percent + 20.0).abs() < 0.1);
    }

    #[test]
    fn test_error_calculation() {
        let data = vec![100.0, 100.0, 100.0];
        let errors = ExpenseForecaster::calculate_errors(&data, 0.5);
        
        // For constant data, errors should be zero
        assert!(errors.iter().all(|&e| e.abs() < 0.01));
    }

    #[test]
    fn test_error_calculation_minimal_data() {
        let data = vec![100.0];
        let errors = ExpenseForecaster::calculate_errors(&data, 0.3);
        
        // Need at least 2 points to calculate errors
        assert!(errors.is_empty());
    }
}
