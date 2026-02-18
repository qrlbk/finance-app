import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { ArrowRightLeft, Plus } from "lucide-react";
import { api, type Account, type TransferWithDetails } from "../lib/api";
import { useToast } from "../components/ui/Toast";
import { EmptyState } from "../components/ui/EmptyState";
import { formatCurrency, formatDate } from "../lib/format";

export function Transfers() {
  const { t } = useTranslation();
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
      const [transfersData, a] = await Promise.all([
        api.getTransfers({ limit: 50 }),
        api.getAccounts(),
      ]);
      setTransfers(transfersData);
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
      showToast(t("transfers.selectDifferent"), "error");
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
      showToast(t("transfers.done"), "success");
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
      showToast(t("transfers.error"), "error");
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
        <h3 className="text-lg font-medium">{t("transfers.title")}</h3>
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
          {t("transfers.newTransfer")}
        </button>
      </div>

      {showForm && (
        <form
          onSubmit={handleSubmit}
          className="p-6 rounded-xl bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700 shadow-sm dark:shadow-none space-y-4 animate-slide-down"
        >
          <h4 className="font-medium">{t("transfers.formTitle")}</h4>
          <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
            <div>
              <label className="block text-sm text-zinc-400 mb-1">{t("transfers.from")}</label>
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
              <label className="block text-sm text-zinc-400 mb-1">{t("transfers.to")}</label>
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
              <label className="block text-sm text-zinc-400 mb-1">{t("transfers.amount")}</label>
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
              <label className="block text-sm text-zinc-400 mb-1">{t("transfers.date")}</label>
              <input
                type="date"
                value={form.date}
                onChange={(e) => setForm((f) => ({ ...f, date: e.target.value }))}
                className="w-full px-4 py-2 rounded-lg bg-zinc-100 dark:bg-zinc-700 border border-zinc-300 dark:border-zinc-600 text-zinc-900 dark:text-white form-transition focus:ring-2 focus:ring-emerald-500"
              />
            </div>
          </div>
          <div>
            <label className="block text-sm text-zinc-400 mb-1">{t("transfers.note")}</label>
            <input
              type="text"
              value={form.note}
              onChange={(e) => setForm((f) => ({ ...f, note: e.target.value }))}
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
              onClick={() => setShowForm(false)}
              className="px-4 py-2 rounded-lg bg-zinc-600 text-white hover:bg-zinc-500 btn-transition"
            >
              {t("common.cancel")}
            </button>
          </div>
        </form>
      )}

      <div className="rounded-xl bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700 overflow-hidden">
        <div className="p-4 border-b border-zinc-200 dark:border-zinc-700">
          <h4 className="font-medium text-zinc-900 dark:text-zinc-100">{t("transfers.history")}</h4>
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
              title={t("transfers.emptyTitle")}
              description={t("transfers.emptyDesc")}
              action={
                <button
                  type="button"
                  onClick={() => setShowForm(true)}
                  className="inline-flex items-center gap-2 px-4 py-2 rounded-lg bg-emerald-600 text-white hover:bg-emerald-700 btn-transition"
                >
                  <Plus size={18} />
                  {t("transfers.newTransfer")}
                </button>
              }
            />
          </div>
        ) : (
          <ul className="divide-y divide-zinc-200 dark:divide-zinc-700">
            {transfers.map((tr) => (
              <li
                key={tr.id}
                className="p-4 flex flex-wrap items-center gap-3 hover:bg-zinc-50 dark:hover:bg-zinc-800/50 transition-colors"
              >
                <span className="font-medium text-zinc-900 dark:text-zinc-100">{tr.from_account_name}</span>
                <ArrowRightLeft size={16} className="text-zinc-400 shrink-0" />
                <span className="font-medium text-zinc-900 dark:text-zinc-100">{tr.to_account_name}</span>
                <span className="font-semibold text-emerald-500">{formatCurrency(tr.amount)} ₸</span>
                <span className="text-sm text-zinc-500 dark:text-zinc-400">{formatDate(tr.date)}</span>
                {tr.note && (
                  <span className="text-sm text-zinc-500 dark:text-zinc-400 truncate max-w-[200px]" title={tr.note}>
                    {tr.note}
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
