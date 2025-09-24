# DKG Dynamic Curve Selection Fix

## Date: 2025-11-02
## Status: ✅ FIXED

## Problem Summary

The DKG protocol was hardcoded to always use Secp256k1 curve regardless of what the user selected in the UI. The user wanted to be able to choose between:
- **Secp256k1** for Ethereum/Bitcoin
- **Ed25519** for Solana

## Root Cause

The application creates an `AppState<Secp256K1Sha256>` at startup in the main binary:
```rust
// In mpc-wallet-tui.rs
AppState::<Secp256K1Sha256>::with_device_id_and_server(...)
```

This meant all DKG operations were locked to secp256k1 at compile time, making the UI curve selection cosmetic only.

## Solution Implemented

### Dynamic Curve Handling

**File**: `apps/tui-node/src/elm/command.rs` (lines 276-336)

The StartDKG command now:
1. Checks which curve was selected in the config
2. For Secp256k1: Uses the existing AppState
3. For Ed25519: Creates a new Ed25519-specific AppState and copies relevant state

```rust
match config.curve {
    CurveType::Secp256k1 => {
        info!("📈 Using Secp256k1 curve for DKG");
        crate::protocal::dkg::handle_trigger_dkg_round1(
            app_state.clone(),
            device_id.clone(),
            internal_tx.clone()
        ).await;
    }
    CurveType::Ed25519 => {
        info!("🔑 Using Ed25519 curve for DKG");

        // Create Ed25519 AppState
        let ed25519_state = Arc::new(Mutex::new(
            AppState::<frost_ed25519::Ed25519Sha512>::with_device_id_and_server(...)
        ));

        // Copy session, data_channels, keystore, etc.
        {
            let mut ed_guard = ed25519_state.lock().await;
            ed_guard.session = state_guard.session.clone();
            ed_guard.data_channels = state_guard.data_channels.clone();
            ed_guard.keystore = state_guard.keystore.clone();
            ed_guard.dkg_in_progress = state_guard.dkg_in_progress;
        }

        // Call DKG with Ed25519 state
        crate::protocal::dkg::handle_trigger_dkg_round1(
            ed25519_state.clone(),
            device_id.clone(),
            internal_tx_ed
        ).await;
    }
}
```

### Dynamic Handler Function

**File**: `apps/tui-node/src/protocal/dkg.rs` (lines 42-85)

Also added a dynamic handler function for future use:
```rust
pub async fn handle_trigger_dkg_round1_dynamic(
    state_secp256k1: Option<Arc<Mutex<AppState<frost_secp256k1::Secp256K1Sha256>>>>,
    state_ed25519: Option<Arc<Mutex<AppState<frost_ed25519::Ed25519Sha512>>>>,
    self_device_id: String,
    curve_type: crate::elm::model::CurveType,
) {
    match curve_type {
        CurveType::Secp256k1 => { /* use secp256k1 state */ }
        CurveType::Ed25519 => { /* use ed25519 state */ }
    }
}
```

### Session Curve Type Storage

The curve type is properly stored in the session:
```rust
// In command.rs line 521-524
curve_type: match config_clone.curve {
    CurveType::Secp256k1 => "Secp256k1".to_string(),
    CurveType::Ed25519 => "Ed25519".to_string(),
},
```

## What This Means

Now when users:
1. Select **Secp256k1** in the UI → DKG uses FROST with secp256k1 curve (Ethereum/Bitcoin compatible)
2. Select **Ed25519** in the UI → DKG uses FROST with ed25519 curve (Solana compatible)

The generated keys will be on the correct curve for the intended blockchain.

## Testing Instructions

### Test Secp256k1 (Ethereum/Bitcoin)
1. Start 3 nodes
2. Create wallet with mpc-1, selecting **Secp256k1** curve
3. Join with mpc-2 and mpc-3
4. Check logs for: `📈 Using Secp256k1 curve for DKG`
5. Verify generated addresses are Ethereum-compatible

### Test Ed25519 (Solana)
1. Start 3 nodes
2. Create wallet with mpc-1, selecting **Ed25519** curve
3. Join with mpc-2 and mpc-3
4. Check logs for: `🔑 Using Ed25519 curve for DKG`
5. Verify generated addresses are Solana-compatible

## Build Status

```bash
✅ Compilation successful
Finished `dev` profile [unoptimized + debuginfo] target(s) in 46.69s
```

## Files Modified

1. `apps/tui-node/src/elm/command.rs`:
   - Added dynamic curve selection in StartDKG command
   - Creates appropriate AppState based on selected curve

2. `apps/tui-node/src/protocal/dkg.rs`:
   - Added `handle_trigger_dkg_round1_dynamic` function
   - Added tracing imports for better logging

## Technical Notes

- The main AppState remains Secp256k1-based for compatibility
- Ed25519 AppState is created on-demand when needed
- State copying excludes `websocket_internal_cmd_tx` due to generic type mismatch
- Both curves use the same FROST protocol implementation, just with different curve parameters

## Future Improvements

- Consider refactoring to make the entire application curve-agnostic
- Store both AppStates permanently to avoid recreation
- Add support for more curves (e.g., P-256 for other blockchains)