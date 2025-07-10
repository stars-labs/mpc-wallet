// Mock implementation of WXT storage for testing

export const mockStorage = {
  data: new Map<string, any>(),
  
  getItem: async (key: string) => {
    return mockStorage.data.get(key) ?? null;
  },
  
  setItem: async (key: string, value: any) => {
    mockStorage.data.set(key, value);
  },
  
  removeItem: async (key: string) => {
    mockStorage.data.delete(key);
  },
  
  clear: async () => {
    mockStorage.data.clear();
  },
  
  defineItem: (key: string, options?: any) => ({
    getValue: async () => {
      const value = mockStorage.data.get(key);
      return value !== undefined ? value : (options?.fallback ?? null);
    },
    setValue: async (value: any) => {
      mockStorage.data.set(key, value);
    },
    removeValue: async () => {
      mockStorage.data.delete(key);
    },
    key,
    options
  })
};

// Helper to reset storage between tests
export const resetMockStorage = () => {
  mockStorage.data.clear();
};