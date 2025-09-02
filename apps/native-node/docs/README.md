# Native Desktop Application Documentation

Documentation for the MPC Wallet native desktop application built with Slint UI framework.

## Documentation Structure

- [`ui/`](./ui/) - User interface design and mockups (coming soon)
- [`guides/`](./guides/) - User guides and tutorials (coming soon)
- [`architecture/`](./architecture/) - Technical architecture documentation (coming soon)

## Overview

The native desktop application provides:

- Cross-platform desktop interface (Linux, macOS, Windows)
- Native performance with Slint UI framework
- Shared core functionality with CLI node
- Modern, responsive UI design
- Real-time status updates

## Features

- **Session Management** - Create and join DKG sessions
- **Keystore Operations** - Import/export encrypted keystores
- **Network Monitoring** - WebRTC/WebSocket status
- **Multi-chain Support** - Ethereum and Solana

## Development

See the main [README.md](../README.md) and [NATIVE_APP_GUIDE.md](../../../docs/NATIVE_APP_GUIDE.md) for development setup.

## Architecture

The native app reuses the CLI node library, implementing the `UIProvider` trait to bridge terminal commands to GUI operations.