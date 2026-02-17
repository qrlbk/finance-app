import { test, expect } from '@playwright/test';

/**
 * E2E checks for LLM (Ollama) integration: verify that when LLM is enabled in settings,
 * the frontend sends predict_category with use_llm / use_embedded / ollama_url / ollama_model.
 * Uses mocks; does not require a running Ollama server.
 */
test.describe('Ollama / LLM integration', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(() => {
      const mockState = {
        accounts: [
          { id: 1, name: 'Основной', account_type: 'checking', balance: 100000, currency: 'KZT' },
        ],
        categories: [
          { id: 1, name: 'Продукты', category_type: 'expense', icon: null, color: null, parent_id: null },
          { id: 2, name: 'Зарплата', category_type: 'income', icon: null, color: null, parent_id: null },
        ],
        transactions: [] as { id: number; amount: number; transaction_type: string }[],
        lastPredictCategoryArgs: null as Record<string, unknown> | null,
      };

      // @ts-expect-error Mock Tauri
      window.__TAURI__ = {
        invoke: async (cmd: string, args?: Record<string, unknown>) => {
          if (cmd === 'predict_category') {
            mockState.lastPredictCategoryArgs = args ?? null;
            return null;
          }
          if (cmd === 'get_accounts') return mockState.accounts;
          if (cmd === 'get_categories') return mockState.categories;
          if (cmd === 'get_summary') return { total_balance: 100000, income_month: 0, expense_month: 0, currencies: ['KZT'] };
          if (cmd === 'get_transactions') return mockState.transactions;
          if (cmd === 'get_transfers') return [];
          if (cmd === 'get_budgets') return [];
          if (cmd === 'get_budget_alerts') return [];
          if (cmd === 'get_insights') return { anomalies: [], forecast: null };
          if (cmd === 'get_recurring_payments') return [];
          if (cmd === 'get_model_status') return { trained: false, trained_at: null, sample_count: null, accuracy: null, transactions_with_categories_count: 0, transactions_with_note_no_category: 0 };
          if (cmd === 'get_embedded_llm_status') return { downloaded: false, download_progress: null, registered_in_ollama: false, error: null };
          if (cmd === 'create_transaction') {
            mockState.transactions.push({ id: mockState.transactions.length + 1, amount: 0, transaction_type: 'expense' });
            return mockState.transactions.length;
          }
          return null;
        },
      };

      (window as unknown as { __lastPredictCategoryArgs: () => Record<string, unknown> | null }).__lastPredictCategoryArgs = () =>
        mockState.lastPredictCategoryArgs;
    });
  });

  test('sends use_llm and ollama params when LLM enabled (manual Ollama)', async ({ page }) => {
    await page.goto('/transactions');
    await page.evaluate(() => {
      localStorage.setItem('llm_enabled', 'true');
      localStorage.setItem('llm_use_embedded', 'false');
      localStorage.setItem('ollama_url', 'http://127.0.0.1:11434');
      localStorage.setItem('ollama_model', 'llama3.2');
    });
    await page.reload();
    await page.getByRole('button', { name: /добавить/i }).first().click();
    const noteInput = page.getByPlaceholder(/описание/i).first();
    await noteInput.fill('Оплата в магазине');
    await page.waitForTimeout(600);
    const args = await page.evaluate(() => (window as unknown as { __lastPredictCategoryArgs: () => Record<string, unknown> | null }).__lastPredictCategoryArgs());
    expect(args).not.toBeNull();
    expect(args?.note).toBe('Оплата в магазине');
    expect(args?.use_llm).toBe(true);
    expect(args?.ollama_url).toBe('http://127.0.0.1:11434');
    expect(args?.ollama_model).toBe('llama3.2');
    expect(args?.transaction_type).toBeDefined();
  });

  test('sends use_embedded when embedded model selected', async ({ page }) => {
    await page.goto('/transactions');
    await page.evaluate(() => {
      localStorage.setItem('llm_enabled', 'true');
      localStorage.setItem('llm_use_embedded', 'true');
    });
    await page.reload();
    await page.getByRole('button', { name: /добавить/i }).first().click();
    const noteInput = page.getByPlaceholder(/описание/i).first();
    await noteInput.fill('Такси');
    await page.waitForTimeout(600);
    const args = await page.evaluate(() => (window as unknown as { __lastPredictCategoryArgs: () => Record<string, unknown> | null }).__lastPredictCategoryArgs());
    expect(args).not.toBeNull();
    expect(args?.use_llm).toBe(true);
    expect(args?.use_embedded).toBe(true);
  });
});
