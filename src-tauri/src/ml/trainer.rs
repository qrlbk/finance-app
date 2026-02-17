use std::collections::{HashMap, HashSet};
use rusqlite::Connection;

use super::tokenizer::Tokenizer;
use super::tfidf::TfIdfVectorizer;
use super::classifier::NaiveBayesClassifier;
use super::model::CategoryModel;
use super::features::{TransactionFeatures, AmountBucket, TimeFeatures};

/// Training data from database
struct TrainingData {
    notes: Vec<String>,
    amounts: Vec<f64>,
    dates: Vec<String>,
    category_ids: Vec<i64>,
    category_names: HashMap<i64, String>,
}

/// Result of model training
#[derive(serde::Serialize)]
pub struct TrainResult {
    pub success: bool,
    pub sample_count: usize,
    pub accuracy: Option<f64>,
    pub message: String,
}

/// Model trainer that builds a category prediction model from database
pub struct ModelTrainer {
    tokenizer: Tokenizer,
    vectorizer: TfIdfVectorizer,
    classifier: NaiveBayesClassifier,
}

impl Default for ModelTrainer {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelTrainer {
    pub fn new() -> Self {
        ModelTrainer {
            tokenizer: Tokenizer::new(),
            vectorizer: TfIdfVectorizer::new(),
            classifier: NaiveBayesClassifier::new(1.0),
        }
    }

    /// Train model from database transactions
    pub fn train_from_db(conn: &Connection) -> Result<CategoryModel, String> {
        let mut trainer = ModelTrainer::new();
        
        // Load training data
        let data = trainer.load_training_data(conn)?;
        
        if data.notes.len() < 20 {
            let with_note_no_cat: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM transactions WHERE (note IS NOT NULL AND note != '') AND category_id IS NULL",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            let hint = if with_note_no_cat > 0 {
                format!(
                    " У вас {} транзакций с описанием без категории — назначьте категории хотя бы 20 из них в разделе «Транзакции», затем обучите модель снова.",
                    with_note_no_cat
                )
            } else {
                String::new()
            };
            return Err(format!(
                "Недостаточно данных для обучения. Нужно минимум 20 транзакций с заполненными категориями и заметками, сейчас: {}.{}",
                data.notes.len(),
                hint
            ));
        }

        // Tokenize all notes with n-grams for better context
        let tokenized: Vec<Vec<String>> = data.notes
            .iter()
            .map(|note| trainer.tokenizer.tokenize_with_ngrams(note))
            .collect();

        // Collect valid data with indices for feature extraction
        let mut valid_tokens = Vec::new();
        let mut valid_labels = Vec::new();
        let mut valid_indices = Vec::new();

        for (i, (tokens, &label)) in tokenized.iter().zip(data.category_ids.iter()).enumerate() {
            if !tokens.is_empty() {
                valid_tokens.push(tokens.clone());
                valid_labels.push(label);
                valid_indices.push(i);
            }
        }

        // Filter out classes with too few samples (min 3 per class)
        const MIN_SAMPLES_PER_CLASS: usize = 3;
        let mut class_counts: HashMap<i64, usize> = HashMap::new();
        for &label in &valid_labels {
            *class_counts.entry(label).or_insert(0) += 1;
        }
        let kept_classes: HashSet<i64> = class_counts
            .iter()
            .filter(|(_, &c)| c >= MIN_SAMPLES_PER_CLASS)
            .map(|(&k, _)| k)
            .collect();
        let mut valid_tokens_filtered = Vec::new();
        let mut valid_labels_filtered = Vec::new();
        let mut valid_indices_filtered = Vec::new();
        for i in 0..valid_tokens.len() {
            let label = valid_labels[i];
            if kept_classes.contains(&label) {
                valid_tokens_filtered.push(valid_tokens[i].clone());
                valid_labels_filtered.push(label);
                valid_indices_filtered.push(valid_indices[i]);
            }
        }
        let valid_tokens = valid_tokens_filtered;
        let valid_labels = valid_labels_filtered;
        let valid_indices = valid_indices_filtered;

        let mut category_names = data.category_names.clone();
        category_names.retain(|k, _| kept_classes.contains(k));

        if valid_tokens.len() < 20 {
            return Err(format!(
                "После токенизации осталось недостаточно данных: {}. Попробуйте добавить больше описательных заметок к транзакциям.",
                valid_tokens.len()
            ));
        }

        // Fit TF-IDF vectorizer
        trainer.vectorizer.fit(&valid_tokens);

        // Transform to combined features (TF-IDF + amount + time)
        let features: Vec<Vec<f64>> = valid_indices
            .iter()
            .zip(valid_tokens.iter())
            .map(|(&idx, tokens)| {
                let tfidf = trainer.vectorizer.transform(tokens);
                let tx_features = TransactionFeatures::new(
                    tfidf,
                    data.amounts[idx],
                    Some(&data.dates[idx]),
                );
                tx_features.to_combined_vector()
            })
            .collect();

        // Fit classifier
        trainer.classifier.fit(&features, &valid_labels);

        // Calculate accuracy using simple cross-validation
        let accuracy = trainer.evaluate_accuracy(&features, &valid_labels);

        // Create and return model with enhanced features flag (category_names filtered to kept classes)
        let model = CategoryModel::new_enhanced(
            &trainer.vectorizer,
            &trainer.classifier,
            category_names,
            valid_tokens.len(),
            Some(accuracy),
            true, // use_enhanced_features
        );

        Ok(model)
    }

    /// Load training data from database
    fn load_training_data(&self, conn: &Connection) -> Result<TrainingData, String> {
        // Get transactions with categories, notes, amounts, and dates
        let mut stmt = conn
            .prepare(
                "SELECT t.note, t.category_id, c.name, t.amount, t.date 
                 FROM transactions t 
                 JOIN categories c ON t.category_id = c.id 
                 WHERE t.category_id IS NOT NULL 
                   AND t.note IS NOT NULL 
                   AND t.note != ''"
            )
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let mut notes = Vec::new();
        let mut amounts = Vec::new();
        let mut dates = Vec::new();
        let mut category_ids = Vec::new();
        let mut category_names: HashMap<i64, String> = HashMap::new();

        let rows = stmt
            .query_map([], |row| {
                let note: String = row.get(0)?;
                let category_id: i64 = row.get(1)?;
                let category_name: String = row.get(2)?;
                let amount: f64 = row.get(3)?;
                let date: String = row.get(4)?;
                Ok((note, category_id, category_name, amount, date))
            })
            .map_err(|e| format!("Failed to execute query: {}", e))?;

        for row in rows {
            let (note, category_id, category_name, amount, date) = row
                .map_err(|e| format!("Failed to read row: {}", e))?;
            
            notes.push(note);
            amounts.push(amount);
            dates.push(date);
            category_ids.push(category_id);
            category_names.insert(category_id, category_name);
        }

        Ok(TrainingData {
            notes,
            amounts,
            dates,
            category_ids,
            category_names,
        })
    }

    /// Evaluate model accuracy using k-fold cross-validation
    fn evaluate_accuracy(&self, features: &[Vec<f64>], labels: &[i64]) -> f64 {
        if features.len() < 5 {
            return 0.0;
        }

        let k = 5.min(features.len() / 2);
        let fold_size = features.len() / k;
        let mut correct = 0;
        let mut total = 0;

        for fold in 0..k {
            let test_start = fold * fold_size;
            let test_end = if fold == k - 1 {
                features.len()
            } else {
                (fold + 1) * fold_size
            };

            // Split into train and test
            let mut train_x = Vec::new();
            let mut train_y = Vec::new();
            let mut test_x = Vec::new();
            let mut test_y = Vec::new();

            for (i, (feat, &label)) in features.iter().zip(labels.iter()).enumerate() {
                if i >= test_start && i < test_end {
                    test_x.push(feat.clone());
                    test_y.push(label);
                } else {
                    train_x.push(feat.clone());
                    train_y.push(label);
                }
            }

            // Train on fold
            let mut clf = NaiveBayesClassifier::new(1.0);
            clf.fit(&train_x, &train_y);

            // Test on fold
            for (feat, &actual) in test_x.iter().zip(test_y.iter()) {
                if let Some((predicted, _)) = clf.predict(feat) {
                    if predicted == actual {
                        correct += 1;
                    }
                }
                total += 1;
            }
        }

        if total > 0 {
            correct as f64 / total as f64
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trainer_creation() {
        let trainer = ModelTrainer::new();
        assert!(!trainer.vectorizer.is_fitted());
    }
}
