import { useState, useEffect } from "react";
import { Link, useNavigate } from "react-router-dom";
import { api, type UserSession } from "../lib/api";

function getErrorMessage(err: unknown, fallback: string): string {
  if (err instanceof Error) return err.message;
  if (typeof err === "string") return err;
  if (err && typeof err === "object" && "message" in err && typeof (err as { message: unknown }).message === "string") {
    return (err as { message: string }).message;
  }
  return fallback;
}

export function Login() {
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [users, setUsers] = useState<UserSession[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const navigate = useNavigate();

  useEffect(() => {
    api.listUsers().then(setUsers).catch(() => setUsers([]));
  }, []);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    if (!username.trim() || !password) {
      setError("Введите имя пользователя и пароль");
      return;
    }
    setLoading(true);
    try {
      await api.login(username.trim(), password);
      navigate("/", { replace: true });
    } catch (err) {
      setError(getErrorMessage(err, "Ошибка входа"));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-zinc-100 dark:bg-zinc-950 p-4">
      <div className="w-full max-w-sm rounded-2xl border border-zinc-200 dark:border-zinc-800 bg-white dark:bg-zinc-900 shadow-xl p-8">
        <h1 className="text-2xl font-semibold text-zinc-900 dark:text-zinc-100 text-center mb-6">
          Вход
        </h1>
        <form onSubmit={handleSubmit} className="space-y-4">
          {error && (
            <p className="text-sm text-red-500 dark:text-red-400 text-center" role="alert">
              {error}
            </p>
          )}
          <div>
            <label htmlFor="login-username" className="block text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-1">
              Имя пользователя
            </label>
            <input
              id="login-username"
              type="text"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              autoComplete="username"
              className="w-full px-4 py-2 rounded-lg border border-zinc-300 dark:border-zinc-600 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-100 focus:ring-2 focus:ring-emerald-500 focus:border-transparent"
              placeholder="default"
            />
          </div>
          <div>
            <label htmlFor="login-password" className="block text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-1">
              Пароль
            </label>
            <input
              id="login-password"
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              autoComplete="current-password"
              className="w-full px-4 py-2 rounded-lg border border-zinc-300 dark:border-zinc-600 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-100 focus:ring-2 focus:ring-emerald-500 focus:border-transparent"
            />
          </div>
          <button
            type="submit"
            disabled={loading}
            className="w-full py-2.5 rounded-lg bg-emerald-600 hover:bg-emerald-700 disabled:opacity-50 text-white font-medium transition-colors"
          >
            {loading ? "Вход…" : "Войти"}
          </button>
        </form>
        <p className="mt-6 text-center text-sm text-zinc-500 dark:text-zinc-400">
          Нет аккаунта?{" "}
          <Link to="/register" className="text-emerald-600 dark:text-emerald-400 hover:underline">
            Регистрация
          </Link>
        </p>
        {users.length > 0 && (
          <p className="mt-2 text-xs text-zinc-400 dark:text-zinc-500 text-center">
            Пользователи: {users.map((u) => u.display_name || u.username).join(", ")}
          </p>
        )}
      </div>
    </div>
  );
}
