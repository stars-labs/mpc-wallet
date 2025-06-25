# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

MPC Wallet is a browser extension that implements Multi-Party Computation for blockchain wallets using the FROST (Flexible Round-Optimized Schnorr Threshold) signature scheme. It enables distributed key generation and signing operations where no single party holds the complete private key. The extension supports both Ethereum (secp256k1) and Solana (ed25519) blockchains.

## Architecture

The extension follows Chrome Extension Manifest V3 architecture with four main contexts:

1. **Background Service Worker** (`src/entrypoints/background/`): Central message router managing WebSocket connections to a signaling server and coordinating communication between components.
   
2. **Popup Page** (`src/entrypoints/popup/`): User interface for wallet operations built with Svelte.
   
3. **Offscreen Document** (`src/entrypoints/offscreen/`): Handles WebRTC connections for peer-to-peer communication and cryptographic operations using Rust/WebAssembly.
   
4. **Content Script** (`src/entrypoints/content/`): Injects wallet provider API into web pages.

### Key Components

- **WebRTC System** (`src/entrypoints/offscreen/webrtc.ts`): Manages peer-to-peer connections and coordinates the MPC protocol.
  
- **Rust/WebAssembly Core** (`src/lib.rs`): Implements the cryptographic operations for the FROST protocol, including keystore import/export functionality for CLI compatibility.
  
- **Message System** (`src/types/messages.ts`): Strongly-typed messages for communication between extension components.

- **Keystore Service** (`src/services/keystoreService.ts`): Manages secure storage of FROST key shares with encryption and backup capabilities.

- **DKG Manager** (`src/entrypoints/offscreen/dkgManager.ts`): Handles the complete Distributed Key Generation lifecycle across multiple rounds.

## Build and Development Commands

### Setup and Installation

```bash
# Install dependencies
bun install

# Build WebAssembly modules from Rust code
bun run build:wasm
```

### Development

```bash
# Start development server with hot reloading
bun run dev

# Run development for specific browsers
bun run dev:firefox
bun run dev:edge

# Clean build artifacts
bun run clean
```

### Testing

```bash
# Run all tests
bun test

# Run specific test suites
bun run test:services    # Test service layer
bun run test:webrtc      # Test WebRTC simple tests
bun run test:webrtc:all  # Run all WebRTC tests
bun run test:dkg         # Test DKG functionality

# Test keystore functionality
bun test tests/keystore-import.test.ts     # Test keystore import/export
bun test tests/wasm-keystore-import.test.ts # Test WASM keystore functions
bun test tests/cli-keystore-decryption.test.ts # Test CLI format compatibility

# Run a single test file
bun test path/to/test.ts
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
```

### Type Checking

```bash
# Run Svelte type checking
bun run check
```

## Development Workflow

1. Start the development server: `bun run dev`
2. Load the extension into Chrome/Edge from the `dist/` directory
3. Monitor logs using the browser's developer console
4. For changes to the Rust code, rebuild WebAssembly: `bun run build:wasm`

## Message Flow Architecture

```
Popup → Background → Offscreen → WebRTC Peers
  ↑         ↓           ↓
  └─────────┴───────────┴──── WebSocket Server
```

All messages are routed through the background service worker, which acts as the central coordinator. The message types are defined in `src/types/messages.ts` and follow a strict pattern-based routing system.

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

- Use the browser's developer tools to debug different extension contexts:
  - Background: chrome://extensions → Service Worker "Inspect"
  - Popup: Right-click popup → Inspect
  - Offscreen: Check background console for offscreen logs
- For WebAssembly debugging, use `console.log` calls from the JavaScript side
- Enable verbose logging by checking console output in all contexts
- For keystore debugging, export keystores from both CLI and extension to compare data structures
- Use the test suite to validate keystore functionality: `bun test tests/keystore-*.test.ts`

## Technology Stack

- **Build System**: Bun + wasm-pack
- **UI Framework**: Svelte 5
- **Extension Framework**: WXT (Web Extension Tools)
- **Cryptography**: FROST threshold signatures (implemented in Rust)
- **P2P Communication**: WebRTC
- **Signaling**: WebSocket
- **Blockchain Libraries**: viem (Ethereum)
- **Storage**: Browser extension storage API with AES-256-GCM encryption
- **Key Derivation**: PBKDF2-SHA256 (extension), Argon2id compatibility (CLI import)
- **Testing**: Bun test runner with comprehensive keystore test coverage

## File Structure

```
src/
├── entrypoints/
│   ├── background/          # Background service worker
│   ├── popup/              # Popup UI (Svelte)
│   ├── offscreen/          # WebRTC and WASM integration
│   └── content/            # Content script for web page injection
├── services/
│   └── keystoreService.ts  # Keystore management and encryption
├── types/
│   ├── messages.ts         # Message type definitions
│   ├── keystore.ts         # Keystore data structures
│   └── dkg.ts             # DKG state management
├── lib.rs                 # Rust/WASM FROST implementation
└── tests/                 # Test suite including keystore tests
```