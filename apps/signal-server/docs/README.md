# Signal Server Documentation

Documentation for the WebRTC signaling servers used by the MPC Wallet.

## Documentation Structure

- [`deployment/`](./deployment/) - Deployment guides and configurations
- [`architecture/`](./architecture/) - Technical architecture documentation (coming soon)

## Overview

The signal server provides:

- WebSocket-based signaling for WebRTC
- Session discovery and management
- Peer connection coordination
- Message relay for offline peers

## Implementations

### 1. Standard Server (`server/`)
- Rust-based WebSocket server
- Runs on standard infrastructure
- Suitable for self-hosting

### 2. Cloudflare Worker (`cloudflare-worker/`)
- Edge-deployed serverless implementation
- Global distribution via Cloudflare network
- Automatic scaling

## Deployment

- For Cloudflare deployment, see [`deployment/cloudflare-deployment.md`](./deployment/cloudflare-deployment.md)
- For standard server deployment, see the server [README](../server/README.md)

## Protocol

The signaling protocol handles:
- Session creation/joining
- Peer discovery
- WebRTC offer/answer exchange
- ICE candidate exchange
- Keepalive and presence