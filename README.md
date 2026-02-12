# Личная бухгалтерия (Finance App)

Десктопное приложение для учёта личных финансов. Работает полностью локально без сервера, данные хранятся в зашифрованной SQLite базе данных.

## О приложении

Finance App — это современное кроссплатформенное приложение для управления личными финансами. Построено на базе Tauri 2 (Rust) и React, обеспечивает высокую производительность и безопасность данных.

### Ключевые особенности

- Полностью локальное хранение данных
- Шифрование БД (SQLCipher)
- ML-функции для автоматизации
- Кроссплатформенность (macOS, Windows, Linux)

## Основные возможности

### Управление финансами
- **Счета** — карта, наличные, накопления с поддержкой разных валют
- **Транзакции** — доходы и расходы с категоризацией
- **Переводы** — между собственными счетами
- **Бюджеты** — лимиты по категориям с уведомлениями при превышении
- **Регулярные платежи** — автоматическое создание повторяющихся транзакций

### ML-функции
- **Предсказание категорий** — автоматическое определение категории по описанию транзакции
- **Прогноз расходов** — предсказание расходов на следующий месяц
- **Обнаружение аномалий** — выявление необычных трат
- **Smart Insights** — умные рекомендации по экономии

### Импорт/Экспорт
- **Импорт выписок** — поддержка PDF выписок Kaspi Bank
- **Экспорт данных** — CSV, JSON, XLSX форматы
- **Резервное копирование** — полный бэкап и восстановление БД

### Интерфейс
- **Dashboard** — сводка по балансу, доходам и расходам
- **Отчёты** — диаграммы по категориям и динамика по месяцам
- **Тёмная/светлая тема** — автоматическое переключение

## Требования

### Для запуска из исходников

| Компонент | Версия |
|-----------|--------|
| Node.js | >= 18.0 |
| Rust | >= 1.75 |
| Tauri CLI | v2.x |

### Системные зависимости

**macOS:**
```bash
xcode-select --install
```

**Windows:**
- Visual Studio Build Tools 2022
- WebView2 (обычно предустановлен в Windows 10/11)

**Linux (Debian/Ubuntu):**
```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
```

**Linux (Fedora):**
```bash
sudo dnf install webkit2gtk4.1-devel \
    openssl-devel \
    curl \
    wget \
    file \
    libappindicator-gtk3-devel \
    librsvg2-devel
```

## Установка и запуск

### 1. Клонирование репозитория

```bash
git clone https://github.com/yourusername/finance-app.git
cd finance-app
```

### 2. Установка зависимостей

```bash
# Установка Tauri CLI (если не установлен)
cargo install tauri-cli

# Установка npm зависимостей
npm install
```

### 3. Режим разработки

```bash
npm run tauri dev
```

Приложение запустится с hot-reload для frontend части.

### 4. Сборка для production

```bash
npm run tauri build
```

Собранные приложения находятся в:
- **macOS:** `src-tauri/target/release/bundle/macos/finance-app.app`
- **Windows:** `src-tauri/target/release/bundle/msi/finance-app_x.x.x_x64_en-US.msi`
- **Linux:** `src-tauri/target/release/bundle/appimage/finance-app_x.x.x_amd64.AppImage`

## Архитектура

```
finance-app/
├── src/                    # Frontend (React + TypeScript)
│   ├── components/         # UI компоненты
│   │   ├── dashboard/      # Компоненты Dashboard
│   │   ├── layout/         # Layout компоненты (Header, Sidebar)
│   │   └── ui/             # Переиспользуемые UI элементы
│   ├── pages/              # Страницы приложения
│   ├── hooks/              # React hooks
│   ├── stores/             # Zustand stores
│   ├── lib/                # Утилиты и API клиент
│   └── test/               # Тестовая конфигурация
├── src-tauri/              # Backend (Rust + Tauri)
│   ├── src/
│   │   ├── commands.rs     # Tauri команды (API)
│   │   ├── db/             # Модуль базы данных
│   │   │   ├── mod.rs      # Подключение к БД
│   │   │   ├── schema.rs   # Схема и миграции
│   │   │   └── queries.rs  # SQL запросы
│   │   ├── ml/             # ML модуль
│   │   │   ├── model.rs    # Модель категоризации
│   │   │   ├── trainer.rs  # Обучение модели
│   │   │   ├── forecast.rs # Прогнозирование
│   │   │   └── anomaly.rs  # Обнаружение аномалий
│   │   ├── bank_import/    # Импорт банковских выписок
│   │   ├── export.rs       # Экспорт данных
│   │   ├── crypto.rs       # Шифрование
│   │   └── security.rs     # Валидация и безопасность
│   └── Cargo.toml          # Rust зависимости
├── e2e/                    # E2E тесты (Playwright)
└── package.json            # npm конфигурация
```

### Модульная диаграмма

```
┌─────────────────────────────────────────────────────────┐
│                    Frontend (React)                      │
│  ┌─────────┐  ┌──────────┐  ┌─────────┐  ┌──────────┐  │
│  │  Pages  │  │Components│  │  Hooks  │  │  Stores  │  │
│  └────┬────┘  └────┬─────┘  └────┬────┘  └────┬─────┘  │
│       └────────────┴─────────────┴────────────┘        │
│                          │                              │
│                    ┌─────┴─────┐                        │
│                    │  API (lib)│                        │
│                    └─────┬─────┘                        │
└──────────────────────────┼──────────────────────────────┘
                           │ Tauri IPC
┌──────────────────────────┼──────────────────────────────┐
│                    Backend (Rust)                        │
│                    ┌─────┴─────┐                        │
│                    │ Commands  │                        │
│                    └─────┬─────┘                        │
│       ┌──────────────────┼──────────────────┐          │
│  ┌────┴────┐       ┌─────┴─────┐      ┌─────┴─────┐    │
│  │   DB    │       │    ML     │      │  Export   │    │
│  │ Module  │       │  Module   │      │  Module   │    │
│  └────┬────┘       └───────────┘      └───────────┘    │
│       │                                                 │
│  ┌────┴────┐                                           │
│  │ SQLite  │                                           │
│  │(Cipher) │                                           │
│  └─────────┘                                           │
└─────────────────────────────────────────────────────────┘
```

## Тестирование

### Frontend тесты (Vitest)

```bash
# Запуск в watch режиме
npm run test

# Однократный запуск
npm run test:run

# С покрытием кода
npm run test:coverage
```

### E2E тесты (Playwright)

```bash
# Запуск E2E тестов
npm run test:e2e

# С интерактивным UI
npm run test:e2e:ui
```

### Backend тесты (Rust)

```bash
cd src-tauri

# Запуск всех тестов
cargo test

# С выводом println
cargo test -- --nocapture

# Конкретный модуль
cargo test db::queries::tests
cargo test ml::forecast::tests
```

## Хранение данных

### Расположение базы данных

| ОС | Путь |
|----|------|
| macOS | `~/Library/Application Support/com.kuralbekadilet475.finance-app/finance.db` |
| Windows | `%APPDATA%/com.kuralbekadilet475.finance-app/finance.db` |
| Linux | `~/.local/share/finance-app/finance.db` |

### Резервное копирование

Бэкап можно создать через приложение (Настройки → Резервная копия) или скопировав файл базы данных.

## Безопасность

### Шифрование базы данных

База данных шифруется с помощью SQLCipher:
- Алгоритм: AES-256-CBC
- KDF: PBKDF2-HMAC-SHA256 (256000 итераций)
- Ключ хранится локально в защищённом файле

### Локальное хранение

- Все данные хранятся только на вашем устройстве
- Никакие данные не передаются на внешние сервера
- ML модели работают локально

## Технологический стек

### Frontend
- **React 19** — UI библиотека
- **TypeScript** — типизация
- **Tailwind CSS 4** — стилизация
- **Recharts** — графики и диаграммы
- **Lucide React** — иконки
- **React Router** — маршрутизация

### Backend
- **Tauri 2** — фреймворк десктопных приложений
- **Rust** — язык программирования
- **rusqlite + SQLCipher** — база данных с шифрованием
- **chrono** — работа с датами
- **serde** — сериализация

### Тестирование
- **Vitest** — unit тесты frontend
- **Playwright** — E2E тесты
- **Rust test** — unit тесты backend

## Лицензия

MIT License

## Автор

Kuralbek Adilet
