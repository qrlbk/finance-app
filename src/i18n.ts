import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import ru from "./locales/ru.json";
import kk from "./locales/kk.json";
import en from "./locales/en.json";

const LOCALE_STORAGE_KEY = "finance-locale";
const SUPPORTED = ["kk", "ru", "en"] as const;
export type Locale = (typeof SUPPORTED)[number];

function getStoredLocale(): Locale | null {
  if (typeof window === "undefined") return null;
  const v = localStorage.getItem(LOCALE_STORAGE_KEY);
  return SUPPORTED.includes(v as Locale) ? (v as Locale) : null;
}

export function setStoredLocale(locale: Locale): void {
  if (typeof window === "undefined") return;
  localStorage.setItem(LOCALE_STORAGE_KEY, locale);
  document.documentElement.lang = locale === "kk" ? "kk-KZ" : locale === "ru" ? "ru-RU" : "en-US";
}

function applyDocumentLang(lng: string): void {
  if (typeof document === "undefined") return;
  const locale = lng === "kk" ? "kk-KZ" : lng === "ru" ? "ru-RU" : "en-US";
  document.documentElement.lang = locale;
}

i18n.use(initReactI18next).init({
  resources: {
    ru: { translation: ru },
    kk: { translation: kk },
    en: { translation: en },
  },
  lng: getStoredLocale() ?? undefined,
  fallbackLng: "ru",
  supportedLngs: [...SUPPORTED],
  interpolation: { escapeValue: false },
  react: { useSuspense: false },
});

const stored = getStoredLocale();
if (stored) {
  i18n.changeLanguage(stored).then(() => {
    applyDocumentLang(stored);
    document.title = i18n.t("app.title");
  });
} else {
  applyDocumentLang(i18n.language || "ru");
  document.title = i18n.t("app.title");
}

i18n.on("languageChanged", (lng) => {
  applyDocumentLang(lng);
  if (SUPPORTED.includes(lng as Locale)) {
    setStoredLocale(lng as Locale);
  }
  const title = i18n.t("app.title");
  if (typeof document !== "undefined" && title) {
    document.title = title;
  }
});

export default i18n;
