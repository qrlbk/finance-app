mod auth;
mod bank_import;
mod commands;
mod crypto;
mod db;
mod embedded_llm;
mod error;
mod export;
mod llm;
mod messages;
mod ml;
mod security;

use commands::AppState;
use tauri::Manager;
use tracing::{info, warn, Level};
use tracing_subscriber::FmtSubscriber;

pub use error::{AppError, AppResult, ValidationError};

fn init_tracing() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(true)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true)
        .compact()
        .init();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize tracing for logging
    init_tracing();
    info!("Starting Finance App");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            db_path: std::sync::Mutex::new(None),
            cache: commands::QueryCache::new(),
            current_user_id: std::sync::Mutex::new(None),
        })
        .setup(|app| {
            info!("Setting up application");
            if let Some(state) = app.try_state::<AppState>() {
                if let Err(e) = commands::init_db_on_startup(app.handle(), state.inner()) {
                    warn!("Failed to init DB: {}", e);
                    eprintln!("Failed to init DB: {}", e);
                }
            }
            info!("Application setup complete");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::register,
            commands::login,
            commands::logout,
            commands::get_current_session,
            commands::list_users,
            commands::get_accounts,
            commands::create_account,
            commands::update_account,
            commands::delete_account,
            commands::reassign_transactions_to_account,
            commands::get_categories,
            commands::get_transactions,
            commands::create_transaction,
            commands::update_transaction,
            commands::delete_transaction,
            commands::get_summary,
            commands::get_expense_by_category,
            commands::get_monthly_totals,
            commands::export_backup,
            commands::restore_backup,
            commands::create_transfer,
            commands::get_transfers,
            // ML commands
            commands::predict_category,
            commands::train_model,
            commands::get_model_status,
            commands::get_embedded_llm_status,
            commands::download_and_register_embedded_model,
            commands::test_embedded_llm,
            commands::start_ollama_server,
            commands::chat_message,
            commands::ensure_ollama_installed,
            commands::get_insights,
            commands::get_smart_insights,
            commands::get_forecast_details,
            // Recurring payments
            commands::get_recurring_payments,
            commands::create_recurring,
            commands::update_recurring,
            commands::delete_recurring,
            commands::process_recurring_payments,
            // Budget commands
            commands::get_budgets,
            commands::create_budget,
            commands::update_budget,
            commands::delete_budget,
            commands::get_budget_alerts,
            // Category management
            commands::create_category,
            commands::update_category,
            commands::delete_category,
            // Export/Import
            commands::export_data,
            commands::import_data,
            commands::import_preview,
            commands::open_file,
            commands::reset_database,
            // Bank statement import
            commands::parse_bank_statement,
            commands::import_bank_transactions,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
