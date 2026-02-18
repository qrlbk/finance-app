<div align="center">

# 💰 Finance App

**Жеке қаржыны басқару · Офлайн · Деректер сіздің құрылғыңызда сақталады**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-FFC131?logo=tauri&logoColor=black)](https://tauri.app/)
[![React](https://img.shields.io/badge/React-19-61DAFB?logo=react&logoColor=black)](https://react.dev/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5-3178C6?logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![Platform](https://img.shields.io/badge/Platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey)](https://github.com/qrlbk/finance-app)

[English](README.md) · [Русский](README.ru.md) · [Қазақша](README.kk.md)

</div>

---

## ✨ Бұл не?

**Finance App** — ақшаны есепке алуға арналған десктоп қолданба: шоттар, транзакциялар, бюджеттер және болжамдар. [Tauri](https://tauri.app/) (Rust) және [React](https://react.dev/) негізінде жасалған, **macOS**, **Windows** және **Linux**-та жұмыс істейді және **толық офлайн** режимінде. Деректер шифрланған SQLite (SQLCipher) деректер қорында жергілікті сақталады.

<table>
<tr>
<td width="33%">

**🔒 Құпиялылық**  
Ештеңе бұлтқа жіберілмейді; бәрі құрылғыңызда қалады.

</td>
<td width="33%">

**🌐 Көп тілділік**  
Қазақша, орысша және ағылшынша — Баптауларда ауыстыру.

</td>
<td width="33%">

**🧠 Ақылды мүмкіндіктер**  
Санат ұсыныстары (ML), шығын болжамы, аномалиялар, қосымша Ollama чаты.

</td>
</tr>
</table>

---

## 🚀 Мүмкіндіктер

| | Бөлім | Не бар |
|:---:|---|---|
| 💳 | **Шоттар** | Қолма-қол, карта, жинақ; көп валюта (KZT, USD, EUR, RUB) және айырбас курсоры. |
| 📊 | **Транзакциялар** | Кіріс пен шығын санаттар, жазбалар, сүзгілер және тағы жүктеу. |
| 🔄 | **Аударымдар** | Шоттар аралық ақша қозғалысы; толық тарих. |
| 📈 | **Бюджеттер** | Санат бойынша лимиттер (апта / ай / жыл) және 80% және 100%-та хабарламалар. |
| 🔁 | **Қайталанатын төлемдер** | Автотөлемдер (күнделікті/апталық/айлық/жылдық); іске қосқанда немесе батырма арқылы өңдеу. |
| 📉 | **Аналитика және есептер** | Өткен аймен салыстыру, болжам, үнемдеу кеңестері, диаграммалар, трендтер. |
| 📁 | **Импорт / экспорт** | CSV, JSON, XLSX; банк PDF (мысалы Kaspi); резервтік көшірме және қалпына келтіру. |
| ⚙️ | **Қолданба** | Қараңғы/ашық тақырып, үш тілде кірістірілген Құжаттама. |

---

## ⚡ Жылдам бастау

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

| Платформа | Нәтиже |
|-----------|--------|
| **macOS** | `.app` → `src-tauri/target/release/bundle/macos/` |
| **Windows** | `.msi` → `src-tauri/target/release/bundle/msi/` |
| **Linux** | `.AppImage` → `src-tauri/target/release/bundle/appimage/` |

---

## 📁 Жоба құрылымы

```
finance-app/
├── src/                 # React фронтенд (TypeScript, Tailwind, i18next)
│   ├── components/      # Layout, дашборд, UI
│   ├── pages/           # Басты бет, Транзакциялар, Шоттар, Есептер, Баптаулар, Құжаттама
│   ├── lib/             # API клиенті, форматтау, i18n
│   └── locales/         # kk.json, ru.json, en.json
├── src-tauri/           # Rust бэкенд (Tauri 2, SQLite/SQLCipher, ML)
├── docs/                # Құжаттама (архитектура, импорт, ML)
└── e2e/                 # Playwright E2E
```

→ Толығырақ: [ARCHITECTURE.md](./ARCHITECTURE.md).

---

## 🧪 Тесттер

```bash
npm run test          # Vitest (watch)
npm run test:run      # Vitest (бір прогон)
npm run test:coverage # Қамту
npm run test:e2e      # Playwright E2E
cd src-tauri && cargo test
```

---

## 🔐 Деректер және қауіпсіздік

- **Деректер қоры жолы:**  
  **macOS** `~/Library/Application Support/com.kuralbekadilet475.finance-app/`  
  **Windows** `%APPDATA%\com.kuralbekadilet475.finance-app\`  
  **Linux** `~/.local/share/finance-app/`
- **Резервтік көшірме:** Баптаулар → Резервтік көшірме (жасау / қалпына келтіру).
- **Шифрлау:** SQLCipher (AES-256). Деректер құрылғыдан шықпайды; ML жергілікті орындалады.

---

## 🛠 Технологиялық стек

| Бөлім | Технологиялар |
|------|----------------|
| **Frontend** | React 19 · TypeScript · Vite 7 · Tailwind CSS 4 · React Router · Recharts · i18next |
| **Backend** | Tauri 2 · Rust · rusqlite + SQLCipher |
| **Тесттер** | Vitest · Playwright · Rust тесттері |

---

<div align="center">

**MIT** · **Kuralbek Adilet**  
[**github.com/qrlbk/finance-app**](https://github.com/qrlbk/finance-app)

</div>
