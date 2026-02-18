import { useTranslation } from "react-i18next";
import { Moon, Sun, Monitor } from "lucide-react";
import type { Theme } from "../../stores/themeStore";

interface SettingsThemeProps {
  theme: Theme;
  onThemeChange: (theme: Theme) => void;
}

export function SettingsTheme({ theme, onThemeChange }: SettingsThemeProps) {
  const { t } = useTranslation();
  return (
    <div>
      <h3 className="text-lg font-medium mb-4">{t("settings.theme")}</h3>
      <div className="flex gap-4">
        <button
          onClick={() => onThemeChange("dark")}
          className={`flex items-center gap-2 px-4 py-3 rounded-xl border transition-colors ${
            theme === "dark"
              ? "bg-zinc-700 dark:bg-zinc-700 border-zinc-600 text-white"
              : "bg-zinc-100 dark:bg-zinc-800 border-zinc-300 dark:border-zinc-700 text-zinc-700 dark:text-zinc-400 hover:border-zinc-400 dark:hover:border-zinc-600"
          }`}
        >
          <Moon size={20} />
          {t("settings.themeDark")}
        </button>
        <button
          onClick={() => onThemeChange("light")}
          className={`flex items-center gap-2 px-4 py-3 rounded-xl border transition-colors ${
            theme === "light"
              ? "bg-zinc-700 dark:bg-zinc-700 border-zinc-600 text-white"
              : "bg-zinc-100 dark:bg-zinc-800 border-zinc-300 dark:border-zinc-700 text-zinc-700 dark:text-zinc-400 hover:border-zinc-400 dark:hover:border-zinc-600"
          }`}
        >
          <Sun size={20} />
          {t("settings.themeLight")}
        </button>
        <button
          onClick={() => onThemeChange("system")}
          className={`flex items-center gap-2 px-4 py-3 rounded-xl border transition-colors ${
            theme === "system"
              ? "bg-zinc-700 dark:bg-zinc-700 border-zinc-600 text-white"
              : "bg-zinc-100 dark:bg-zinc-800 border-zinc-300 dark:border-zinc-700 text-zinc-700 dark:text-zinc-400 hover:border-zinc-400 dark:hover:border-zinc-600"
          }`}
        >
          <Monitor size={20} />
          {t("settings.themeSystem")}
        </button>
      </div>
    </div>
  );
}
