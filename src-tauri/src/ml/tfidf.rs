use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// TF-IDF Vectorizer for converting text to numerical features
#[derive(Clone, Serialize, Deserialize)]
pub struct TfIdfVectorizer {
    /// Word to index mapping
    vocabulary: HashMap<String, usize>,
    /// Inverse Document Frequency weights
    idf: Vec<f64>,
    /// Whether the vectorizer has been fitted
    fitted: bool,
}

impl Default for TfIdfVectorizer {
    fn default() -> Self {
        Self::new()
    }
}

impl TfIdfVectorizer {
    pub fn new() -> Self {
        TfIdfVectorizer {
            vocabulary: HashMap::new(),
            idf: Vec::new(),
            fitted: false,
        }
    }

    /// Fit the vectorizer on a corpus of tokenized documents
    pub fn fit(&mut self, documents: &[Vec<String>]) {
        let n_docs = documents.len() as f64;
        
        // Build vocabulary
        let mut word_doc_count: HashMap<String, usize> = HashMap::new();
        
        for doc in documents {
            // Count unique words per document
            let unique_words: std::collections::HashSet<&String> = doc.iter().collect();
            for word in unique_words {
                *word_doc_count.entry(word.clone()).or_insert(0) += 1;
            }
        }
        
        // Create vocabulary (sorted for consistency)
        let mut words: Vec<_> = word_doc_count.keys().cloned().collect();
        words.sort();
        
        self.vocabulary.clear();
        for (idx, word) in words.iter().enumerate() {
            self.vocabulary.insert(word.clone(), idx);
        }
        
        // Calculate IDF for each word
        // IDF = log((n_docs + 1) / (doc_count + 1)) + 1 (smoothed)
        self.idf = vec![0.0; self.vocabulary.len()];
        for (word, &idx) in &self.vocabulary {
            let doc_count = word_doc_count.get(word).copied().unwrap_or(0) as f64;
            self.idf[idx] = ((n_docs + 1.0) / (doc_count + 1.0)).ln() + 1.0;
        }
        
        self.fitted = true;
    }

    /// Transform a single tokenized document to TF-IDF vector
    pub fn transform(&self, tokens: &[String]) -> Vec<f64> {
        if !self.fitted || self.vocabulary.is_empty() {
            return vec![];
        }
        
        let mut tfidf = vec![0.0; self.vocabulary.len()];
        
        // Count term frequencies
        let mut tf: HashMap<&String, usize> = HashMap::new();
        for token in tokens {
            *tf.entry(token).or_insert(0) += 1;
        }
        
        let n_terms = tokens.len() as f64;
        if n_terms == 0.0 {
            return tfidf;
        }
        
        // Calculate TF-IDF
        for (word, &count) in &tf {
            if let Some(&idx) = self.vocabulary.get(*word) {
                // TF = count / total_terms (normalized)
                let term_freq = count as f64 / n_terms;
                tfidf[idx] = term_freq * self.idf[idx];
            }
        }
        
        // L2 normalize the vector
        let norm: f64 = tfidf.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm > 0.0 {
            for val in &mut tfidf {
                *val /= norm;
            }
        }
        
        tfidf
    }

    /// Transform multiple documents
    pub fn transform_many(&self, documents: &[Vec<String>]) -> Vec<Vec<f64>> {
        documents.iter().map(|doc| self.transform(doc)).collect()
    }

    /// Get vocabulary size
    pub fn vocabulary_size(&self) -> usize {
        self.vocabulary.len()
    }

    /// Check if fitted
    pub fn is_fitted(&self) -> bool {
        self.fitted
    }

    /// Get vocabulary reference
    pub fn vocabulary(&self) -> &HashMap<String, usize> {
        &self.vocabulary
    }

    /// Get IDF weights reference
    pub fn idf_weights(&self) -> &Vec<f64> {
        &self.idf
    }

    /// Create from saved state
    pub fn from_saved(vocabulary: HashMap<String, usize>, idf: Vec<f64>) -> Self {
        TfIdfVectorizer {
            vocabulary,
            idf,
            fitted: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fit_transform() {
        let docs = vec![
            vec!["glovo".to_string(), "пицца".to_string()],
            vec!["магнит".to_string(), "продукты".to_string()],
            vec!["glovo".to_string(), "бургер".to_string()],
        ];

        let mut vectorizer = TfIdfVectorizer::new();
        vectorizer.fit(&docs);

        assert!(vectorizer.vocabulary_size() > 0);
        assert!(vectorizer.is_fitted());

        let vec = vectorizer.transform(&["glovo".to_string(), "пицца".to_string()]);
        assert_eq!(vec.len(), vectorizer.vocabulary_size());
    }

    #[test]
    fn test_empty_document() {
        let docs = vec![
            vec!["test".to_string()],
        ];

        let mut vectorizer = TfIdfVectorizer::new();
        vectorizer.fit(&docs);

        let vec = vectorizer.transform(&[]);
        assert!(vec.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn test_unfitted_vectorizer() {
        let vectorizer = TfIdfVectorizer::new();
        assert!(!vectorizer.is_fitted());
        assert_eq!(vectorizer.vocabulary_size(), 0);
        
        let vec = vectorizer.transform(&["test".to_string()]);
        assert!(vec.is_empty());
    }

    #[test]
    fn test_idf_weighting() {
        // Word appearing in all docs should have lower IDF
        // Word appearing in one doc should have higher IDF
        let docs = vec![
            vec!["common".to_string(), "unique1".to_string()],
            vec!["common".to_string(), "unique2".to_string()],
            vec!["common".to_string(), "unique3".to_string()],
        ];

        let mut vectorizer = TfIdfVectorizer::new();
        vectorizer.fit(&docs);

        let common_idx = *vectorizer.vocabulary().get("common").unwrap();
        let unique_idx = *vectorizer.vocabulary().get("unique1").unwrap();
        
        // IDF for common word should be lower than unique word
        assert!(vectorizer.idf_weights()[common_idx] < vectorizer.idf_weights()[unique_idx]);
    }

    #[test]
    fn test_l2_normalization() {
        let docs = vec![
            vec!["test".to_string(), "document".to_string()],
        ];

        let mut vectorizer = TfIdfVectorizer::new();
        vectorizer.fit(&docs);

        let vec = vectorizer.transform(&["test".to_string(), "document".to_string()]);
        
        // L2 norm should be 1.0 (or close to it due to floating point)
        let norm: f64 = vec.iter().map(|x| x * x).sum::<f64>().sqrt();
        assert!((norm - 1.0).abs() < 0.0001 || norm == 0.0);
    }

    #[test]
    fn test_unknown_words() {
        let docs = vec![
            vec!["known".to_string()],
        ];

        let mut vectorizer = TfIdfVectorizer::new();
        vectorizer.fit(&docs);

        // Unknown word should not affect the vector
        let vec = vectorizer.transform(&["unknown".to_string()]);
        assert!(vec.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn test_transform_many() {
        let docs = vec![
            vec!["test".to_string()],
            vec!["document".to_string()],
        ];

        let mut vectorizer = TfIdfVectorizer::new();
        vectorizer.fit(&docs);

        let vecs = vectorizer.transform_many(&docs);
        assert_eq!(vecs.len(), 2);
        assert_eq!(vecs[0].len(), vectorizer.vocabulary_size());
    }

    #[test]
    fn test_from_saved() {
        let mut vocabulary = HashMap::new();
        vocabulary.insert("test".to_string(), 0);
        vocabulary.insert("word".to_string(), 1);
        let idf = vec![1.5, 2.0];

        let vectorizer = TfIdfVectorizer::from_saved(vocabulary, idf);
        
        assert!(vectorizer.is_fitted());
        assert_eq!(vectorizer.vocabulary_size(), 2);
    }

    #[test]
    fn test_vocabulary_consistency() {
        let docs = vec![
            vec!["apple".to_string(), "banana".to_string()],
            vec!["cherry".to_string(), "apple".to_string()],
        ];

        let mut v1 = TfIdfVectorizer::new();
        let mut v2 = TfIdfVectorizer::new();
        
        v1.fit(&docs);
        v2.fit(&docs);

        // Same documents should produce same vocabulary ordering
        assert_eq!(v1.vocabulary(), v2.vocabulary());
    }
}
