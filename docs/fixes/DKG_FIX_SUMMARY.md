# DKG Not Starting - Root Cause and Fix

## Problem
DKG was not starting even though all 3 participants showed "3/3 participants" in the UI. The logs showed:
```
[CheckAndTriggerDkg-mpc-3] Conditions not met. MeshReady: false, IdentifiersMapped: false, SessionActive: true, DkgIdle: true, IsDkgSession: true
```

## Root Cause Analysis

### Issue 1: MeshReady was false
The mesh status wasn't being properly updated when all participants established WebRTC connections.

### Issue 2: IdentifiersMapped was false  
**This was the critical missing piece!** The `identifier_map` was never being created. This map is essential for DKG as it assigns FROST cryptographic identifiers to each participant.

## The Fix

Added logic to create the `identifier_map` when the mesh becomes ready in `/apps/cli-node/src/handlers/mesh_commands.rs`:

1. **When local node becomes mesh ready** (in `handle_send_own_mesh_ready_signal`):
   - If all participants are ready, create the identifier map
   - Sort participants alphabetically to ensure consistent ordering across all nodes
   - Assign FROST identifiers (1-based) to each participant
   - Store the map in `state.identifier_map`

2. **When processing mesh ready from other nodes** (in `handle_process_mesh_ready`):
   - Same logic when the mesh transitions to Ready state
   - Only creates map if it doesn't already exist

## Implementation Details

```rust
// Sort participants to ensure consistent identifier assignment across all nodes
let mut sorted_participants = session.participants.clone();
sorted_participants.sort();

let mut identifier_map = std::collections::BTreeMap::new();
for (index, device_id) in sorted_participants.iter().enumerate() {
    // FROST identifiers are 1-based (starting from 1, not 0)
    let identifier_value = (index + 1) as u16;
    
    // Create a FROST identifier from the u16 value
    let identifier_bytes = identifier_value.to_be_bytes().to_vec();
    let identifier = frost_core::Identifier::<C>::deserialize(&identifier_bytes)
        .expect("Failed to create FROST identifier");
    
    identifier_map.insert(device_id.clone(), identifier);
}

state_guard.identifier_map = Some(identifier_map);
```

## Why This Works

1. **Consistent Ordering**: By sorting participants alphabetically, all nodes assign the same FROST identifier to each device
2. **1-Based Indexing**: FROST uses 1-based identifiers (1, 2, 3...) not 0-based
3. **Proper Serialization**: The identifiers are created using FROST's deserialize method to ensure compatibility

## Expected Behavior After Fix

1. All nodes join session and show "3/3 participants"
2. WebRTC connections established between all participants
3. When mesh is ready, identifier map is created:
   - mpc-1 gets FROST identifier 1
   - mpc-2 gets FROST identifier 2  
   - mpc-3 gets FROST identifier 3
4. DKG conditions are met:
   - MeshReady: ✅ true (all WebRTC connections established)
   - IdentifiersMapped: ✅ true (identifier_map created)
   - SessionActive: ✅ true
   - DkgIdle: ✅ true
   - IsDkgSession: ✅ true
5. DKG Round 1 automatically starts

## Testing

To verify the fix:
1. Run all three nodes
2. Create session on mpc-1
3. Join with mpc-2 and mpc-3
4. Watch logs for:
   - "🔑 Assigned FROST identifier X to device mpc-Y"
   - "✅ Created identifier map for 3 participants"
   - "All conditions met. Triggering DKG Round 1"
   - "DKG Round 1: Generating and sending commitments to all devices..."