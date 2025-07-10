# Native Desktop Application Guide

## Overview

The MPC Wallet Native Node is a cross-platform desktop application that provides a graphical interface for FROST multi-party computation operations. Built with Slint UI framework, it offers native performance and a modern user experience.

## Features

- 🖥️ **Cross-Platform**: Runs on Windows, macOS, and Linux
- 🔄 **Real-time Updates**: Live status and logging
- 🔐 **Secure**: Same cryptography as CLI and browser extension
- 🌐 **Network Connected**: WebSocket and WebRTC support
- 📱 **Modern UI**: Clean, intuitive interface

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/stars-labs/mpc-wallet.git
cd mpc-wallet

# Build the native application
cargo build --release --bin mpc-wallet-native

# Run the application
cargo run --bin mpc-wallet-native
```

### Pre-built Binaries

Coming soon! Check the [releases page](https://github.com/stars-labs/mpc-wallet/releases) for pre-built binaries.

## User Interface

### Main Window Components

#### 1. Header Bar
- **Application Title**: "MPC Wallet Native Node"
- **Connection Status**: Real-time WebSocket connection indicator
- **Device ID**: Unique identifier for this device

#### 2. Tab Navigation
- **Session**: Create and join MPC sessions
- **DKG**: Distributed Key Generation status
- **Signing**: Transaction signing interface
- **Logs**: Real-time activity logs

### Session Tab

#### Device Information
- Displays your unique Device ID
- Shows WebSocket connection status
- "Connect to Server" button

#### Create New Session
- **Session ID**: Unique identifier for the session
- **Participants**: Total number of devices (2-10)
- **Threshold**: Minimum signers required (must be ≤ participants)
- **Create Session**: Initiates a new MPC session

#### Session Invites
- Lists pending session invitations
- Shows who invited you and session parameters
- Accept/Reject buttons for each invite

### DKG Tab

#### Progress Tracking
- Visual progress bars for each DKG round
- List of participants and their status
- Real-time updates as the protocol progresses

#### Generated Address
- Displays the blockchain address after successful DKG
- Copy button for easy sharing

### Signing Tab

#### Transaction Input
- Multi-line text area for transaction data
- Blockchain selector (Ethereum/Solana)
- "Initiate Signing" button

#### Signing Requests
- Lists pending signature requests
- Shows requester and transaction details
- Approve/Reject options

### Logs Tab

#### Activity Monitor
- Real-time log messages
- Scrollable history
- Debug information for troubleshooting

## Workflows

### Creating a Session

1. Click "Connect to Server" to establish WebSocket connection
2. Enter a unique Session ID
3. Set number of participants (e.g., 3)
4. Set threshold (e.g., 2 for 2-of-3)
5. Click "Create Session"
6. Share Session ID with other participants

### Joining a Session

1. Ensure WebSocket is connected
2. Wait for session invite to appear
3. Review session parameters
4. Click "Accept" to join
5. Wait for all participants to join

### Distributed Key Generation (DKG)

1. Once all participants have joined, go to DKG tab
2. Click "Start DKG" (session creator only)
3. Monitor progress through 3 rounds
4. Upon completion, see generated address

### Signing Transactions

1. Go to Signing tab
2. Enter transaction data
3. Select blockchain (Ethereum/Solana)
4. Click "Initiate Signing"
5. Other participants approve in their apps
6. Signature is generated when threshold is met

## Configuration

The application stores configuration in:
- **Linux**: `~/.config/mpc-wallet/native-node.toml`
- **macOS**: `~/Library/Application Support/mpc-wallet/native-node.toml`
- **Windows**: `%APPDATA%\mpc-wallet\native-node.toml`

### Configuration Options

```toml
websocket_url = "wss://auto-life.tech"
data_dir = "/path/to/data"
log_level = "info"
auto_connect = false
default_threshold = 2
default_participants = 3
```

## Keyboard Shortcuts

- **Ctrl+Q**: Quit application
- **Ctrl+Tab**: Switch tabs
- **Ctrl+C**: Copy selected text
- **Ctrl+V**: Paste text

## Troubleshooting

### Connection Issues

**Problem**: Cannot connect to WebSocket server
- Check internet connection
- Verify firewall settings
- Try alternative server URL in config

**Problem**: WebRTC connections failing
- Ensure UDP ports are not blocked
- Check NAT/firewall configuration
- Try using a TURN server

### Display Issues

**Problem**: Text appears blurry
- Check display scaling settings
- Try setting `SLINT_SCALE_FACTOR=1.0`

**Problem**: Window too small/large
- Resize window by dragging corners
- Adjust DPI settings in your OS

### Performance Issues

**Problem**: UI feels sluggish
- Close unnecessary applications
- Check CPU/memory usage
- Try software renderer: `SLINT_BACKEND=software`

## Security Considerations

1. **Private Keys**: Never stored on disk unencrypted
2. **Network**: All communications use TLS/WebSocket Secure
3. **Keystore**: Password-protected with strong encryption
4. **Sessions**: Ephemeral - not persisted between runs

## Development

### Building from Source

```bash
# Install dependencies
cargo build --bin mpc-wallet-native

# Run in development mode
RUST_LOG=debug cargo run --bin mpc-wallet-native

# Run tests
cargo test --bin mpc-wallet-native
```

### Architecture

The native app follows the same architecture as the CLI:
- Command-based message handling
- Shared state management
- Async operations with UI updates
- WebSocket/WebRTC for networking

### Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## Roadmap

### Near Term (v2.1)
- [ ] Full WebRTC implementation
- [ ] Complete DKG functionality
- [ ] Transaction signing
- [ ] Keystore import/export

### Medium Term (v2.2)
- [ ] Multiple wallet support
- [ ] Hardware wallet integration
- [ ] Batch signing
- [ ] QR code support

### Long Term (v3.0)
- [ ] Mobile companion app
- [ ] Cloud backup
- [ ] Enterprise features
- [ ] Plugin system

## Support

- **GitHub Issues**: [Report bugs or request features](https://github.com/stars-labs/mpc-wallet/issues)
- **Documentation**: [Full documentation](https://github.com/stars-labs/mpc-wallet/wiki)
- **Community**: Join our Discord/Telegram (coming soon)

## License

Apache License 2.0 - see [LICENSE](https://github.com/stars-labs/mpc-wallet/blob/main/LICENSE) for details.