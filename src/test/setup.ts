import '@testing-library/jest-dom';
import { vi } from 'vitest';

// Extend global interfaces for TypeScript
declare global {
  interface Window {
    __TAURI__: {
      invoke: ReturnType<typeof vi.fn>;
    };
  }
  
  // eslint-disable-next-line no-var
  var __TAURI__: {
    invoke: ReturnType<typeof vi.fn>;
  };
  
  // eslint-disable-next-line no-var
  var ResizeObserver: ReturnType<typeof vi.fn>;
  
  // eslint-disable-next-line no-var
  var IntersectionObserver: ReturnType<typeof vi.fn>;
}

// Mock Tauri API
const mockInvoke = vi.fn();

(globalThis as any).__TAURI__ = {
  invoke: mockInvoke,
};

// Mock window.matchMedia for responsive tests
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation((query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(), // deprecated
    removeListener: vi.fn(), // deprecated
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
});

// Mock ResizeObserver
(globalThis as any).ResizeObserver = vi.fn().mockImplementation(() => ({
  observe: vi.fn(),
  unobserve: vi.fn(),
  disconnect: vi.fn(),
}));

// Mock IntersectionObserver
(globalThis as any).IntersectionObserver = vi.fn().mockImplementation(() => ({
  observe: vi.fn(),
  unobserve: vi.fn(),
  disconnect: vi.fn(),
}));

export { mockInvoke };
