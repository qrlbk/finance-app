use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};

use super::tfidf::TfIdfVectorizer;
use super::classifier::NaiveBayesClassifier;
use super::tokenizer::Tokenizer;
use super::features::TransactionFeatures;

/// Serializable category prediction model
#[derive(Serialize, Deserialize)]
pub struct CategoryModel {
    /// Model version for compatibility
    pub version: u32,
    /// TF-IDF vocabulary (word -> index)
    pub vocabulary: HashMap<String, usize>,
    /// TF-IDF IDF weights
    pub idf_weights: Vec<f64>,
    /// Naive Bayes class log priors
    pub class_log_priors: HashMap<i64, f64>,
    /// Naive Bayes feature log probabilities per class
    pub feature_log_probs: HashMap<i64, Vec<f64>>,
    /// Category ID to name mapping
    pub category_names: HashMap<i64, String>,
    /// Training timestamp
    pub trained_at: String,
    /// Number of samples used for training
    pub sample_count: usize,
    /// Model accuracy (from cross-validation)
    pub accuracy: Option<f64>,
    /// Laplace smoothing parameter
    pub alpha: f64,
    /// Whether model uses enhanced features (n-grams, amount, time)
    #[serde(default)]
    pub use_enhanced_features: bool,
}

impl CategoryModel {
    /// Current model version
    pub const CURRENT_VERSION: u32 = 2;

    /// Create a new model from trained components
    pub fn new(
        vectorizer: &TfIdfVectorizer,
        classifier: &NaiveBayesClassifier,
        category_names: HashMap<i64, String>,
        sample_count: usize,
        accuracy: Option<f64>,
    ) -> Self {
        Self::new_enhanced(vectorizer, classifier, category_names, sample_count, accuracy, false)
    }

    /// Create a new model with enhanced features
    pub fn new_enhanced(
        vectorizer: &TfIdfVectorizer,
        classifier: &NaiveBayesClassifier,
        category_names: HashMap<i64, String>,
        sample_count: usize,
        accuracy: Option<f64>,
        use_enhanced_features: bool,
    ) -> Self {
        CategoryModel {
            version: Self::CURRENT_VERSION,
            vocabulary: vectorizer.vocabulary().clone(),
            idf_weights: vectorizer.idf_weights().clone(),
            class_log_priors: classifier.class_log_priors().clone(),
            feature_log_probs: classifier.feature_log_probs().clone(),
            category_names,
            trained_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            sample_count,
            accuracy,
            alpha: 1.0,
            use_enhanced_features,
        }
    }

    /// Save model to JSON file
    pub fn save(&self, path: &Path) -> Result<(), String> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize model: {}", e))?;

        fs::write(path, json)
            .map_err(|e| format!("Failed to write model file: {}", e))?;

        Ok(())
    }

    /// Load model from JSON file
    pub fn load(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Err("Model file not found".to_string());
        }

        let json = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read model file: {}", e))?;

        let model: CategoryModel = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse model: {}", e))?;

        // Version check
        if model.version > Self::CURRENT_VERSION {
            return Err(format!(
                "Model version {} is newer than supported version {}",
                model.version, Self::CURRENT_VERSION
            ));
        }

        Ok(model)
    }

    /// Predict category for text (basic prediction)
    pub fn predict(&self, text: &str) -> Option<(i64, String, f64)> {
        self.predict_with_context(text, None, None)
    }

    /// Predict category with additional context (amount, date)
    pub fn predict_with_context(
        &self,
        text: &str,
        amount: Option<f64>,
        date: Option<&str>,
    ) -> Option<(i64, String, f64)> {
        let tokenizer = Tokenizer::new();
        let vectorizer = TfIdfVectorizer::from_saved(
            self.vocabulary.clone(),
            self.idf_weights.clone(),
        );
        let classifier = NaiveBayesClassifier::from_saved(
            self.class_log_priors.clone(),
            self.feature_log_probs.clone(),
            self.alpha,
        );

        // Use n-grams if enhanced features are enabled
        let tokens = if self.use_enhanced_features {
            tokenizer.tokenize_with_ngrams(text)
        } else {
            tokenizer.tokenize(text)
        };
        
        if tokens.is_empty() {
            return None;
        }

        let tfidf_features = vectorizer.transform(&tokens);
        if tfidf_features.is_empty() {
            return None;
        }

        // Build final feature vector
        let features = if self.use_enhanced_features && (amount.is_some() || date.is_some()) {
            let tx_features = TransactionFeatures::new(
                tfidf_features,
                amount.unwrap_or(0.0),
                date,
            );
            tx_features.to_combined_vector()
        } else {
            tfidf_features
        };

        let (category_id, confidence) = classifier.predict(&features)?;
        let category_name = self.category_names.get(&category_id)
            .cloned()
            .unwrap_or_else(|| "Unknown".to_string());

        Some((category_id, category_name, confidence))
    }

    /// Get model status info
    pub fn status(&self) -> ModelStatus {
        ModelStatus {
            trained: true,
            trained_at: Some(self.trained_at.clone()),
            sample_count: Some(self.sample_count),
            accuracy: self.accuracy,
        }
    }
}

/// Model status information
#[derive(Serialize, Deserialize, Clone)]
pub struct ModelStatus {
    pub trained: bool,
    pub trained_at: Option<String>,
    pub sample_count: Option<usize>,
    pub accuracy: Option<f64>,
}

impl Default for ModelStatus {
    fn default() -> Self {
        ModelStatus {
            trained: false,
            trained_at: None,
            sample_count: None,
            accuracy: None,
        }
    }
}

/// Get the default model file path
pub fn get_model_path() -> Result<std::path::PathBuf, String> {
    let dirs = directories::ProjectDirs::from("", "", "finance-app")
        .ok_or("Could not determine data directory")?;
    
    let data_dir = dirs.data_dir();
    Ok(data_dir.join("ml_model.json"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    #[test]
    fn test_save_load_model() {
        let mut vectorizer = TfIdfVectorizer::new();
        vectorizer.fit(&[vec!["test".to_string()]]);

        let mut classifier = NaiveBayesClassifier::new(1.0);
        classifier.fit(&[vec![1.0]], &[1]);

        let mut category_names = HashMap::new();
        category_names.insert(1, "Test".to_string());

        let model = CategoryModel::new(&vectorizer, &classifier, category_names, 1, Some(0.95));

        let path = temp_dir().join("test_model.json");
        model.save(&path).unwrap();

        let loaded = CategoryModel::load(&path).unwrap();
        assert_eq!(loaded.version, CategoryModel::CURRENT_VERSION);
        assert_eq!(loaded.sample_count, 1);

        // Cleanup
        let _ = fs::remove_file(&path);
    }
}
