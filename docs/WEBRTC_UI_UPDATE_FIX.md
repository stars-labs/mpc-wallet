# WebRTC UI Update and DKG Auto-Start Fix

## Issues Fixed

### 1. Self-Participant Display
- **Problem**: Nodes were showing themselves in their participant lists
- **Solution**: Added check to skip self when building participant list in app.rs

### 2. Mesh Count Display
- **Problem**: Mesh ready count was incorrect (showing 1/2 when it should be 2/2)
- **Solution**: Fixed comparison to use `==` instead of `>=` for all_connected check

### 3. DKG Not Starting
- **Problem**: DKG protocol wasn't starting when mesh was ready
- **Solution**: Added automatic DKG triggering when all participants are connected

## Implementation Details

### Files Modified

#### 1. `/apps/tui-node/src/elm/app.rs`
```rust
// Skip self in participant list (lines 330-333)
if participant == &self.model.device_id {
    continue;
}

// Fixed mesh calculation (line 366)
let all_connected = mesh_ready_count == expected_other_participants;

// Added debug logging (lines 328, 368)
info!("📋 Session participants: {:?}, self: {}", session.participants, self.model.device_id);
info!("🔗 Mesh status calculation: ready_count={}, expected={}, all_connected={}", ...);
```

#### 2. `/apps/tui-node/src/elm/update.rs`
```rust
// Added DKG auto-start logic (lines 297-338)
let should_start_dkg = if let Some(ref session) = model.active_session {
    // Count connected participants (excluding self)
    let connected_count = session.participants.iter()
        .filter(|p| **p != model.device_id)
        .filter(|p| {
            model.network_state.participant_webrtc_status.get(*p)
                .map_or(false, |(_, data_channel_open)| *data_channel_open)
        })
        .count();
    
    // Check if all participants connected and DKG not started
    connected_count == expected_other_participants && 
    !model.wallet_state.dkg_in_progress
} else {
    false
};

// Trigger DKG when ready
if should_start_dkg {
    info!("🎯 All participants connected! Starting DKG protocol...");
    model.wallet_state.dkg_in_progress = true;
    Some(Command::StartDKGProtocol)
}
```

#### 3. `/apps/tui-node/src/elm/model.rs`
```rust
// Added dkg_in_progress field (line 88)
pub struct WalletState {
    // ... existing fields ...
    pub dkg_in_progress: bool,
}
```

#### 4. `/apps/tui-node/src/elm/command.rs`
```rust
// Added StartDKGProtocol command (line 47)
pub enum Command {
    // ... existing commands ...
    StartDKGProtocol,
}

// Added handler (lines 2199-2227)
Command::StartDKGProtocol => {
    info!("🚀 Starting DKG protocol - mesh is ready!");
    
    if let Some(session) = &state.session {
        // Update UI to show DKG is starting
        let _ = tx.send(Message::UpdateDKGProgress {
            round: crate::elm::model::DKGRound::Round1,
            progress: 25.0,
        });
        
        // TODO: Connect to actual DKG protocol handler
        info!("⚠️ DKG protocol trigger not yet fully implemented");
    }
}
```

## How It Works

### Connection Flow
1. Participants join session and establish WebSocket connections
2. WebRTC connections are initiated between all participants
3. Data channels are opened for P2P communication
4. Each connection update triggers `UpdateParticipantWebRTCStatus`

### DKG Trigger Logic
1. When WebRTC status updates, check if all participants connected
2. Count participants with open data channels (excluding self)
3. If count equals expected participants and DKG not started:
   - Set `dkg_in_progress` flag to prevent duplicate starts
   - Send `StartDKGProtocol` command
4. Command handler updates UI and triggers protocol

### Expected Behavior
For a 3-node setup (2-of-3 threshold):
1. Each node shows only OTHER participants (2 each)
2. P2P status updates: "0/2" → "1/2" → "2/2"
3. When mesh shows "2/2", DKG automatically starts
4. UI updates to show "Round 1: Generating commitments..."

## Testing

Run three nodes and verify:
```bash
# Terminal 1
RUST_LOG=info cargo run --bin mpc-wallet-tui -- --device-id mpc-1

# Terminal 2  
RUST_LOG=info cargo run --bin mpc-wallet-tui -- --device-id mpc-2

# Terminal 3
RUST_LOG=info cargo run --bin mpc-wallet-tui -- --device-id mpc-3
```

Look for these log messages:
- "📋 Session participants" - Shows participant list
- "Skipping self" - Confirms self exclusion
- "🔗 Mesh status calculation" - Shows connection counts
- "🎯 All participants connected! Starting DKG protocol..." - DKG trigger
- "🚀 Starting DKG protocol - mesh is ready!" - DKG handler

## TODO

The actual DKG protocol implementation needs to be connected in the `StartDKGProtocol` command handler. Currently it only logs and updates the UI but doesn't execute the cryptographic protocol.

To complete the implementation:
1. Call the DKG coordinator's `start_round1` method
2. Broadcast Round 1 commitments through WebRTC data channels
3. Handle incoming DKG messages from peers
4. Progress through Round 2 and finalization

## Summary

This fix ensures:
- Nodes don't show themselves in participant lists
- Mesh counts are accurate (2/2 for 3 nodes)
- DKG automatically starts when all connections are ready
- UI properly reflects connection and protocol status

The P2P mesh now correctly forms and triggers the DKG protocol, though the actual cryptographic implementation still needs to be connected.