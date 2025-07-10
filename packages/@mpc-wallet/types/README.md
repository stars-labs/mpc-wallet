# @mpc-wallet/types

Shared TypeScript type definitions for the MPC Wallet ecosystem.

## Installation

```bash
bun add @mpc-wallet/types
# or
npm install @mpc-wallet/types
```

## Usage

### Import specific types

```typescript
import { AppState, SessionInfo, DkgState } from '@mpc-wallet/types';
```

### Import message types

```typescript
import { 
    PopupToBackgroundMessage,
    BackgroundToOffscreenMessage,
    MESSAGE_TYPES 
} from '@mpc-wallet/types';
```

### Import constants and utilities

```typescript
import { 
    INITIAL_APP_STATE,
    MeshStatusType,
    validateSessionProposal 
} from '@mpc-wallet/types';
```

## Available Types

### Core Types
- `AppState` - Central application state
- `SessionInfo` - MPC session information
- `DkgState` - Distributed Key Generation states
- `MeshStatus` - WebRTC mesh network status

### Message Types
- `PopupToBackgroundMessage` - Messages from popup to background
- `BackgroundToOffscreenMessage` - Messages to offscreen document
- `WebRTCAppMessage` - Application messages over WebRTC

### Keystore Types
- `KeyShareData` - FROST key share data structure
- `ExtensionWalletMetadata` - Wallet metadata for extension
- `KeystoreBackup` - Backup format for keystores

### Network Types
- `Chain` - Blockchain network configuration
- `Account` - User account information

## Type Organization

Types are organized by domain:
- `account.ts` - Account management types
- `appstate.ts` - Application state types
- `dkg.ts` - DKG protocol types
- `keystore.ts` - Keystore and wallet types
- `mesh.ts` - WebRTC mesh network types
- `messages.ts` - Inter-component message types
- `network.ts` - Blockchain network types
- `session.ts` - MPC session types
- `webrtc.ts` - WebRTC communication types
- `websocket.ts` - WebSocket signaling types

## Development

```bash
# Build the package
bun run build

# Watch mode
bun run dev

# Clean build artifacts
bun run clean
```