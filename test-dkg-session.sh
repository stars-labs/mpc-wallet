#!/bin/bash

# Test script to verify DKG session ID and participants fixes
echo "Testing DKG session with 3 MPC nodes..."
echo "======================================="
echo ""

# Kill any existing mpc-wallet-tui processes
pkill -f mpc-wallet-tui || true

# Clean up old log files
rm -f mpc-wallet-mpc-*.log

# Start signal server in background if not running
if ! pgrep -f webrtc-signal-server > /dev/null; then
    echo "Starting signal server..."
    cargo run --bin webrtc-signal-server > signal-server.log 2>&1 &
    SIGNAL_PID=$!
    sleep 2
fi

# Build the TUI
echo "Building TUI..."
cargo build --release --bin mpc-wallet-tui

# Start mpc-1 (proposer)
echo "Starting mpc-1 (proposer)..."
RUST_LOG=info cargo run --release --bin mpc-wallet-tui -- --device-id mpc-1 --headless > mpc-1-output.log 2>&1 &
MPC1_PID=$!

sleep 3

# Start mpc-2 (joiner)
echo "Starting mpc-2 (joiner)..."
RUST_LOG=info cargo run --release --bin mpc-wallet-tui -- --device-id mpc-2 --headless > mpc-2-output.log 2>&1 &
MPC2_PID=$!

sleep 3

# Start mpc-3 (joiner)
echo "Starting mpc-3 (joiner)..."
RUST_LOG=info cargo run --release --bin mpc-wallet-tui -- --device-id mpc-3 --headless > mpc-3-output.log 2>&1 &
MPC3_PID=$!

echo ""
echo "All nodes started. Waiting 10 seconds for DKG to initialize..."
sleep 10

echo ""
echo "Checking session IDs in logs..."
echo "================================"

# Check session IDs from logs
echo "mpc-1 session ID:"
grep "Generated DKG session ID\|UpdateDKGSessionId\|Created DKG session" mpc-wallet-mpc-1.log | tail -1

echo "mpc-2 session ID:"
grep "Joining DKG session\|UpdateDKGSessionId" mpc-wallet-mpc-2.log | tail -1

echo "mpc-3 session ID:"
grep "Joining DKG session\|UpdateDKGSessionId" mpc-wallet-mpc-3.log | tail -1

echo ""
echo "Checking participants lists..."
echo "=============================="

# Check participants
echo "mpc-1 participants:"
grep "Current participants\|Updated session participants" mpc-wallet-mpc-1.log | tail -1

echo "mpc-2 participants:"
grep "Current participants\|Updated session participants" mpc-wallet-mpc-2.log | tail -1

echo "mpc-3 participants:"
grep "Current participants\|Updated session participants" mpc-wallet-mpc-3.log | tail -1

echo ""
echo "Checking WebRTC connections..."
echo "=============================="

# Check WebRTC
echo "mpc-1 WebRTC:"
grep "Initiating WebRTC\|WebRTC connections established" mpc-wallet-mpc-1.log | tail -1

echo "mpc-2 WebRTC:"
grep "Initiating WebRTC\|WebRTC connections established" mpc-wallet-mpc-2.log | tail -1

echo "mpc-3 WebRTC:"
grep "Initiating WebRTC\|WebRTC connections established" mpc-wallet-mpc-3.log | tail -1

echo ""
echo "Test completed. Cleaning up..."

# Kill processes
kill $MPC1_PID $MPC2_PID $MPC3_PID 2>/dev/null || true
if [ ! -z "$SIGNAL_PID" ]; then
    kill $SIGNAL_PID 2>/dev/null || true
fi

echo "Done!"