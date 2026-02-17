import { useState, useEffect } from "react";
import {
  Upload,
  FileText,
  CheckCircle,
  AlertTriangle,
  ArrowRight,
  ArrowLeft,
  Building2,
  Calendar,
  RefreshCw,
  Download,
} from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import {
  api,
  type Account,
  type Category,
  type ParsedStatement,
  type ParsedTransaction,
  type ImportTransaction,
  type BankImportResult,
} from "../lib/api";
import { useToast } from "../components/ui/Toast";

type Step = "upload" | "preview" | "result";

export function Import() {
  const { showToast } = useToast();
  const [step, setStep] = useState<Step>("upload");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Data
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [categories, setCategories] = useState<Category[]>([]);
  const [statement, setStatement] = useState<ParsedStatement | null>(null);
  const [selectedAccountId, setSelectedAccountId] = useState<number | null>(null);
  const [skipDuplicates, setSkipDuplicates] = useState(true);
  const [result, setResult] = useState<BankImportResult | null>(null);
  const [showTrainPrompt, setShowTrainPrompt] = useState(false);
  const [trainingInProgress, setTrainingInProgress] = useState(false);

  // Transaction editing
  const [editedTransactions, setEditedTransactions] = useState<
    (ParsedTransaction & { selected: boolean; categoryId: number | null; forceImport?: boolean })[]
  >([]);

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    try {
      const [accs, cats] = await Promise.all([
        api.getAccounts(),
        api.getCategories(),
      ]);
      setAccounts(accs);
      setCategories(cats);
      if (accs.length > 0 && !selectedAccountId) {
        setSelectedAccountId(accs[0].id);
      }
    } catch (e) {
      console.error("Failed to load data:", e);
    }
  };

  const handleFileSelect = async () => {
    try {
      setError(null);
      const selected = await open({
        multiple: false,
        directory: false,
        filters: [{ name: "PDF", extensions: ["pdf"] }],
      });

      if (selected && typeof selected === "string") {
        setLoading(true);
        const llmEnabled = localStorage.getItem("llm_enabled") === "true";
        const useEmbedded = localStorage.getItem("llm_use_embedded") === "true";
        const parsed = await api.parseBankStatement(selected, {
          useLlm: llmEnabled || useEmbedded || undefined,
          useEmbedded: useEmbedded || undefined,
          ollamaUrl: !useEmbedded && llmEnabled ? (localStorage.getItem("ollama_url") ?? undefined) : undefined,
          ollamaModel: !useEmbedded && llmEnabled ? (localStorage.getItem("ollama_model") ?? undefined) : undefined,
        });
        setStatement(parsed);

        // Initialize edited transactions
        setEditedTransactions(
          parsed.transactions.map((tx) => ({
            ...tx,
            selected: !tx.is_duplicate,
            categoryId: tx.suggested_category_id,
            forceImport: false,
          }))
        );

        setStep("preview");
        showToast(`Найдено ${parsed.transactions.length} транзакций`, "success");
      }
    } catch (e) {
      setError(String(e));
      showToast("Ошибка чтения файла", "error");
    } finally {
      setLoading(false);
    }
  };

  const handleImport = async () => {
    if (!selectedAccountId || !statement) return;

    const toImport: ImportTransaction[] = editedTransactions
      .filter((tx) => tx.selected)
      .map((tx) => ({
        date: tx.date,
        amount: tx.amount,
        transaction_type: tx.transaction_type,
        description: tx.description,
        category_id: tx.categoryId,
        skip_if_duplicate: tx.is_duplicate ? !tx.forceImport : true,
      }));

    if (toImport.length === 0) {
      showToast("Выберите транзакции для импорта", "warning");
      return;
    }

    try {
      setLoading(true);
      const importResult = await api.importBankTransactions({
        transactions: toImport,
        account_id: selectedAccountId,
        skip_duplicates: skipDuplicates,
      });
      setResult(importResult);
      setStep("result");
      setShowTrainPrompt(importResult.imported > 0);

      if (importResult.imported > 0) {
        showToast(`Импортировано: ${importResult.imported} транзакций`, "success");
      }
    } catch (e) {
      setError(String(e));
      showToast("Ошибка импорта", "error");
    } finally {
      setLoading(false);
    }
  };

  const handleReset = () => {
    setStep("upload");
    setStatement(null);
    setEditedTransactions([]);
    setResult(null);
    setError(null);
    setShowTrainPrompt(false);
  };

  const handleTrainModel = async () => {
    try {
      setTrainingInProgress(true);
      const trainResult = await api.trainModel();
      showToast(trainResult.message || "Модель обучена", trainResult.success ? "success" : "info");
      setShowTrainPrompt(false);
    } catch (e) {
      showToast("Ошибка обучения модели", "error");
    } finally {
      setTrainingInProgress(false);
    }
  };

  const toggleTransaction = (index: number) => {
    setEditedTransactions((prev) =>
      prev.map((tx, i) => (i === index ? { ...tx, selected: !tx.selected } : tx))
    );
  };

  const selectAll = () => {
    setEditedTransactions((prev) => prev.map((tx) => ({ ...tx, selected: true })));
  };

  const deselectAll = () => {
    setEditedTransactions((prev) => prev.map((tx) => ({ ...tx, selected: false })));
  };

  const updateCategory = (index: number, categoryId: number | null) => {
    setEditedTransactions((prev) =>
      prev.map((tx, i) => (i === index ? { ...tx, categoryId } : tx))
    );
  };

  const formatAmount = (amount: number, type: string) => {
    const formatted = new Intl.NumberFormat("ru-KZ").format(amount);
    return type === "income" ? `+${formatted}` : `-${formatted}`;
  };

  const formatDate = (dateStr: string) => {
    const date = new Date(dateStr);
    return date.toLocaleDateString("ru-KZ", {
      day: "2-digit",
      month: "2-digit",
      year: "2-digit",
    });
  };

  const selectedCount = editedTransactions.filter((tx) => tx.selected).length;
  const duplicateCount = editedTransactions.filter((tx) => tx.is_duplicate).length;

  return (
    <div className="p-6 max-w-6xl mx-auto">
      {/* Header */}
      <div className="mb-6">
        <h1 className="text-2xl font-semibold text-zinc-900 dark:text-white">
          Импорт выписки
        </h1>
        <p className="text-zinc-500 dark:text-zinc-400 mt-1">
          Загрузите PDF выписку из банка для автоматического импорта транзакций
        </p>
      </div>

      {/* Steps Indicator */}
      <div className="flex items-center gap-2 mb-8">
        {[
          { id: "upload", label: "Загрузка", icon: Upload },
          { id: "preview", label: "Просмотр", icon: FileText },
          { id: "result", label: "Результат", icon: CheckCircle },
        ].map((s, i) => (
          <div key={s.id} className="flex items-center">
            {i > 0 && (
              <div
                className={`w-12 h-0.5 mx-2 ${
                  step === s.id || (step === "result" && i <= 2) || (step === "preview" && i <= 1)
                    ? "bg-emerald-500"
                    : "bg-zinc-300 dark:bg-zinc-600"
                }`}
              />
            )}
            <div
              className={`flex items-center gap-2 px-4 py-2 rounded-lg transition-colors ${
                step === s.id
                  ? "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400"
                  : "text-zinc-500 dark:text-zinc-400"
              }`}
            >
              <s.icon size={18} />
              <span className="font-medium">{s.label}</span>
            </div>
          </div>
        ))}
      </div>

      {/* Error */}
      {error && (
        <div className="mb-6 p-4 rounded-lg bg-red-500/10 text-red-600 dark:text-red-400 border border-red-500/20">
          <div className="flex items-center gap-2">
            <AlertTriangle size={18} />
            <span>{error}</span>
          </div>
        </div>
      )}

      {/* Step 1: Upload */}
      {step === "upload" && (
        <div className="bg-white dark:bg-zinc-900 rounded-xl border border-zinc-200 dark:border-zinc-700 p-8">
          <div
            onClick={handleFileSelect}
            className={`
              border-2 border-dashed rounded-xl p-12 text-center cursor-pointer transition-all
              ${loading ? "opacity-50 pointer-events-none" : ""}
              border-zinc-300 dark:border-zinc-600 hover:border-emerald-500 dark:hover:border-emerald-500
              hover:bg-emerald-500/5
            `}
          >
            {loading ? (
              <RefreshCw size={48} className="mx-auto mb-4 text-emerald-500 animate-spin" />
            ) : (
              <Upload size={48} className="mx-auto mb-4 text-zinc-400" />
            )}
            <h3 className="text-lg font-medium text-zinc-900 dark:text-white mb-2">
              {loading ? "Обработка файла..." : "Нажмите для выбора PDF файла"}
            </h3>
            <p className="text-zinc-500 dark:text-zinc-400">
              Поддерживаемые банки: Kaspi Bank
            </p>
          </div>

          <div className="mt-6 p-4 rounded-lg bg-blue-500/10 border border-blue-500/20">
            <div className="flex items-start gap-3">
              <Building2 size={20} className="text-blue-500 mt-0.5" />
              <div>
                <h4 className="font-medium text-blue-600 dark:text-blue-400">
                  Как получить выписку?
                </h4>
                <ul className="mt-2 text-sm text-zinc-600 dark:text-zinc-400 space-y-1">
                  <li>• Kaspi: Приложение → Мой Банк → Выписка → Скачать PDF</li>
                </ul>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Step 2: Preview */}
      {step === "preview" && statement && (
        <div className="space-y-6">
          {/* Statement Info */}
          <div className="bg-white dark:bg-zinc-900 rounded-xl border border-zinc-200 dark:border-zinc-700 p-6">
            <div className="flex items-center justify-between mb-4">
              <div className="flex items-center gap-3">
                <div className="p-2.5 rounded-lg bg-emerald-500/10">
                  <Building2 size={24} className="text-emerald-500" />
                </div>
                <div>
                  <h3 className="font-semibold text-zinc-900 dark:text-white">
                    {statement.bank}
                  </h3>
                  <p className="text-sm text-zinc-500">
                    {statement.card && `Карта ${statement.card}`}
                    {statement.account && ` • ${statement.account}`}
                  </p>
                </div>
              </div>
              <div className="flex items-center gap-2 text-sm text-zinc-500">
                <Calendar size={16} />
                <span>
                  {formatDate(statement.period_start)} — {formatDate(statement.period_end)}
                </span>
              </div>
            </div>

            <div className="grid grid-cols-3 gap-4 pt-4 border-t border-zinc-200 dark:border-zinc-700">
              <div>
                <span className="text-sm text-zinc-500">Всего транзакций</span>
                <p className="text-lg font-semibold text-zinc-900 dark:text-white">
                  {statement.transactions.length}
                </p>
              </div>
              <div>
                <span className="text-sm text-zinc-500">Выбрано для импорта</span>
                <p className="text-lg font-semibold text-emerald-600">
                  {selectedCount}
                </p>
              </div>
              <div>
                <span className="text-sm text-zinc-500">Возможных дубликатов</span>
                <p className="text-lg font-semibold text-amber-600">
                  {duplicateCount}
                </p>
              </div>
            </div>
          </div>

          {/* Import Options */}
          <div className="bg-white dark:bg-zinc-900 rounded-xl border border-zinc-200 dark:border-zinc-700 p-6">
            <h3 className="font-semibold text-zinc-900 dark:text-white mb-4">
              Настройки импорта
            </h3>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="block text-sm font-medium text-zinc-600 dark:text-zinc-300 mb-1.5">
                  Счёт для импорта
                </label>
                <select
                  value={selectedAccountId ?? ""}
                  onChange={(e) => setSelectedAccountId(Number(e.target.value))}
                  className="w-full px-3 py-2 rounded-lg border border-zinc-300 dark:border-zinc-600 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-white"
                >
                  {accounts.map((acc) => (
                    <option key={acc.id} value={acc.id}>
                      {acc.name} ({new Intl.NumberFormat("ru-KZ").format(acc.balance)} ₸)
                    </option>
                  ))}
                </select>
              </div>
              <div className="flex items-end">
                <label className="flex items-center gap-2 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={skipDuplicates}
                    onChange={(e) => setSkipDuplicates(e.target.checked)}
                    className="w-4 h-4 rounded border-zinc-300 text-emerald-500 focus:ring-emerald-500"
                  />
                  <span className="text-sm text-zinc-600 dark:text-zinc-300">
                    Пропускать дубликаты
                  </span>
                </label>
              </div>
            </div>
          </div>

          {/* Transactions Table */}
          <div className="bg-white dark:bg-zinc-900 rounded-xl border border-zinc-200 dark:border-zinc-700 overflow-hidden">
            <div className="p-4 border-b border-zinc-200 dark:border-zinc-700 flex items-center justify-between">
              <h3 className="font-semibold text-zinc-900 dark:text-white">
                Транзакции
              </h3>
              <div className="flex gap-2">
                <button
                  onClick={selectAll}
                  className="px-3 py-1.5 text-sm rounded-lg bg-zinc-100 dark:bg-zinc-800 text-zinc-600 dark:text-zinc-400 hover:bg-zinc-200 dark:hover:bg-zinc-700"
                >
                  Выбрать все
                </button>
                <button
                  onClick={deselectAll}
                  className="px-3 py-1.5 text-sm rounded-lg bg-zinc-100 dark:bg-zinc-800 text-zinc-600 dark:text-zinc-400 hover:bg-zinc-200 dark:hover:bg-zinc-700"
                >
                  Снять выбор
                </button>
              </div>
            </div>

            <div className="max-h-96 overflow-y-auto">
              <table className="w-full">
                <thead className="bg-zinc-50 dark:bg-zinc-800/50 sticky top-0">
                  <tr className="text-left text-sm text-zinc-500">
                    <th className="px-4 py-3 w-12"></th>
                    <th className="px-4 py-3">Дата</th>
                    <th className="px-4 py-3">Описание</th>
                    <th className="px-4 py-3">Тип</th>
                    <th className="px-4 py-3">Категория</th>
                    <th className="px-4 py-3 text-right">Сумма</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-zinc-200 dark:divide-zinc-700">
                  {editedTransactions.map((tx, index) => (
                    <tr
                      key={index}
                      className={`
                        transition-colors
                        ${tx.selected ? "bg-white dark:bg-zinc-900" : "bg-zinc-50 dark:bg-zinc-800/30 opacity-60"}
                        ${tx.is_duplicate ? "bg-amber-50 dark:bg-amber-900/10" : ""}
                      `}
                    >
                      <td className="px-4 py-3">
                        <input
                          type="checkbox"
                          checked={tx.selected}
                          onChange={() => toggleTransaction(index)}
                          className="w-4 h-4 rounded border-zinc-300 text-emerald-500 focus:ring-emerald-500"
                        />
                      </td>
                      <td className="px-4 py-3 text-sm text-zinc-600 dark:text-zinc-400">
                        {formatDate(tx.date)}
                      </td>
                      <td className="px-4 py-3">
                        <div className="flex items-center gap-2">
                          <span className="text-sm text-zinc-900 dark:text-white truncate max-w-[200px]">
                            {tx.description}
                          </span>
                          {tx.is_duplicate && (
                            <span className="inline-flex items-center gap-2">
                              <span className="px-1.5 py-0.5 text-xs rounded bg-amber-500/10 text-amber-600">
                                Дубликат?
                              </span>
                              <label className="inline-flex items-center gap-1 text-xs text-zinc-600 dark:text-zinc-400 cursor-pointer">
                                <input
                                  type="checkbox"
                                  checked={!!tx.forceImport}
                                  onChange={() =>
                                    setEditedTransactions((prev) =>
                                      prev.map((p, i) =>
                                        i === index ? { ...p, forceImport: !p.forceImport } : p
                                      )
                                    )}
                                  className="w-3.5 h-3.5 rounded border-zinc-300 text-emerald-500"
                                />
                                Всё равно импортировать
                              </label>
                            </span>
                          )}
                        </div>
                      </td>
                      <td className="px-4 py-3">
                        <span
                          className={`px-2 py-0.5 text-xs rounded ${
                            tx.transaction_type === "income"
                              ? "bg-emerald-500/10 text-emerald-600"
                              : "bg-red-500/10 text-red-600"
                          }`}
                        >
                          {tx.original_type}
                        </span>
                      </td>
                      <td className="px-4 py-3">
                        <select
                          value={tx.categoryId ?? ""}
                          onChange={(e) =>
                            updateCategory(index, e.target.value ? Number(e.target.value) : null)
                          }
                          className="w-full px-2 py-1 text-sm rounded border border-zinc-300 dark:border-zinc-600 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-white"
                        >
                          <option value="">Без категории</option>
                          {categories
                            .filter((c) => c.category_type === tx.transaction_type)
                            .map((cat) => (
                              <option key={cat.id} value={cat.id}>
                                {cat.name}
                                {tx.suggested_category_id === cat.id && tx.confidence
                                  ? ` (${Math.round(tx.confidence * 100)}%)`
                                  : ""}
                              </option>
                            ))}
                        </select>
                      </td>
                      <td className="px-4 py-3 text-right">
                        <span
                          className={`font-medium ${
                            tx.transaction_type === "income"
                              ? "text-emerald-600"
                              : "text-red-600"
                          }`}
                        >
                          {formatAmount(tx.amount, tx.transaction_type)} ₸
                        </span>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>

          {/* Actions */}
          <div className="flex justify-between">
            <button
              onClick={handleReset}
              className="flex items-center gap-2 px-4 py-2 rounded-lg bg-zinc-100 dark:bg-zinc-800 text-zinc-600 dark:text-zinc-400 hover:bg-zinc-200 dark:hover:bg-zinc-700"
            >
              <ArrowLeft size={18} />
              Назад
            </button>
            <button
              onClick={handleImport}
              disabled={loading || selectedCount === 0}
              className="flex items-center gap-2 px-6 py-2 rounded-lg bg-emerald-500 text-white hover:bg-emerald-600 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {loading ? (
                <RefreshCw size={18} className="animate-spin" />
              ) : (
                <Download size={18} />
              )}
              Импортировать ({selectedCount})
            </button>
          </div>
        </div>
      )}

      {/* Step 3: Result */}
      {step === "result" && result && (
        <div className="bg-white dark:bg-zinc-900 rounded-xl border border-zinc-200 dark:border-zinc-700 p-8 text-center">
          <div className="w-16 h-16 mx-auto mb-4 rounded-full bg-emerald-500/10 flex items-center justify-center">
            <CheckCircle size={32} className="text-emerald-500" />
          </div>
          <h2 className="text-xl font-semibold text-zinc-900 dark:text-white mb-2">
            Импорт завершён
          </h2>

          <div className="grid grid-cols-3 gap-4 my-6 max-w-md mx-auto">
            <div className="p-4 rounded-lg bg-emerald-500/10">
              <p className="text-2xl font-bold text-emerald-600">{result.imported}</p>
              <p className="text-sm text-zinc-500">Импортировано</p>
            </div>
            <div className="p-4 rounded-lg bg-amber-500/10">
              <p className="text-2xl font-bold text-amber-600">{result.skipped_duplicates}</p>
              <p className="text-sm text-zinc-500">Пропущено</p>
            </div>
            <div className="p-4 rounded-lg bg-red-500/10">
              <p className="text-2xl font-bold text-red-600">{result.failed}</p>
              <p className="text-sm text-zinc-500">Ошибок</p>
            </div>
          </div>

          {result.errors.length > 0 && (
            <div className="mb-6 p-4 rounded-lg bg-red-500/10 text-left">
              <h4 className="font-medium text-red-600 mb-2">Ошибки:</h4>
              <ul className="text-sm text-red-500 space-y-1">
                {result.errors.slice(0, 5).map((err, i) => (
                  <li key={i}>• {err}</li>
                ))}
                {result.errors.length > 5 && (
                  <li>... и ещё {result.errors.length - 5}</li>
                )}
              </ul>
            </div>
          )}

          {showTrainPrompt && result.imported > 0 && (
            <div className="mb-6 p-4 rounded-xl border border-zinc-200 dark:border-zinc-700 bg-zinc-50 dark:bg-zinc-800/50">
              <p className="text-zinc-700 dark:text-zinc-300 mb-3">Обучить модель по новым данным?</p>
              <div className="flex justify-center gap-3">
                <button
                  type="button"
                  onClick={handleTrainModel}
                  disabled={trainingInProgress}
                  className="px-4 py-2 rounded-lg bg-emerald-500 text-white hover:bg-emerald-600 disabled:opacity-50"
                >
                  {trainingInProgress ? "Обучение…" : "Обучить"}
                </button>
                <button
                  type="button"
                  onClick={() => setShowTrainPrompt(false)}
                  className="px-4 py-2 rounded-lg bg-zinc-200 dark:bg-zinc-700 text-zinc-700 dark:text-zinc-300 hover:bg-zinc-300 dark:hover:bg-zinc-600"
                >
                  Позже
                </button>
              </div>
            </div>
          )}

          <div className="flex justify-center gap-4">
            <button
              onClick={handleReset}
              className="flex items-center gap-2 px-4 py-2 rounded-lg bg-zinc-100 dark:bg-zinc-800 text-zinc-600 dark:text-zinc-400 hover:bg-zinc-200 dark:hover:bg-zinc-700"
            >
              <RefreshCw size={18} />
              Импортировать ещё
            </button>
            <a
              href="/transactions"
              className="flex items-center gap-2 px-4 py-2 rounded-lg bg-emerald-500 text-white hover:bg-emerald-600"
            >
              Перейти к транзакциям
              <ArrowRight size={18} />
            </a>
          </div>
        </div>
      )}
    </div>
  );
}
