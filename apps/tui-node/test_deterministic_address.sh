#!/bin/bash
# Test script to verify deterministic address generation

echo "=== Testing Deterministic Address Generation ==="
echo "This test will create wallets with the same name and verify they generate identical addresses"
echo ""

# Test wallet name
WALLET_NAME="TestDeterministicWallet"

# Function to extract session ID from logs
extract_session_id() {
    local log_file="$1"
    grep "Deriving group key - Session ID:" "$log_file" | head -1 | sed -E 's/.*Session ID: ([^,]+),.*/\1/'
}

# Function to extract group address from logs
extract_group_address() {
    local log_file="$1"
    grep "Generated ethereum address:" "$log_file" | head -1 | sed -E 's/.*address: //'
}

# Run test 1
echo "Test 1: Creating wallet with name '$WALLET_NAME'"
RUST_LOG=info cargo run --bin mpc-wallet-tui -- --device-id test1 --headless 2>&1 | tee test1.log &
PID1=$!
sleep 5
echo "create_wallet:$WALLET_NAME:2:2" | nc -w 1 localhost 8080 2>/dev/null || true
sleep 3
kill $PID1 2>/dev/null || true
wait $PID1 2>/dev/null

SESSION_ID_1=$(extract_session_id "test1.log")
ADDRESS_1=$(extract_group_address "test1.log")

echo "  Session ID: $SESSION_ID_1"
echo "  Address: $ADDRESS_1"
echo ""

# Run test 2
echo "Test 2: Creating wallet with same name '$WALLET_NAME'"
RUST_LOG=info cargo run --bin mpc-wallet-tui -- --device-id test2 --headless 2>&1 | tee test2.log &
PID2=$!
sleep 5
echo "create_wallet:$WALLET_NAME:2:2" | nc -w 1 localhost 8080 2>/dev/null || true
sleep 3
kill $PID2 2>/dev/null || true
wait $PID2 2>/dev/null

SESSION_ID_2=$(extract_session_id "test2.log")
ADDRESS_2=$(extract_group_address "test2.log")

echo "  Session ID: $SESSION_ID_2"
echo "  Address: $ADDRESS_2"
echo ""

# Compare results
echo "=== Results ==="
if [ "$SESSION_ID_1" = "$SESSION_ID_2" ]; then
    echo "✅ Session IDs match: Both are '$SESSION_ID_1'"
else
    echo "❌ Session IDs differ!"
    echo "   Test 1: $SESSION_ID_1"
    echo "   Test 2: $SESSION_ID_2"
fi

if [ "$ADDRESS_1" = "$ADDRESS_2" ]; then
    echo "✅ Addresses match: Both are '$ADDRESS_1'"
else
    echo "❌ Addresses differ!"
    echo "   Test 1: $ADDRESS_1"
    echo "   Test 2: $ADDRESS_2"
fi

# Cleanup
rm -f test1.log test2.log

echo ""
echo "Test complete!"