# Critical Bug Fix: Session Discovery Auto-Join Issue

## Problem
**Critical Bug**: When browsing available sessions, nodes were automatically joining the session without user consent. 

### Symptoms:
- mpc-2 and mpc-3 appeared as "Connected" in the DKG Progress screen while still on the Session Discovery screen
- Ghost participants with random IDs like "mpc-fc567c6a" and "mpc-33016cac" appeared in participant lists
- Nodes were counted as participants before actually selecting "Join Session"

## Root Causes

### 1. Temporary Discovery IDs (Fixed Earlier)
- The code was creating random temporary IDs for session discovery
- These temporary IDs were being counted as separate participants
- **Fix**: Use real device IDs for discovery (already fixed)

### 2. Auto-Registration During Discovery (Fixed Now)
- The discovery code was calling `ClientMsg::Register` just to browse sessions
- Registration adds the device to the connected devices list
- The server broadcasts this to all participants, making it appear the device has joined
- **Critical Security Issue**: Users were unknowingly joining sessions

## The Fix

Changed session discovery to NOT register the device:

```rust
// OLD (BAD) - Registers device just for browsing
let register_msg = serde_json::json!({
    "type": "register", 
    "device_id": device_id
});

// NEW (GOOD) - Just requests sessions without registering
// DO NOT register for discovery - just request sessions
// Registration should only happen when actually joining a session
let request_msg = serde_json::json!({
    "type": "request_active_sessions"
});
```

## Impact

### Before Fix:
1. User selects "Join Session" to browse
2. Node automatically registers with server
3. Node appears as "Connected" to other participants
4. User hasn't even selected a session yet!
5. Privacy and security compromised

### After Fix:
1. User selects "Join Session" to browse
2. Node requests session list WITHOUT registering
3. Node remains anonymous while browsing
4. Only registers when user explicitly selects a session and confirms join
5. Proper consent-based participation

## Testing

After this fix:
- Nodes can browse available sessions without appearing as participants
- Only the actual participants who explicitly joined will be shown
- No ghost participants with temporary IDs
- Proper separation between browsing and joining

## Security Implications

This was a critical security/privacy issue because:
- Users' device IDs were exposed before they chose to join
- Sessions could appear to have participants who were just browsing
- No way to distinguish between browsers and actual participants
- Could lead to premature DKG initiation with wrong participant count