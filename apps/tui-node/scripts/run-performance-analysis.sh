#!/bin/bash

# Performance Analysis Script for MPC Wallet TUI
# This script runs comprehensive performance tests and generates reports

set -e

echo "=================================="
echo "MPC Wallet TUI Performance Analysis"
echo "=================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Create reports directory
REPORTS_DIR="performance-reports"
mkdir -p "$REPORTS_DIR"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
REPORT_FILE="$REPORTS_DIR/performance_report_$TIMESTAMP.txt"

# Function to print section headers
print_section() {
    echo -e "${GREEN}═══════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}  $1${NC}"
    echo -e "${GREEN}═══════════════════════════════════════════════════${NC}"
}

# Function to run and time a command
run_timed() {
    local name=$1
    shift
    echo -n "Running $name... "
    start_time=$(date +%s%N)
    
    if "$@" > /dev/null 2>&1; then
        end_time=$(date +%s%N)
        elapsed=$(( (end_time - start_time) / 1000000 ))
        echo -e "${GREEN}✓${NC} (${elapsed}ms)"
        echo "$name: ${elapsed}ms" >> "$REPORT_FILE"
        return 0
    else
        echo -e "${RED}✗${NC}"
        echo "$name: FAILED" >> "$REPORT_FILE"
        return 1
    fi
}

# Start report
echo "Performance Analysis Report - $(date)" > "$REPORT_FILE"
echo "======================================" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

# 1. Build Tests
print_section "1. Build Performance"

echo "Cleaning previous builds..."
cargo clean

echo "Testing debug build performance..."
run_timed "Debug Build" cargo build --lib

echo "Testing release build performance..."
run_timed "Release Build" cargo build --release --lib

echo "" >> "$REPORT_FILE"

# 2. Unit Tests Performance
print_section "2. Unit Test Performance"

echo "Running unit tests..."
if cargo test --lib --release -- --nocapture 2>&1 | grep -E "test result:|running" > temp_test_results.txt; then
    echo -e "${GREEN}✓${NC} Unit tests completed"
    cat temp_test_results.txt >> "$REPORT_FILE"
    rm temp_test_results.txt
else
    echo -e "${YELLOW}⚠${NC} Some tests may have failed"
fi

echo "" >> "$REPORT_FILE"

# 3. Load Tests
print_section "3. Load Testing"

echo "Running load tests (this may take a while)..."
echo "Load Test Results:" >> "$REPORT_FILE"
echo "-----------------" >> "$REPORT_FILE"

# Run each load test individually
for test in test_high_frequency_messages test_concurrent_senders test_memory_under_load test_navigation_stress test_websocket_message_flood test_keystore_load test_sustained_load test_overload_recovery; do
    echo -n "  - $test: "
    if timeout 30 cargo test --release --test load_test $test -- --nocapture 2>&1 | grep -E "test .* ok|PASSED|throughput|messages/second" > temp_load_$test.txt; then
        echo -e "${GREEN}✓${NC}"
        echo "$test: PASSED" >> "$REPORT_FILE"
        cat temp_load_$test.txt >> "$REPORT_FILE"
        rm temp_load_$test.txt
    else
        echo -e "${YELLOW}⚠${NC}"
        echo "$test: TIMEOUT or FAILED" >> "$REPORT_FILE"
    fi
done

echo "" >> "$REPORT_FILE"

# 4. Benchmarks
print_section "4. Performance Benchmarks"

echo "Running criterion benchmarks..."
echo "Benchmark Results:" >> "$REPORT_FILE"
echo "-----------------" >> "$REPORT_FILE"

if command -v cargo-criterion &> /dev/null; then
    cargo criterion --message-format=json 2>/dev/null | jq -r '.reason' | grep -E "benchmark-complete" >> "$REPORT_FILE" || true
else
    echo "Installing criterion..."
    cargo install cargo-criterion
    
    # Run benchmarks and capture output
    if cargo bench --bench performance_benchmark 2>&1 | grep -E "time:|throughput:" > temp_bench.txt; then
        echo -e "${GREEN}✓${NC} Benchmarks completed"
        cat temp_bench.txt >> "$REPORT_FILE"
        rm temp_bench.txt
    else
        echo -e "${YELLOW}⚠${NC} Benchmarks partially completed"
    fi
fi

# HTML reports will be in target/criterion/
if [ -d "target/criterion" ]; then
    echo "HTML benchmark reports available in: target/criterion/report/index.html"
    echo "HTML reports: target/criterion/report/index.html" >> "$REPORT_FILE"
fi

echo "" >> "$REPORT_FILE"

# 5. Memory Analysis
print_section "5. Memory Analysis"

echo "Analyzing memory usage patterns..."
echo "Memory Analysis:" >> "$REPORT_FILE"
echo "---------------" >> "$REPORT_FILE"

# Check if valgrind is available (Linux only)
if command -v valgrind &> /dev/null; then
    echo "Running valgrind memory check..."
    timeout 10 valgrind --leak-check=summary --track-origins=yes cargo run --release --bin mpc-wallet-tui -- --headless 2>&1 | grep -E "total heap usage:|definitely lost:|ERROR SUMMARY:" >> "$REPORT_FILE" || true
else
    echo "Valgrind not available, skipping detailed memory analysis"
    echo "Valgrind not available" >> "$REPORT_FILE"
fi

# Basic memory check using /proc (Linux)
if [ -f "/proc/self/status" ]; then
    echo "Current process memory:" >> "$REPORT_FILE"
    grep -E "VmSize:|VmRSS:|VmPeak:" /proc/self/status >> "$REPORT_FILE" || true
fi

echo "" >> "$REPORT_FILE"

# 6. CPU Profiling
print_section "6. CPU Profile Analysis"

echo "Checking for profiling tools..."
echo "CPU Profile:" >> "$REPORT_FILE"
echo "-----------" >> "$REPORT_FILE"

if command -v perf &> /dev/null; then
    echo "Using perf for CPU profiling..."
    timeout 5 perf record -F 99 cargo run --release --bin mpc-wallet-tui -- --headless 2>/dev/null || true
    perf report --stdio 2>/dev/null | head -20 >> "$REPORT_FILE" || true
    rm -f perf.data
else
    echo "perf not available, skipping CPU profiling"
    echo "perf not available" >> "$REPORT_FILE"
fi

echo "" >> "$REPORT_FILE"

# 7. Code Metrics
print_section "7. Code Metrics"

echo "Calculating code metrics..."
echo "Code Metrics:" >> "$REPORT_FILE"
echo "------------" >> "$REPORT_FILE"

# Count lines of code
echo "Lines of Code:" >> "$REPORT_FILE"
find src -name "*.rs" -type f | xargs wc -l | tail -1 >> "$REPORT_FILE"

# Count number of files
echo "Number of source files: $(find src -name "*.rs" -type f | wc -l)" >> "$REPORT_FILE"

# Check for TODO/FIXME comments
echo "Technical debt indicators:" >> "$REPORT_FILE"
echo "  TODOs: $(grep -r "TODO" src --include="*.rs" | wc -l)" >> "$REPORT_FILE"
echo "  FIXMEs: $(grep -r "FIXME" src --include="*.rs" | wc -l)" >> "$REPORT_FILE"

echo "" >> "$REPORT_FILE"

# 8. Generate Summary
print_section "8. Performance Summary"

echo "================================" >> "$REPORT_FILE"
echo "PERFORMANCE SUMMARY" >> "$REPORT_FILE"
echo "================================" >> "$REPORT_FILE"

# Check if any optimizations are needed
NEEDS_OPTIMIZATION=false

# Check build time
if grep -q "Release Build:" "$REPORT_FILE"; then
    BUILD_TIME=$(grep "Release Build:" "$REPORT_FILE" | awk '{print $3}' | tr -d 'ms')
    if [ "$BUILD_TIME" -gt 30000 ]; then
        echo "⚠ Build time is slow (>30s)" >> "$REPORT_FILE"
        NEEDS_OPTIMIZATION=true
    else
        echo "✓ Build time is acceptable" >> "$REPORT_FILE"
    fi
fi

# Check for test failures
if grep -q "FAILED" "$REPORT_FILE"; then
    echo "⚠ Some tests failed" >> "$REPORT_FILE"
    NEEDS_OPTIMIZATION=true
else
    echo "✓ All tests passed" >> "$REPORT_FILE"
fi

# Final recommendations
echo "" >> "$REPORT_FILE"
echo "RECOMMENDATIONS:" >> "$REPORT_FILE"
echo "---------------" >> "$REPORT_FILE"

if [ "$NEEDS_OPTIMIZATION" = true ]; then
    echo "1. Review failed tests and fix issues" >> "$REPORT_FILE"
    echo "2. Consider implementing the optimizations in performance-analysis.md" >> "$REPORT_FILE"
    echo "3. Run benchmarks regularly to track performance regressions" >> "$REPORT_FILE"
else
    echo "✓ Performance is within acceptable limits" >> "$REPORT_FILE"
    echo "Continue monitoring with regular benchmark runs" >> "$REPORT_FILE"
fi

# Display summary
echo ""
echo -e "${GREEN}═══════════════════════════════════════════════════${NC}"
echo -e "${GREEN}  Analysis Complete!${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════${NC}"
echo ""
echo "Full report saved to: $REPORT_FILE"
echo ""

# Show quick summary
echo "Quick Summary:"
echo "-------------"
tail -20 "$REPORT_FILE"

# Open HTML reports if available
if [ -d "target/criterion" ] && command -v xdg-open &> /dev/null; then
    echo ""
    echo "Opening benchmark reports in browser..."
    xdg-open "target/criterion/report/index.html" 2>/dev/null || true
fi

exit 0