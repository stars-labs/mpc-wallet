# Mesh Status Display Fix

## Problem
The P2P mesh status was showing incorrect counts (e.g., "Mesh: 0/2") even when WebRTC connections were established. The mesh ready count was never being calculated or updated.

## Root Cause
The `update_mesh_status` method in `DKGProgressComponent` was never being called, so the mesh ready count always remained at 0.

## Solution

### 1. Added Mesh Calculation in app.rs
When mounting the DKGProgress component, now calculate the mesh status based on actual WebRTC connections:

```rust
// Calculate and update mesh status if we have an active session
if let Some(ref session) = self.model.active_session {
    // Count how many participants have data channels open (excluding self)
    let mesh_ready_count = session.participants.iter()
        .filter(|p| **p != self.model.device_id) // Exclude self
        .filter(|p| {
            self.model.network_state.participant_webrtc_status.get(*p)
                .map_or(false, |(_, data_channel_open)| *data_channel_open)
        })
        .count();
    
    // Check if all expected participants have data channels open
    let expected_other_participants = (total_participants as usize).saturating_sub(1);
    let all_connected = mesh_ready_count >= expected_other_participants;
    
    // Update mesh status in the component
    dkg_progress.update_mesh_status(mesh_ready_count, all_connected);
}
```

### 2. Force UI Remount on WebRTC Status Updates
Modified the update handler to force a remount when WebRTC status changes:

```rust
Message::UpdateParticipantWebRTCStatus { device_id, webrtc_connected, data_channel_open } => {
    // ... store status ...
    
    // Force a remount to update the display with new WebRTC status
    if matches!(model.current_screen, Screen::DKGProgress { .. }) {
        Some(Command::SendMessage(Message::ForceRemount))
    } else {
        None
    }
}
```

## How It Works

1. **WebRTC Status Updates**: When a connection state changes, `UpdateParticipantWebRTCStatus` message is sent
2. **Status Storage**: The status is stored in `model.network_state.participant_webrtc_status`
3. **Force Remount**: If on DKGProgress screen, a `ForceRemount` message is sent
4. **Mesh Calculation**: When remounting, the mesh ready count is calculated from stored status
5. **UI Update**: The component displays the correct mesh count

## Expected Display

For a 3-node setup (2-of-3 threshold):
- Initial: "Mesh: 0/2"
- One peer connected: "Mesh: 1/2"
- All peers connected: "Mesh: 2/2"

The mesh count represents connections to OTHER participants (excludes self), which is why it shows "2/2" for 3 total participants.

## Files Modified

1. `/apps/tui-node/src/elm/app.rs` - Added mesh calculation when mounting DKGProgress
2. `/apps/tui-node/src/elm/update.rs` - Force remount on WebRTC status changes

## Testing

To verify the fix:
1. Start three nodes (mpc-1, mpc-2, mpc-3)
2. Create DKG session on mpc-1
3. Join with mpc-2 and mpc-3
4. Observe mesh status updates from "0/2" → "1/2" → "2/2"

The mesh status should now accurately reflect the number of established WebRTC data channels.