import { useEffect, useState } from "react";
import { PieChart, Pie, Cell, ResponsiveContainer, BarChart, Bar, XAxis, YAxis, Tooltip, Legend } from "recharts";
import { TrendingUp, TrendingDown, PieChart as PieIcon, BarChart3, Wallet, Table2, Sparkles } from "lucide-react";
import { api, type ForecastDetails } from "../lib/api";

const COLORS = ["#22c55e", "#3b82f6", "#f97316", "#eab308", "#ec4899", "#8b5cf6", "#06b6d4", "#64748b"];

function formatAmount(amount: number) {
  return new Intl.NumberFormat("ru-KZ", {
    style: "decimal",
    minimumFractionDigits: 0,
    maximumFractionDigits: 0,
  }).format(amount) + " ₸";
}

// Custom tooltip component
function CustomTooltip({ active, payload, label }: any) {
  if (active && payload && payload.length) {
    return (
      <div className="bg-white dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 rounded-lg shadow-lg p-3">
        {label && <p className="text-sm text-zinc-500 dark:text-zinc-400 mb-1">{label}</p>}
        {payload.map((entry: any, index: number) => (
          <div key={index} className="flex items-center gap-2">
            <div 
              className="w-3 h-3 rounded-full" 
              style={{ backgroundColor: entry.color }}
            />
            <span className="text-sm font-medium text-zinc-900 dark:text-zinc-100">
              {entry.name}: {formatAmount(entry.value)}
            </span>
          </div>
        ))}
      </div>
    );
  }
  return null;
}

// Custom pie chart label
function renderCustomLabel({ cx, cy, midAngle, innerRadius, outerRadius, percent, name }: any) {
  if (percent < 0.05) return null; // Don't show labels for small slices
  
  const RADIAN = Math.PI / 180;
  const radius = innerRadius + (outerRadius - innerRadius) * 1.4;
  const x = cx + radius * Math.cos(-midAngle * RADIAN);
  const y = cy + radius * Math.sin(-midAngle * RADIAN);

  return (
    <text
      x={x}
      y={y}
      fill="currentColor"
      textAnchor={x > cx ? 'start' : 'end'}
      dominantBaseline="central"
      className="text-xs fill-zinc-600 dark:fill-zinc-400"
    >
      {`${name} ${(percent * 100).toFixed(0)}%`}
    </text>
  );
}

export function Reports() {
  const [categoryData, setCategoryData] = useState<{ category_name: string; total: number }[]>([]);
  const [monthlyData, setMonthlyData] = useState<{ month: string; income: number; expense: number }[]>([]);
  const [summary, setSummary] = useState<{ total_balance: number; income_month: number; expense_month: number } | null>(null);
  const [forecast, setForecast] = useState<ForecastDetails | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [year, setYear] = useState(new Date().getFullYear());
  const [month, setMonth] = useState(new Date().getMonth() + 1);
  const [monthsCount, setMonthsCount] = useState(6);
  const [includeSubcategories, setIncludeSubcategories] = useState(false);

  useEffect(() => {
    const load = async () => {
      try {
        setLoading(true);
        setError(null);
        const [cat, mon, sum, fc] = await Promise.all([
          api.getExpenseByCategory({ year, month, include_children: includeSubcategories }),
          api.getMonthlyTotals({ months: monthsCount }),
          api.getSummary().catch(() => null),
          api.getForecastDetails().catch(() => null),
        ]);
        setCategoryData(cat);
        setMonthlyData(mon);
        setSummary(sum ?? null);
        setForecast(fc ?? null);
      } catch (e) {
        setError(String(e));
      } finally {
        setLoading(false);
      }
    };
    load();
  }, [year, month, monthsCount, includeSubcategories]);

  const pieData = categoryData.map((d, i) => ({
    name: d.category_name,
    value: d.total,
    color: COLORS[i % COLORS.length],
  }));

  const totalExpense = categoryData.reduce((sum, d) => sum + d.total, 0);

  // Calculate monthly stats
  const currentMonthData = monthlyData[monthlyData.length - 1];
  const prevMonthData = monthlyData[monthlyData.length - 2];
  
  const incomeChange = currentMonthData && prevMonthData && prevMonthData.income > 0
    ? ((currentMonthData.income - prevMonthData.income) / prevMonthData.income * 100).toFixed(0)
    : null;
  
  const expenseChange = currentMonthData && prevMonthData && prevMonthData.expense > 0
    ? ((currentMonthData.expense - prevMonthData.expense) / prevMonthData.expense * 100).toFixed(0)
    : null;

  return (
    <div className="space-y-6">
      {error && (
        <div className="p-4 rounded-lg bg-red-500/10 text-red-500 border border-red-500/20 animate-shake">
          {error}
        </div>
      )}

      {/* Summary card */}
      {summary && (
        <div className="p-5 rounded-xl bg-gradient-to-br from-emerald-500/10 to-emerald-600/5 border border-emerald-500/20 dark:border-emerald-500/30 animate-fade-in">
          <div className="flex items-center gap-3">
            <div className="p-2.5 rounded-lg bg-emerald-500/20">
              <Wallet size={22} className="text-emerald-500" />
            </div>
            <div>
              <p className="text-sm text-zinc-500 dark:text-zinc-400">Общий баланс</p>
              <p className="text-2xl font-bold text-emerald-600 dark:text-emerald-400">{formatAmount(summary.total_balance)}</p>
            </div>
            <div className="ml-auto text-right text-sm">
              <p className="text-zinc-500 dark:text-zinc-400">Накопления за месяц</p>
              <p className={`font-semibold ${summary.income_month - summary.expense_month >= 0 ? "text-emerald-500" : "text-red-500"}`}>
                {formatAmount(summary.income_month - summary.expense_month)}
              </p>
            </div>
          </div>
        </div>
      )}

      {/* Period selector */}
      <div className="flex flex-wrap gap-4 items-end">
        <div>
          <label className="block text-sm text-zinc-500 dark:text-zinc-400 mb-1">Год (для круговой диаграммы)</label>
          <select
            value={year}
            onChange={(e) => setYear(+e.target.value)}
            className="px-4 py-2 rounded-lg bg-white dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 text-zinc-900 dark:text-white form-transition focus:ring-2 focus:ring-emerald-500"
          >
            {[2024, 2025, 2026, 2027].map((y) => (
              <option key={y} value={y}>
                {y}
              </option>
            ))}
          </select>
        </div>
        <div>
          <label className="block text-sm text-zinc-500 dark:text-zinc-400 mb-1">Месяц</label>
          <select
            value={month}
            onChange={(e) => setMonth(+e.target.value)}
            className="px-4 py-2 rounded-lg bg-white dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 text-zinc-900 dark:text-white form-transition focus:ring-2 focus:ring-emerald-500"
          >
            {[
              "Январь", "Февраль", "Март", "Апрель", "Май", "Июнь",
              "Июль", "Август", "Сентябрь", "Октябрь", "Ноябрь", "Декабрь",
            ].map((name, i) => (
              <option key={i} value={i + 1}>
                {name}
              </option>
            ))}
          </select>
        </div>
        <div>
          <label className="block text-sm text-zinc-500 dark:text-zinc-400 mb-1">Период графика (мес.)</label>
          <select
            value={monthsCount}
            onChange={(e) => setMonthsCount(+e.target.value)}
            className="px-4 py-2 rounded-lg bg-white dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 text-zinc-900 dark:text-white form-transition focus:ring-2 focus:ring-emerald-500"
          >
            <option value={6}>6 месяцев</option>
            <option value={12}>12 месяцев</option>
          </select>
        </div>
        <div className="flex items-end">
          <label className="flex items-center gap-2 cursor-pointer text-sm text-zinc-600 dark:text-zinc-300">
            <input
              type="checkbox"
              checked={includeSubcategories}
              onChange={(e) => setIncludeSubcategories(e.target.checked)}
              className="rounded border-zinc-300 dark:border-zinc-600 text-emerald-500 focus:ring-emerald-500"
            />
            Включая подкатегории
          </label>
        </div>
      </div>

      {/* Stats cards */}
      {currentMonthData && (
        <div className="grid gap-4 sm:grid-cols-2 animate-fade-in">
          <div className="p-5 rounded-xl bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700 card-hover">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <div className="p-2.5 rounded-lg bg-emerald-500/10">
                  <TrendingUp size={20} className="text-emerald-500" />
                </div>
                <div>
                  <p className="text-sm text-zinc-500 dark:text-zinc-400">Доход за месяц</p>
                  <p className="text-xl font-bold text-emerald-500">{formatAmount(currentMonthData.income)}</p>
                </div>
              </div>
              {incomeChange && (
                <span className={`text-sm font-medium ${Number(incomeChange) >= 0 ? "text-emerald-500" : "text-red-500"}`}>
                  {Number(incomeChange) >= 0 ? "+" : ""}{incomeChange}%
                </span>
              )}
            </div>
          </div>
          <div className="p-5 rounded-xl bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700 card-hover">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <div className="p-2.5 rounded-lg bg-red-500/10">
                  <TrendingDown size={20} className="text-red-500" />
                </div>
                <div>
                  <p className="text-sm text-zinc-500 dark:text-zinc-400">Расход за месяц</p>
                  <p className="text-xl font-bold text-red-500">{formatAmount(currentMonthData.expense)}</p>
                </div>
              </div>
              {expenseChange && (
                <span className={`text-sm font-medium ${Number(expenseChange) <= 0 ? "text-emerald-500" : "text-red-500"}`}>
                  {Number(expenseChange) >= 0 ? "+" : ""}{expenseChange}%
                </span>
              )}
            </div>
          </div>
        </div>
      )}

      <div className="grid gap-6 lg:grid-cols-2">
        {/* Pie Chart */}
        <div className="p-6 rounded-xl bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700 shadow-sm dark:shadow-none animate-slide-up">
          <div className="flex items-center gap-2 mb-4">
            <PieIcon size={18} className="text-zinc-400" />
            <h4 className="font-medium text-zinc-900 dark:text-zinc-100">Расходы по категориям</h4>
          </div>
          {loading ? (
            <div className="h-64 flex items-center justify-center">
              <div className="w-32 h-32 rounded-full border-4 border-zinc-200 dark:border-zinc-700 border-t-emerald-500 animate-spin" />
            </div>
          ) : pieData.length > 0 ? (
            <>
              <ResponsiveContainer width="100%" height={280}>
                <PieChart>
                  <Pie
                    data={pieData}
                    cx="50%"
                    cy="50%"
                    innerRadius={60}
                    outerRadius={100}
                    paddingAngle={2}
                    dataKey="value"
                    label={renderCustomLabel}
                    labelLine={false}
                    animationBegin={0}
                    animationDuration={800}
                  >
                    {pieData.map((entry, index) => (
                      <Cell key={index} fill={entry.color} />
                    ))}
                  </Pie>
                  <Tooltip content={<CustomTooltip />} />
                </PieChart>
              </ResponsiveContainer>
              
              {/* Legend */}
              <div className="mt-4 grid grid-cols-2 gap-2">
                {pieData.map((entry, i) => (
                  <div key={i} className="flex items-center gap-2 text-sm">
                    <div 
                      className="w-3 h-3 rounded-full shrink-0" 
                      style={{ backgroundColor: entry.color }}
                    />
                    <span className="text-zinc-600 dark:text-zinc-400 truncate">{entry.name}</span>
                    <span className="text-zinc-900 dark:text-zinc-200 font-medium ml-auto">
                      {((entry.value / totalExpense) * 100).toFixed(0)}%
                    </span>
                  </div>
                ))}
              </div>
            </>
          ) : (
            <div className="h-64 flex flex-col items-center justify-center text-zinc-500">
              <PieIcon size={40} className="text-zinc-300 dark:text-zinc-700 mb-3" />
              <p>Нет данных за выбранный период</p>
            </div>
          )}
        </div>

        {/* Bar Chart */}
        <div className="p-6 rounded-xl bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700 shadow-sm dark:shadow-none animate-slide-up" style={{ animationDelay: "0.1s" }}>
          <div className="flex items-center gap-2 mb-4">
            <BarChart3 size={18} className="text-zinc-400" />
            <h4 className="font-medium text-zinc-900 dark:text-zinc-100">Доходы и расходы по месяцам</h4>
          </div>
          {loading ? (
            <div className="h-64 flex items-center justify-center">
              <div className="w-32 h-32 rounded-full border-4 border-zinc-200 dark:border-zinc-700 border-t-emerald-500 animate-spin" />
            </div>
          ) : monthlyData.length > 0 ? (
            <ResponsiveContainer width="100%" height={320}>
              <BarChart data={monthlyData} margin={{ top: 20, right: 20, left: 0, bottom: 5 }}>
                <XAxis 
                  dataKey="month" 
                  stroke="#71717a" 
                  fontSize={12}
                  tickLine={false}
                  axisLine={false}
                />
                <YAxis 
                  stroke="#71717a" 
                  fontSize={12} 
                  tickFormatter={(v) => `${v / 1000}к`}
                  tickLine={false}
                  axisLine={false}
                />
                <Tooltip content={<CustomTooltip />} />
                <Legend 
                  verticalAlign="top"
                  height={36}
                  iconType="circle"
                  formatter={(value) => (
                    <span className="text-sm text-zinc-600 dark:text-zinc-400">{value}</span>
                  )}
                />
                <Bar 
                  dataKey="income" 
                  fill="#22c55e" 
                  name="Доход" 
                  radius={[4, 4, 0, 0]}
                  animationBegin={0}
                  animationDuration={800}
                />
                <Bar 
                  dataKey="expense" 
                  fill="#ef4444" 
                  name="Расход" 
                  radius={[4, 4, 0, 0]}
                  animationBegin={200}
                  animationDuration={800}
                />
              </BarChart>
            </ResponsiveContainer>
          ) : (
            <div className="h-64 flex flex-col items-center justify-center text-zinc-500">
              <BarChart3 size={40} className="text-zinc-300 dark:text-zinc-700 mb-3" />
              <p>Нет данных</p>
            </div>
          )}
        </div>
      </div>

      {/* Table: expenses by category */}
      <div className="p-6 rounded-xl bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700 shadow-sm dark:shadow-none animate-slide-up" style={{ animationDelay: "0.15s" }}>
        <div className="flex items-center gap-2 mb-4">
          <Table2 size={18} className="text-zinc-400" />
          <h4 className="font-medium text-zinc-900 dark:text-zinc-100">Расходы по категориям — таблица</h4>
        </div>
        {loading ? (
          <div className="h-32 flex items-center justify-center">
            <div className="w-8 h-8 rounded-full border-2 border-zinc-200 dark:border-zinc-700 border-t-emerald-500 animate-spin" />
          </div>
        ) : categoryData.length > 0 ? (
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-zinc-200 dark:border-zinc-700 text-left text-zinc-500 dark:text-zinc-400">
                  <th className="pb-3 pr-4 font-medium">Категория</th>
                  <th className="pb-3 pr-4 font-medium text-right">Сумма</th>
                  <th className="pb-3 font-medium text-right">% от общих расходов</th>
                </tr>
              </thead>
              <tbody>
                {categoryData.map((row, i) => (
                  <tr key={i} className="border-b border-zinc-100 dark:border-zinc-800 last:border-0">
                    <td className="py-2.5 pr-4 text-zinc-900 dark:text-zinc-100">{row.category_name}</td>
                    <td className="py-2.5 pr-4 text-right font-medium text-red-500">{formatAmount(row.total)}</td>
                    <td className="py-2.5 text-right text-zinc-600 dark:text-zinc-400">
                      {totalExpense > 0 ? ((row.total / totalExpense) * 100).toFixed(1) : 0}%
                    </td>
                  </tr>
                ))}
              </tbody>
              <tfoot>
                <tr className="border-t-2 border-zinc-200 dark:border-zinc-700 font-semibold text-zinc-900 dark:text-zinc-100">
                  <td className="pt-3 pr-4">Итого</td>
                  <td className="pt-3 pr-4 text-right text-red-500">{formatAmount(totalExpense)}</td>
                  <td className="pt-3 text-right">100%</td>
                </tr>
              </tfoot>
            </table>
          </div>
        ) : (
          <p className="text-zinc-500 dark:text-zinc-400 text-center py-6">Нет данных за выбранный период</p>
        )}
      </div>

      {/* Forecast */}
      {forecast && (
        <div className="p-6 rounded-xl bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700 shadow-sm dark:shadow-none animate-slide-up" style={{ animationDelay: "0.2s" }}>
          <div className="flex items-center gap-2 mb-4">
            <Sparkles size={18} className="text-amber-500" />
            <h4 className="font-medium text-zinc-900 dark:text-zinc-100">Прогноз расходов на следующий месяц</h4>
          </div>
          <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
            <div className="p-4 rounded-lg bg-amber-500/10 border border-amber-500/20">
              <p className="text-sm text-zinc-500 dark:text-zinc-400 mb-1">Ожидаемые расходы</p>
              <p className="text-xl font-bold text-amber-600 dark:text-amber-400">
                {formatAmount(forecast.overall.predicted_expense)}
              </p>
              <p className="text-xs text-zinc-500 dark:text-zinc-400 mt-1">
                диапазон: {formatAmount(forecast.overall.confidence_low)} — {formatAmount(forecast.overall.confidence_high)}
              </p>
              <p className="text-xs mt-1">
                Тренд:{" "}
                <span className={forecast.overall.trend === "up" ? "text-red-500" : forecast.overall.trend === "down" ? "text-emerald-500" : "text-zinc-500"}>
                  {forecast.overall.trend === "up" ? "↑ рост" : forecast.overall.trend === "down" ? "↓ снижение" : "→ стабильно"} ({forecast.overall.trend_percent > 0 ? "+" : ""}{forecast.overall.trend_percent.toFixed(0)}%)
                </span>
              </p>
            </div>
            {forecast.by_category.length > 0 && (
              <div className="sm:col-span-2">
                <p className="text-sm text-zinc-500 dark:text-zinc-400 mb-2">По категориям</p>
                <div className="overflow-x-auto">
                  <table className="w-full text-sm">
                    <thead>
                      <tr className="border-b border-zinc-200 dark:border-zinc-700 text-left text-zinc-500 dark:text-zinc-400">
                        <th className="pb-2 pr-4 font-medium">Категория</th>
                        <th className="pb-2 font-medium text-right">Прогноз</th>
                      </tr>
                    </thead>
                    <tbody>
                      {forecast.by_category.map((c) => (
                        <tr key={c.category_id} className="border-b border-zinc-100 dark:border-zinc-800">
                          <td className="py-1.5 pr-4 text-zinc-900 dark:text-zinc-100">{c.category_name}</td>
                          <td className="py-1.5 text-right font-medium text-amber-600 dark:text-amber-400">{formatAmount(c.predicted_expense)}</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
