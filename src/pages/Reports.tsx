import { useEffect, useState } from "react";
import { PieChart, Pie, Cell, ResponsiveContainer, BarChart, Bar, XAxis, YAxis, Tooltip, Legend } from "recharts";
import { TrendingUp, TrendingDown, PieChart as PieIcon, BarChart3 } from "lucide-react";
import { api } from "../lib/api";

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
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [year, setYear] = useState(new Date().getFullYear());
  const [month, setMonth] = useState(new Date().getMonth() + 1);

  useEffect(() => {
    const load = async () => {
      try {
        setLoading(true);
        setError(null);
        const [cat, mon] = await Promise.all([
          api.getExpenseByCategory({ year, month }),
          api.getMonthlyTotals({ months: 6 }),
        ]);
        setCategoryData(cat);
        setMonthlyData(mon);
      } catch (e) {
        setError(String(e));
      } finally {
        setLoading(false);
      }
    };
    load();
  }, [year, month]);

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

      {/* Period selector */}
      <div className="flex flex-wrap gap-4 items-end">
        <div>
          <label className="block text-sm text-zinc-500 dark:text-zinc-400 mb-1">Год</label>
          <select
            value={year}
            onChange={(e) => setYear(+e.target.value)}
            className="px-4 py-2 rounded-lg bg-white dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 text-zinc-900 dark:text-white form-transition focus:ring-2 focus:ring-emerald-500"
          >
            {[2024, 2025, 2026].map((y) => (
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
    </div>
  );
}
