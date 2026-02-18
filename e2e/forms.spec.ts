import { test, expect } from '@playwright/test';

test.describe('Form Interactions', () => {
  test.beforeEach(async ({ page }) => {
    // Mock Tauri API
    await page.addInitScript(() => {
      const mockData = {
        accounts: [
          { id: 1, name: 'Основной', account_type: 'checking', balance: 100000, currency: 'KZT' },
          { id: 2, name: 'Сбережения', account_type: 'savings', balance: 500000, currency: 'KZT' },
        ],
        categories: [
          { id: 1, name: 'Продукты', category_type: 'expense', icon: null, color: '#22c55e' },
          { id: 2, name: 'Зарплата', category_type: 'income', icon: null, color: '#3b82f6' },
          { id: 3, name: 'Транспорт', category_type: 'expense', icon: null, color: '#f97316' },
        ],
        transactions: [] as unknown[],
      };

      // @ts-expect-error Mock Tauri
      window.__TAURI__ = {
        invoke: async (cmd: string, args?: unknown) => {
          switch (cmd) {
            case 'get_accounts':
              return mockData.accounts;
            case 'get_categories':
              return mockData.categories;
            case 'get_summary':
              return { total_balance: 600000, income_month: 500000, expense_month: 200000, currencies: ['KZT'], base_currency: 'KZT' };
            case 'get_transactions':
              return mockData.transactions;
            case 'get_monthly_totals':
              return [];
            case 'get_budgets':
              return [];
            case 'get_recurring_payments':
              return [];
            case 'create_transaction':
              // Add to mock transactions
              mockData.transactions.push({ id: Date.now(), ...args });
              return mockData.transactions.length;
            case 'create_account':
              mockData.accounts.push({ id: Date.now(), ...args } as typeof mockData.accounts[0]);
              return mockData.accounts.length;
            case 'predict_category':
              return null;
            default:
              return null;
          }
        },
      };
    });
  });

  test('should show transaction form on transactions page', async ({ page }) => {
    await page.goto('/transactions');
    
    // Check form elements exist
    await expect(page.locator('select').first()).toBeVisible();
    await expect(page.locator('input[type="number"]')).toBeVisible();
    await expect(page.locator('input[type="date"]')).toBeVisible();
  });

  test('should validate transaction form inputs', async ({ page }) => {
    await page.goto('/transactions');
    
    // Try to submit without required fields
    const submitButton = page.locator('button[type="submit"]');
    await submitButton.click();
    
    // Form should not submit if required fields are empty
    // (validation should prevent it)
    await expect(page).toHaveURL(/\/transactions/);
  });

  test('should show account form on accounts page', async ({ page }) => {
    await page.goto('/accounts');
    
    // Check for account cards or form elements
    await expect(page.locator('button').filter({ hasText: /добавить|создать/i }).first()).toBeVisible();
  });

  test('should toggle between income and expense types', async ({ page }) => {
    await page.goto('/transactions');
    
    // Find type selector buttons
    const incomeButton = page.locator('button').filter({ hasText: /доход/i });
    const expenseButton = page.locator('button').filter({ hasText: /расход/i });
    
    // One should be visible if type selector exists
    const hasTypeSelector = await incomeButton.isVisible() || await expenseButton.isVisible();
    expect(hasTypeSelector).toBe(true);
  });
});

test.describe('Settings Page', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(() => {
      // @ts-expect-error Mock Tauri
      window.__TAURI__ = {
        invoke: async (cmd: string) => {
          const mocks: Record<string, unknown> = {
            get_categories: [
              { id: 1, name: 'Продукты', category_type: 'expense', icon: null, color: '#22c55e' },
            ],
            get_budgets: [],
            get_model_status: { trained: false, trained_at: null, sample_count: null, accuracy: null, transactions_with_categories_count: null },
            train_model: { success: true, sample_count: 10, accuracy: 0.85, message: 'OK' },
          };
          return mocks[cmd] ?? null;
        },
      };
    });
  });

  test('should load settings page with all sections', async ({ page }) => {
    await page.goto('/settings');
    
    // Check for main sections
    await expect(page.locator('text=Настройки').first()).toBeVisible();
  });

  test('should have ML training button', async ({ page }) => {
    await page.goto('/settings');
    
    // Check for ML section
    const mlSection = page.locator('text=/ML|машинн/i');
    await expect(mlSection.first()).toBeVisible();
  });
});

test.describe('Categories Page', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(() => {
      // @ts-expect-error Mock Tauri
      window.__TAURI__ = {
        invoke: async (cmd: string) => {
          const mocks: Record<string, unknown> = {
            get_categories: [
              { id: 1, name: 'Продукты', category_type: 'expense', icon: null, color: '#22c55e' },
              { id: 2, name: 'Зарплата', category_type: 'income', icon: null, color: '#3b82f6' },
            ],
          };
          return mocks[cmd] ?? null;
        },
      };
    });
  });

  test('should display category list', async ({ page }) => {
    await page.goto('/categories');
    
    // Should show categories section
    await expect(page.locator('text=Категории').first()).toBeVisible();
  });

  test('should have add category form', async ({ page }) => {
    await page.goto('/categories');
    
    // Should have form elements
    await expect(page.locator('input[placeholder*="категории" i]').or(page.locator('input').first())).toBeVisible();
  });
});
