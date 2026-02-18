import { useTranslation } from "react-i18next";

const SECTION_IDS = [
  "gettingStarted",
  "accounts",
  "transactions",
  "transfers",
  "budgets",
  "recurring",
  "insightsReports",
  "importExport",
  "backup",
  "settings",
  "language",
] as const;

export function Docs() {
  const { t } = useTranslation();

  return (
    <article className="max-w-3xl mx-auto">
      <h1 className="text-2xl font-bold text-zinc-900 dark:text-zinc-100 mb-2">
        {t("docs.title")}
      </h1>
      <p className="text-zinc-500 dark:text-zinc-400 text-sm mb-8">
        {t("docs.subtitle")}
      </p>

      <nav className="mb-10 p-4 rounded-xl bg-zinc-50 dark:bg-zinc-800/50 border border-zinc-200 dark:border-zinc-700">
        <h2 className="text-sm font-semibold text-zinc-500 dark:text-zinc-400 uppercase tracking-wide mb-3">
          {t("docs.tocTitle")}
        </h2>
        <ul className="space-y-2">
          {SECTION_IDS.map((id) => (
            <li key={id}>
              <a
                href={`#${id}`}
                className="text-emerald-600 dark:text-emerald-400 hover:underline focus:outline-none focus:ring-2 focus:ring-emerald-500 rounded"
              >
                {t(`docs.sections.${id}.title`)}
              </a>
            </li>
          ))}
        </ul>
      </nav>

      <div className="space-y-8">
        {SECTION_IDS.map((id) => {
          const body = t(`docs.sections.${id}.body`);
          const paragraphs = body.split("\n\n").filter(Boolean);
          return (
            <section
              key={id}
              id={id}
              className="scroll-mt-20 p-6 rounded-xl border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900/50"
            >
              <h2 className="text-lg font-semibold text-zinc-900 dark:text-zinc-100 mb-4">
                {t(`docs.sections.${id}.title`)}
              </h2>
              <div className="text-zinc-600 dark:text-zinc-300 text-base leading-relaxed space-y-3">
                {paragraphs.map((para, i) => (
                  <p key={i} className="whitespace-pre-line">
                    {para}
                  </p>
                ))}
              </div>
            </section>
          );
        })}
      </div>
    </article>
  );
}
