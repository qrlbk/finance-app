import type { ReactNode } from "react";

declare global {
  interface Window {
    __TAURI_INTERNALS__?: { invoke?: unknown };
    __TAURI__?: { invoke?: unknown };
  }
}

function isTauri(): boolean {
  if (typeof window === "undefined") return true; // SSR
  const inv = window.__TAURI_INTERNALS__?.invoke ?? window.__TAURI__?.invoke;
  return typeof inv === "function";
}

interface TauriGuardProps {
  children: ReactNode;
}

/** Рендерит children только внутри окна Tauri. В браузере показывает подсказку. */
export function TauriGuard({ children }: TauriGuardProps) {
  if (!isTauri()) {
    return (
      <div className="min-h-screen bg-zinc-100 dark:bg-zinc-950 text-zinc-900 dark:text-zinc-100 flex items-center justify-center p-6">
        <div className="max-w-md text-center space-y-4">
          <h1 className="text-xl font-semibold">Запустите приложение через Tauri</h1>
          <p className="text-zinc-600 dark:text-zinc-400 text-sm">
            Эта программа — настольное приложение. Открытие в обычном браузере не поддерживается.
          </p>
          <p className="text-zinc-600 dark:text-zinc-400 text-sm">
            В терминале выполните: <code className="bg-zinc-200 dark:bg-zinc-800 px-2 py-1 rounded">npm run tauri dev</code>
          </p>
        </div>
      </div>
    );
  }
  return <>{children}</>;
}
