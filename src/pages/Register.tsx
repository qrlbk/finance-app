import { useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { api } from "../lib/api";

function getErrorMessage(err: unknown, fallback: string): string {
  if (err instanceof Error) return err.message;
  if (typeof err === "string") return err;
  if (err && typeof err === "object" && "message" in err && typeof (err as { message: unknown }).message === "string") {
    return (err as { message: string }).message;
  }
  return fallback;
}

export function Register() {
  const { t } = useTranslation();
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [displayName, setDisplayName] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const navigate = useNavigate();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    const u = username.trim();
    if (!u || !password || !displayName.trim()) {
      setError(t("register.errorFillAll"));
      return;
    }
    if (password.length < 6) {
      setError(t("register.errorPasswordLength"));
      return;
    }
    setLoading(true);
    try {
      await api.register(u, password, displayName.trim());
      navigate("/", { replace: true });
    } catch (err) {
      setError(getErrorMessage(err, t("register.errorFailed")));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-zinc-100 dark:bg-zinc-950 p-4">
      <div className="w-full max-w-sm rounded-2xl border border-zinc-200 dark:border-zinc-800 bg-white dark:bg-zinc-900 shadow-xl p-8">
        <h1 className="text-2xl font-semibold text-zinc-900 dark:text-zinc-100 text-center mb-6">
          {t("register.title")}
        </h1>
        <form onSubmit={handleSubmit} className="space-y-4">
          {error && (
            <p className="text-sm text-red-500 dark:text-red-400 text-center" role="alert">
              {error}
            </p>
          )}
          <div>
            <label htmlFor="reg-username" className="block text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-1">
              {t("register.username")}
            </label>
            <input
              id="reg-username"
              type="text"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              autoComplete="username"
              className="w-full px-4 py-2 rounded-lg border border-zinc-300 dark:border-zinc-600 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-100 focus:ring-2 focus:ring-emerald-500 focus:border-transparent"
              placeholder={t("register.placeholderUsername")}
            />
          </div>
          <div>
            <label htmlFor="reg-display" className="block text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-1">
              {t("register.displayName")}
            </label>
            <input
              id="reg-display"
              type="text"
              value={displayName}
              onChange={(e) => setDisplayName(e.target.value)}
              autoComplete="name"
              className="w-full px-4 py-2 rounded-lg border border-zinc-300 dark:border-zinc-600 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-100 focus:ring-2 focus:ring-emerald-500 focus:border-transparent"
              placeholder={t("register.placeholderDisplay")}
            />
          </div>
          <div>
            <label htmlFor="reg-password" className="block text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-1">
              {t("register.password")}
            </label>
            <input
              id="reg-password"
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              autoComplete="new-password"
              className="w-full px-4 py-2 rounded-lg border border-zinc-300 dark:border-zinc-600 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-100 focus:ring-2 focus:ring-emerald-500 focus:border-transparent"
            />
          </div>
          <button
            type="submit"
            disabled={loading}
            className="w-full py-2.5 rounded-lg bg-emerald-600 hover:bg-emerald-700 disabled:opacity-50 text-white font-medium transition-colors"
          >
            {loading ? t("register.submitting") : t("register.submit")}
          </button>
        </form>
        <p className="mt-6 text-center text-sm text-zinc-500 dark:text-zinc-400">
          {t("register.hasAccount")}{" "}
          <Link to="/login" className="text-emerald-600 dark:text-emerald-400 hover:underline">
            {t("register.loginLink")}
          </Link>
        </p>
      </div>
    </div>
  );
}
