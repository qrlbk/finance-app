import { useState, useRef, useEffect } from "react";
import { Link } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { Plus, Wallet, Bell } from "lucide-react";
import type { BudgetAlert } from "../../lib/api";
import { formatCurrency } from "../../lib/format";

function currencyLabel(code: string) {
  const symbols: Record<string, string> = { KZT: "₸", USD: "$", EUR: "€", RUB: "₽" };
  return symbols[code] ?? code;
}

interface HeaderProps {
  title: string;
  totalBalance?: number;
  balanceLoading?: boolean;
  budgetAlerts?: BudgetAlert[];
  currencies?: string[];
  baseCurrency?: string;
}

export function Header({ title, totalBalance, balanceLoading, budgetAlerts = [], currencies = [], baseCurrency = "KZT" }: HeaderProps) {
  const { t } = useTranslation();
  const [alertsOpen, setAlertsOpen] = useState(false);
  const alertsRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (alertsRef.current && !alertsRef.current.contains(e.target as Node)) {
        setAlertsOpen(false);
      }
    };
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  return (
    <header className="h-16 px-6 flex items-center justify-between border-b border-zinc-200 dark:border-zinc-800 bg-white dark:bg-zinc-950 text-zinc-900 dark:text-zinc-100 flex-shrink-0">
      <h2 className="text-lg font-medium">{title}</h2>
      
      <div className="flex items-center gap-4">
        {budgetAlerts.length > 0 && (
          <div className="relative" ref={alertsRef}>
            <button
              type="button"
              onClick={() => setAlertsOpen((v) => !v)}
              className="relative p-2 rounded-lg text-zinc-500 hover:bg-zinc-100 dark:hover:bg-zinc-800 hover:text-amber-500 btn-transition"
              aria-label={t("header.budgetAlerts")}
            >
              <Bell size={20} />
              <span className="absolute -top-0.5 -right-0.5 min-w-[18px] h-[18px] flex items-center justify-center rounded-full bg-amber-500 text-white text-xs font-medium px-1">
                {budgetAlerts.length}
              </span>
            </button>
            {alertsOpen && (
              <div className="absolute right-0 top-full mt-1 w-72 rounded-xl border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900 shadow-lg py-2 z-50">
                <div className="px-3 py-2 border-b border-zinc-200 dark:border-zinc-700">
                  <span className="text-sm font-medium text-zinc-700 dark:text-zinc-300">{t("header.budgets")}</span>
                </div>
                <ul className="max-h-60 overflow-y-auto">
                  {budgetAlerts.map((alert, i) => (
                    <li key={i} className="px-3 py-2 text-sm border-b border-zinc-100 dark:border-zinc-800 last:border-0">
                      <span className="font-medium text-zinc-900 dark:text-zinc-100">{alert.category_name}</span>
                      <span className={alert.severity === "exceeded" ? " text-red-500" : " text-amber-500"}>
                        {" "}
                        {alert.severity === "exceeded"
                          ? t("header.budgetExceeded", { percent: Math.round(alert.percent_used) })
                          : t("header.budgetNearLimit", { percent: Math.round(alert.percent_used) })}
                      </span>
                    </li>
                  ))}
                </ul>
              </div>
            )}
          </div>
        )}

        {currencies.length > 1 && (
          <span className="text-xs text-zinc-500 dark:text-zinc-400 max-w-[140px] sm:max-w-[180px] truncate" title={t("header.balanceInBase")}>
            {t("common.inCurrency", { currency: baseCurrency })}
          </span>
        )}
        <div className="hidden sm:flex items-center gap-2 px-4 py-2 rounded-lg bg-zinc-100 dark:bg-zinc-800/50" title={currencies.length > 1 ? t("header.balanceInBaseCurrency", { currency: baseCurrency }) : undefined}>
          <Wallet size={18} className="text-blue-400" />
          {balanceLoading ? (
            <div className="h-5 w-24 rounded bg-zinc-200 dark:bg-zinc-700 animate-pulse" />
          ) : (
            <span className="font-medium text-blue-400">
              {formatCurrency(totalBalance ?? 0)} {currencyLabel(baseCurrency)}
            </span>
          )}
        </div>
        
        <Link
          to="/transactions"
          className="flex items-center gap-2 px-4 py-2 rounded-lg bg-emerald-600 text-white hover:bg-emerald-700 btn-transition shadow-sm hover:shadow-md"
        >
          <Plus size={18} />
          <span className="hidden sm:inline">{t("header.add")}</span>
        </Link>
      </div>
    </header>
  );
}
