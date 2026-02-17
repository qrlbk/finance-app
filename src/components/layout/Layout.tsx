import { useEffect, useState, useRef } from "react";
import { Outlet, useLocation } from "react-router-dom";
import { Sidebar } from "./Sidebar";
import { Header } from "./Header";
import { api, type Summary, type BudgetAlert } from "../../lib/api";
import { useToast } from "../ui/Toast";

const pageTitles: Record<string, string> = {
  "/": "Главная",
  "/transactions": "Транзакции",
  "/transfers": "Переводы",
  "/import": "Импорт выписки",
  "/accounts": "Счета",
  "/recurring": "Автоплатежи",
  "/categories": "Категории",
  "/insights": "Аналитика",
  "/reports": "Отчёты",
  "/chat": "Чат",
  "/settings": "Настройки",
};

export function Layout() {
  const { pathname } = useLocation();
  const title = pageTitles[pathname] ?? "Финансы";
  const [summary, setSummary] = useState<Summary | null>(null);
  const [loading, setLoading] = useState(true);
  const [budgetAlerts, setBudgetAlerts] = useState<BudgetAlert[]>([]);
  const recurringProcessedRef = useRef(false);
  const { showToast } = useToast();

  // Run recurring payments once on app load (non-blocking)
  useEffect(() => {
    if (recurringProcessedRef.current) return;
    recurringProcessedRef.current = true;
    api.processRecurringPayments().then((created) => {
      if (created.length > 0) {
        showToast(`Проведено автоплатежей: ${created.length}`, "success");
      }
    }).catch(() => { /* ignore */ });
  }, [showToast]);

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

    // Refresh summary when navigating to dashboard or after transactions
    const interval = setInterval(loadSummary, 30000);
    return () => clearInterval(interval);
  }, [pathname]);

  useEffect(() => {
    api.getBudgetAlerts().then(setBudgetAlerts).catch(() => setBudgetAlerts([]));
  }, [pathname]);

  return (
    <div className="flex h-screen overflow-hidden bg-zinc-100 dark:bg-zinc-950 text-zinc-900 dark:text-zinc-100">
      {/* Fixed Sidebar */}
      <Sidebar totalBalance={summary?.total_balance} currencies={summary?.currencies ?? []} />
      
      {/* Scrollable Content Area */}
      <div className="flex-1 flex flex-col min-w-0 h-screen overflow-hidden">
        <Header 
          title={title} 
          totalBalance={summary?.total_balance} 
          balanceLoading={loading}
          budgetAlerts={budgetAlerts}
          currencies={summary?.currencies ?? []}
        />
        <main className="flex-1 p-6 overflow-y-auto">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
