# System Patterns - MPC Wallet Extension

## Architectural Patterns

### 1. Multi-Context Extension Architecture
```
Background Service Worker (Central Hub)
    ├── Popup UI (Svelte Components)
    ├── Offscreen Document (WebRTC Handler)
    ├── Content Script (Web Integration)
    └── Injected Script (Page Provider)
```

**Pattern**: Single background service worker acts as message router between specialized contexts
**Benefit**: Isolates concerns while maintaining secure communication channels

### 2. Message-Driven Communication
```typescript
// Type-safe message system with runtime validation
export type BackgroundMessage = BaseMessage & (
    | { type: 'proposeSession'; session_id: string; total: number; threshold: number; participants: string[] }
    | { type: 'acceptSession'; session_id: string; accepted: boolean }
    // ... other message types
);
```

**Pattern**: Strongly-typed message contracts with validation functions
**Implementation**: `/src/types/messages.ts` - comprehensive type system for all inter-context communication

### 3. State Management Pattern
```typescript
// Centralized app state in background script
let appState: AppState = {
    peerId: "",
    connectedPeers: [],
    wsConnected: false,
    sessionInfo: null,
    invites: [],
    meshStatus: { type: MeshStatusType.Incomplete },
    dkgState: DkgState.Idle,
    webrtcConnections: {}
};
```

**Pattern**: Single source of truth in background script, broadcasted to all contexts
**Sync Mechanism**: Port-based communication for reactive UI updates

### 4. Offscreen Document Pattern for WebRTC
```typescript
// DOM-dependent operations isolated to offscreen document
// WebRTC operations require DOM context but don't need UI
class WebRTCManager {
    // Handles peer connections without blocking UI
}
```

**Pattern**: Offscreen document for WebRTC operations that need DOM but not user interface
**Benefit**: Keeps background service worker clean while enabling WebRTC functionality

## Communication Patterns

### 1. Port-Based Popup Communication
```typescript
// Persistent connection for reactive updates
chrome.runtime.onConnect.addListener((port) => {
    if (port.name === "popup") {
        port.postMessage(initialStateMessage);
        port.onMessage.addListener(handlePopupMessage);
    }
});
```

**Pattern**: Long-lived connections for real-time state synchronization
**Use Case**: Popup UI needs immediate updates when state changes

### 2. Safe Offscreen Messaging with Retries
```typescript
async function safelySendOffscreenMessage(
    message: BackgroundToOffscreenMessage, 
    messageDescription: string, 
    maxRetries = 3
): Promise<{ success: boolean, error?: string }>
```

**Pattern**: Resilient messaging with automatic retry logic
**Handles**: Offscreen document lifecycle and timing issues

### 3. WebSocket to WebRTC Bridge
```typescript
// Background receives WebSocket messages and forwards to offscreen
const relayViaWs: OffscreenMessage = {
    type: "relayViaWs",
    to: targetPeerId,
    data: webrtcSignalData
};
```

**Pattern**: Background script bridges WebSocket signaling server with WebRTC peer connections
**Separation**: Network I/O in background, WebRTC operations in offscreen

## Error Handling Patterns

### 1. Graceful Degradation
```typescript
// Extension functions without WebRTC if offscreen creation fails
if (await chrome.offscreen.hasDocument()) {
    // Full WebRTC functionality
} else {
    // Fallback to WebSocket-only mode
}
```

**Pattern**: Progressive enhancement with fallback capabilities
**Implementation**: Multi-layer error handling at each abstraction level

### 2. Message Validation Pipeline
```typescript
// Runtime type validation for all inter-context messages
if (!validateMessage(message)) {
    console.warn("Invalid message structure:", message);
    sendResponse({ success: false, error: "Invalid message structure" });
    return;
}
```

**Pattern**: Fail-fast validation with detailed error reporting
**Security**: Prevents malformed messages from causing system instability

### 3. State Recovery Mechanisms
```typescript
// Automatic state synchronization on reconnection
port.onMessage.addListener(handleBackgroundMessage);
// Request initial state on popup connection
chrome.runtime.sendMessage({ type: "getState" });
```

**Pattern**: Self-healing state management with recovery on context reconnection

## Build and Development Patterns

### 1. WXT Framework Integration
```typescript
// wxt.config.ts - Unified configuration for all extension contexts
export default defineConfig({
    srcDir: 'src',
    modules: ['@wxt-dev/module-svelte'],
    vite: () => ({ plugins: [wasm(), topLevelAwait(), tailwindcss()] })
});
```

**Pattern**: Single configuration for multi-context extension development
**Benefits**: Hot reload, TypeScript support, modern build pipeline

### 2. Rust/WASM Integration
```typescript
// WebAssembly for cryptographic operations
import { wasm_function } from '../wasm/crypto_module';
// Vite plugins handle WASM loading and top-level await
```

**Pattern**: Heavy cryptographic computation in WASM for performance
**Integration**: Seamless TypeScript/WASM boundary with proper type definitions

### 3. Type Safety Across Contexts
```typescript
// Shared type definitions across all extension contexts
import type { SessionInfo, WebRTCAppMessage } from '../../types/appstate';
```

**Pattern**: Centralized type definitions prevent context communication errors
**Enforcement**: TypeScript compiler ensures type consistency across contexts

[2025-06-07 18:30:45] - System patterns documentation established