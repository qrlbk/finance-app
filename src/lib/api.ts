import { invoke } from "@tauri-apps/api/core";

export interface UserSession {
  id: number;
  username: string;
  display_name: string;
}

export interface Account {
  id: number;
  name: string;
  account_type: string;
  balance: number;
  currency: string;
}

export interface Category {
  id: number;
  name: string;
  category_type: string;
  icon: string | null;
  color: string | null;
  parent_id: number | null;
}

export interface TransactionWithDetails {
  id: number;
  account_id: number;
  account_name: string;
  category_id: number | null;
  category_name: string | null;
  amount: number;
  transaction_type: string;
  note: string | null;
  date: string;
}

export interface TransferWithDetails {
  id: number;
  from_account_id: number;
  from_account_name: string;
  to_account_id: number;
  to_account_name: string;
  amount: number;
  date: string;
  note: string | null;
}

export interface Summary {
  total_balance: number;
  income_month: number;
  expense_month: number;
  currencies: string[];
}

// ML Types
export interface CategoryPrediction {
  category_id: number;
  category_name: string;
  confidence: number;
}

export interface TrainResult {
  success: boolean;
  sample_count: number;
  accuracy: number | null;
  message: string;
}

export interface ModelStatus {
  trained: boolean;
  trained_at: string | null;
  sample_count: number | null;
  accuracy: number | null;
  transactions_with_categories_count: number | null;
  transactions_with_note_no_category: number | null;
}

export interface EmbeddedLlmStatus {
  downloaded: boolean;
  download_progress: number | null;
  registered_in_ollama: boolean;
  ollama_reachable: boolean;
  error: string | null;
}

export interface TestEmbeddedLlmResult {
  success: boolean;
  message: string;
}

export interface Anomaly {
  message: string;
  severity: "warning" | "alert";
  category: string | null;
  expected: number;
  actual: number;
}

export interface Forecast {
  predicted_expense: number;
  confidence_low: number;
  confidence_high: number;
  trend: "up" | "down" | "stable";
  trend_percent: number;
}

export interface CategoryForecast {
  category_id: number;
  category_name: string;
  predicted_expense: number;
}

export interface ForecastDetails {
  overall: Forecast;
  by_category: CategoryForecast[];
}

export interface Insights {
  anomalies: Anomaly[];
  forecast: Forecast | null;
  /** Number of months of expense data available (needed 3 for forecasting) */
  months_of_data: number;
}

// Smart Insights Types
export interface SpendingPattern {
  category: string;
  category_color: string | null;
  avg_amount: number;
  total_transactions: number;
  typical_frequency: string;
  trend: "increasing" | "decreasing" | "stable";
  trend_percent: number;
}

export interface SavingsSuggestion {
  category: string;
  current_spending: number;
  suggested_limit: number;
  potential_savings: number;
  suggestion: string;
  confidence: number;
}

export interface MonthlyComparison {
  current_month_total: number;
  previous_month_total: number;
  change_percent: number;
  top_increase_category: string | null;
  top_decrease_category: string | null;
}

export interface SmartInsights {
  patterns: SpendingPattern[];
  suggestions: SavingsSuggestion[];
  monthly_comparison: MonthlyComparison;
  high_spending_days: string[];
}

// Recurring Payment Types
export interface RecurringPayment {
  id: number;
  account_id: number;
  account_name: string;
  category_id: number | null;
  category_name: string | null;
  amount: number;
  payment_type: string;
  frequency: string;
  next_date: string;
  end_date: string | null;
  note: string | null;
  is_active: boolean;
}

// Budget Types
export interface Budget {
  id: number;
  category_id: number;
  category_name: string;
  category_color: string | null;
  amount: number;
  spent: number;
  remaining: number;
  percent_used: number;
  period: string;
}

export interface BudgetAlert {
  category_name: string;
  percent_used: number;
  severity: "warning" | "exceeded";
}

// Export/Import Types
export interface ImportResult {
  transactions_imported: number;
  duplicates_skipped: number;
  accounts_imported: number;
  categories_imported: number;
  errors: string[];
  total_parsed?: number;
}

export interface ImportPreview {
  headers?: string[] | null;
  rows?: string[][] | null;
  transaction_count?: number | null;
}

// Bank Statement Import Types
export interface ParsedTransaction {
  date: string;
  amount: number;
  transaction_type: string;
  description: string;
  original_type: string;
  suggested_category_id: number | null;
  confidence: number | null;
  is_duplicate: boolean;
}

export interface ParsedStatement {
  bank: string;
  period_start: string;
  period_end: string;
  account: string | null;
  card: string | null;
  transactions: ParsedTransaction[];
}

export interface BankImportResult {
  imported: number;
  skipped_duplicates: number;
  failed: number;
  errors: string[];
}

export interface ImportTransaction {
  date: string;
  amount: number;
  transaction_type: string;
  description: string;
  category_id: number | null;
  skip_if_duplicate: boolean;
}

export const api = {
  // Auth (no session required)
  register: (username: string, password: string, displayName: string) =>
    invoke<UserSession>("register", { args: { username, password, displayName } }),
  login: (username: string, password: string) =>
    invoke<UserSession>("login", { username, password }),
  logout: () => invoke<void>("logout"),
  getCurrentSession: () => invoke<UserSession | null>("get_current_session"),
  listUsers: () => invoke<UserSession[]>("list_users"),

  getAccounts: () => invoke<Account[]>("get_accounts"),
  createAccount: (input: {
    name: string;
    account_type: string;
    currency?: string;
    /** Стартовый капитал — сумма, которая уже есть на счёте (на карте и т.д.) */
    initial_balance?: number;
  }) => invoke<number>("create_account", { input }),
  updateAccount: (input: { id: number; name: string; account_type: string; currency?: string }) =>
    invoke("update_account", { input }),
  deleteAccount: (id: number) => invoke("delete_account", { id }),
  reassignTransactionsToAccount: (from_account_id: number, to_account_id: number) =>
    invoke("reassign_transactions_to_account", { from_account_id, to_account_id }),

  getCategories: () => invoke<Category[]>("get_categories"),

  getTransactions: (input?: {
    limit?: number;
    offset?: number;
    account_id?: number;
    date_from?: string;
    date_to?: string;
    category_id?: number;
    uncategorized_only?: boolean;
    transaction_type?: string;
    search_note?: string;
  }) => invoke<TransactionWithDetails[]>("get_transactions", { input: input ?? {} }),
  createTransaction: (input: {
    account_id: number;
    category_id?: number | null;
    amount: number;
    transaction_type: string;
    note?: string | null;
    date: string;
  }) => invoke<number>("create_transaction", { input }),
  updateTransaction: (input: {
    id: number;
    account_id: number;
    category_id?: number | null;
    amount: number;
    transaction_type: string;
    note?: string | null;
    date: string;
  }) => invoke("update_transaction", { input }),
  deleteTransaction: (id: number) => invoke("delete_transaction", { id }),

  getSummary: () => invoke<Summary>("get_summary"),

  getExpenseByCategory: (input: { year: number; month: number; include_children?: boolean }) =>
    invoke<{ category_name: string; total: number }[]>("get_expense_by_category", { input }),
  getMonthlyTotals: (input?: { months?: number }) =>
    invoke<{ month: string; income: number; expense: number }[]>("get_monthly_totals", {
      input: input ?? { months: 6 },
    }),

  exportBackup: () => invoke<string>("export_backup"),
  restoreBackup: (path: string) => invoke("restore_backup", { path }),
  resetDatabase: () => invoke("reset_database"),

  createTransfer: (input: {
    from_account_id: number;
    to_account_id: number;
    amount: number;
    date: string;
    note?: string | null;
  }) => invoke("create_transfer", { input }),
  getTransfers: (input?: { limit?: number }) =>
    invoke<TransferWithDetails[]>("get_transfers", { input: input ?? {} }),

  // ML methods
  predictCategory: (
    note: string,
    amount?: number,
    date?: string,
    confidenceThreshold?: number,
    options?: { useLlm?: boolean; ollamaUrl?: string; ollamaModel?: string; transactionType?: string; useEmbedded?: boolean }
  ) =>
    invoke<CategoryPrediction | null>("predict_category", {
      note,
      amount,
      date,
      confidence_threshold: confidenceThreshold ?? undefined,
      use_llm: options?.useLlm ?? undefined,
      ollama_url: options?.ollamaUrl ?? undefined,
      ollama_model: options?.ollamaModel ?? undefined,
      transaction_type: options?.transactionType ?? undefined,
      use_embedded: options?.useEmbedded ?? undefined,
    }),
  trainModel: () => invoke<TrainResult>("train_model"),
  getModelStatus: () => invoke<ModelStatus>("get_model_status"),
  getInsights: () => invoke<Insights>("get_insights"),
  getSmartInsights: () => invoke<SmartInsights>("get_smart_insights"),

  // Recurring Payments
  getRecurringPayments: () => invoke<RecurringPayment[]>("get_recurring_payments"),
  createRecurring: (input: {
    account_id: number;
    category_id?: number | null;
    amount: number;
    payment_type: string;
    frequency: string;
    next_date: string;
    end_date?: string | null;
    note?: string | null;
  }) => invoke<number>("create_recurring", { input }),
  updateRecurring: (input: {
    id: number;
    account_id: number;
    category_id?: number | null;
    amount: number;
    payment_type: string;
    frequency: string;
    next_date: string;
    end_date?: string | null;
    note?: string | null;
    is_active: boolean;
  }) => invoke("update_recurring", { input }),
  deleteRecurring: (id: number) => invoke("delete_recurring", { id }),
  processRecurringPayments: () => invoke<number[]>("process_recurring_payments"),

  // Budgets
  getBudgets: () => invoke<Budget[]>("get_budgets"),
  createBudget: (input: { category_id: number; amount: number; period: string }) =>
    invoke<number>("create_budget", { input }),
  updateBudget: (input: { id: number; amount: number }) =>
    invoke("update_budget", { input }),
  deleteBudget: (id: number) => invoke("delete_budget", { id }),
  getBudgetAlerts: () => invoke<BudgetAlert[]>("get_budget_alerts"),

  // Category Management
  createCategory: (input: {
    name: string;
    category_type: string;
    icon?: string | null;
    color?: string | null;
    parent_id?: number | null;
  }) => invoke<number>("create_category", { input }),
  updateCategory: (input: {
    id: number;
    name: string;
    icon?: string | null;
    color?: string | null;
  }) => invoke("update_category", { input }),
  deleteCategory: (id: number) => invoke("delete_category", { id }),

  // Export/Import
  exportData: (input: {
    format: string;
    date_from?: string | null;
    date_to?: string | null;
    include_accounts: boolean;
    include_categories: boolean;
    account_id?: number | null;
    category_id?: number | null;
  }) => invoke<string>("export_data", { input }),
  importData: (input: {
    path: string;
    format: string;
    default_account_id?: number | null;
    skip_duplicates?: boolean;
  }) => invoke<ImportResult>("import_data", { input }),
  importPreview: (path: string, format: string) =>
    invoke<ImportPreview>("import_preview", { path, format }),

  // Open file with system application
  openFile: (path: string) => invoke("open_file", { path }),

  // Detailed Forecasts
  getForecastDetails: () => invoke<ForecastDetails>("get_forecast_details"),

  // Bank Statement Import
  parseBankStatement: (
    path: string,
    options?: { useLlm?: boolean; ollamaUrl?: string; ollamaModel?: string; useEmbedded?: boolean }
  ) =>
    invoke<ParsedStatement>("parse_bank_statement", {
      path,
      use_llm: options?.useLlm ?? undefined,
      ollama_url: options?.ollamaUrl ?? undefined,
      ollama_model: options?.ollamaModel ?? undefined,
      use_embedded: options?.useEmbedded ?? undefined,
    }),
  getEmbeddedLlmStatus: () => invoke<EmbeddedLlmStatus>("get_embedded_llm_status"),
  /** Проверить/установить Ollama. Возвращает "Ready" или "OpenedDownload". */
  ensureOllamaInstalled: () =>
    invoke<"Ready" | "OpenedDownload">("ensure_ollama_installed"),
  /** Запустить сервер Ollama в фоне (если ещё не запущен). */
  startOllamaServer: () => invoke<void>("start_ollama_server"),
  /** Чат с Ollama. options — context (данные пользователя), systemPrompt, useEmbedded, ollamaUrl, ollamaModel. */
  chatMessage: (
    message: string,
    options?: {
      context?: string;
      systemPrompt?: string;
      useEmbedded?: boolean;
      ollamaUrl?: string;
      ollamaModel?: string;
    }
  ) =>
    invoke<string>("chat_message", {
      message,
      system_prompt: options?.systemPrompt ?? undefined,
      context: options?.context ?? undefined,
      use_embedded: options?.useEmbedded ?? undefined,
      ollama_url: options?.ollamaUrl ?? undefined,
      ollama_model: options?.ollamaModel ?? undefined,
    }),
  downloadAndRegisterEmbeddedModel: () =>
    invoke<void>("download_and_register_embedded_model"),
  testEmbeddedLlm: () =>
    invoke<TestEmbeddedLlmResult>("test_embedded_llm"),
  importBankTransactions: (input: {
    transactions: ImportTransaction[];
    account_id: number;
    skip_duplicates: boolean;
  }) => invoke<BankImportResult>("import_bank_transactions", { input }),
};
