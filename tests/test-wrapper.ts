// Test wrapper that provides mocks for #imports
import { jest } from 'bun:test';

// Create module mock
const importsMock = {
  browser: (global as any).chrome,
  storage: {
    defineItem: jest.fn((name: string) => ({
      getValue: jest.fn().mockResolvedValue(null),
      setValue: jest.fn().mockResolvedValue(undefined),
      removeValue: jest.fn().mockResolvedValue(undefined),
      watch: jest.fn()
    }))
  }
};

// Use Bun's module mock functionality
Bun.mock.module('#imports', () => importsMock);

// Export for use in tests
export { importsMock };