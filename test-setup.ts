// Test setup file for Bun - automatically loaded before each test
import { beforeAll, afterAll } from 'bun:test';

// Global WASM initialization
let wasmInitialized = false;

beforeAll(async () => {
    if (!wasmInitialized) {
        try {
            // Dynamic import to avoid issues during test discovery
            const wasmInit = await import('../pkg/mpc_wallet.js');
            await wasmInit.default();
            console.log('✅ WASM initialized globally for all tests');
            wasmInitialized = true;
        } catch (error) {
            console.warn('⚠️ WASM initialization failed globally, tests will handle individually:', error);
        }
    }
});

afterAll(() => {
    // Cleanup if needed
    console.log('🧹 Test cleanup completed');
});

// Global test utilities
globalThis.hexEncode = (str: string): string => {
    return Buffer.from(str, 'utf8').toString('hex');
};

globalThis.participantIndexToHexKey = (index: number, isSecp256k1: boolean): string => {
    const buffer = Buffer.alloc(32);
    if (isSecp256k1) {
        buffer.writeUInt32BE(index, 28);
    } else {
        buffer.writeUInt16LE(index, 0);
    }
    return buffer.toString('hex');
};

// Export for explicit imports
export { wasmInitialized };
