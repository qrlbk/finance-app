# ML Model — Transaction Category Prediction

This document describes how the ML model works: from training on database data to predicting transaction category from the note text (and optionally amount/date).

---

## 1. Overview

The model solves a **text classification** task: from a transaction description (note, amount, date) it predicts an income/expense category.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         TRAINING (train_model)                                │
├─────────────────────────────────────────────────────────────────────────────┤
│  DB (transactions + categories)                                              │
│        │                                                                     │
│        ▼                                                                     │
│  Load: note, category_id, amount, date                                       │
│        │                                                                     │
│        ▼                                                                     │
│  Tokenizer → TF-IDF → Features (TF-IDF + amount + time) → NaiveBayes         │
│        │                                                                     │
│        ▼                                                                     │
│  CategoryModel (vocabulary, idf, priors, probs, category_names)               │
│        │                                                                     │
│        ▼                                                                     │
│  Save to ml_model.json                                                       │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                         PREDICTION (predict_category)                        │
├─────────────────────────────────────────────────────────────────────────────┤
│  Input: note, amount?, date?                                                 │
│        │                                                                     │
│        ▼                                                                     │
│  Load CategoryModel from ml_model.json                                       │
│        │                                                                     │
│        ▼                                                                     │
│  Tokenizer → TF-IDF (saved) → Features → NaiveBayes (saved)                  │
│        │                                                                     │
│        ▼                                                                     │
│  (category_id, category_name, confidence)                                    │
│        │                                                                     │
│        ▼                                                                     │
│  Filter: confidence >= 0.3 → return to user                                   │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Training pipeline (summary)

1. **Load training data** — `SELECT note, category_id, amount, date` from transactions with non-null category and non-empty note.
2. **Minimum 20 samples** — Else return “insufficient data.”
3. **Tokenize** — Each note → tokens (with optional n-grams).
4. **Filter** — Keep only rows with non-empty tokens; require ≥ 20 again.
5. **TF-IDF fit** — Build vocabulary and IDF from corpus.
6. **Features** — For each row: TF-IDF vector + optional amount bucket + time features → combined vector.
7. **Naive Bayes fit** — On (features, labels).
8. **Evaluate** — e.g. 5-fold cross-validation for accuracy.
9. **Save** — CategoryModel to JSON (`ml_model.json` or `category_model.json`).

---

## 3. Prediction pipeline (summary)

1. **Input check** — Note non-empty and length ≥ 3; else return `None`.
2. **Model path** — If file missing, return `None`.
3. **Load model** — `CategoryModel::load(path)`.
4. **Tokenize** — Same as training (with n-grams if `use_enhanced_features`).
5. **Transform** — `vectorizer.transform(tokens)` → TF-IDF vector.
6. **Optional** — If enhanced and amount/date present, build `TransactionFeatures` and combined vector.
7. **Predict** — `classifier.predict(features)` → (category_id, confidence).
8. **Threshold** — If confidence < 0.3 (in `commands.rs`), return `None`; else return `CategoryPrediction`.

---

## 4. Components

| Component | File | Purpose |
|-----------|------|---------|
| **Tokenizer** | `ml/tokenizer.rs` | Split text into words; remove stop-words (RU/KZ), short words, numbers; optional n-grams (e.g. bigrams). |
| **TfIdfVectorizer** | `ml/tfidf.rs` | Build vocabulary and IDF from corpus; transform tokens to L2-normalized TF-IDF vectors. |
| **TransactionFeatures** | `ml/features.rs` | Extend TF-IDF: amount bucket (VerySmall..VeryLarge), time (weekday, weekend, month start/end). |
| **NaiveBayesClassifier** | `ml/classifier.rs` | Multinomial Naive Bayes with Laplace smoothing (alpha=1). Returns class and confidence. |
| **CategoryModel** | `ml/model.rs` | Serializable wrapper: vocabulary, IDF, classifier weights, category names; save/load JSON; predict / predict_with_context. |
| **ModelTrainer** | `ml/trainer.rs` | Load from DB, run training pipeline, evaluate (k-fold), create and save CategoryModel. |

---

## 5. Tokenization (detail)

1. **Lowercase** the note.
2. **Split** on word boundaries (Unicode).
3. **Filter:** length ≥ 2, not numeric-only, not stop-word (e.g. “и”, “в”, “оплата”, “және”).
4. **Optional (enhanced):** add bigrams (e.g. “glovo”, “доставка”, “пиццы” → also “glovo_доставка”, “доставка_пиццы”).

Output: `Vec<String>` of tokens.

---

## 6. TF-IDF (fit and transform)

**Fit (training):**

- Count document frequency per term.
- Build sorted vocabulary (term → index).
- IDF[term] = ln((N_docs + 1) / (doc_count(term) + 1)) + 1 (smoothed).

**Transform (per document):**

- TF = term count / total terms.
- TF-IDF[i] = TF[i] * IDF[i] for terms in vocabulary.
- L2-normalize the vector.

---

## 7. Enhanced features (TransactionFeatures)

Used only when `use_enhanced_features` and amount/date are available.

- **Amount bucket** — One-hot over 5 buckets: VerySmall (<1k), Small (1k–5k), Medium (5k–20k), Large (20k–100k), VeryLarge (100k+).
- **Time** — Weekday (7 one-hot), is_weekend, is_month_start (days 1–5), is_month_end (days 26–31).

Combined vector = [ TF-IDF ] + [ amount bucket ] + [ time ].

---

## 8. Naive Bayes (fit and predict)

**Fit:**

- Class counts → log priors: log P(c) = ln(count[c] / N).
- Per class: sum of feature vectors → Laplace smoothing → log P(x_i | c).

**Predict:**

- For each class: log P(c|x) ∝ log P(c) + Σ x_i * log P(x_i|c).
- Class = argmax. Confidence from log-probabilities (e.g. softmax).

Output: (category_id, confidence).

---

## 9. Where the model is used

| Place | Action |
|-------|--------|
| **Settings** | “Train model” → `train_model`; show status via `get_model_status`. |
| **Transactions** | On note input (debounced) → `predict_category(note, amount, date)`; show suggested category. |
| **Import** | For each imported row, optional `predict_category_internal(description)` for suggested_category_id and confidence. |

---

## 10. Model file and version

- **Path:** e.g. `ProjectDirs::data_dir("finance-app") / "ml_model.json"` (or `category_model.json`).
- **Version:** e.g. `CategoryModel::CURRENT_VERSION = 2`. If file version > supported, return error on load.

---

## 11. Improvement recommendations

### 11.1 Hybrid: rules + ML

- **Idea:** 60–80% of transactions can be categorized by rules (exact/substring match from history); ML for the rest.
- **Implementation:** Before ML, check a rule engine (e.g. normalized substring or pattern → category_id). On match, return (category, confidence=1.0). Otherwise run current ML pipeline.
- **Effect:** Stable categories for repeated descriptions; fewer ML calls.

### 11.2 Stemming / lemmatization (Russian, Kazakh)

- **Current:** Lowercase + stop-words + length filter.
- **Improvement:** Add stemming (e.g. Snowball) so different word forms map to one stem; optionally lemmatization if a light Rust solution exists. Use stems in TF-IDF vocabulary.

### 11.3 Class imbalance

- **Issue:** Naive Bayes priors favor frequent categories.
- **Options:** Class weights in training; per-class confidence thresholds; minimum samples per class or merge rare categories.

### 11.4 Feature independence (Naive Bayes)

- **Issue:** Redundant features (e.g. weekday one-hot + is_weekend) violate independence assumption.
- **Recommendation:** Simplify time features (e.g. keep either weekday or is_weekend + month_start/end). Keep amount buckets; avoid duplicating the same information in multiple columns.

### 11.5 Vocabulary quality

- Min document frequency for terms (exclude hapax legomena).
- Max vocabulary size (e.g. top-N by document frequency or IDF).
- Expand domain stop-words (e.g. bank statement boilerplate).

### 11.6 Confidence threshold and calibration

- **Current:** Fixed threshold 0.3.
- **Improvement:** Make threshold configurable (e.g. in app settings). Optionally calibrate confidence on a held-out set (e.g. temperature scaling) so reported confidence matches empirical accuracy.

### 11.7 Alternative models (optional)

- **fastText (Rust)** — Fast training and inference, subword; good for short text.
- **Embeddings + classifier** — Semantic similarity; e.g. fastembed-rs + k-NN or linear classifier.
- **Sentence transformers** — High accuracy but typically Python; Rust would require FFI.

**Practical path:** Keep TF-IDF + Naive Bayes as the main lightweight pipeline; add rule-based layer (11.1); improve preprocessing and imbalance (11.2–11.3). Consider fastText or embeddings later if more accuracy is needed.

### 11.8 Feedback and retraining

- Log accept/reject of suggestions (locally) for future retraining or metrics.
- Prompt “Retrain model?” after N new categorized transactions (e.g. +50).
- On retrain, use all labeled transactions to preserve rare categories.

### 11.9 Implementation priority

1. **High impact, moderate effort:** Rules + ML (11.1), class imbalance (11.3), configurable threshold (11.6).
2. **Medium impact:** Stemming (11.2), vocabulary pruning (11.5), simpler time features (11.4).
3. **Long term:** Confidence calibration, rule layer from “decision history,” optional second classifier (fastText or embeddings).
