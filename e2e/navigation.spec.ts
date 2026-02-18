import { test, expect } from '@playwright/test';

// Note: These tests run against the Vite dev server and mock Tauri API calls
// For full E2E testing with Tauri, use tauri-driver or webdriver

test.describe('Navigation', () => {
  test.beforeEach(async ({ page }) => {
    // Mock the Tauri API
    await page.addInitScript(() => {
      // @ts-expect-error Mock Tauri
      window.__TAURI__ = {
        invoke: async (cmd: string) => {
          // Mock responses for different commands
          const mocks: Record<string, unknown> = {
            get_accounts: [
              { id: 1, name: 'Основной', account_type: 'checking', balance: 100000, currency: 'KZT' }
            ],
            get_categories: [
              { id: 1, name: 'Продукты', category_type: 'expense', icon: null, color: '#22c55e' },
              { id: 2, name: 'Зарплата', category_type: 'income', icon: null, color: '#3b82f6' },
            ],
            get_summary: { total_balance: 100000, income_month: 500000, expense_month: 200000, currencies: ['KZT'], base_currency: 'KZT' },
            get_transactions: [],
            get_monthly_totals: [
              { month: '2024-01', income: 500000, expense: 200000 },
              { month: '2024-02', income: 500000, expense: 180000 },
            ],
            get_budgets: [],
            get_insights: { anomalies: [], forecast: null },
          };
          return mocks[cmd] ?? null;
        },
      };
    });
    await page.goto('/');
  });

  test('should load dashboard page', async ({ page }) => {
    await expect(page).toHaveTitle(/Finance/);
    // Check for main layout elements
    await expect(page.locator('nav')).toBeVisible();
  });

  test('should navigate to transactions page', async ({ page }) => {
    await page.click('a[href="/transactions"]');
    await expect(page).toHaveURL(/\/transactions/);
  });

  test('should navigate to accounts page', async ({ page }) => {
    await page.click('a[href="/accounts"]');
    await expect(page).toHaveURL(/\/accounts/);
  });

  test('should navigate to reports page', async ({ page }) => {
    await page.click('a[href="/reports"]');
    await expect(page).toHaveURL(/\/reports/);
  });

  test('should navigate to settings page', async ({ page }) => {
    await page.click('a[href="/settings"]');
    await expect(page).toHaveURL(/\/settings/);
  });

  test('should navigate to recurring payments page', async ({ page }) => {
    await page.click('a[href="/recurring"]');
    await expect(page).toHaveURL(/\/recurring/);
  });

  test('should navigate to categories page', async ({ page }) => {
    await page.click('a[href="/categories"]');
    await expect(page).toHaveURL(/\/categories/);
  });

  test('should navigate to insights page', async ({ page }) => {
    await page.click('a[href="/insights"]');
    await expect(page).toHaveURL(/\/insights/);
  });
});
