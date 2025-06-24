# Test Structure

This directory contains all tests for the MPC Wallet browser extension.

## Organization

```
tests/
├── components/        # UI component tests
├── config/           # Configuration tests
├── entrypoints/      # Extension entrypoint tests
│   ├── background/   # Background service worker tests
│   └── offscreen/    # Offscreen document tests (WebRTC, FROST)
├── integration/      # Integration tests
├── services/         # Service layer tests
└── setup-bun.ts     # Test setup and utilities for Bun
```

## Running Tests

All tests now use Bun for consistent WebAssembly (WASM) support:

```bash
# Run all tests
npm test

# Run tests in watch mode
npm run test:watch

# Run specific test suites
npm run test:unit        # Components, config, services
npm run test:integration # Integration tests
npm run test:webrtc     # WebRTC tests

# Run with coverage
npm run test:coverage
```

## Test Runner

- **Bun**: Used for all tests with native WebAssembly support
- Tests are preloaded with `setup-bun.ts` which provides Chrome API mocks and crypto mocks

## Migration from Vitest

All tests have been migrated from Vitest to Bun to ensure proper WebAssembly support throughout the test suite. Key changes:

1. Import from `'bun:test'` instead of `'vitest'`
2. Use `jest.fn()` for mocking (Bun uses Jest-compatible APIs)
3. WASM modules work natively without special configuration

## Writing Tests

1. Place test files next to the code they test with `.test.ts` extension
2. Use descriptive test names
3. Follow the existing test patterns
4. Mock external dependencies appropriately
5. Ensure tests are deterministic and don't depend on external state