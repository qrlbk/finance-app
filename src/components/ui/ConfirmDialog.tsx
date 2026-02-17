import { useEffect, useRef } from "react";
import { AlertTriangle, Info } from "lucide-react";

interface ConfirmDialogProps {
  open: boolean;
  title: string;
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
  variant?: "danger" | "default";
  loading?: boolean;
  loadingConfirmLabel?: string;
  onConfirm: () => void;
  onCancel: () => void;
}

export function ConfirmDialog({
  open,
  title,
  message,
  confirmLabel = "Подтвердить",
  cancelLabel = "Отмена",
  variant = "default",
  loading = false,
  loadingConfirmLabel = "Загрузка…",
  onConfirm,
  onCancel,
}: ConfirmDialogProps) {
  const confirmButtonRef = useRef<HTMLButtonElement>(null);
  const dialogRef = useRef<HTMLDivElement>(null);

  // Focus confirm button when dialog opens
  useEffect(() => {
    if (open && confirmButtonRef.current) {
      setTimeout(() => confirmButtonRef.current?.focus(), 100);
    }
  }, [open]);

  // Handle escape key
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && open) {
        onCancel();
      }
    };
    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [open, onCancel]);

  if (!open) return null;

  const confirmClass =
    variant === "danger"
      ? "bg-red-600 hover:bg-red-700 text-white focus:ring-red-500"
      : "bg-emerald-600 hover:bg-emerald-700 text-white focus:ring-emerald-500";

  const Icon = variant === "danger" ? AlertTriangle : Info;
  const iconBg = variant === "danger" ? "bg-red-500/10" : "bg-blue-500/10";
  const iconColor = variant === "danger" ? "text-red-500" : "text-blue-500";

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center p-4 animate-fade-in"
      role="dialog"
      aria-modal="true"
      aria-labelledby="confirm-dialog-title"
      onClick={onCancel}
    >
      {/* Backdrop with blur */}
      <div className="absolute inset-0 bg-black/50 backdrop-blur-overlay" />
      
      {/* Dialog */}
      <div
        ref={dialogRef}
        className="relative w-full max-w-md rounded-xl bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700 shadow-2xl p-6 animate-scale-in"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Icon */}
        <div className={`w-12 h-12 rounded-full ${iconBg} flex items-center justify-center mb-4`}>
          <Icon size={24} className={iconColor} />
        </div>

        <h3 
          id="confirm-dialog-title" 
          className="text-lg font-semibold text-zinc-900 dark:text-zinc-100 mb-2"
        >
          {title}
        </h3>
        <p className="text-zinc-600 dark:text-zinc-400 mb-6">{message}</p>
        
        <div className="flex justify-end gap-3">
          <button
            type="button"
            onClick={onCancel}
            className="px-4 py-2.5 rounded-lg bg-zinc-100 dark:bg-zinc-800 text-zinc-700 dark:text-zinc-300 hover:bg-zinc-200 dark:hover:bg-zinc-700 btn-transition focus:outline-none focus:ring-2 focus:ring-zinc-500 focus:ring-offset-2 dark:focus:ring-offset-zinc-900"
          >
            {cancelLabel}
          </button>
          <button
            ref={confirmButtonRef}
            type="button"
            onClick={onConfirm}
            disabled={loading}
            className={`px-4 py-2.5 rounded-lg btn-transition focus:outline-none focus:ring-2 focus:ring-offset-2 dark:focus:ring-offset-zinc-900 disabled:opacity-50 disabled:cursor-not-allowed ${confirmClass}`}
          >
            {loading ? loadingConfirmLabel : confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}
