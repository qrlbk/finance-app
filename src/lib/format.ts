import i18n from "../i18n";

const LOCALE_MAP: Record<string, string> = {
  kk: "kk-KZ",
  ru: "ru-KZ",
  en: "en-US",
};

/**
 * Map i18n language code to Intl locale for number/date formatting.
 */
export function getIntlLocale(lang?: string): string {
  const l = lang || i18n.language || "ru";
  const base = l.split("-")[0];
  return LOCALE_MAP[base] || "ru-KZ";
}

/**
 * Format a number as currency/decimal for the current (or given) locale.
 */
export function formatCurrency(
  amount: number,
  localeOrLang?: string,
  options?: { minimumFractionDigits?: number; maximumFractionDigits?: number }
): string {
  const locale = getIntlLocale(localeOrLang);
  const { minimumFractionDigits = 0, maximumFractionDigits = 0 } = options || {};
  return new Intl.NumberFormat(locale, {
    style: "decimal",
    minimumFractionDigits,
    maximumFractionDigits,
  }).format(amount);
}

/**
 * Format a date for the current (or given) locale.
 */
export function formatDate(
  date: Date | string,
  localeOrLang?: string,
  options?: Intl.DateTimeFormatOptions
): string {
  const locale = getIntlLocale(localeOrLang);
  const d = typeof date === "string" ? new Date(date + (date.length === 10 ? "T12:00:00" : "")) : date;
  if (isNaN(d.getTime())) return String(date);
  return d.toLocaleDateString(locale, options ?? { day: "2-digit", month: "2-digit", year: "numeric" });
}

/**
 * Format month name (long) for the current (or given) locale.
 */
export function formatMonthLong(date: Date, localeOrLang?: string): string {
  const locale = getIntlLocale(localeOrLang);
  return date.toLocaleDateString(locale, { month: "long" });
}
