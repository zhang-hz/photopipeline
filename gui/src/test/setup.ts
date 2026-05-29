import "@testing-library/jest-dom/vitest";

// Mock Tauri invoke for store tests
const mockInvoke = vi.fn();
(globalThis as any).__TAURI_INTERNALS__ = { invoke: mockInvoke };
