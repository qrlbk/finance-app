<p align="center">
  <strong>Finance App</strong><br>
  Personal finance manager. Offline-first. Your data stays on your device.
</p>

<p align="center">
  <a href="README.ru.md">Русский</a> · <a href="README.kk.md">Қазақша</a>
</p>

---

## What is this?

**Finance App** is a desktop app for tracking money: accounts, transactions, budgets, and forecasts. It’s built with [Tauri](https://tauri.app/) (Rust) and [React](https://react.dev/), runs on **macOS**, **Windows**, and **Linux**, and works **fully offline**. Data is stored locally in an encrypted SQLite database (SQLCipher).

- **Privacy-first** — Nothing is sent to the cloud; everything stays on your machine.
- **Multi-language UI** — Kazakh, Russian, and English (switch in Settings).
- **Smart features** — ML category suggestions, expense forecasting, anomaly detection, optional Ollama chat.

---

## Features

| Area | What you get |
|------|----------------|
| **Accounts** | Cash, card, savings; multi-currency (KZT, USD, EUR, RUB) and exchange rates. |
| **Transactions** | Income & expenses with categories, notes, filters, and “Load more”. |
| **Transfers** | Move money between your accounts; full history. |
| **Budgets** | Limits per category (week / month / year) with alerts at 80% and 100%. |
| **Recurring** | Auto payments (daily/weekly/monthly/yearly); processed on startup or by button. |
| **Insights & reports** | Comparison with last month, forecast, savings tips, pie charts, trends. |
| **Import / export** | CSV, JSON, XLSX; bank PDF (e.g. Kaspi); backup & restore. |
| **App** | Dark/light theme, in-app Help (Docs) in three languages. |

---

## Quick start

**Requirements:** Node.js ≥ 18, Rust ≥ 1.75, [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS.

```bash
git clone https://github.com/qrlbk/finance-app.git
cd finance-app
npm install
npm run tauri dev
```

**Production build:**

```bash
npm run tauri build
```

Artifacts: **macOS** — `.app` in `src-tauri/target/release/bundle/macos/` · **Windows** — `.msi` in `src-tauri/target/release/bundle/msi/` · **Linux** — `.AppImage` in `src-tauri/target/release/bundle/appimage/`.

---

## Project layout

```
finance-app/
├── src/                 # React frontend (TypeScript, Tailwind, i18next)
│   ├── components/      # Layout, dashboard, UI
│   ├── pages/           # Dashboard, Transactions, Accounts, Reports, Settings, Docs, …
│   ├── lib/             # API client, format helpers, i18n
│   └── locales/         # kk.json, ru.json, en.json
├── src-tauri/           # Rust backend (Tauri 2, SQLite/SQLCipher, ML)
├── docs/                # Design docs (architecture, import, ML)
└── e2e/                 # Playwright E2E
```

See [ARCHITECTURE.md](./ARCHITECTURE.md) for details.

---

## Tests

```bash
npm run test          # Vitest (watch)
npm run test:run      # Vitest (single run)
npm run test:coverage # Coverage
npm run test:e2e      # Playwright E2E
cd src-tauri && cargo test
```

---

## Data & security

- **DB path:** macOS `~/Library/Application Support/com.kuralbekadilet475.finance-app/` · Windows `%APPDATA%\com.kuralbekadilet475.finance-app\` · Linux `~/.local/share/finance-app/`.
- **Backup:** Settings → Backup (create/restore).
- **Encryption:** SQLCipher (AES-256). No data leaves the device; ML runs locally.

---

## Tech stack

| Part | Stack |
|------|--------|
| Frontend | React 19, TypeScript, Vite 7, Tailwind CSS 4, React Router, Recharts, i18next |
| Backend | Tauri 2, Rust, rusqlite + SQLCipher |
| Tests | Vitest, Playwright, Rust tests |

---

## License & author

**MIT** · **Kuralbek Adilet**

Repository: [github.com/qrlbk/finance-app](https://github.com/qrlbk/finance-app)
