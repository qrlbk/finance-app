<div align="center">

# 💰 Finance App

**Учёт личных финансов · Офлайн · Данные хранятся на вашем устройстве**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-FFC131?logo=tauri&logoColor=black)](https://tauri.app/)
[![React](https://img.shields.io/badge/React-19-61DAFB?logo=react&logoColor=black)](https://react.dev/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5-3178C6?logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![Platform](https://img.shields.io/badge/Platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey)](https://github.com/qrlbk/finance-app)

[English](README.md) · [Русский](README.ru.md) · [Қазақша](README.kk.md)

</div>

---

## ✨ Что это?

**Finance App** — десктопное приложение для учёта денег: счета, транзакции, бюджеты и прогнозы. Собрано на [Tauri](https://tauri.app/) (Rust) и [React](https://react.dev/), работает на **macOS**, **Windows** и **Linux** и полностью **офлайн**. Данные хранятся локально в зашифрованной SQLite (SQLCipher).

<table>
<tr>
<td width="33%">

**🔒 Приватность**  
Ничего не отправляется в облако; всё остаётся на вашем устройстве.

</td>
<td width="33%">

**🌐 Мультиязычность**  
Казахский, русский и английский — переключение в Настройках.

</td>
<td width="33%">

**🧠 Умные функции**  
Подсказки категорий (ML), прогноз расходов, аномалии, опциональный чат на Ollama.

</td>
</tr>
</table>

---

## 🚀 Возможности

| | Раздел | Что есть |
|:---:|---|---|
| 💳 | **Счета** | Наличные, карта, накопления; мультивалютность (KZT, USD, EUR, RUB) и курсы. |
| 📊 | **Транзакции** | Доходы и расходы с категориями, заметками, фильтрами и подгрузкой. |
| 🔄 | **Переводы** | Движение денег между счетами; полная история. |
| 📈 | **Бюджеты** | Лимиты по категориям (неделя / месяц / год) и уведомления на 80% и 100%. |
| 🔁 | **Автоплатежи** | Повторяющиеся платежи (ежедневно/еженедельно/ежемесячно/ежегодно); проведение при старте или по кнопке. |
| 📉 | **Аналитика и отчёты** | Сравнение с прошлым месяцем, прогноз, советы по экономии, диаграммы, тренды. |
| 📁 | **Импорт / экспорт** | CSV, JSON, XLSX; банковский PDF (например Kaspi); резервная копия и восстановление. |
| ⚙️ | **Приложение** | Тёмная/светлая тема, встроенная Документация на трёх языках. |

---

## ⚡ Быстрый старт

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

| Платформа | Результат |
|-----------|-----------|
| **macOS** | `.app` → `src-tauri/target/release/bundle/macos/` |
| **Windows** | `.msi` → `src-tauri/target/release/bundle/msi/` |
| **Linux** | `.AppImage` → `src-tauri/target/release/bundle/appimage/` |

---

## 📁 Структура проекта

```
finance-app/
├── src/                 # React-фронтенд (TypeScript, Tailwind, i18next)
│   ├── components/      # Layout, дашборд, UI
│   ├── pages/           # Главная, Транзакции, Счета, Отчёты, Настройки, Документация
│   ├── lib/             # API-клиент, форматирование, i18n
│   └── locales/         # kk.json, ru.json, en.json
├── src-tauri/           # Rust-бэкенд (Tauri 2, SQLite/SQLCipher, ML)
├── docs/                # Документация (архитектура, импорт, ML)
└── e2e/                 # Playwright E2E
```

→ Подробнее: [ARCHITECTURE.md](./ARCHITECTURE.md).

---

## 🧪 Тесты

```bash
npm run test          # Vitest (watch)
npm run test:run      # Vitest (один прогон)
npm run test:coverage # Покрытие
npm run test:e2e      # Playwright E2E
cd src-tauri && cargo test
```

---

## 🔐 Данные и безопасность

- **Путь к БД:**  
  **macOS** `~/Library/Application Support/com.kuralbekadilet475.finance-app/`  
  **Windows** `%APPDATA%\com.kuralbekadilet475.finance-app\`  
  **Linux** `~/.local/share/finance-app/`
- **Резервная копия:** Настройки → Резервная копия (создать / восстановить).
- **Шифрование:** SQLCipher (AES-256). Данные не покидают устройство; ML выполняется локально.

---

## 🛠 Стек технологий

| Часть | Технологии |
|------|------------|
| **Frontend** | React 19 · TypeScript · Vite 7 · Tailwind CSS 4 · React Router · Recharts · i18next |
| **Backend** | Tauri 2 · Rust · rusqlite + SQLCipher |
| **Тесты** | Vitest · Playwright · тесты на Rust |

---

<div align="center">

**MIT** · **Kuralbek Adilet**  
[**github.com/qrlbk/finance-app**](https://github.com/qrlbk/finance-app)

</div>
