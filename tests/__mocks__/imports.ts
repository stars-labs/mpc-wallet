// Mock for #imports used by WXT
import { jest } from 'bun:test';

export const browser = (global as any).chrome || {
  runtime: {
    id: 'test-extension-id',
    sendMessage: jest.fn(),
    onMessage: {
      addListener: jest.fn(),
      removeListener: jest.fn()
    }
  },
  storage: {
    local: {
      get: jest.fn(),
      set: jest.fn(),
      remove: jest.fn()
    }
  }
};

// Create a function to get fresh storage data for each test
let getStorageData: () => Record<string, any> = () => ({});

// Function to reset storage data
export const resetStorageData = (dataGetter?: () => Record<string, any>) => {
  if (dataGetter) {
    getStorageData = dataGetter;
  } else {
    const freshData: Record<string, any> = {};
    getStorageData = () => freshData;
  }
  
  // Also reset all mock implementations to use the new data
  if (storage.getItem) {
    storage.getItem.mockImplementation(async (name: string) => getStorageData()[name] || null);
  }
  if (storage.setItem) {
    storage.setItem.mockImplementation(async (name: string, value: any) => {
      getStorageData()[name] = value;
    });
  }
  if (storage.removeItem) {
    storage.removeItem.mockImplementation(async (name: string) => {
      delete getStorageData()[name];
    });
  }
  if (storage.clear) {
    storage.clear.mockImplementation(async () => {
      const data = getStorageData();
      Object.keys(data).forEach(key => delete data[key]);
    });
  }
};

export const storage = {
  defineItem: jest.fn((name: string) => ({
    getValue: jest.fn().mockImplementation(async () => getStorageData()[name] || null),
    setValue: jest.fn().mockImplementation(async (value: any) => {
      getStorageData()[name] = value;
    }),
    removeValue: jest.fn().mockImplementation(async () => {
      delete getStorageData()[name];
    }),
    watch: jest.fn()
  })),
  getItem: jest.fn().mockImplementation(async (name: string) => getStorageData()[name] || null),
  setItem: jest.fn().mockImplementation(async (name: string, value: any) => {
    getStorageData()[name] = value;
  }),
  removeItem: jest.fn().mockImplementation(async (name: string) => {
    delete getStorageData()[name];
  }),
  clear: jest.fn().mockImplementation(async () => {
    const data = getStorageData();
    Object.keys(data).forEach(key => delete data[key]);
  })
};

// Initialize with empty storage
resetStorageData();

export default {
  browser,
  storage
};