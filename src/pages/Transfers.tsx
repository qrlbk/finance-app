import { useEffect, useState } from "react";
import { ArrowRightLeft, Plus } from "lucide-react";
import { api, type Account, type TransferWithDetails } from "../lib/api";
import { useToast } from "../components/ui/Toast";
import { EmptyState } from "../components/ui/EmptyState";

function formatAmount(amount: number) {
  return new Intl.NumberFormat("ru-KZ", {
    style: "decimal",
    minimumFractionDigits: 0,
    maximumFractionDigits: 0,
  }).format(amount);
}

function formatDate(dateStr: string) {
  return new Date(dateStr + "T12:00:00").toLocaleDateString("ru-KZ", {
    day: "2-digit",
    month: "2-digit",
    year: "numeric",
  });
}

export function Transfers() {
  const { showToast } = useToast();
  const [transfers, setTransfers] = useState<TransferWithDetails[]>([]);
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showForm, setShowForm] = useState(false);
  const [form, setForm] = useState({
    from_account_id: 0,
    to_account_id: 0,
    amount: "",
    date: new Date().toISOString().slice(0, 10),
    note: "",
  });

  const load = async () => {
    try {
      setLoading(true);
      setError(null);
      const [t, a] = await Promise.all([
        api.getTransfers({ limit: 50 }),
        api.getAccounts(),
      ]);
      setTransfers(t);
      setAccounts(a);
      if (a.length > 0 && form.from_account_id === 0) {
        setForm((prev) => ({
          ...prev,
          from_account_id: a[0].id,
          to_account_id: a[1]?.id ?? a[0].id,
        }));
      }
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    load();
  }, []);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    const amount = parseFloat(form.amount);
    if (isNaN(amount) || amount <= 0) return;
    if (form.from_account_id === form.to_account_id) {
      showToast("Выберите разные счета", "error");
      return;
    }
    try {
      await api.createTransfer({
        from_account_id: form.from_account_id,
        to_account_id: form.to_account_id,
        amount,
        date: form.date,
        note: form.note.trim() || null,
      });
      showToast("Перевод выполнен", "success");
      setForm((prev) => ({
        ...prev,
        amount: "",
        date: new Date().toISOString().slice(0, 10),
        note: "",
      }));
      setShowForm(false);
      load();
    } catch (err) {
      setError(String(err));
      showToast("Ошибка при переводе", "error");
    }
  };

  return (
    <div className="space-y-6">
      {error && (
        <div className="p-4 rounded-lg bg-red-500/10 text-red-500 border border-red-500/20">
          {error}
        </div>
      )}

      <div className="flex justify-between items-center">
        <h3 className="text-lg font-medium">Переводы</h3>
        <button
          type="button"
          onClick={() => {
            setShowForm(true);
            if (accounts.length > 0) {
              setForm((prev) => ({
                ...prev,
                from_account_id: accounts[0].id,
                to_account_id: accounts[1]?.id ?? accounts[0].id,
                amount: "",
                date: new Date().toISOString().slice(0, 10),
                note: "",
              }));
            }
          }}
          className="flex items-center gap-2 px-4 py-2 rounded-lg bg-emerald-600 text-white hover:bg-emerald-700 btn-transition"
        >
          <Plus size={18} />
          Новый перевод
        </button>
      </div>

      {showForm && (
        <form
          onSubmit={handleSubmit}
          className="p-6 rounded-xl bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700 shadow-sm dark:shadow-none space-y-4 animate-slide-down"
        >
          <h4 className="font-medium">Перевод между счетами</h4>
          <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
            <div>
              <label className="block text-sm text-zinc-400 mb-1">Со счёта</label>
              <select
                value={form.from_account_id}
                onChange={(e) => setForm((f) => ({ ...f, from_account_id: +e.target.value }))}
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
              <label className="block text-sm text-zinc-400 mb-1">На счёт</label>
              <select
                value={form.to_account_id}
                onChange={(e) => setForm((f) => ({ ...f, to_account_id: +e.target.value }))}
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
              <label className="block text-sm text-zinc-400 mb-1">Сумма</label>
              <input
                type="number"
                step="0.01"
                min="0"
                value={form.amount}
                onChange={(e) => setForm((f) => ({ ...f, amount: e.target.value }))}
                className="w-full px-4 py-2 rounded-lg bg-zinc-100 dark:bg-zinc-700 border border-zinc-300 dark:border-zinc-600 text-zinc-900 dark:text-white form-transition focus:ring-2 focus:ring-emerald-500"
                required
              />
            </div>
            <div>
              <label className="block text-sm text-zinc-400 mb-1">Дата</label>
              <input
                type="date"
                value={form.date}
                onChange={(e) => setForm((f) => ({ ...f, date: e.target.value }))}
                className="w-full px-4 py-2 rounded-lg bg-zinc-100 dark:bg-zinc-700 border border-zinc-300 dark:border-zinc-600 text-zinc-900 dark:text-white form-transition focus:ring-2 focus:ring-emerald-500"
              />
            </div>
          </div>
          <div>
            <label className="block text-sm text-zinc-400 mb-1">Заметка</label>
            <input
              type="text"
              value={form.note}
              onChange={(e) => setForm((f) => ({ ...f, note: e.target.value }))}
              className="w-full px-4 py-2 rounded-lg bg-zinc-100 dark:bg-zinc-700 border border-zinc-300 dark:border-zinc-600 text-zinc-900 dark:text-white placeholder-zinc-500 form-transition focus:ring-2 focus:ring-emerald-500"
              placeholder="Описание перевода"
            />
          </div>
          <div className="flex gap-2">
            <button
              type="submit"
              className="px-4 py-2 rounded-lg bg-emerald-600 text-white hover:bg-emerald-700 btn-transition"
            >
              Выполнить перевод
            </button>
            <button
              type="button"
              onClick={() => setShowForm(false)}
              className="px-4 py-2 rounded-lg bg-zinc-600 text-white hover:bg-zinc-500 btn-transition"
            >
              Отмена
            </button>
          </div>
        </form>
      )}

      <div className="rounded-xl bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700 overflow-hidden">
        <div className="p-4 border-b border-zinc-200 dark:border-zinc-700">
          <h4 className="font-medium text-zinc-900 dark:text-zinc-100">История переводов</h4>
        </div>
        {loading ? (
          <div className="p-6 space-y-3">
            {[1, 2, 3].map((i) => (
              <div key={i} className="h-14 rounded-lg bg-zinc-200 dark:bg-zinc-700 animate-pulse" />
            ))}
          </div>
        ) : transfers.length === 0 ? (
          <div className="p-6">
            <EmptyState
              icon={ArrowRightLeft}
              title="Нет переводов"
              description="Переводы между вашими счетами появятся здесь после первого перевода."
              action={
                <button
                  type="button"
                  onClick={() => setShowForm(true)}
                  className="inline-flex items-center gap-2 px-4 py-2 rounded-lg bg-emerald-600 text-white hover:bg-emerald-700 btn-transition"
                >
                  <Plus size={18} />
                  Новый перевод
                </button>
              }
            />
          </div>
        ) : (
          <ul className="divide-y divide-zinc-200 dark:divide-zinc-700">
            {transfers.map((t) => (
              <li
                key={t.id}
                className="p-4 flex flex-wrap items-center gap-3 hover:bg-zinc-50 dark:hover:bg-zinc-800/50 transition-colors"
              >
                <span className="font-medium text-zinc-900 dark:text-zinc-100">{t.from_account_name}</span>
                <ArrowRightLeft size={16} className="text-zinc-400 shrink-0" />
                <span className="font-medium text-zinc-900 dark:text-zinc-100">{t.to_account_name}</span>
                <span className="font-semibold text-emerald-500">{formatAmount(t.amount)} ₸</span>
                <span className="text-sm text-zinc-500 dark:text-zinc-400">{formatDate(t.date)}</span>
                {t.note && (
                  <span className="text-sm text-zinc-500 dark:text-zinc-400 truncate max-w-[200px]" title={t.note}>
                    {t.note}
                  </span>
                )}
              </li>
            ))}
          </ul>
        )}
      </div>
    </div>
  );
}
