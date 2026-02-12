import { useEffect, useState } from "react";
import { Outlet, useLocation } from "react-router-dom";
import { Sidebar } from "./Sidebar";
import { Header } from "./Header";
import { api, type Summary } from "../../lib/api";

const pageTitles: Record<string, string> = {
  "/": "Главная",
  "/transactions": "Транзакции",
  "/import": "Импорт выписки",
  "/accounts": "Счета",
  "/reports": "Отчёты",
  "/settings": "Настройки",
};

export function Layout() {
  const { pathname } = useLocation();
  const title = pageTitles[pathname] ?? "Финансы";
  const [summary, setSummary] = useState<Summary | null>(null);
  const [loading, setLoading] = useState(true);

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

  return (
    <div className="flex h-screen overflow-hidden bg-zinc-100 dark:bg-zinc-950 text-zinc-900 dark:text-zinc-100">
      {/* Fixed Sidebar */}
      <Sidebar totalBalance={summary?.total_balance} />
      
      {/* Scrollable Content Area */}
      <div className="flex-1 flex flex-col min-w-0 h-screen overflow-hidden">
        <Header 
          title={title} 
          totalBalance={summary?.total_balance} 
          balanceLoading={loading} 
        />
        <main className="flex-1 p-6 overflow-y-auto">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
