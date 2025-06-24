import { beforeEach, mock } from 'bun:test';
import { jest } from 'bun:test';
import * as mockImports from './wxt-imports-mock';

// Mock the #imports module
mock.module('#imports', () => mockImports);

// Create a shared mock storage
const createMockStorage = () => {
    const mockStorage: any = {};
    return {
        get: jest.fn(async (keys: string | string[] | null | undefined) => {
            if (keys === null || keys === undefined) {
                return mockStorage;
            }
            if (typeof keys === 'string') {
                return { [keys]: mockStorage[keys] };
            }
            if (Array.isArray(keys)) {
                const result: any = {};
                keys.forEach(key => {
                    if (mockStorage.hasOwnProperty(key)) {
                        result[key] = mockStorage[key];
                    }
                });
                return result;
            }
            return {};
        }),
        set: jest.fn(async (items: Record<string, any>) => {
            Object.assign(mockStorage, items);
        }),
        remove: jest.fn(async (keys: string | string[]) => {
            if (typeof keys === 'string') {
                delete mockStorage[keys];
            } else {
                keys.forEach(key => delete mockStorage[key]);
            }
        }),
        clear: jest.fn(async () => {
            Object.keys(mockStorage).forEach(key => delete mockStorage[key]);
        })
    };
};

// Create crypto mocks
const createCryptoMocks = () => ({
    generateKey: jest.fn(),
    importKey: jest.fn(),
    exportKey: jest.fn(),
    encrypt: jest.fn(),
    decrypt: jest.fn(),
    sign: jest.fn(),
    verify: jest.fn(),
    digest: jest.fn(),
    deriveBits: jest.fn(),
    deriveKey: jest.fn(),
    wrapKey: jest.fn(),
    unwrapKey: jest.fn()
});

// Mock chrome API
(global as any).chrome = {
    storage: {
        local: createMockStorage()
    },
    runtime: {
        id: 'test-extension-id',
        sendMessage: jest.fn().mockResolvedValue({ success: true }),
        onMessage: {
            addListener: jest.fn(),
            removeListener: jest.fn()
        }
    }
};

// Mock crypto API
(global as any).crypto = {
    subtle: createCryptoMocks(),
    getRandomValues: jest.fn((arr: any) => {
        for (let i = 0; i < arr.length; i++) {
            arr[i] = Math.floor(Math.random() * 256);
        }
        return arr;
    }),
    randomUUID: jest.fn(() => 'test-uuid-' + Math.random().toString(36).substr(2, 9))
};

// Mock WebSocket
(global as any).WebSocket = jest.fn(() => ({
    send: jest.fn(),
    close: jest.fn(),
    addEventListener: jest.fn(),
    removeEventListener: jest.fn(),
    readyState: 1
}));

// Mock RTCPeerConnection
(global as any).RTCPeerConnection = jest.fn(() => ({
    createDataChannel: jest.fn(() => ({
        send: jest.fn(),
        close: jest.fn(),
        addEventListener: jest.fn(),
        removeEventListener: jest.fn(),
        readyState: 'open'
    })),
    createOffer: jest.fn(() => Promise.resolve({ type: 'offer', sdp: 'mock-sdp' })),
    createAnswer: jest.fn(() => Promise.resolve({ type: 'answer', sdp: 'mock-sdp' })),
    setLocalDescription: jest.fn(),
    setRemoteDescription: jest.fn(),
    addIceCandidate: jest.fn(),
    close: jest.fn(),
    addEventListener: jest.fn(),
    removeEventListener: jest.fn()
}));

// Reset mocks before each test
beforeEach(() => {
    jest.clearAllMocks();
    
    // Reset storage
    (chrome.storage.local as any) = createMockStorage();
    
    // Reset crypto mocks
    (crypto.subtle as any) = createCryptoMocks();
});

// Export mocks for use in tests
export const mockChrome = (global as any).chrome;
export const mockCrypto = (global as any).crypto;