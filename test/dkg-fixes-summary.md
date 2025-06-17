# DKG Synchronization Fixes Summary

This document summarizes all the fixes made to resolve the DKG synchronization issues where mpc-2 was stuck and not sending Round 2 packages.

## Issues Found and Fixed

### 1. Missing acceptSession Handler Registration
**Problem**: The offscreen document wasn't registering the acceptSession handler, causing "No handler registered" errors.
**Fix**: Added `messageRouter.registerHandler('acceptSession', handleAcceptSession)` in offscreen/index.ts

### 2. Buffered Packages Cleared During DKG Init
**Problem**: `_resetDkgState()` was clearing buffered Round 1 packages during DKG initialization, causing them to be lost.
**Fix**: Save buffered packages before reset and restore them after:
```typescript
const savedRound1Packages = [...this.bufferedRound1Packages];
const savedRound2Packages = [...this.bufferedRound2Packages];
this._resetDkgState();
this.bufferedRound1Packages = savedRound1Packages;
this.bufferedRound2Packages = savedRound2Packages;
```

### 3. Replay Logic Not Triggering Round 2
**Problem**: After replaying buffered Round 1 packages, the system wasn't checking if it should proceed to Round 2.
**Fix**: Added Round 2 transition check after replay:
```typescript
if (hasAllPackages && finalCanStart) {
  this._log(`🔄 All Round 1 packages received after replay. Moving to Round 2.`);
  this._updateDkgState(DkgState.Round2InProgress);
  await this._generateAndBroadcastRound2();
}
```

### 4. Own Round 1 Package Not Added to WASM (Critical Fix)
**Problem**: The node's own Round 1 package was never added to the FROST DKG WASM instance, causing `can_start_round2()` to return false.
**Fix**: Explicitly add own package after generation:
```typescript
const myIndex = (this.sessionInfo?.participants.indexOf(this.localPeerId) ?? -1) + 1;
this.frostDkg.add_round1_package(myIndex, round1Package);
```

## Root Cause
The FROST WASM implementation requires ALL packages (including the node's own package) to be explicitly added via `add_round1_package()`. The assumption that `generate_round1()` automatically includes the own package was incorrect.

## Testing
After these fixes, mpc-2 should:
1. Buffer Round 1 packages that arrive early
2. Generate and add its own Round 1 package to WASM
3. Replay buffered packages after DKG init
4. Check and proceed to Round 2 when all packages are ready
5. Generate and broadcast its Round 2 package
6. Complete DKG successfully with all peers

## Key Logs to Verify Success
- "Adding own Round 1 package to FROST DKG with index 2"
- "Successfully added own Round 1 package. Total: 1"
- "WASM can_start_round2=true" (after receiving all packages)
- "All Round 1 packages received and can proceed. Moving to Round 2."
- "Broadcasting Round 2 package to 2 peers"
- "DKG completed successfully. Group public key: ..."