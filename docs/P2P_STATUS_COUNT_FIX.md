# P2P Status Count Fix - Complete Solution

## Problem
The P2P status in the DKG Progress UI was showing "0/2" despite WebRTC connections being established and data channels opening. The issue had two parts:
1. WebRTC status updates weren't updating the UI component
2. The counts were comparing against wrong totals

## Root Causes

### Issue 1: Status Updates Not Applied
- `Message::UpdateParticipantWebRTCStatus` was received but only triggered a UI remount
- The actual component's participant list wasn't being updated

### Issue 2: Incorrect Count Comparison
- `total_participants` = 3 (includes self)
- WebRTC connections are only tracked for OTHER participants (2 in a 3-node setup)
- UI was comparing 2 connections against 3 total, showing "0/3" or incorrect ratios

## Solution

### Part 1: Update Component State
In `elm/update.rs`, changed the handler to actually update the component:
```rust
Message::UpdateParticipantWebRTCStatus { device_id, webrtc_connected, data_channel_open } => {
    // Update the DKG progress component if it's active
    if let Screen::DKGProgress { ref mut component, .. } = model.current_screen {
        component.update_webrtc_status(device_id, webrtc_connected, data_channel_open);
        None
    } else {
        None
    }
}
```

### Part 2: Fix Count Display
In `elm/components/dkg_progress.rs`, adjusted counts to compare against other participants only:
```rust
// total_participants includes self, but we only track connections to OTHER participants
let other_participants = self.total_participants.saturating_sub(1);

// Display counts against other_participants
format!("WebRTC: {}/{} | Channels: {}/{} | Mesh: {}/{}",
        webrtc_connected, other_participants,
        data_channels_open, other_participants,
        self.mesh_ready_count, other_participants)
```

### Part 3: Fix "All Connected" Logic
Updated the check for all data channels being open:
```rust
let expected_other_participants = self.total_participants.saturating_sub(1) as usize;
self.all_data_channels_open = self.participants.len() >= expected_other_participants &&
    self.participants.iter().all(|p| p.data_channel_open);
```

## Files Modified

1. **`elm/update.rs`**
   - Line 289-295: Changed handler to call `component.update_webrtc_status()`

2. **`elm/components/dkg_progress.rs`**
   - Line 409: Calculate `other_participants` for correct comparison
   - Lines 415-418: Display counts against `other_participants` instead of `total_participants`
   - Lines 154-156: Fix "all channels open" check to use `expected_other_participants`

## Expected Behavior

For a 3-node setup (mpc-1, mpc-2, mpc-3):
- Each node connects to 2 other nodes
- P2P Status should show "2/2" when fully connected
- Channels should show "2/2" when all data channels are open
- Mesh should show "2/2" when mesh ready signals received

## How It Works

1. **Participant Tracking**:
   - `total_participants` = 3 (includes self)
   - `other_participants` = 2 (excludes self)
   - WebRTC connections tracked only for other participants

2. **Status Updates Flow**:
   - WebRTC connection established → `UpdateParticipantWebRTCStatus` message
   - Message handler updates component state
   - Component recalculates counts and displays correct ratios

3. **Display Logic**:
   - Green when all connections/channels established
   - Yellow when some connections established
   - Red when no connections

## Testing
After this fix:
1. Start 3 nodes
2. P2P Status should show correct counts (e.g., "2/2" not "2/3")
3. Status should update in real-time as connections establish
4. All participants should show "Channel Open" status when mesh forms