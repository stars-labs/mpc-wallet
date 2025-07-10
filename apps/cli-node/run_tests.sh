#!/bin/bash

# Run all tests for the FROST MPC CLI Node project
echo "Running all tests for FROST MPC CLI Node..."
echo "=========================================="

# Set test timeout
export RUST_TEST_THREADS=4
export RUST_BACKTRACE=1

# Run unit tests
echo ""
echo "Running unit tests..."
cargo test --lib -- --test-threads=4

# Run integration tests
echo ""
echo "Running integration tests..."
cargo test --test '*' -- --test-threads=4

# Run doc tests
echo ""
echo "Running documentation tests..."
cargo test --doc

# Generate test coverage report (requires cargo-tarpaulin)
echo ""
echo "Generating test coverage report..."
if command -v cargo-tarpaulin &> /dev/null; then
    cargo tarpaulin --out Html --output-dir target/coverage
    echo "Coverage report generated at: target/coverage/tarpaulin-report.html"
else
    echo "cargo-tarpaulin not installed. Skipping coverage report."
    echo "Install with: cargo install cargo-tarpaulin"
fi

echo ""
echo "All tests completed!"