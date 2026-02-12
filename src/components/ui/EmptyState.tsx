import type { ReactNode } from "react";
import type { LucideIcon } from "lucide-react";

interface EmptyStateProps {
  icon: LucideIcon;
  title: string;
  description?: string;
  action?: ReactNode;
  iconClassName?: string;
}

export function EmptyState({
  icon: Icon,
  title,
  description,
  action,
  iconClassName = "text-zinc-400",
}: EmptyStateProps) {
  return (
    <div className="rounded-xl border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900/50 p-12 text-center animate-fade-in">
      <div className="flex justify-center mb-4">
        <div className="p-4 rounded-full bg-zinc-100 dark:bg-zinc-800">
          <Icon size={40} className={iconClassName} />
        </div>
      </div>
      <h3 className="text-lg font-medium text-zinc-900 dark:text-zinc-100 mb-2">
        {title}
      </h3>
      {description && (
        <p className="text-zinc-500 dark:text-zinc-400 mb-6 max-w-sm mx-auto">
          {description}
        </p>
      )}
      {action && <div className="flex justify-center">{action}</div>}
    </div>
  );
}
