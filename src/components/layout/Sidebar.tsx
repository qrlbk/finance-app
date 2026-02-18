import { NavLink, useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { LayoutDashboard, ArrowLeftRight, Wallet, BarChart3, Settings, TrendingUp, Repeat, Tag, Lightbulb, FileUp, ArrowRightLeft, MessageCircle, LogOut } from "lucide-react";
import { api } from "../../lib/api";
import { formatCurrency } from "../../lib/format";

const navConfig: { to: string; icon: typeof LayoutDashboard; labelKey: string }[] = [
  { to: "/", icon: LayoutDashboard, labelKey: "nav.dashboard" },
  { to: "/transactions", icon: ArrowLeftRight, labelKey: "nav.transactions" },
  { to: "/transfers", icon: ArrowRightLeft, labelKey: "nav.transfers" },
  { to: "/import", icon: FileUp, labelKey: "nav.import" },
  { to: "/accounts", icon: Wallet, labelKey: "nav.accounts" },
  { to: "/recurring", icon: Repeat, labelKey: "nav.recurring" },
  { to: "/categories", icon: Tag, labelKey: "nav.categories" },
  { to: "/insights", icon: Lightbulb, labelKey: "nav.insights" },
  { to: "/reports", icon: BarChart3, labelKey: "nav.reports" },
  { to: "/chat", icon: MessageCircle, labelKey: "nav.chat" },
  { to: "/settings", icon: Settings, labelKey: "nav.settings" },
];

interface SidebarProps {
  totalBalance?: number;
  currencies?: string[];
}

export function Sidebar({ totalBalance, currencies = [] }: SidebarProps) {
  const { t } = useTranslation();
  const navigate = useNavigate();

  const handleLogout = async () => {
    try {
      await api.logout();
      navigate("/login", { replace: true });
    } catch {
      navigate("/login", { replace: true });
    }
  };

  return (
    <aside className="w-64 h-screen flex-shrink-0 bg-zinc-900 dark:bg-zinc-900 text-zinc-100 flex flex-col sticky top-0">
      <div className="p-6 border-b border-zinc-700 dark:border-zinc-700 flex-shrink-0">
        <h1 className="text-xl font-semibold">{t("nav.appName")}</h1>
      </div>
      <nav className="flex-1 p-4 space-y-1 overflow-y-auto">
        {navConfig.map(({ to, icon: Icon, labelKey }) => (
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
                <span>{t(labelKey)}</span>
              </>
            )}
          </NavLink>
        ))}
      </nav>
      
      <div className="p-4 border-t border-zinc-700 flex-shrink-0 space-y-2">
        <div className="p-4 rounded-xl bg-gradient-to-br from-zinc-800 to-zinc-800/50 border border-zinc-700/50">
          <div className="flex items-center gap-2 text-zinc-400 text-sm mb-2">
            <TrendingUp size={14} />
            <span>{t("sidebar.totalBalance")}</span>
          </div>
          {totalBalance !== undefined ? (
            <>
              <p className="text-lg font-semibold text-emerald-400">
                {formatCurrency(totalBalance)} ₸
                {currencies.length > 1 && <span className="block text-xs font-normal text-amber-400/90 mt-0.5">({t("common.withoutConversion")})</span>}
              </p>
              {currencies.length > 1 && (
                <p className="text-xs text-amber-400/90 mt-1" title={t("sidebar.balanceHint")}>
                  {t("common.multipleCurrencies")}
                </p>
              )}
            </>
          ) : (
            <div className="h-6 w-28 rounded bg-zinc-700 animate-pulse" />
          )}
        </div>
        <button
          type="button"
          onClick={handleLogout}
          className="w-full flex items-center gap-3 px-4 py-3 rounded-lg text-zinc-400 hover:bg-zinc-800 hover:text-zinc-200 transition-colors"
        >
          <LogOut size={20} />
          <span>{t("sidebar.logout")}</span>
        </button>
      </div>
    </aside>
  );
}
