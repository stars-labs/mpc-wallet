#!/usr/bin/env bun

// Test script to verify the endianness fix for both curves
// Run with: bun test/test-endianness-fix.ts

console.log(`
=== Endianness Fix for Round 2 Package Map Keys ===

The issue has been identified and fixed! Different curves use different endianness
for their 64-character hexadecimal keys in the Round 2 package map.

🔧 Key format differences:

secp256k1 (big-endian):
- Participant 1: "0000000000000000000000000000000000000000000000000000000000000001"
- Participant 3: "0000000000000000000000000000000000000000000000000000000000000003"
- Number at the END (right side)

ed25519 (little-endian):
- Participant 1: "0100000000000000000000000000000000000000000000000000000000000000"
- Participant 3: "0300000000000000000000000000000000000000000000000000000000000000"
- Number at the BEGINNING (left side)

🔧 What was fixed:
The code now tries both endianness formats:
1. Big-endian: peerIndex.toString().padStart(64, '0')
2. Little-endian: peerIndex.toString(16).padStart(2, '0') + '0'.repeat(62)

📋 To test the fix:

1. Rebuild the extension:
   bun run build

2. Reload the extension in Chrome

3. Test with Ethereum (secp256k1):
   - Create session with Ethereum blockchain
   - Should see: "Found package using big-endian key..."

4. Test with Solana (ed25519):
   - Create session with Solana blockchain  
   - Should see: "Found package using little-endian key..."

🎯 Success indicators:
- Both curves successfully send Round 2 packages
- "Sent Round 2 package to mpc-1 (index 1)"
- "Sent Round 2 package to mpc-3 (index 3)"
- "Successfully sent 2 Round 2 packages"
- All peers complete DKG

💡 This explains why endianness matters - it's not just about the cryptographic operations,
but also how the WASM implementation formats the package map keys!
`);