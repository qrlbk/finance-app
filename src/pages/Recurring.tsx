import { useEffect, useState } from "react";
import { Plus, Pencil, Trash2, Play, Pause, RefreshCw, Calendar, Repeat } from "lucide-react";
import { api, type RecurringPayment, type Account, type Category } from "../lib/api";
import { ConfirmDialog } from "../components/ui/ConfirmDialog";
import { EmptyState } from "../components/ui/EmptyState";
import { useToast } from "../components/ui/Toast";

function formatAmount(amount: number, type: string) {
  const formatted = new Intl.NumberFormat("ru-KZ", {
    style: "decimal",
    minimumFractionDigits: 0,
    maximumFractionDigits: 0,
  }).format(amount);
  return type === "income" ? `+${formatted}` : `-${formatted}`;
}

function formatDate(dateStr: string) {
  return new Date(dateStr + "T12:00:00").toLocaleDateString("ru-KZ", {
    day: "2-digit",
    month: "2-digit",
    year: "numeric",
  });
}

const frequencyLabels: Record<string, string> = {
  daily: "Ежедневно",
  weekly: "Еженедельно",
  monthly: "Ежемесячно",
  yearly: "Ежегодно",
};

export function Recurring() {
  const { showToast } = useToast();
  const [payments, setPayments] = useState<RecurringPayment[]>([]);
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [categories, setCategories] = useState<Category[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showForm, setShowForm] = useState(false);
  const [editingId, setEditingId] = useState<number | null>(null);
  const [deleteConfirmId, setDeleteConfirmId] = useState<number | null>(null);
  const [processing, setProcessing] = useState(false);

  const [form, setForm] = useState({
    account_id: 0,
    category_id: null as number | null,
    amount: "",
    payment_type: "expense" as "income" | "expense",
    frequency: "monthly",
    next_date: new Date().toISOString().slice(0, 10),
    end_date: "",
    note: "",
    is_active: true,
  });

  const loadData = async () => {
    try {
      setLoading(true);
      setError(null);
      const [pmts, accs, cats] = await Promise.all([
        api.getRecurringPayments(),
        api.getAccounts(),
        api.getCategories(),
      ]);
      setPayments(pmts);
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

  useEffect(() => {
    loadData();
  }, []);

  const incomeCategories = categories.filter((c) => c.category_type === "income");
  const expenseCategories = categories.filter((c) => c.category_type === "expense");
  const formCategories = form.payment_type === "income" ? incomeCategories : expenseCategories;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    const amount = parseFloat(form.amount);
    if (isNaN(amount) || amount <= 0) {
      showToast("Введите положительную сумму", "error");
      return;
    }
    if (!form.account_id || !accounts.some((a) => a.id === form.account_id)) {
      showToast("Выберите счёт", "error");
      return;
    }

    try {
      if (editingId) {
        await api.updateRecurring({
          id: editingId,
          account_id: form.account_id,
          category_id: form.category_id,
          amount,
          payment_type: form.payment_type,
          frequency: form.frequency,
          next_date: form.next_date,
          end_date: form.end_date || null,
          note: form.note || null,
          is_active: form.is_active,
        });
        showToast("Платёж обновлён", "success");
      } else {
        await api.createRecurring({
          account_id: form.account_id,
          category_id: form.category_id,
          amount,
          payment_type: form.payment_type,
          frequency: form.frequency,
          next_date: form.next_date,
          end_date: form.end_date || null,
          note: form.note || null,
        });
        showToast("Платёж создан", "success");
      }
      resetForm();
      loadData();
    } catch (e) {
      setError(String(e));
      showToast("Ошибка при сохранении", "error");
    }
  };

  const resetForm = () => {
    setForm({
      account_id: accounts[0]?.id ?? 0,
      category_id: null,
      amount: "",
      payment_type: "expense",
      frequency: "monthly",
      next_date: new Date().toISOString().slice(0, 10),
      end_date: "",
      note: "",
      is_active: true,
    });
    setShowForm(false);
    setEditingId(null);
  };

  const handleEdit = (p: RecurringPayment) => {
    setForm({
      account_id: p.account_id,
      category_id: p.category_id,
      amount: String(p.amount),
      payment_type: p.payment_type as "income" | "expense",
      frequency: p.frequency,
      next_date: p.next_date,
      end_date: p.end_date ?? "",
      note: p.note ?? "",
      is_active: p.is_active,
    });
    setEditingId(p.id);
    setShowForm(true);
  };

  const handleToggleActive = async (p: RecurringPayment) => {
    try {
      await api.updateRecurring({
        id: p.id,
        account_id: p.account_id,
        category_id: p.category_id,
        amount: p.amount,
        payment_type: p.payment_type,
        frequency: p.frequency,
        next_date: p.next_date,
        end_date: p.end_date,
        note: p.note,
        is_active: !p.is_active,
      });
      showToast(p.is_active ? "Платёж приостановлен" : "Платёж активирован", "success");
      loadData();
    } catch (e) {
      showToast("Ошибка", "error");
    }
  };

  const handleDeleteConfirm = async () => {
    if (deleteConfirmId === null) return;
    try {
      await api.deleteRecurring(deleteConfirmId);
      setDeleteConfirmId(null);
      loadData();
      showToast("Платёж удалён", "success");
    } catch (e) {
      setError(String(e));
      showToast("Ошибка при удалении", "error");
    }
  };

  const handleProcessPayments = async () => {
    try {
      setProcessing(true);
      const created = await api.processRecurringPayments();
      if (created.length > 0) {
        showToast(`Создано транзакций: ${created.length}`, "success");
      } else {
        showToast("Нет платежей для обработки", "info");
      }
      loadData();
    } catch (e) {
      showToast("Ошибка обработки", "error");
    } finally {
      setProcessing(false);
    }
  };

  const activePayments = payments.filter(p => p.is_active);
  const inactivePayments = payments.filter(p => !p.is_active);

  return (
    <div className="space-y-6">
      {error && (
        <div className="p-4 rounded-lg bg-red-500/10 text-red-500 border border-red-500/20 animate-shake">
          {error}
        </div>
      )}

      <div className="flex justify-between items-center">
        <h3 className="text-lg font-medium">Повторяющиеся платежи</h3>
        <div className="flex gap-2">
          <button
            onClick={handleProcessPayments}
            disabled={processing}
            className="flex items-center gap-2 px-4 py-2 rounded-lg bg-zinc-600 text-white hover:bg-zinc-500 btn-transition disabled:opacity-50"
          >
            <RefreshCw size={18} className={processing ? "animate-spin" : ""} />
            <span className="hidden sm:inline">Обработать</span>
          </button>
          <button
            type="button"
            onClick={() => {
              setShowForm(true);
              setEditingId(null);
              setForm({
                account_id: accounts[0]?.id ?? 0,
                category_id: null,
                amount: "",
                payment_type: "expense",
                frequency: "monthly",
                next_date: new Date().toISOString().slice(0, 10),
                end_date: "",
                note: "",
                is_active: true,
              });
            }}
            disabled={accounts.length === 0}
            className="flex items-center gap-2 px-4 py-2 rounded-lg bg-emerald-600 text-white hover:bg-emerald-700 btn-transition disabled:opacity-50 disabled:pointer-events-none"
          >
            <Plus size={18} />
            Добавить
          </button>
        </div>
      </div>
      {accounts.length === 0 && !loading && (
        <p className="text-sm text-zinc-500 dark:text-zinc-400">
          Сначала добавьте счёт в разделе «Счета».
        </p>
      )}

      {/* Form */}
      {showForm && (
        <form
          onSubmit={handleSubmit}
          className="p-6 rounded-xl bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700 shadow-sm dark:shadow-none space-y-4 animate-slide-down"
        >
          <h4 className="font-medium">
            {editingId ? "Редактировать платёж" : "Новый повторяющийся платёж"}
          </h4>
          
          <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
            <div>
              <label className="block text-sm text-zinc-400 mb-1">Тип</label>
              <div className="flex gap-2">
                <button
                  type="button"
                  onClick={() => setForm((f) => ({ ...f, payment_type: "expense", category_id: null }))}
                  className={`flex-1 px-4 py-2 rounded-lg border btn-transition ${
                    form.payment_type === "expense"
                      ? "bg-red-500/10 border-red-500/30 text-red-500"
                      : "bg-zinc-100 dark:bg-zinc-800 border-zinc-300 dark:border-zinc-600 text-zinc-600 dark:text-zinc-400"
                  }`}
                >
                  Расход
                </button>
                <button
                  type="button"
                  onClick={() => setForm((f) => ({ ...f, payment_type: "income", category_id: null }))}
                  className={`flex-1 px-4 py-2 rounded-lg border btn-transition ${
                    form.payment_type === "income"
                      ? "bg-emerald-500/10 border-emerald-500/30 text-emerald-500"
                      : "bg-zinc-100 dark:bg-zinc-800 border-zinc-300 dark:border-zinc-600 text-zinc-600 dark:text-zinc-400"
                  }`}
                >
                  Доход
                </button>
              </div>
            </div>

            <div>
              <label className="block text-sm text-zinc-400 mb-1">Счёт</label>
              <select
                value={form.account_id}
                onChange={(e) => setForm((f) => ({ ...f, account_id: +e.target.value }))}
                className="w-full px-4 py-2 rounded-lg bg-zinc-100 dark:bg-zinc-700 border border-zinc-300 dark:border-zinc-600 text-zinc-900 dark:text-white form-transition focus:ring-2 focus:ring-emerald-500"
                required
              >
                {accounts.map((a) => (
                  <option key={a.id} value={a.id}>{a.name}</option>
                ))}
              </select>
            </div>

            <div>
              <label className="block text-sm text-zinc-400 mb-1">Категория</label>
              <select
                value={form.category_id ?? ""}
                onChange={(e) => setForm((f) => ({ ...f, category_id: e.target.value ? +e.target.value : null }))}
                className="w-full px-4 py-2 rounded-lg bg-zinc-100 dark:bg-zinc-700 border border-zinc-300 dark:border-zinc-600 text-zinc-900 dark:text-white form-transition focus:ring-2 focus:ring-emerald-500"
              >
                <option value="">Без категории</option>
                {formCategories.map((c) => (
                  <option key={c.id} value={c.id}>{c.name}</option>
                ))}
              </select>
            </div>

            <div>
              <label className="block text-sm text-zinc-400 mb-1">Сумма</label>
              <input
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
              <label className="block text-sm text-zinc-400 mb-1">Частота</label>
              <select
                value={form.frequency}
                onChange={(e) => setForm((f) => ({ ...f, frequency: e.target.value }))}
                className="w-full px-4 py-2 rounded-lg bg-zinc-100 dark:bg-zinc-700 border border-zinc-300 dark:border-zinc-600 text-zinc-900 dark:text-white form-transition focus:ring-2 focus:ring-emerald-500"
              >
                <option value="daily">Ежедневно</option>
                <option value="weekly">Еженедельно</option>
                <option value="monthly">Ежемесячно</option>
                <option value="yearly">Ежегодно</option>
              </select>
            </div>

            <div>
              <label className="block text-sm text-zinc-400 mb-1">Следующая дата</label>
              <input
                type="date"
                value={form.next_date}
                onChange={(e) => setForm((f) => ({ ...f, next_date: e.target.value }))}
                className="w-full px-4 py-2 rounded-lg bg-zinc-100 dark:bg-zinc-700 border border-zinc-300 dark:border-zinc-600 text-zinc-900 dark:text-white form-transition focus:ring-2 focus:ring-emerald-500"
                required
              />
            </div>

            <div>
              <label className="block text-sm text-zinc-400 mb-1">Дата окончания (опц.)</label>
              <input
                type="date"
                value={form.end_date}
                onChange={(e) => setForm((f) => ({ ...f, end_date: e.target.value }))}
                className="w-full px-4 py-2 rounded-lg bg-zinc-100 dark:bg-zinc-700 border border-zinc-300 dark:border-zinc-600 text-zinc-900 dark:text-white form-transition focus:ring-2 focus:ring-emerald-500"
              />
            </div>

            <div className="sm:col-span-2">
              <label className="block text-sm text-zinc-400 mb-1">Заметка</label>
              <input
                type="text"
                value={form.note}
                onChange={(e) => setForm((f) => ({ ...f, note: e.target.value }))}
                className="w-full px-4 py-2 rounded-lg bg-zinc-100 dark:bg-zinc-700 border border-zinc-300 dark:border-zinc-600 text-zinc-900 dark:text-white placeholder-zinc-500 form-transition focus:ring-2 focus:ring-emerald-500"
                placeholder="Описание платежа"
              />
            </div>

            {editingId && (
              <div className="flex items-center gap-2">
                <input
                  type="checkbox"
                  id="is_active"
                  checked={form.is_active}
                  onChange={(e) => setForm((f) => ({ ...f, is_active: e.target.checked }))}
                  className="w-4 h-4 rounded border-zinc-300 dark:border-zinc-600"
                />
                <label htmlFor="is_active" className="text-sm text-zinc-600 dark:text-zinc-300">
                  Активен
                </label>
              </div>
            )}
          </div>

          <div className="flex gap-2">
            <button
              type="submit"
              className="px-4 py-2 rounded-lg bg-emerald-600 text-white hover:bg-emerald-700 btn-transition"
            >
              {editingId ? "Сохранить" : "Добавить"}
            </button>
            <button
              type="button"
              onClick={resetForm}
              className="px-4 py-2 rounded-lg bg-zinc-600 text-white hover:bg-zinc-500 btn-transition"
            >
              Отмена
            </button>
          </div>
        </form>
      )}

      {/* Active Payments */}
      {activePayments.length > 0 && (
        <div className="space-y-4">
          <h4 className="text-sm font-medium text-zinc-500 dark:text-zinc-400">Активные платежи</h4>
          <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
            {activePayments.map((p) => (
              <div
                key={p.id}
                className="p-4 rounded-xl bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700 hover:border-zinc-300 dark:hover:border-zinc-600 transition-colors"
              >
                <div className="flex justify-between items-start mb-3">
                  <div className="flex items-center gap-2">
                    <Repeat size={16} className="text-zinc-400" />
                    <span className="text-xs text-zinc-500 dark:text-zinc-400">
                      {frequencyLabels[p.frequency]}
                    </span>
                  </div>
                  <div className="flex gap-1">
                    <button
                      onClick={() => handleToggleActive(p)}
                      className="p-1.5 rounded text-zinc-400 hover:bg-amber-500/20 hover:text-amber-500 btn-transition"
                      title="Приостановить"
                    >
                      <Pause size={14} />
                    </button>
                    <button
                      onClick={() => handleEdit(p)}
                      className="p-1.5 rounded text-zinc-400 hover:bg-zinc-200 dark:hover:bg-zinc-700 hover:text-zinc-600 dark:hover:text-white btn-transition"
                    >
                      <Pencil size={14} />
                    </button>
                    <button
                      onClick={() => setDeleteConfirmId(p.id)}
                      className="p-1.5 rounded text-zinc-400 hover:bg-red-500/20 hover:text-red-500 btn-transition"
                    >
                      <Trash2 size={14} />
                    </button>
                  </div>
                </div>

                <p className={`text-2xl font-bold mb-1 ${
                  p.payment_type === "income" ? "text-emerald-500" : "text-zinc-900 dark:text-white"
                }`}>
                  {formatAmount(p.amount, p.payment_type)} ₸
                </p>

                <p className="text-sm text-zinc-600 dark:text-zinc-300 mb-2">
                  {p.note || p.category_name || "Без описания"}
                </p>

                <div className="flex items-center gap-2 text-xs text-zinc-400">
                  <Calendar size={12} />
                  <span>Следующий: {formatDate(p.next_date)}</span>
                </div>
                
                <p className="text-xs text-zinc-400 mt-1">{p.account_name}</p>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Inactive Payments */}
      {inactivePayments.length > 0 && (
        <div className="space-y-4">
          <h4 className="text-sm font-medium text-zinc-500 dark:text-zinc-400">Приостановленные</h4>
          <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
            {inactivePayments.map((p) => (
              <div
                key={p.id}
                className="p-4 rounded-xl bg-zinc-50 dark:bg-zinc-900/50 border border-zinc-200 dark:border-zinc-800 opacity-60"
              >
                <div className="flex justify-between items-start mb-3">
                  <div className="flex items-center gap-2">
                    <Repeat size={16} className="text-zinc-400" />
                    <span className="text-xs text-zinc-500">{frequencyLabels[p.frequency]}</span>
                  </div>
                  <div className="flex gap-1">
                    <button
                      onClick={() => handleToggleActive(p)}
                      className="p-1.5 rounded text-zinc-400 hover:bg-emerald-500/20 hover:text-emerald-500 btn-transition"
                      title="Активировать"
                    >
                      <Play size={14} />
                    </button>
                    <button
                      onClick={() => handleEdit(p)}
                      className="p-1.5 rounded text-zinc-400 hover:bg-zinc-200 dark:hover:bg-zinc-700 btn-transition"
                    >
                      <Pencil size={14} />
                    </button>
                    <button
                      onClick={() => setDeleteConfirmId(p.id)}
                      className="p-1.5 rounded text-zinc-400 hover:bg-red-500/20 hover:text-red-500 btn-transition"
                    >
                      <Trash2 size={14} />
                    </button>
                  </div>
                </div>

                <p className="text-2xl font-bold mb-1 text-zinc-400">
                  {formatAmount(p.amount, p.payment_type)} ₸
                </p>

                <p className="text-sm text-zinc-500 mb-2">
                  {p.note || p.category_name || "Без описания"}
                </p>

                <p className="text-xs text-zinc-400">{p.account_name}</p>
              </div>
            ))}
          </div>
        </div>
      )}

      {!loading && payments.length === 0 && !showForm && (
        <EmptyState
          icon={Repeat}
          title="Нет повторяющихся платежей"
          description="Настройте автоматические платежи для подписок, ЖКХ, или регулярного дохода."
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
                  payment_type: "expense",
                  frequency: "monthly",
                  next_date: new Date().toISOString().slice(0, 10),
                  end_date: "",
                  note: "",
                  is_active: true,
                });
              }}
              disabled={accounts.length === 0}
              className="inline-flex items-center gap-2 px-4 py-2 rounded-lg bg-emerald-600 text-white hover:bg-emerald-700 btn-transition disabled:opacity-50 disabled:pointer-events-none"
            >
              <Plus size={18} />
              Добавить платёж
            </button>
          }
        />
      )}

      {loading && (
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {Array.from({ length: 3 }).map((_, i) => (
            <div key={i} className="p-4 rounded-xl bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700">
              <div className="h-4 w-24 rounded bg-zinc-200 dark:bg-zinc-700 animate-pulse mb-3" />
              <div className="h-8 w-32 rounded bg-zinc-200 dark:bg-zinc-700 animate-pulse mb-2" />
              <div className="h-4 w-40 rounded bg-zinc-200 dark:bg-zinc-700 animate-pulse" />
            </div>
          ))}
        </div>
      )}

      <ConfirmDialog
        open={deleteConfirmId !== null}
        title="Удалить платёж?"
        message="Эта операция необратима. Уже созданные транзакции не будут удалены."
        confirmLabel="Удалить"
        variant="danger"
        onConfirm={handleDeleteConfirm}
        onCancel={() => setDeleteConfirmId(null)}
      />
    </div>
  );
}
