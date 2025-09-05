#!/bin/bash

# Test script to run the TUI and capture debug output

echo "Running MPC Wallet TUI with debug logging..."
echo "Press Ctrl+C to stop"
echo "Log file: mpc-wallet-debug.log"
echo ""

# Run with debug logging to file
RUST_LOG=debug cargo run --bin mpc-wallet-tui 2>mpc-wallet-debug.log

echo ""
echo "TUI exited. Check mpc-wallet-debug.log for debug output"