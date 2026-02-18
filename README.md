<div align="center">

# 💰 Finance App

**Personal finance manager · Offline-first · Your data stays on your device**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-FFC131?logo=tauri&logoColor=black)](https://tauri.app/)
[![React](https://img.shields.io/badge/React-19-61DAFB?logo=react&logoColor=black)](https://react.dev/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5-3178C6?logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![Platform](https://img.shields.io/badge/Platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey)](https://github.com/qrlbk/finance-app)

[English](README.md) · [Русский](README.ru.md) · [Қазақша](README.kk.md)

</div>

---

## ✨ What is this?

**Finance App** is a desktop app for tracking money: accounts, transactions, budgets, and forecasts. Built with [Tauri](https://tauri.app/) (Rust) and [React](https://react.dev/), it runs on **macOS**, **Windows**, and **Linux** and works **fully offline**. Data is stored locally in an encrypted SQLite database (SQLCipher).

<table>
<tr>
<td width="33%">

**🔒 Privacy-first**  
Nothing is sent to the cloud; everything stays on your machine.

</td>
<td width="33%">

**🌐 Multi-language**  
Kazakh, Russian, and English — switch in Settings.

</td>
<td width="33%">

**🧠 Smart features**  
ML category suggestions, forecasting, anomaly detection, optional Ollama chat.

</td>
</tr>
</table>

---

## 🚀 Features

| | Area | What you get |
|:---:|---|---|
| 💳 | **Accounts** | Cash, card, savings; multi-currency (KZT, USD, EUR, RUB) and exchange rates. |
| 📊 | **Transactions** | Income & expenses with categories, notes, filters, and “Load more”. |
| 🔄 | **Transfers** | Move money between accounts; full history. |
| 📈 | **Budgets** | Limits per category (week / month / year) with alerts at 80% and 100%. |
| 🔁 | **Recurring** | Auto payments (daily/weekly/monthly/yearly); run on startup or by button. |
| 📉 | **Insights & reports** | Compare with last month, forecast, savings tips, charts, trends. |
| 📁 | **Import / export** | CSV, JSON, XLSX; bank PDF (e.g. Kaspi); backup & restore. |
| ⚙️ | **App** | Dark/light theme, in-app Docs in three languages. |

---

## ⚡ Quick start

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

| Platform | Output |
|----------|--------|
| **macOS** | `.app` → `src-tauri/target/release/bundle/macos/` |
| **Windows** | `.msi` → `src-tauri/target/release/bundle/msi/` |
| **Linux** | `.AppImage` → `src-tauri/target/release/bundle/appimage/` |

---

## 📁 Project layout

```
finance-app/
├── src/                 # React frontend (TypeScript, Tailwind, i18next)
│   ├── components/      # Layout, dashboard, UI
│   ├── pages/           # Dashboard, Transactions, Accounts, Reports, Settings, Docs
│   ├── lib/             # API client, format helpers, i18n
│   └── locales/         # kk.json, ru.json, en.json
├── src-tauri/           # Rust backend (Tauri 2, SQLite/SQLCipher, ML)
├── docs/                # Design docs (architecture, import, ML)
└── e2e/                 # Playwright E2E
```

→ See [ARCHITECTURE.md](./ARCHITECTURE.md) for details.

---

## 🧪 Tests

```bash
npm run test          # Vitest (watch)
npm run test:run      # Vitest (single run)
npm run test:coverage # Coverage
npm run test:e2e      # Playwright E2E
cd src-tauri && cargo test
```

---

## 🔐 Data & security

- **DB path:**  
  **macOS** `~/Library/Application Support/com.kuralbekadilet475.finance-app/`  
  **Windows** `%APPDATA%\com.kuralbekadilet475.finance-app\`  
  **Linux** `~/.local/share/finance-app/`
- **Backup:** Settings → Backup (create / restore).
- **Encryption:** SQLCipher (AES-256). No data leaves the device; ML runs locally.

---

## 🛠 Tech stack

| Part | Stack |
|------|--------|
| **Frontend** | React 19 · TypeScript · Vite 7 · Tailwind CSS 4 · React Router · Recharts · i18next |
| **Backend** | Tauri 2 · Rust · rusqlite + SQLCipher |
| **Tests** | Vitest · Playwright · Rust tests |

---

<div align="center">

**MIT** · **Kuralbek Adilet**  
[**github.com/qrlbk/finance-app**](https://github.com/qrlbk/finance-app)

</div>
