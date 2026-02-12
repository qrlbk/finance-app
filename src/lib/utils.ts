/**
 * Utility functions for the finance app
 */

/**
 * Format a number as currency amount
 * @param amount The numeric amount
 * @param currency The currency code (default: KZT)
 * @returns Formatted string with currency symbol
 */
export function formatAmount(amount: number, currency = "KZT"): string {
  const currencySymbols: Record<string, string> = {
    KZT: "₸",
    USD: "$",
    EUR: "€",
    RUB: "₽",
  };
  
  const symbol = currencySymbols[currency] || currency;
  
  const formatted = new Intl.NumberFormat("ru-KZ", {
    style: "decimal",
    minimumFractionDigits: 0,
    maximumFractionDigits: 2,
  }).format(Math.abs(amount));
  
  return `${formatted} ${symbol}`;
}

/**
 * Format amount with sign based on transaction type
 * @param amount The numeric amount
 * @param type Transaction type ('income' | 'expense')
 * @returns Formatted string with +/- prefix
 */
export function formatAmountWithSign(amount: number, type: string): string {
  const prefix = type === "income" ? "+" : "-";
  return `${prefix}${formatAmount(amount)}`;
}

/**
 * Format a date string to locale-friendly format
 * @param dateStr ISO date string (YYYY-MM-DD)
 * @returns Formatted date string
 */
export function formatDate(dateStr: string): string {
  if (!dateStr) return dateStr;
  const date = new Date(dateStr);
  if (isNaN(date.getTime())) return dateStr;
  return date.toLocaleDateString("ru-RU", {
    day: "numeric",
    month: "short",
    year: "numeric",
  });
}

/**
 * Format a date string to show relative time (today, yesterday, etc.)
 * @param dateStr ISO date string (YYYY-MM-DD)
 * @returns Relative time string
 */
export function formatRelativeDate(dateStr: string): string {
  if (!dateStr) return dateStr;
  const date = new Date(dateStr);
  if (isNaN(date.getTime())) return dateStr;
  
  const today = new Date();
  const yesterday = new Date(today);
  yesterday.setDate(yesterday.getDate() - 1);
  
  const dateOnly = date.toDateString();
  
  if (dateOnly === today.toDateString()) {
    return "Сегодня";
  }
  if (dateOnly === yesterday.toDateString()) {
    return "Вчера";
  }
  
  return formatDate(dateStr);
}

/**
 * Format a number as percentage
 * @param value The numeric value
 * @param decimals Number of decimal places
 * @returns Formatted percentage string
 */
export function formatPercent(value: number, decimals = 1): string {
  return `${value >= 0 ? "+" : ""}${value.toFixed(decimals)}%`;
}

/**
 * Validate that a string is a valid date
 * @param dateStr The date string to validate
 * @returns true if valid date
 */
export function isValidDate(dateStr: string): boolean {
  if (!dateStr) return false;
  const date = new Date(dateStr);
  return !isNaN(date.getTime());
}

/**
 * Get the current date as ISO string (YYYY-MM-DD)
 * @returns Current date string
 */
export function getCurrentDate(): string {
  return new Date().toISOString().split("T")[0];
}

/**
 * Get start of month date as ISO string
 * @param date Optional date, defaults to current
 * @returns Start of month date string
 */
export function getStartOfMonth(date = new Date()): string {
  const d = new Date(date.getFullYear(), date.getMonth(), 1);
  const year = d.getFullYear();
  const month = String(d.getMonth() + 1).padStart(2, '0');
  return `${year}-${month}-01`;
}

/**
 * Get end of month date as ISO string
 * @param date Optional date, defaults to current
 * @returns End of month date string
 */
export function getEndOfMonth(date = new Date()): string {
  const d = new Date(date.getFullYear(), date.getMonth() + 1, 0);
  const year = d.getFullYear();
  const month = String(d.getMonth() + 1).padStart(2, '0');
  const day = String(d.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
}

/**
 * Clamp a number between min and max
 */
export function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

/**
 * Calculate percentage (with safe division)
 */
export function calculatePercent(part: number, total: number): number {
  if (total === 0) return 0;
  return (part / total) * 100;
}

/**
 * Debounce a function
 */
export function debounce<T extends (...args: unknown[]) => void>(
  fn: T,
  delay: number
): (...args: Parameters<T>) => void {
  let timeoutId: ReturnType<typeof setTimeout>;
  return (...args: Parameters<T>) => {
    clearTimeout(timeoutId);
    timeoutId = setTimeout(() => fn(...args), delay);
  };
}

/**
 * Truncate text with ellipsis
 */
export function truncate(text: string, maxLength: number): string {
  if (text.length <= maxLength) return text;
  return text.slice(0, maxLength - 3) + "...";
}

/**
 * Capitalize first letter
 */
export function capitalize(text: string): string {
  if (!text) return "";
  return text.charAt(0).toUpperCase() + text.slice(1);
}

/**
 * Generate a random color for categories
 */
export function generateCategoryColor(): string {
  const colors = [
    "#ef4444", "#f97316", "#f59e0b", "#eab308",
    "#84cc16", "#22c55e", "#10b981", "#14b8a6",
    "#06b6d4", "#0ea5e9", "#3b82f6", "#6366f1",
    "#8b5cf6", "#a855f7", "#d946ef", "#ec4899",
  ];
  return colors[Math.floor(Math.random() * colors.length)];
}
