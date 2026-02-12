//! Feature engineering module for ML model
//! Extracts additional features from transactions beyond text

use chrono::{NaiveDate, Datelike, Weekday};

/// Amount buckets for categorization
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AmountBucket {
    VerySmall,  // 0-1000
    Small,      // 1000-5000
    Medium,     // 5000-20000
    Large,      // 20000-100000
    VeryLarge,  // 100000+
}

impl AmountBucket {
    pub fn from_amount(amount: f64) -> Self {
        let abs_amount = amount.abs();
        if abs_amount < 1000.0 {
            AmountBucket::VerySmall
        } else if abs_amount < 5000.0 {
            AmountBucket::Small
        } else if abs_amount < 20000.0 {
            AmountBucket::Medium
        } else if abs_amount < 100000.0 {
            AmountBucket::Large
        } else {
            AmountBucket::VeryLarge
        }
    }

    pub fn as_index(&self) -> usize {
        match self {
            AmountBucket::VerySmall => 0,
            AmountBucket::Small => 1,
            AmountBucket::Medium => 2,
            AmountBucket::Large => 3,
            AmountBucket::VeryLarge => 4,
        }
    }

    pub fn count() -> usize {
        5
    }
}

/// Time-based features
#[derive(Debug, Clone)]
pub struct TimeFeatures {
    pub day_of_week: usize,    // 0-6 (Mon-Sun)
    pub day_of_month: usize,   // 1-31
    pub is_weekend: bool,
    pub is_month_start: bool,  // 1-5
    pub is_month_end: bool,    // 26-31
}

impl TimeFeatures {
    pub fn from_date(date_str: &str) -> Option<Self> {
        let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok()?;
        let day_of_week = date.weekday().num_days_from_monday() as usize;
        let day_of_month = date.day() as usize;
        
        Some(TimeFeatures {
            day_of_week,
            day_of_month,
            is_weekend: matches!(date.weekday(), Weekday::Sat | Weekday::Sun),
            is_month_start: day_of_month <= 5,
            is_month_end: day_of_month >= 26,
        })
    }

    /// Convert to feature vector
    pub fn to_features(&self) -> Vec<f64> {
        let mut features = vec![0.0; Self::feature_count()];
        
        // One-hot encode day of week (7 features)
        features[self.day_of_week] = 1.0;
        
        // Is weekend (1 feature)
        features[7] = if self.is_weekend { 1.0 } else { 0.0 };
        
        // Is month start (1 feature)
        features[8] = if self.is_month_start { 1.0 } else { 0.0 };
        
        // Is month end (1 feature)
        features[9] = if self.is_month_end { 1.0 } else { 0.0 };
        
        features
    }

    pub fn feature_count() -> usize {
        10 // 7 day_of_week + 1 is_weekend + 1 is_month_start + 1 is_month_end
    }
}

/// Combined transaction features
#[derive(Debug, Clone)]
pub struct TransactionFeatures {
    pub tfidf_vector: Vec<f64>,
    pub amount_bucket: AmountBucket,
    pub time_features: Option<TimeFeatures>,
}

impl TransactionFeatures {
    pub fn new(tfidf_vector: Vec<f64>, amount: f64, date: Option<&str>) -> Self {
        TransactionFeatures {
            tfidf_vector,
            amount_bucket: AmountBucket::from_amount(amount),
            time_features: date.and_then(TimeFeatures::from_date),
        }
    }

    /// Convert to a combined feature vector
    /// Format: [tfidf...] [amount_bucket_one_hot] [time_features]
    pub fn to_combined_vector(&self) -> Vec<f64> {
        let mut combined = self.tfidf_vector.clone();
        
        // Add amount bucket (one-hot encoded)
        let mut amount_features = vec![0.0; AmountBucket::count()];
        amount_features[self.amount_bucket.as_index()] = 1.0;
        combined.extend(amount_features);
        
        // Add time features if available
        if let Some(ref time) = self.time_features {
            combined.extend(time.to_features());
        } else {
            // Pad with zeros if time features not available
            combined.extend(vec![0.0; TimeFeatures::feature_count()]);
        }
        
        combined
    }

    /// Get the number of additional features (beyond TF-IDF)
    pub fn additional_feature_count() -> usize {
        AmountBucket::count() + TimeFeatures::feature_count()
    }
}

/// Helper function to extract features from transaction data
pub fn extract_features(
    tfidf_vector: Vec<f64>,
    amount: f64,
    date: &str,
) -> TransactionFeatures {
    TransactionFeatures::new(tfidf_vector, amount, Some(date))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amount_bucket() {
        assert_eq!(AmountBucket::from_amount(500.0), AmountBucket::VerySmall);
        assert_eq!(AmountBucket::from_amount(2000.0), AmountBucket::Small);
        assert_eq!(AmountBucket::from_amount(10000.0), AmountBucket::Medium);
        assert_eq!(AmountBucket::from_amount(50000.0), AmountBucket::Large);
        assert_eq!(AmountBucket::from_amount(200000.0), AmountBucket::VeryLarge);
    }

    #[test]
    fn test_time_features() {
        let features = TimeFeatures::from_date("2024-01-15").unwrap();
        assert!(!features.is_weekend);
        assert!(!features.is_month_start);
        assert!(!features.is_month_end);
        
        let weekend = TimeFeatures::from_date("2024-01-13").unwrap(); // Saturday
        assert!(weekend.is_weekend);
        
        let month_start = TimeFeatures::from_date("2024-01-03").unwrap();
        assert!(month_start.is_month_start);
        
        let month_end = TimeFeatures::from_date("2024-01-30").unwrap();
        assert!(month_end.is_month_end);
    }

    #[test]
    fn test_combined_features() {
        let tfidf = vec![0.5, 0.3, 0.2];
        let features = TransactionFeatures::new(tfidf.clone(), 3000.0, Some("2024-01-15"));
        
        let combined = features.to_combined_vector();
        
        // Should have tfidf + amount bucket + time features
        let expected_len = tfidf.len() + AmountBucket::count() + TimeFeatures::feature_count();
        assert_eq!(combined.len(), expected_len);
    }
}
