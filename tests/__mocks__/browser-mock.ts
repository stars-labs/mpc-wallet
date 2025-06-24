// Browser mock for tests
export const browser = (global as any).chrome || {
  runtime: {
    id: 'test-extension-id',
    sendMessage: jest.fn ? jest.fn() : (() => {}),
    onMessage: {
      addListener: jest.fn ? jest.fn() : (() => {}),
      removeListener: jest.fn ? jest.fn() : (() => {})
    }
  },
  storage: {
    local: {
      get: jest.fn ? jest.fn(() => Promise.resolve({})) : (() => Promise.resolve({})),
      set: jest.fn ? jest.fn(() => Promise.resolve()) : (() => Promise.resolve()),
      remove: jest.fn ? jest.fn(() => Promise.resolve()) : (() => Promise.resolve())
    }
  }
};