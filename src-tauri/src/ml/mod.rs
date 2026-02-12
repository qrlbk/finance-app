pub mod tokenizer;
pub mod tfidf;
pub mod classifier;
pub mod model;
pub mod trainer;
pub mod anomaly;
pub mod forecast;
pub mod features;
pub mod insights;

pub use model::CategoryModel;
pub use trainer::ModelTrainer;
pub use anomaly::{AnomalyDetector, Anomaly};
pub use forecast::{ExpenseForecaster, Forecast};
pub use features::{TransactionFeatures, AmountBucket, TimeFeatures};
pub use insights::{SmartInsights, SpendingPattern, SavingsSuggestion};
