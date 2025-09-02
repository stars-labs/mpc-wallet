# Final DKG Fix - FROST Identifier Creation

## Problem
DKG was not starting even after all 3 participants joined and showed "Participants (3/3)". The identifier_map was being created incorrectly, causing FROST to fail when trying to use the identifiers.

## Root Cause
The FROST identifier creation was using only 2 bytes (u16.to_be_bytes()) when FROST actually requires a 32-byte array with the u16 value placed at the end (bytes 30-31).

## The Fix

### Before (Incorrect):
```rust
let identifier_bytes = identifier_value.to_be_bytes().to_vec();
let identifier = frost_core::Identifier::<C>::deserialize(&identifier_bytes)
    .expect("Failed to create FROST identifier");
```

### After (Correct):
```rust
// FROST identifiers require a 32-byte array with the u16 value at the end
let bytes = identifier_value.to_be_bytes();
let mut padded_bytes = [0u8; 32];
padded_bytes[30] = bytes[0];
padded_bytes[31] = bytes[1];

let identifier = frost_core::Identifier::<C>::deserialize(&padded_bytes)
    .expect("Failed to create FROST identifier");
```

## Why This Works
- FROST identifiers are represented as 32-byte arrays (256 bits)
- The actual identifier value (1, 2, 3...) is stored in the last 2 bytes
- The rest of the bytes are padded with zeros
- This matches the format expected by the FROST library's deserialize method

## Expected Behavior After Fix

1. **All nodes join session** → Show "Participants (3/3)"
2. **WebRTC connections established** → All show "Connected"
3. **Mesh becomes ready** → Triggers identifier map creation
4. **Identifier map created correctly**:
   - mpc-1 → FROST Identifier 1 (0x00...0001)
   - mpc-2 → FROST Identifier 2 (0x00...0002)
   - mpc-3 → FROST Identifier 3 (0x00...0003)
5. **DKG conditions check passes**:
   - MeshReady: ✅ true
   - IdentifiersMapped: ✅ true
   - SessionActive: ✅ true
   - DkgIdle: ✅ true
   - IsDkgSession: ✅ true
6. **DKG Round 1 starts automatically**:
   - "All conditions met. Triggering DKG Round 1"
   - Each node generates and broadcasts commitments
   - Proceeds through DKG rounds to generate key shares

## Files Modified
- `/apps/cli-node/src/handlers/mesh_commands.rs` (lines 156-161 and 338-343)

## Testing
Run all three nodes and observe:
1. "🔑 Assigned FROST identifier 1 to device mpc-1"
2. "✅ Created identifier map for 3 participants"
3. "All conditions met. Triggering DKG Round 1"
4. "DKG Round 1: Generating and sending commitments to all devices..."
5. DKG should complete successfully with all nodes receiving their key shares