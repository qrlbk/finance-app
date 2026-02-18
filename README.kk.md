<p align="center">
  <strong>Finance App</strong><br>
  Жеке қаржыны басқару. Офлайн жұмыс. Деректер сіздің құрылғыңызда сақталады.
</p>

<p align="center">
  <a href="README.md">English</a> · <a href="README.ru.md">Русский</a>
</p>

---

## Бұл не?

**Finance App** — ақшаны есепке алуға арналған десктоп қолданба: шоттар, транзакциялар, бюджеттер және болжамдар. [Tauri](https://tauri.app/) (Rust) және [React](https://react.dev/) негізінде жасалған, **macOS**, **Windows** және **Linux**-та жұмыс істейді және **толық офлайн** режимінде. Деректер шифрланған SQLite (SQLCipher) деректер қорында жергілікті сақталады.

- **Құпиялылық** — ештеңе бұлтқа жіберілмейді; бәрі құрылғыңызда қалады.
- **Көп тілді интерфейс** — қазақша, орысша және ағылшынша (Баптауларда ауыстыру).
- **Ақылды мүмкіндіктер** — санат ұсыныстары (ML), шығын болжамы, аномалиялар, қосымша Ollama чаты.

---

## Мүмкіндіктер

| Бөлім | Не бар |
|--------|--------|
| **Шоттар** | Қолма-қол, карта, жинақ; көп валюта (KZT, USD, EUR, RUB) және айырбас курсоры. |
| **Транзакциялар** | Кіріс пен шығын санаттар, жазбалар, сүзгілер және тағы жүктеу. |
| **Аударымдар** | Шоттар аралық ақша қозғалысы; толық тарих. |
| **Бюджеттер** | Санат бойынша лимиттер (апта / ай / жыл) және 80% және 100%-та хабарламалар. |
| **Қайталанатын төлемдер** | Автотөлемдер (күнделікті/апталық/айлық/жылдық); іске қосқанда немесе батырма арқылы өңдеу. |
| **Аналитика және есептер** | Өткен аймен салыстыру, болжам, үнемдеу кеңестері, диаграммалар, трендтер. |
| **Импорт / экспорт** | CSV, JSON, XLSX; банк PDF (мысалы Kaspi); резервтік көшірме және қалпына келтіру. |
| **Қолданба** | Қараңғы/ашық тақырып, үш тілде кірістірілген көмек (Құжаттама). |

---

## Жылдам бастау

**Қажет:** Node.js ≥ 18, Rust ≥ 1.75, [Tauri алғышарттары](https://v2.tauri.app/start/prerequisites/) ОС бойынша.

```bash
git clone https://github.com/qrlbk/finance-app.git
cd finance-app
npm install
npm run tauri dev
```

**Production құрастыру:**

```bash
npm run tauri build
```

Нәтиже: **macOS** — `.app` в `src-tauri/target/release/bundle/macos/` · **Windows** — `.msi` в `src-tauri/target/release/bundle/msi/` · **Linux** — `.AppImage` в `src-tauri/target/release/bundle/appimage/`.

---

## Жоба құрылымы

```
finance-app/
├── src/                 # React фронтенд (TypeScript, Tailwind, i18next)
│   ├── components/      # Layout, дашборд, UI
│   ├── pages/           # Басты бет, Транзакциялар, Шоттар, Есептер, Баптаулар, Құжаттама, …
│   ├── lib/             # API клиенті, форматтау, i18n
│   └── locales/         # kk.json, ru.json, en.json
├── src-tauri/           # Rust бэкенд (Tauri 2, SQLite/SQLCipher, ML)
├── docs/                # Құжаттама (архитектура, импорт, ML)
└── e2e/                 # Playwright E2E
```

Толығырақ: [ARCHITECTURE.md](./ARCHITECTURE.md).

---

## Тесттер

```bash
npm run test          # Vitest (watch)
npm run test:run      # Vitest (бір прогон)
npm run test:coverage # Қамту
npm run test:e2e      # Playwright E2E
cd src-tauri && cargo test
```

---

## Деректер және қауіпсіздік

- **Деректер қоры жолы:** macOS `~/Library/Application Support/com.kuralbekadilet475.finance-app/` · Windows `%APPDATA%\com.kuralbekadilet475.finance-app\` · Linux `~/.local/share/finance-app/`.
- **Резервтік көшірме:** Баптаулар → Резервтік көшірме (жасау/қалпына келтіру).
- **Шифрлау:** SQLCipher (AES-256). Деректер құрылғыдан шықпайды; ML жергілікті орындалады.

---

## Технологиялық стек

| Бөлім | Технологиялар |
|-------|----------------|
| Frontend | React 19, TypeScript, Vite 7, Tailwind CSS 4, React Router, Recharts, i18next |
| Backend | Tauri 2, Rust, rusqlite + SQLCipher |
| Тесттер | Vitest, Playwright, Rust тесттері |

---

## Лицензия және автор

**MIT** · **Kuralbek Adilet**

Репозиторий: [github.com/qrlbk/finance-app](https://github.com/qrlbk/finance-app)
