import { useEffect, useState } from "react";
import { Plus, Pencil, Trash2, Banknote, CreditCard, PiggyBank, Wallet } from "lucide-react";
import { api, type Account } from "../lib/api";
import { ConfirmDialog } from "../components/ui/ConfirmDialog";
import { EmptyState } from "../components/ui/EmptyState";
import { useToast } from "../components/ui/Toast";

const accountTypeLabels: Record<string, string> = {
  cash: "Наличные",
  card: "Карта",
  savings: "Накопления",
};

const accountTypeIcons: Record<string, typeof Banknote> = {
  cash: Banknote,
  card: CreditCard,
  savings: PiggyBank,
};

// Gradient styles for different account types
const accountTypeStyles: Record<string, { gradient: string; iconBg: string; iconColor: string }> = {
  cash: {
    gradient: "from-emerald-500/10 to-emerald-600/5",
    iconBg: "bg-emerald-500/20",
    iconColor: "text-emerald-500",
  },
  card: {
    gradient: "from-blue-500/10 to-blue-600/5",
    iconBg: "bg-blue-500/20",
    iconColor: "text-blue-500",
  },
  savings: {
    gradient: "from-purple-500/10 to-purple-600/5",
    iconBg: "bg-purple-500/20",
    iconColor: "text-purple-500",
  },
};

function formatAmount(amount: number, currency: string = "KZT") {
  return new Intl.NumberFormat("ru-KZ", {
    style: "decimal",
    minimumFractionDigits: 0,
    maximumFractionDigits: 0,
  }).format(amount) + ` ${currency}`;
}

export function Accounts() {
  const { showToast } = useToast();
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showForm, setShowForm] = useState(false);
  const [editingId, setEditingId] = useState<number | null>(null);
  const [deleteConfirm, setDeleteConfirm] = useState<{ id: number } | null>(null);
  const [form, setForm] = useState({ name: "", account_type: "card", currency: "KZT" });

  const loadAccounts = async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await api.getAccounts();
      setAccounts(data);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadAccounts();
  }, []);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      if (editingId) {
        await api.updateAccount({
          id: editingId,
          name: form.name,
          account_type: form.account_type,
          currency: form.currency,
        });
        showToast("Счёт обновлён", "success");
      } else {
        await api.createAccount(form);
        showToast("Счёт создан", "success");
      }
      setForm({ name: "", account_type: "card", currency: "KZT" });
      setShowForm(false);
      setEditingId(null);
      loadAccounts();
    } catch (e) {
      setError(String(e));
      showToast("Ошибка при сохранении", "error");
    }
  };

  const handleEdit = (acc: Account) => {
    setForm({
      name: acc.name,
      account_type: acc.account_type,
      currency: acc.currency,
    });
    setEditingId(acc.id);
    setShowForm(true);
  };

  const handleDeleteClick = (id: number) => {
    setDeleteConfirm({ id });
  };

  const handleDeleteConfirm = async () => {
    if (!deleteConfirm) return;
    try {
      await api.deleteAccount(deleteConfirm.id);
      setDeleteConfirm(null);
      loadAccounts();
      showToast("Счёт удалён", "success");
    } catch (e) {
      setError(String(e));
      showToast("Ошибка при удалении", "error");
    }
  };

  const cancelForm = () => {
    setShowForm(false);
    setEditingId(null);
    setForm({ name: "", account_type: "card", currency: "KZT" });
  };

  // Calculate total balance
  const totalBalance = accounts.reduce((sum, acc) => sum + acc.balance, 0);

  return (
    <div className="space-y-6">
      {error && (
        <div className="p-4 rounded-lg bg-red-500/10 text-red-500 border border-red-500/20 animate-shake">
          {error}
        </div>
      )}

      <div className="flex justify-between items-center">
        <div>
          <h3 className="text-lg font-medium">Мои счета</h3>
          {accounts.length > 0 && (
            <p className="text-sm text-zinc-500 dark:text-zinc-400 mt-1">
              Общий баланс: <span className="font-semibold text-emerald-500">{formatAmount(totalBalance)}</span>
            </p>
          )}
        </div>
        <button
          onClick={() => {
            setShowForm(true);
            setEditingId(null);
            setForm({ name: "", account_type: "card", currency: "KZT" });
          }}
          className="flex items-center gap-2 px-4 py-2 rounded-lg bg-emerald-600 text-white hover:bg-emerald-700 btn-transition"
        >
          <Plus size={18} />
          Добавить счёт
        </button>
      </div>

      {showForm && (
        <form
          onSubmit={handleSubmit}
          className="p-6 rounded-xl bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700 shadow-sm dark:shadow-none space-y-4 animate-slide-down"
        >
          <h4 className="font-medium">{editingId ? "Редактировать счёт" : "Новый счёт"}</h4>
          <div>
            <label className="block text-sm text-zinc-400 mb-1">Название</label>
            <input
              type="text"
              value={form.name}
              onChange={(e) => setForm((f) => ({ ...f, name: e.target.value }))}
              className="w-full px-4 py-2 rounded-lg bg-zinc-100 dark:bg-zinc-700 border border-zinc-300 dark:border-zinc-600 text-zinc-900 dark:text-white placeholder-zinc-500 form-transition focus:ring-2 focus:ring-emerald-500 focus:border-transparent"
              placeholder="Например: Основная карта"
              autoFocus
              required
            />
          </div>
          <div>
            <label className="block text-sm text-zinc-400 mb-2">Тип</label>
            <div className="grid grid-cols-3 gap-3">
              {Object.entries(accountTypeLabels).map(([value, label]) => {
                const Icon = accountTypeIcons[value] ?? Banknote;
                const styles = accountTypeStyles[value] ?? accountTypeStyles.card;
                const isSelected = form.account_type === value;
                return (
                  <button
                    key={value}
                    type="button"
                    onClick={() => setForm((f) => ({ ...f, account_type: value }))}
                    className={`p-4 rounded-xl border-2 btn-transition flex flex-col items-center gap-2 ${
                      isSelected
                        ? `border-emerald-500 bg-emerald-500/10`
                        : `border-zinc-200 dark:border-zinc-700 hover:border-zinc-300 dark:hover:border-zinc-600`
                    }`}
                  >
                    <div className={`p-2 rounded-lg ${styles.iconBg}`}>
                      <Icon size={20} className={styles.iconColor} />
                    </div>
                    <span className={`text-sm font-medium ${isSelected ? "text-emerald-500" : "text-zinc-600 dark:text-zinc-400"}`}>
                      {label}
                    </span>
                  </button>
                );
              })}
            </div>
          </div>
          <div>
            <label className="block text-sm text-zinc-400 mb-1">Валюта</label>
            <select
              value={form.currency}
              onChange={(e) => setForm((f) => ({ ...f, currency: e.target.value }))}
              className="w-full px-4 py-2 rounded-lg bg-zinc-100 dark:bg-zinc-700 border border-zinc-300 dark:border-zinc-600 text-zinc-900 dark:text-white form-transition focus:ring-2 focus:ring-emerald-500"
            >
              <option value="KZT">KZT - Тенге</option>
              <option value="USD">USD - Доллар</option>
              <option value="EUR">EUR - Евро</option>
              <option value="RUB">RUB - Рубль</option>
            </select>
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
              onClick={cancelForm}
              className="px-4 py-2 rounded-lg bg-zinc-600 text-white hover:bg-zinc-500 btn-transition"
            >
              Отмена
            </button>
          </div>
        </form>
      )}

      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {loading ? (
          Array.from({ length: 6 }).map((_, i) => (
            <div
              key={i}
              className="p-6 rounded-xl bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700 animate-pulse"
            >
              <div className="flex justify-between items-start">
                <div className="flex items-center gap-3">
                  <div className="p-2 rounded-lg bg-zinc-200 dark:bg-zinc-700 w-10 h-10" />
                  <div className="space-y-2">
                    <div className="h-4 w-28 rounded bg-zinc-200 dark:bg-zinc-700" />
                    <div className="h-3 w-20 rounded bg-zinc-200 dark:bg-zinc-700" />
                  </div>
                </div>
              </div>
              <div className="mt-4 h-6 w-24 rounded bg-zinc-200 dark:bg-zinc-700" />
            </div>
          ))
        ) : (
        accounts.map((acc, index) => {
          const Icon = accountTypeIcons[acc.account_type] ?? Banknote;
          const styles = accountTypeStyles[acc.account_type] ?? accountTypeStyles.card;
          return (
            <div
              key={acc.id}
              className={`p-6 rounded-xl bg-gradient-to-br ${styles.gradient} bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700 shadow-sm dark:shadow-none card-hover animate-stagger animate-stagger-${Math.min(index + 1, 6)}`}
            >
              <div className="flex justify-between items-start">
                <div className="flex items-center gap-3">
                  <div className={`p-2.5 rounded-xl ${styles.iconBg}`}>
                    <Icon size={24} className={styles.iconColor} />
                  </div>
                  <div>
                    <h4 className="font-medium text-zinc-900 dark:text-zinc-100">{acc.name}</h4>
                    <p className="text-sm text-zinc-500 dark:text-zinc-400">
                      {accountTypeLabels[acc.account_type] ?? acc.account_type}
                    </p>
                  </div>
                </div>
                <div className="flex gap-1">
                  <button
                    onClick={() => handleEdit(acc)}
                    className="p-2 rounded-lg text-zinc-400 hover:bg-zinc-200 dark:hover:bg-zinc-700 hover:text-zinc-600 dark:hover:text-white btn-transition"
                  >
                    <Pencil size={16} />
                  </button>
                  <button
                    onClick={() => handleDeleteClick(acc.id)}
                    className="p-2 rounded-lg text-zinc-400 hover:bg-red-500/20 hover:text-red-500 btn-transition"
                  >
                    <Trash2 size={16} />
                  </button>
                </div>
              </div>
              <p className={`mt-4 text-2xl font-bold ${
                acc.balance >= 0 ? styles.iconColor : "text-red-500"
              }`}>
                {formatAmount(acc.balance, acc.currency)}
              </p>
            </div>
          );
        })
        )}
      </div>

      {!loading && accounts.length === 0 && !showForm && (
        <EmptyState
          icon={Wallet}
          title="Нет счетов"
          description="Добавьте первый счёт, чтобы начать отслеживать свои финансы. Это может быть карта, наличные или накопления."
          action={
            <button
              type="button"
              onClick={() => {
                setShowForm(true);
                setEditingId(null);
                setForm({ name: "", account_type: "card", currency: "KZT" });
              }}
              className="inline-flex items-center gap-2 px-4 py-2 rounded-lg bg-emerald-600 text-white hover:bg-emerald-700 btn-transition"
            >
              <Plus size={18} />
              Добавить счёт
            </button>
          }
        />
      )}

      <ConfirmDialog
        open={deleteConfirm !== null}
        title="Удалить счёт?"
        message="Счёт можно удалить только если по нему нет транзакций."
        confirmLabel="Удалить"
        variant="danger"
        onConfirm={handleDeleteConfirm}
        onCancel={() => setDeleteConfirm(null)}
      />
    </div>
  );
}
