#!/usr/bin/env bash

# E2E Test Suite for Offline FROST MPC Wallet
# This script runs all end-to-end tests and examples

echo "================================================"
echo "🧪 Running E2E Test Suite for Offline MPC Wallet"
echo "================================================"
echo

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track overall status
ALL_PASSED=true

# Function to run a test
run_test() {
    local test_name=$1
    local test_command=$2
    
    echo -e "${YELLOW}Running: $test_name${NC}"
    echo "Command: $test_command"
    echo "----------------------------------------"
    
    if eval "$test_command"; then
        echo -e "${GREEN}✅ $test_name PASSED${NC}"
    else
        echo -e "${RED}❌ $test_name FAILED${NC}"
        ALL_PASSED=false
    fi
    echo
}

# Test 1: Basic offline DKG demo
run_test "Offline DKG Demo" \
    "cargo run --example offline_dkg_demo 2>&1 | grep -q 'DKG CEREMONY COMPLETE'"

# Test 2: Offline DKG with signing demo
run_test "Offline DKG + Signing Demo" \
    "cargo run --example offline_dkg_signing_demo 2>&1 | grep -q 'COMPLETE OFFLINE WORKFLOW SUCCESS'"

# Test 3: Real FROST DKG and signing
run_test "Real FROST Implementation" \
    "cargo run --example offline_frost_dkg_signing 2>&1 | grep -q 'Final verification: Signature is valid'"

# Test 4: TUI simulation with Ethereum transaction
run_test "TUI Simulation with Ethereum" \
    "cargo run --example offline_frost_tui_simulation 2>&1 | grep -q 'TUI Interaction Summary'"

# Test 5: Run unit tests for offline DKG demo
run_test "Offline DKG Demo Tests" \
    "cargo test --example offline_dkg_demo 2>&1 | grep -q 'test result: ok'"

# Test 6: Run unit tests for DKG + signing
run_test "DKG + Signing Tests" \
    "cargo test --example offline_dkg_signing_demo 2>&1 | grep -q 'test result: ok'"

# Test 7: Run unit tests for real FROST
run_test "Real FROST Tests" \
    "cargo test --example offline_frost_dkg_signing 2>&1 | grep -q 'test result: ok'"

# Test 8: Run unit tests for TUI simulation
run_test "TUI Simulation Tests" \
    "cargo test --example offline_frost_tui_simulation 2>&1 | grep -q 'test result: ok'"

# Summary
echo "================================================"
echo "📊 E2E Test Suite Summary"
echo "================================================"

if [ "$ALL_PASSED" = true ]; then
    echo -e "${GREEN}🎉 ALL TESTS PASSED!${NC}"
    echo
    echo "✅ Offline DKG workflow verified"
    echo "✅ Real FROST cryptography working"
    echo "✅ TUI simulation functional"
    echo "✅ Ethereum transaction signing tested"
    echo "✅ All unit tests passing"
    exit 0
else
    echo -e "${RED}❌ SOME TESTS FAILED${NC}"
    echo "Please review the output above for details."
    exit 1
fi