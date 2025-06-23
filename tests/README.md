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
└── setup.ts         # Test setup and utilities
```

## Running Tests

```bash
# Run all tests
npm test

# Run tests in watch mode
npm run test:watch

# Run specific test suites
npm run test:unit        # Components, config, services
npm run test:integration # Integration tests
npm run test:webrtc     # WebRTC tests (uses Bun)

# Run with coverage
npm run test:coverage

# Open test UI
npm run test:ui
```

## Test Runners

- **Vitest**: Used for most tests (components, services, integration)
- **Bun**: Used for WebRTC tests due to WASM requirements

## Writing Tests

1. Place test files next to the code they test with `.test.ts` extension
2. Use descriptive test names
3. Follow the existing test patterns
4. Mock external dependencies appropriately
5. Ensure tests are deterministic and don't depend on external state