# 🎯 ROOT CAUSE FOUND & FIXED: Race Condition in DKG Start

## The Problem
DKG was not starting because of a **race condition** where `dkg_in_progress` flag was set BEFORE the actual DKG command could run.

## The Bug Flow (BEFORE FIX)

1. **update.rs** detects mesh is ready
2. **update.rs** sets `dkg_in_progress = true` ❌ (TOO EARLY!)
3. **update.rs** sends `Command::StartDKGProtocol` 
4. **command.rs** receives `StartDKGProtocol`
5. **command.rs** checks `dkg_in_progress` - sees it's already true
6. **command.rs** logs "DKG already running" and returns early ❌
7. **RESULT**: DKG never actually starts!

## The Fixed Flow (AFTER FIX)

1. **update.rs** detects mesh is ready
2. **update.rs** sends `Command::StartDKGProtocol` (WITHOUT setting flag)
3. **command.rs** receives `StartDKGProtocol`
4. **command.rs** checks `DkgState` - sees it's Idle
5. **command.rs** sets `dkg_in_progress = true` ✅ (CORRECT TIME)
6. **command.rs** triggers actual DKG protocol
7. **RESULT**: DKG starts successfully!

## The Code Fix

### In `/apps/tui-node/src/elm/update.rs` (line 326):
```rust
// BEFORE (BUG):
if should_start_dkg {
    info!("🎯 All participants connected! Starting DKG protocol...");
    model.wallet_state.dkg_in_progress = true;  // ❌ TOO EARLY!
    Some(Command::StartDKGProtocol)
}

// AFTER (FIXED):
if should_start_dkg {
    info!("🎯 All participants connected! Starting DKG protocol...");
    // DON'T set dkg_in_progress here - let the command handler do it
    // to avoid race condition where it's set before the command runs
    Some(Command::StartDKGProtocol)
}
```

## How to Test

1. **Rebuild**: Already done - `cargo build --bin mpc-wallet-tui`
2. **Start 3 terminals** with the new binary:
   ```bash
   # Terminal 1
   cargo run --bin mpc-wallet-tui -- --device-id mpc-1
   
   # Terminal 2 
   cargo run --bin mpc-wallet-tui -- --device-id mpc-2
   
   # Terminal 3
   cargo run --bin mpc-wallet-tui -- --device-id mpc-3
   ```

3. **Start DKG** in mpc-1 (press 1 for "Create New Wallet")

4. **Verify it works** - you should see in the logs:
   - "🚀 Starting DKG protocol - mesh is ready!"
   - "🔄 Starting actual DKG Round 1 protocol..."
   - NO MORE "DKG protocol already running" errors!

## Why This Happened

This is a classic **race condition** bug where state was being set in the wrong layer:
- The **update layer** (UI state management) was setting a flag
- The **command layer** (business logic) was checking that same flag
- The flag was set BEFORE the command ran, causing the command to think it was already done

The fix: Let the command layer manage its own state!