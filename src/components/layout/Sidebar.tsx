import { NavLink } from "react-router-dom";
import { LayoutDashboard, ArrowLeftRight, Wallet, BarChart3, Settings, TrendingUp, Repeat, Tag, Lightbulb, FileUp } from "lucide-react";

const navItems = [
  { to: "/", icon: LayoutDashboard, label: "Главная" },
  { to: "/transactions", icon: ArrowLeftRight, label: "Транзакции" },
  { to: "/import", icon: FileUp, label: "Импорт выписки" },
  { to: "/accounts", icon: Wallet, label: "Счета" },
  { to: "/recurring", icon: Repeat, label: "Автоплатежи" },
  { to: "/categories", icon: Tag, label: "Категории" },
  { to: "/insights", icon: Lightbulb, label: "Аналитика" },
  { to: "/reports", icon: BarChart3, label: "Отчёты" },
  { to: "/settings", icon: Settings, label: "Настройки" },
];

function formatAmount(amount: number) {
  return new Intl.NumberFormat("ru-KZ", {
    style: "decimal",
    minimumFractionDigits: 0,
    maximumFractionDigits: 0,
  }).format(amount);
}

interface SidebarProps {
  totalBalance?: number;
}

export function Sidebar({ totalBalance }: SidebarProps) {
  return (
    <aside className="w-64 h-screen flex-shrink-0 bg-zinc-900 dark:bg-zinc-900 text-zinc-100 flex flex-col sticky top-0">
      <div className="p-6 border-b border-zinc-700 dark:border-zinc-700 flex-shrink-0">
        <h1 className="text-xl font-semibold">Финансы</h1>
      </div>
      <nav className="flex-1 p-4 space-y-1 overflow-y-auto">
        {navItems.map(({ to, icon: Icon, label }) => (
          <NavLink
            key={to}
            to={to}
            end={to === "/"}
            className={({ isActive }) =>
              `relative flex items-center gap-3 px-4 py-3 rounded-lg nav-link-transition ${
                isActive 
                  ? "bg-zinc-700/80 text-white pl-5" 
                  : "text-zinc-400 hover:bg-zinc-800 hover:text-zinc-200"
              }`
            }
          >
            {({ isActive }) => (
              <>
                {isActive && (
                  <span className="absolute left-0 top-1/2 -translate-y-1/2 w-1 h-6 bg-emerald-500 rounded-r-full" />
                )}
                <Icon size={20} />
                <span>{label}</span>
              </>
            )}
          </NavLink>
        ))}
      </nav>
      
      {/* Balance section at bottom */}
      <div className="p-4 border-t border-zinc-700 flex-shrink-0">
        <div className="p-4 rounded-xl bg-gradient-to-br from-zinc-800 to-zinc-800/50 border border-zinc-700/50">
          <div className="flex items-center gap-2 text-zinc-400 text-sm mb-2">
            <TrendingUp size={14} />
            <span>Общий баланс</span>
          </div>
          {totalBalance !== undefined ? (
            <p className="text-lg font-semibold text-emerald-400">
              {formatAmount(totalBalance)} ₸
            </p>
          ) : (
            <div className="h-6 w-28 rounded bg-zinc-700 animate-pulse" />
          )}
        </div>
      </div>
    </aside>
  );
}
