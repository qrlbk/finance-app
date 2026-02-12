import { Link } from "react-router-dom";
import { Plus, Wallet } from "lucide-react";

function formatAmount(amount: number) {
  return new Intl.NumberFormat("ru-KZ", {
    style: "decimal",
    minimumFractionDigits: 0,
    maximumFractionDigits: 0,
  }).format(amount);
}

interface HeaderProps {
  title: string;
  totalBalance?: number;
  balanceLoading?: boolean;
}

export function Header({ title, totalBalance, balanceLoading }: HeaderProps) {
  return (
    <header className="h-16 px-6 flex items-center justify-between border-b border-zinc-200 dark:border-zinc-800 bg-white dark:bg-zinc-950 text-zinc-900 dark:text-zinc-100 flex-shrink-0">
      <h2 className="text-lg font-medium">{title}</h2>
      
      <div className="flex items-center gap-4">
        {/* Balance display */}
        <div className="hidden sm:flex items-center gap-2 px-4 py-2 rounded-lg bg-zinc-100 dark:bg-zinc-800/50">
          <Wallet size={18} className="text-blue-400" />
          {balanceLoading ? (
            <div className="h-5 w-24 rounded bg-zinc-200 dark:bg-zinc-700 animate-pulse" />
          ) : (
            <span className="font-medium text-blue-400">
              {formatAmount(totalBalance ?? 0)} ₸
            </span>
          )}
        </div>
        
        {/* Quick add button */}
        <Link
          to="/transactions"
          className="flex items-center gap-2 px-4 py-2 rounded-lg bg-emerald-600 text-white hover:bg-emerald-700 btn-transition shadow-sm hover:shadow-md"
        >
          <Plus size={18} />
          <span className="hidden sm:inline">Добавить</span>
        </Link>
      </div>
    </header>
  );
}
