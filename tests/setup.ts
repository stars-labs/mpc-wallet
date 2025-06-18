import { vi } from 'vitest';

// Mock browser APIs that are not available in test environment
global.chrome = {
    runtime: {
        sendMessage: vi.fn(),
        onMessage: {
            addListener: vi.fn(),
            removeListener: vi.fn()
        },
        getURL: vi.fn((path) => `chrome-extension://mock-extension-id/${path}`),
        id: 'mock-extension-id'
    },
    storage: {
        local: {
            get: vi.fn(),
            set: vi.fn(),
            remove: vi.fn(),
            clear: vi.fn()
        },
        sync: {
            get: vi.fn(),
            set: vi.fn(),
            remove: vi.fn(),
            clear: vi.fn()
        }
    },
    tabs: {
        query: vi.fn(),
        sendMessage: vi.fn(),
        create: vi.fn(),
        update: vi.fn(),
        remove: vi.fn()
    },
    windows: {
        create: vi.fn(),
        update: vi.fn(),
        remove: vi.fn(),
        getCurrent: vi.fn()
    },
    action: {
        setIcon: vi.fn(),
        setBadgeText: vi.fn(),
        setBadgeBackgroundColor: vi.fn()
    },
    offscreen: {
        createDocument: vi.fn(),
        closeDocument: vi.fn(),
        hasDocument: vi.fn()
    }
} as any;

// Mock WebRTC APIs
global.RTCPeerConnection = vi.fn() as any;
global.RTCDataChannel = vi.fn() as any;
global.RTCSessionDescription = vi.fn() as any;
global.RTCIceCandidate = vi.fn() as any;

// Mock WebSocket
global.WebSocket = vi.fn(() => ({
    send: vi.fn(),
    close: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    readyState: 1
})) as any;

// Mock crypto.subtle if not available
if (!global.crypto) {
    global.crypto = {} as any;
}

if (!global.crypto.subtle) {
    global.crypto.subtle = {
        generateKey: vi.fn(),
        importKey: vi.fn(),
        exportKey: vi.fn(),
        encrypt: vi.fn(),
        decrypt: vi.fn(),
        sign: vi.fn(),
        verify: vi.fn(),
        digest: vi.fn(),
        deriveBits: vi.fn(),
        deriveKey: vi.fn(),
        generateKey: vi.fn(),
        wrapKey: vi.fn(),
        unwrapKey: vi.fn()
    } as any;
}

// Mock crypto.getRandomValues if not available
if (!global.crypto.getRandomValues) {
    global.crypto.getRandomValues = vi.fn((array: any) => {
        for (let i = 0; i < array.length; i++) {
            array[i] = Math.floor(Math.random() * 256);
        }
        return array;
    });
}

// Setup TextEncoder/TextDecoder for Node environment
import { TextEncoder, TextDecoder } from 'util';
global.TextEncoder = TextEncoder as any;
global.TextDecoder = TextDecoder as any;

// Mock window object for tests that need it
global.window = {
    ...global.window,
    location: {
        href: 'http://localhost:3000',
        origin: 'http://localhost:3000',
        protocol: 'http:',
        host: 'localhost:3000',
        hostname: 'localhost',
        port: '3000',
        pathname: '/',
        search: '',
        hash: ''
    },
    dispatchEvent: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    postMessage: vi.fn()
} as any;

// Clean up chrome.storage mock data between tests
beforeEach(() => {
    const mockStorage: any = {};
    
    (global.chrome.storage.local.get as any).mockImplementation((keys: string | string[] | null, callback?: (items: any) => void) => {
        const result: any = {};
        if (keys === null || keys === undefined) {
            Object.assign(result, mockStorage);
        } else if (typeof keys === 'string') {
            if (keys in mockStorage) {
                result[keys] = mockStorage[keys];
            }
        } else if (Array.isArray(keys)) {
            keys.forEach(key => {
                if (key in mockStorage) {
                    result[key] = mockStorage[key];
                }
            });
        }
        
        if (callback) {
            callback(result);
            return undefined;
        }
        return Promise.resolve(result);
    });
    
    (global.chrome.storage.local.set as any).mockImplementation((items: any, callback?: () => void) => {
        Object.assign(mockStorage, items);
        if (callback) {
            callback();
            return undefined;
        }
        return Promise.resolve();
    });
    
    (global.chrome.storage.local.remove as any).mockImplementation((keys: string | string[], callback?: () => void) => {
        if (typeof keys === 'string') {
            delete mockStorage[keys];
        } else if (Array.isArray(keys)) {
            keys.forEach(key => delete mockStorage[key]);
        }
        if (callback) {
            callback();
            return undefined;
        }
        return Promise.resolve();
    });
    
    (global.chrome.storage.local.clear as any).mockImplementation((callback?: () => void) => {
        Object.keys(mockStorage).forEach(key => delete mockStorage[key]);
        if (callback) {
            callback();
            return undefined;
        }
        return Promise.resolve();
    });
});