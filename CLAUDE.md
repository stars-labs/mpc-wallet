# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

MPC Wallet is a monorepo containing a browser extension, CLI node, and WebRTC signaling servers that implement Multi-Party Computation for blockchain wallets using the FROST (Flexible Round-Optimized Schnorr Threshold) signature scheme. It enables distributed key generation and signing operations where no single party holds the complete private key. The system supports both Ethereum (secp256k1) and Solana (ed25519) blockchains.

### Monorepo Structure

```
apps/
├── browser-extension/    # Chrome/Firefox extension with UI
├── cli-node/            # Rust CLI for MPC node operations (also used as library)
├── native-node/         # Native desktop app with Slint UI (shares CLI node core)
└── signal-server/       # WebRTC signaling infrastructure
    ├── server/          # Standard WebSocket server
    └── cloudflare-worker/ # Edge deployment

packages/@mpc-wallet/
├── frost-core/         # Shared FROST Rust library (NEW)
├── core-wasm/          # FROST WebAssembly bindings (thin wrapper)
└── types/              # Shared TypeScript types
```

## Architecture

The extension follows Chrome Extension Manifest V3 architecture with four main contexts:

1. **Background Service Worker** (`apps/browser-extension/src/entrypoints/background/`): Central message router managing WebSocket connections to a signaling server and coordinating communication between components.
   
2. **Popup Page** (`apps/browser-extension/src/entrypoints/popup/`): User interface for wallet operations built with Svelte.
   
3. **Offscreen Document** (`apps/browser-extension/src/entrypoints/offscreen/`): Handles WebRTC connections for peer-to-peer communication and cryptographic operations using Rust/WebAssembly.
   
4. **Content Script** (`apps/browser-extension/src/entrypoints/content/`): Injects wallet provider API into web pages.

### Key Components

- **WebRTC System** (`apps/browser-extension/src/entrypoints/offscreen/webrtc.ts`): Manages peer-to-peer connections and coordinates the MPC protocol.
  
- **Shared FROST Core** (`packages/@mpc-wallet/frost-core/`): Shared Rust library implementing core FROST cryptographic operations, keystore management, and encryption. Used by WASM, CLI, and native applications to eliminate code duplication.

- **Rust/WebAssembly Core** (`packages/@mpc-wallet/core-wasm/src/lib.rs`): Thin wrapper around frost-core providing WASM bindings for browser usage.

- **CLI Node Library** (`apps/cli-node/src/lib.rs`): The CLI node exposes its core functionality as a Rust library, allowing the native desktop app to reuse all the WebSocket, WebRTC, DKG, and signing logic without duplication.

- **Native Desktop App** (`apps/native-node/`): Desktop application with Slint UI that uses the CLI node as a library. Features an adapter pattern to bridge UI events to the CLI's command system.
  
- **Shared Types** (`packages/@mpc-wallet/types/`): Centralized TypeScript type definitions shared across all applications. Includes messages, state, session, DKG, keystore, and network types.
  
- **Message System** (`packages/@mpc-wallet/types/src/messages.ts`): Strongly-typed messages for communication between extension components.

- **Keystore Service** (`apps/browser-extension/src/services/keystoreService.ts`): Manages secure storage of FROST key shares with encryption and backup capabilities.

- **DKG Manager** (`apps/browser-extension/src/entrypoints/offscreen/dkgManager.ts`): Handles the complete Distributed Key Generation lifecycle across multiple rounds.

## Build and Development Commands

### Prerequisites

```bash
# Install Bun runtime
curl -fsSL https://bun.sh/install | bash

# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

### Development

```bash
# Install dependencies
bun install

# Build WASM (required before running dev)
bun run build:wasm          # Production WASM build
bun run build:wasm:dev      # Development WASM build (faster, larger)

# Start development server with hot reloading
bun run dev                 # Auto-rebuilds WASM on changes

# Run development for specific browsers
bun run dev:firefox
bun run dev:edge

# Type checking
bun run check               # Svelte type checking

# Clean build artifacts
bun run clean              # Remove dist and .wxt directories
```

### Testing

```bash
# Run all tests with setup preload
bun test

# Run specific test categories
bun run test:unit          # Unit tests only (services, components, config)
bun run test:integration   # Integration tests only
bun run test:services      # Service layer tests
bun run test:webrtc        # WebRTC tests (simple)
bun run test:webrtc:all    # All WebRTC tests
bun run test:dkg           # DKG functionality tests

# Run with options
bun test --watch           # Watch mode
bun test --coverage        # Generate coverage reports

# Test keystore functionality
bun test tests/keystore-import.test.ts         # Keystore import/export
bun test tests/wasm-keystore-import.test.ts    # WASM keystore functions
bun test tests/cli-keystore-decryption.test.ts # CLI format compatibility

# Run a single test file
bun test path/to/test.ts

# Run test scripts
./scripts/test/run-all-tests.sh   # Categorized test suites
./scripts/test/run-tests.sh       # Exclude import issues
./scripts/test-dkg-ui.sh          # Validate DKG UI
```

### Production Build

```bash
# Build for production
bun run build

# Build for specific browsers
bun run build:firefox
bun run build:edge

# Create extension ZIP
bun run zip

# Utility scripts
./scripts/build/fix-all-syntax-errors.sh  # Fix syntax errors
./scripts/build/remove-debug-logs.sh      # Clean debug logs
```

### Performance and Benchmarking

```bash
# Performance monitoring
bun scripts/performance.ts   # Build performance tracking
bun scripts/benchmark.ts     # Runtime benchmarking
```

## Development Workflow

### Browser Extension
1. Start the development server: `bun run dev`
2. Load the extension into Chrome/Edge from the `dist/` directory
3. Monitor logs using the browser's developer console:
   - Background: chrome://extensions → Service Worker "Inspect"
   - Popup: Right-click popup → Inspect
   - Offscreen: Check background console for offscreen logs
4. For changes to the Rust code, the dev server auto-rebuilds WASM

### Native Desktop Application
1. Build and run: `cd apps/native-node && cargo run`
2. The native app shares the CLI node's core functionality via library imports
3. UI is built with Slint framework for native performance
4. Uses the same WebSocket/WebRTC infrastructure as CLI and browser extension
5. Keystore files are compatible between all three applications

## Message Flow Architecture

```
Popup → Background → Offscreen → WebRTC Peers
  ↑         ↓           ↓
  └─────────┴───────────┴──── WebSocket Server
```

All messages are routed through the background service worker, which acts as the central coordinator. The message types are defined in `src/types/messages.ts` and follow a strict pattern-based routing system.

## Test Organization

```
tests/
├── components/          # UI component tests
├── config/             # Configuration tests
├── entrypoints/        # Extension-specific tests
│   ├── background/     # Service worker tests
│   └── offscreen/      # WebRTC and FROST tests
├── integration/        # End-to-end integration tests
├── services/           # Service layer tests
├── __mocks__/          # Test mocks and stubs
└── legacy/             # Legacy test files (reference only)
```

Test coverage is automatically generated in `coverage/index.html` when running with `--coverage`.

## Configuration Files

- `bunfig.toml` - Bun test and coverage settings
- `wxt.config.ts` - Extension manifest and build configuration
- `tsconfig.json` - TypeScript configuration
- `tailwind.config.js` - Styling configuration
- `flake.nix` - Nix dependency management

## WebSocket Server

- Default: `wss://auto-life.tech`
- Local development server available in `webrtc-signal-server/`
- Cloudflare Worker implementation in `webrtc-signal-server-cloudflare-worker/`

## Multi-Chain Configuration

Chain support is configured in `src/config/chains.ts`. The extension provides a unified wallet interface across:
- Ethereum (secp256k1 curve)
- Solana (ed25519 curve)

## Keystore Import/Export

The extension supports importing and exporting FROST keystore data for interoperability with CLI nodes:

### Import Keystore from CLI
- Use the "Import Keystore from CLI" button in the popup UI
- Supports both encrypted (.dat) and unencrypted (.json) CLI keystore files
- Automatically converts CLI format to extension-compatible format
- Maintains participant index mapping and session metadata

### Export Keystore for Backup
- Use the "Export Keystore for Backup" button after DKG completion
- Exports in CLI-compatible JSON format for cross-platform use
- Includes both core CLI fields and extension compatibility fields
- Supports both secp256k1 (Ethereum) and ed25519 (Solana) curves

### Format Compatibility
- **CLI Format**: Uses Argon2id key derivation with AES-256-GCM encryption
- **Extension Format**: Uses PBKDF2-SHA256 with AES-256-GCM for browser compatibility
- **Automatic Conversion**: WASM handles format detection and bidirectional conversion
- **Participant Indexing**: Maintains consistent 1-based indexing across platforms

## Troubleshooting

### Common Issues

- **WXT Dev Server Issues**: Sometimes the WXT dev server doesn't properly reload after changes. Try stopping and restarting the server.
  
- **Offscreen Document Problems**: If functionality depending on the offscreen document isn't working, check browser console for errors and ensure the document is loaded. Use the "Create Offscreen" button in the popup for debugging.
  
- **WebSocket Connection Errors**: These can occur if the signaling server (`wss://auto-life.tech`) is not reachable. Check connection status in the background service worker logs.

- **"Receiving end does not exist" errors**: Usually indicates the offscreen document hasn't been created yet or has crashed.

- **Keystore Import Issues**: Ensure the CLI keystore file format is correct. Check console logs for specific parsing errors. Verify that the CLI and extension are using compatible FROST versions.

- **Signing Failures**: If signing fails after importing a CLI keystore, verify that all participants are using the same key material by comparing exported keystores bit-by-bit.

### Debugging Tips

- Use the browser's developer tools to debug different extension contexts
- For WebAssembly debugging, use `console.log` calls from the JavaScript side
- Enable verbose logging by checking console output in all contexts
- For keystore debugging, export keystores from both CLI and extension to compare data structures
- Use the test suite to validate keystore functionality: `bun test tests/keystore-*.test.ts`
- Check test coverage reports in `coverage/index.html`

## Technology Stack

- **Runtime**: Bun (fast JavaScript runtime) for web, Rust for native
- **Build System**: Bun + wasm-pack + WXT for web, Cargo for native
- **UI Frameworks**: 
  - Browser Extension: Svelte 5 with TailwindCSS
  - Native Desktop: Slint UI framework (Rust-native GUI)
  - CLI: Ratatui for terminal UI
- **Extension Framework**: WXT (Web Extension Tools)
- **Cryptography**: FROST threshold signatures (implemented in Rust)
- **P2P Communication**: WebRTC with WebSocket signaling
- **Blockchain Libraries**: viem (Ethereum interactions), ethers-rs, solana-sdk
- **Storage**: Browser extension storage API with AES-256-GCM encryption
- **Key Derivation**: PBKDF2-SHA256 (extension), Argon2id compatibility (CLI import)
- **Testing**: Bun test runner with comprehensive test coverage
- **Development Environment**: NixOS with Nix flake for dependencies
- **Code Sharing**: CLI node exposes lib.rs for reuse in native desktop app

## AI Development Cycle

The project uses a memory bank system for AI assistance. When working with AI tools:

1. **Session Initialization**: Review existing knowledge base and project context
2. **Context Preservation**: Capture architectural decisions and implementation progress
3. **Cross-Session Continuity**: Document outcomes and update task complexity
4. **Project Lifecycle**: Use structured project phases and requirement tracking

Memory categories:
- `architecture` - Core system design decisions
- `implementation` - Technical details and code insights
- `debugging` - Problem resolution patterns
- `performance` - Optimization discoveries
- `integration` - Cross-component interactions
- `testing` - Test strategies and validation

Working directory: `/home/freeman.xiong/Documents/github/hecoinfo/mpc-wallet`