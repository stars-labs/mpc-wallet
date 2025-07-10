# FROST MPC CLI Node Documentation

This directory contains the documentation for the frost-mpc-cli-node crate, which implements threshold signatures using the FROST protocol for both Solana (Ed25519) and Ethereum (secp256k1) blockchains.

## Documentation Structure

### User Guides
- [1. Getting Started](user_guides/01_getting_started.md) - How to use the CLI application
- [2. Keystore Overview](user_guides/02_keystore_overview.md) - Introduction to the keystore functionality
- [3. Keystore Usage](user_guides/03_keystore_usage.md) - Detailed guide for using the keystore features

### Architecture Documentation
- [1. Keystore Design](architecture/01_keystore_design.md) - Technical design of the keystore system

### Protocol Documentation
- [1. WebRTC Signaling](protocol/01_webrtc_signaling.md) - WebRTC signaling protocol details

## Key Features

The FROST MPC CLI node implements threshold signatures with distributed key generation. It uses:

1. WebRTC for device-to-device communication in a mesh network
2. Terminal UI for user interaction
3. Supports both Solana (Ed25519) and Ethereum (secp256k1) keys
4. Signaling server for WebRTC connection establishment

The protocol flow involves:
- Node registration and device discovery
- Session negotiation and WebRTC mesh formation
- Distributed Key Generation (DKG)
- Threshold signing

## Build and Run

```bash
# Build the CLI node
cargo build --release -p frost-mpc-cli-node

# Run the CLI node
cargo run --release -p frost-mpc-cli-node -- --device-id <your-id> --curve <secp256k1|ed25519>
```

For detailed usage instructions, please refer to the [Getting Started Guide](user_guides/01_getting_started.md).