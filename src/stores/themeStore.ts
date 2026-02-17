const STORAGE_KEY = "finance-theme";

export type Theme = "dark" | "light" | "system";

function getSystemPrefersDark(): boolean {
  if (typeof window === "undefined") return false;
  return window.matchMedia("(prefers-color-scheme: dark)").matches;
}

/** Текущая эффективная тема (dark/light). При "system" — по ОС. */
export function getEffectiveTheme(): "dark" | "light" {
  const stored = getTheme();
  if (stored === "system") return getSystemPrefersDark() ? "dark" : "light";
  return stored;
}

export function getTheme(): Theme {
  if (typeof window === "undefined") return "dark";
  const v = localStorage.getItem(STORAGE_KEY) as Theme | null;
  return v === "dark" || v === "light" || v === "system" ? v : "dark";
}

export function setTheme(theme: Theme) {
  localStorage.setItem(STORAGE_KEY, theme);
  const isDark = theme === "system" ? getSystemPrefersDark() : theme === "dark";
  document.documentElement.classList.toggle("dark", isDark);
}

let systemListenerAdded = false;

export function initTheme() {
  const theme = getTheme();
  setTheme(theme);
  if (typeof window !== "undefined" && !systemListenerAdded) {
    systemListenerAdded = true;
    window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change", () => {
      if (getTheme() === "system") setTheme("system");
    });
  }
}
