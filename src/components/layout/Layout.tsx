import { useEffect, useState, useRef } from "react";
import { Outlet, useLocation } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { Sidebar } from "./Sidebar";
import { Header } from "./Header";
import { api, type Summary, type BudgetAlert } from "../../lib/api";
import { useToast } from "../ui/Toast";

const pathToTitleKey: Record<string, string> = {
  "/": "nav.dashboard",
  "/transactions": "nav.transactions",
  "/transfers": "nav.transfers",
  "/import": "nav.import",
  "/accounts": "nav.accounts",
  "/recurring": "nav.recurring",
  "/categories": "nav.categories",
  "/insights": "nav.insights",
  "/reports": "nav.reports",
  "/chat": "nav.chat",
  "/docs": "docs.title",
  "/settings": "nav.settings",
};

export function Layout() {
  const { t } = useTranslation();
  const { pathname } = useLocation();
  const titleKey = pathToTitleKey[pathname] ?? "nav.appName";
  const title = t(titleKey);
  const [summary, setSummary] = useState<Summary | null>(null);
  const [loading, setLoading] = useState(true);
  const [budgetAlerts, setBudgetAlerts] = useState<BudgetAlert[]>([]);
  const recurringProcessedRef = useRef(false);
  const { showToast } = useToast();

  useEffect(() => {
    if (recurringProcessedRef.current) return;
    recurringProcessedRef.current = true;
    api.processRecurringPayments().then((created) => {
      if (created.length > 0) {
        showToast(t("layout.recurringProcessed", { count: created.length }), "success");
      }
    }).catch(() => { /* ignore */ });
  }, [showToast, t]);

  useEffect(() => {
    const loadSummary = async () => {
      try {
        const data = await api.getSummary();
        setSummary(data);
      } catch {
        // Ignore errors for header balance
      } finally {
        setLoading(false);
      }
    };
    loadSummary();

    const interval = setInterval(loadSummary, 30000);
    return () => clearInterval(interval);
  }, [pathname]);

  useEffect(() => {
    api.getBudgetAlerts().then(setBudgetAlerts).catch(() => setBudgetAlerts([]));
  }, [pathname]);

  return (
    <div className="flex h-screen overflow-hidden bg-zinc-100 dark:bg-zinc-950 text-zinc-900 dark:text-zinc-100">
      <Sidebar totalBalance={summary?.total_balance} currencies={summary?.currencies ?? []} />
      
      <div className="flex-1 flex flex-col min-w-0 h-screen overflow-hidden">
        <Header 
          title={title} 
          totalBalance={summary?.total_balance} 
          balanceLoading={loading}
          budgetAlerts={budgetAlerts}
          currencies={summary?.currencies ?? []}
          baseCurrency={summary?.base_currency ?? "KZT"}
        />
        <main className="flex-1 p-6 overflow-y-auto">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
