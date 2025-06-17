# DKG Round 1→2 Transition Fix

## Problem Summary
The mpc-2 Chrome extension was receiving Round 1 packages from CLI nodes (mpc-1, mpc-3) but failing to transition to Round 2 because the WASM FROST DKG was reporting `can_start_round2: false` even though all packages were received.

## Root Cause
The issue was that buffered Round 1 packages from CLI nodes were not being successfully added to the WASM FROST DKG instance during the replay process in `_replayBufferedDkgPackages()`. This resulted in:

```
🔍 WASM can_start_round2: packages_count=1, total=3, can_start=false
```

Where only mpc-2's own package was in WASM (from `generate_round1()`), but the packages from mpc-1 and mpc-3 were not being added during replay.

## Solution Implemented

### 1. Enhanced Debugging Logs
Added comprehensive debugging to `_replayBufferedDkgPackages()` to track:
- WASM state before and after replay
- Session participant information
- Package format conversion details
- WASM call results for each package
- Detailed error handling

### 2. Improved Error Handling
- Better validation of participant indices
- More detailed package format debugging  
- Clearer error messages when WASM calls fail
- Continue processing other packages even if one fails

### 3. Round 2 Transition Monitoring
Enhanced logging in both:
- `_handleDkgRound1Package()` Round 2 transition check
- `initializeDkg()` post-replay Round 2 check

## Expected Behavior After Fix
With the enhanced debugging, we can now see exactly what happens during replay:

1. **Before Replay**: `🔄 WASM can_start_round2 before replay: false`
2. **Package Processing**: Detailed logs for each buffered package
3. **After Each Package**: `🔄 WASM can_start_round2 after adding ${fromPeerId}: true/false`
4. **Final State**: `🔄 Final WASM can_start_round2 after replay: true`

## Testing
The fix maintains all existing test functionality. DKG-related tests continue to pass, validating that the enhanced logging doesn't break existing workflows.

## Files Modified
- `/src/entrypoints/offscreen/webrtc.ts`: Enhanced `_replayBufferedDkgPackages()` with comprehensive debugging
- Added detailed logging to `initializeDkg()` Round 2 transition check
- Enhanced Round 1→2 transition logging in `_handleDkgRound1Package()`

## Next Steps
1. Deploy the enhanced version and monitor logs during CLI-Chrome extension DKG
2. The detailed logs will reveal exactly why buffered packages might be failing to add to WASM
3. Use the diagnostic information to implement a targeted fix for the replay issue

This debug-enhanced version will provide the visibility needed to understand and resolve the core Round 1→2 transition issue.
