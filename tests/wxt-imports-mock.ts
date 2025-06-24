// Module mock for #imports used in service files during testing
import { jest } from 'bun:test';

// Use the same storage isolation approach as the other mock
let getStorageData: () => Record<string, any> = () => ({});

export const resetWxtStorageData = (dataGetter?: () => Record<string, any>) => {
  if (dataGetter) {
    getStorageData = dataGetter;
  } else {
    const freshData: Record<string, any> = {};
    getStorageData = () => freshData;
  }
};

// Initialize with empty storage
resetWxtStorageData();

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

export default {
  browser,
  storage
};