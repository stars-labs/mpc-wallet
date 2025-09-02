# Running 3-Node MPC Test

## Fixes Applied
1. ✅ Fixed session initialization - creator now properly added to accepted_devices
2. ✅ Fixed race condition in WebRTC connection creation 
3. ✅ Added SetSession command to sync TUI with AppState
4. ✅ Compiled fresh debug binary with all fixes

## Instructions to Test

### Terminal 1 - MPC-1 (Session Creator)
```bash
cd /home/freeman.xiong/Documents/github/hecoinfo/mpc-wallet
RUST_LOG=info ./target/debug/cli_node --device-id mpc-1
```
1. Press `n` for "New Wallet"
2. Select `2 of 3 (secp256k1)`
3. Note the session code that appears

### Terminal 2 - MPC-2 (Joiner)
```bash
cd /home/freeman.xiong/Documents/github/hecoinfo/mpc-wallet
RUST_LOG=info ./target/debug/cli_node --device-id mpc-2
```
1. Press `d` for "Discover Wallets"
2. Press `j` to join the discovered session

### Terminal 3 - MPC-3 (Joiner)
```bash
cd /home/freeman.xiong/Documents/github/hecoinfo/mpc-wallet
RUST_LOG=info ./target/debug/cli_node --device-id mpc-3
```
1. Press `d` for "Discover Wallets"
2. Press `j` to join the discovered session

## What Should Happen
1. MPC-1 should show "Participants (3/3)" after both joiners connect
2. MPC-2 and MPC-3 should show "Connected to other participants" 
3. All nodes should establish WebRTC connections (mesh network)
4. DKG should start automatically once mesh is ready

## Monitoring
- Watch the logs in each terminal for connection status
- Look for "WebRTC CONNECTED" messages
- Check for "Mesh ready" notifications

## If Issues Persist
The debug binary at `./target/debug/cli_node` was compiled at 20:24 with all fixes.
Make sure you're using this fresh binary, not an older one.