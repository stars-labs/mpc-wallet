#!/usr/bin/env bash

echo "Running MPC Wallet TUI with maximum debug logging"
echo "================================================"
echo ""
echo "This will show ALL debug messages for key handling."
echo "Watch for these log messages:"
echo "  - 'Read terminal event' - when a key is pressed"
echo "  - 'handle_key_event' - when processing the key"  
echo "  - 'Processing message' - when handling the message"
echo "  - 'ScrollUp/ScrollDown' - when arrow keys work"
echo ""
echo "Press Ctrl+C to exit"
echo ""

# Run with TRACE level logging for the elm module
RUST_LOG=trace,tui_node::elm=trace cargo run --bin mpc-wallet-tui -- --device-id debug-test