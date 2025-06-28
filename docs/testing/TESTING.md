# Testing Guide

## Test Organization

All tests are now consolidated in the `/tests` directory with the following structure:

```
tests/
├── components/        # UI component tests
├── config/           # Configuration tests  
├── entrypoints/      # Extension entrypoint tests
│   ├── background/   # Background service worker tests
│   └── offscreen/    # Offscreen document tests (WebRTC, FROST)
├── integration/      # Integration tests
├── services/         # Service layer tests
├── setup.ts          # Test setup and global mocks
└── README.md         # Test documentation
```

## Available Test Scripts

```bash
# Development
npm run dev              # Start dev server
npm run build            # Build extension
npm run build:wasm       # Build WASM modules

# Testing
npm run test             # Run all tests
npm run test:watch       # Run tests in watch mode
npm run test:coverage    # Run tests with coverage
npm run test:ui          # Open Vitest UI
npm run test:unit        # Run unit tests only
npm run test:integration # Run integration tests only
npm run test:webrtc      # Run WebRTC tests (uses Bun)

# Utilities
npm run check            # Run Svelte type checking
npm run clean            # Clean build artifacts
```

## Test Runners

- **Vitest**: Primary test runner for most tests
- **Bun**: Used specifically for WebRTC tests due to WASM requirements

## Writing Tests

1. Place test files in appropriate directory under `/tests`
2. Use `.test.ts` extension
3. Import test utilities from `vitest`
4. Mock external dependencies appropriately
5. Follow existing patterns in the codebase

## Running Specific Tests

```bash
# Run a specific test file
npm test path/to/test.test.ts

# Run tests matching a pattern
npm test -- --grep "signing"

# Run tests for a specific service
npm test tests/services/walletController.test.ts
```