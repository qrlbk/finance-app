import { useState, useEffect } from "react";
import { Navigate, useLocation } from "react-router-dom";
import { api } from "../lib/api";

interface RequireAuthProps {
  children: React.ReactNode;
}

export function RequireAuth({ children }: RequireAuthProps) {
  const [session, setSession] = useState<unknown>(undefined);
  const location = useLocation();

  useEffect(() => {
    api.getCurrentSession().then(setSession).catch(() => setSession(null));
  }, []);

  if (session === undefined) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-zinc-100 dark:bg-zinc-950">
        <div className="text-zinc-500 dark:text-zinc-400">Загрузка…</div>
      </div>
    );
  }

  if (session === null) {
    return <Navigate to="/login" state={{ from: location }} replace />;
  }

  return <>{children}</>;
}
