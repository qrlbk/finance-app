import { useState, useEffect } from "react";
import { Brain, RefreshCw, CheckCircle, XCircle, PiggyBank, Plus, Trash2, FileDown, FileUp, FileJson, FileSpreadsheet, ExternalLink, FileText, DollarSign } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import { getTheme, setTheme, initTheme, type Theme } from "../stores/themeStore";
import { api, type ModelStatus, type Budget, type Category, type ImportResult, type Account, type EmbeddedLlmStatus, type ExchangeRateRow } from "../lib/api";
import { ConfirmDialog } from "../components/ui/ConfirmDialog";
import { useToast } from "../components/ui/Toast";
import { SettingsTheme } from "./settings/SettingsTheme";
import { SettingsBackup } from "./settings/SettingsBackup";

export function Settings() {
  const { showToast } = useToast();
  const [theme, setThemeState] = useState<Theme>(getTheme());
  const [backupPath, setBackupPath] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [backupExporting, setBackupExporting] = useState(false);
  const [restoreConfirmPath, setRestoreConfirmPath] = useState<string | null>(null);
  const [restoreInProgress, setRestoreInProgress] = useState(false);
  const [resetDbConfirm, setResetDbConfirm] = useState(false);
  const [resetInProgress, setResetInProgress] = useState(false);

  // ML State
  const ML_THRESHOLD_KEY = "ml_confidence_threshold";
  const ML_THRESHOLD_DEFAULT = 0.3;
  const LLM_ENABLED_KEY = "llm_enabled";
  const LLM_USE_EMBEDDED_KEY = "llm_use_embedded";
  const OLLAMA_URL_KEY = "ollama_url";
  const OLLAMA_MODEL_KEY = "ollama_model";
  const [modelStatus, setModelStatus] = useState<ModelStatus | null>(null);
  const [mlLoading, setMlLoading] = useState(true);
  const [training, setTraining] = useState(false);
  const [confidenceThreshold, setConfidenceThreshold] = useState(() => {
    try {
      const v = localStorage.getItem(ML_THRESHOLD_KEY);
      if (v != null) {
        const n = parseFloat(v);
        if (!Number.isNaN(n) && n >= 0.2 && n <= 0.9) return n;
      }
    } catch {}
    return ML_THRESHOLD_DEFAULT;
  });
  const [llmEnabled, setLlmEnabled] = useState(() => localStorage.getItem(LLM_ENABLED_KEY) === "true");
  const [useEmbeddedLlm, setUseEmbeddedLlm] = useState(() => localStorage.getItem(LLM_USE_EMBEDDED_KEY) === "true");
  const [ollamaUrl, setOllamaUrl] = useState(() => localStorage.getItem(OLLAMA_URL_KEY) ?? "http://127.0.0.1:11434");
  const [ollamaModel, setOllamaModel] = useState(() => localStorage.getItem(OLLAMA_MODEL_KEY) ?? "llama3.2");
  const [embeddedLlmStatus, setEmbeddedLlmStatus] = useState<EmbeddedLlmStatus | null>(null);
  const [embeddedDownloading, setEmbeddedDownloading] = useState(false);
  const [embeddedTestingLlm, setEmbeddedTestingLlm] = useState(false);

  // Budget State
  const [budgets, setBudgets] = useState<Budget[]>([]);
  const [categories, setCategories] = useState<Category[]>([]);
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [allCategories, setAllCategories] = useState<Category[]>([]);
  const [showBudgetForm, setShowBudgetForm] = useState(false);
  const [budgetForm, setBudgetForm] = useState({
    category_id: 0,
    amount: "",
    period: "monthly",
  });
  const [deleteBudgetId, setDeleteBudgetId] = useState<number | null>(null);

  // Export State
  const [showExportForm, setShowExportForm] = useState(false);
  const [exportForm, setExportForm] = useState({
    format: "xlsx",
    date_from: "",
    date_to: "",
    include_accounts: true,
    include_categories: true,
    account_id: 0,
    category_id: 0,
  });
  const [exporting, setExporting] = useState(false);
  const [exportPath, setExportPath] = useState<string | null>(null);
  const [importResult, setImportResult] = useState<ImportResult | null>(null);
  const [importDefaultAccountId, setImportDefaultAccountId] = useState<number | null>(null);
  const [importSkipDuplicates, setImportSkipDuplicates] = useState(true);
  const [showTrainPromptAfterImport, setShowTrainPromptAfterImport] = useState(false);
  const [trainingAfterImport, setTrainingAfterImport] = useState(false);

  // Base currency and exchange rates
  const COMMON_CURRENCIES = ["KZT", "USD", "EUR", "RUB"];
  const [baseCurrency, setBaseCurrencyState] = useState<string>("KZT");
  const [exchangeRates, setExchangeRates] = useState<ExchangeRateRow[]>([]);
  const [rateForm, setRateForm] = useState({ from_currency: "USD", to_currency: "KZT", rate: "", date: new Date().toISOString().slice(0, 10) });
  const [addingRate, setAddingRate] = useState(false);

  const loadEmbeddedLlmStatus = async () => {
    try {
      const s = await api.getEmbeddedLlmStatus();
      setEmbeddedLlmStatus(s);
      if (s.downloaded && s.download_progress == null) setEmbeddedDownloading(false);
    } catch {
      setEmbeddedLlmStatus(null);
    }
  };

  const handleDownloadEmbeddedModel = async () => {
    setEmbeddedDownloading(true);
    try {
      const ollamaResult = await api.ensureOllamaInstalled();
      if (ollamaResult === "OpenedDownload") {
        showToast(
          "Открыта страница загрузки Ollama. Установите Ollama, затем снова нажмите «Скачать модель».",
          "info"
        );
        setEmbeddedDownloading(false);
        return;
      }
      await api.downloadAndRegisterEmbeddedModel();
      showToast("Модель скачана и зарегистрирована", "success");
      await loadEmbeddedLlmStatus();
    } catch (e) {
      showToast(String(e) || "Ошибка загрузки", "error");
    } finally {
      setEmbeddedDownloading(false);
    }
  };

  const handleTestEmbeddedLlm = async () => {
    setEmbeddedTestingLlm(true);
    try {
      const result = await api.testEmbeddedLlm();
      showToast(result.message, result.success ? "success" : "error");
      await loadEmbeddedLlmStatus();
    } catch (e) {
      showToast(String(e) || "Ошибка проверки", "error");
    } finally {
      setEmbeddedTestingLlm(false);
    }
  };

  useEffect(() => {
    initTheme();
    loadModelStatus();
    loadBudgets();
    loadCurrencySettings();
    loadEmbeddedLlmStatus();
  }, []);

  useEffect(() => {
    if (embeddedDownloading || (embeddedLlmStatus?.download_progress != null)) {
      const id = setInterval(loadEmbeddedLlmStatus, 800);
      return () => clearInterval(id);
    }
  }, [embeddedDownloading, embeddedLlmStatus?.download_progress]);

  const loadBudgets = async () => {
    try {
      const [b, c, a] = await Promise.all([api.getBudgets(), api.getCategories(), api.getAccounts()]);
      setBudgets(b);
      setCategories(c.filter(cat => cat.category_type === "expense"));
      setAllCategories(c);
      setAccounts(a);
      if (c.length > 0 && budgetForm.category_id === 0) {
        const expenseCats = c.filter(cat => cat.category_type === "expense");
        if (expenseCats.length > 0) {
          setBudgetForm(prev => ({ ...prev, category_id: expenseCats[0].id }));
        }
      }
    } catch (e) {
      console.error("Failed to load budgets:", e);
    }
  };

  const loadCurrencySettings = async () => {
    try {
      const [base, rates] = await Promise.all([api.getBaseCurrency(), api.getExchangeRates()]);
      setBaseCurrencyState(base);
      setExchangeRates(rates);
    } catch (e) {
      console.error("Failed to load currency settings:", e);
    }
  };

  const handleCreateBudget = async (e: React.FormEvent) => {
    e.preventDefault();
    const amount = parseFloat(budgetForm.amount);
    if (isNaN(amount) || amount <= 0) {
      showToast("Введите положительную сумму", "error");
      return;
    }
    if (!budgetForm.category_id || !categories.some((c) => c.id === budgetForm.category_id)) {
      showToast("Выберите категорию", "error");
      return;
    }
    try {
      await api.createBudget({
        category_id: budgetForm.category_id,
        amount,
        period: budgetForm.period,
      });
      showToast("Бюджет создан", "success");
      setShowBudgetForm(false);
      setBudgetForm({ category_id: categories[0]?.id ?? 0, amount: "", period: "monthly" });
      loadBudgets();
    } catch (e) {
      showToast(String(e), "error");
    }
  };

  const handleDeleteBudget = async () => {
    if (deleteBudgetId === null) return;
    try {
      await api.deleteBudget(deleteBudgetId);
      showToast("Бюджет удалён", "success");
      setDeleteBudgetId(null);
      loadBudgets();
    } catch (e) {
      showToast(String(e), "error");
    }
  };

  const handleExport = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      setExporting(true);
      setError(null);
      setExportPath(null);
      const path = await api.exportData({
        format: exportForm.format,
        date_from: exportForm.date_from || null,
        date_to: exportForm.date_to || null,
        include_accounts: exportForm.include_accounts,
        include_categories: exportForm.include_categories,
        account_id: exportForm.account_id || null,
        category_id: exportForm.category_id || null,
      });
      setExportPath(path);
      showToast("Данные экспортированы", "success");
    } catch (e) {
      setError(String(e));
      showToast("Ошибка экспорта", "error");
    } finally {
      setExporting(false);
    }
  };

  const handleImportClick = async () => {
    try {
      setError(null);
      setImportResult(null);
      const selected = await open({
        multiple: false,
        directory: false,
        filters: [
          { name: "CSV", extensions: ["csv"] },
          { name: "JSON", extensions: ["json"] },
          { name: "Excel", extensions: ["xlsx"] },
        ],
      });
      if (selected != null && typeof selected === "string") {
        const path = selected.startsWith("file://") ? decodeURIComponent(selected.slice(7)) : selected;
        const format = path.toLowerCase().endsWith(".json") ? "json" : path.toLowerCase().endsWith(".xlsx") ? "xlsx" : "csv";
        const result = await api.importData({
          path,
          format,
          default_account_id: importDefaultAccountId ?? undefined,
          skip_duplicates: importSkipDuplicates,
        });
        setImportResult(result);
        setShowTrainPromptAfterImport(result.transactions_imported > 0);
        if (result.transactions_imported > 0) {
          showToast(`Импортировано: ${result.transactions_imported} транзакций`, "success");
        }
        if (result.errors.length > 0) {
          showToast(`Ошибок при импорте: ${result.errors.length}`, "warning");
        }
      }
    } catch (e) {
      const msg = getErrorMessage(e);
      setError(msg);
      showToast(msg, "error");
    }
  };

  const loadModelStatus = async () => {
    try {
      setMlLoading(true);
      const status = await api.getModelStatus();
      setModelStatus(status);
    } catch (e) {
      console.error("Failed to load model status:", e);
    } finally {
      setMlLoading(false);
    }
  };

  const handleTrainModel = async () => {
    try {
      setTraining(true);
      setError(null);
      const result = await api.trainModel();
      if (result.success) {
        showToast(result.message, "success");
        loadModelStatus();
      } else {
        showToast(result.message, "error");
      }
    } catch (e) {
      setError(String(e));
      showToast("Ошибка при обучении модели", "error");
    } finally {
      setTraining(false);
    }
  };

  const handleThemeChange = (t: Theme) => {
    setTheme(t);
    setThemeState(t);
  };

  const getErrorMessage = (e: unknown): string => {
    if (e instanceof Error) return e.message;
    if (typeof e === "string") return e;
    if (e && typeof e === "object" && "message" in e && typeof (e as { message: unknown }).message === "string")
      return (e as { message: string }).message;
    return "Неизвестная ошибка";
  };

  const handleBackupExport = async () => {
    try {
      setError(null);
      setBackupExporting(true);
      const path = await api.exportBackup();
      setBackupPath(path);
      showToast("Резервная копия создана", "success");
    } catch (e) {
      const msg = getErrorMessage(e);
      setError(msg);
      showToast(msg, "error");
    } finally {
      setBackupExporting(false);
    }
  };

  /** Открыть папку с файлом в проводнике (для .db нет приложения по умолчанию) */
  const openContainingFolder = (filePath: string) => {
    const i = Math.max(filePath.lastIndexOf("/"), filePath.lastIndexOf("\\"));
    const dir = i > 0 ? filePath.slice(0, i) : filePath;
    handleOpenPath(dir);
  };

  const handleOpenPath = async (path: string) => {
    try {
      await api.openFile(path);
    } catch (e) {
      showToast(getErrorMessage(e), "error");
    }
  };

  const handleRestoreClick = async () => {
    try {
      setError(null);
      const selected = await open({
        multiple: false,
        directory: false,
        filters: [{ name: "База SQLite", extensions: ["db"] }],
      });
      if (selected != null && typeof selected === "string") {
        const path = selected.startsWith("file://") ? decodeURIComponent(selected.slice(7)) : selected;
        setRestoreConfirmPath(path);
      }
    } catch (e) {
      const msg = getErrorMessage(e);
      setError(msg);
      showToast(msg, "error");
    }
  };

  const handleRestoreConfirm = async () => {
    if (!restoreConfirmPath) return;
    try {
      setError(null);
      setRestoreInProgress(true);
      await api.restoreBackup(restoreConfirmPath);
      setRestoreConfirmPath(null);
      showToast("Данные восстановлены. Перезагрузка…", "success");
      window.location.reload();
    } catch (e) {
      const msg = getErrorMessage(e);
      setError(msg);
      showToast(msg, "error");
    } finally {
      setRestoreInProgress(false);
    }
  };

  const handleResetDbConfirm = async () => {
    try {
      setError(null);
      setResetInProgress(true);
      await api.resetDatabase();
      setResetDbConfirm(false);
      showToast("База данных очищена. Перезагрузка…", "success");
      window.location.reload();
    } catch (e) {
      const msg = getErrorMessage(e);
      setError(msg);
      showToast(msg, "error");
    } finally {
      setResetInProgress(false);
    }
  };

  return (
    <div className="space-y-8 max-w-xl">
      <SettingsTheme theme={theme} onThemeChange={handleThemeChange} />

      <SettingsBackup
        error={error}
        backupPath={backupPath}
        backupExporting={backupExporting}
        onBackupExport={handleBackupExport}
        onRestoreClick={handleRestoreClick}
        onResetClick={() => setResetDbConfirm(true)}
        openContainingFolder={openContainingFolder}
      />

      {/* Base currency and exchange rates */}
      <div>
        <h3 className="text-lg font-medium mb-4 flex items-center gap-2">
          <DollarSign size={20} className="text-emerald-500" />
          Валюта и курсы
        </h3>
        <p className="text-sm text-zinc-400 mb-4">
          Базовая валюта используется для общего баланса и отчётов. Добавьте курсы для конвертации (например, 1 USD = 450 KZT).
        </p>
        <div className="p-4 rounded-xl bg-zinc-100 dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 space-y-4 mb-4">
          <div>
            <label className="block text-xs text-zinc-400 mb-1">Базовая валюта</label>
            <select
              value={baseCurrency}
              onChange={async (e) => {
                const v = e.target.value;
                try {
                  await api.setBaseCurrency(v);
                  setBaseCurrencyState(v);
                  showToast("Базовая валюта сохранена", "success");
                } catch (e) {
                  showToast(getErrorMessage(e), "error");
                }
              }}
              className="px-3 py-2 rounded-lg bg-white dark:bg-zinc-700 border border-zinc-300 dark:border-zinc-600 text-zinc-900 dark:text-white text-sm"
            >
              {COMMON_CURRENCIES.map((c) => (
                <option key={c} value={c}>{c}</option>
              ))}
            </select>
          </div>
          <div>
            <span className="block text-xs text-zinc-400 mb-2">Курсы валют (последние по дате)</span>
            {exchangeRates.length === 0 ? (
              <p className="text-sm text-zinc-500 dark:text-zinc-400">Нет добавленных курсов. Добавьте курс ниже.</p>
            ) : (
              <ul className="text-sm space-y-1 max-h-32 overflow-auto">
                {exchangeRates.map((r, i) => (
                  <li key={i} className="text-zinc-700 dark:text-zinc-300">
                    1 {r.from_currency} = {Number(r.rate).toFixed(4)} {r.to_currency} ({r.date})
                  </li>
                ))}
              </ul>
            )}
          </div>
          <form
            onSubmit={async (e) => {
              e.preventDefault();
              const rate = parseFloat(rateForm.rate);
              if (rateForm.from_currency === rateForm.to_currency || !Number.isFinite(rate) || rate <= 0) {
                showToast("Укажите разные валюты и положительный курс", "error");
                return;
              }
              try {
                setAddingRate(true);
                await api.addExchangeRate({
                  from_currency: rateForm.from_currency,
                  to_currency: rateForm.to_currency,
                  rate,
                  date: rateForm.date,
                });
                showToast("Курс добавлен", "success");
                setRateForm(prev => ({ ...prev, rate: "" }));
                loadCurrencySettings();
              } catch (err) {
                showToast(getErrorMessage(err), "error");
              } finally {
                setAddingRate(false);
              }
            }}
            className="flex flex-wrap items-end gap-3"
          >
            <div>
              <label className="block text-xs text-zinc-400 mb-1">Из</label>
              <select
                value={rateForm.from_currency}
                onChange={(e) => setRateForm(f => ({ ...f, from_currency: e.target.value }))}
                className="px-3 py-1.5 rounded-lg bg-white dark:bg-zinc-700 border border-zinc-300 dark:border-zinc-600 text-sm"
              >
                {COMMON_CURRENCIES.map((c) => (
                  <option key={c} value={c}>{c}</option>
                ))}
              </select>
            </div>
            <div>
              <label className="block text-xs text-zinc-400 mb-1">В</label>
              <select
                value={rateForm.to_currency}
                onChange={(e) => setRateForm(f => ({ ...f, to_currency: e.target.value }))}
                className="px-3 py-1.5 rounded-lg bg-white dark:bg-zinc-700 border border-zinc-300 dark:border-zinc-600 text-sm"
              >
                {COMMON_CURRENCIES.map((c) => (
                  <option key={c} value={c}>{c}</option>
                ))}
              </select>
            </div>
            <div>
              <label className="block text-xs text-zinc-400 mb-1">Курс (1 из = N в)</label>
              <input
                type="number"
                step="any"
                min="0"
                value={rateForm.rate}
                onChange={(e) => setRateForm(f => ({ ...f, rate: e.target.value }))}
                placeholder="450"
                className="w-24 px-3 py-1.5 rounded-lg bg-white dark:bg-zinc-700 border border-zinc-300 dark:border-zinc-600 text-sm"
              />
            </div>
            <div>
              <label className="block text-xs text-zinc-400 mb-1">Дата</label>
              <input
                type="date"
                value={rateForm.date}
                onChange={(e) => setRateForm(f => ({ ...f, date: e.target.value }))}
                className="px-3 py-1.5 rounded-lg bg-white dark:bg-zinc-700 border border-zinc-300 dark:border-zinc-600 text-sm"
              />
            </div>
            <button
              type="submit"
              disabled={addingRate}
              className="px-4 py-2 rounded-lg bg-emerald-500 text-white hover:bg-emerald-600 disabled:opacity-50 text-sm"
            >
              {addingRate ? "Добавление…" : "Добавить курс"}
            </button>
          </form>
        </div>
      </div>

      {/* Export/Import Section */}
      <div>
        <h3 className="text-lg font-medium mb-4 flex items-center gap-2">
          <FileDown size={20} className="text-blue-500" />
          Экспорт и импорт данных
        </h3>
        <p className="text-sm text-zinc-400 mb-4">
          Экспорт: выберите формат (Excel, CSV, JSON) и при необходимости фильтры — файл сохранится и можно открыть папку. Импорт: нажмите «Импорт», выберите файл CSV или JSON (лучше всего — ранее экспортированный из этого приложения).
        </p>

        {exportPath && (
          <div className="p-4 rounded-lg bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 mb-4">
            <div className="flex items-center justify-between gap-4">
              <span className="text-sm break-all">Файл сохранён: {exportPath}</span>
              <button
                onClick={() => openContainingFolder(exportPath)}
                className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-emerald-600 text-white hover:bg-emerald-700 btn-transition text-sm shrink-0"
              >
                <ExternalLink size={14} />
                Открыть папку
              </button>
            </div>
          </div>
        )}

        {/* Настройки импорта */}
        <div className="flex flex-wrap items-center gap-4 mb-4 p-3 rounded-lg bg-zinc-100 dark:bg-zinc-800/50 border border-zinc-200 dark:border-zinc-700">
          <span className="text-sm text-zinc-500 dark:text-zinc-400">При импорте:</span>
          <div className="flex items-center gap-2">
            <label className="text-sm text-zinc-600 dark:text-zinc-300">Счёт по умолчанию</label>
            <select
              value={importDefaultAccountId ?? ""}
              onChange={(e) => setImportDefaultAccountId(e.target.value ? Number(e.target.value) : null)}
              className="px-3 py-1.5 rounded-lg bg-white dark:bg-zinc-700 border border-zinc-300 dark:border-zinc-600 text-zinc-900 dark:text-white text-sm"
            >
              <option value="">Не выбран</option>
              {accounts.map((acc) => (
                <option key={acc.id} value={acc.id}>{acc.name}</option>
              ))}
            </select>
          </div>
          <label className="flex items-center gap-2 text-sm text-zinc-600 dark:text-zinc-300 cursor-pointer">
            <input
              type="checkbox"
              checked={importSkipDuplicates}
              onChange={(e) => setImportSkipDuplicates(e.target.checked)}
              className="w-4 h-4 rounded"
            />
            Пропускать дубликаты
          </label>
        </div>

        {importResult && (
          <div className={`p-4 rounded-lg border mb-4 ${
            importResult.errors.length > 0 
              ? "bg-amber-500/10 text-amber-400 border-amber-500/20" 
              : "bg-emerald-500/10 text-emerald-400 border-emerald-500/20"
          }`}>
            <p>Импортировано транзакций: {importResult.transactions_imported}</p>
            {(importResult.duplicates_skipped ?? 0) > 0 && (
              <p className="text-sm mt-1">Пропущено дубликатов: {importResult.duplicates_skipped}</p>
            )}
            {(importResult.total_parsed ?? 0) > 0 && (
              <p className="text-sm text-zinc-500 dark:text-zinc-400 mt-1">Обработано строк: {importResult.total_parsed}</p>
            )}
            {importResult.errors.length > 0 && (
              <details className="mt-2">
                <summary className="cursor-pointer text-sm">Ошибки ({importResult.errors.length})</summary>
                <ul className="mt-2 text-xs space-y-1 max-h-32 overflow-auto">
                  {importResult.errors.slice(0, 10).map((err, i) => (
                    <li key={i}>{err}</li>
                  ))}
                  {importResult.errors.length > 10 && (
                    <li>...и ещё {importResult.errors.length - 10} ошибок</li>
                  )}
                </ul>
              </details>
            )}
          </div>
        )}

        {showTrainPromptAfterImport && importResult && importResult.transactions_imported > 0 && (
          <div className="p-4 rounded-lg border border-zinc-200 dark:border-zinc-700 bg-zinc-50 dark:bg-zinc-800/50 mb-4">
            <p className="text-zinc-700 dark:text-zinc-300 mb-3">Обучить модель по новым данным?</p>
            <div className="flex gap-3">
              <button
                type="button"
                onClick={async () => {
                  try {
                    setTrainingAfterImport(true);
                    const res = await api.trainModel();
                    showToast(res.message || "Модель обучена", res.success ? "success" : "info");
                    setShowTrainPromptAfterImport(false);
                  } catch {
                    showToast("Ошибка обучения модели", "error");
                  } finally {
                    setTrainingAfterImport(false);
                  }
                }}
                disabled={trainingAfterImport}
                className="px-4 py-2 rounded-lg bg-emerald-500 text-white hover:bg-emerald-600 disabled:opacity-50"
              >
                {trainingAfterImport ? "Обучение…" : "Обучить"}
              </button>
              <button
                type="button"
                onClick={() => setShowTrainPromptAfterImport(false)}
                className="px-4 py-2 rounded-lg bg-zinc-200 dark:bg-zinc-700 text-zinc-700 dark:text-zinc-300 hover:bg-zinc-300 dark:hover:bg-zinc-600"
              >
                Позже
              </button>
            </div>
          </div>
        )}

        {showExportForm ? (
          <form onSubmit={handleExport} className="p-4 rounded-xl bg-zinc-100 dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 space-y-4 mb-4">
            <div>
              <label className="block text-xs text-zinc-400 mb-1">Формат</label>
              <div className="flex gap-2">
                <button
                  type="button"
                  onClick={() => setExportForm(f => ({ ...f, format: "xlsx" }))}
                  className={`flex-1 flex items-center justify-center gap-2 px-4 py-2 rounded-lg border btn-transition ${
                    exportForm.format === "xlsx"
                      ? "bg-emerald-500/10 border-emerald-500/30 text-emerald-500"
                      : "bg-zinc-100 dark:bg-zinc-700 border-zinc-300 dark:border-zinc-600 text-zinc-600 dark:text-zinc-400"
                  }`}
                >
                  <FileSpreadsheet size={16} />
                  Excel
                </button>
                <button
                  type="button"
                  onClick={() => setExportForm(f => ({ ...f, format: "csv" }))}
                  className={`flex-1 flex items-center justify-center gap-2 px-4 py-2 rounded-lg border btn-transition ${
                    exportForm.format === "csv"
                      ? "bg-blue-500/10 border-blue-500/30 text-blue-500"
                      : "bg-zinc-100 dark:bg-zinc-700 border-zinc-300 dark:border-zinc-600 text-zinc-600 dark:text-zinc-400"
                  }`}
                >
                  <FileText size={16} />
                  CSV
                </button>
                <button
                  type="button"
                  onClick={() => setExportForm(f => ({ ...f, format: "json" }))}
                  className={`flex-1 flex items-center justify-center gap-2 px-4 py-2 rounded-lg border btn-transition ${
                    exportForm.format === "json"
                      ? "bg-blue-500/10 border-blue-500/30 text-blue-500"
                      : "bg-zinc-100 dark:bg-zinc-700 border-zinc-300 dark:border-zinc-600 text-zinc-600 dark:text-zinc-400"
                  }`}
                >
                  <FileJson size={16} />
                  JSON
                </button>
              </div>
            </div>

            <div className="grid gap-4 sm:grid-cols-2">
              <div>
                <label className="block text-xs text-zinc-400 mb-1">Счёт (все)</label>
                <select
                  value={exportForm.account_id}
                  onChange={(e) => setExportForm(f => ({ ...f, account_id: Number(e.target.value) }))}
                  className="w-full px-3 py-2 rounded-lg bg-white dark:bg-zinc-700 border border-zinc-300 dark:border-zinc-600 text-zinc-900 dark:text-white text-sm"
                >
                  <option value={0}>Все счета</option>
                  {accounts.map((acc) => (
                    <option key={acc.id} value={acc.id}>{acc.name}</option>
                  ))}
                </select>
              </div>
              <div>
                <label className="block text-xs text-zinc-400 mb-1">Категория (все)</label>
                <select
                  value={exportForm.category_id}
                  onChange={(e) => setExportForm(f => ({ ...f, category_id: Number(e.target.value) }))}
                  className="w-full px-3 py-2 rounded-lg bg-white dark:bg-zinc-700 border border-zinc-300 dark:border-zinc-600 text-zinc-900 dark:text-white text-sm"
                >
                  <option value={0}>Все категории</option>
                  {allCategories.map((cat) => (
                    <option key={cat.id} value={cat.id}>{cat.name}</option>
                  ))}
                </select>
              </div>
              <div>
                <label className="block text-xs text-zinc-400 mb-1">Дата от (опц.)</label>
                <input
                  type="date"
                  value={exportForm.date_from}
                  onChange={(e) => setExportForm(f => ({ ...f, date_from: e.target.value }))}
                  className="w-full px-3 py-2 rounded-lg bg-white dark:bg-zinc-700 border border-zinc-300 dark:border-zinc-600 text-zinc-900 dark:text-white text-sm"
                />
              </div>
              <div>
                <label className="block text-xs text-zinc-400 mb-1">Дата до (опц.)</label>
                <input
                  type="date"
                  value={exportForm.date_to}
                  onChange={(e) => setExportForm(f => ({ ...f, date_to: e.target.value }))}
                  className="w-full px-3 py-2 rounded-lg bg-white dark:bg-zinc-700 border border-zinc-300 dark:border-zinc-600 text-zinc-900 dark:text-white text-sm"
                />
              </div>
            </div>

            {exportForm.format === "json" && (
              <div className="flex flex-wrap gap-4">
                <label className="flex items-center gap-2 text-sm text-zinc-600 dark:text-zinc-300">
                  <input
                    type="checkbox"
                    checked={exportForm.include_accounts}
                    onChange={(e) => setExportForm(f => ({ ...f, include_accounts: e.target.checked }))}
                    className="w-4 h-4 rounded"
                  />
                  Включить счета
                </label>
                <label className="flex items-center gap-2 text-sm text-zinc-600 dark:text-zinc-300">
                  <input
                    type="checkbox"
                    checked={exportForm.include_categories}
                    onChange={(e) => setExportForm(f => ({ ...f, include_categories: e.target.checked }))}
                    className="w-4 h-4 rounded"
                  />
                  Включить категории
                </label>
              </div>
            )}

            <div className="flex gap-2">
              <button
                type="submit"
                disabled={exporting}
                className="flex items-center gap-2 px-4 py-2 rounded-lg bg-blue-600 text-white hover:bg-blue-700 btn-transition disabled:opacity-50"
              >
                <FileDown size={18} className={exporting ? "animate-pulse" : ""} />
                {exporting ? "Экспорт..." : "Экспортировать"}
              </button>
              <button
                type="button"
                onClick={() => setShowExportForm(false)}
                className="px-4 py-2 rounded-lg bg-zinc-600 text-white hover:bg-zinc-500 btn-transition"
              >
                Отмена
              </button>
            </div>
          </form>
        ) : (
          <div className="flex gap-2">
            <button
              onClick={() => setShowExportForm(true)}
              className="flex items-center gap-2 px-4 py-2 rounded-lg bg-blue-600 text-white hover:bg-blue-700 transition-colors"
            >
              <FileDown size={18} />
              Экспорт
            </button>
            <button
              onClick={handleImportClick}
              className="flex items-center gap-2 px-4 py-2 rounded-lg bg-zinc-600 text-white hover:bg-zinc-500 transition-colors"
            >
              <FileUp size={18} />
              Импорт
            </button>
          </div>
        )}
      </div>

      {/* ML Section */}
      <div>
        <h3 className="text-lg font-medium mb-4 flex items-center gap-2">
          <Brain size={20} className="text-purple-500" />
          Машинное обучение
        </h3>
        <p className="text-sm text-zinc-400 mb-4">
          Модель предсказывает категории для транзакций на основе их описания. Чем больше транзакций с заполненными категориями и заметками, тем точнее предсказания.
        </p>

        <div className="p-4 rounded-xl bg-zinc-100 dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 space-y-4">
          {mlLoading ? (
            <div className="flex items-center gap-2 text-zinc-400">
              <RefreshCw size={16} className="animate-spin" />
              <span>Загрузка статуса...</span>
            </div>
          ) : modelStatus ? (
            <>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <span className="text-xs text-zinc-400 block mb-1">Статус модели</span>
                  <div className="flex items-center gap-2">
                    {modelStatus.trained ? (
                      <>
                        <CheckCircle size={16} className="text-emerald-500" />
                        <span className="text-emerald-500 font-medium">Обучена</span>
                      </>
                    ) : (
                      <>
                        <XCircle size={16} className="text-zinc-400" />
                        <span className="text-zinc-400">Не обучена</span>
                      </>
                    )}
                  </div>
                </div>
                {modelStatus.trained && modelStatus.trained_at && (
                  <div>
                    <span className="text-xs text-zinc-400 block mb-1">Дата обучения</span>
                    <span className="text-zinc-600 dark:text-zinc-300">{modelStatus.trained_at}</span>
                  </div>
                )}
                {modelStatus.sample_count !== null && modelStatus.sample_count > 0 && (
                  <div>
                    <span className="text-xs text-zinc-400 block mb-1">Транзакций в модели</span>
                    <span className="text-zinc-600 dark:text-zinc-300">{modelStatus.sample_count}</span>
                  </div>
                )}
                {modelStatus.accuracy !== null && (
                  <div>
                    <span className="text-xs text-zinc-400 block mb-1">Точность</span>
                    <span className="text-zinc-600 dark:text-zinc-300">~{Math.round(modelStatus.accuracy * 100)}%</span>
                  </div>
                )}
              </div>

              <div className="pt-2 border-t border-zinc-200 dark:border-zinc-700 space-y-2">
                <div>
                  <label className="text-xs text-zinc-400 block mb-1">Порог уверенности</label>
                  <div className="flex items-center gap-3">
                    <input
                      type="range"
                      min={0.2}
                      max={0.9}
                      step={0.05}
                      value={confidenceThreshold}
                      onChange={(e) => {
                        const v = parseFloat(e.target.value);
                        setConfidenceThreshold(v);
                        try {
                          localStorage.setItem(ML_THRESHOLD_KEY, String(v));
                        } catch {}
                      }}
                      className="flex-1 h-2 rounded-lg appearance-none bg-zinc-200 dark:bg-zinc-600 accent-purple-500"
                    />
                    <span className="text-sm text-zinc-600 dark:text-zinc-300 w-10">
                      {Math.round(confidenceThreshold * 100)}%
                    </span>
                  </div>
                  <p className="text-xs text-zinc-400 mt-1">
                    Чем выше — тем реже показываются подсказки, но они надёжнее
                  </p>
                </div>
                <button
                  onClick={handleTrainModel}
                  disabled={training}
                  className={`flex items-center gap-2 px-4 py-2 rounded-lg text-white transition-colors ${
                    training
                      ? "bg-purple-600/50 cursor-not-allowed"
                      : "bg-purple-600 hover:bg-purple-700"
                  }`}
                >
                  <RefreshCw size={18} className={training ? "animate-spin" : ""} />
                  {training ? "Обучение..." : modelStatus.trained ? "Переобучить модель" : "Обучить модель"}
                </button>
                <p className="text-xs text-zinc-400 mt-2">
                  Требуется минимум 20 транзакций с заполненными категориями и заметками
                </p>
                {modelStatus.transactions_with_categories_count != null &&
                  modelStatus.transactions_with_categories_count < 20 &&
                  (modelStatus.transactions_with_note_no_category ?? 0) > 0 && (
                    <p className="text-sm text-amber-600 dark:text-amber-400 mt-2">
                      У вас {modelStatus.transactions_with_note_no_category} транзакций с описанием без категории. Назначьте категории хотя бы 20 из них в разделе «Транзакции», затем обучите модель.
                    </p>
                  )}
                {modelStatus.trained &&
                  modelStatus.sample_count != null &&
                  modelStatus.transactions_with_categories_count != null &&
                  modelStatus.transactions_with_categories_count - modelStatus.sample_count > 50 && (
                    <p className="text-sm text-amber-600 dark:text-amber-400 mt-2">
                      Добавлено много новых транзакций. Рекомендуем переобучить модель для лучших предсказаний.
                    </p>
                  )}
              </div>
            </>
          ) : (
            <div className="text-zinc-400">Не удалось загрузить статус модели</div>
          )}
        </div>

        {/* LLM — подсказки категорий: встроенная модель или Ollama вручную */}
        <div className="mt-6 p-4 rounded-xl bg-zinc-100 dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 space-y-4">
          <h4 className="font-medium text-zinc-800 dark:text-zinc-200">Подсказки категорий (LLM)</h4>
          <p className="text-sm text-zinc-400">
            Поставьте галочку, выберите «Встроенная модель» и нажмите «Скачать модель» — Ollama установится автоматически (если ещё не установлен), затем скачается и настроится модель (~390 МБ).
          </p>
          <label className="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              checked={llmEnabled}
              onChange={(e) => {
                const v = e.target.checked;
                setLlmEnabled(v);
                localStorage.setItem(LLM_ENABLED_KEY, v ? "true" : "false");
              }}
              className="rounded border-zinc-300 dark:border-zinc-600 text-purple-600 focus:ring-purple-500"
            />
            <span className="text-sm text-zinc-700 dark:text-zinc-300">Использовать LLM для подсказки категорий</span>
          </label>
          {llmEnabled && (
            <>
              <div className="flex flex-col gap-3">
                <label className="flex items-center gap-2 cursor-pointer">
                  <input
                    type="radio"
                    name="llm_source"
                    checked={useEmbeddedLlm}
                    onChange={() => {
                      setUseEmbeddedLlm(true);
                      localStorage.setItem(LLM_USE_EMBEDDED_KEY, "true");
                    }}
                    className="text-purple-600 focus:ring-purple-500"
                  />
                  <span className="text-sm text-zinc-700 dark:text-zinc-300">Встроенная модель</span>
                </label>
                {useEmbeddedLlm && embeddedLlmStatus && (
                  <div className="pl-6 text-sm space-y-2">
                    {embeddedLlmStatus.error && (
                      <p className="text-amber-600 dark:text-amber-400">{embeddedLlmStatus.error}</p>
                    )}
                    {embeddedLlmStatus.downloaded && embeddedLlmStatus.registered_in_ollama && embeddedLlmStatus.ollama_reachable && !embeddedLlmStatus.error && (
                      <p className="text-emerald-600 dark:text-emerald-400">Модель готова к работе.</p>
                    )}
                    {embeddedLlmStatus.download_progress != null && (
                      <p className="text-zinc-500">Скачивание: {embeddedLlmStatus.download_progress}%</p>
                    )}
                    {!embeddedLlmStatus.downloaded && (
                      <p className="text-zinc-500">Нажмите «Скачать модель» — Ollama установится при необходимости, затем загрузится модель (~390 МБ). Запустите Ollama перед использованием.</p>
                    )}
                    {embeddedLlmStatus.downloaded && !embeddedLlmStatus.ollama_reachable && !embeddedLlmStatus.error && (
                      <p className="text-zinc-500">Модель скачана. Запустите Ollama и нажмите «Проверить».</p>
                    )}
                    {(!embeddedLlmStatus.downloaded || !embeddedLlmStatus.registered_in_ollama) && (
                      <button
                        type="button"
                        onClick={handleDownloadEmbeddedModel}
                        disabled={embeddedDownloading}
                        className="px-3 py-1.5 rounded-lg bg-purple-600 text-white text-sm hover:bg-purple-700 disabled:opacity-50"
                      >
                        {embeddedDownloading ? "Скачивание…" : "Скачать модель"}
                      </button>
                    )}
                    {(embeddedLlmStatus.downloaded && embeddedLlmStatus.registered_in_ollama) && (
                      <button
                        type="button"
                        onClick={handleTestEmbeddedLlm}
                        disabled={embeddedTestingLlm}
                        className="ml-2 px-3 py-1.5 rounded-lg bg-zinc-600 text-white text-sm hover:bg-zinc-500 disabled:opacity-50"
                      >
                        {embeddedTestingLlm ? "Проверка…" : "Проверить"}
                      </button>
                    )}
                  </div>
                )}
                <label className="flex items-center gap-2 cursor-pointer">
                  <input
                    type="radio"
                    name="llm_source"
                    checked={!useEmbeddedLlm}
                    onChange={() => {
                      setUseEmbeddedLlm(false);
                      localStorage.setItem(LLM_USE_EMBEDDED_KEY, "false");
                    }}
                    className="text-purple-600 focus:ring-purple-500"
                  />
                  <span className="text-sm text-zinc-700 dark:text-zinc-300">Ollama вручную (URL и модель)</span>
                </label>
                {!useEmbeddedLlm && (
                  <div className="pl-6 grid gap-3 sm:grid-cols-2">
                    <div>
                      <label className="text-xs text-zinc-400 block mb-1">URL Ollama</label>
                      <input
                        type="text"
                        value={ollamaUrl}
                        onChange={(e) => {
                          setOllamaUrl(e.target.value);
                          localStorage.setItem(OLLAMA_URL_KEY, e.target.value);
                        }}
                        placeholder="http://127.0.0.1:11434"
                        className="w-full px-3 py-2 rounded-lg border border-zinc-200 dark:border-zinc-600 bg-white dark:bg-zinc-700 text-zinc-900 dark:text-zinc-100 text-sm"
                      />
                    </div>
                    <div>
                      <label className="text-xs text-zinc-400 block mb-1">Модель</label>
                      <input
                        type="text"
                        value={ollamaModel}
                        onChange={(e) => {
                          setOllamaModel(e.target.value);
                          localStorage.setItem(OLLAMA_MODEL_KEY, e.target.value);
                        }}
                        placeholder="llama3.2"
                        className="w-full px-3 py-2 rounded-lg border border-zinc-200 dark:border-zinc-600 bg-white dark:bg-zinc-700 text-zinc-900 dark:text-zinc-100 text-sm"
                      />
                    </div>
                  </div>
                )}
              </div>
            </>
          )}
        </div>
      </div>

      {/* Budget Section */}
      <div>
        <h3 className="text-lg font-medium mb-4 flex items-center gap-2">
          <PiggyBank size={20} className="text-emerald-500" />
          Бюджеты по категориям
        </h3>
        <p className="text-sm text-zinc-400 mb-4">
          Установите лимиты расходов по категориям. Вы получите уведомления при достижении 80% и 100% бюджета.
        </p>

        {/* Budget List */}
        {budgets.length > 0 && (
          <div className="space-y-3 mb-4">
            {budgets.map((budget) => (
              <div
                key={budget.id}
                className="p-4 rounded-xl bg-zinc-100 dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700"
              >
                <div className="flex justify-between items-start mb-2">
                  <div>
                    <span className="font-medium">{budget.category_name}</span>
                    <span className="text-xs text-zinc-400 ml-2">
                      {budget.period === "monthly" ? "в месяц" : budget.period === "weekly" ? "в неделю" : "в год"}
                    </span>
                  </div>
                  <button
                    onClick={() => setDeleteBudgetId(budget.id)}
                    className="p-1 rounded text-zinc-400 hover:bg-red-500/20 hover:text-red-500 btn-transition"
                  >
                    <Trash2 size={14} />
                  </button>
                </div>
                
                {/* Progress bar */}
                <div className="relative h-2 rounded-full bg-zinc-200 dark:bg-zinc-700 overflow-hidden mb-2">
                  <div
                    className={`absolute left-0 top-0 h-full rounded-full transition-all ${
                      budget.percent_used >= 100
                        ? "bg-red-500"
                        : budget.percent_used >= 80
                        ? "bg-amber-500"
                        : "bg-emerald-500"
                    }`}
                    style={{ width: `${Math.min(budget.percent_used, 100)}%` }}
                  />
                </div>
                
                <div className="flex justify-between text-sm">
                  <span className="text-zinc-500 dark:text-zinc-400">
                    {new Intl.NumberFormat("ru-KZ").format(budget.spent)} ₸ / {new Intl.NumberFormat("ru-KZ").format(budget.amount)} ₸
                  </span>
                  <span className={`font-medium ${
                    budget.percent_used >= 100
                      ? "text-red-500"
                      : budget.percent_used >= 80
                      ? "text-amber-500"
                      : "text-emerald-500"
                  }`}>
                    {Math.round(budget.percent_used)}%
                  </span>
                </div>
              </div>
            ))}
          </div>
        )}

        {/* Budget Form */}
        {showBudgetForm ? (
          <form onSubmit={handleCreateBudget} className="p-4 rounded-xl bg-zinc-100 dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 space-y-4">
            <div className="grid gap-4 sm:grid-cols-3">
              <div>
                <label className="block text-xs text-zinc-400 mb-1">Категория</label>
                <select
                  value={budgetForm.category_id}
                  onChange={(e) => setBudgetForm((f) => ({ ...f, category_id: +e.target.value }))}
                  className="w-full px-3 py-2 rounded-lg bg-white dark:bg-zinc-700 border border-zinc-300 dark:border-zinc-600 text-zinc-900 dark:text-white text-sm"
                  required
                >
                  {categories.map((c) => (
                    <option key={c.id} value={c.id}>{c.name}</option>
                  ))}
                </select>
              </div>
              <div>
                <label className="block text-xs text-zinc-400 mb-1">Лимит (₸)</label>
                <input
                  type="number"
                  min="0"
                  step="100"
                  value={budgetForm.amount}
                  onChange={(e) => setBudgetForm((f) => ({ ...f, amount: e.target.value }))}
                  className="w-full px-3 py-2 rounded-lg bg-white dark:bg-zinc-700 border border-zinc-300 dark:border-zinc-600 text-zinc-900 dark:text-white text-sm"
                  placeholder="50000"
                  required
                />
              </div>
              <div>
                <label className="block text-xs text-zinc-400 mb-1">Период</label>
                <select
                  value={budgetForm.period}
                  onChange={(e) => setBudgetForm((f) => ({ ...f, period: e.target.value }))}
                  className="w-full px-3 py-2 rounded-lg bg-white dark:bg-zinc-700 border border-zinc-300 dark:border-zinc-600 text-zinc-900 dark:text-white text-sm"
                >
                  <option value="weekly">Еженедельно</option>
                  <option value="monthly">Ежемесячно</option>
                  <option value="yearly">Ежегодно</option>
                </select>
              </div>
            </div>
            <div className="flex gap-2">
              <button
                type="submit"
                className="px-4 py-2 rounded-lg bg-emerald-600 text-white hover:bg-emerald-700 btn-transition text-sm"
              >
                Создать
              </button>
              <button
                type="button"
                onClick={() => setShowBudgetForm(false)}
                className="px-4 py-2 rounded-lg bg-zinc-600 text-white hover:bg-zinc-500 btn-transition text-sm"
              >
                Отмена
              </button>
            </div>
          </form>
        ) : (
          <button
            type="button"
            onClick={() => setShowBudgetForm(true)}
            disabled={categories.length === 0}
            className="flex items-center gap-2 px-4 py-2 rounded-lg bg-emerald-600 text-white hover:bg-emerald-700 btn-transition disabled:opacity-50 disabled:pointer-events-none"
          >
            <Plus size={18} />
            Добавить бюджет
          </button>
        )}
        {categories.length === 0 && (
          <p className="text-sm text-zinc-500 dark:text-zinc-400 mt-2">
            Сначала добавьте расходные категории в разделе «Категории».
          </p>
        )}
      </div>

      <ConfirmDialog
        open={restoreConfirmPath !== null}
        title="Восстановить из копии?"
        message="Убедитесь, что бэкап создан этим приложением. Иначе данные могут не открыться. Текущие данные будут заменены. Продолжить?"
        confirmLabel="Восстановить"
        variant="danger"
        loading={restoreInProgress}
        loadingConfirmLabel="Восстановление…"
        onConfirm={handleRestoreConfirm}
        onCancel={() => !restoreInProgress && setRestoreConfirmPath(null)}
      />

      <ConfirmDialog
        open={resetDbConfirm}
        title="Сбросить базу данных?"
        message="Будут удалены все счета, транзакции, категории, бюджеты и правила текущего пользователя. Останутся только категории по умолчанию. Это действие необратимо."
        confirmLabel="Сбросить"
        variant="danger"
        loading={resetInProgress}
        loadingConfirmLabel="Сброс…"
        onConfirm={handleResetDbConfirm}
        onCancel={() => !resetInProgress && setResetDbConfirm(false)}
      />

      <ConfirmDialog
        open={deleteBudgetId !== null}
        title="Удалить бюджет?"
        message="Эта операция необратима."
        confirmLabel="Удалить"
        variant="danger"
        onConfirm={handleDeleteBudget}
        onCancel={() => setDeleteBudgetId(null)}
      />
    </div>
  );
}
