import { Wallet, TrendingUp, TrendingDown, ArrowUpRight, ArrowDownRight } from "lucide-react";
import { PieChart, Pie, Cell, ResponsiveContainer } from "recharts";
import type { Summary } from "../../lib/api";

const COLORS = ["#22c55e", "#3b82f6", "#f97316", "#eab308", "#ec4899", "#8b5cf6"];

function currencyLabel(code: string) {
  const symbols: Record<string, string> = { KZT: "₸", USD: "$", EUR: "€", RUB: "₽" };
  return symbols[code] ?? code;
}

function formatAmount(amount: number, baseCurrency: string = "KZT") {
  return new Intl.NumberFormat("ru-KZ", {
    style: "decimal",
    minimumFractionDigits: 0,
    maximumFractionDigits: 0,
  }).format(amount) + " " + currencyLabel(baseCurrency);
}

interface SummaryCardsProps {
  summary: Summary | null;
  loading?: boolean;
  expenseByCategory?: { category_name: string; total: number }[];
  /** When multiple currencies, show warning that total is without conversion */
  currencies?: string[];
}

export function SummaryCards({ summary, loading, expenseByCategory = [], currencies = [] }: SummaryCardsProps) {
  if (loading) {
    return (
      <div className="grid gap-4 lg:grid-cols-3">
        <div className="lg:col-span-2 h-40 rounded-xl bg-zinc-200 dark:bg-zinc-800 animate-pulse" />
        <div className="h-40 rounded-xl bg-zinc-200 dark:bg-zinc-800 animate-pulse" />
        <div className="h-28 rounded-xl bg-zinc-200 dark:bg-zinc-800 animate-pulse" />
        <div className="h-28 rounded-xl bg-zinc-200 dark:bg-zinc-800 animate-pulse" />
      </div>
    );
  }

  if (!summary) return null;

  // Calculate savings rate
  const savingsRate = summary.income_month > 0 
    ? Math.round(((summary.income_month - summary.expense_month) / summary.income_month) * 100)
    : 0;

  const pieData = expenseByCategory.slice(0, 5).map((d, i) => ({
    name: d.category_name,
    value: d.total,
    color: COLORS[i % COLORS.length],
  }));

  return (
    <div className="grid gap-4 lg:grid-cols-3 animate-fade-in">
      {/* Hero balance card - spans 2 columns */}
      <div className="lg:col-span-2 p-6 rounded-xl bg-gradient-to-br from-blue-600 to-blue-700 dark:from-blue-600 dark:to-blue-800 text-white shadow-lg card-hover">
        <div className="flex items-start justify-between">
          <div>
            <div className="flex items-center gap-2 text-blue-100 mb-1">
              <Wallet size={18} />
              <span className="text-sm font-medium">Общий баланс</span>
            </div>
            <p className="text-4xl font-bold mb-2">
              {formatAmount(summary.total_balance, summary.base_currency ?? "KZT")}
            </p>
            {currencies.length > 1 && (
              <p className="text-sm text-blue-200/90 mb-2" title="Итоги в базовой валюте. Курсы в Настройках.">
                в {summary.base_currency ?? "KZT"} ({currencies.join(", ")})
              </p>
            )}
            <div className="flex items-center gap-1 text-sm">
              {savingsRate >= 0 ? (
                <>
                  <ArrowUpRight size={16} className="text-emerald-300" />
                  <span className="text-emerald-300">Сбережения: {savingsRate}%</span>
                </>
              ) : (
                <>
                  <ArrowDownRight size={16} className="text-red-300" />
                  <span className="text-red-300">Перерасход</span>
                </>
              )}
            </div>
          </div>
          {/* Mini pie chart */}
          {pieData.length > 0 && (
            <div className="w-24 h-24">
              <ResponsiveContainer width="100%" height="100%">
                <PieChart>
                  <Pie
                    data={pieData}
                    cx="50%"
                    cy="50%"
                    innerRadius={25}
                    outerRadius={40}
                    paddingAngle={2}
                    dataKey="value"
                  >
                    {pieData.map((entry, index) => (
                      <Cell key={index} fill={entry.color} />
                    ))}
                  </Pie>
                </PieChart>
              </ResponsiveContainer>
            </div>
          )}
        </div>
      </div>

      {/* Expense categories summary */}
      <div className="p-5 rounded-xl bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700 shadow-sm dark:shadow-none card-hover">
        <h4 className="text-sm font-medium text-zinc-500 dark:text-zinc-400 mb-3">Куда уходят деньги</h4>
        {pieData.length > 0 ? (
          <div className="space-y-2">
            {pieData.slice(0, 4).map((item, i) => (
              <div key={i} className="flex items-center justify-between text-sm">
                <div className="flex items-center gap-2">
                  <div className="w-2 h-2 rounded-full" style={{ backgroundColor: item.color }} />
                  <span className="text-zinc-600 dark:text-zinc-300 truncate max-w-[100px]">{item.name}</span>
                </div>
                <span className="font-medium text-zinc-900 dark:text-zinc-100">
                  {formatAmount(item.value, summary.base_currency ?? "KZT")}
                </span>
              </div>
            ))}
          </div>
        ) : (
          <p className="text-sm text-zinc-400">Нет данных за этот месяц</p>
        )}
      </div>

      {/* Income card */}
      <div className="p-5 rounded-xl bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700 shadow-sm dark:shadow-none card-hover">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="p-2.5 rounded-lg bg-emerald-500/10">
              <TrendingUp size={22} className="text-emerald-500" />
            </div>
            <div>
              <p className="text-sm text-zinc-500 dark:text-zinc-400">Доход за месяц</p>
              <p className="text-xl font-semibold text-emerald-500">
                +{formatAmount(summary.income_month, summary.base_currency ?? "KZT")}
              </p>
            </div>
          </div>
        </div>
      </div>

      {/* Expense card */}
      <div className="p-5 rounded-xl bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700 shadow-sm dark:shadow-none card-hover">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="p-2.5 rounded-lg bg-red-500/10">
              <TrendingDown size={22} className="text-red-500" />
            </div>
            <div>
              <p className="text-sm text-zinc-500 dark:text-zinc-400">Расход за месяц</p>
              <p className="text-xl font-semibold text-red-500">
                -{formatAmount(summary.expense_month, summary.base_currency ?? "KZT")}
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
