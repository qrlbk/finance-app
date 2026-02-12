import { Link } from "react-router-dom";
import { ArrowRight, ShoppingBag, Utensils, Car, Home, Briefcase, Gift, CreditCard } from "lucide-react";
import type { TransactionWithDetails } from "../../lib/api";

// Category icons mapping (simplified)
const categoryIcons: Record<string, typeof ShoppingBag> = {
  "Продукты": Utensils,
  "Транспорт": Car,
  "Жильё": Home,
  "Зарплата": Briefcase,
  "Подарки": Gift,
};

function formatAmount(amount: number, type: string) {
  const abs = Math.abs(amount);
  const formatted = new Intl.NumberFormat("ru-KZ", {
    style: "decimal",
    minimumFractionDigits: 0,
    maximumFractionDigits: 0,
  }).format(abs);
  return type === "income" ? `+${formatted} ₸` : `-${formatted} ₸`;
}

function formatDate(dateStr: string) {
  const date = new Date(dateStr + "T12:00:00");
  const today = new Date();
  const yesterday = new Date(today);
  yesterday.setDate(yesterday.getDate() - 1);

  if (date.toDateString() === today.toDateString()) {
    return "Сегодня";
  }
  if (date.toDateString() === yesterday.toDateString()) {
    return "Вчера";
  }
  return date.toLocaleDateString("ru-KZ", {
    day: "2-digit",
    month: "short",
  });
}

interface QuickStatsProps {
  transactions: TransactionWithDetails[];
  loading?: boolean;
}

export function QuickStats({ transactions, loading }: QuickStatsProps) {
  const recent = transactions.slice(0, 5);

  if (loading) {
    return (
      <div className="p-6 rounded-xl bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700 shadow-sm dark:shadow-none">
        <div className="space-y-3">
          {[1, 2, 3, 4, 5].map((i) => (
            <div key={i} className="h-14 rounded-lg bg-zinc-200 dark:bg-zinc-700 animate-pulse" />
          ))}
        </div>
      </div>
    );
  }

  return (
    <div className="p-6 rounded-xl bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700 shadow-sm dark:shadow-none animate-slide-up">
      <div className="flex justify-between items-center mb-5">
        <h4 className="font-medium text-zinc-900 dark:text-zinc-100">Последние транзакции</h4>
        <Link
          to="/transactions"
          className="flex items-center gap-1 text-sm text-emerald-600 dark:text-emerald-400 hover:text-emerald-500 dark:hover:text-emerald-300 btn-transition"
        >
          Все <ArrowRight size={14} />
        </Link>
      </div>
      <div className="space-y-2">
        {recent.map((tx, index) => {
          const Icon = categoryIcons[tx.category_name ?? ""] ?? CreditCard;
          return (
            <div
              key={tx.id}
              className={`flex items-center gap-4 p-3 rounded-lg hover:bg-zinc-50 dark:hover:bg-zinc-800/50 transition-colors animate-stagger animate-stagger-${index + 1}`}
            >
              <div className={`p-2.5 rounded-lg ${
                tx.transaction_type === "income" 
                  ? "bg-emerald-500/10" 
                  : "bg-zinc-100 dark:bg-zinc-800"
              }`}>
                <Icon size={18} className={
                  tx.transaction_type === "income" 
                    ? "text-emerald-500" 
                    : "text-zinc-500 dark:text-zinc-400"
                } />
              </div>
              <div className="flex-1 min-w-0">
                <p className="font-medium text-zinc-900 dark:text-zinc-100 truncate">
                  {tx.category_name ?? tx.note ?? "Без категории"}
                </p>
                <p className="text-sm text-zinc-500 dark:text-zinc-400">
                  {formatDate(tx.date)} · {tx.account_name}
                </p>
              </div>
              <span
                className={`font-semibold whitespace-nowrap ${
                  tx.transaction_type === "income" ? "text-emerald-500" : "text-red-500"
                }`}
              >
                {formatAmount(tx.amount, tx.transaction_type)}
              </span>
            </div>
          );
        })}
      </div>
      {recent.length === 0 && (
        <div className="text-center py-8">
          <CreditCard size={40} className="mx-auto text-zinc-300 dark:text-zinc-600 mb-3" />
          <p className="text-zinc-500 dark:text-zinc-400">Пока нет транзакций</p>
          <Link
            to="/transactions"
            className="inline-block mt-3 text-sm text-emerald-600 dark:text-emerald-400 hover:underline"
          >
            Добавить первую транзакцию
          </Link>
        </div>
      )}
    </div>
  );
}
