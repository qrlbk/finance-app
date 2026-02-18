<p align="center">
  <strong>Finance App</strong><br>
  Учёт личных финансов. Работа офлайн. Данные хранятся на вашем устройстве.
</p>

<p align="center">
  <a href="README.md">English</a> · <a href="README.kk.md">Қазақша</a>
</p>

---

## Что это?

**Finance App** — десктопное приложение для учёта денег: счета, транзакции, бюджеты и прогнозы. Собрано на [Tauri](https://tauri.app/) (Rust) и [React](https://react.dev/), работает на **macOS**, **Windows** и **Linux** и полностью **офлайн**. Данные хранятся локально в зашифрованной SQLite (SQLCipher).

- **Приватность** — ничего не отправляется в облако; всё остаётся на вашем устройстве.
- **Мультиязычный интерфейс** — казахский, русский и английский (переключение в Настройках).
- **Умные функции** — подсказки категорий (ML), прогноз расходов, аномалии, опциональный чат на Ollama.

---

## Возможности

| Раздел | Что есть |
|--------|----------|
| **Счета** | Наличные, карта, накопления; мультивалютность (KZT, USD, EUR, RUB) и курсы. |
| **Транзакции** | Доходы и расходы с категориями, заметками, фильтрами и подгрузкой. |
| **Переводы** | Движение денег между счетами; полная история. |
| **Бюджеты** | Лимиты по категориям (неделя / месяц / год) и уведомления на 80% и 100%. |
| **Автоплатежи** | Повторяющиеся платежи (ежедневно/еженедельно/ежемесячно/ежегодно); проведение при старте или по кнопке. |
| **Аналитика и отчёты** | Сравнение с прошлым месяцем, прогноз, советы по экономии, диаграммы, тренды. |
| **Импорт / экспорт** | CSV, JSON, XLSX; банковский PDF (например Kaspi); резервная копия и восстановление. |
| **Приложение** | Тёмная/светлая тема, встроенная справка (Документация) на трёх языках. |

---

## Быстрый старт

**Нужно:** Node.js ≥ 18, Rust ≥ 1.75, [предусловия Tauri](https://v2.tauri.app/start/prerequisites/) для вашей ОС.

```bash
git clone https://github.com/qrlbk/finance-app.git
cd finance-app
npm install
npm run tauri dev
```

**Сборка для production:**

```bash
npm run tauri build
```

Результат: **macOS** — `.app` в `src-tauri/target/release/bundle/macos/` · **Windows** — `.msi` в `src-tauri/target/release/bundle/msi/` · **Linux** — `.AppImage` в `src-tauri/target/release/bundle/appimage/`.

---

## Структура проекта

```
finance-app/
├── src/                 # React-фронтенд (TypeScript, Tailwind, i18next)
│   ├── components/      # Layout, дашборд, UI
│   ├── pages/           # Главная, Транзакции, Счета, Отчёты, Настройки, Документация, …
│   ├── lib/             # API-клиент, форматирование, i18n
│   └── locales/         # kk.json, ru.json, en.json
├── src-tauri/           # Rust-бэкенд (Tauri 2, SQLite/SQLCipher, ML)
├── docs/                # Документация (архитектура, импорт, ML)
└── e2e/                 # Playwright E2E
```

Подробнее: [ARCHITECTURE.md](./ARCHITECTURE.md).

---

## Тесты

```bash
npm run test          # Vitest (watch)
npm run test:run      # Vitest (один прогон)
npm run test:coverage # Покрытие
npm run test:e2e      # Playwright E2E
cd src-tauri && cargo test
```

---

## Данные и безопасность

- **Путь к БД:** macOS `~/Library/Application Support/com.kuralbekadilet475.finance-app/` · Windows `%APPDATA%\com.kuralbekadilet475.finance-app\` · Linux `~/.local/share/finance-app/`.
- **Резервная копия:** Настройки → Резервная копия (создать/восстановить).
- **Шифрование:** SQLCipher (AES-256). Данные не покидают устройство; ML выполняется локально.

---

## Стек технологий

| Часть | Технологии |
|-------|------------|
| Frontend | React 19, TypeScript, Vite 7, Tailwind CSS 4, React Router, Recharts, i18next |
| Backend | Tauri 2, Rust, rusqlite + SQLCipher |
| Тесты | Vitest, Playwright, тесты на Rust |

---

## Лицензия и автор

**MIT** · **Kuralbek Adilet**

Репозиторий: [github.com/qrlbk/finance-app](https://github.com/qrlbk/finance-app)
