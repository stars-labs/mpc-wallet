// ===================================================================
// WASM INITIALIZER MODULE
// ===================================================================
//
// This module handles the initialization of FROST DKG WASM modules.
// It provides both a class-based interface and standalone functions
// for compatibility with different usage patterns.
//
// Responsibilities:
// - Initialize WASM modules (FrostDkgEd25519, FrostDkgSecp256k1)
// - Make WASM classes globally available
// - Provide initialization status checking
// - Handle WASM initialization errors gracefully
// ===================================================================

import wasmInit, { FrostDkgEd25519, FrostDkgSecp256k1 } from '../../../pkg/mpc_wallet.js';

/**
 * Global flag to track WASM initialization status
 */
let wasmInitialized = false;

/**
 * Class-based WASM initializer for object-oriented usage
 */
export class WasmInitializer {
    private static instance: WasmInitializer | null = null;

    /**
     * Get singleton instance
     */
    static getInstance(): WasmInitializer {
        if (!WasmInitializer.instance) {
            WasmInitializer.instance = new WasmInitializer();
        }
        return WasmInitializer.instance;
    }

    /**
     * Initialize WASM modules
     */
    async initialize(): Promise<boolean> {
        return await initializeWasmModules();
    }

    /**
     * Check initialization status
     */
    isInitialized(): boolean {
        return isWasmInitialized();
    }

    /**
     * Get detailed status
     */
    getStatus(): {
        initialized: boolean;
        hasEd25519: boolean;
        hasSecp256k1: boolean;
    } {
        return getWasmStatus();
    }

    /**
     * Force re-initialization
     */
    async reinitialize(): Promise<boolean> {
        return await reinitializeWasm();
    }
}

/**
 * Initialize FROST DKG WASM modules
 * This function must be called before using any WASM functionality
 */
export async function initializeWasmModules(): Promise<boolean> {
    try {
//         console.log("🔧 [WASM] Initializing FROST DKG WASM modules...");
//         console.log("🔧 [WASM] typeof wasmInit:", typeof wasmInit);
//         console.log("🔧 [WASM] typeof FrostDkgEd25519:", typeof FrostDkgEd25519);
//         console.log("🔧 [WASM] typeof FrostDkgSecp256k1:", typeof FrostDkgSecp256k1);

        // Initialize the WASM module
        await wasmInit();
//         console.log("🔧 [WASM] wasmInit() completed successfully");

        // Make WASM classes available globally for WebRTCManager
        (globalThis as any).FrostDkgEd25519 = FrostDkgEd25519;
        (globalThis as any).FrostDkgSecp256k1 = FrostDkgSecp256k1;

//         console.log("🔧 [WASM] Set globalThis.FrostDkgEd25519 to:", typeof (globalThis as any).FrostDkgEd25519);
//         console.log("🔧 [WASM] Set globalThis.FrostDkgSecp256k1 to:", typeof (globalThis as any).FrostDkgSecp256k1);

        // Also set on global if available (for Node.js-like environments)
        if (typeof global !== 'undefined') {
            (global as any).FrostDkgEd25519 = FrostDkgEd25519;
            (global as any).FrostDkgSecp256k1 = FrostDkgSecp256k1;
//             console.log("🔧 [WASM] Also set on global for Node.js compatibility");
        }

        // Test instance creation to verify WASM is working
        try {
            const testInstance = new FrostDkgSecp256k1();
//             console.log("🔧 [WASM] Test instance creation SUCCESS");
//             console.log("🔧 [WASM] Test instance type:", testInstance.constructor.name);
//             console.log("🔧 [WASM] Test instance has add_round1_package:", typeof testInstance.add_round1_package);
        } catch (testError) {
//             console.log("🔧 [WASM] Test instance creation FAILED:", testError);
        }

        wasmInitialized = true;
//         console.log("✅ [WASM] FROST DKG WASM modules initialized successfully");
//         console.log("📦 [WASM] Available modules: FrostDkgEd25519, FrostDkgSecp256k1");

        return true;
    } catch (error) {
        console.error("❌ [WASM] Failed to initialize FROST DKG WASM modules:", error);
        console.error("❌ [WASM] Error details:", JSON.stringify(error));
        console.error("❌ [WASM] Error stack:", error instanceof Error ? error.stack : 'No stack trace');
        wasmInitialized = false;
        return false;
    }
}

/**
 * Check if WASM modules are initialized and ready to use
 */
export function isWasmInitialized(): boolean {
    return wasmInitialized;
}

/**
 * Get the WASM initialization status with detailed information
 */
export function getWasmStatus(): {
    initialized: boolean;
    hasEd25519: boolean;
    hasSecp256k1: boolean;
} {
    return {
        initialized: wasmInitialized,
        hasEd25519: typeof (globalThis as any).FrostDkgEd25519 !== 'undefined',
        hasSecp256k1: typeof (globalThis as any).FrostDkgSecp256k1 !== 'undefined'
    };
}

/**
 * Force re-initialization of WASM modules (for testing/recovery)
 */
export async function reinitializeWasm(): Promise<boolean> {
//     console.log("🔄 [WASM] Force re-initializing WASM modules...");
    wasmInitialized = false;
    return await initializeWasmModules();
}
