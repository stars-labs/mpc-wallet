# Final SessionUpdate Broadcast Fix

## Problem Identified
mpc-3 was showing only "1/3 participants" because it wasn't receiving SessionUpdate messages when other participants joined.

## Root Cause
The SessionUpdate broadcast logic was incorrectly excluding the newly joined participant from receiving the update. 

When mpc-3 joined and sent SessionResponse to mpc-1, the creator (mpc-1) would:
1. Add mpc-3 to accepted_devices list
2. Try to broadcast SessionUpdate to all participants
3. BUT exclude both `from_device_id` (mpc-3) AND `device_id` (mpc-1) 
4. Result: Only mpc-2 received the update!

This meant mpc-3 never learned about mpc-2 being in the session, so it only knew about itself.

## The Fix
Changed the broadcast logic to only exclude the sender (mpc-1), NOT the newly joined participant.

### Before:
```rust
// Excluded both from_device_id (newly joined) and device_id (self)
if participant != &from_device_id && participant != &device_id {
    // Send update
}
```

### After:
```rust
// Only exclude ourselves (the creator), NOT the newly joined participant
if participant != &device_id {
    // Send update - this ensures newly joined participant gets the full list
}
```

## Why This Fix Works
- When mpc-2 joins: mpc-2 receives update with `[mpc-1, mpc-2]`
- When mpc-3 joins: 
  - mpc-2 receives update with `[mpc-1, mpc-2, mpc-3]`
  - **mpc-3 ALSO receives update with `[mpc-1, mpc-2, mpc-3]`** ← This was missing!

## Expected Behavior After Fix
1. mpc-1 creates session → shows 1/3 participants
2. mpc-2 joins → both show 2/3 participants
3. mpc-3 joins → ALL THREE show 3/3 participants
4. DKG can start when mesh is ready

## Testing
Run the nodes again and check:
1. mpc-3 should receive "📢 Received SessionUpdate from mpc-1" in logs
2. mpc-3 should update to show 3/3 participants
3. All nodes should show the same participant count

## File Modified
`/apps/cli-node/src/handlers/session_handler.rs` - Fixed broadcast logic in `handle_process_session_response`