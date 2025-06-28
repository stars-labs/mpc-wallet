# Scripts Directory

This directory contains utility scripts for development, testing, and building the MPC Wallet extension.

## Structure

### `/test`
Testing scripts:
- `run-all-tests.sh` - Run all test suites
- `run-tests.sh` - Run specific tests
- `test-dkg-ui.sh` - Test DKG UI functionality

### `/build`
Build and maintenance scripts:
- `fix-all-syntax-errors.sh` - Fix syntax errors in source files
- `fix-bun-imports.js` - Fix import statements for Bun compatibility
- `remove-debug-logs.sh` - Remove debug logging statements

### `/`
Development utilities:
- `benchmark.ts` - Performance benchmarking
- `dev.ts` - Development server script
- `performance.ts` - Performance testing utilities

## Usage

### Running Tests
```bash
./scripts/test/run-all-tests.sh
```

### Development
```bash
bun run scripts/dev.ts
```

### Build Fixes
```bash
./scripts/build/fix-all-syntax-errors.sh
```