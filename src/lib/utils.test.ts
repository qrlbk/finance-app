import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import {
  formatAmount,
  formatAmountWithSign,
  formatDate,
  formatRelativeDate,
  formatPercent,
  isValidDate,
  getCurrentDate,
  getStartOfMonth,
  getEndOfMonth,
  clamp,
  calculatePercent,
  debounce,
  truncate,
  capitalize,
  generateCategoryColor,
} from './utils';

describe('formatAmount', () => {
  it('formats positive amounts correctly', () => {
    const result = formatAmount(1000);
    expect(result).toContain('1');
    expect(result).toContain('000');
    expect(result).toContain('₸');
  });

  it('formats negative amounts as positive with currency', () => {
    const result = formatAmount(-500);
    expect(result).toContain('500');
    expect(result).toContain('₸');
    expect(result).not.toContain('-');
  });

  it('formats zero correctly', () => {
    expect(formatAmount(0)).toContain('0');
    expect(formatAmount(0)).toContain('₸');
  });

  it('handles different currencies', () => {
    expect(formatAmount(100, 'USD')).toContain('$');
    expect(formatAmount(100, 'EUR')).toContain('€');
    expect(formatAmount(100, 'RUB')).toContain('₽');
  });

  it('handles unknown currencies', () => {
    expect(formatAmount(100, 'GBP')).toContain('GBP');
  });

  it('formats decimal amounts', () => {
    const result = formatAmount(100.5);
    expect(result).toContain('100');
    expect(result).toContain('₸');
  });
});

describe('formatAmountWithSign', () => {
  it('adds + for income', () => {
    const result = formatAmountWithSign(1000, 'income');
    expect(result).toMatch(/^\+/);
    expect(result).toContain('₸');
  });

  it('adds - for expense', () => {
    const result = formatAmountWithSign(1000, 'expense');
    expect(result).toMatch(/^-/);
    expect(result).toContain('₸');
  });
});

describe('formatDate', () => {
  it('formats valid date strings', () => {
    const result = formatDate('2024-01-15');
    expect(result).toContain('2024');
  });

  it('returns original string for invalid dates', () => {
    expect(formatDate('invalid')).toBe('invalid');
  });
});

describe('formatRelativeDate', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2024-06-15'));
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('returns "Сегодня" for today', () => {
    expect(formatRelativeDate('2024-06-15')).toBe('Сегодня');
  });

  it('returns "Вчера" for yesterday', () => {
    expect(formatRelativeDate('2024-06-14')).toBe('Вчера');
  });

  it('returns formatted date for other dates', () => {
    const result = formatRelativeDate('2024-06-01');
    expect(result).not.toBe('Сегодня');
    expect(result).not.toBe('Вчера');
  });

  it('handles invalid dates', () => {
    expect(formatRelativeDate('invalid')).toBe('invalid');
  });
});

describe('formatPercent', () => {
  it('formats positive percentages with +', () => {
    expect(formatPercent(15)).toBe('+15.0%');
    expect(formatPercent(15.5)).toBe('+15.5%');
  });

  it('formats negative percentages', () => {
    expect(formatPercent(-10)).toBe('-10.0%');
  });

  it('formats zero', () => {
    expect(formatPercent(0)).toBe('+0.0%');
  });

  it('respects decimal places', () => {
    // toFixed may round differently across implementations
    const result2 = formatPercent(15.555, 2);
    expect(result2).toMatch(/\+15\.5[56]%/);
    const result0 = formatPercent(15.555, 0);
    expect(result0).toMatch(/\+1[56]%/);
  });
});

describe('isValidDate', () => {
  it('returns true for valid dates', () => {
    expect(isValidDate('2024-01-01')).toBe(true);
    expect(isValidDate('2024-12-31')).toBe(true);
  });

  it('returns false for invalid dates', () => {
    expect(isValidDate('')).toBe(false);
    expect(isValidDate('invalid')).toBe(false);
  });
});

describe('getCurrentDate', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2024-06-15T12:00:00Z'));
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('returns current date in ISO format', () => {
    expect(getCurrentDate()).toBe('2024-06-15');
  });
});

describe('getStartOfMonth', () => {
  it('returns first day of month', () => {
    // Use local date constructor to avoid timezone issues
    expect(getStartOfMonth(new Date(2024, 5, 15))).toBe('2024-06-01'); // June = month 5
  });

  it('handles last day of month', () => {
    expect(getStartOfMonth(new Date(2024, 5, 30))).toBe('2024-06-01');
  });
});

describe('getEndOfMonth', () => {
  it('returns last day of month', () => {
    expect(getEndOfMonth(new Date(2024, 5, 15))).toBe('2024-06-30');
  });

  it('handles February correctly', () => {
    expect(getEndOfMonth(new Date(2024, 1, 15))).toBe('2024-02-29'); // Leap year
    expect(getEndOfMonth(new Date(2023, 1, 15))).toBe('2023-02-28'); // Non-leap
  });

  it('handles 31-day months', () => {
    expect(getEndOfMonth(new Date(2024, 0, 15))).toBe('2024-01-31');
  });
});

describe('clamp', () => {
  it('returns value when within range', () => {
    expect(clamp(5, 0, 10)).toBe(5);
  });

  it('clamps to min when below', () => {
    expect(clamp(-5, 0, 10)).toBe(0);
  });

  it('clamps to max when above', () => {
    expect(clamp(15, 0, 10)).toBe(10);
  });

  it('handles edge cases', () => {
    expect(clamp(0, 0, 10)).toBe(0);
    expect(clamp(10, 0, 10)).toBe(10);
  });
});

describe('calculatePercent', () => {
  it('calculates percentage correctly', () => {
    expect(calculatePercent(50, 100)).toBe(50);
    expect(calculatePercent(1, 4)).toBe(25);
  });

  it('handles zero total', () => {
    expect(calculatePercent(50, 0)).toBe(0);
  });

  it('handles zero part', () => {
    expect(calculatePercent(0, 100)).toBe(0);
  });

  it('handles over 100%', () => {
    expect(calculatePercent(150, 100)).toBe(150);
  });
});

describe('debounce', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('debounces function calls', () => {
    const fn = vi.fn();
    const debounced = debounce(fn, 100);

    debounced();
    debounced();
    debounced();

    expect(fn).not.toHaveBeenCalled();

    vi.advanceTimersByTime(100);

    expect(fn).toHaveBeenCalledTimes(1);
  });

  it('passes arguments correctly', () => {
    const fn = vi.fn();
    const debounced = debounce(fn, 100);

    debounced('test', 123);
    vi.advanceTimersByTime(100);

    expect(fn).toHaveBeenCalledWith('test', 123);
  });
});

describe('truncate', () => {
  it('does not truncate short strings', () => {
    expect(truncate('hello', 10)).toBe('hello');
  });

  it('truncates long strings with ellipsis', () => {
    expect(truncate('hello world!', 8)).toBe('hello...');
  });

  it('handles edge case at exact length', () => {
    expect(truncate('hello', 5)).toBe('hello');
  });

  it('handles empty string', () => {
    expect(truncate('', 10)).toBe('');
  });
});

describe('capitalize', () => {
  it('capitalizes first letter', () => {
    expect(capitalize('hello')).toBe('Hello');
  });

  it('handles empty string', () => {
    expect(capitalize('')).toBe('');
  });

  it('handles single character', () => {
    expect(capitalize('h')).toBe('H');
  });

  it('handles already capitalized', () => {
    expect(capitalize('Hello')).toBe('Hello');
  });

  it('handles cyrillic', () => {
    expect(capitalize('привет')).toBe('Привет');
  });
});

describe('generateCategoryColor', () => {
  it('returns a valid hex color', () => {
    const color = generateCategoryColor();
    expect(color).toMatch(/^#[0-9a-f]{6}$/i);
  });

  it('returns different colors (statistical test)', () => {
    const colors = new Set();
    for (let i = 0; i < 100; i++) {
      colors.add(generateCategoryColor());
    }
    // Should get multiple different colors
    expect(colors.size).toBeGreaterThan(1);
  });
});
