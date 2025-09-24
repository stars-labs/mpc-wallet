# WebRTC Full Mesh Formation Fix

## Problem
The WebRTC mesh was not forming completely. Only mpc-1 (session creator) was connecting to other nodes, but mpc-2 and mpc-3 were not establishing connections with each other, preventing the full mesh from forming.

## Root Causes
1. **Incomplete re-initiation logic**: When new participants joined, only the session creator was re-initiating WebRTC connections
2. **Missing mesh verification**: No automatic verification and retry mechanism for incomplete connections
3. **Timing issues**: Nodes were not re-initiating connections when other participants joined

## Solutions Implemented

### 1. Enhanced StartDKG WebSocket Handler (command.rs lines 509-584)
- Added `prev_count` tracking to detect new participants
- Implemented re-initiation on EACH new participant join (not just when all join)
- Added final mesh formation check when all participants connect
- Scheduled mesh verification after 2 seconds delay

### 2. Added InitiateWebRTCConnections Command Handler (command.rs lines 1843-1895)
- Filters out self from participants list
- Spawns WebRTC initiation task
- Automatically triggers mesh verification after initiation
- Uses `simple_initiate_webrtc_with_channel` for connection establishment

### 3. Enhanced VerifyMeshConnectivity Message Handler (update.rs lines 432-463)
- Checks active session for expected connections
- Compares connected peers vs expected participants
- Re-initiates connections if mesh is incomplete
- Triggers DKG protocol when mesh is complete

### 4. Improved VerifyWebRTCMesh and EnsureFullMesh Commands (command.rs lines 2013-2177)
- Verifies current mesh status against expected connections
- Identifies missing connections specifically
- Re-initiates connections only to missing participants
- Provides detailed logging for debugging

### 5. Fixed JoinDKG Path (command.rs lines 1284-1300)
- Added same re-initiation logic as StartDKG
- Tracks new participants and triggers WebRTC connections
- Ensures all joining nodes participate in mesh formation

## Key Changes Summary

1. **command.rs**:
   - Added re-initiation on each new participant in StartDKG
   - Implemented InitiateWebRTCConnections handler
   - Enhanced mesh verification commands
   - Fixed participant filtering to exclude self

2. **update.rs**:
   - Enhanced VerifyMeshConnectivity with smart re-initiation
   - Fixed field access (using `model.active_session` instead of incorrect path)
   - Added automatic DKG triggering when mesh completes

3. **app.rs**:
   - Fixed participant display to skip self
   - Corrected mesh ready count calculation

4. **dkg_progress.rs**:
   - Fixed "Waiting for participant" placeholder count
   - Used `total_participants - 1` for expected other participants

## Result
The WebRTC mesh now forms completely with all nodes connecting to all other nodes. The system automatically:
- Re-initiates connections when new participants join
- Verifies mesh completeness
- Retries failed connections
- Starts DKG automatically when mesh is ready

## Testing
Run with 3 nodes:
```bash
# Terminal 1
RUST_LOG=info cargo run --bin mpc-wallet-tui -- --signal-server ws://0.0.0.0:9000 --device-id mpc-1

# Terminal 2
RUST_LOG=info cargo run --bin mpc-wallet-tui -- --signal-server ws://0.0.0.0:9000 --device-id mpc-2

# Terminal 3
RUST_LOG=info cargo run --bin mpc-wallet-tui -- --signal-server ws://0.0.0.0:9000 --device-id mpc-3
```

All nodes should now show 2/2 connections and form a complete mesh.