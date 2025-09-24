# Participant Update Fix - Complete WebRTC Mesh Formation

## Critical Issue Found
mpc-2 and mpc-3 were NOT establishing WebRTC connections with each other. They only connected to mpc-1 (session creator).

## Root Cause Analysis

### Signal Server Behavior
When a new participant joins, the server sends a special participant update message:
```json
{
  "type": "participant_update",
  "session_id": "...",
  "session_info": {
    "participants": ["mpc-1", "mpc-2", "mpc-3"],
    ...
  }
}
```

This is sent as a `ServerMsg::Relay` with `from: "server"` to ALL connected devices.

### The Bug
In our WebSocket handlers (both StartDKG and JoinDKG), we were ONLY processing Relay messages where `from != "server"`:

```rust
// OLD CODE - BUGGY
if from != "server" {
    // Handle WebRTC signals from other devices
    // ...
}
// Participant updates from server were IGNORED!
```

This meant participant update messages were completely ignored, so:
- When mpc-3 joined, mpc-2 didn't know about it
- mpc-2 never initiated a WebRTC connection to mpc-3
- The mesh remained incomplete

## The Fix

Added proper handling for server messages with participant updates:

```rust
// Check if it's a participant update from the server
if from == "server" {
    if let Some(msg_type) = data.get("type").and_then(|v| v.as_str()) {
        if msg_type == "participant_update" {
            // Extract and process participant list
            // Track previous count to detect NEW participants
            // Initiate WebRTC to ALL participants (excluding self)
            // This ensures mpc-2 connects to mpc-3 when mpc-3 joins
        }
    }
}
```

### Key Changes in command.rs

1. **StartDKG Handler (lines 591-652)**:
   - Added handler for `from == "server"` with `type == "participant_update"`
   - Tracks participant count changes
   - Triggers WebRTC initiation when new participants detected

2. **JoinDKG Handler (lines 1366-1427)**:
   - Same participant update handler added
   - Ensures joining nodes also respond to new participants

## Result

Now when mpc-3 joins:
1. Server broadcasts participant update to ALL nodes
2. mpc-1 receives update and initiates to mpc-3 ✅
3. **mpc-2 ALSO receives update and initiates to mpc-3** ✅ (THIS WAS MISSING!)
4. Full mesh forms with all nodes connected to all others

## Testing

The fix ensures:
- mpc-1 ↔ mpc-2 ✅
- mpc-1 ↔ mpc-3 ✅  
- **mpc-2 ↔ mpc-3** ✅ (NOW WORKS!)

All nodes now properly form a complete WebRTC mesh network.