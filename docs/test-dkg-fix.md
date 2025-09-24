# DKG Participant Count Fix Test Plan

## Issue Identified
DKG was starting prematurely with insufficient participants due to multiple trigger points:

1. **Wrong trigger**: "Connected to all 0 other participants" at 08:54:21.453696
2. **Correct trigger**: "connected=2/2, current_participants=3/3" at 08:54:26.230905

## Fix Applied
- ✅ Fixed update.rs lines 310-324 to use `session.total` instead of current participant count
- ✅ Added proper participant count checking: `current_total_participants >= required_total_participants`
- ✅ Added participant change handling in command.rs lines 714-746

## Current Status
The fix is implemented but logs show old code was still running during the last test. The current binary should be correct.

## Test Plan
1. Ensure fresh build: `cargo build --bin mpc-wallet-tui`
2. Start all 3 terminals simultaneously
3. Verify DKG waits for all 3 participants before starting
4. Check logs show correct trigger logic only

## Expected Behavior
- DKG should NOT start until all 3 participants are connected
- Log should show: "connected=2/2, current_participants=3/3" before DKG starts
- No premature "Connected to all 0 other participants" triggers

## Verification Commands
```bash
# Check for old problematic messages
grep "Connected to all 0 other participants" mpc-wallet-mpc-*.log

# Check for correct trigger sequence  
grep "current_participants=3/3" mpc-wallet-mpc-*.log
```