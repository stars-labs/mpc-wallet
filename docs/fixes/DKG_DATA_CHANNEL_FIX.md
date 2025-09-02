# DKG Data Channel Issue - Root Cause Analysis and Fix

## Problem Summary
The DKG (Distributed Key Generation) was not starting even when all 3 participants were connected. The logs showed "WebRTC CONNECTED" but the mesh was never becoming ready, preventing DKG from starting.

## Root Cause
The data channels were not being opened between peers, even though WebRTC connections were established. This was due to a logic issue in the offer creation process.

## Technical Details

### Issue 1: Data Channel Creation Without Offer
- Data channels were created in `create_and_setup_device_connection()` 
- But offers were created later in `initiate_offers_for_session()`
- This separation meant the data channel wasn't included in the SDP offer

### Issue 2: Offer Creation Logic Bug
In `initiate_offers_for_session()`, the code checked if negotiation was needed:
```rust
let negotiation_needed = match current_state {
    RTCPeerConnectionState::New | ... => true,
    _ => false, // This was wrong!
};
```

The problem: When the peer connection was in "Connecting" state (which happens right after creation), it would skip creating the offer, thinking negotiation wasn't needed.

## Fixes Applied

### Fix 1: Updated Offer Creation Logic
```rust
// We need to create an offer if:
// 1. Connection is new/closed/failed/disconnected
// 2. Connection is connecting but we haven't sent an offer yet (signaling state is stable)
let negotiation_needed = match current_state {
    RTCPeerConnectionState::New
    | RTCPeerConnectionState::Closed
    | RTCPeerConnectionState::Disconnected
    | RTCPeerConnectionState::Failed => true,
    RTCPeerConnectionState::Connecting => {
        // If we're connecting but signaling is stable, we haven't sent an offer yet
        matches!(signaling_state, 
            webrtc::peer_connection::signaling_state::RTCSignalingState::Stable)
    },
    _ => false,
};
```

### Fix 2: Enhanced Debug Logging
Added detailed logging throughout the data channel creation and callback setup process:
- Data channel creation confirmation with Arc pointer addresses
- on_open callback triggering
- on_data_channel callback for responder side
- ReportChannelOpen command sending

### Fix 3: Session Update Broadcasting
Fixed the SessionUpdate broadcast logic to properly include newly joined participants (previously fixed).

### Fix 4: UI State Transitions
Fixed the TUI to properly transition from JoinWallet state to Normal state after successfully joining a session.

## Verification Steps
1. Start 3 MPC nodes (mpc-1, mpc-2, mpc-3)
2. Create a DKG session with mpc-1 (threshold 2 of 3)
3. Join the session with mpc-2 and mpc-3
4. Verify:
   - All nodes show "3/3 participants"
   - WebRTC connections establish ("WebRTC CONNECTED")
   - Data channels open (look for "DATA CHANNEL OPENED" logs)
   - Mesh becomes ready (MeshStatus::Ready)
   - Identifier map is created
   - DKG Round 1 starts automatically

## Key Debug Commands
```bash
# Check data channel events
grep -E "DATA CHANNEL|on_data_channel|on_open" mpc-*.log

# Check mesh readiness
grep -E "Mesh.*ready|identifier.*map" mpc-*.log

# Check DKG trigger
grep -E "CheckAndTriggerDkg|TriggerDkgRound1" mpc-*.log
```

## Lessons Learned
1. WebRTC data channels must be created before creating offers
2. Peer connection states can transition quickly - don't assume "New" state
3. The "politeness" pattern in WebRTC requires careful state management
4. Comprehensive debug logging is essential for distributed systems debugging

## Future Improvements
1. Consider creating data channels and offers in a single atomic operation
2. Add integration tests for the complete DKG flow
3. Implement better WebRTC state machine tracking
4. Add metrics/telemetry for connection establishment timing