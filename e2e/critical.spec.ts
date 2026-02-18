import { test, expect } from '@playwright/test';

/**
 * Critical E2E scenarios with mocked Tauri backend.
 * Simulates: account → transaction → dashboard totals; transfer; export; import page.
 * Uses in-memory mock state so tests do not touch real user data.
 */
function installCriticalMocks(page: { addInitScript: (fn: () => void) => Promise<void> }, state: {
  accounts: { id: number; name: string; account_type: string; balance: number; currency: string }[];
  transactions: { id: number; amount: number; transaction_type: string }[];
  transfers: { id: number; from_account_id: number; to_account_id: number; amount: number; date: string }[];
  summary: { total_balance: number; income_month: number; expense_month: number; currencies: string[]; base_currency: string };
}) {
  return page.addInitScript((s: typeof state) => {
    const mockState = s as typeof state;
    if (!mockState.summary.currencies) mockState.summary.currencies = [];
    if (!mockState.summary.base_currency) mockState.summary.base_currency = 'KZT';

    // @ts-expect-error Mock Tauri
    window.__TAURI__ = {
      invoke: async (cmd: string, args?: { input?: unknown }) => {
        const input = args?.input ?? {};
        switch (cmd) {
          case 'get_accounts':
            return mockState.accounts;
          case 'get_categories':
            return [
              { id: 1, name: 'Продукты', category_type: 'expense', icon: null, color: '#22c55e', parent_id: null },
              { id: 2, name: 'Зарплата', category_type: 'income', icon: null, color: '#3b82f6', parent_id: null },
            ];
          case 'get_summary':
            return mockState.summary;
          case 'get_transactions':
            return mockState.transactions.map((t, i) => ({
              id: t.id || i + 1,
              account_id: 1,
              account_name: 'Основной',
              category_id: 1,
              category_name: 'Продукты',
              amount: t.amount,
              transaction_type: t.transaction_type,
              note: null,
              date: new Date().toISOString().slice(0, 10),
            }));
          case 'get_transfers':
            return (mockState.transfers || []).map((tr, i) => ({
              id: tr.id || i + 1,
              from_account_id: tr.from_account_id,
              from_account_name: 'Счёт 1',
              to_account_id: tr.to_account_id,
              to_account_name: 'Счёт 2',
              amount: tr.amount,
              date: tr.date,
              note: null,
            }));
          case 'create_account': {
            const inp = input as { name?: string; account_type?: string; currency?: string };
            const id = mockState.accounts.length + 1;
            mockState.accounts.push({
              id,
              name: inp.name || 'Новый счёт',
              account_type: inp.account_type || 'checking',
              balance: 0,
              currency: inp.currency || 'KZT',
            });
            return id;
          }
          case 'create_transaction': {
            const inp = input as { amount?: number; transaction_type?: string };
            const amount = Number(inp.amount) || 0;
            const txType = (inp.transaction_type as string) || 'expense';
            const signed = txType === 'income' ? Math.abs(amount) : -Math.abs(amount);
            mockState.transactions.push({
              id: mockState.transactions.length + 1,
              amount: signed,
              transaction_type: txType,
            });
            mockState.summary.total_balance += signed;
            if (txType === 'income') mockState.summary.income_month += Math.abs(amount);
            else mockState.summary.expense_month += Math.abs(amount);
            return mockState.transactions.length;
          }
          case 'create_transfer': {
            const inp = input as { from_account_id?: number; to_account_id?: number; amount?: number; date?: string };
            mockState.transfers.push({
              id: mockState.transfers.length + 1,
              from_account_id: inp.from_account_id ?? 1,
              to_account_id: inp.to_account_id ?? 2,
              amount: inp.amount ?? 0,
              date: (inp.date as string) || new Date().toISOString().slice(0, 10),
            });
            return undefined;
          }
          case 'export_data':
            return '/tmp/finance_export_test.csv';
          case 'get_budgets':
            return [];
          case 'get_monthly_totals':
            return [];
          case 'get_budget_alerts':
            return [];
          case 'get_current_session':
            return { id: 1, username: 'test', display_name: 'Test User' };
          case 'get_insights':
            return { anomalies: [], forecast: null };
          case 'get_recurring_payments':
            return [];
          case 'predict_category':
            return null;
          default:
            return null;
        }
      },
    };
  }, state);
}

test.describe('Critical: Account and transactions', () => {
  const state = {
    accounts: [
      { id: 1, name: 'Основной', account_type: 'checking', balance: 100000, currency: 'KZT' },
      { id: 2, name: 'Сбережения', account_type: 'savings', balance: 50000, currency: 'KZT' },
    ],
    transactions: [] as { id: number; amount: number; transaction_type: string }[],
    transfers: [] as { id: number; from_account_id: number; to_account_id: number; amount: number; date: string }[],
    summary: {
      total_balance: 150000,
      income_month: 0,
      expense_month: 0,
      currencies: ['KZT'],
      base_currency: 'KZT',
    },
  };

  test.beforeEach(async ({ page }) => {
    state.transactions = [];
    state.transfers = [];
    state.summary = { total_balance: 150000, income_month: 0, expense_month: 0, currencies: ['KZT'], base_currency: 'KZT' };
    await installCriticalMocks(page, state);
    await page.goto('/');
  });

  test('create account then add income and expense, dashboard shows totals', async ({ page }) => {
    await page.goto('/accounts');
    await page.locator('button').filter({ hasText: /добавить|создать|новый/i }).first().click();
    await page.fill('input[placeholder*="назван" i], input[name="name"]', 'Тест-счёт');
    await page.locator('button[type="submit"]').click();
    await expect(page.locator('text=Тест-счёт').first()).toBeVisible({ timeout: 5000 });

    await page.goto('/transactions');
    await page.locator('button').filter({ hasText: /доход/i }).first().click();
    await page.fill('input[type="number"]', '5000');
    await page.locator('button[type="submit"]').click();

    await page.goto('/');
    await expect(page.locator('text=150').first()).toBeVisible({ timeout: 5000 });
    await expect(page.locator('text=5').first()).toBeVisible({ timeout: 3000 });
  });

  test('create transfer flow', async ({ page }) => {
    await page.goto('/transfers');
    await expect(page.locator('text=Переводы').first()).toBeVisible({ timeout: 5000 });
    await page.locator('button').filter({ hasText: /перевод|добавить/i }).first().click();
    await page.fill('input[type="number"]', '1000');
    await page.locator('form').locator('button[type="submit"]').click();
    await expect(page.getByText(/перевод выполнен|успешно/i)).toBeVisible({ timeout: 5000 });
  });
});

test.describe('Critical: Export', () => {
  const state = {
    accounts: [{ id: 1, name: 'Основной', account_type: 'checking', balance: 10000, currency: 'KZT' }],
    transactions: [],
    transfers: [],
    summary: { total_balance: 10000, income_month: 0, expense_month: 0, currencies: ['KZT'], base_currency: 'KZT' },
  };

  test.beforeEach(async ({ page }) => {
    await installCriticalMocks(page, state);
  });

  test('export data from settings', async ({ page }) => {
    await page.goto('/settings');
    await page.locator('button').filter({ hasText: /Экспортировать/i }).first().click();
    await expect(page.getByText(/Данные экспортированы|Файл сохранён/i)).toBeVisible({ timeout: 10000 });
  });
});

test.describe('Critical: Settings page', () => {
  test.beforeEach(async ({ page }) => {
    await installCriticalMocks(page, {
      accounts: [{ id: 1, name: 'Основной', account_type: 'checking', balance: 10000, currency: 'KZT' }],
      transactions: [],
      transfers: [],
      summary: { total_balance: 10000, income_month: 0, expense_month: 0, currencies: ['KZT'], base_currency: 'KZT' },
    });
  });

  test('settings page loads with theme and backup sections', async ({ page }) => {
    await page.goto('/settings');
    await expect(page.getByRole('heading', { name: /Тема/i })).toBeVisible({ timeout: 5000 });
    await expect(page.getByText(/Тёмная|Светлая|Системная/)).toBeVisible();
    await expect(page.getByRole('heading', { name: /Резервная копия/i })).toBeVisible();
    await expect(page.getByText(/Создать резервную копию/)).toBeVisible();
  });
});

test.describe('Critical: Reports page', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(() => {
      const mock: Record<string, () => unknown> = {
        get_current_session: () => ({ id: 1, username: 'test', display_name: 'Test User' }),
        get_accounts: () => [{ id: 1, name: 'Основной', account_type: 'checking', balance: 0, currency: 'KZT' }],
        get_categories: () => [{ id: 1, name: 'Продукты', category_type: 'expense', icon: null, color: null, parent_id: null }],
        get_summary: () => ({ total_balance: 0, income_month: 0, expense_month: 0, currencies: ['KZT'], base_currency: 'KZT' }),
        get_expense_by_category: () => [],
        get_monthly_totals: () => [],
        get_forecast_details: () => ({ overall: { predicted_expense: 0, confidence_low: 0, confidence_high: 0, trend: 'stable', trend_percent: 0 }, by_category: [] }),
      };
      // @ts-expect-error Mock Tauri
      window.__TAURI__ = {
        invoke: async (cmd: string) => {
          const fn = mock[cmd];
          return fn ? fn() : null;
        },
      };
    });
  });

  test('reports page loads and shows include subcategories option', async ({ page }) => {
    await page.goto('/reports');
    await expect(page.locator('text=Отчёты').first()).toBeVisible({ timeout: 5000 });
    await expect(page.getByLabel(/Включая подкатегории/i)).toBeVisible({ timeout: 3000 });
  });
});

test.describe('Critical: Import page', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(() => {
      // @ts-expect-error Mock Tauri
      window.__TAURI__ = {
        invoke: async (cmd: string) => {
          if (cmd === 'get_accounts') return [{ id: 1, name: 'Основной', account_type: 'checking', balance: 0, currency: 'KZT' }];
          if (cmd === 'get_categories') return [{ id: 1, name: 'Продукты', category_type: 'expense', icon: null, color: null, parent_id: null }];
          return null;
        },
      };
    });
  });

  test('import page loads and shows upload step', async ({ page }) => {
    await page.goto('/import');
    await expect(page.locator('text=Импорт').first()).toBeVisible({ timeout: 5000 });
    await expect(page.locator('text=выписк|PDF|загрузи/i').first()).toBeVisible({ timeout: 3000 });
  });
});
