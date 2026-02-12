import { useEffect, useState } from "react";
import { TrendingUp, TrendingDown, Minus, Lightbulb, PiggyBank, BarChart3, Calendar, ArrowUpRight, ArrowDownRight, Target } from "lucide-react";
import { api, type SmartInsights, type Insights, type ForecastDetails } from "../lib/api";

function formatAmount(amount: number) {
  return new Intl.NumberFormat("ru-KZ", {
    style: "decimal",
    minimumFractionDigits: 0,
    maximumFractionDigits: 0,
  }).format(amount);
}

export function InsightsPage() {
  const [smartInsights, setSmartInsights] = useState<SmartInsights | null>(null);
  const [basicInsights, setBasicInsights] = useState<Insights | null>(null);
  const [forecastDetails, setForecastDetails] = useState<ForecastDetails | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadData = async () => {
    try {
      setLoading(true);
      setError(null);
      const [smart, basic, forecast] = await Promise.all([
        api.getSmartInsights().catch(() => null),
        api.getInsights().catch(() => null),
        api.getForecastDetails().catch(() => null),
      ]);
      setSmartInsights(smart);
      setBasicInsights(basic);
      setForecastDetails(forecast);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadData();
  }, []);

  const getNextMonthName = () => {
    const now = new Date();
    const nextMonth = new Date(now.getFullYear(), now.getMonth() + 1, 1);
    return nextMonth.toLocaleDateString("ru-RU", { month: "long" });
  };

  const getCurrentMonthName = () => {
    const now = new Date();
    return now.toLocaleDateString("ru-RU", { month: "long" });
  };

  if (loading) {
    return (
      <div className="space-y-6">
        <h3 className="text-lg font-medium">Аналитика</h3>
        <div className="grid gap-4 md:grid-cols-2">
          {Array.from({ length: 4 }).map((_, i) => (
            <div key={i} className="p-5 rounded-xl border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900/50">
              <div className="h-6 w-32 rounded bg-zinc-200 dark:bg-zinc-700 animate-pulse mb-4" />
              <div className="space-y-3">
                <div className="h-4 w-full rounded bg-zinc-200 dark:bg-zinc-700 animate-pulse" />
                <div className="h-4 w-3/4 rounded bg-zinc-200 dark:bg-zinc-700 animate-pulse" />
              </div>
            </div>
          ))}
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-4 rounded-lg bg-red-500/10 text-red-500 border border-red-500/20">
        {error}
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <h3 className="text-lg font-medium">Аналитика</h3>

      <div className="grid gap-4 md:grid-cols-2">
        {/* Monthly Comparison */}
        {smartInsights && (
          <div className="p-5 rounded-xl border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900/50">
            <div className="flex items-center gap-2 mb-4">
              <div className="p-2 rounded-lg bg-blue-500/10">
                <Calendar size={18} className="text-blue-500" />
              </div>
              <h4 className="font-medium">Сравнение с прошлым месяцем</h4>
            </div>

            <div className="space-y-3">
              <div className="flex justify-between">
                <span className="text-zinc-500 dark:text-zinc-400">Расходы ({getCurrentMonthName()})</span>
                <span className="font-semibold">{formatAmount(smartInsights.monthly_comparison.current_month_total)} ₸</span>
              </div>
              <div className="flex justify-between">
                <span className="text-zinc-500 dark:text-zinc-400">Прошлый месяц</span>
                <span className="text-zinc-600 dark:text-zinc-300">{formatAmount(smartInsights.monthly_comparison.previous_month_total)} ₸</span>
              </div>
              
              <div className="pt-2 border-t border-zinc-200 dark:border-zinc-700">
                <div className="flex items-center gap-2">
                  {smartInsights.monthly_comparison.change_percent > 0 ? (
                    <>
                      <ArrowUpRight size={16} className="text-red-500" />
                      <span className="text-red-500">
                        +{smartInsights.monthly_comparison.change_percent.toFixed(0)}% к прошлому месяцу
                      </span>
                    </>
                  ) : smartInsights.monthly_comparison.change_percent < 0 ? (
                    <>
                      <ArrowDownRight size={16} className="text-emerald-500" />
                      <span className="text-emerald-500">
                        {smartInsights.monthly_comparison.change_percent.toFixed(0)}% к прошлому месяцу
                      </span>
                    </>
                  ) : (
                    <>
                      <Minus size={16} className="text-zinc-400" />
                      <span className="text-zinc-400">Без изменений</span>
                    </>
                  )}
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Forecast */}
        {basicInsights?.forecast && (
          <div className="p-5 rounded-xl border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900/50">
            <div className="flex items-center gap-2 mb-4">
              <div className="p-2 rounded-lg bg-purple-500/10">
                <BarChart3 size={18} className="text-purple-500" />
              </div>
              <h4 className="font-medium">Прогноз на {getNextMonthName()}</h4>
            </div>

            <div className="space-y-3">
              <div>
                <span className="text-sm text-zinc-400">Ожидаемые расходы</span>
                <p className="text-2xl font-semibold">
                  ~{formatAmount(basicInsights.forecast.predicted_expense)} ₸
                </p>
              </div>
              <div className="text-sm text-zinc-400">
                Диапазон: {formatAmount(basicInsights.forecast.confidence_low)} – {formatAmount(basicInsights.forecast.confidence_high)} ₸
              </div>
              <div className="flex items-center gap-2 pt-2 border-t border-zinc-200 dark:border-zinc-700">
                {basicInsights.forecast.trend === "up" && (
                  <>
                    <TrendingUp size={16} className="text-red-500" />
                    <span className="text-sm text-red-500">Рост расходов</span>
                  </>
                )}
                {basicInsights.forecast.trend === "down" && (
                  <>
                    <TrendingDown size={16} className="text-emerald-500" />
                    <span className="text-sm text-emerald-500">Снижение расходов</span>
                  </>
                )}
                {basicInsights.forecast.trend === "stable" && (
                  <>
                    <Minus size={16} className="text-zinc-400" />
                    <span className="text-sm text-zinc-400">Стабильный уровень</span>
                  </>
                )}
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Category Forecasts */}
      {forecastDetails && forecastDetails.by_category.length > 0 && (
        <div className="p-5 rounded-xl border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900/50">
          <div className="flex items-center gap-2 mb-4">
            <div className="p-2 rounded-lg bg-indigo-500/10">
              <Target size={18} className="text-indigo-500" />
            </div>
            <h4 className="font-medium">Прогноз по категориям на {getNextMonthName()}</h4>
          </div>

          <div className="grid gap-3 sm:grid-cols-2">
            {forecastDetails.by_category
              .sort((a, b) => b.predicted_expense - a.predicted_expense)
              .slice(0, 8)
              .map((cat) => (
                <div
                  key={cat.category_id}
                  className="flex items-center justify-between p-3 rounded-lg bg-zinc-50 dark:bg-zinc-800/50"
                >
                  <span className="font-medium text-sm">{cat.category_name}</span>
                  <span className="text-sm text-zinc-600 dark:text-zinc-300">
                    ~{formatAmount(cat.predicted_expense)} ₸
                  </span>
                </div>
              ))}
          </div>

          {forecastDetails.by_category.length > 8 && (
            <p className="text-xs text-zinc-400 mt-3 text-center">
              Показаны топ 8 категорий из {forecastDetails.by_category.length}
            </p>
          )}
        </div>
      )}

      {/* Savings Suggestions */}
      {smartInsights && smartInsights.suggestions.length > 0 && (
        <div className="p-5 rounded-xl border border-amber-500/30 bg-amber-500/5">
          <div className="flex items-center gap-2 mb-4">
            <div className="p-2 rounded-lg bg-amber-500/10">
              <Lightbulb size={18} className="text-amber-500" />
            </div>
            <h4 className="font-medium">Рекомендации по экономии</h4>
          </div>

          <div className="space-y-3">
            {smartInsights.suggestions.map((suggestion, i) => (
              <div key={i} className="p-4 rounded-lg bg-white dark:bg-zinc-900/50 border border-zinc-200 dark:border-zinc-700">
                <p className="text-zinc-700 dark:text-zinc-200 mb-2">{suggestion.suggestion}</p>
                <div className="flex items-center gap-4 text-sm text-zinc-500 dark:text-zinc-400">
                  <span>Возможная экономия: <strong className="text-emerald-500">{formatAmount(suggestion.potential_savings)} ₸</strong></span>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Spending Patterns */}
      {smartInsights && smartInsights.patterns.length > 0 && (
        <div className="p-5 rounded-xl border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900/50">
          <div className="flex items-center gap-2 mb-4">
            <div className="p-2 rounded-lg bg-emerald-500/10">
              <PiggyBank size={18} className="text-emerald-500" />
            </div>
            <h4 className="font-medium">Паттерны расходов</h4>
          </div>

          <div className="space-y-3">
            {smartInsights.patterns.slice(0, 6).map((pattern, i) => (
              <div key={i} className="flex items-center justify-between p-3 rounded-lg bg-zinc-50 dark:bg-zinc-800/50">
                <div className="flex items-center gap-3">
                  <div
                    className="w-3 h-3 rounded-full"
                    style={{ backgroundColor: pattern.category_color || "#64748b" }}
                  />
                  <div>
                    <span className="font-medium">{pattern.category}</span>
                    <p className="text-xs text-zinc-400">{pattern.typical_frequency}</p>
                  </div>
                </div>
                <div className="text-right">
                  <span className="font-medium">~{formatAmount(pattern.avg_amount)} ₸</span>
                  <div className="flex items-center gap-1 justify-end">
                    {pattern.trend === "increasing" && (
                      <>
                        <TrendingUp size={12} className="text-red-500" />
                        <span className="text-xs text-red-500">+{Math.abs(pattern.trend_percent).toFixed(0)}%</span>
                      </>
                    )}
                    {pattern.trend === "decreasing" && (
                      <>
                        <TrendingDown size={12} className="text-emerald-500" />
                        <span className="text-xs text-emerald-500">{pattern.trend_percent.toFixed(0)}%</span>
                      </>
                    )}
                    {pattern.trend === "stable" && (
                      <span className="text-xs text-zinc-400">стабильно</span>
                    )}
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Anomalies */}
      {basicInsights && basicInsights.anomalies.length > 0 && (
        <div className="p-5 rounded-xl border border-red-500/30 bg-red-500/5">
          <div className="flex items-center gap-2 mb-4">
            <div className="p-2 rounded-lg bg-red-500/10">
              <TrendingUp size={18} className="text-red-500" />
            </div>
            <h4 className="font-medium">Необычные траты</h4>
          </div>

          <ul className="space-y-2">
            {basicInsights.anomalies.map((anomaly, i) => (
              <li key={i} className="flex items-start gap-2">
                <span
                  className={`mt-1.5 w-2 h-2 rounded-full shrink-0 ${
                    anomaly.severity === "alert" ? "bg-red-500" : "bg-amber-500"
                  }`}
                />
                <span className="text-sm text-zinc-600 dark:text-zinc-300">{anomaly.message}</span>
              </li>
            ))}
          </ul>
        </div>
      )}

      {/* Empty state */}
      {(!smartInsights || (smartInsights.patterns.length === 0 && smartInsights.suggestions.length === 0)) && 
       (!basicInsights || (!basicInsights.forecast && basicInsights.anomalies.length === 0)) && (
        <div className="text-center py-12">
          <BarChart3 size={48} className="mx-auto text-zinc-300 dark:text-zinc-600 mb-4" />
          <h4 className="text-lg font-medium text-zinc-600 dark:text-zinc-400 mb-2">Недостаточно данных</h4>
          <p className="text-zinc-400">
            Добавьте больше транзакций, чтобы увидеть аналитику и рекомендации
          </p>
        </div>
      )}
    </div>
  );
}
