#!/bin/bash

echo "Testing MPC Wallet TUI Key Handling"
echo "===================================="
echo ""
echo "This will run the TUI with debug logging for key events."
echo "Try pressing:"
echo "  - Arrow Up/Down to navigate"
echo "  - Enter to select"
echo "  - Esc to go back"
echo "  - Ctrl+Q to quit"
echo ""
echo "Debug output will be saved to: mpc-wallet-keys-debug.log"
echo ""
echo "Starting in 3 seconds..."
sleep 3

# Run with debug logging specifically for the elm module
RUST_LOG=debug,tui_node::elm=trace cargo run --bin mpc-wallet-tui -- --device-id test-keys 2>mpc-wallet-keys-debug.log

echo ""
echo "TUI exited. Check mpc-wallet-keys-debug.log for debug output"
echo ""
echo "Quick summary of key events:"
grep -i "key\|arrow\|esc\|enter" mpc-wallet-keys-debug.log | tail -20