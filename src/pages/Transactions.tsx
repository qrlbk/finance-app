import { useEffect, useState, useRef } from "react";
import { useTranslation } from "react-i18next";
import { Plus, Pencil, Trash2, Search, Filter, ArrowRightLeft, ChevronDown, Calendar, CreditCard, Lightbulb, Check, X } from "lucide-react";
import { api, type TransactionWithDetails, type Account, type Category, type CategoryPrediction } from "../lib/api";
import { ConfirmDialog } from "../components/ui/ConfirmDialog";
import { EmptyState } from "../components/ui/EmptyState";
import { useToast } from "../components/ui/Toast";
import { useDebounce } from "../hooks/useDebounce";
import { formatCurrency, formatDate } from "../lib/format";

function formatAmount(amount: number, type: string) {
  const abs = Math.abs(amount);
  const formatted = formatCurrency(abs);
  return type === "income" ? `+${formatted}` : `-${formatted}`;
}

function formatDateDisplay(dateStr: string) {
  return formatDate(new Date(dateStr + "T12:00:00"));
}

// Get date presets
function getDatePresets() {
  const today = new Date();
  const startOfMonth = new Date(today.getFullYear(), today.getMonth(), 1);
  const startOfWeek = new Date(today);
  startOfWeek.setDate(today.getDate() - today.getDay() + 1);
  
  return {
    thisMonth: {
      from: startOfMonth.toISOString().slice(0, 10),
      to: today.toISOString().slice(0, 10),
    },
    thisWeek: {
      from: startOfWeek.toISOString().slice(0, 10),
      to: today.toISOString().slice(0, 10),
    },
  };
}

export function Transactions() {
  const { t } = useTranslation();
  const { showToast } = useToast();
  const [transactions, setTransactions] = useState<TransactionWithDetails[]>([]);
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [categories, setCategories] = useState<Category[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showForm, setShowForm] = useState(false);
  const [editingId, setEditingId] = useState<number | null>(null);
  const [deleteConfirmId, setDeleteConfirmId] = useState<number | null>(null);
  const [deletingId, setDeletingId] = useState<number | null>(null);
  const [assigningCategoryId, setAssigningCategoryId] = useState<number | null>(null);
  const [autoAssigning, setAutoAssigning] = useState(false);
  const [showTransferForm, setShowTransferForm] = useState(false);
  const [showFilters, setShowFilters] = useState(false);
  const [hasMore, setHasMore] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);
  const formRef = useRef<HTMLFormElement>(null);
  const amountInputRef = useRef<HTMLInputElement>(null);

  const [transferForm, setTransferForm] = useState({
    from_account_id: 0,
    to_account_id: 0,
    amount: "",
    date: new Date().toISOString().slice(0, 10),
    note: "",
  });
  const [filters, setFilters] = useState({
    date_from: "",
    date_to: "",
    account_id: null as number | null,
    category_id: null as number | null,
    uncategorized_only: false,
    transaction_type: "" as "" | "income" | "expense",
    search_note: "",
  });
  const [form, setForm] = useState({
    account_id: 0,
    category_id: null as number | null,
    amount: "",
    transaction_type: "expense" as "income" | "expense",
    note: "",
    date: new Date().toISOString().slice(0, 10),
  });

  // ML Category Prediction
  const [suggestedCategory, setSuggestedCategory] = useState<CategoryPrediction | null>(null);
  const debouncedNote = useDebounce(form.note, 300);

  const PAGE_SIZE = 50;

const emptyFilters = {
    date_from: "",
    date_to: "",
    account_id: null as number | null,
    category_id: null as number | null,
    uncategorized_only: false,
    transaction_type: "" as "" | "income" | "expense",
    search_note: "",
  };

  const hasActiveFilters = filters.date_from || filters.date_to || 
    filters.account_id || filters.category_id || 
    filters.uncategorized_only ||
    filters.transaction_type || filters.search_note;

  const loadData = async (filterOverride?: typeof filters) => {
    const f = filterOverride ?? filters;
    try {
      setLoading(true);
      setError(null);
      setHasMore(true);
      const filterParams = {
        limit: PAGE_SIZE,
        offset: 0,
        date_from: f.date_from || undefined,
        date_to: f.date_to || undefined,
        account_id: f.account_id ?? undefined,
        category_id: f.category_id ?? undefined,
        uncategorized_only: f.uncategorized_only || undefined,
        transaction_type: f.transaction_type || undefined,
        search_note: f.search_note.trim() || undefined,
      };
      const [txs, accs, cats] = await Promise.all([
        api.getTransactions(filterParams),
        api.getAccounts(),
        api.getCategories(),
      ]);
      setTransactions(txs);
      setHasMore(txs.length === PAGE_SIZE);
      setAccounts(accs);
      setCategories(cats);
      if (accs.length > 0 && form.account_id === 0) {
        setForm((prev) => ({ ...prev, account_id: accs[0].id }));
      }
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const loadMore = async () => {
    const f = filters;
    try {
      setLoadingMore(true);
      const filterParams = {
        limit: PAGE_SIZE,
        offset: transactions.length,
        date_from: f.date_from || undefined,
        date_to: f.date_to || undefined,
        account_id: f.account_id ?? undefined,
        category_id: f.category_id ?? undefined,
        uncategorized_only: f.uncategorized_only || undefined,
        transaction_type: f.transaction_type || undefined,
        search_note: f.search_note.trim() || undefined,
      };
      const txs = await api.getTransactions(filterParams);
      setTransactions((prev) => [...prev, ...txs]);
      setHasMore(txs.length === PAGE_SIZE);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoadingMore(false);
    }
  };

  const applyFilters = () => loadData();

  const handleResetFilters = () => {
    setFilters(emptyFilters);
    loadData(emptyFilters);
  };

  const applyPreset = (preset: "thisMonth" | "thisWeek" | "uncategorized") => {
    if (preset === "uncategorized") {
      const newFilters = { ...emptyFilters, uncategorized_only: true };
      setFilters(newFilters);
      loadData(newFilters);
      return;
    }
    const presets = getDatePresets();
    const newFilters = {
      ...emptyFilters,
      date_from: presets[preset].from,
      date_to: presets[preset].to,
    };
    setFilters(newFilters);
    loadData(newFilters);
  };

  useEffect(() => {
    loadData();
  }, []);

  // Auto-focus amount field when form opens
  useEffect(() => {
    if (showForm && amountInputRef.current) {
      setTimeout(() => amountInputRef.current?.focus(), 100);
    }
  }, [showForm]);

  // ML: Fetch category prediction when note changes
  useEffect(() => {
    const fetchPrediction = async () => {
      // Only predict if:
      // - note has at least 3 characters
      // - no category is selected yet
      // - it's an expense transaction (most common use case)
      if (debouncedNote.trim().length >= 3 && !form.category_id && form.transaction_type === "expense") {
        try {
          const amount = parseFloat(form.amount) || undefined;
          let threshold: number | undefined;
          try {
            const v = localStorage.getItem("ml_confidence_threshold");
            if (v != null) {
              const n = parseFloat(v);
              if (!Number.isNaN(n) && n >= 0.2 && n <= 0.9) threshold = n;
            }
          } catch {}
          const llmEnabled = localStorage.getItem("llm_enabled") === "true";
          const useEmbedded = localStorage.getItem("llm_use_embedded") === "true";
          const prediction = await api.predictCategory(debouncedNote, amount, form.date, threshold, {
            useLlm: llmEnabled || useEmbedded || undefined,
            useEmbedded: useEmbedded || undefined,
            ollamaUrl: !useEmbedded && llmEnabled ? (localStorage.getItem("ollama_url") ?? undefined) : undefined,
            ollamaModel: !useEmbedded && llmEnabled ? (localStorage.getItem("ollama_model") ?? undefined) : undefined,
            transactionType: form.transaction_type,
          });
          setSuggestedCategory(prediction);
        } catch {
          setSuggestedCategory(null);
        }
      } else {
        setSuggestedCategory(null);
      }
    };

    fetchPrediction();
  }, [debouncedNote, form.category_id, form.transaction_type, form.amount, form.date]);

  const incomeCategories = categories.filter((c) => c.category_type === "income");
  const expenseCategories = categories.filter((c) => c.category_type === "expense");
  const formCategories = form.transaction_type === "income" ? incomeCategories : expenseCategories;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    const amount = parseFloat(form.amount);
    if (isNaN(amount) || amount <= 0) return;
    try {
      if (editingId) {
        await api.updateTransaction({
          id: editingId,
          account_id: form.account_id,
          category_id: form.category_id,
          amount,
          transaction_type: form.transaction_type,
          note: form.note || null,
          date: form.date,
        });
        showToast(t("transactions.updated"), "success");
      } else {
        await api.createTransaction({
          account_id: form.account_id,
          category_id: form.category_id,
          amount,
          transaction_type: form.transaction_type,
          note: form.note || null,
          date: form.date,
        });
        showToast(t("transactions.added"), "success");
      }
      setForm({
        account_id: accounts[0]?.id ?? 0,
        category_id: null,
        amount: "",
        transaction_type: "expense",
        note: "",
        date: new Date().toISOString().slice(0, 10),
      });
      setShowForm(false);
      setEditingId(null);
      loadData();
    } catch (e) {
      setError(String(e));
      showToast(t("transactions.errorSave"), "error");
    }
  };

  const handleEdit = (tx: TransactionWithDetails) => {
    setForm({
      account_id: tx.account_id,
      category_id: tx.category_id,
      amount: String(Math.abs(tx.amount)),
      transaction_type: tx.transaction_type as "income" | "expense",
      note: tx.note ?? "",
      date: tx.date,
    });
    setEditingId(tx.id);
    setShowForm(true);
  };

  const handleDeleteClick = (id: number) => {
    setDeleteConfirmId(id);
  };

  const handleDeleteConfirm = async () => {
    if (deleteConfirmId === null) return;
    try {
      setDeletingId(deleteConfirmId);
      await api.deleteTransaction(deleteConfirmId);
      // Wait for animation
      setTimeout(() => {
        setDeleteConfirmId(null);
        setDeletingId(null);
        loadData();
        showToast(t("transactions.deleted"), "success");
      }, 300);
    } catch (e) {
      setError(String(e));
      setDeletingId(null);
      showToast(t("transactions.errorDelete"), "error");
    }
  };

  const handleTransferSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    const amount = parseFloat(transferForm.amount);
    if (isNaN(amount) || amount <= 0) return;
    if (transferForm.from_account_id === transferForm.to_account_id) return;
    try {
      await api.createTransfer({
        from_account_id: transferForm.from_account_id,
        to_account_id: transferForm.to_account_id,
        amount,
        date: transferForm.date,
        note: transferForm.note.trim() || null,
      });
      setShowTransferForm(false);
      setTransferForm({
        from_account_id: accounts[0]?.id ?? 0,
        to_account_id: accounts[1]?.id ?? 0,
        amount: "",
        date: new Date().toISOString().slice(0, 10),
        note: "",
      });
      loadData();
      showToast(t("transactions.transferDone"), "success");
    } catch (err) {
      setError(String(err));
      showToast(t("transactions.transferError"), "error");
    }
  };

  const handleQuickAssignCategory = async (tx: TransactionWithDetails, categoryId: number) => {
    if (!categoryId) return;
    setAssigningCategoryId(tx.id);
    try {
      await api.updateTransaction({
        id: tx.id,
        account_id: tx.account_id,
        category_id: categoryId,
        amount: Math.abs(tx.amount),
        transaction_type: tx.transaction_type,
        note: tx.note ?? null,
        date: tx.date,
      });
      setTransactions((prev) =>
        prev.map((t) =>
          t.id === tx.id
            ? {
                ...t,
                category_id: categoryId,
                category_name: categories.find((c) => c.id === categoryId)?.name ?? null,
              }
            : t
        )
      );
      showToast(t("transactions.categoryAssigned"), "success");
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      showToast(msg ? t("transactions.categoryAssignErrorMsg", { msg }) : t("transactions.categoryAssignError"), "error");
    } finally {
      setAssigningCategoryId(null);
    }
  };

  const handleAutoAssignCategories = async () => {
    const llmEnabled = localStorage.getItem("llm_enabled") === "true";
    const useEmbedded = localStorage.getItem("llm_use_embedded") === "true";
    if (!llmEnabled && !useEmbedded) {
      showToast(t("transactions.enableLLM"), "info");
      return;
    }
    setAutoAssigning(true);
    try {
      const allUncategorized = await api.getTransactions({
        uncategorized_only: true,
        limit: 500,
      });
      const uncategorized = allUncategorized.filter((tx) => (tx.note?.trim()?.length ?? 0) >= 2);
      if (uncategorized.length === 0) {
        showToast(t("transactions.noUncategorized"), "info");
        setAutoAssigning(false);
        return;
      }
      api.startOllamaServer();
      await new Promise((r) => setTimeout(r, 3000));
      let assigned = 0;
      let lastError: string | null = null;
      const threshold = (() => {
        try {
          const v = localStorage.getItem("ml_confidence_threshold");
          if (v != null) {
            const n = parseFloat(v);
            if (!Number.isNaN(n) && n >= 0.2 && n <= 0.9) return n;
          }
        } catch {}
        return 0.3;
      })();
      const options = {
        useLlm: true,
        useEmbedded: useEmbedded || undefined,
        ollamaUrl: !useEmbedded && llmEnabled ? (localStorage.getItem("ollama_url") ?? undefined) : undefined,
        ollamaModel: !useEmbedded && llmEnabled ? (localStorage.getItem("ollama_model") ?? undefined) : undefined,
      };
      for (const tx of uncategorized) {
        try {
          const prediction = await api.predictCategory(
            tx.note!.trim(),
            tx.amount,
            tx.date,
            threshold,
            { ...options, transactionType: tx.transaction_type }
          );
          if (prediction?.category_id) {
            await api.updateTransaction({
              id: tx.id,
              account_id: tx.account_id,
              category_id: prediction.category_id,
              amount: Math.abs(tx.amount),
              transaction_type: tx.transaction_type,
              note: tx.note ?? null,
              date: tx.date,
            });
            setTransactions((prev) =>
              prev.map((t) =>
                t.id === tx.id
                  ? {
                      ...t,
                      category_id: prediction.category_id,
                      category_name: prediction.category_name,
                    }
                  : t
              )
            );
            assigned += 1;
          }
        } catch (e) {
          lastError = e instanceof Error ? e.message : String(e);
        }
      }
      if (assigned > 0) {
        showToast(t("transactions.assignedCount", { assigned, total: uncategorized.length }), "success");
        const filterParams = {
          limit: PAGE_SIZE,
          offset: 0,
          date_from: filters.date_from || undefined,
          date_to: filters.date_to || undefined,
          account_id: filters.account_id ?? undefined,
          category_id: filters.category_id ?? undefined,
          uncategorized_only: filters.uncategorized_only || undefined,
          transaction_type: filters.transaction_type || undefined,
          search_note: filters.search_note.trim() || undefined,
        };
        const txs = await api.getTransactions(filterParams);
        setTransactions(txs);
        setHasMore(txs.length === PAGE_SIZE);
      } else if (lastError) {
        showToast(t("transactions.suggestionsError", { error: lastError }), "error");
      } else {
        showToast(t("transactions.noSuggestions"), "info");
      }
    } finally {
      setAutoAssigning(false);
    }
  };

  return (
    <div className="space-y-6">
      {error && (
        <div className="p-4 rounded-lg bg-red-500/10 text-red-500 border border-red-500/20 animate-shake">
          {error}
        </div>
      )}

      <div className="flex justify-between items-center">
        <h3 className="text-lg font-medium">{t("transactions.title")}</h3>
        <div className="flex gap-2">
          <button
            type="button"
            onClick={() => {
              setShowTransferForm(true);
              setTransferForm((prev) => ({
                ...prev,
                from_account_id: accounts[0]?.id ?? 0,
                to_account_id: accounts[1]?.id ?? accounts[0]?.id ?? 0,
                amount: "",
                date: new Date().toISOString().slice(0, 10),
                note: "",
              }));
            }}
            className="flex items-center gap-2 px-4 py-2 rounded-lg bg-zinc-600 text-white hover:bg-zinc-500 btn-transition"
          >
            <ArrowRightLeft size={18} />
            <span className="hidden sm:inline">{t("transactions.transferBtn")}</span>
          </button>
          <button
            onClick={() => {
              setShowForm(true);
              setEditingId(null);
              setForm({
                account_id: accounts[0]?.id ?? 0,
                category_id: null,
                amount: "",
                transaction_type: "expense",
                note: "",
                date: new Date().toISOString().slice(0, 10),
              });
            }}
            className="flex items-center gap-2 px-4 py-2 rounded-lg bg-emerald-600 text-white hover:bg-emerald-700 btn-transition"
          >
            <Plus size={18} />
            {t("common.add")}
          </button>
        </div>
      </div>

      {/* Transfer Form */}
      {showTransferForm && (
        <form
          onSubmit={handleTransferSubmit}
          className="p-6 rounded-xl bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700 shadow-sm dark:shadow-none space-y-4 animate-slide-down"
        >
          <h4 className="font-medium">{t("transactions.transferFormTitle")}</h4>
          <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
            <div>
              <label className="block text-sm text-zinc-400 mb-1">{t("transactions.fromAccount")}</label>
              <select
                value={transferForm.from_account_id}
                onChange={(e) => setTransferForm((f) => ({ ...f, from_account_id: +e.target.value }))}
                className="w-full px-4 py-2 rounded-lg bg-zinc-100 dark:bg-zinc-700 border border-zinc-300 dark:border-zinc-600 text-zinc-900 dark:text-white form-transition focus:ring-2 focus:ring-emerald-500"
                required
              >
                {accounts.map((a) => (
                  <option key={a.id} value={a.id}>
                    {a.name}
                  </option>
                ))}
              </select>
            </div>
            <div>
              <label className="block text-sm text-zinc-400 mb-1">{t("transactions.toAccount")}</label>
              <select
                value={transferForm.to_account_id}
                onChange={(e) => setTransferForm((f) => ({ ...f, to_account_id: +e.target.value }))}
                className="w-full px-4 py-2 rounded-lg bg-zinc-100 dark:bg-zinc-700 border border-zinc-300 dark:border-zinc-600 text-zinc-900 dark:text-white form-transition focus:ring-2 focus:ring-emerald-500"
                required
              >
                {accounts.map((a) => (
                  <option key={a.id} value={a.id}>
                    {a.name}
                  </option>
                ))}
              </select>
            </div>
            <div>
              <label className="block text-sm text-zinc-400 mb-1">{t("transactions.amount")}</label>
              <input
                type="number"
                step="0.01"
                min="0"
                value={transferForm.amount}
                onChange={(e) => setTransferForm((f) => ({ ...f, amount: e.target.value }))}
                className="w-full px-4 py-2 rounded-lg bg-zinc-100 dark:bg-zinc-700 border border-zinc-300 dark:border-zinc-600 text-zinc-900 dark:text-white form-transition focus:ring-2 focus:ring-emerald-500"
                required
              />
            </div>
            <div>
              <label className="block text-sm text-zinc-400 mb-1">{t("transactions.date")}</label>
              <input
                type="date"
                value={transferForm.date}
                onChange={(e) => setTransferForm((f) => ({ ...f, date: e.target.value }))}
                className="w-full px-4 py-2 rounded-lg bg-zinc-100 dark:bg-zinc-700 border border-zinc-300 dark:border-zinc-600 text-zinc-900 dark:text-white form-transition focus:ring-2 focus:ring-emerald-500"
              />
            </div>
          </div>
          <div>
            <label className="block text-sm text-zinc-400 mb-1">{t("transactions.note")}</label>
            <input
              type="text"
              value={transferForm.note}
              onChange={(e) => setTransferForm((f) => ({ ...f, note: e.target.value }))}
              className="w-full px-4 py-2 rounded-lg bg-zinc-100 dark:bg-zinc-700 border border-zinc-300 dark:border-zinc-600 text-zinc-900 dark:text-white placeholder-zinc-500 form-transition focus:ring-2 focus:ring-emerald-500"
              placeholder={t("transactions.transferDescription")}
            />
          </div>
          <div className="flex gap-2">
            <button
              type="submit"
              className="px-4 py-2 rounded-lg bg-emerald-600 text-white hover:bg-emerald-700 btn-transition"
            >
              {t("transfers.execute")}
            </button>
            <button
              type="button"
              onClick={() => setShowTransferForm(false)}
              className="px-4 py-2 rounded-lg bg-zinc-600 text-white hover:bg-zinc-500 btn-transition"
            >
              {t("common.cancel")}
            </button>
          </div>
        </form>
      )}

      {/* Filters Panel */}
      <div className="rounded-xl bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700 overflow-hidden">
        {/* Filter header - always visible */}
        <button
          type="button"
          onClick={() => setShowFilters(!showFilters)}
          className="w-full p-4 flex items-center justify-between text-sm font-medium text-zinc-500 dark:text-zinc-400 hover:bg-zinc-50 dark:hover:bg-zinc-800/50 transition-colors"
        >
          <div className="flex items-center gap-2">
            <Filter size={16} />
            <span>{t("transactions.filters")}</span>
            {hasActiveFilters && (
              <span className="px-2 py-0.5 rounded-full bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 text-xs">
                {t("transactions.filtersActive")}
              </span>
            )}
          </div>
          <ChevronDown 
            size={16} 
            className={`transition-transform duration-200 ${showFilters ? "rotate-180" : ""}`} 
          />
        </button>

        {/* Collapsible filter content */}
        <div className={`collapse-transition ${showFilters ? "collapse-open" : "collapse-closed"}`}>
          <div className="p-4 pt-0 space-y-4 border-t border-zinc-200 dark:border-zinc-700">
            {/* Preset buttons */}
            <div className="flex flex-wrap gap-2 pt-4">
              <button
                type="button"
                onClick={() => applyPreset("thisMonth")}
                className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-zinc-100 dark:bg-zinc-800 text-zinc-600 dark:text-zinc-300 hover:bg-zinc-200 dark:hover:bg-zinc-700 text-sm btn-transition"
              >
                <Calendar size={14} />
                {t("transactions.thisMonth")}
              </button>
              <button
                type="button"
                onClick={() => applyPreset("thisWeek")}
                className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-zinc-100 dark:bg-zinc-800 text-zinc-600 dark:text-zinc-300 hover:bg-zinc-200 dark:hover:bg-zinc-700 text-sm btn-transition"
              >
                <Calendar size={14} />
                {t("transactions.thisWeek")}
              </button>
              <button
                type="button"
                onClick={() => applyPreset("uncategorized")}
                className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-sm btn-transition ${
                  filters.uncategorized_only
                    ? "bg-amber-500/20 text-amber-600 dark:text-amber-400"
                    : "bg-zinc-100 dark:bg-zinc-800 text-zinc-600 dark:text-zinc-300 hover:bg-zinc-200 dark:hover:bg-zinc-700"
                }`}
              >
                <Lightbulb size={14} />
                {t("transactions.uncategorized")}
              </button>
              {filters.uncategorized_only && transactions.some((tx) => !tx.category_id && (tx.note?.trim()?.length ?? 0) >= 2) && (
                <button
                  type="button"
                  onClick={handleAutoAssignCategories}
                  disabled={autoAssigning}
                  className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-sm bg-purple-500/20 text-purple-600 dark:text-purple-400 hover:bg-purple-500/30 disabled:opacity-50 btn-transition"
                >
                  {autoAssigning ? t("transactions.assigning") : t("transactions.autoAssign")}
                </button>
              )}
            </div>

            <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-6">
              <div>
                <label className="block text-xs text-zinc-400 mb-1">{t("transactions.dateFrom")}</label>
                <input
                  type="date"
                  value={filters.date_from}
                  onChange={(e) => setFilters((f) => ({ ...f, date_from: e.target.value }))}
                  className="w-full px-3 py-2 rounded-lg bg-zinc-100 dark:bg-zinc-800 border border-zinc-300 dark:border-zinc-600 text-zinc-900 dark:text-white text-sm form-transition"
                />
              </div>
              <div>
                <label className="block text-xs text-zinc-400 mb-1">{t("transactions.dateTo")}</label>
                <input
                  type="date"
                  value={filters.date_to}
                  onChange={(e) => setFilters((f) => ({ ...f, date_to: e.target.value }))}
                  className="w-full px-3 py-2 rounded-lg bg-zinc-100 dark:bg-zinc-800 border border-zinc-300 dark:border-zinc-600 text-zinc-900 dark:text-white text-sm form-transition"
                />
              </div>
              <div>
                <label className="block text-xs text-zinc-400 mb-1">{t("transactions.accountFilter")}</label>
                <select
                  value={filters.account_id ?? ""}
                  onChange={(e) =>
                    setFilters((f) => ({
                      ...f,
                      account_id: e.target.value ? +e.target.value : null,
                    }))
                  }
                  className="w-full px-3 py-2 rounded-lg bg-zinc-100 dark:bg-zinc-800 border border-zinc-300 dark:border-zinc-600 text-zinc-900 dark:text-white text-sm form-transition"
                >
                  <option value="">{t("common.all")}</option>
                  {accounts.map((a) => (
                    <option key={a.id} value={a.id}>
                      {a.name}
                    </option>
                  ))}
                </select>
              </div>
              <div>
                <label className="block text-xs text-zinc-400 mb-1">{t("settings.category")}</label>
                <select
                  value={filters.uncategorized_only ? "uncategorized" : (filters.category_id ?? "")}
                  onChange={(e) => {
                    const v = e.target.value;
                    setFilters((f) => ({
                      ...f,
                      uncategorized_only: v === "uncategorized",
                      category_id: v && v !== "uncategorized" ? +v : null,
                    }));
                  }}
                  className="w-full px-3 py-2 rounded-lg bg-zinc-100 dark:bg-zinc-800 border border-zinc-300 dark:border-zinc-600 text-zinc-900 dark:text-white text-sm form-transition"
                >
                  <option value="">{t("common.all")}</option>
                  <option value="uncategorized">{t("transactions.uncategorized")}</option>
                  {categories.map((c) => (
                    <option key={c.id} value={c.id}>
                      {c.name}
                    </option>
                  ))}
                </select>
              </div>
              <div>
                <label className="block text-xs text-zinc-400 mb-1">{t("transactions.typeFilter")}</label>
                <select
                  value={filters.transaction_type}
                  onChange={(e) =>
                    setFilters((f) => ({
                      ...f,
                      transaction_type: e.target.value as "" | "income" | "expense",
                    }))
                  }
                  className="w-full px-3 py-2 rounded-lg bg-zinc-100 dark:bg-zinc-800 border border-zinc-300 dark:border-zinc-600 text-zinc-900 dark:text-white text-sm form-transition"
                >
                  <option value="">{t("common.all")}</option>
                  <option value="income">{t("transactions.income")}</option>
                  <option value="expense">{t("transactions.expense")}</option>
                </select>
              </div>
              <div>
                <label className="block text-xs text-zinc-400 mb-1">{t("transactions.searchNote")}</label>
                <div className="relative">
                  <Search className="absolute left-3 top-1/2 -translate-y-1/2 size-4 text-zinc-400" />
                  <input
                    type="text"
                    value={filters.search_note}
                    onChange={(e) => setFilters((f) => ({ ...f, search_note: e.target.value }))}
                    placeholder={t("transactions.placeholderNote")}
                    className="w-full pl-9 pr-3 py-2 rounded-lg bg-zinc-100 dark:bg-zinc-800 border border-zinc-300 dark:border-zinc-600 text-zinc-900 dark:text-white text-sm placeholder-zinc-500 form-transition"
                  />
                </div>
              </div>
            </div>
            <div className="flex gap-2">
              <button
                type="button"
                onClick={applyFilters}
                className="px-4 py-2 rounded-lg bg-emerald-600 text-white hover:bg-emerald-700 btn-transition text-sm"
              >
                {t("transactions.applyFilters")}
              </button>
              <button
                type="button"
                onClick={handleResetFilters}
                className="px-4 py-2 rounded-lg bg-zinc-200 dark:bg-zinc-700 text-zinc-700 dark:text-zinc-300 hover:bg-zinc-300 dark:hover:bg-zinc-600 btn-transition text-sm"
              >
                {t("transactions.resetFilters")}
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* Transaction Form */}
      {showForm && (
        <form
          ref={formRef}
          onSubmit={handleSubmit}
          className="p-6 rounded-xl bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700 shadow-sm dark:shadow-none space-y-4 animate-slide-down"
        >
          <h4 className="font-medium">
            {editingId ? t("transactions.edit") : t("transactions.new")}
          </h4>
          <div className="grid gap-4 sm:grid-cols-2">
            <div>
              <label className="block text-sm text-zinc-400 mb-1">{t("transactions.typeFilter")}</label>
              <div className="flex gap-2">
                <button
                  type="button"
                  onClick={() => setForm((f) => ({ ...f, transaction_type: "expense", category_id: null }))}
                  className={`flex-1 px-4 py-2 rounded-lg border btn-transition ${
                    form.transaction_type === "expense"
                      ? "bg-red-500/10 border-red-500/30 text-red-500"
                      : "bg-zinc-100 dark:bg-zinc-800 border-zinc-300 dark:border-zinc-600 text-zinc-600 dark:text-zinc-400"
                  }`}
                >
                  {t("transactions.expense")}
                </button>
                <button
                  type="button"
                  onClick={() => setForm((f) => ({ ...f, transaction_type: "income", category_id: null }))}
                  className={`flex-1 px-4 py-2 rounded-lg border btn-transition ${
                    form.transaction_type === "income"
                      ? "bg-emerald-500/10 border-emerald-500/30 text-emerald-500"
                      : "bg-zinc-100 dark:bg-zinc-800 border-zinc-300 dark:border-zinc-600 text-zinc-600 dark:text-zinc-400"
                  }`}
                >
                  {t("transactions.income")}
                </button>
              </div>
            </div>
            <div>
              <label className="block text-sm text-zinc-400 mb-1">{t("transactions.accountFilter")}</label>
              <select
                value={form.account_id}
                onChange={(e) => setForm((f) => ({ ...f, account_id: +e.target.value }))}
                className="w-full px-4 py-2 rounded-lg bg-zinc-100 dark:bg-zinc-700 border border-zinc-300 dark:border-zinc-600 text-zinc-900 dark:text-white form-transition focus:ring-2 focus:ring-emerald-500"
                required
              >
                {accounts.map((a) => (
                  <option key={a.id} value={a.id}>
                    {a.name}
                  </option>
                ))}
              </select>
            </div>
            <div>
              <label className="block text-sm text-zinc-400 mb-1">{t("settings.category")}</label>
              <select
                value={form.category_id ?? ""}
                onChange={(e) =>
                  setForm((f) => ({
                    ...f,
                    category_id: e.target.value ? +e.target.value : null,
                  }))
                }
                className="w-full px-4 py-2 rounded-lg bg-zinc-100 dark:bg-zinc-700 border border-zinc-300 dark:border-zinc-600 text-zinc-900 dark:text-white form-transition focus:ring-2 focus:ring-emerald-500"
              >
                <option value="">{t("transactions.uncategorized")}</option>
                {formCategories.map((c) => (
                  <option key={c.id} value={c.id}>
                    {c.name}
                  </option>
                ))}
              </select>
            </div>
            <div>
              <label className="block text-sm text-zinc-400 mb-1">{t("transactions.amount")}</label>
              <input
                ref={amountInputRef}
                type="number"
                step="0.01"
                min="0"
                value={form.amount}
                onChange={(e) => setForm((f) => ({ ...f, amount: e.target.value }))}
                className="w-full px-4 py-2 rounded-lg bg-zinc-100 dark:bg-zinc-700 border border-zinc-300 dark:border-zinc-600 text-zinc-900 dark:text-white form-transition focus:ring-2 focus:ring-emerald-500"
                placeholder="0"
                required
              />
            </div>
            <div>
              <label className="block text-sm text-zinc-400 mb-1">{t("transactions.date")}</label>
              <input
                type="date"
                value={form.date}
                onChange={(e) => setForm((f) => ({ ...f, date: e.target.value }))}
                className="w-full px-4 py-2 rounded-lg bg-zinc-100 dark:bg-zinc-700 border border-zinc-300 dark:border-zinc-600 text-zinc-900 dark:text-white form-transition focus:ring-2 focus:ring-emerald-500"
              />
            </div>
          </div>
          <div>
            <label className="block text-sm text-zinc-400 mb-1">{t("transactions.note")}</label>
            <input
              type="text"
              value={form.note}
              onChange={(e) => setForm((f) => ({ ...f, note: e.target.value }))}
              className="w-full px-4 py-2 rounded-lg bg-zinc-100 dark:bg-zinc-700 border border-zinc-300 dark:border-zinc-600 text-zinc-900 dark:text-white placeholder-zinc-500 form-transition focus:ring-2 focus:ring-emerald-500"
              placeholder={t("transactions.placeholderDescription")}
            />
            
            {/* ML Category Suggestion */}
            {suggestedCategory && (
              <div className="mt-2 p-3 rounded-lg bg-amber-500/10 border border-amber-500/20 animate-fade-in">
                <div className="flex items-center justify-between gap-3">
                  <div className="flex items-center gap-2 text-sm">
                    <Lightbulb size={16} className="text-amber-500" />
                    <span className="text-zinc-600 dark:text-zinc-300">
                      {t("transactions.suggestLabel", { name: suggestedCategory.category_name })}
                    </span>
                    <span className="text-xs text-zinc-400">
                      ({Math.round(suggestedCategory.confidence * 100)}%)
                    </span>
                  </div>
                  <div className="flex gap-1">
                    <button
                      type="button"
                      onClick={() => {
                        setForm((f) => ({ ...f, category_id: suggestedCategory.category_id }));
                        setSuggestedCategory(null);
                        showToast(t("transactions.applyCategory"), "success");
                      }}
                      className="p-1.5 rounded bg-emerald-500/10 text-emerald-600 hover:bg-emerald-500/20 btn-transition"
                      title={t("transactions.accept")}
                    >
                      <Check size={14} />
                    </button>
                    <button
                      type="button"
                      onClick={() => setSuggestedCategory(null)}
                      className="p-1.5 rounded bg-zinc-500/10 text-zinc-500 hover:bg-zinc-500/20 btn-transition"
                      title={t("transactions.ignore")}
                    >
                      <X size={14} />
                    </button>
                  </div>
                </div>
              </div>
            )}
          </div>
          <div className="flex gap-2">
            <button
              type="submit"
              className="px-4 py-2 rounded-lg bg-emerald-600 text-white hover:bg-emerald-700 btn-transition"
            >
              {editingId ? t("common.save") : t("common.add")}
            </button>
            <button
              type="button"
              onClick={() => {
                setShowForm(false);
                setEditingId(null);
              }}
              className="px-4 py-2 rounded-lg bg-zinc-600 text-white hover:bg-zinc-500 btn-transition"
            >
              {t("common.cancel")}
            </button>
          </div>
        </form>
      )}

      {/* Transactions Table */}
      <div className="rounded-xl border border-zinc-200 dark:border-zinc-700 overflow-hidden bg-white dark:bg-transparent">
        <table className="w-full">
          <thead>
            <tr className="bg-zinc-100 dark:bg-zinc-800 text-left text-sm text-zinc-500 dark:text-zinc-400">
              <th className="px-4 py-3">{t("transactions.date")}</th>
              <th className="px-4 py-3">{t("transactions.accountFilter")}</th>
              <th className="px-4 py-3">{t("settings.category")}</th>
              <th className="px-4 py-3">{t("transactions.amount")}</th>
              <th className="px-4 py-3 hidden sm:table-cell">{t("transactions.note")}</th>
              <th className="px-4 py-3 w-20"></th>
            </tr>
          </thead>
          <tbody>
            {loading ? (
              Array.from({ length: 6 }).map((_, i) => (
                <tr key={i} className="border-t border-zinc-200 dark:border-zinc-700">
                  <td className="px-4 py-3"><div className="h-5 w-20 rounded bg-zinc-200 dark:bg-zinc-700 animate-pulse" /></td>
                  <td className="px-4 py-3"><div className="h-5 w-24 rounded bg-zinc-200 dark:bg-zinc-700 animate-pulse" /></td>
                  <td className="px-4 py-3"><div className="h-5 w-20 rounded bg-zinc-200 dark:bg-zinc-700 animate-pulse" /></td>
                  <td className="px-4 py-3"><div className="h-5 w-16 rounded bg-zinc-200 dark:bg-zinc-700 animate-pulse" /></td>
                  <td className="px-4 py-3 hidden sm:table-cell"><div className="h-5 w-32 rounded bg-zinc-200 dark:bg-zinc-700 animate-pulse" /></td>
                  <td className="px-4 py-3 w-20" />
                </tr>
              ))
            ) : (
            transactions.map((tx) => (
              <tr
                key={tx.id}
                className={`border-t border-zinc-200 dark:border-zinc-700 hover:bg-zinc-50 dark:hover:bg-zinc-800/50 transition-colors ${
                  deletingId === tx.id ? "row-deleting" : ""
                }`}
              >
                <td className="px-4 py-3 text-zinc-600 dark:text-zinc-300">{formatDateDisplay(tx.date)}</td>
                <td className="px-4 py-3">
                  <div className="flex items-center gap-2">
                    <CreditCard size={14} className="text-zinc-400" />
                    <span>{tx.account_name}</span>
                  </div>
                </td>
                <td className="px-4 py-3">
                  {tx.category_name ? (
                    <span className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${
                      tx.transaction_type === "income" 
                        ? "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400"
                        : "bg-zinc-100 dark:bg-zinc-800 text-zinc-600 dark:text-zinc-400"
                    }`}>
                      {tx.category_name}
                    </span>
                  ) : (
                    <select
                      value=""
                      onChange={(e) => {
                        const v = e.target.value ? +e.target.value : 0;
                        if (v) handleQuickAssignCategory(tx, v);
                      }}
                      disabled={assigningCategoryId === tx.id}
                      className="min-w-[120px] px-2 py-1 rounded border border-zinc-300 dark:border-zinc-600 bg-white dark:bg-zinc-800 text-zinc-700 dark:text-zinc-200 text-xs focus:ring-2 focus:ring-emerald-500 disabled:opacity-50"
                    >
                      <option value="">{t("transactions.selectCategoryPlaceholder")}</option>
                      {(tx.transaction_type === "income" ? incomeCategories : expenseCategories).map((c) => (
                        <option key={c.id} value={c.id}>
                          {c.name}
                        </option>
                      ))}
                    </select>
                  )}
                </td>
                <td
                  className={`px-4 py-3 font-semibold ${
                    tx.transaction_type === "income" ? "text-emerald-500" : "text-red-500"
                  }`}
                >
                  {formatAmount(tx.amount, tx.transaction_type)} ₸
                </td>
                <td className="px-4 py-3 text-zinc-400 hidden sm:table-cell max-w-[200px] truncate">
                  {tx.note ?? "—"}
                </td>
                <td className="px-4 py-3">
                  <div className="flex gap-1">
                    <button
                      onClick={() => handleEdit(tx)}
                      className="p-1.5 rounded text-zinc-400 hover:bg-zinc-200 dark:hover:bg-zinc-700 hover:text-zinc-600 dark:hover:text-white btn-transition"
                    >
                      <Pencil size={14} />
                    </button>
                    <button
                      onClick={() => handleDeleteClick(tx.id)}
                      className="p-1.5 rounded text-zinc-400 hover:bg-red-500/20 hover:text-red-500 btn-transition"
                    >
                      <Trash2 size={14} />
                    </button>
                  </div>
                </td>
              </tr>
            ))
            )}
          </tbody>
        </table>
      </div>

      {!loading && hasMore && transactions.length > 0 && (
        <div className="flex justify-center py-4">
          <button
            type="button"
            onClick={loadMore}
            disabled={loadingMore}
            className="px-4 py-2 rounded-lg border border-zinc-300 dark:border-zinc-600 bg-white dark:bg-zinc-800 text-zinc-700 dark:text-zinc-200 hover:bg-zinc-50 dark:hover:bg-zinc-700 btn-transition disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {loadingMore ? t("common.loading") : t("transactions.loadMore")}
          </button>
        </div>
      )}

      {!loading && transactions.length === 0 && !showForm && (
        <EmptyState
          icon={CreditCard}
          title={t("transactions.emptyTitle")}
          description={t("transactions.emptyDesc")}
          action={
            <button
              type="button"
              onClick={() => {
                setShowForm(true);
                setEditingId(null);
                setForm({
                  account_id: accounts[0]?.id ?? 0,
                  category_id: null,
                  amount: "",
                  transaction_type: "expense",
                  note: "",
                  date: new Date().toISOString().slice(0, 10),
                });
              }}
              className="inline-flex items-center gap-2 px-4 py-2 rounded-lg bg-emerald-600 text-white hover:bg-emerald-700 btn-transition"
            >
              <Plus size={18} />
              {t("quickStats.addFirst")}
            </button>
          }
        />
      )}

      <ConfirmDialog
        open={deleteConfirmId !== null}
        title={t("transactions.deleteConfirmTitle")}
        message={t("transactions.deleteConfirmMessage")}
        confirmLabel={t("common.delete")}
        variant="danger"
        onConfirm={handleDeleteConfirm}
        onCancel={() => setDeleteConfirmId(null)}
      />
    </div>
  );
}
