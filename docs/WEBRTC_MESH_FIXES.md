# WebRTC P2P Mesh Connection Fixes

## Critical Issues & Solutions

### 1. ICE Candidate Race Condition Fix

**Problem**: ICE candidates being added before remote description is set

**Solution**: Queue ICE candidates until remote description is ready

```rust
// In apps/tui-node/src/elm/command.rs
// Add to the WebRTC handler section

struct PendingIceCandidates {
    candidates: Vec<RTCIceCandidateInit>,
}

// When receiving ICE candidate:
if peer_connection.remote_description().await.is_none() {
    // Queue the candidate
    pending_candidates.entry(device_id.clone())
        .or_insert_with(Vec::new)
        .push(candidate);
    log::info!("🔄 Queued ICE candidate from {} (remote description not ready)", device_id);
} else {
    // Add immediately
    peer_connection.add_ice_candidate(candidate).await?;
    log::info!("✅ Added ICE candidate from {}", device_id);
}

// After setting remote description:
if let Some(pending) = pending_candidates.remove(&device_id) {
    for candidate in pending {
        peer_connection.add_ice_candidate(candidate).await?;
        log::info!("✅ Added queued ICE candidate from {}", device_id);
    }
}
```

### 2. Symmetric Connection Verification

**Problem**: Asymmetric connection status between peers

**Solution**: Implement bidirectional connection verification

```rust
// In apps/tui-node/src/network/webrtc_simple.rs

// Add connection verification
async fn verify_connection(peer: &str, data_channel: &RTCDataChannel) -> bool {
    // Send ping message
    let ping_msg = format!("{{\"type\":\"ping\",\"from\":\"{}\"}}", self_id);
    data_channel.send_text(ping_msg).await.ok();
    
    // Wait for pong response with timeout
    let timeout = Duration::from_secs(5);
    match timeout_future(timeout, wait_for_pong()).await {
        Ok(_) => {
            log::info!("✅ Bidirectional connection verified with {}", peer);
            true
        }
        Err(_) => {
            log::error!("❌ Connection verification failed with {}", peer);
            false
        }
    }
}
```

### 3. Curve Type Consistency

**Problem**: Mismatched curve types during DKG

**Solution**: Validate curve type before joining session

```rust
// In apps/tui-node/src/elm/command.rs

// Before joining session:
if session_info.curve_type != expected_curve {
    log::error!("❌ Curve mismatch: session uses {:?}, expected {:?}", 
                session_info.curve_type, expected_curve);
    return Err("Curve type mismatch - cannot join session");
}

// Store curve type in session state
session_state.curve_type = session_info.curve_type.clone();
```

### 4. IPv6 Handling Improvements

**Problem**: IPv6 address binding failures cluttering logs

**Solution**: Filter IPv6 candidates or handle gracefully

```rust
// In apps/tui-node/src/network/webrtc_simple.rs

// Configure ICE to prefer IPv4
let mut config = RTCConfiguration::default();
config.ice_servers = vec![RTCIceServer {
    urls: vec!["stun:stun.l.google.com:19302".to_owned()],
    ..Default::default()
}];

// Add IP filtering
let setting_engine = SettingEngine::default();
setting_engine.set_network_types(vec![NetworkType::UDP4]); // IPv4 only

let api = APIBuilder::new()
    .with_setting_engine(setting_engine)
    .build();
```

### 5. Enhanced Mesh Status Monitoring

**Problem**: Incomplete mesh formation not clearly reported

**Solution**: Add comprehensive mesh status tracking

```rust
// In apps/tui-node/src/elm/model.rs

#[derive(Debug, Clone)]
pub struct MeshStatus {
    pub expected_peers: usize,
    pub connected_peers: HashMap<String, ConnectionStatus>,
    pub data_channels_open: HashMap<String, bool>,
    pub last_ping_times: HashMap<String, Instant>,
}

#[derive(Debug, Clone)]
pub enum ConnectionStatus {
    Connecting,
    Connected,
    Verified,  // Bidirectional verified
    Failed(String),
}

// In the UI, show clear status:
fn render_mesh_status(&self) -> String {
    let connected = self.mesh_status.connected_peers
        .values()
        .filter(|s| matches!(s, ConnectionStatus::Verified))
        .count();
    
    format!("Mesh Status: {}/{} peers verified", 
            connected, self.mesh_status.expected_peers)
}
```

## Testing Recommendations

1. **Test with consistent configuration**:
   ```bash
   # All nodes use same signal server
   cargo run --bin mpc-wallet-tui -- --signal-server ws://0.0.0.0:9000
   ```

2. **Enable verbose WebRTC logging**:
   ```bash
   RUST_LOG=tui_node::network=debug,webrtc=debug cargo run
   ```

3. **Test connection order**:
   - Start mpc-1 and create session
   - Start mpc-2 and join
   - Wait for connection establishment
   - Start mpc-3 and join
   - Verify full mesh

4. **Monitor mesh formation**:
   - Check logs for "WebRTC mesh is ready"
   - Verify bidirectional data channels
   - Confirm all participants see same connection count

## Quick Fixes to Try Immediately

1. **Increase connection timeout**: The current timeout might be too short
2. **Add retry logic**: Retry failed connections up to 3 times
3. **Use STUN servers**: Add Google STUN servers for better NAT traversal
4. **Synchronize connection attempts**: Use a delay between peer connections

## Configuration File Updates

Create `.env` file for consistent settings:
```env
SIGNAL_SERVER=ws://0.0.0.0:9000
DEFAULT_CURVE=Ed25519
CONNECTION_TIMEOUT=30
MAX_RETRIES=3
ENABLE_IPV6=false
```

## Monitoring Dashboard

Add real-time mesh status to TUI:
```
╭─────────────────────────────────────╮
│ WebRTC Mesh Status                  │
├─────────────────────────────────────┤
│ Signal Server: ✅ Connected         │
│ Participants:  3/3                  │
│                                     │
│ Peer Connections:                   │
│ • mpc-1 → mpc-2: ✅ Verified       │
│ • mpc-1 → mpc-3: ⚠️ Connecting     │
│ • mpc-2 → mpc-3: ❌ Failed         │
│                                     │
│ Data Channels: 1/3 Open            │
│ Last Activity: 2s ago              │
╰─────────────────────────────────────╯
```