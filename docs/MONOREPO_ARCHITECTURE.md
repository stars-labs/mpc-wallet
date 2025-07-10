# MPC Wallet Monorepo Architecture

## Overview

The MPC Wallet has been restructured as a monorepo to support multiple platforms while sharing code efficiently. This document describes the architecture and development practices.

## Directory Structure

```
mpc-wallet/
├── apps/                      # All applications
│   ├── browser-extension/     # Chrome/Firefox extension
│   ├── cli-node/             # Rust CLI for MPC operations
│   ├── native-node/          # Native desktop application
│   └── signal-server/        # WebRTC signaling servers
│       ├── server/           # Standard WebSocket server
│       └── cloudflare-worker/ # Edge deployment
│
├── packages/@mpc-wallet/      # Shared packages
│   ├── frost-core/           # Core FROST cryptography (Rust)
│   ├── core-wasm/            # WebAssembly bindings
│   └── types/                # TypeScript type definitions
│
├── scripts/                   # Monorepo build scripts
├── Cargo.toml                # Rust workspace root
├── package.json              # Bun workspace root
└── flake.nix                 # Nix development environment
```

## Applications

### Browser Extension (`apps/browser-extension/`)
- **Technology**: TypeScript, Svelte, WXT framework
- **Features**: Web3 wallet, FROST MPC, multi-chain support
- **Build**: `bun run build`
- **Dev**: `bun run dev`

### CLI Node (`apps/cli-node/`)
- **Technology**: Rust, TUI (ratatui), WebRTC
- **Features**: Terminal UI, offline mode, keystore management
- **Build**: `cargo build --bin cli_node`
- **Run**: `cargo run --bin cli_node -- --device-id Device-001`

### Native Node (`apps/native-node/`)
- **Technology**: Rust, Slint UI framework
- **Features**: Cross-platform GUI, real-time updates, session management
- **Build**: `cargo build --bin mpc-wallet-native`
- **Run**: `cargo run --bin mpc-wallet-native`

### Signal Servers (`apps/signal-server/`)
- **Standard Server**: Rust WebSocket server for development
- **Cloudflare Worker**: Edge deployment for production

## Shared Packages

### `@mpc-wallet/frost-core`
Core FROST implementation in Rust, shared between CLI and WASM:
- DKG (Distributed Key Generation)
- Threshold signing
- Keystore management
- Multi-curve support (secp256k1, ed25519)

### `@mpc-wallet/core-wasm`
Thin WebAssembly wrapper around frost-core:
- Browser-compatible cryptography
- Async/await interface
- TypeScript bindings

### `@mpc-wallet/types`
Centralized TypeScript type definitions:
- Message types for all communication
- State management interfaces
- Keystore formats
- Network protocols

## Development Workflow

### Prerequisites
```bash
# Install Bun
curl -fsSL https://bun.sh/install | bash

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Or use Nix
nix develop
```

### Building Everything
```bash
# Build all packages and apps
./scripts/build-all.sh

# Or individually:
bun install                    # Install JS dependencies
bun run build:wasm            # Build WASM package
bun run build                 # Build browser extension
cargo build                   # Build all Rust apps
```

### Testing
```bash
# Run all tests
./scripts/test-all.sh

# Or individually:
bun test                      # JS/TS tests
cargo test                    # Rust tests
```

### Development Tips

1. **Shared Types**: Always define types in `@mpc-wallet/types`
2. **Crypto Code**: Implement in `frost-core`, not in apps
3. **Import Paths**: Use `@mpc-wallet/types` not relative paths
4. **Workspace Commands**: Run from root, not subdirectories

## Architecture Principles

### 1. Code Sharing
- Cryptographic operations in `frost-core`
- Business logic in shared packages
- UI-specific code in respective apps

### 2. Type Safety
- Single source of truth for types
- Consistent interfaces across platforms
- Strong typing for all messages

### 3. Platform Independence
- Core logic independent of runtime
- Platform-specific code isolated
- Shared protocols and formats

### 4. Modularity
- Each app can be developed independently
- Shared packages versioned separately
- Clear dependency boundaries

## Communication Flow

```
Browser Extension          CLI Node              Native Node
       |                      |                      |
       |------WebSocket-------|------WebSocket------|
                              |
                        Signal Server
                              |
       |------WebRTC----------|------WebRTC---------|
       
All apps use the same:
- Message types (@mpc-wallet/types)
- Cryptography (@mpc-wallet/frost-core)
- Network protocols
```

## Adding New Features

1. **Define types** in `@mpc-wallet/types`
2. **Implement crypto** in `frost-core` if needed
3. **Add to apps** with platform-specific UI
4. **Test across platforms** to ensure compatibility

## Future Expansion

The monorepo structure enables:
- Mobile applications (React Native)
- Additional blockchain support
- Hardware wallet integration
- Enterprise features
- Improved offline capabilities

## Troubleshooting

### Common Issues

1. **Import errors**: Ensure `@mpc-wallet/types` is built
2. **WASM not found**: Run `bun run build:wasm` first
3. **Type conflicts**: Check for duplicate type definitions
4. **Build failures**: Clean and rebuild from root

### Build Order
1. `packages/@mpc-wallet/types`
2. `packages/@mpc-wallet/frost-core`
3. `packages/@mpc-wallet/core-wasm`
4. Applications

## Contributing

When contributing:
1. Follow the monorepo structure
2. Add tests for shared packages
3. Update types when adding features
4. Ensure cross-platform compatibility
5. Document platform-specific code

This architecture provides a solid foundation for the MPC Wallet ecosystem while maintaining code quality and developer experience.