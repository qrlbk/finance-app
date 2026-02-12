import '@testing-library/jest-dom';
import { vi } from 'vitest';

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

// Mock window.__TAURI__
Object.defineProperty(window, '__TAURI__', {
  value: {
    invoke: vi.fn(),
  },
  writable: true,
});
