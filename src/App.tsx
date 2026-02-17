import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import { Layout } from "./components/layout/Layout";
import { Dashboard } from "./pages/Dashboard";
import { Transactions } from "./pages/Transactions";
import { Accounts } from "./pages/Accounts";
import { Reports } from "./pages/Reports";
import { Settings } from "./pages/Settings";
import { Recurring } from "./pages/Recurring";
import { Categories } from "./pages/Categories";
import { InsightsPage } from "./pages/Insights";
import { Import } from "./pages/Import";
import { Transfers } from "./pages/Transfers";
import { Chat } from "./pages/Chat";
import { Login } from "./pages/Login";
import { Register } from "./pages/Register";
import { ToastProvider } from "./components/ui/Toast";
import { ErrorBoundary } from "./components/ErrorBoundary";
import { TauriGuard } from "./components/TauriGuard";
import { RequireAuth } from "./components/RequireAuth";

function App() {
  return (
    <TauriGuard>
      <ToastProvider>
        <BrowserRouter>
          <Routes>
            <Route path="/login" element={<Login />} />
            <Route path="/register" element={<Register />} />
            <Route
              path="/"
              element={
                <RequireAuth>
                  <Layout />
                </RequireAuth>
              }
            >
              <Route index element={<Dashboard />} />
              <Route path="transactions" element={<Transactions />} />
              <Route path="transfers" element={<Transfers />} />
              <Route path="accounts" element={<Accounts />} />
              <Route path="recurring" element={<Recurring />} />
              <Route path="categories" element={<Categories />} />
              <Route path="insights" element={<InsightsPage />} />
              <Route path="import" element={<Import />} />
              <Route path="reports" element={<Reports />} />
              <Route path="chat" element={<ErrorBoundary><Chat /></ErrorBoundary>} />
              <Route path="settings" element={<Settings />} />
            </Route>
            <Route path="*" element={<Navigate to="/" replace />} />
          </Routes>
        </BrowserRouter>
      </ToastProvider>
    </TauriGuard>
  );
}

export default App;
