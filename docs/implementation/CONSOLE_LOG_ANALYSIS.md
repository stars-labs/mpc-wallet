# Console Log Analysis

## Summary
Total console statements found: 756

## Categories

### 1. **Essential Logs (KEEP)** - Critical for production monitoring

#### Error Handling
- `console.error` statements that report actual runtime errors
- WebSocket connection errors
- WASM initialization failures
- RPC request failures
- Cryptographic operation failures

#### Security & Permissions
- Permission grant/revoke events
- Account connection events
- Signature request approvals/rejections

### 2. **Debug Logs (REMOVE)** - Development/debugging only

#### Component Lifecycle & State
- Component mount/unmount logs
- State update logs
- UI preference save/load logs
- Message routing debug logs

#### WebRTC Connection Debug
- Peer connection state changes
- ICE candidate exchanges
- Data channel status updates
- Connection establishment logs

#### Message Flow Debug
- Message type logging
- Message routing logs
- Inter-component communication logs

#### WASM Module Debug
- Module loading progress
- Type checking logs
- Test instance creation logs

### 3. **Informational Logs (SELECTIVE)** - Consider keeping some

#### User Actions
- Account creation/selection
- Network changes
- Settings updates

#### System Status
- WebSocket connection status
- Offscreen document lifecycle
- Session management events

## File-by-File Recommendations

### Critical Files (Most logs should be removed)

1. **src/entrypoints/offscreen/webrtc.ts**
   - Lines 783-813: WASM module resolution debug logs - REMOVE
   - Line 80: Default onLog handler - KEEP (but make configurable)

2. **src/entrypoints/offscreen/messageRouter.ts**
   - Lines 81-87, 99-113, 122: Message parsing debug logs - REMOVE
   - Lines 59, 63-64, 70: Message routing logs - REMOVE

3. **src/entrypoints/background/messageHandlers.ts**
   - Lines 66-105: Decorative message processing logs - REMOVE
   - Keep only error logs

4. **src/entrypoints/offscreen/webrtcConnection.ts**
   - Most connection state logs (lines 59-239) - REMOVE
   - Keep only critical errors

5. **src/entrypoints/offscreen/wasmInitializer.ts**
   - Lines 78-113: WASM initialization debug logs - REMOVE
   - Lines 117-119: Keep error logs

### Service Files

1. **src/services/permissionService.ts**
   - Keep permission grant/revoke logs (security-relevant)

2. **src/services/keystoreService.ts**
   - Keep error logs only

3. **src/services/accountService.ts**
   - Keep account creation/deletion logs (audit trail)

### UI Components

1. **src/components/Settings.svelte**
   - Remove most UI state logs
   - Keep network addition/removal logs

2. **src/components/SignatureRequest.svelte**
   - Keep approval/rejection logs (security audit)

3. **src/components/AccountManager.svelte**
   - Keep account creation logs
   - Remove UI state logs

### Background Services

1. **src/entrypoints/background/index.ts**
   - Lines 175-181: Message routing debug logs - REMOVE
   - Keep initialization success/failure logs

2. **src/entrypoints/background/webSocketManager.ts**
   - Keep connection/disconnection logs
   - Remove message detail logs

3. **src/entrypoints/background/stateManager.ts**
   - Remove state update logs
   - Keep persistence failure warnings

## Implementation Strategy

1. **Create a Logger Service**
   - Implement log levels (ERROR, WARN, INFO, DEBUG)
   - Environment-based filtering
   - Structured logging with context

2. **Replace Console Statements**
   - Use logger service instead of direct console calls
   - Set production log level to ERROR/WARN only
   - Enable DEBUG logs in development

3. **Add Log Configuration**
   - Environment variable for log level
   - Runtime log level adjustment
   - Log output formatting options

## Priority Removal List (Top offenders by line count)

1. src/entrypoints/offscreen/webrtc.ts (30+ debug logs)
2. src/entrypoints/offscreen/webrtcConnection.ts (25+ debug logs)
3. src/entrypoints/background/messageHandlers.ts (40+ debug logs)
4. src/entrypoints/offscreen/messageRouter.ts (20+ debug logs)
5. src/entrypoints/background/stateManager.ts (25+ debug logs)
6. src/entrypoints/offscreen/wasmInitializer.ts (15+ debug logs)
7. src/entrypoints/background/patternRouter.ts (decorative logs)
8. src/entrypoints/content/provider.ts (20+ debug logs)

## Logs to Definitely Keep

1. **Security Events**
   - Permission changes
   - Account connections
   - Signature approvals/rejections

2. **Critical Errors**
   - WASM initialization failures
   - WebSocket connection failures
   - RPC errors
   - Cryptographic operation failures

3. **Audit Trail**
   - Account creation/deletion
   - Network additions/removals
   - Transaction submissions

4. **User-Facing Errors**
   - Failed operations that need user attention
   - Configuration issues