#!/usr/bin/env bash

# Run Bun tests excluding files with import issues
echo "Running tests with Bun..."

# Find all test files, excluding those that need #imports
TEST_FILES=$(find tests -name "*.test.ts" -type f | grep -v "walletClient.test.ts\|keystoreService.test.ts\|permissionService.test.ts\|accountService.test.ts" | tr '\n' ' ')

# Run the tests
bun test $TEST_FILES --preload ./tests/setup-bun.ts

echo "Tests completed!"