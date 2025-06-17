#!/usr/bin/env bun

// Debug script to analyze DKG synchronization issues
// Run with: bun test/debug-dkg-sync.ts

console.log(`
=== DKG Synchronization Debug Guide ===

When you see "still not send dkg round 2", check for these patterns in the logs:

1. Missing Round 1 Packages:
   - Look for: "WARNING: Received Round 2 package from X but still missing Round 1 packages from: Y"
   - This indicates peer Y's Round 1 package was lost

2. Package Request Mechanism:
   - Look for: "🔄 Requesting missing Round 1 packages from:"
   - Look for: "📤 Sent Round 1 package request to"
   - Look for: "📥 Received DKG package request"
   - Look for: "📤 Resent Round 1 package to"
   - Look for: "✅ Processed resent Round 1 package"

3. Common Issues:
   a) Timing Issue: Package arrives before blockchain is set
      - Solution: Improved buffering (already implemented)
      - Solution: Package request mechanism (now implemented)
   
   b) WebRTC Connection Issues:
      - Check: "Data channel opened with"
      - Check: "Mesh status: Ready"
   
   c) WASM Initialization:
      - Check: "Created ed25519 FROST DKG instance" or "Created secp256k1 FROST DKG instance"
      - Check: "FROST DKG initialized successfully"

4. Debugging Steps:
   a) Check if all peers have the same blockchain setting (ethereum vs solana)
   b) Verify all peers are in the mesh (MeshReady state)
   c) Look for "Buffered Round 1 package" messages
   d) Check for "Auto-triggering DKG" messages
   e) Verify "All Round 1 packages received" before Round 2 starts

5. Expected Flow:
   1. All peers join session and mesh becomes ready
   2. DKG initializes with correct blockchain
   3. Round 1 packages are generated and sent
   4. All peers receive all Round 1 packages
   5. Round 2 starts and packages are sent
   6. DKG completes with group public key

6. New Package Request Flow:
   - If a peer receives Round 2 while missing Round 1 packages:
     1. It detects the missing packages
     2. Requests them from peers who sent Round 2
     3. Peers resend their Round 1 packages
     4. Missing packages are processed
     5. Peer can then proceed to Round 2

To test the fix:
1. Start 3 peers
2. Create a session
3. Accept on all peers
4. Start DKG
5. Watch for package request messages if synchronization issues occur
`);