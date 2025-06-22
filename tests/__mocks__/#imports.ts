// Mock storage implementation for tests
const mockStorageData = new Map<string, any>();

// Mock WXT storage API
export const storage = {
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

// Mock WXT browser API
export const browser = {
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
};

// Export all mocks
export default {
  storage,
  browser
};