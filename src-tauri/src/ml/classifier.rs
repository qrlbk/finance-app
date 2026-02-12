use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Multinomial Naive Bayes Classifier with Laplace smoothing
#[derive(Clone, Serialize, Deserialize)]
pub struct NaiveBayesClassifier {
    /// P(category) - prior probabilities (log scale)
    class_log_priors: HashMap<i64, f64>,
    /// P(feature|category) - conditional probabilities (log scale)
    feature_log_probs: HashMap<i64, Vec<f64>>,
    /// Laplace smoothing parameter
    alpha: f64,
    /// Number of features
    n_features: usize,
    /// Classes (category IDs)
    classes: Vec<i64>,
}

impl Default for NaiveBayesClassifier {
    fn default() -> Self {
        Self::new(1.0)
    }
}

impl NaiveBayesClassifier {
    pub fn new(alpha: f64) -> Self {
        NaiveBayesClassifier {
            class_log_priors: HashMap::new(),
            feature_log_probs: HashMap::new(),
            alpha,
            n_features: 0,
            classes: Vec::new(),
        }
    }

    /// Fit the classifier on TF-IDF features and labels
    /// X: TF-IDF vectors, y: category IDs
    pub fn fit(&mut self, x: &[Vec<f64>], y: &[i64]) {
        if x.is_empty() || y.is_empty() || x.len() != y.len() {
            return;
        }

        self.n_features = x[0].len();
        let n_samples = x.len() as f64;

        // Count samples per class
        let mut class_counts: HashMap<i64, usize> = HashMap::new();
        for &label in y {
            *class_counts.entry(label).or_insert(0) += 1;
        }

        self.classes = class_counts.keys().cloned().collect();
        self.classes.sort();

        // Calculate prior probabilities (log scale)
        self.class_log_priors.clear();
        for (&class, &count) in &class_counts {
            self.class_log_priors.insert(class, (count as f64 / n_samples).ln());
        }

        // Calculate feature probabilities per class (log scale)
        self.feature_log_probs.clear();
        
        for &class in &self.classes {
            // Get all samples for this class
            let class_samples: Vec<&Vec<f64>> = x.iter()
                .zip(y.iter())
                .filter(|(_, &label)| label == class)
                .map(|(sample, _)| sample)
                .collect();

            // Sum features for this class
            let mut feature_sums = vec![0.0; self.n_features];
            for sample in &class_samples {
                for (i, &val) in sample.iter().enumerate() {
                    feature_sums[i] += val;
                }
            }

            // Apply Laplace smoothing and convert to log probabilities
            let total_sum: f64 = feature_sums.iter().sum::<f64>() + self.alpha * self.n_features as f64;
            let log_probs: Vec<f64> = feature_sums
                .iter()
                .map(|&sum| ((sum + self.alpha) / total_sum).ln())
                .collect();

            self.feature_log_probs.insert(class, log_probs);
        }
    }

    /// Predict class for a single TF-IDF vector
    /// Returns (category_id, confidence) or None if not fitted
    pub fn predict(&self, x: &[f64]) -> Option<(i64, f64)> {
        if self.classes.is_empty() || x.len() != self.n_features {
            return None;
        }

        let log_probs = self.predict_log_proba(x);
        
        // Find class with highest probability
        log_probs
            .into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(class, log_prob)| {
                // Convert log probability to confidence (0-1 scale)
                // Using softmax-like normalization
                (class, self.log_prob_to_confidence(log_prob))
            })
    }

    /// Get probability distribution over all classes
    pub fn predict_proba(&self, x: &[f64]) -> HashMap<i64, f64> {
        let log_probs = self.predict_log_proba(x);
        
        // Convert log probs to probabilities using softmax
        let max_log: f64 = log_probs.values().cloned().fold(f64::NEG_INFINITY, f64::max);
        let exp_sum: f64 = log_probs.values().map(|&lp| (lp - max_log).exp()).sum();
        
        log_probs
            .into_iter()
            .map(|(class, lp)| (class, (lp - max_log).exp() / exp_sum))
            .collect()
    }

    /// Get log probabilities for all classes
    fn predict_log_proba(&self, x: &[f64]) -> HashMap<i64, f64> {
        let mut log_probs: HashMap<i64, f64> = HashMap::new();

        for &class in &self.classes {
            let prior = self.class_log_priors.get(&class).copied().unwrap_or(f64::NEG_INFINITY);
            let feature_probs = self.feature_log_probs.get(&class);

            let likelihood: f64 = if let Some(probs) = feature_probs {
                x.iter()
                    .zip(probs.iter())
                    .map(|(&xi, &log_p)| xi * log_p)
                    .sum()
            } else {
                0.0
            };

            log_probs.insert(class, prior + likelihood);
        }

        log_probs
    }

    /// Convert log probability to a confidence score (0-1)
    fn log_prob_to_confidence(&self, log_prob: f64) -> f64 {
        // Get all log probs and apply softmax
        let all_log_probs: Vec<f64> = self.class_log_priors.values().cloned().collect();
        if all_log_probs.is_empty() {
            return 0.0;
        }
        
        let max_log: f64 = all_log_probs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let exp_sum: f64 = all_log_probs.iter().map(|&lp| (lp - max_log).exp()).sum();
        
        (log_prob - max_log).exp() / exp_sum
    }

    /// Get classes
    pub fn classes(&self) -> &[i64] {
        &self.classes
    }

    /// Get class priors
    pub fn class_log_priors(&self) -> &HashMap<i64, f64> {
        &self.class_log_priors
    }

    /// Get feature log probs
    pub fn feature_log_probs(&self) -> &HashMap<i64, Vec<f64>> {
        &self.feature_log_probs
    }

    /// Create from saved state
    pub fn from_saved(
        class_log_priors: HashMap<i64, f64>,
        feature_log_probs: HashMap<i64, Vec<f64>>,
        alpha: f64,
    ) -> Self {
        let classes: Vec<i64> = class_log_priors.keys().cloned().collect();
        let n_features = feature_log_probs.values().next().map(|v| v.len()).unwrap_or(0);
        
        NaiveBayesClassifier {
            class_log_priors,
            feature_log_probs,
            alpha,
            n_features,
            classes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fit_predict() {
        let x = vec![
            vec![0.5, 0.5, 0.0],
            vec![0.6, 0.4, 0.0],
            vec![0.0, 0.0, 1.0],
            vec![0.0, 0.1, 0.9],
        ];
        let y = vec![1, 1, 2, 2];

        let mut clf = NaiveBayesClassifier::new(1.0);
        clf.fit(&x, &y);

        // Test prediction for food-like vector
        let pred = clf.predict(&[0.5, 0.5, 0.0]);
        assert!(pred.is_some());
        let (class, _conf) = pred.unwrap();
        assert_eq!(class, 1);
    }

    #[test]
    fn test_predict_proba() {
        let x = vec![
            vec![1.0, 0.0],
            vec![0.0, 1.0],
        ];
        let y = vec![1, 2];

        let mut clf = NaiveBayesClassifier::new(1.0);
        clf.fit(&x, &y);

        let proba = clf.predict_proba(&[1.0, 0.0]);
        assert!(proba.get(&1).unwrap() > proba.get(&2).unwrap());
    }

    #[test]
    fn test_empty_data() {
        let mut clf = NaiveBayesClassifier::new(1.0);
        clf.fit(&[], &[]);
        
        assert!(clf.classes().is_empty());
        assert!(clf.predict(&[0.5]).is_none());
    }

    #[test]
    fn test_mismatched_lengths() {
        let x = vec![vec![1.0, 0.0]];
        let y = vec![1, 2]; // More labels than samples

        let mut clf = NaiveBayesClassifier::new(1.0);
        clf.fit(&x, &y);
        
        // Should handle gracefully
        assert!(clf.classes().is_empty());
    }

    #[test]
    fn test_wrong_feature_count() {
        let x = vec![
            vec![1.0, 0.0],
            vec![0.0, 1.0],
        ];
        let y = vec![1, 2];

        let mut clf = NaiveBayesClassifier::new(1.0);
        clf.fit(&x, &y);

        // Wrong number of features
        let pred = clf.predict(&[0.5, 0.5, 0.5]);
        assert!(pred.is_none());
    }

    #[test]
    fn test_laplace_smoothing() {
        // Test with different alpha values
        let x = vec![
            vec![1.0, 0.0],
            vec![0.0, 1.0],
        ];
        let y = vec![1, 2];

        let mut clf_low_alpha = NaiveBayesClassifier::new(0.1);
        let mut clf_high_alpha = NaiveBayesClassifier::new(10.0);
        
        clf_low_alpha.fit(&x, &y);
        clf_high_alpha.fit(&x, &y);

        // Both should work, high alpha makes predictions more uniform
        assert!(clf_low_alpha.predict(&[1.0, 0.0]).is_some());
        assert!(clf_high_alpha.predict(&[1.0, 0.0]).is_some());
    }

    #[test]
    fn test_multiple_classes() {
        let x = vec![
            vec![1.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0],
            vec![0.0, 0.0, 1.0],
        ];
        let y = vec![1, 2, 3];

        let mut clf = NaiveBayesClassifier::new(1.0);
        clf.fit(&x, &y);

        assert_eq!(clf.classes().len(), 3);
        
        let proba = clf.predict_proba(&[1.0, 0.0, 0.0]);
        assert!(proba.contains_key(&1));
        assert!(proba.contains_key(&2));
        assert!(proba.contains_key(&3));
    }

    #[test]
    fn test_probabilities_sum_to_one() {
        let x = vec![
            vec![1.0, 0.0],
            vec![0.0, 1.0],
            vec![0.5, 0.5],
        ];
        let y = vec![1, 2, 1];

        let mut clf = NaiveBayesClassifier::new(1.0);
        clf.fit(&x, &y);

        let proba = clf.predict_proba(&[0.3, 0.7]);
        let sum: f64 = proba.values().sum();
        
        // Probabilities should sum to approximately 1
        assert!((sum - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_confidence_range() {
        let x = vec![
            vec![1.0, 0.0],
            vec![0.0, 1.0],
        ];
        let y = vec![1, 2];

        let mut clf = NaiveBayesClassifier::new(1.0);
        clf.fit(&x, &y);

        let (_, conf) = clf.predict(&[0.8, 0.2]).unwrap();
        
        // Confidence should be between 0 and 1
        assert!(conf >= 0.0 && conf <= 1.0);
    }

    #[test]
    fn test_from_saved() {
        let mut class_log_priors = HashMap::new();
        class_log_priors.insert(1, -0.693); // ln(0.5)
        class_log_priors.insert(2, -0.693);
        
        let mut feature_log_probs = HashMap::new();
        feature_log_probs.insert(1, vec![-0.5, -1.0]);
        feature_log_probs.insert(2, vec![-1.0, -0.5]);

        let clf = NaiveBayesClassifier::from_saved(class_log_priors, feature_log_probs, 1.0);
        
        assert_eq!(clf.classes().len(), 2);
        assert!(clf.predict(&[0.5, 0.5]).is_some());
    }

    #[test]
    fn test_consistent_predictions() {
        let x = vec![
            vec![1.0, 0.0],
            vec![0.0, 1.0],
        ];
        let y = vec![1, 2];

        let mut clf = NaiveBayesClassifier::new(1.0);
        clf.fit(&x, &y);

        // Same input should give same output
        let test_vec = vec![0.7, 0.3];
        let pred1 = clf.predict(&test_vec);
        let pred2 = clf.predict(&test_vec);
        
        assert_eq!(pred1.unwrap().0, pred2.unwrap().0);
    }
}
