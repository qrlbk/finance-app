use std::collections::HashSet;
use unicode_segmentation::UnicodeSegmentation;
use rust_stemmers::{Algorithm, Stemmer};

/// Tokenizer with support for Russian/Kazakh text
pub struct Tokenizer {
    stop_words: HashSet<String>,
}

impl Default for Tokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer {
    pub fn new() -> Self {
        let stop_words: HashSet<String> = [
            // Russian stop words
            "и", "в", "во", "не", "что", "он", "на", "я", "с", "со", "как", "а", "то", "все",
            "она", "так", "его", "но", "да", "ты", "к", "у", "же", "вы", "за", "бы", "по",
            "только", "её", "мне", "было", "вот", "от", "меня", "ещё", "нет", "о", "из", "ему",
            "теперь", "когда", "уже", "вам", "ним", "них", "там", "тогда", "кто", "этот", "того",
            "потому", "этого", "какой", "после", "через", "эти", "нас", "про", "всего", "них",
            "какая", "много", "разве", "три", "можно", "при", "для", "до", "под", "над",
            // Kazakh stop words
            "және", "бұл", "мен", "үшін", "оның", "сол", "бір", "осы", "деп", "болып",
            "бар", "жоқ", "емес", "тек", "да", "де", "қана", "ғана", "әрі",
            // Common transaction words to ignore
            "оплата", "платеж", "перевод", "покупка", "чек", "касса", "терминал",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        Tokenizer { stop_words }
    }

    /// Tokenize text into normalized words (lowercase, filtered, Russian stemmed).
    /// After enabling stemming, users should retrain the ML model.
    pub fn tokenize(&self, text: &str) -> Vec<String> {
        let text_lower = text.to_lowercase();
        let stemmer = Stemmer::create(Algorithm::Russian);

        text_lower
            .unicode_words()
            .filter(|word| {
                if word.chars().count() < 2 {
                    return false;
                }
                if word.chars().all(|c| c.is_ascii_digit()) {
                    return false;
                }
                if self.stop_words.contains(*word) {
                    return false;
                }
                true
            })
            .map(|word| stemmer.stem(word).to_string())
            .collect()
    }

    /// Tokenize multiple documents
    pub fn tokenize_documents(&self, documents: &[String]) -> Vec<Vec<String>> {
        documents.iter().map(|doc| self.tokenize(doc)).collect()
    }

    /// Tokenize with n-grams (bigrams by default)
    pub fn tokenize_with_ngrams(&self, text: &str) -> Vec<String> {
        let tokens = self.tokenize(text);
        let mut result = tokens.clone();
        
        // Add bigrams for better context
        for window in tokens.windows(2) {
            result.push(format!("{}_{}", window[0], window[1]));
        }
        
        result
    }

    /// Tokenize with custom n-gram size
    pub fn tokenize_with_ngrams_n(&self, text: &str, n: usize) -> Vec<String> {
        let tokens = self.tokenize(text);
        let mut result = tokens.clone();
        
        // Add n-grams from 2 to n
        for ngram_size in 2..=n.min(tokens.len()) {
            for window in tokens.windows(ngram_size) {
                result.push(window.join("_"));
            }
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokenization() {
        let tokenizer = Tokenizer::new();
        let tokens = tokenizer.tokenize("Glovo доставка пиццы");
        assert_eq!(tokens.len(), 3);
        assert!(tokens.contains(&"glovo".to_string()));
        assert!(tokens.iter().any(|t| t.starts_with("достав"))); // доставка stem
        assert!(tokens.iter().any(|t| t.starts_with("пиц")));   // пиццы stem
    }

    #[test]
    fn test_stop_words_removal() {
        let tokenizer = Tokenizer::new();
        let tokens = tokenizer.tokenize("оплата в магазине за продукты");
        // оплата, в, за are stop words and removed; магазине -> магазин, продукты -> продукт
        assert!(tokens.contains(&"магазин".to_string()));
        assert!(tokens.contains(&"продукт".to_string()));
    }

    #[test]
    fn test_number_removal() {
        let tokenizer = Tokenizer::new();
        let tokens = tokenizer.tokenize("Магнит 12345 продукты");
        assert!(!tokens.contains(&"12345".to_string()));
        assert_eq!(tokens.len(), 2);
        assert!(tokens.contains(&"продукт".to_string()));
    }

    #[test]
    fn test_short_words_removal() {
        let tokenizer = Tokenizer::new();
        let tokens = tokenizer.tokenize("a b cd магазин");
        assert!(!tokens.contains(&"a".to_string()));
        assert!(!tokens.contains(&"b".to_string()));
        assert!(tokens.contains(&"cd".to_string()));
    }

    #[test]
    fn test_lowercase_normalization() {
        let tokenizer = Tokenizer::new();
        let tokens = tokenizer.tokenize("МАГАЗИН Магазин магазин");
        // All normalized to lowercase and stemmed (магазин is already stem)
        assert_eq!(tokens.iter().filter(|t| t == &"магазин").count(), 3);
    }

    #[test]
    fn test_kazakh_stop_words() {
        let tokenizer = Tokenizer::new();
        let tokens = tokenizer.tokenize("және бұл магазин үшін");
        assert!(!tokens.contains(&"және".to_string()));
        assert!(!tokens.contains(&"бұл".to_string()));
        assert!(!tokens.contains(&"үшін".to_string()));
        assert!(tokens.contains(&"магазин".to_string()));
    }

    #[test]
    fn test_ngrams_bigrams() {
        let tokenizer = Tokenizer::new();
        let tokens = tokenizer.tokenize_with_ngrams("супермаркет продукты");
        // Stemmed: супермаркет -> супермаркет, продукты -> продукт
        assert!(tokens.contains(&"супермаркет".to_string()));
        assert!(tokens.contains(&"продукт".to_string()));
        assert!(tokens.contains(&"супермаркет_продукт".to_string()));
    }

    #[test]
    fn test_ngrams_custom_n() {
        let tokenizer = Tokenizer::new();
        let tokens = tokenizer.tokenize_with_ngrams_n("магазин продукты еда", 3);
        // Stemmed: продукты -> продукт, еда -> ед
        assert!(tokens.contains(&"магазин".to_string()));
        assert!(tokens.contains(&"магазин_продукт".to_string()));
        assert!(tokens.contains(&"магазин_продукт_ед".to_string()));
    }

    #[test]
    fn test_empty_input() {
        let tokenizer = Tokenizer::new();
        let tokens = tokenizer.tokenize("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_only_stop_words() {
        let tokenizer = Tokenizer::new();
        let tokens = tokenizer.tokenize("и в на за");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_tokenize_documents() {
        let tokenizer = Tokenizer::new();
        let docs = vec![
            "магазин продукты".to_string(),
            "кафе обед".to_string(),
        ];
        let tokenized = tokenizer.tokenize_documents(&docs);
        assert_eq!(tokenized.len(), 2);
        assert!(tokenized[0].contains(&"магазин".to_string()));
        assert!(tokenized[0].contains(&"продукт".to_string()));
        assert!(tokenized[1].contains(&"обед".to_string())); // кафе may stem; обед -> обед
    }

    #[test]
    fn test_mixed_script_text() {
        let tokenizer = Tokenizer::new();
        let tokens = tokenizer.tokenize("KFC ресторан 5000");
        assert!(tokens.contains(&"kfc".to_string()));
        // ресторан stems to "ресторан" (unchanged) or "рест" depending on algorithm
        assert!(tokens.iter().any(|t| t.starts_with("рест")));
        assert!(!tokens.contains(&"5000".to_string()));
    }
}
