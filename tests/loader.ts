// Bun loader for handling #imports
import { jest } from 'bun:test';

// Override require to handle #imports
const originalRequire = require;
(global as any).require = function(id: string) {
  if (id === '#imports') {
    return {
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
  }
  return originalRequire.apply(this, arguments);
};