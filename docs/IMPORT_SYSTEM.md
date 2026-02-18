# Import System — Design Document

Professional import pipeline for CSV, JSON, and Excel with predictable results and clear error reporting.

---

## Goals

- **Predictable outcome** — Import from CSV/JSON/Excel with consistent behavior and understandable errors.
- **External files** — Support “foreign” files via column mapping, default account, and date format options.
- **Safety** — Duplicate detection and validation before writing to the database (preview + report).

---

## 1. Sources and formats

| Format | Scenario | Mapping |
|--------|----------|---------|
| **JSON** | Export from this app | Fixed `ExportData` structure; no mapping. |
| **CSV** | App export or external (bank, spreadsheet) | Headers: EN/RU supported (Date/Дата, Amount/Сумма, Account/Счёт, Category/Категория, Type/Тип, Note/Заметка). Optional: column mapping from UI. |
| **XLSX** | App export or Excel | First sheet; fixed column order or first-row headers. |

---

## 2. Import flow (UX)

1. **File selection** — User picks CSV / JSON / XLSX.
2. **Preview (CSV/XLSX)** — Show first 10–20 rows and column list; for CSV, suggest mapping to fields (Date, Amount, Account, Category, Type, Note).
3. **Import settings**
   - **Default account** — Used when the file has no Account column or value is not found.
   - **Skip duplicates** — Duplicate = same (date, amount, note) per account; when enabled, skip and count in report (`duplicates_skipped`).
   - **Date format (CSV)** — Auto (YYYY-MM-DD, DD.MM.YYYY, DD/MM/YYYY) or explicit.
4. **Run import** — Progress for large files, then summary.
5. **Report** — Imported count, skipped duplicates, errors (row numbers and messages).

---

## 3. API (backend)

### 3.1 Preview (no DB write)

- **`import_preview(path, format)`**
  - **CSV:** Read header + first 15 rows; return `{ headers: string[], rows: string[][] }`. UI can suggest mapping from headers.
  - **JSON:** Return `{ format: "json", transaction_count: number }` without full parse.
  - **XLSX:** First sheet, first row = headers, next 15 rows; same `{ headers, rows }`.

### 3.2 Import with options

- **`import_data(input)`** — Extended input:
  - `path`, `format` (as now).
  - **Options:**
    - `default_account_id: number | null` — Used when row has no account or account not found.
    - `skip_duplicates: boolean` — Do not insert duplicates (date + amount + note + account).
    - `date_format: "auto" | "iso" | "dmy" | "mdy"` — For CSV (optional).

- **Result** (extend `ImportResult`):
  - `transactions_imported: number`
  - `duplicates_skipped: number`
  - `accounts_imported`, `categories_imported` (for compatibility)
  - `errors: string[]` — Messages with row numbers
  - `total_parsed: number` — Total rows processed

---

## 4. Duplicates

- **Key:** `(account_id, date, amount, note)`.
- When `skip_duplicates: true`, before insert check for existing row (e.g. by `user_id`, `account_id`, `date`, `amount`, `note`). If exists, do not insert and increment `duplicates_skipped`.

---

## 5. CSV: flexible headers

Support synonyms (first match by header):

| Field   | Synonyms (examples) |
|--------|----------------------|
| Date   | `Date`, `Дата`, `date`, `дата` |
| Amount | `Amount`, `Сумма`, `amount`, `сумма` |
| Account| `Account`, `Счёт`, `account`, `счёт` |
| Category | `Category`, `Категория`, `category`, `категория` |
| Type  | `Type`, `Тип`, `type`, `тип` |
| Note  | `Note`, `Заметка`, `Note`, `note`, `заметка`, `Описание`, `Description` |

Future: accept explicit column index → field mapping from UI.

---

## 6. Validation and errors

- Per row: valid date (parsed), valid amount (number), account (found or default), type (income/expense or from amount sign).
- Errors do not stop import: collect in `errors`, continue. Return full report at the end.
- Row numbers in report: 1-based; for CSV, row 2 = first data row.

---

## 7. Implementation order

1. **Extend result:** `duplicates_skipped`, `total_parsed`; skip logic in `import_csv` / `import_json` / `import_xlsx`.
2. **Import options:** `default_account_id`, `skip_duplicates` in `import_data` and pass into import functions.
3. **CSV:** Flexible headers (EN/RU synonyms), date parsing (auto/iso/dmy/mdy).
4. **`import_preview`** for CSV and XLSX.
5. **Frontend:** Settings step (default account, “Skip duplicates”), call preview, show report (imported / skipped / errors).

Later: UI column mapping, other formats (e.g. OFX/QIF), category import from file.
