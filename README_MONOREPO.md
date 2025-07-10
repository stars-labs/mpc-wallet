# MPC Wallet - Multi-Platform Threshold Wallet

A comprehensive Multi-Party Computation (MPC) wallet ecosystem supporting browser extensions, CLI tools, and native desktop applications. Built with FROST (Flexible Round-Optimized Schnorr Threshold) signatures for secure distributed key management.

## 🏗️ Monorepo Structure

```
mpc-wallet/
├── apps/                      # Applications
│   ├── browser-extension/     # Chrome/Firefox extension
│   ├── cli-node/             # Terminal-based MPC node
│   ├── native-node/          # Desktop application
│   └── signal-server/        # WebRTC signaling servers
├── packages/@mpc-wallet/      # Shared packages
│   ├── frost-core/           # Core FROST cryptography
│   ├── core-wasm/            # WebAssembly bindings
│   └── types/                # TypeScript definitions
└── scripts/                   # Build and test scripts
```

## 🚀 Quick Start

### Prerequisites

```bash
# Install Bun (JavaScript runtime)
curl -fsSL https://bun.sh/install | bash

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Or use Nix (recommended)
nix develop
```

### Build Everything

```bash
# Clone the repository
git clone https://github.com/stars-labs/mpc-wallet.git
cd mpc-wallet

# Install dependencies and build all packages
./scripts/build-all.sh

# Or build individually:
bun install                    # Install JS dependencies
bun run build:wasm            # Build WASM package
bun run build                 # Build browser extension
cargo build                   # Build Rust applications
```

## 📱 Applications

### Browser Extension

Multi-chain Web3 wallet with FROST MPC support:

```bash
cd apps/browser-extension
bun run dev                    # Development mode
bun run build                  # Production build
```

Features:
- ✅ Ethereum & Solana support
- ✅ Distributed key generation
- ✅ Threshold signing (t-of-n)
- ✅ WalletConnect integration
- ✅ Hardware wallet compatible

### CLI Node

Terminal-based MPC participant node:

```bash
cargo run --bin cli_node -- --device-id Device-001
```

Features:
- ✅ TUI interface with real-time updates
- ✅ Offline transaction signing
- ✅ Keystore import/export
- ✅ WebRTC mesh networking
- ✅ Multi-blockchain support

### Native Desktop App

Cross-platform desktop application with modern UI:

```bash
cargo run --bin mpc-wallet-native
```

Features:
- ✅ Native performance with Slint UI
- ✅ Real-time session management
- ✅ Visual DKG progress tracking
- ✅ Cross-platform (Windows/macOS/Linux)
- 🚧 Full feature parity with CLI (in progress)

## 🔧 Development

### Project Structure

Each application follows a similar architecture:
- **State Management**: Centralized app state with async updates
- **Message Passing**: Command-based internal communication
- **Networking**: WebSocket for signaling, WebRTC for P2P
- **Cryptography**: Shared FROST implementation

### Working with the Monorepo

```bash
# Run tests for everything
./scripts/test-all.sh

# Clean all build artifacts
./scripts/clean-all.sh

# Format code
cargo fmt --all
bun run format

# Lint code
cargo clippy --all
bun run lint
```

### Key Concepts

1. **FROST Protocol**: Flexible Round-Optimized Schnorr Threshold signatures
2. **MPC**: Multi-Party Computation for distributed key management
3. **WebRTC**: Peer-to-peer communication between participants
4. **Threshold Signatures**: Require t-of-n participants to sign

## 🛠️ Architecture

### Communication Flow

```
Browser Extension          CLI Node              Native Desktop
       |                      |                      |
       |------WebSocket-------|------WebSocket------|
                              |
                        Signal Server
                              |
       |------WebRTC----------|------WebRTC---------|
```

### Security Model

- **No Single Point of Failure**: Keys are distributed across participants
- **Threshold Security**: Requires multiple participants to sign
- **End-to-End Encrypted**: All communications are encrypted
- **Open Source**: Fully auditable codebase

## 📚 Documentation

- [Monorepo Architecture](./docs/MONOREPO_ARCHITECTURE.md)
- [Native App Guide](./docs/NATIVE_APP_GUIDE.md)
- [CLI Node Documentation](./apps/cli-node/docs/index.md)
- [Browser Extension Guide](./apps/browser-extension/README.md)
- [FROST Protocol Details](./packages/@mpc-wallet/frost-core/README.md)

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

### Code Style

- Rust: Follow standard Rust conventions
- TypeScript: ESLint + Prettier configuration
- Commits: Conventional commits format

## 📊 Project Status

### Current Release: v2.0

- ✅ Monorepo migration complete
- ✅ Native desktop app MVP
- ✅ Shared cryptographic libraries
- ✅ Unified type system

### Roadmap

**v2.1** (Q1 2025)
- [ ] Full WebRTC in native app
- [ ] Complete feature parity
- [ ] Mobile app development

**v2.2** (Q2 2025)
- [ ] Hardware wallet support
- [ ] Advanced key management
- [ ] Enterprise features

## 🔗 Resources

- **Website**: [Coming Soon]
- **Documentation**: [GitHub Wiki](https://github.com/stars-labs/mpc-wallet/wiki)
- **Issues**: [GitHub Issues](https://github.com/stars-labs/mpc-wallet/issues)
- **Discussions**: [GitHub Discussions](https://github.com/stars-labs/mpc-wallet/discussions)

## 📄 License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- FROST protocol implementation from [ZCash Foundation](https://github.com/ZcashFoundation/frost)
- Slint UI framework from [Slint.dev](https://slint.dev)
- WebRTC implementation using [webrtc-rs](https://github.com/webrtc-rs/webrtc)

---

Built with ❤️ by the Stars Labs team