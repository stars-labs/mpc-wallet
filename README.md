# MPC Wallet

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![TypeScript](https://img.shields.io/badge/typescript-%23007ACC.svg?style=flat&logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![WebRTC](https://img.shields.io/badge/WebRTC-333333?style=flat&logo=webrtc&logoColor=white)](https://webrtc.org/)

A production-ready Multi-Party Computation (MPC) wallet implementing FROST (Flexible Round-Optimized Schnorr Threshold) signatures for secure distributed key management across multiple platforms.

## Overview

MPC Wallet enables threshold signatures where private keys are split across multiple parties, requiring a minimum threshold to sign transactions. No single party ever has access to the complete private key, providing superior security for digital asset management.

### Key Features

- **Threshold Signatures**: Configurable t-of-n threshold signing
- **Multi-Platform**: Browser extension, desktop GUI, and terminal UI
- **Multi-Chain Support**: Ethereum (secp256k1) and Solana (ed25519)
- **Peer-to-Peer**: Direct WebRTC connections between participants
- **Offline Mode**: Air-gapped operations for maximum security
- **Production Ready**: Comprehensive testing and security audits

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/your-org/mpc-wallet.git
cd mpc-wallet

# Install dependencies
bun install

# Build WASM modules
bun run build:wasm

# Start development
bun run dev
```

### Basic Usage

#### Browser Extension

1. Build and load the extension:
```bash
cd apps/browser-extension
bun run dev
```

2. Load unpacked extension in Chrome from `.output/chrome-mv3`

3. Create a wallet through the extension popup

#### Terminal UI

```bash
# Run the TUI application
cargo run -p tui-node -- --device-id Device-001

# Create a wallet (interactive)
> create my_wallet 2 3
```

#### Desktop Application

```bash
# Run the native desktop app
cargo run -p native-node
```

## Documentation

### 📚 Documentation Hub
- [Documentation Center](docs/README.md) - Main documentation hub with complete index
- [Technical Documentation](MPC_WALLET_TECHNICAL_DOCUMENTATION.md) - Comprehensive technical reference (100+ pages)
- [Contributing Guidelines](CONTRIBUTING.md) - How to contribute to the project

### 🏗️ Architecture & Design
- [Architecture Overview](docs/architecture/README.md) - System design and architectural decisions
- [Monorepo Architecture](docs/MONOREPO_ARCHITECTURE.md) - Monorepo structure and organization
- [Wallet-Centric Flow](docs/architecture/WALLET_CENTRIC_FLOW.md) - Wallet-first architecture design
- [Security Model](docs/security/README.md) - Security considerations and threat model

### 📖 Application Documentation

#### Browser Extension
- [Browser Extension Guide](apps/browser-extension/docs/README.md) - Complete browser extension documentation
- [Extension Architecture](apps/browser-extension/docs/architecture/) - Extension design patterns
- [UI Components](apps/browser-extension/docs/ui/README.md) - UI implementation and components
- [User Guides](apps/browser-extension/docs/guides/) - Step-by-step usage guides

#### Terminal UI (TUI)
- [TUI Documentation](apps/tui-node/docs/README.md) - Terminal UI comprehensive guide
- [TUI Architecture](apps/tui-node/docs/architecture/ARCHITECTURE.md) - System architecture
- [DKG Flows](apps/tui-node/docs/architecture/DKG_FLOWS.md) - Distributed key generation flows
- [User Guide](apps/tui-node/docs/guides/USER_GUIDE.md) - Complete user manual
- [Keystore Design](apps/tui-node/docs/architecture/01_keystore_design.md) - Keystore implementation
- [Protocol Specs](apps/tui-node/docs/protocol/) - WebRTC and keystore session protocols
- [UI Wireframes](apps/tui-node/docs/ui/) - Comprehensive UI specifications and wireframes
- [Offline Mode](apps/tui-node/docs/guides/offline-mode.md) - Air-gapped operation guide

#### Native Desktop Application
- [Native App Guide](apps/native-node/docs/README.md) - Desktop application documentation
- [Architecture](apps/native-node/docs/architecture/) - Native app architecture
- [UI Guide](apps/native-node/docs/ui/) - Slint UI framework documentation

#### Signal Server
- [Signal Server Guide](apps/signal-server/docs/README.md) - WebRTC signaling server
- [Deployment](apps/signal-server/docs/deployment/cloudflare-deployment.md) - Cloudflare deployment guide
- [Architecture](apps/signal-server/docs/architecture/) - Server architecture and design

### 🔧 Development Resources
- [Development Guide](docs/development/README.md) - Setup and development workflow
- [API Reference](docs/api/README.md) - Complete API documentation
- [Testing Documentation](docs/testing/README.md) - Testing strategies and tools
  - [Test Coverage](docs/testing/COVERAGE.md) - Code coverage reports
  - [E2E Testing](docs/testing/E2E_TEST_SUMMARY.md) - End-to-end test documentation
  - [Running Tests](docs/testing/RUN_TEST_INSTRUCTIONS.md) - How to run test suites

### 🚀 Deployment & Operations
- [Deployment Guide](docs/deployment/README.md) - Production deployment instructions
- [Cloudflare Deployment](docs/deployment/CLOUDFLARE_DEPLOYMENT.md) - Deploy to Cloudflare Workers
- [TUI Deployment Guide](apps/tui-node/docs/DEPLOYMENT_GUIDE.md) - Deploy TUI application

### 🔍 Implementation Details
- [Implementation Docs](docs/implementation/) - Feature implementation details
  - [EIP-6963 Implementation](docs/implementation/EIP-6963-IMPLEMENTATION.md) - Wallet provider discovery
  - [Multi-Layer2 Support](docs/implementation/MULTI_LAYER2_SUPPORT.md) - Layer 2 chain support
  - [WebRTC Fix Summary](docs/implementation/WEBRTC_FIX_SUMMARY.md) - P2P connection fixes

### 🐛 Bug Fixes & Solutions
- [Bug Fix Documentation](docs/fixes/README.md) - Documented bug fixes and solutions
- [DKG Fixes](docs/fixes/DKG_FIX_SUMMARY.md) - Distributed key generation fixes
- [Session Discovery](docs/fixes/SESSION_DISCOVERY_FIX.md) - Session management fixes
- [Complete Fix Summary](docs/fixes/COMPLETE_FIX_SUMMARY.md) - All implemented fixes

### 📝 Additional Resources
- [Changelog](docs/CHANGELOG.md) - Version history and release notes
- [Documentation Status](DOCUMENTATION_STATUS.md) - Current state of documentation

## Project Structure

```
mpc-wallet/
├── apps/                         # Applications
│   ├── browser-extension/        # Chrome/Firefox extension
│   ├── native-node/             # Desktop GUI application
│   ├── tui-node/                # Terminal UI application
│   └── signal-server/           # WebRTC signaling server
│
├── packages/@mpc-wallet/         # Shared packages
│   ├── frost-core/              # FROST protocol implementation
│   ├── core-wasm/               # WebAssembly bindings
│   └── types/                   # TypeScript definitions
│
├── docs/                        # Documentation
├── scripts/                     # Build and utility scripts
└── tests/                       # Integration tests
```

## Technology Stack

### Core Technologies

- **Rust**: Core cryptographic implementation
- **TypeScript**: Browser extension and web components
- **WebAssembly**: Bridge between Rust and JavaScript
- **WebRTC**: Peer-to-peer communication
- **Svelte**: Browser extension UI
- **Slint**: Native desktop UI framework
- **Ratatui**: Terminal UI framework

### Cryptography

- **FROST**: Threshold signature scheme
- **secp256k1**: Ethereum signatures
- **ed25519**: Solana signatures
- **AES-256-GCM**: Encryption at rest
- **PBKDF2**: Key derivation

## Use Cases

### Individual Users
- Secure personal wallet with distributed backups
- Multi-device wallet control
- Enhanced security for high-value accounts

### Organizations
- Corporate treasury management
- Multi-signature custody solutions
- Distributed key management for exchanges
- Secure validator key management

### Developers
- Integration into existing applications
- Custom threshold signature implementations
- Research and development platform

## Security

The MPC Wallet has been designed with security as the primary concern:

- Private keys never exist in complete form
- All communication is end-to-end encrypted
- Comprehensive input validation and sanitization
- Regular security audits and updates

For detailed security information, see our [Security Documentation](docs/security/README.md).

## Performance

### Benchmarks

| Operation | Participants | Time | Network |
|-----------|-------------|------|---------|
| DKG | 3 | 1.2s | 45KB |
| Sign | 3 | 45ms | 15KB |
| Verify | 1 | 15ms | - |

### Scalability

- Supports up to 100 participants
- Horizontal scaling for signal servers
- Optimized for mobile and low-bandwidth environments

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details on:

- Code of Conduct
- Development setup
- Submitting pull requests
- Reporting issues
- Security vulnerabilities

## Support

### Community

- [Discord](https://discord.gg/mpc-wallet) - Join our community
- [GitHub Issues](https://github.com/your-org/mpc-wallet/issues) - Report bugs
- [Documentation](https://docs.mpc-wallet.io) - Full documentation

### Commercial Support

For enterprise support and custom development, contact: enterprise@mpc-wallet.io

## Roadmap

### Q1 2025
- [x] Browser extension MVP
- [x] Terminal UI application
- [x] Desktop application
- [ ] Mobile application (in progress)

### Q2 2025
- [ ] Hardware wallet integration
- [ ] Additional blockchain support
- [ ] Advanced recovery mechanisms
- [ ] Enterprise features

### Q3 2025
- [ ] Formal verification
- [ ] Performance optimizations
- [ ] Enhanced UI/UX
- [ ] Regulatory compliance features

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [FROST Paper](https://eprint.iacr.org/2020/852) by Komlo & Goldberg
- [ZCash Foundation](https://github.com/ZcashFoundation/frost) for FROST implementation
- [WebRTC Project](https://webrtc.org/) for P2P communication
- All our contributors and community members

## Citation

If you use this software in your research, please cite:

```bibtex
@software{mpc_wallet,
  title = {MPC Wallet: Multi-Party Computation Wallet},
  author = {MPC Wallet Team},
  year = {2025},
  url = {https://github.com/your-org/mpc-wallet}
}
```

---

**Built with ❤️ by the MPC Wallet Team**

*Secure. Distributed. Open Source.*