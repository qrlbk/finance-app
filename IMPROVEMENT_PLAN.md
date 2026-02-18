# Finance App — Improvement Plan

A structured plan of improvements and future work, based on codebase analysis (Rust/Tauri backend, React frontend, SQLite).

---

## 1. Critical logic gaps (addressed)

### 1.1 Recurring payments auto-run

**Previously:** `process_recurring_payments` ran only when the user clicked “Process” on the Recurring page.

**Resolution:** On app startup (e.g. in Layout or backend init), `process_due_recurring` is called and the count of created transactions can be shown (e.g. “Processed N recurring payments”).

---

### 1.2 Budget period (weekly/yearly)

**Previously:** `get_budgets_with_spending` always used the current **month** for `spent`, regardless of `budget.period` (weekly/monthly/yearly).

**Resolution:** Spending window is computed per budget from `period`: current week for weekly, current month for monthly, current year for yearly.

---

### 1.3 Budget alerts in UI

**Previously:** `getBudgetAlerts()` existed but was not used in the UI.

**Resolution:** Header (or Layout) loads budget alerts and shows an icon with a dropdown (e.g. bell) listing exceeded or near-limit budgets. Dashboard can show the same block.

---

### 1.4 Multi-currency in totals

**Previously:** `get_summary` summed raw balances; different account currencies were mixed.

**Resolution:** Base currency and exchange rates in settings; `get_summary` converts balances to base currency. UI shows a warning when multiple currencies exist without conversion. Reports and dashboard use base currency.

---

## 2. UX and feature completeness (addressed)

### 2.1 Transfers page

- Route `/transfers` and a dedicated Transfers page with history.
- Sidebar entry “Transfers.”
- Dashboard button “Transfer” next to Add income/expense.

### 2.2 Page titles in Layout

- All routes (e.g. `/recurring`, `/categories`, `/insights`, `/import`) have entries in `pageTitles` so the window title matches the current section.

### 2.3 Account deletion with transaction reassignment

- When deleting an account that has transactions, the UI offers “Reassign transactions to another account” and a account selector.
- Backend: e.g. `reassign_transactions_to_account(from_id, to_id)` (UPDATE + balance recalc), then allow delete.

### 2.4 Category hierarchy (parent_id)

- Categories page shows a tree (parent/children). When creating a category, optional parent selection.
- Reports and filters: option “Include subcategories” when aggregating by category.

---

## 3. Export and import

### 3.1 Export filters

- Export options include optional `account_id` and `category_id` so users can export a single account or category.

### 3.2 Import XLSX

- Support XLSX in addition to CSV/JSON (e.g. calamine in Rust), with a documented column layout (date, amount, type, account, category, note).

### 3.3 Backup restore (SQLCipher)

- After restore, verify the DB opens with the current SQLCipher key; on failure, rollback and show a clear message. Warn user: “Ensure the backup was created by this app; otherwise data may not open.”

---

## 4. Quality and reliability

### 4.1 Transaction pagination

- API: `limit` + `offset` (or cursor). UI: load 50–100 per page with “Load more” or infinite scroll.

### 4.2 Consistent error messages

- All user-facing messages in one language (or keys for i18n). Backend and frontend use the same set of message keys/constants.

### 4.3 Import duplicate detection

- Stricter duplicate key: e.g. exact match on date + rounded amount + normalized note. Option to “Import anyway” for rows marked as duplicates. Optional hash storage for future checks.

### 4.4 E2E coverage

- E2E scenarios: create account → add income/expense → transfer → check dashboard totals; optional: import sample PDF, export, restore backup on a test DB.

---

## 5. ML and analytics

### 5.1 Train after import

- After a successful import (bank statement or CSV/JSON/XLSX), prompt: “Train model on new data?” or run training in background and notify when done.

### 5.2 Forecast and reports by currency

- Once multi-currency and conversion exist, forecasts and category reports should use base currency (or explicit currency) so amounts are comparable.

---

## 6. Prioritization summary

| Priority | Task |
|----------|------|
| **P0** | Auto-run recurring on startup; budget period (weekly/yearly); show budget alerts in UI |
| **P1** | Multi-currency (base + rates); Transfers page; page titles |
| **P2** | Reassign transactions on account delete; category tree in UI; export filters; XLSX import; transaction pagination |
| **P3** | Unified error language; duplicate rules; E2E; train-after-import; backup restore checks |

---

## 7. Current implementation status

Most P0–P2 items above are **already implemented** in the codebase:

- **P0:** Recurring runs on startup; budget period in `get_budgets_with_spending`; budget alerts in Header.
- **P1:** Base currency, `exchange_rates` table, conversion in `get_summary`; Transfers page; page titles; multi-currency warning.
- **P2:** Reassign transactions; category hierarchy in UI; export filters; XLSX import; pagination (“Load more”).
- **P3:** Centralized error messages (e.g. `messages.rs`); backup restore verification; “Train model?” after import in Settings and Import page.

**Planned for later:** multi-profile, richer charts, notifications.

---

## 8. What is already in good shape

- Backend validation (accounts, transactions, recurring, budgets).
- Category/account caching with invalidation.
- ML: category prediction, forecast, anomalies, Smart Insights.
- Export CSV/JSON/XLSX; import CSV/JSON/XLSX; bank statement import (e.g. Kaspi) with duplicate check and category suggestion.
- Recurring: frequencies, end_date, is_active, process on button and on startup.
- Budgets: spent and percent by period; alerts on backend.
- Tests: commands, schema, migrations.
- UI: theme, toasts, confirm dialogs, empty states, filters, ML category suggestion on transaction form, i18n (Kazakh, Russian, English).

Use this plan as a checklist: P0 first, then P1–P2 as needed, P3 when time allows.
