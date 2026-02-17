import { useState, useRef, useEffect } from "react";
import { MessageCircle, Send, Bot, User } from "lucide-react";
import { api } from "../lib/api";
import { useToast } from "../components/ui/Toast";

const SYSTEM_PROMPT =
  "Ты дружелюбный помощник в приложении «Финансы». Отвечай на русском на основе данных пользователя: баланс, доходы и расходы за месяц, бюджеты по категориям. Если пользователь спрашивает о бюджете или финансах — используй только переданные данные.";

function formatAmount(n: number) {
  return new Intl.NumberFormat("ru-KZ", { maximumFractionDigits: 0 }).format(n);
}

async function buildChatContext(): Promise<string> {
  const now = new Date();
  const year = now.getFullYear();
  const month = now.getMonth() + 1;
  const parts: string[] = [];
  try {
    const summary = await api.getSummary();
    parts.push(
      `Баланс: ${formatAmount(summary.total_balance)} ₸. Доход за текущий месяц: ${formatAmount(summary.income_month)} ₸. Расход за текущий месяц: ${formatAmount(summary.expense_month)} ₸.`
    );
    if (summary.currencies.length > 0) {
      parts.push(`Валюты в приложении: ${summary.currencies.join(", ")}.`);
    }
  } catch {
    // ignore
  }
  try {
    const budgets = await api.getBudgets();
    if (budgets.length > 0) {
      const budgetLines = budgets.map(
        (b) =>
          `Бюджет «${b.category_name}» (${b.period}): лимит ${formatAmount(b.amount)} ₸, потрачено ${formatAmount(b.spent)} ₸, осталось ${formatAmount(b.remaining)} ₸ (использовано ${Math.round(b.percent_used)}%).`
      );
      parts.push("Бюджеты по категориям: " + budgetLines.join(" "));
    }
  } catch {
    // ignore
  }
  try {
    const alerts = await api.getBudgetAlerts();
    if (alerts.length > 0) {
      const alertLines = alerts.map(
        (a) => `«${a.category_name}»: ${a.severity === "exceeded" ? "превышен" : "предупреждение"} (${Math.round(a.percent_used)}%)`
      );
      parts.push("Предупреждения по бюджетам: " + alertLines.join(". "));
    }
  } catch {
    // ignore
  }
  try {
    const byCategory = await api.getExpenseByCategory({ year, month });
    if (byCategory.length > 0) {
      const lines = byCategory.map((c) => `${c.category_name}: ${formatAmount(c.total)} ₸`).join(", ");
      parts.push(`Расходы по категориям за текущий месяц: ${lines}.`);
    }
  } catch {
    // ignore
  }
  return parts.join("\n");
}

interface ChatMessage {
  role: "user" | "assistant";
  content: string;
}

const TYPING_INTERVAL_MS = 22;
const TYPING_CHUNK_SIZE = 1;

export function Chat() {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState("");
  const [loading, setLoading] = useState(false);
  const [typingBuffer, setTypingBuffer] = useState<string | null>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const { showToast } = useToast();

  const scrollToBottom = () => messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  // Анимация печати по символам для ответа ассистента
  useEffect(() => {
    if (!typingBuffer) return;
    let index = 0;
    const full = typingBuffer;
    const id = setInterval(() => {
      index += TYPING_CHUNK_SIZE;
      const shown = full.slice(0, Math.min(index, full.length));
      setMessages((prev) => {
        const next = [...prev];
        const last = next[next.length - 1];
        if (last?.role !== "assistant") return prev;
        next[next.length - 1] = { ...last, content: shown };
        return next;
      });
      if (index >= full.length) {
        clearInterval(id);
        setTypingBuffer(null);
      }
    }, TYPING_INTERVAL_MS);
    return () => clearInterval(id);
  }, [typingBuffer]);

  const llmEnabled = typeof localStorage !== "undefined" && localStorage.getItem("llm_enabled") === "true";
  const useEmbedded = typeof localStorage !== "undefined" && localStorage.getItem("llm_use_embedded") === "true";
  const ollamaUrl = typeof localStorage !== "undefined" ? (localStorage.getItem("ollama_url") ?? undefined) : undefined;
  const ollamaModel = typeof localStorage !== "undefined" ? (localStorage.getItem("ollama_model") ?? undefined) : undefined;

  const handleSend = async () => {
    const text = input.trim();
    if (!text || loading) return;
    if (!llmEnabled && !useEmbedded) {
      showToast("Включите LLM в Настройках: «Подсказки категорий» → «Встроенная модель» или «Ollama вручную».", "info");
      return;
    }
    setInput("");
    setMessages((prev) => [...prev, { role: "user", content: text }]);
    setLoading(true);
    try {
      const context = await buildChatContext();
      api.startOllamaServer();
      await new Promise((r) => setTimeout(r, 1500));
      const reply = await api.chatMessage(text, {
        systemPrompt: SYSTEM_PROMPT,
        context: context || undefined,
        useEmbedded: useEmbedded || undefined,
        ollamaUrl: !useEmbedded && llmEnabled ? ollamaUrl ?? undefined : undefined,
        ollamaModel: !useEmbedded && llmEnabled ? ollamaModel ?? undefined : undefined,
      });
      const fullText = reply.trim() || "—";
      setMessages((prev) => [...prev, { role: "assistant", content: "" }]);
      setTypingBuffer(fullText);
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      showToast(`Ошибка: ${msg}. Запустите Ollama и нажмите «Проверить» в Настройках.`, "error");
      setMessages((prev) => [...prev, { role: "assistant", content: `Ошибка: ${msg}` }]);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center gap-2">
        <MessageCircle className="text-emerald-500" size={24} />
        <h3 className="text-lg font-medium">Чат</h3>
      </div>
      <div className="rounded-xl border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900/50 flex flex-col min-h-[400px]">
        <div className="flex-1 overflow-y-auto p-4 space-y-4 min-h-[320px]">
          {messages.length === 0 && (
            <div className="text-center text-zinc-500 dark:text-zinc-400 py-8">
              <Bot size={40} className="mx-auto mb-2 opacity-60" />
              <p>Напишите сообщение — ответит модель Ollama.</p>
              <p className="text-sm mt-1">У ассистента есть доступ к вашему балансу, бюджетам и расходам — можно спросить: «Расскажи о моём бюджете».</p>
            </div>
          )}
          {messages.map((m, i) => {
            const isLastAssistant = m.role === "assistant" && i === messages.length - 1;
            const isTyping = isLastAssistant && typingBuffer !== null;
            return (
            <div
              key={i}
              className={`flex gap-3 ${m.role === "user" ? "justify-end" : "justify-start"}`}
            >
              {m.role === "assistant" && (
                <div className="flex-shrink-0 w-8 h-8 rounded-full bg-emerald-500/20 flex items-center justify-center">
                  <Bot size={16} className="text-emerald-500" />
                </div>
              )}
              <div
                className={`max-w-[85%] rounded-2xl px-4 py-2.5 ${
                  m.role === "user"
                    ? "bg-emerald-600 text-white"
                    : "bg-zinc-200 dark:bg-zinc-700 text-zinc-900 dark:text-zinc-100"
                }`}
              >
                <p className="text-sm whitespace-pre-wrap break-words inline">
                  {m.content}
                  {isTyping && (
                    <span className="inline-block w-0.5 h-4 ml-0.5 bg-emerald-500 align-middle animate-pulse" style={{ animationDuration: "0.6s" }} />
                  )}
                </p>
              </div>
              {m.role === "user" && (
                <div className="flex-shrink-0 w-8 h-8 rounded-full bg-zinc-500/30 flex items-center justify-center">
                  <User size={16} className="text-zinc-400" />
                </div>
              )}
            </div>
            );
          })}
          {loading && (
            <div className="flex gap-3 justify-start">
              <div className="flex-shrink-0 w-8 h-8 rounded-full bg-emerald-500/20 flex items-center justify-center">
                <Bot size={16} className="text-emerald-500" />
              </div>
              <div className="rounded-2xl px-4 py-2.5 bg-zinc-200 dark:bg-zinc-700">
                <span className="text-zinc-500 animate-pulse">...</span>
              </div>
            </div>
          )}
          <div ref={messagesEndRef} />
        </div>
        <div className="p-3 border-t border-zinc-200 dark:border-zinc-700">
          <form
            onSubmit={(e) => {
              e.preventDefault();
              handleSend();
            }}
            className="flex gap-2"
          >
            <input
              type="text"
              value={input}
              onChange={(e) => setInput(e.target.value)}
              placeholder="Напишите сообщение..."
              className="flex-1 rounded-lg border border-zinc-300 dark:border-zinc-600 bg-white dark:bg-zinc-800 px-4 py-2.5 text-sm focus:outline-none focus:ring-2 focus:ring-emerald-500"
              disabled={loading}
            />
            <button
              type="submit"
              disabled={loading || !input.trim()}
              className="rounded-lg bg-emerald-600 text-white px-4 py-2.5 flex items-center gap-2 hover:bg-emerald-500 disabled:opacity-50 disabled:pointer-events-none"
            >
              <Send size={18} />
              Отправить
            </button>
          </form>
        </div>
      </div>
    </div>
  );
}
