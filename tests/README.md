# Test Directory Structure

This directory contains all unit tests for the MPC Wallet Chrome Extension.

## Structure

```
tests/
├── services/                    # Service layer tests
│   ├── accountService.test.ts   # Account management tests
│   ├── networkService.test.ts   # Network/blockchain tests
│   ├── walletClient.test.ts     # Wallet client tests
│   └── walletController.test.ts # Wallet controller tests
└── entrypoints/
    └── offscreen/               # Offscreen WebRTC and DKG tests
        ├── webrtc.test.ts           # Main WebRTC functionality tests
        ├── webrtc.dkg.test.ts       # Distributed Key Generation tests
        ├── webrtc.errors.test.ts    # Error handling tests
        ├── webrtc.mesh.test.ts      # Mesh networking tests
        ├── webrtc.signing.test.ts   # FROST signing tests
        ├── webrtc.simple.test.ts    # Basic WebRTC tests
        ├── webrtc.setblockchain.test.ts # Blockchain switching tests
        └── webrtc.environment.test.ts   # Environment-specific tests
```

## Running Tests

### All Tests
```bash
npm run test
# or
npm run test:all
```

### Service Tests Only
```bash
npm run test:services
```

### WebRTC Tests Only
```bash
npm run test:webrtc:all  # All WebRTC tests
npm run test:webrtc      # Simple WebRTC tests only
npm run test:dkg         # DKG-specific tests
npm run test:webrtc:errors # Error handling tests
```

## Test Structure

Each test file follows this pattern:
- Import test framework and dependencies
- Import the module under test from `../../src/...`
- Set up test data and utilities
- Organize tests using `describe` blocks for features
- Use `it` blocks for individual test cases

## Test Dependencies

Tests are configured to:
- Use Bun as the test runner
- Have a 30-second timeout for complex operations
- Initialize WASM modules for cryptographic operations
- Use proper cleanup for resources

## Coverage

Test coverage excludes:
- WASM bindings (`pkg/`)
- Rust build artifacts (`target/`)
- Test files themselves (`tests/`)
- Node modules and other standard exclusions

See `bun.test.config.ts` for complete coverage configuration.
