#!/bin/bash
# Build all packages in the monorepo

set -e

echo "🔨 Building MPC Wallet Monorepo..."

# Build WASM package first
echo "📦 Building @mpc-wallet/core-wasm..."
cd packages/@mpc-wallet/core-wasm
bun run build
cd ../../..

# Build TypeScript packages
echo "📦 Building @mpc-wallet/types..."
cd packages/@mpc-wallet/types
bun run build
cd ../../..

echo "📦 Building @mpc-wallet/utils..."
cd packages/@mpc-wallet/utils
bun run build
cd ../../..

# Build browser extension
echo "🌐 Building browser extension..."
cd apps/browser-extension
bun run build
cd ../..

echo "✅ Build complete!"