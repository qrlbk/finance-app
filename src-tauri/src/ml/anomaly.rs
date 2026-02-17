use rusqlite::Connection;
use serde::Serialize;

/// Detected anomaly in spending
#[derive(Serialize, Clone)]
pub struct Anomaly {
    pub message: String,
    pub severity: String, // "warning" | "alert"
    pub category: Option<String>,
    pub expected: f64,
    pub actual: f64,
}

/// Anomaly detector using Z-score method
pub struct AnomalyDetector;

impl AnomalyDetector {
    /// Detect anomalies in recent spending compared to historical patterns
    /// Uses Z-score: if spending deviates > 2 std from mean, it's anomalous
    pub fn detect_anomalies(conn: &Connection, user_id: i64, days: i32) -> Result<Vec<Anomaly>, String> {
        let mut anomalies = Vec::new();

        if let Some(anomaly) = Self::check_total_spending_anomaly(conn, user_id, days)? {
            anomalies.push(anomaly);
        }

        let category_anomalies = Self::check_category_anomalies(conn, user_id, days)?;
        anomalies.extend(category_anomalies);

        Ok(anomalies)
    }

    /// Check if total recent spending is anomalous
    fn check_total_spending_anomaly(conn: &Connection, user_id: i64, days: i32) -> Result<Option<Anomaly>, String> {
        let mut stmt = conn
            .prepare(
                "SELECT strftime('%Y-%m', date) as month, SUM(amount) as total
                 FROM transactions
                 WHERE user_id = ?1 AND type = 'expense'
                   AND date >= date('now', '-6 months')
                 GROUP BY month
                 ORDER BY month"
            )
            .map_err(|e| format!("Query error: {}", e))?;

        let monthly_totals: Vec<f64> = stmt
            .query_map([user_id], |row| {
                let total: f64 = row.get(1)?;
                Ok(total)
            })
            .map_err(|e| format!("Query error: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        if monthly_totals.len() < 3 {
            return Ok(None); // Not enough data
        }

        // Calculate mean and std (excluding current month)
        let historical = &monthly_totals[..monthly_totals.len() - 1];
        let (mean, std) = Self::calculate_stats(historical);

        if std < 1.0 {
            return Ok(None); // Too little variance
        }

        let current: f64 = conn
            .query_row(
                &format!(
                    "SELECT COALESCE(SUM(amount), 0) FROM transactions 
                     WHERE user_id = ?1 AND type = 'expense' AND date >= date('now', '-{} days')",
                    days
                ),
                [user_id],
                |row| row.get(0),
            )
            .unwrap_or(0.0);

        // Normalize to monthly rate
        let current_monthly = current * (30.0 / days as f64);
        let z_score = (current_monthly - mean) / std;

        if z_score > 2.0 {
            let percent_over = ((current_monthly - mean) / mean * 100.0).round();
            Ok(Some(Anomaly {
                message: format!(
                    "Общие расходы за последние {} дней на {}% выше среднего",
                    days, percent_over
                ),
                severity: if z_score > 3.0 { "alert".to_string() } else { "warning".to_string() },
                category: None,
                expected: mean,
                actual: current_monthly,
            }))
        } else {
            Ok(None)
        }
    }

    /// Check for anomalies in specific categories
    fn check_category_anomalies(conn: &Connection, user_id: i64, days: i32) -> Result<Vec<Anomaly>, String> {
        let mut anomalies = Vec::new();

        let mut stmt = conn
            .prepare(
                "SELECT c.id, c.name, 
                        AVG(monthly.total) as avg_monthly,
                        COUNT(monthly.total) as months_count
                 FROM categories c
                 JOIN (
                     SELECT category_id, 
                            strftime('%Y-%m', date) as month,
                            SUM(amount) as total
                     FROM transactions
                     WHERE user_id = ?1 AND type = 'expense'
                       AND date >= date('now', '-6 months')
                       AND category_id IS NOT NULL
                     GROUP BY category_id, month
                 ) monthly ON c.id = monthly.category_id
                 WHERE c.user_id = ?1
                 GROUP BY c.id
                 HAVING months_count >= 3"
            )
            .map_err(|e| format!("Query error: {}", e))?;

        struct CategoryStats {
            id: i64,
            name: String,
            avg_monthly: f64,
        }

        let categories: Vec<CategoryStats> = stmt
            .query_map([user_id], |row| {
                Ok(CategoryStats {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    avg_monthly: row.get(2)?,
                })
            })
            .map_err(|e| format!("Query error: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        // Check each category for anomalies
        for cat in categories {
            // Get historical monthly values for this category
            let mut hist_stmt = conn
                .prepare(
                    "SELECT SUM(amount) as total
                     FROM transactions
                     WHERE user_id = ?1 AND type = 'expense'
                       AND category_id = ?2
                       AND date >= date('now', '-6 months')
                       AND date < date('now', 'start of month')
                     GROUP BY strftime('%Y-%m', date)"
                )
                .map_err(|e| format!("Query error: {}", e))?;

            let historical: Vec<f64> = hist_stmt
                .query_map(rusqlite::params![user_id, cat.id], |row| row.get(0))
                .map_err(|e| format!("Query error: {}", e))?
                .filter_map(|r| r.ok())
                .collect();

            if historical.len() < 3 {
                continue;
            }

            let (mean, std) = Self::calculate_stats(&historical);
            if std < 1.0 {
                continue;
            }

            // Get current period spending for this category
            let current: f64 = conn
                .query_row(
                    &format!(
                        "SELECT COALESCE(SUM(amount), 0) FROM transactions 
                         WHERE user_id = ?1 AND type = 'expense' 
                           AND category_id = ?2 
                           AND date >= date('now', '-{} days')",
                        days
                    ),
                    rusqlite::params![user_id, cat.id],
                    |row| row.get(0),
                )
                .unwrap_or(0.0);

            let current_monthly = current * (30.0 / days as f64);
            let z_score = (current_monthly - mean) / std;

            if z_score > 2.0 {
                let percent_over = ((current_monthly - mean) / mean * 100.0).round();
                anomalies.push(Anomaly {
                    message: format!(
                        "Категория \"{}\" превысила обычный уровень на {}%",
                        cat.name, percent_over
                    ),
                    severity: if z_score > 3.0 { "alert".to_string() } else { "warning".to_string() },
                    category: Some(cat.name),
                    expected: mean,
                    actual: current_monthly,
                });
            }
        }

        Ok(anomalies)
    }

    /// Check if a single transaction amount is anomalous for its category
    pub fn is_anomaly(conn: &Connection, amount: f64, category_id: Option<i64>) -> bool {
        let category_id = match category_id {
            Some(id) => id,
            None => return false,
        };

        // Get historical amounts for this category
        let amounts: Vec<f64> = conn
            .prepare(
                "SELECT amount FROM transactions 
                 WHERE type = 'expense' 
                   AND category_id = ?1 
                   AND date >= date('now', '-3 months')"
            )
            .ok()
            .and_then(|mut stmt| {
                stmt.query_map([category_id], |row| row.get(0))
                    .ok()
                    .map(|rows| rows.filter_map(|r| r.ok()).collect())
            })
            .unwrap_or_default();

        if amounts.len() < 5 {
            return false;
        }

        let (mean, std) = Self::calculate_stats(&amounts);
        if std < 1.0 {
            return false;
        }

        let z_score = (amount - mean).abs() / std;
        z_score > 3.0
    }

    /// Calculate mean and standard deviation
    fn calculate_stats(values: &[f64]) -> (f64, f64) {
        if values.is_empty() {
            return (0.0, 0.0);
        }

        let n = values.len() as f64;
        let mean = values.iter().sum::<f64>() / n;
        
        let variance = values.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / n;
        
        let std = variance.sqrt();
        
        (mean, std)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_stats() {
        let values = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        let (mean, std) = AnomalyDetector::calculate_stats(&values);
        assert!((mean - 30.0).abs() < 0.01);
        assert!(std > 0.0);
    }

    #[test]
    fn test_empty_stats() {
        let (mean, std) = AnomalyDetector::calculate_stats(&[]);
        assert_eq!(mean, 0.0);
        assert_eq!(std, 0.0);
    }

    #[test]
    fn test_single_value_stats() {
        let values = vec![100.0];
        let (mean, std) = AnomalyDetector::calculate_stats(&values);
        assert_eq!(mean, 100.0);
        assert_eq!(std, 0.0); // Single value has no variance
    }

    #[test]
    fn test_identical_values_stats() {
        let values = vec![50.0, 50.0, 50.0, 50.0];
        let (mean, std) = AnomalyDetector::calculate_stats(&values);
        assert_eq!(mean, 50.0);
        assert_eq!(std, 0.0); // Identical values have no variance
    }

    #[test]
    fn test_known_std_values() {
        // For values 2, 4, 4, 4, 5, 5, 7, 9 (classic example)
        // Mean = 5, Variance = 4, Std = 2
        let values = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let (mean, std) = AnomalyDetector::calculate_stats(&values);
        assert!((mean - 5.0).abs() < 0.01);
        assert!((std - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_negative_values() {
        let values = vec![-10.0, -5.0, 0.0, 5.0, 10.0];
        let (mean, std) = AnomalyDetector::calculate_stats(&values);
        assert_eq!(mean, 0.0);
        assert!(std > 0.0);
    }

    #[test]
    fn test_large_values() {
        let values = vec![1000000.0, 1000100.0, 1000050.0];
        let (mean, std) = AnomalyDetector::calculate_stats(&values);
        assert!((mean - 1000050.0).abs() < 1.0);
        assert!(std > 0.0);
    }

    #[test]
    fn test_z_score_calculation() {
        // If mean=100, std=10, value=130 -> z_score = 3.0
        let values = vec![90.0, 95.0, 100.0, 105.0, 110.0]; // mean≈100
        let (mean, std) = AnomalyDetector::calculate_stats(&values);
        
        // Verify mean
        assert!((mean - 100.0).abs() < 0.1);
        
        // For an outlier that's 3 std devs away:
        let test_value = mean + 3.0 * std;
        let z_score = (test_value - mean).abs() / std;
        assert!((z_score - 3.0).abs() < 0.01);
    }
}
