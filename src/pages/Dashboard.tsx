import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { TrendingUp, TrendingDown, BarChart3, AlertTriangle, ArrowUpRight, ArrowDownRight, Minus, PiggyBank, Clock, ArrowRightLeft } from "lucide-react";
import { SummaryCards } from "../components/dashboard/SummaryCards";
import { QuickStats } from "../components/dashboard/QuickStats";
import { api, type Summary, type TransactionWithDetails, type Insights, type Budget, type BudgetAlert } from "../lib/api";
import { formatCurrency, formatMonthLong } from "../lib/format";

export function Dashboard() {
  const { t } = useTranslation();
  const [summary, setSummary] = useState<Summary | null>(null);
  const [transactions, setTransactions] = useState<TransactionWithDetails[]>([]);
  const [expenseByCategory, setExpenseByCategory] = useState<{ category_name: string; total: number }[]>([]);
  const [insights, setInsights] = useState<Insights | null>(null);
  const [budgets, setBudgets] = useState<Budget[]>([]);
  const [budgetAlerts, setBudgetAlerts] = useState<BudgetAlert[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const load = async () => {
    try {
      setLoading(true);
      setError(null);
      const now = new Date();
      const [s, txs, expenses, insightsData, budgetData, alertsData] = await Promise.all([
        api.getSummary(),
        api.getTransactions({ limit: 10 }),
        api.getExpenseByCategory({ year: now.getFullYear(), month: now.getMonth() + 1 }),
        api.getInsights().catch(() => null),
        api.getBudgets().catch(() => []),
        api.getBudgetAlerts().catch(() => []),
      ]);
      setSummary(s);
      setTransactions(txs);
      setExpenseByCategory(expenses);
      setInsights(insightsData);
      setBudgets(budgetData);
      setBudgetAlerts(alertsData);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const getNextMonthName = () => {
    const now = new Date();
    const nextMonth = new Date(now.getFullYear(), now.getMonth() + 1, 1);
    return formatMonthLong(nextMonth);
  };

  useEffect(() => {
    load();
  }, []);

  return (
    <div className="space-y-6">
      {error && (
        <div className="p-4 rounded-lg bg-red-500/10 text-red-500 border border-red-500/20 flex items-center justify-between gap-4 animate-shake">
          <span>{error}</span>
          <button
            type="button"
            onClick={load}
            className="px-4 py-2 rounded-lg bg-red-500/20 hover:bg-red-500/30 text-red-400 btn-transition shrink-0"
          >
            {t("dashboard.retry")}
          </button>
        </div>
      )}

      {summary?.currencies && summary.currencies.length > 1 && (
        <div className="p-4 rounded-xl border border-amber-500/30 bg-amber-500/5 dark:bg-amber-500/10 animate-fade-in" role="alert">
          <p className="text-sm text-zinc-700 dark:text-zinc-300">
            {t("dashboard.summaryNoConversion", { currencies: summary.currencies.join(", ") })}
          </p>
        </div>
      )}

      {budgetAlerts.length > 0 && (
        <div className="p-4 rounded-xl border border-amber-500/30 bg-amber-500/5 dark:bg-amber-500/10 animate-fade-in">
          <div className="flex items-center gap-2 mb-2">
            <AlertTriangle size={18} className="text-amber-500" />
            <h3 className="font-medium text-zinc-900 dark:text-zinc-100">{t("dashboard.budgetAlertsTitle")}</h3>
          </div>
          <ul className="space-y-1 text-sm text-zinc-600 dark:text-zinc-300">
            {budgetAlerts.map((alert, i) => (
              <li key={i}>
                <span className="font-medium">{alert.category_name}</span>
                <span className={alert.severity === "exceeded" ? " text-red-500" : " text-amber-500"}>
                  {" "}
                  {alert.severity === "exceeded"
                    ? t("dashboard.budgetExceeded", { percent: Math.round(alert.percent_used) })
                    : t("dashboard.budgetNearLimit", { percent: Math.round(alert.percent_used) })}
                </span>
              </li>
            ))}
          </ul>
        </div>
      )}
      
      <SummaryCards 
        summary={summary} 
        loading={loading} 
        expenseByCategory={expenseByCategory}
        currencies={summary?.currencies}
      />
      
      <div className="flex flex-wrap gap-3 animate-fade-in">
        <Link
          to="/transactions?type=income"
          className="flex items-center gap-2 px-4 py-2.5 rounded-lg bg-emerald-600 text-white hover:bg-emerald-700 btn-transition shadow-sm hover:shadow-md"
        >
          <TrendingUp size={18} />
          {t("dashboard.addIncome")}
        </Link>
        <Link
          to="/transactions?type=expense"
          className="flex items-center gap-2 px-4 py-2.5 rounded-lg bg-zinc-600 text-white hover:bg-zinc-500 btn-transition shadow-sm hover:shadow-md"
        >
          <TrendingDown size={18} />
          {t("dashboard.addExpense")}
        </Link>
        <Link
          to="/transfers"
          className="flex items-center gap-2 px-4 py-2.5 rounded-lg bg-blue-600 text-white hover:bg-blue-700 btn-transition shadow-sm hover:shadow-md"
        >
          <ArrowRightLeft size={18} />
          {t("dashboard.transfer")}
        </Link>
      </div>
      
      <QuickStats transactions={transactions} loading={loading} />

      {budgets.length > 0 && (
        <div className="p-5 rounded-xl border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900/50 animate-fade-in">
          <div className="flex items-center justify-between mb-4">
            <div className="flex items-center gap-2">
              <div className="p-2 rounded-lg bg-emerald-500/10">
                <PiggyBank size={18} className="text-emerald-500" />
              </div>
              <h3 className="font-medium text-zinc-900 dark:text-zinc-100">{t("dashboard.budgets")}</h3>
            </div>
            <Link
              to="/settings"
              className="text-sm text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300 btn-transition"
            >
              {t("dashboard.manageBudgets")}
            </Link>
          </div>
          
          <div className="space-y-3">
            {budgets.slice(0, 4).map((budget) => (
              <div key={budget.id}>
                <div className="flex justify-between text-sm mb-1">
                  <span className="text-zinc-600 dark:text-zinc-300">{budget.category_name}</span>
                  <span className={`font-medium ${
                    budget.percent_used >= 100
                      ? "text-red-500"
                      : budget.percent_used >= 80
                      ? "text-amber-500"
                      : "text-emerald-500"
                  }`}>
                    {formatCurrency(budget.spent)} / {formatCurrency(budget.amount)} ₸
                  </span>
                </div>
                <div className="relative h-2 rounded-full bg-zinc-200 dark:bg-zinc-700 overflow-hidden">
                  <div
                    className={`absolute left-0 top-0 h-full rounded-full transition-all ${
                      budget.percent_used >= 100
                        ? "bg-red-500"
                        : budget.percent_used >= 80
                        ? "bg-amber-500"
                        : "bg-emerald-500"
                    }`}
                    style={{ width: `${Math.min(budget.percent_used, 100)}%` }}
                  />
                </div>
              </div>
            ))}
          </div>
          
          {budgets.length > 4 && (
            <p className="text-xs text-zinc-400 mt-3">
              {t("dashboard.andMoreBudgets", { count: budgets.length - 4 })}
            </p>
          )}
        </div>
      )}

      {insights && (insights.forecast || insights.anomalies.length > 0) && (
        <div className="grid gap-4 md:grid-cols-2 animate-fade-in">
          {insights.forecast && (
            <div className="p-5 rounded-xl border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900/50 card-hover">
              <div className="flex items-center gap-2 mb-4">
                <div className="p-2 rounded-lg bg-purple-500/10">
                  <BarChart3 size={18} className="text-purple-500" />
                </div>
                <h3 className="font-medium text-zinc-900 dark:text-zinc-100">
                  {t("dashboard.forecastFor", { month: getNextMonthName() })}
                </h3>
              </div>

              <div className="space-y-3">
                <div>
                  <span className="text-sm text-zinc-400">{t("dashboard.expectedExpenses")}</span>
                  <div className="text-2xl font-semibold text-zinc-900 dark:text-zinc-100">
                    ~{formatCurrency(insights.forecast.predicted_expense)} ₸
                  </div>
                </div>

                <div className="text-sm text-zinc-400">
                  {t("dashboard.range", {
                    low: formatCurrency(insights.forecast.confidence_low),
                    high: formatCurrency(insights.forecast.confidence_high),
                  })}
                </div>

                <div className="flex items-center gap-2 pt-2 border-t border-zinc-200 dark:border-zinc-700">
                  {insights.forecast.trend === "up" && (
                    <>
                      <ArrowUpRight size={16} className="text-red-500" />
                      <span className="text-sm text-red-500">
                        {t("dashboard.trendUp", { percent: Math.abs(insights.forecast.trend_percent) })}
                      </span>
                    </>
                  )}
                  {insights.forecast.trend === "down" && (
                    <>
                      <ArrowDownRight size={16} className="text-emerald-500" />
                      <span className="text-sm text-emerald-500">
                        {t("dashboard.trendDown", { percent: insights.forecast.trend_percent })}
                      </span>
                    </>
                  )}
                  {insights.forecast.trend === "stable" && (
                    <>
                      <Minus size={16} className="text-zinc-400" />
                      <span className="text-sm text-zinc-400">
                        {t("dashboard.trendStable")}
                      </span>
                    </>
                  )}
                </div>
              </div>
            </div>
          )}

          {insights.anomalies.length > 0 && (
            <div className="p-5 rounded-xl border border-amber-500/30 bg-amber-500/5 dark:bg-amber-500/10">
              <div className="flex items-center gap-2 mb-4">
                <div className="p-2 rounded-lg bg-amber-500/10">
                  <AlertTriangle size={18} className="text-amber-500" />
                </div>
                <h3 className="font-medium text-zinc-900 dark:text-zinc-100">
                  {t("dashboard.attention")}
                </h3>
              </div>

              <ul className="space-y-3">
                {insights.anomalies.slice(0, 3).map((anomaly, index) => (
                  <li key={index} className="flex items-start gap-2">
                    <span
                      className={`mt-1.5 w-2 h-2 rounded-full shrink-0 ${
                        anomaly.severity === "alert" ? "bg-red-500" : "bg-amber-500"
                      }`}
                    />
                    <span className="text-sm text-zinc-600 dark:text-zinc-300">
                      {anomaly.message}
                    </span>
                  </li>
                ))}
              </ul>

              {insights.anomalies.length > 3 && (
                <p className="text-xs text-zinc-400 mt-3">
                  {t("dashboard.andMoreAlerts", { count: insights.anomalies.length - 3 })}
                </p>
              )}
            </div>
          )}
        </div>
      )}

      {insights && !insights.forecast && insights.anomalies.length === 0 && insights.months_of_data < 3 && (
        <div className="p-5 rounded-xl border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900/50 animate-fade-in">
          <div className="flex items-center gap-3">
            <div className="p-2.5 rounded-lg bg-purple-500/10">
              <Clock size={20} className="text-purple-500" />
            </div>
            <div className="flex-1">
              <h3 className="font-medium text-zinc-900 dark:text-zinc-100 mb-1">
                {t("dashboard.forecastTitle")}
              </h3>
              <p className="text-sm text-zinc-500 dark:text-zinc-400">
                {t("dashboard.forecastNeedData")}
              </p>
              <div className="mt-2 flex items-center gap-2">
                <div className="flex-1 h-2 rounded-full bg-zinc-200 dark:bg-zinc-700 overflow-hidden">
                  <div 
                    className="h-full rounded-full bg-purple-500 transition-all"
                    style={{ width: `${Math.min((insights.months_of_data / 3) * 100, 100)}%` }}
                  />
                </div>
                <span className="text-xs text-zinc-400 whitespace-nowrap">
                  {t("dashboard.monthsProgress", { current: insights.months_of_data })}
                </span>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
