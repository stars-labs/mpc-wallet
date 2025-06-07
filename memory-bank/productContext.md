# MPC Wallet Chrome Extension - Product Context

## Project Overview
A Multi-Party Computation (MPC) wallet browser extension built with WXT, Svelte, and Rust/WebAssembly. This extension enables secure distributed key generation and signing operations across multiple parties using WebRTC for peer-to-peer communication.

## Core Features
- **Multi-Chain Support**: Ethereum (secp256k1) and Solana (ed25519) networks
- **Distributed Key Generation**: FROST-based DKG for secure key sharing
- **WebRTC P2P Communication**: Direct peer-to-peer communication for MPC sessions
- **Chrome Extension Integration**: Browser-native wallet with web page APIs
- **Secure Message Routing**: Type-safe message system across extension contexts

## Architecture Components

### 1. Background Page (Service Worker)
- Central message router for all communication
- WebSocket client management for signaling server
- Account and network services
- Offscreen document lifecycle management
- RPC request handling for blockchain operations

### 2. Popup Page (UI)
- User interface for wallet operations
- Display connection status and peer information
- Session management UI for MPC operations
- Crypto operations (signing, address generation)

### 3. Offscreen Page (WebRTC Handler)
- WebRTC connection management
- P2P communication handling
- MPC session coordination
- DOM-dependent operations

### 4. Content Script (Web Integration)
- Injects wallet API into web pages
- Provides `window.ethereum` compatibility
- Proxies JSON-RPC requests to background script
- Manages web page wallet interactions

## Technology Stack
- **Frontend**: Svelte, TypeScript, Tailwind CSS
- **Build System**: WXT (Web Extension Tools)
- **Crypto**: Rust/WebAssembly with FROST implementation
- **Communication**: WebRTC for P2P, WebSocket for signaling
- **Development**: NixOS, Nix flake for dependency management

## Target Users
- Developers and organizations requiring secure multi-party wallets
- Users needing shared custody solutions
- Teams implementing threshold signatures for enhanced security

## Value Proposition
- Eliminates single points of failure in key management
- Enables secure collaborative transaction signing
- Provides browser-native MPC wallet experience
- Offers enterprise-grade security with user-friendly interface

[2025-06-07 18:27:17] - Initial product context establishment