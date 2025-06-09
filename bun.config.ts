// Bun development configuration
export default {
    // Development server configuration
    server: {
        port: 3000,
        hostname: 'localhost'
    },

    // Build configuration for Bun
    build: {
        target: 'browser',
        minify: process.env.NODE_ENV === 'production',
        splitting: true,
        external: ['chrome'],
        define: {
            'process.env.NODE_ENV': JSON.stringify(process.env.NODE_ENV || 'development')
        }
    },

    // Test configuration
    test: {
        timeout: 30000,
        coverage: {
            enabled: true,
            reports: ['text', 'html', 'lcov'],
            dir: './coverage',
            include: ['src/**/*.ts', 'src/**/*.js'],
            exclude: [
                'src/**/*.test.ts',
                'src/**/*.spec.ts',
                'pkg/**',
                'target/**',
                'node_modules/**'
            ]
        }
    },

    // WASM configuration
    wasm: {
        async: true,
        syncWebAssembly: false
    }
};
