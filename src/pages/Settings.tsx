import { useState, useEffect } from "react";
import { Moon, Sun, Download, Upload, Brain, RefreshCw, CheckCircle, XCircle, PiggyBank, Plus, Trash2, FileDown, FileUp, FileJson, FileSpreadsheet, ExternalLink, FileText } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import { getTheme, setTheme, initTheme, type Theme } from "../stores/themeStore";
import { api, type ModelStatus, type Budget, type Category, type ImportResult } from "../lib/api";
import { ConfirmDialog } from "../components/ui/ConfirmDialog";
import { useToast } from "../components/ui/Toast";

export function Settings() {
  const { showToast } = useToast();
  const [theme, setThemeState] = useState<Theme>(getTheme());
  const [backupPath, setBackupPath] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [restoreConfirmPath, setRestoreConfirmPath] = useState<string | null>(null);

  // ML State
  const [modelStatus, setModelStatus] = useState<ModelStatus | null>(null);
  const [mlLoading, setMlLoading] = useState(true);
  const [training, setTraining] = useState(false);

  // Budget State
  const [budgets, setBudgets] = useState<Budget[]>([]);
  const [categories, setCategories] = useState<Category[]>([]);
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
  });
  const [exporting, setExporting] = useState(false);
  const [exportPath, setExportPath] = useState<string | null>(null);
  const [importResult, setImportResult] = useState<ImportResult | null>(null);

  useEffect(() => {
    initTheme();
    loadModelStatus();
    loadBudgets();
  }, []);

  const loadBudgets = async () => {
    try {
      const [b, c] = await Promise.all([api.getBudgets(), api.getCategories()]);
      setBudgets(b);
      setCategories(c.filter(cat => cat.category_type === "expense"));
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

  const handleCreateBudget = async (e: React.FormEvent) => {
    e.preventDefault();
    const amount = parseFloat(budgetForm.amount);
    if (isNaN(amount) || amount <= 0) return;
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
          { name: "CSV/JSON", extensions: ["csv", "json"] },
        ],
      });
      if (selected && typeof selected === "string") {
        const format = selected.toLowerCase().endsWith(".json") ? "json" : "csv";
        const result = await api.importData({ path: selected, format });
        setImportResult(result);
        if (result.transactions_imported > 0) {
          showToast(`Импортировано: ${result.transactions_imported} транзакций`, "success");
        }
        if (result.errors.length > 0) {
          showToast(`Ошибок при импорте: ${result.errors.length}`, "warning");
        }
      }
    } catch (e) {
      setError(String(e));
      showToast("Ошибка импорта", "error");
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

  const handleBackupExport = async () => {
    try {
      setError(null);
      const path = await api.exportBackup();
      setBackupPath(path);
      showToast("Резервная копия создана", "success");
    } catch (e) {
      setError(String(e));
      showToast("Ошибка при создании копии", "error");
    }
  };

  const handleOpenFile = async (path: string) => {
    try {
      await api.openFile(path);
    } catch (e) {
      showToast(String(e), "error");
    }
  };

  const handleRestoreClick = async () => {
    try {
      setError(null);
      const selected = await open({
        multiple: false,
        directory: false,
        filters: [{ name: "SQLite", extensions: ["db"] }],
      });
      if (selected && typeof selected === "string") {
        setRestoreConfirmPath(selected);
      }
    } catch (e) {
      setError(String(e));
    }
  };

  const handleRestoreConfirm = async () => {
    if (!restoreConfirmPath) return;
    try {
      setError(null);
      await api.restoreBackup(restoreConfirmPath);
      setRestoreConfirmPath(null);
      window.location.reload();
    } catch (e) {
      setError(String(e));
    }
  };

  return (
    <div className="space-y-8 max-w-xl">
      <div>
        <h3 className="text-lg font-medium mb-4">Тема</h3>
        <div className="flex gap-4">
          <button
            onClick={() => handleThemeChange("dark")}
            className={`flex items-center gap-2 px-4 py-3 rounded-xl border transition-colors ${
              theme === "dark"
                ? "bg-zinc-700 dark:bg-zinc-700 border-zinc-600 text-white"
                : "bg-zinc-100 dark:bg-zinc-800 border-zinc-300 dark:border-zinc-700 text-zinc-700 dark:text-zinc-400 hover:border-zinc-400 dark:hover:border-zinc-600"
            }`}
          >
            <Moon size={20} />
            Тёмная
          </button>
          <button
            onClick={() => handleThemeChange("light")}
            className={`flex items-center gap-2 px-4 py-3 rounded-xl border transition-colors ${
              theme === "light"
                ? "bg-zinc-700 dark:bg-zinc-700 border-zinc-600 text-white"
                : "bg-zinc-100 dark:bg-zinc-800 border-zinc-300 dark:border-zinc-700 text-zinc-700 dark:text-zinc-400 hover:border-zinc-400 dark:hover:border-zinc-600"
            }`}
          >
            <Sun size={20} />
            Светлая
          </button>
        </div>
      </div>

      <div>
        <h3 className="text-lg font-medium mb-4">Резервная копия</h3>
        <p className="text-sm text-zinc-400 mb-4">
          Создаёт копию базы данных в папке приложения. Используйте для бэкапа перед обновлениями.
        </p>
        {error && (
          <div className="p-4 rounded-lg bg-red-500/10 text-red-500 border border-red-500/20 mb-4">
            {error}
          </div>
        )}
        {backupPath && (
          <div className="p-4 rounded-lg bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 mb-4">
            <div className="flex items-center justify-between gap-4">
              <span className="text-sm break-all">Копия сохранена: {backupPath}</span>
              <button
                onClick={() => handleOpenFile(backupPath)}
                className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-emerald-600 text-white hover:bg-emerald-700 btn-transition text-sm shrink-0"
              >
                <ExternalLink size={14} />
                Открыть
              </button>
            </div>
          </div>
        )}
        <div className="flex gap-2">
          <button
            onClick={handleBackupExport}
            className="flex items-center gap-2 px-4 py-2 rounded-lg bg-emerald-600 text-white hover:bg-emerald-700 transition-colors"
          >
            <Download size={18} />
            Создать резервную копию
          </button>
          <button
            onClick={handleRestoreClick}
            className="flex items-center gap-2 px-4 py-2 rounded-lg bg-zinc-600 text-white hover:bg-zinc-500 transition-colors"
          >
            <Upload size={18} />
            Восстановить из копии
          </button>
        </div>
      </div>

      {/* Export/Import Section */}
      <div>
        <h3 className="text-lg font-medium mb-4 flex items-center gap-2">
          <FileDown size={20} className="text-blue-500" />
          Экспорт и импорт данных
        </h3>
        <p className="text-sm text-zinc-400 mb-4">
          Экспортируйте транзакции в Excel, CSV или JSON. Файл можно сразу открыть после экспорта.
        </p>

        {exportPath && (
          <div className="p-4 rounded-lg bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 mb-4">
            <div className="flex items-center justify-between gap-4">
              <span className="text-sm break-all">Файл сохранён: {exportPath}</span>
              <button
                onClick={() => handleOpenFile(exportPath)}
                className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-emerald-600 text-white hover:bg-emerald-700 btn-transition text-sm shrink-0"
              >
                <ExternalLink size={14} />
                Открыть
              </button>
            </div>
          </div>
        )}

        {importResult && (
          <div className={`p-4 rounded-lg border mb-4 ${
            importResult.errors.length > 0 
              ? "bg-amber-500/10 text-amber-400 border-amber-500/20" 
              : "bg-emerald-500/10 text-emerald-400 border-emerald-500/20"
          }`}>
            <p>Импортировано транзакций: {importResult.transactions_imported}</p>
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

              <div className="pt-2 border-t border-zinc-200 dark:border-zinc-700">
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
              </div>
            </>
          ) : (
            <div className="text-zinc-400">Не удалось загрузить статус модели</div>
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
            onClick={() => setShowBudgetForm(true)}
            className="flex items-center gap-2 px-4 py-2 rounded-lg bg-emerald-600 text-white hover:bg-emerald-700 btn-transition"
          >
            <Plus size={18} />
            Добавить бюджет
          </button>
        )}
      </div>

      <ConfirmDialog
        open={restoreConfirmPath !== null}
        title="Восстановить из копии?"
        message="Текущие данные будут заменены. Продолжить?"
        confirmLabel="Восстановить"
        variant="danger"
        onConfirm={handleRestoreConfirm}
        onCancel={() => setRestoreConfirmPath(null)}
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
