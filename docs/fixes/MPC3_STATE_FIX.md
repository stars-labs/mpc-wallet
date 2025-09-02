# MPC-3 State Synchronization Fix

## Issues Fixed

### 1. Wrong Leader Display (mpc-3 showing as leader instead of mpc-1)
**Root Cause**: The proposer_id was being set incorrectly when creating/updating invites from SessionProposal messages.

**Fix Applied**:
- Changed from using `from` (the relay sender) to using `proposal.proposer_device_id` 
- This ensures the actual session creator is recorded as the proposer

### 2. Wrong Participant Count (1/3 instead of 3/3)
**Root Cause**: SessionUpdate messages weren't being received/processed by mpc-3

**Debugging Added**:
- Added logging for raw websocket message types
- Added detailed SessionUpdate reception logging
- Added logging for accepted_devices updates
- Added warnings when SessionUpdate is ignored

### 3. Enhanced Debug Logging

Added comprehensive logging at multiple points:
1. **Raw message type logging**: Shows websocket_msg_type before parsing
2. **SessionUpdate details**: Logs session_id, update_type, and accepted_devices
3. **State updates**: Shows before/after accepted_devices lists
4. **Error cases**: Logs when updates are ignored and why

## Files Modified

### `/apps/cli-node/src/network/websocket.rs`
- Fixed proposer_id assignment in invite creation
- Added comprehensive debug logging throughout
- Fixed borrow checker issues with proper state management
- Added raw message logging for debugging

## Expected Behavior After Fix

1. **mpc-3 will show correct leader**: Should display "mpc-1 (Leader)" not "mpc-3 (Leader)"
2. **Participant count will update correctly**: When SessionUpdate messages are received
3. **Better debugging**: Clear logs showing why updates might be ignored

## Testing Instructions

1. Start all three nodes with the updated code
2. Create session on mpc-1
3. Join from mpc-2 and mpc-3
4. Check the logs for:
   - "🔍 Relay message type from..." - Shows incoming message types
   - "📢 Received SessionUpdate from..." - Confirms update reception
   - "✅ Updated accepted_devices..." - Shows participant list updates
   - "⚠️ SessionUpdate ignored..." - Explains why updates are dropped

## Remaining Work

If SessionUpdate messages are still not being received:
1. Check that the broadcast in session_handler.rs is working
2. Verify the SessionUpdate struct serialization matches expected format
3. Ensure WebSocket relay is forwarding messages correctly