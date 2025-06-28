# MPC Wallet Chrome Extension

A Multi-Party Computation (MPC) wallet browser extension built with WXT, Svelte, and Rust/WebAssembly. This extension enables secure distributed key generation and signing operations across multiple parties using the FROST (Flexible Round-Optimized Schnorr Threshold) signature scheme.

## 🚀 Quick Start

```bash
# Install dependencies
bun install

# Build WASM modules
bun run build:wasm

# Start development server
bun run dev

# Build for production
bun run build
```

## 📁 Project Structure

```
mpc-wallet/
├── src/                    # Source code
│   ├── entrypoints/       # Extension entry points
│   │   ├── background/    # Service worker
│   │   ├── content/       # Content scripts
│   │   ├── offscreen/     # Offscreen document
│   │   └── popup/         # Extension popup UI
│   ├── components/        # Svelte UI components
│   ├── services/          # Business logic
│   ├── types/             # TypeScript definitions
│   └── lib.rs            # Rust/WASM implementation
├── docs/                  # Documentation
│   ├── architecture/      # Technical architecture
│   ├── implementation/    # Implementation details
│   └── testing/          # Testing documentation
├── scripts/              # Utility scripts
│   ├── build/            # Build scripts
│   └── test/             # Test scripts
├── tests/                # Test suites
├── test-data/            # Test fixtures and data
└── test-fixtures/        # Manual test files
```

## 🏗️ Architecture Overview

The extension follows Chrome Extension Manifest V3 architecture with four main contexts:

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Popup Page    │    │ Background Page │    │ Offscreen Page  │
│                 │    │                 │    │                 │
│ - UI Components │    │ - Service Worker│    │ - WebRTC Manager│
│ - State Display │◄──►│ - Message Router│◄──►│ - FROST DKG     │
│ - User Actions  │    │ - WebSocket     │    │ - Crypto Ops    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌─────────────────┐
                    │ Content Script  │
                    │                 │
                    │ - Web3 Provider │◄── Web Page
                    │ - JSON-RPC Proxy│    (window.ethereum)
                    └─────────────────┘
```

## ✨ Features

- **FROST MPC Protocol**: Secure threshold signatures using FROST
- **Multi-Chain Support**: Ethereum (secp256k1) and Solana (ed25519)
- **Distributed Key Generation**: Generate keys across multiple parties
- **WebRTC P2P Communication**: Direct peer-to-peer connections
- **CLI Compatibility**: Import/export keystores compatible with CLI nodes
- **Web3 Integration**: EIP-1193 compatible provider

## 🛠️ Development

### Prerequisites

- [Bun](https://bun.sh/) runtime
- Chrome/Chromium browser
- Rust toolchain with `wasm-pack`

### Building from Source

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd mpc-wallet
   ```

2. **Install dependencies**
   ```bash
   bun install
   ```

3. **Build WASM modules**
   ```bash
   bun run build:wasm
   ```

4. **Start development server**
   ```bash
   bun run dev
   ```

5. **Load extension in Chrome**
   - Navigate to `chrome://extensions/`
   - Enable "Developer mode"
   - Click "Load unpacked"
   - Select the `dist` folder

### Available Scripts

- `bun run dev` - Start development server with hot reload
- `bun run build` - Build production extension
- `bun run build:wasm` - Build Rust/WASM modules
- `bun run test` - Run test suite
- `bun run check` - Run Svelte type checking

## 📖 Documentation

- [Development Guide](./docs/DEVELOPMENT.md)
- [Architecture Overview](./docs/architecture/)
- [Testing Guide](./docs/testing/TESTING.md)
- [CLI Keystore Format](./docs/cli-keystore-format.md)
- [DKG Test Guide](./docs/DKG_TEST_GUIDE.md)

## 🧪 Testing

```bash
# Run all tests
bun test

# Run specific test suites
bun test services/
bun test components/
bun test webrtc

# Run with coverage
bun test --coverage
```

## 🔐 Security

- **No Single Point of Failure**: Keys are distributed using MPC
- **Threshold Signatures**: Requires t-of-n participants to sign
- **Secure Communication**: WebRTC encrypted channels
- **Isolated Contexts**: Crypto operations in offscreen document

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

Apache-2.0 License - see LICENSE file for details

## 🙏 Acknowledgments

- Built with [WXT](https://wxt.dev/) - Next-gen Web Extension Framework
- FROST implementation using [frost-core](https://github.com/ZcashFoundation/frost)
- UI components with [Svelte](https://svelte.dev/)

## 📞 Support

- Create an issue for bug reports or feature requests
- Check [existing issues](https://github.com/your-repo/issues) before creating new ones
- See [CLAUDE.md](./CLAUDE.md) for AI assistant guidance