# API Documentation

## Overview

Complete API reference for all MPC Wallet components and interfaces.

## Contents

### Core APIs
- [FROST Protocol API](frost-api.md) - Core cryptographic operations
- [Wallet API](wallet-api.md) - Wallet management operations
- [Session API](session-api.md) - Session management and coordination

### Application APIs
- [Browser Extension API](browser-extension-api.md) - Chrome/Firefox extension APIs
- [CLI Commands](cli-commands.md) - Terminal UI command reference
- [Desktop API](desktop-api.md) - Native desktop application APIs

### Network APIs
- [WebSocket Protocol](websocket-protocol.md) - Signal server protocol
- [WebRTC Messages](webrtc-messages.md) - P2P message formats
- [REST API](rest-api.md) - HTTP API endpoints

### Integration APIs
- [Web3 Provider](web3-provider.md) - Ethereum provider implementation
- [RPC Methods](rpc-methods.md) - JSON-RPC method reference
- [Event System](events.md) - Event subscription and handling

## Quick Reference

### Browser Extension

```typescript
// Initialize wallet
const wallet = await chrome.runtime.sendMessage({
  type: 'CREATE_WALLET',
  payload: {
    name: 'My Wallet',
    threshold: 2,
    participants: 3
  }
});

// Sign transaction
const signature = await chrome.runtime.sendMessage({
  type: 'SIGN_TRANSACTION',
  payload: {
    walletId: wallet.id,
    transaction: txData
  }
});
```

### Terminal UI

```bash
# Create wallet
mpc-wallet-tui create my_wallet 2 3

# Sign message
mpc-wallet-tui sign wallet_id "message to sign"

# Export wallet
mpc-wallet-tui export wallet_id ./backup.json
```

### WebSocket Protocol

```json
{
  "type": "CREATE_SESSION",
  "deviceId": "Device-001",
  "payload": {
    "threshold": 2,
    "participants": 3,
    "blockchain": "ethereum"
  }
}
```

## API Conventions

### Request Format
- JSON for all request/response bodies
- UTF-8 encoding
- ISO 8601 timestamps
- UUID v4 for identifiers

### Response Format
```json
{
  "success": true,
  "data": {},
  "error": null,
  "timestamp": "2025-01-20T10:30:00Z"
}
```

### Error Format
```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "INVALID_THRESHOLD",
    "message": "Threshold must be less than or equal to participants",
    "details": {}
  },
  "timestamp": "2025-01-20T10:30:00Z"
}
```

## Authentication

### API Key Authentication
```http
Authorization: Bearer API_KEY_HERE
```

### Session Authentication
```http
X-Session-Id: SESSION_ID_HERE
X-Device-Id: DEVICE_ID_HERE
```

## Rate Limits

| Endpoint | Limit | Window |
|----------|-------|--------|
| Create Wallet | 10 | 1 hour |
| Sign Transaction | 100 | 1 minute |
| Get Balance | 1000 | 1 minute |
| WebSocket Messages | 100 | 1 second |

## Versioning

The API follows semantic versioning (SemVer):
- Current version: `v1`
- Version in URL: `/api/v1/endpoint`
- Breaking changes increment major version

## SDK References

### JavaScript/TypeScript
```bash
npm install @mpc-wallet/sdk
```

### Rust
```toml
[dependencies]
mpc-wallet-sdk = "1.0"
```

### Python
```bash
pip install mpc-wallet
```

## Navigation

- [← Back to Main Documentation](../README.md)
- [← Security Documentation](../security/README.md)
- [Development Guide →](../development/README.md)