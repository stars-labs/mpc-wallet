# MPC Wallet Extension

A Multi-Party Computation (MPC) wallet browser extension built with WXT, Svelte, and Rust/WebAssembly. This extension enables secure distributed key generation and signing operations across multiple parties using WebRTC for peer-to-peer communication.

## Table of Contents

- [Architecture Overview](#architecture-overview)
- [Components](#components)
- [Message System](#message-system)
- [WebSocket Communication](#websocket-communication)
- [WebRTC Management](#webrtc-management)
- [Installation](#installation)
- [Development](#development)
- [Usage](#usage)
- [API Reference](#api-reference)

## Architecture Overview

The MPC Wallet Extension follows a Chrome Extension Manifest V3 architecture with four main contexts that communicate via strongly-typed messages:

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Popup Page    │    │ Background Page │    │ Offscreen Page  │
│                 │    │                 │    │                 │
│ - UI Components │    │ - Service Worker│    │ - WebRTC Manager│
│ - State Display │    │ - Message Router│    │ - DOM Access    │
│ - User Actions  │    │ - WebSocket     │    │ - Crypto Ops    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌─────────────────┐
                    │ Content Script  │
                    │                 │
                    │ - Web Page Hook │
                    │ - JSON-RPC Proxy│
                    └─────────────────┘
                                 │
                    ┌─────────────────┐
                    │  Web Page API   │
                    │                 │
                    │ - window.ethereum│
                    │ - Wallet Methods│
                    └─────────────────┘
```

## Components

### 1. Background Page (Service Worker)
**Location:** `/src/entrypoints/background/index.ts`

**Responsibilities:**
- Central message router for all communication
- WebSocket client management for signaling server
- Account and network services
- Offscreen document lifecycle management
- RPC request handling for blockchain operations

**Key Services:**
- `AccountService`: Manages wallet accounts and private keys
- `NetworkService`: Handles blockchain network configurations
- `WalletClientService`: Provides blockchain client functionality
- `WebSocketClient`: Manages connection to signaling server

### 2. Popup Page (UI)
**Location:** `/src/entrypoints/popup/App.svelte`

**Responsibilities:**
- User interface for wallet operations
- Display connection status and peer information
- Session management UI for MPC operations
- Crypto operations (signing, address generation)

**Features:**
- Private key generation and management
- Multi-chain support (Ethereum/Solana)
- Message signing and address derivation
- Real-time peer discovery and session management
- WebRTC connection status monitoring

### 3. Offscreen Page (WebRTC Handler)
**Location:** `/src/entrypoints/offscreen/index.ts`

**Responsibilities:**
- WebRTC connection management
- P2P communication handling
- MPC session coordination
- DOM-dependent operations

**Key Components:**
- `WebRTCManager`: Handles peer-to-peer connections
- Session proposal and acceptance logic
- Data channel management for MPC communication
- ICE candidate exchange and connection establishment

### 4. Content Script (Web Integration)
**Location:** `/src/entrypoints/content/index.ts`

**Responsibilities:**
- Injects wallet API into web pages
- Provides `window.ethereum` compatibility
- Proxies JSON-RPC requests to background script
- Manages web page wallet interactions

## Message System

The extension uses a comprehensive type-safe message system defined in `/src/types/messages.ts`:

### Core Message Types

#### Popup ↔ Background Communication
```typescript
// Popup to Background
export type PopupToBackgroundMsg = BaseMessage & (
    | { type: 'GET_WALLET_STATE' }
    | { type: 'START_DKG_SESSION'; participants: string[] }
    | { type: 'CONNECT_WEBSOCKET'; url: string }
    | { type: 'LIST_PEERS' }
    // ... other message types
);

// Background to Popup
export type BackgroundToPopupMsg = BaseMessage & (
    | { type: 'WALLET_STATE'; isUnlocked: boolean; accounts: string[] }
    | { type: 'DKG_SESSION_STARTED'; sessionId: string; success: boolean }
    | { type: 'WEBSOCKET_CONNECTED'; success: boolean; error?: string }
    | { type: 'PEERS_LIST'; peers: string[] }
    // ... other message types
);
```

#### Background ↔ Offscreen Communication
```typescript
// Background to Offscreen
export type BackgroundToOffscreenMsg = BaseMessage & (
    | { type: 'InitializeDkg'; participants: string[] }
    | { type: 'ConnectWebSocket'; url: string }
    | { type: 'RelayMessage'; to: string; data: any }
    // ... other message types
);

// Offscreen to Background
export type OffscreenToBackgroundMsg = BaseMessage & (
    | { type: 'DkgState'; dkg_state: DkgState }
    | { type: 'SessionInfo'; session_info: SessionInfo }
    | { type: 'WebSocketConnected'; success: boolean; error?: string }
    // ... other message types
);
```

### Message Flow Patterns

1. **Extension Initialization**
   ```
   Background Script → WebSocket → Services → Offscreen Creation
   ```

2. **MPC Session Flow**
   ```
   Popup → Background → Offscreen → WebRTC Setup → Peer Communication
   ```

3. **WebRTC Signaling**
   ```
   Peer A → Background → WebSocket → Background → Peer B
   ```

## WebSocket Communication

### Server Connection
**Location:** Background Page (`/src/entrypoints/background/websocket.ts`)

The WebSocket client connects to a signaling server for peer discovery and WebRTC signaling:

```typescript
const WEBSOCKET_URL = "wss://auto-life.tech";
wsClient = new WebSocketClient(WEBSOCKET_URL);
```

### Message Types
- **Registration**: Peers register with their unique ID
- **Peer Discovery**: List available peers for MPC sessions
- **Relay**: Forward WebRTC signaling data between peers
- **Session Management**: Coordinate MPC session proposals

### Connection Management
- Automatic reconnection with exponential backoff
- Connection state monitoring and UI updates
- Error handling and recovery mechanisms

## WebRTC Management

### Peer Connection Setup
**Location:** Offscreen Page (`/src/entrypoints/offscreen/webrtc.ts`)

The WebRTC manager handles:
- **Peer Connection Creation**: RTCPeerConnection instances for each participant
- **Data Channel Setup**: Reliable data channels for MPC communication
- **ICE Handling**: STUN/TURN server configuration and candidate exchange
- **Connection State Monitoring**: Track connection health and handle failures

### Session Management
```typescript
// Session Proposal
webRTCManager.proposeSession(sessionId, total, threshold, participants);

// Session Acceptance
webRTCManager.acceptSession(sessionId);

// Mesh Status Tracking
enum MeshStatusType {
    Incomplete,
    PartiallyReady,
    Ready
}
```

### Security Features
- **Origin Validation**: Verify message sources
- **Encrypted Channels**: Secure WebRTC data transmission
- **Isolated Contexts**: Separate WebRTC operations in offscreen context

## Installation

### Prerequisites
- Node.js 18+ and npm/yarn
- Chrome/Chromium browser for testing
- Rust toolchain for WASM compilation

### Development Setup
```bash
# Clone the repository
git clone <repository-url>
cd mpc-wallet

# Install dependencies
npm install

# Build WASM modules
npm run build:wasm

# Start development server
npm run dev

# Build for production
npm run build
```

### Extension Installation
1. Build the extension: `npm run build`
2. Open Chrome and navigate to `chrome://extensions/`
3. Enable "Developer mode"
4. Click "Load unpacked" and select the `dist` folder

## Development

### Project Structure
```
src/
├── entrypoints/
│   ├── background/     # Service worker
│   ├── content/        # Content scripts
│   ├── offscreen/      # Offscreen document
│   └── popup/          # Extension popup UI
├── types/              # TypeScript type definitions
├── services/           # Business logic services
└── components/         # Svelte UI components
```

### Key Files
- `src/types/messages.ts`: Message type definitions
- `src/types/appstate.ts`: Application state types
- `src/entrypoints/background/index.ts`: Main background script
- `src/entrypoints/offscreen/webrtc.ts`: WebRTC management
- `src/entrypoints/popup/App.svelte`: Main UI component

### Testing
```bash
# Run type checking
npm run type-check

# Run linting
npm run lint

# Run tests
npm run test
```

## Usage

### Basic Wallet Operations
1. **Generate Wallet**: Click "Show Wallet Address" to create/display address
2. **Sign Messages**: Enter message and click "Sign Message"
3. **Chain Support**: Switch between Ethereum (secp256k1) and Solana (ed25519)

### MPC Session Management
1. **Peer Discovery**: Click "List Peers" to find available participants
2. **Create Session**: Click "Propose Session" with 3+ peers
3. **Join Session**: Accept incoming session invitations
4. **Monitor Status**: View connection and session state in real-time

### Advanced Features
- **Network Switching**: Change blockchain networks
- **Account Management**: Import/export private keys
- **Connection Diagnostics**: Debug WebRTC and WebSocket issues

## API Reference

### Background Script API
```typescript
// Account Management
handleAccountManagement(action: string, payload: any)

// Network Management
handleNetworkManagement(action: string, payload: any)

// RPC Handling
handleRpcRequest(request: JsonRpcRequest)
```

### WebRTC Manager API
```typescript
// Session Management
proposeSession(sessionId: string, total: number, threshold: number, participants: string[])
acceptSession(sessionId: string)
resetSession()

// Communication
sendWebRTCAppMessage(toPeerId: string, message: WebRTCAppMessage)
```

### WebSocket Client API
```typescript
// Connection Management
connect()
disconnect()
register(peerId: string)

// Communication
relayMessage(to: string, data: any)
listPeers()
```

## Error Handling and Recovery

### Offscreen Document Management
- Background script ensures offscreen document exists before forwarding messages
- Creation is protected against concurrent attempts
- Ready signal confirms initialization before use

### WebSocket Reconnection
- Automatic reconnection with exponential backoff
- State synchronization on reconnection
- UI reflects connection status changes

### WebRTC Connection Recovery
- ICE connection state monitoring
- Automatic cleanup of failed connections
- Session reset capabilities for stuck states

## Security Considerations

1. **Message Validation**: All messages are strongly typed and validated
2. **Origin Checking**: Content scripts verify message sources
3. **Isolated Contexts**: WebRTC operations isolated to offscreen context
4. **Secure Communication**: All external communication via WebSocket/WebRTC
5. **Private Key Security**: Keys stored securely in extension storage

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make changes with proper TypeScript typing
4. Add tests for new functionality
5. Submit a pull request

## License

[Add your license information here]

## Support

For issues and questions:
- Create an issue in the repository
- Check the console logs for debugging information
- Use the built-in diagnostic tools in the popup UI
