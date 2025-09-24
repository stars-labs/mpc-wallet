# 🚨 Architecture Issue: JoinDKG Cannot Create WebSocket Connections

## **Problem Identified**

The shared WebSocket channel architecture is **too restrictive**. I made the mistake of assuming all participants share the same WebSocket connection, but actually:

**❌ Wrong Architecture Assumption:**
- All participants share one WebSocket connection

**✅ Correct Architecture:**
- **Each participant has their own WebSocket connection** to the signal server
- **Each participant has their own WebSocket channel** for their connection
- **The coordinator and participants are separate processes** with separate connections

## **Evidence from Logs:**

```
mpc-1: ✅ WebSocket connected successfully!
mpc-1: ✅ Registered as device: mpc-1

mpc-2: ✅ WebSocket connected successfully! 
mpc-2: ❌ No WebSocket connection available - please create session first

mpc-3: ✅ WebSocket connected successfully!
mpc-3: ❌ No WebSocket connection available - please create session first
```

**Signal server only sees mpc-1** because mpc-2 and mpc-3 fail to register after connecting.

## **Root Cause:**

The JoinDKG logic now requires a **pre-existing WebSocket channel** (expecting shared channel), but participants need to **create their own WebSocket connections** and channels when joining.

## **Fix Required:**

**Revert the shared channel architecture** and implement the **correct architecture**:

1. **Each process creates its own WebSocket connection**
2. **Each process creates its own WebSocket channel** 
3. **Prevent duplicate channels within the same process** (not across processes)
4. **Each participant registers independently** with the signal server

## **Immediate Action:**

Revert the JoinDKG WebSocket channel logic to **allow participants to create their own connections** instead of requiring a pre-existing shared channel.

**The issue is architectural misunderstanding, not technical implementation.**