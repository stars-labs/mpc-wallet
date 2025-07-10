# MPC Wallet Monorepo Structure

This document describes the monorepo organization for the MPC Wallet project.

## 📁 Directory Structure

```
mpc-wallet/
├── apps/                              # Deployable applications
│   ├── browser-extension/             # Chrome/Firefox extension
│   ├── cli-node/                      # Rust CLI for MPC operations
│   └── signal-server/                 # WebRTC signaling servers
│       ├── server/                    # Standard server implementation
│       └── cloudflare-worker/         # Edge deployment version
│
├── packages/                          # Shared packages
│   └── @mpc-wallet/
│       ├── core-wasm/                 # FROST WASM bindings
│       ├── types/                     # Shared TypeScript types
│       └── utils/                     # Shared utilities
│
├── scripts/                           # Monorepo management scripts
│   ├── build-all.sh                   # Build all packages
│   ├── test-all.sh                    # Run all tests
│   └── clean-all.sh                   # Clean all artifacts
│
├── docs/                              # Centralized documentation
├── package.json                       # Root workspace configuration
├── Cargo.toml                         # Rust workspace configuration
└── bun.lockb                          # Bun lockfile
```

## 🚀 Quick Start

```bash
# Install dependencies
bun install

# Build everything
bun run build

# Start development
bun run dev

# Run tests
bun run test

# Clean all build artifacts
bun run clean
```

## 📦 Packages

### Apps

- **`@mpc-wallet/browser-extension`**: Main browser extension with UI
- **`cli-node`**: Command-line interface for running MPC nodes
- **`webrtc-signal-server`**: WebSocket signaling server for WebRTC
- **`webrtc-signal-server-cloudflare-worker`**: Edge-deployed signaling server

### Shared Packages

- **`@mpc-wallet/core-wasm`**: WebAssembly bindings for FROST protocol
- **`@mpc-wallet/types`**: Shared TypeScript type definitions
- **`@mpc-wallet/utils`**: Common utility functions

## 🛠️ Development Workflow

### Working on the Browser Extension

```bash
cd apps/browser-extension
bun run dev
```

### Building WASM Package

```bash
cd packages/@mpc-wallet/core-wasm
bun run build
```

### Running CLI Node

```bash
cd apps/cli-node
cargo run -- --help
```

## 📝 Workspace Configuration

### Bun Workspaces (package.json)

```json
{
  "workspaces": [
    "apps/*",
    "packages/@mpc-wallet/*"
  ]
}
```

### Cargo Workspaces (Cargo.toml)

```toml
[workspace]
members = [
    "apps/cli-node",
    "apps/signal-server/server",
    "apps/signal-server/cloudflare-worker",
    "packages/@mpc-wallet/core-wasm",
]
```

## 🔗 Inter-Package Dependencies

- Browser Extension depends on:
  - `@mpc-wallet/core-wasm` (FROST crypto operations)
  - `@mpc-wallet/types` (TypeScript types)
  - `@mpc-wallet/utils` (shared utilities)

- CLI Node depends on:
  - `webrtc-signal-server` (local path dependency)

## 🧪 Testing Strategy

- Unit tests: Run within each package
- Integration tests: In `apps/browser-extension/tests/integration/`
- E2E tests: Manual testing with multiple browser instances

## 🚢 Deployment

- Browser Extension: Build and upload to Chrome/Firefox stores
- CLI Node: Build standalone binaries with `cargo build --release`
- Signal Server: Deploy to cloud providers or Cloudflare Workers

## 📚 Documentation

All documentation is centralized in the `docs/` directory at the root level for easy access and maintenance.