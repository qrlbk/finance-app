import { BrowserRouter, Routes, Route } from "react-router-dom";
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
import { ToastProvider } from "./components/ui/Toast";

function App() {
  return (
    <ToastProvider>
      <BrowserRouter>
        <Routes>
          <Route path="/" element={<Layout />}>
            <Route index element={<Dashboard />} />
            <Route path="transactions" element={<Transactions />} />
            <Route path="accounts" element={<Accounts />} />
            <Route path="recurring" element={<Recurring />} />
            <Route path="categories" element={<Categories />} />
            <Route path="insights" element={<InsightsPage />} />
            <Route path="import" element={<Import />} />
            <Route path="reports" element={<Reports />} />
            <Route path="settings" element={<Settings />} />
          </Route>
        </Routes>
      </BrowserRouter>
    </ToastProvider>
  );
}

export default App;
