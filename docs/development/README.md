# Development Guide

## Overview

Complete guide for developing and contributing to the MPC Wallet project.

## Contents

### Getting Started
- [Environment Setup](environment-setup.md) - Development environment configuration
- [Building from Source](building.md) - Build instructions for all components
- [Development Workflow](workflow.md) - Recommended development practices

### Development Guides
- [Browser Extension Development](browser-extension-dev.md) - Chrome/Firefox extension development
- [Rust Development](rust-development.md) - Rust application development
- [WASM Development](wasm-development.md) - WebAssembly module development

### Testing
- [Unit Testing](unit-testing.md) - Writing and running unit tests
- [Integration Testing](integration-testing.md) - End-to-end testing
- [Performance Testing](performance-testing.md) - Benchmarking and optimization

### Tools & Resources
- [Development Tools](tools.md) - Recommended tools and extensions
- [Debugging Guide](debugging.md) - Debugging techniques and tools
- [Troubleshooting](troubleshooting.md) - Common issues and solutions

## Quick Start

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Bun
curl -fsSL https://bun.sh/install | bash

# Install wasm-pack
cargo install wasm-pack

# Install additional tools
cargo install cargo-watch
cargo install cargo-expand
```

### Project Setup

```bash
# Clone repository
git clone https://github.com/your-org/mpc-wallet.git
cd mpc-wallet

# Install dependencies
bun install
cargo build --workspace

# Build WASM module
cd packages/@mpc-wallet/core-wasm
wasm-pack build

# Start development
cd apps/browser-extension
bun run dev
```

## Development Environment

### Recommended IDE Setup

#### VS Code
```json
{
  "extensions": [
    "rust-lang.rust-analyzer",
    "svelte.svelte-vscode",
    "dbaeumer.vscode-eslint",
    "esbenp.prettier-vscode"
  ]
}
```

#### Rust Analyzer Settings
```json
{
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.check.command": "clippy"
}
```

### Environment Variables

```bash
# .env.local
RUST_LOG=debug
SIGNAL_SERVER_URL=ws://localhost:8080
STUN_SERVER=stun:stun.l.google.com:19302
```

## Code Style

### Rust Style Guide
- Follow standard Rust conventions
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Prefer explicit error handling over panics

### TypeScript Style Guide
- Use Prettier for formatting
- Use ESLint for linting
- Prefer functional programming patterns
- Use TypeScript strict mode

### Commit Convention
```
type(scope): description

[optional body]

[optional footer]
```

Types: feat, fix, docs, style, refactor, test, chore

## Project Structure

```
mpc-wallet/
├── apps/                    # Applications
│   ├── browser-extension/   # TypeScript/Svelte
│   ├── tui-node/           # Rust/TUI
│   └── native-node/        # Rust/Slint
│
├── packages/               # Shared packages
│   └── @mpc-wallet/
│       ├── frost-core/     # Rust library
│       └── core-wasm/      # WASM bindings
│
├── scripts/                # Build scripts
├── tests/                  # Integration tests
└── docs/                   # Documentation
```

## Common Tasks

### Running Tests

```bash
# Run all tests
./scripts/test-all.sh

# Run specific test suite
cargo test -p tui-node
bun test browser-extension

# Run with coverage
cargo tarpaulin --workspace
```

### Building for Production

```bash
# Build all components
./scripts/build-all.sh

# Build specific component
cargo build --release -p tui-node
cd apps/browser-extension && bun run build
```

### Debugging

```bash
# Enable debug logging
RUST_LOG=debug cargo run

# Chrome DevTools for extension
chrome://extensions/
# Enable Developer mode
# Click "Inspect views: background page"

# Use Rust debugger
rust-gdb target/debug/tui-node
```

## Performance Optimization

### Profiling Rust Code
```bash
# CPU profiling
cargo build --release
perf record --call-graph=dwarf ./target/release/tui-node
perf report

# Memory profiling
valgrind --tool=massif ./target/release/tui-node
ms_print massif.out.*
```

### Chrome Performance
```javascript
// Performance timing
performance.mark('dkg-start');
// ... DKG operation
performance.mark('dkg-end');
performance.measure('dkg', 'dkg-start', 'dkg-end');
```

## Continuous Integration

### GitHub Actions Workflow
```yaml
name: CI
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
      - run: cargo test --workspace
      - run: bun test
```

## Release Process

1. Update version numbers
2. Update CHANGELOG.md
3. Create release branch
4. Run full test suite
5. Build release artifacts
6. Create GitHub release
7. Publish to package registries

## Getting Help

### Resources
- [Discord Community](https://discord.gg/mpc-wallet)
- [GitHub Discussions](https://github.com/your-org/mpc-wallet/discussions)
- [Stack Overflow](https://stackoverflow.com/questions/tagged/mpc-wallet)

### Debugging Tips
1. Check logs with `RUST_LOG=debug`
2. Use browser DevTools for extension
3. Enable verbose WebRTC logging
4. Check network tab for WebSocket messages

## Navigation

- [← Back to Main Documentation](../README.md)
- [← API Documentation](../api/README.md)
- [Testing Guide →](../testing/README.md)