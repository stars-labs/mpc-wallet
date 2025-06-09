# DKG Implementation Test Guide

This guide explains how to test the newly implemented FROST DKG (Distributed Key Generation) functionality in the MPC wallet extension.

## What Was Fixed

The DKG flow was stuck because nodes weren't properly transitioning to DKG Round 1 after the mesh became ready. The issue was that `checkAndTriggerDkg()` only set the state but didn't implement the actual FROST DKG protocol.

## Implementation Summary

### Core Changes Made

1. **WASM Integration**: Added `FrostDkgEd25519` import from the existing WASM FROST library
2. **State Management**: Added DKG tracking properties to WebRTCManager:
   - `frostDkg: FrostDkgEd25519 | null` - FROST DKG instance
   - `participantIndex: number | null` - Current node's index in DKG
   - `receivedRound1Packages: Set<string>` - Track Round 1 packages received
   - `receivedRound2Packages: Set<string>` - Track Round 2 packages received

3. **Complete DKG Protocol Implementation**:
   - `_initializeDkg()`: Initialize FROST DKG with session parameters
   - `_generateAndBroadcastRound1()`: Generate and broadcast Round 1 packages
   - `_handleDkgRound1Package()`: Process incoming Round 1 packages
   - `_checkRound1Completion()`: Check if ready to proceed to Round 2
   - `_generateAndBroadcastRound2()`: Generate and broadcast Round 2 packages
   - `_handleDkgRound2Package()`: Process incoming Round 2 packages
   - `_checkRound2Completion()`: Check if ready to finalize
   - `_finalizeDkg()`: Complete DKG and generate group public key
   - `_resetDkgState()`: Clean up DKG state

4. **Message Handling**: Updated WebRTC message handlers to process DKG package messages

5. **Status API**: Added monitoring methods:
   - `getDkgStatus()`: Comprehensive DKG state information
   - `getGroupPublicKey()`: Returns group public key when DKG complete
   - `getSolanaAddress()`: Returns derived Solana address when DKG complete

6. **Offscreen Integration**: Added message handlers in `index.ts` for the new DKG status methods

## Testing the DKG Flow

### Prerequisites
- Extension builds successfully (✅ Confirmed - 1.1 MB total size)
- WASM module loads correctly in browser extension context
- Multiple browser instances or test environment for multi-node testing

### Test Steps

#### 1. Basic Status Check
Load the extension and verify the new DKG methods are accessible:
```javascript
// In extension console/popup
chrome.runtime.sendMessage({
  type: "getDkgStatus"
}, (response) => {
  console.log("DKG Status:", response);
});
```

#### 2. Multi-Node DKG Test Setup
1. Set up 3 browser instances (minimum threshold)
2. Create a session with 3 participants
3. Have all participants accept the session
4. Monitor DKG progression through the states:
   - `WaitingForMesh` → `Round1InProgress` → `Round2InProgress` → `Complete`

#### 3. Monitor DKG State Transitions
Track the DKG flow through console logs and status calls:
```javascript
// Check DKG status periodically
const checkDkgStatus = () => {
  chrome.runtime.sendMessage({ type: "getDkgStatus" }, (response) => {
    if (response.success) {
      console.log("DKG State:", response.data.state);
      console.log("Round 1 Packages:", response.data.receivedRound1Count, "/", response.data.expectedParticipants);
      console.log("Round 2 Packages:", response.data.receivedRound2Count, "/", response.data.expectedParticipants);
    }
  });
};

// Check every 2 seconds
setInterval(checkDkgStatus, 2000);
```

#### 4. Verify Completion
Once DKG completes, verify the results:
```javascript
// Get group public key
chrome.runtime.sendMessage({ type: "getGroupPublicKey" }, (response) => {
  console.log("Group Public Key:", response.data.groupPublicKey);
});

// Get Solana address
chrome.runtime.sendMessage({ type: "getSolanaAddress" }, (response) => {
  console.log("Solana Address:", response.data.solanaAddress);
});
```

### Expected Behavior

1. **Mesh Ready**: When all participants are connected, DKG should automatically start
2. **Round 1**: Each node generates Round 1 packages and broadcasts to peers
3. **Round 1 Collection**: Nodes collect packages from all other participants
4. **Round 2**: Once all Round 1 packages received, Round 2 begins automatically
5. **Round 2 Collection**: Nodes collect Round 2 packages from all participants
6. **Finalization**: DKG completes and group public key is generated
7. **Solana Integration**: Solana address is derived from the group public key

### Debugging DKG Issues

#### Console Log Monitoring
Watch for these key log messages:
- `"Starting DKG process for session"`
- `"Generated Round 1 package"`
- `"Received DKG Round 1 package from"`
- `"All Round 1 packages received, proceeding to Round 2"`
- `"DKG completed successfully"`

#### Common Issues to Check
1. **WASM Loading**: Verify `FrostDkgEd25519` loads without errors
2. **Message Routing**: Ensure DKG packages are properly broadcast via WebRTC
3. **Participant Indexing**: Confirm each node has correct participant index
4. **Package Collection**: Verify all nodes receive packages from all other nodes
5. **State Synchronization**: Check that all nodes progress through DKG states together

#### Error Recovery
If DKG fails or gets stuck:
1. Check console for specific error messages
2. Verify WebRTC connectivity between all nodes
3. Ensure session has minimum required participants (3)
4. Restart session if necessary - DKG state resets with new sessions

## Files Modified

1. **`src/entrypoints/offscreen/webrtc.ts`**: Main DKG implementation (~200 lines added)
2. **`src/entrypoints/offscreen/index.ts`**: Added DKG status message handlers

## Next Steps for Production

1. **Error Recovery**: Add robust error handling for failed DKG attempts
2. **Retry Logic**: Implement automatic retry for transient failures
3. **Timeout Handling**: Add timeouts for DKG rounds to prevent indefinite waiting
4. **Persistence**: Consider persisting DKG results for session recovery
5. **Security Review**: Audit FROST DKG implementation for security issues
6. **Performance**: Optimize for larger participant counts if needed

## Build Status
✅ Extension builds successfully (1.1 MB total)
✅ WASM integration working
✅ No TypeScript compilation errors
✅ All DKG methods exposed via message handlers
