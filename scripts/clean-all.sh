#!/bin/bash
# Clean all build artifacts in the monorepo

echo "🧹 Cleaning MPC Wallet Monorepo..."

# Clean root
echo "📁 Cleaning root..."
rm -rf node_modules dist target .wxt coverage

# Clean packages
echo "📦 Cleaning packages..."
rm -rf packages/@mpc-wallet/*/node_modules
rm -rf packages/@mpc-wallet/*/dist
rm -rf packages/@mpc-wallet/*/pkg
rm -rf packages/@mpc-wallet/*/target

# Clean apps
echo "📱 Cleaning apps..."
rm -rf apps/*/node_modules
rm -rf apps/*/dist
rm -rf apps/*/.wxt
rm -rf apps/*/target

# Clean Rust target directories
echo "🦀 Cleaning Rust targets..."
rm -rf apps/cli-node/target
rm -rf apps/signal-server/*/target

echo "✅ Clean complete!"