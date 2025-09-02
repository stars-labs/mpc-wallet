# Session Update Broadcast Fix

## Problem
After mpc-2 accepted the session, mpc-3 was not being informed about this acceptance, causing mpc-3 to still show "1/3 participants" while mpc-2 correctly showed "3/3 participants".

## Root Cause
The SessionUpdate broadcast in `handle_process_session_response` was sending raw JSON instead of a properly structured `WebSocketMessage::SessionUpdate` that the websocket handler expects.

## Fix Applied
Changed the broadcast mechanism to:
1. Create a proper `SessionUpdate` struct with:
   - `session_id`: The session being updated
   - `accepted_devices`: The complete list of accepted devices
   - `update_type`: `ParticipantJoined` with the device_id of the newly accepted participant

2. Wrap the SessionUpdate in `WebSocketMessage::SessionUpdate`

3. Serialize the wrapped message to JSON for the Relay command

## Files Modified
- `/apps/cli-node/src/handlers/session_handler.rs`
  - Fixed `handle_process_session_response` to create proper SessionUpdate messages
  - Added imports for SessionUpdate, SessionUpdateType, and WebSocketMessage

## Expected Behavior After Fix
1. mpc-1 creates session and shows 1/3 participants
2. mpc-2 joins and accepts → sends SessionResponse to mpc-1
3. mpc-1 processes SessionResponse and broadcasts SessionUpdate to mpc-3
4. mpc-3 receives SessionUpdate and updates its participant count to 2/3
5. mpc-3 joins and accepts → sends SessionResponse to mpc-1
6. mpc-1 processes SessionResponse and broadcasts SessionUpdate to mpc-2
7. All nodes show 3/3 participants
8. DKG can begin once WebRTC mesh is fully formed

## Testing
After this fix, run the test again:
1. Start all three nodes
2. Create session on mpc-1
3. Join on mpc-2 and mpc-3
4. All nodes should show correct participant counts
5. DKG should start automatically when all participants are connected