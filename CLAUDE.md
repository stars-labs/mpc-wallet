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
  
- **Rust/WebAssembly Core** (`src/lib.rs`): Implements the cryptographic operations for the FROST protocol.
  
- **Message System** (`src/types/messages.ts`): Strongly-typed messages for communication between extension components.

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

## Troubleshooting

### Common Issues

- **WXT Dev Server Issues**: Sometimes the WXT dev server doesn't properly reload after changes. Try stopping and restarting the server.
  
- **Offscreen Document Problems**: If functionality depending on the offscreen document isn't working, check browser console for errors and ensure the document is loaded. Use the "Create Offscreen" button in the popup for debugging.
  
- **WebSocket Connection Errors**: These can occur if the signaling server (`wss://auto-life.tech`) is not reachable. Check connection status in the background service worker logs.

- **"Receiving end does not exist" errors**: Usually indicates the offscreen document hasn't been created yet or has crashed.

### Debugging Tips

- Use the browser's developer tools to debug different extension contexts:
  - Background: chrome://extensions → Service Worker "Inspect"
  - Popup: Right-click popup → Inspect
  - Offscreen: Check background console for offscreen logs
- For WebAssembly debugging, use `console.log` calls from the JavaScript side
- Enable verbose logging by checking console output in all contexts

## Technology Stack

- **Build System**: Bun + wasm-pack
- **UI Framework**: Svelte 5
- **Extension Framework**: WXT (Web Extension Tools)
- **Cryptography**: FROST threshold signatures (implemented in Rust)
- **P2P Communication**: WebRTC
- **Signaling**: WebSocket
- **Blockchain Libraries**: viem (Ethereum)
- **Testing**: Bun test runner