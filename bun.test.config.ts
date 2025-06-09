// Bun test configuration for FROST signing tests
export default {
    testTimeout: 30000,
    coverage: {
        enabled: true,
        include: [
            "src/entrypoints/offscreen/webrtc.ts",
            "src/services/**/*.ts",
            "src/types/**/*.ts"
        ],
        exclude: [
            "**/*.test.ts",
            "**/*.spec.ts",
            "pkg/**",
            "target/**"
        ],
        reports: ["text", "html", "lcov"]
    },
    beforeAll: async () => {
        // Global WASM initialization for all tests
        try {
            const wasmInit = await import('./pkg/mpc_wallet.js');
            await wasmInit.default();
            console.log('✅ WASM initialized globally for FROST tests');
        } catch (error) {
            console.warn('⚠️ WASM initialization failed:', error);
        }
    }
};
