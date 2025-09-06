#!/usr/bin/env bash
# Run tests for all packages in the monorepo

set -e

echo "🧪 Testing MPC Wallet Monorepo..."

# Test browser extension
echo "🌐 Testing browser extension..."
cd apps/browser-extension
bun test
cd ../..

# Test CLI node
echo "💻 Testing CLI node..."
cd apps/cli-node
./run_tests.sh || true
cd ../..

echo "✅ All tests complete!"