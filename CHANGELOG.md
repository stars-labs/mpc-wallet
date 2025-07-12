# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.0] - 2025-07-10

### Added
- **Monorepo Structure**: Complete project reorganization into apps/ and packages/ directories
- **Native Desktop Application**: Cross-platform desktop app built with Slint UI framework
  - Real-time WebSocket connections
  - Session management interface
  - Visual DKG progress tracking
  - Modern tabbed UI (Session, DKG, Signing, Logs)
- **Shared Packages**:
  - `@mpc-wallet/frost-core`: Shared FROST cryptographic library
  - `@mpc-wallet/core-wasm`: WebAssembly bindings for browser
  - `@mpc-wallet/types`: Centralized TypeScript type definitions
- **Enhanced Build System**:
  - Unified Cargo workspace for Rust projects
  - Bun workspace for TypeScript packages
  - Monorepo-wide build scripts
- **Documentation**:
  - Comprehensive monorepo architecture guide
  - Native desktop application user guide
  - Updated README for multi-platform support

### Changed
- **Browser Extension**: Moved from root to `apps/browser-extension/`
- **Import Paths**: All TypeScript imports now use `@mpc-wallet/types`
- **Build Commands**: Must be run from project root
- **Dependencies**: Updated Nix flake with GUI support (Wayland, X11)

### Fixed
- Code duplication between CLI and WASM implementations
- Type inconsistencies across different apps
- Build complexity with multiple independent projects

### Breaking Changes
- File paths have changed due to monorepo structure
- Import statements need updating to use @mpc-wallet/types
- Build commands must be run from repository root
- Browser extension location moved

## [1.0.0] - 2025-07-09

### Added
- Initial release with browser extension and CLI node
- FROST threshold signature implementation
- WebRTC peer-to-peer networking
- Multi-blockchain support (Ethereum, Solana)
- Keystore import/export functionality