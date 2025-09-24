# 🚨 CRITICAL WebSocket Issue Found: Connection Reuse Problem

## **Root Cause Identified**

You are absolutely right that **WebSocket connections should NOT break!** 

The issue is **NOT** that the connection is inherently broken, but rather a **connection lifecycle management problem**.

## **Evidence from Logs:**

### ✅ **WebSocket Sends to mpc-1 Work:**
```
✅ Sent through WebSocket successfully (to mpc-1)
```

### ❌ **WebSocket Sends to mpc-3 Fail:**
```
❌ Failed to send through WebSocket: IO error: Broken pipe (os error 32) (to mpc-3)
```

### 🔍 **Pattern Analysis:**
- **Same timestamps**: Success and failures happen at the same time
- **Destination specific**: Failures are consistently when sending to mpc-3
- **Connection works**: The WebSocket connection itself is functional (mpc-1 messages work)

## **Root Cause: WebSocket Sink Reuse Issue**

The problem is likely:

1. **Multiple WebSocket sender tasks** are created for the same connection
2. **WebSocket sink** is being used concurrently by multiple tasks
3. **First sender task** works fine (mpc-1 messages)
4. **Second sender task** gets a "broken pipe" because the sink is already consumed

## **Fix Required:**

The WebSocket sender task implementation needs to:

1. **Ensure single sender task** per WebSocket connection
2. **Properly handle concurrent sends** through a single channel
3. **Not create multiple sinks** from the same WebSocket connection

## **Immediate Fix Needed:**

Check the WebSocket connection setup to ensure:
- Only **one WebSocket sender task** is spawned
- Only **one sink** is created per WebSocket connection  
- All messages go through the **same message channel** to the single sender

**This is a WebSocket architecture issue, not a connection stability issue!**

The fix is to ensure proper WebSocket sender task lifecycle management rather than connection recovery.