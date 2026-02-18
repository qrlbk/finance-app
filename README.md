# Finance App — Personal Finance Manager

A **desktop application** for tracking personal finances. It runs entirely **offline** with no server required; all data is stored in an encrypted SQLite database on your device.

---

## About

**Finance App** is a modern, cross-platform personal finance manager built with **Tauri 2** (Rust) and **React**. It combines high performance with strong data security and runs natively on macOS, Windows, and Linux.

### Highlights

- **100% local** — Your data never leaves your machine
- **Encrypted storage** — SQLCipher (AES-256) for the database
- **ML-powered** — Category suggestions, expense forecasting, anomaly detection
- **Multi-language** — Kazakh, Russian, and English

---

## Features

### Finance Management

| Feature | Description |
|--------|-------------|
| **Accounts** | Cash, card, and savings accounts with multi-currency support |
| **Transactions** | Income and expenses with categories and notes |
| **Transfers** | Move money between your own accounts |
| **Budgets** | Category limits with alerts at 80% and 100% |
| **Recurring payments** | Automatic creation of repeating transactions (daily/weekly/monthly/yearly) |

### Machine Learning

| Feature | Description |
|--------|-------------|
| **Category prediction** | Suggests categories from transaction descriptions |
| **Expense forecast** | Predicts next month’s spending with confidence range |
| **Anomaly detection** | Flags unusual spending patterns |
| **Smart insights** | Savings recommendations and spending patterns |

### Import & Export

| Feature | Description |
|--------|-------------|
| **Bank statements** | Import PDF statements (e.g. Kaspi Bank) |
| **Data export** | CSV, JSON, XLSX with optional filters |
| **Backup & restore** | Full database backup and restore |

### Interface

- **Dashboard** — Balance, income, expenses, budgets, and forecasts
- **Reports** — Category pie charts and monthly trends
- **Insights** — Analytics and recommendations
- **Dark / light theme** — With system preference support
- **Chat** — Optional AI assistant (Ollama) with access to your financial context

---

## Requirements

### To run from source

| Component | Version |
|-----------|--------|
| Node.js | ≥ 18 |
| Rust | ≥ 1.75 |
| Tauri CLI | v2.x |

### System dependencies

**macOS**

```bash
xcode-select --install
```

**Windows**

- Visual Studio Build Tools 2022
- WebView2 (usually preinstalled on Windows 10/11)

**Linux (Debian/Ubuntu)**

```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev \
    build-essential curl wget file \
    libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

**Linux (Fedora)**

```bash
sudo dnf install webkit2gtk4.1-devel openssl-devel curl wget file \
    libappindicator-gtk3-devel librsvg2-devel
```

---

## Installation & Run

### 1. Clone the repository

```bash
git clone https://github.com/yourusername/finance-app.git
cd finance-app
```

### 2. Install dependencies

```bash
# Install Tauri CLI (if needed)
cargo install tauri-cli

# Install npm dependencies
npm install
```

### 3. Development mode

```bash
npm run tauri dev
```

The app starts with hot-reload for the frontend.

### 4. Production build

```bash
npm run tauri build
```

Output locations:

- **macOS:** `src-tauri/target/release/bundle/macos/finance-app.app`
- **Windows:** `src-tauri/target/release/bundle/msi/finance-app_*_x64_en-US.msi`
- **Linux:** `src-tauri/target/release/bundle/appimage/finance-app_*_amd64.AppImage`

---

## Project structure

```
finance-app/
├── src/                    # Frontend (React + TypeScript)
│   ├── components/         # UI components (layout, dashboard, ui)
│   ├── pages/              # App pages (Dashboard, Transactions, etc.)
│   ├── hooks/              # React hooks
│   ├── lib/                # API client, formatting, i18n
│   ├── locales/            # Translations (kk, ru, en)
│   └── stores/             # Theme and local state
├── src-tauri/              # Backend (Rust + Tauri)
│   └── src/
│       ├── commands.rs     # Tauri commands (API)
│       ├── db/             # Database (schema, queries)
│       ├── ml/             # ML (tokenizer, TF-IDF, classifier, forecast)
│       ├── bank_import/    # Bank statement import
│       └── export.rs       # Data export
├── docs/                   # Additional documentation
└── e2e/                    # E2E tests (Playwright)
```

See **[ARCHITECTURE.md](./ARCHITECTURE.md)** for a full technical overview.

---

## Testing

### Frontend (Vitest)

```bash
npm run test          # Watch mode
npm run test:run      # Single run
npm run test:coverage # With coverage
```

### E2E (Playwright)

```bash
npm run test:e2e      # Run E2E tests
npm run test:e2e:ui   # With interactive UI
```

### Backend (Rust)

```bash
cd src-tauri
cargo test
cargo test -- --nocapture
cargo test db::queries::tests
cargo test ml::forecast::tests
```

---

## Data storage

### Database location

| OS | Path |
|----|------|
| macOS | `~/Library/Application Support/com.kuralbekadilet475.finance-app/finance.db` |
| Windows | `%APPDATA%\com.kuralbekadilet475.finance-app\finance.db` |
| Linux | `~/.local/share/finance-app/finance.db` |

### Backup

Create a backup from **Settings → Backup**, or copy the database file manually.

---

## Security

- **Database encryption** — SQLCipher (AES-256-CBC, PBKDF2-HMAC-SHA256).
- **Local only** — No data is sent to external servers.
- **ML on device** — All ML models run locally.

---

## Tech stack

| Layer | Technologies |
|-------|--------------|
| **Frontend** | React 19, TypeScript, Tailwind CSS 4, Vite 7, React Router, Recharts, i18next |
| **Backend** | Tauri 2, Rust, rusqlite + SQLCipher |
| **Testing** | Vitest, Playwright, Rust tests |

---

## License

MIT License

---

## Author

Kuralbek Adilet
