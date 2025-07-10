// Preload script for Bun tests to mock #imports module
import { mock } from 'bun:test';

// First, register the mock module immediately
mock.module("#imports", () => {
  // Mock storage implementation
  const mockStorageData = new Map<string, any>();

  const mockStorage = {
    getItem: async (key: string) => {
      return mockStorageData.get(key) ?? null;
    },
    
    setItem: async (key: string, value: any) => {
      mockStorageData.set(key, value);
    },
    
    removeItem: async (key: string) => {
      mockStorageData.delete(key);
    },
    
    clear: async () => {
      mockStorageData.clear();
    },
    
    defineItem: (key: string, options?: any) => ({
      getValue: async () => {
        const value = mockStorageData.get(key);
        return value !== undefined ? value : (options?.fallback ?? null);
      },
      setValue: async (value: any) => {
        mockStorageData.set(key, value);
      },
      removeValue: async () => {
        mockStorageData.delete(key);
      },
      key,
      options
    })
  };

  return {
    storage: mockStorage,
    browser: {
      runtime: {
        sendMessage: () => Promise.resolve(),
        onMessage: {
          addListener: () => {},
          removeListener: () => {}
        },
        getURL: (path: string) => `chrome-extension://mock-extension-id/${path}`,
        id: 'mock-extension-id'
      },
      storage: {
        local: {
          get: () => Promise.resolve({}),
          set: () => Promise.resolve(),
          remove: () => Promise.resolve(),
          clear: () => Promise.resolve()
        }
      }
    }
  };
});