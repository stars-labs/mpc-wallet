# Browser Extension Documentation

## Overview

The MPC Wallet Browser Extension is a Manifest V3 Chrome/Firefox extension that provides secure multi-party computation wallet functionality directly in the browser. Built with TypeScript, Svelte, and WebAssembly, it enables threshold signatures without any single party having access to complete private keys.

## Documentation Structure

### Core Documentation
- [Architecture](architecture/) - Extension architecture, message passing, state management
- [API Reference](api/) - Extension APIs, Web3 provider, message protocols
- [User Interface](ui/) - Popup interface, DKG flow, transaction signing
- [Development Guide](guides/) - Setup, development workflow, debugging

## Quick Start

### Prerequisites
- Node.js 18+ or Bun runtime
- Chrome/Firefox browser
- Rust toolchain with wasm-pack

### Installation

```bash
# Navigate to extension directory
cd apps/browser-extension

# Install dependencies
bun install

# Build WASM module (from root)
bun run build:wasm

# Start development server
bun run dev
```

### Loading the Extension

#### Chrome
1. Navigate to `chrome://extensions/`
2. Enable "Developer mode"
3. Click "Load unpacked"
4. Select `.output/chrome-mv3` directory

#### Firefox
1. Navigate to `about:debugging`
2. Click "This Firefox"
3. Click "Load Temporary Add-on"
4. Select `.output/firefox-mv2/manifest.json`

## Architecture Overview

```
┌──────────────────────────────────────────────────┐
│                   Web Page                        │
│            (dApp with window.ethereum)            │
└────────────────────┬─────────────────────────────┘
                     │
              Content Script
         (Provider injection & RPC)
                     │
┌────────────────────┼─────────────────────────────┐
│   Extension        │                             │
├────────────────────┼─────────────────────────────┤
│  Popup UI     Background Worker    Offscreen     │
│  (Svelte)    (Service Worker)      Document      │
│                                                   │
│  • Wallet UI  • Message Router   • WebRTC P2P    │
│  • Settings   • State Manager    • FROST WASM    │
│  • Accounts   • WebSocket Client • Crypto Ops    │
└───────────────────────────────────────────────────┘
```

## Key Features

### Security
- **Threshold Signatures**: t-of-n participants required for signing
- **No Single Point of Failure**: Private keys never exist in complete form
- **Encrypted Storage**: AES-256-GCM encryption for local storage
- **Secure Communication**: WebRTC with DTLS encryption

### Functionality
- **Multi-Chain Support**: Ethereum (secp256k1) and Solana (ed25519)
- **Web3 Provider**: EIP-1193 compatible provider for dApps
- **P2P Communication**: Direct WebRTC connections between participants
- **Keystore Management**: Import/export compatible with CLI wallet

### User Experience
- **Simple Setup**: One-click wallet creation
- **Visual DKG Flow**: Guided distributed key generation
- **Transaction Preview**: Clear transaction details before signing
- **Account Management**: Multiple wallets and accounts

## Development

### Project Structure

```
browser-extension/
├── src/
│   ├── entrypoints/      # Extension entry points
│   │   ├── background/   # Service worker
│   │   ├── content/      # Content scripts
│   │   ├── offscreen/    # Offscreen document
│   │   └── popup/        # Popup UI
│   ├── components/       # Svelte components
│   ├── services/         # Business logic
│   ├── types/           # TypeScript definitions
│   └── utils/           # Utility functions
├── public/              # Static assets
├── wxt.config.ts        # WXT framework config
└── package.json         # Dependencies
```

### Key Technologies

- **WXT**: Extension framework for cross-browser development
- **Svelte**: Reactive UI framework for popup interface
- **TypeScript**: Type-safe JavaScript
- **WebAssembly**: High-performance cryptographic operations
- **WebRTC**: Peer-to-peer communication

### Testing

```bash
# Run unit tests
bun test

# Run E2E tests
bun run test:e2e

# Test specific component
bun test AccountManager
```

### Building for Production

```bash
# Build for Chrome
bun run build:chrome

# Build for Firefox
bun run build:firefox

# Build for all browsers
bun run build
```

## API Integration

### For dApp Developers

```javascript
// Check if MPC Wallet is installed
if (window.ethereum && window.ethereum.isMPCWallet) {
  // Request account access
  const accounts = await window.ethereum.request({
    method: 'eth_requestAccounts'
  });
  
  // Send transaction
  const txHash = await window.ethereum.request({
    method: 'eth_sendTransaction',
    params: [{
      from: accounts[0],
      to: '0x...',
      value: '0x...'
    }]
  });
}
```

### Extension APIs

```typescript
// Send message to background
chrome.runtime.sendMessage({
  type: 'CREATE_WALLET',
  payload: {
    name: 'My Wallet',
    threshold: 2,
    participants: 3
  }
});

// Listen for updates
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message.type === 'WALLET_CREATED') {
    console.log('Wallet created:', message.wallet);
  }
});
```

## Troubleshooting

### Common Issues

#### Extension not loading
- Ensure developer mode is enabled
- Check console for errors
- Verify WASM module is built

#### WebRTC connection failures
- Check firewall settings
- Verify STUN/TURN servers
- Enable WebRTC debugging in chrome://webrtc-internals

#### Transaction signing errors
- Verify all participants are online
- Check threshold requirements
- Review gas settings

## Resources

- [Main Project Documentation](../../../docs/README.md)
- [API Reference](api/)
- [Architecture Details](architecture/)
- [UI Documentation](ui/)
- [Chrome Extension Docs](https://developer.chrome.com/docs/extensions/mv3/)
- [WXT Framework](https://wxt.dev/)

## Support

For issues and questions:
- [GitHub Issues](https://github.com/your-org/mpc-wallet/issues)
- [Discord Community](https://discord.gg/mpc-wallet)
- [Documentation](../../../docs/)

---

[← Back to Apps](../../README.md) | [→ Architecture](architecture/)