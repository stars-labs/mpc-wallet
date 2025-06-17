#!/usr/bin/env bun

// Test script to debug Round 2 package map key format
// Run with: bun test/test-round2-map-fix.ts

console.log(`
=== DKG Round 2 Package Map Debug ===

I've added extensive debugging to determine why Round 2 packages aren't being sent.
The WASM is generating a package map, but we can't find the packages using the expected keys.

🔧 What I added:
1. Log the package map keys to see what format they're in
2. Log the package map structure (first 200 chars)
3. Try multiple key formats: string, numeric, padded
4. Detailed logging of what keys we're looking for vs what's available

📋 To test the fix:

1. Rebuild the extension:
   bun run build

2. Reload the extension in Chrome

3. Start 3 instances and create a session

4. Watch for these debug messages in mpc-2:
   - "Package map keys: [...]" - Shows actual keys in the map
   - "Package map structure: {...}" - Shows the map structure
   - "Looking for package for mpc-1 with index 1 (key: "1")"
   - "Available keys in packageMap: [...]"
   - Either "Found package using X key" or error messages

🔍 Possible key formats:
- String indices: "1", "2", "3"
- Numeric indices: 1, 2, 3 (without quotes)
- Padded hex: "0000000000000000000000000000000000000001"
- Peer IDs: "mpc-1", "mpc-2", "mpc-3"

💡 Once we identify the correct key format, we can fix the package extraction logic.

🎯 Success indicators:
- "Found package using X key" messages
- "Sent Round 2 package to mpc-1 (index 1)"
- "Sent Round 2 package to mpc-3 (index 3)"
- "Successfully sent 2 Round 2 packages"
`);