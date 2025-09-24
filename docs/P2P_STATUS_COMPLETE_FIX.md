# P2P Status Display - Complete Fix

## Problem
The P2P status was showing "0/2" despite WebRTC connections being established. The issue had multiple layers:
1. UI messages weren't reaching the component
2. Component state wasn't being updated
3. Counts were comparing against wrong totals

## Solution Overview

### 1. Added UI Message Sending (webrtc_simple.rs)
- Added `ui_msg_tx` parameter to send status updates
- Send `UpdateParticipantWebRTCStatus` when connections change
- Send updates when data channels open

### 2. Store WebRTC Status in Model (model.rs)
Added to NetworkState:
```rust
pub participant_webrtc_status: HashMap<String, (bool, bool)>
```
Stores `(webrtc_connected, data_channel_open)` for each participant

### 3. Update Model on Status Change (update.rs)
```rust
Message::UpdateParticipantWebRTCStatus { device_id, webrtc_connected, data_channel_open } => {
    model.network_state.participant_webrtc_status
        .entry(device_id.clone())
        .and_modify(|status| {
            status.0 = webrtc_connected;
            status.1 = data_channel_open;
        })
        .or_insert((webrtc_connected, data_channel_open));
}
```

### 4. Use Stored Status in UI (app.rs)
When creating DKGProgressComponent:
```rust
if let Some(&(webrtc_connected, data_channel_open)) = 
    self.model.network_state.participant_webrtc_status.get(participant) {
    dkg_progress.update_webrtc_status(
        participant.clone(),
        webrtc_connected,
        data_channel_open
    );
}
```

### 5. Fix Count Display (dkg_progress.rs)
```rust
// total_participants includes self, but we only track connections to OTHER participants
let other_participants = self.total_participants.saturating_sub(1);

format!("WebRTC: {}/{} | Channels: {}/{}",
        webrtc_connected, other_participants,
        data_channels_open, other_participants)
```

## Complete Data Flow

1. **WebRTC Connection Established**
   - `webrtc_simple.rs` detects state change
   - Sends `Message::UpdateParticipantWebRTCStatus`

2. **Message Processing**
   - `update.rs` receives message
   - Stores status in `model.network_state.participant_webrtc_status`

3. **UI Rendering**
   - `app.rs` creates DKGProgressComponent
   - Reads status from model
   - Updates component with actual WebRTC status

4. **Display Logic**
   - Component counts connections to OTHER participants
   - Shows "2/2" for 3-node setup (not "2/3")
   - Color codes based on connection state

## Files Modified

1. `network/webrtc_simple.rs` - Send UI updates
2. `network/websocket_sender.rs` - Debug logging
3. `elm/model.rs` - Add status storage
4. `elm/update.rs` - Store status in model
5. `elm/app.rs` - Use stored status
6. `elm/components/dkg_progress.rs` - Fix count display
7. `elm/command.rs` - Pass UI sender, fix warnings
8. `handlers/session_handler.rs` - Update call signatures
9. `handlers/session_rejoin.rs` - Update call signatures

## Testing
The fix ensures:
- P2P status shows correct counts (e.g., "2/2")
- Updates happen in real-time
- Status persists across UI refreshes
- Proper color coding (green/yellow/red)

## Key Insight
The component is recreated on each render, so status must be stored in the Model and applied when creating the component, not updated directly on the component.