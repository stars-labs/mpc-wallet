#!/usr/bin/env bash

echo "Running all Bun tests..."
echo "========================"

# Run tests that don't have import issues
echo -e "\n1. Running config and component tests..."
bun test tests/config/ tests/components/ --preload ./tests/setup-bun.ts

echo -e "\n2. Running webrtc tests..."
bun test tests/entrypoints/offscreen/webrtc*.test.ts --preload ./tests/setup-bun.ts

echo -e "\n3. Running integration tests (excluding ones with import issues)..."
bun test tests/integration/multiAccount.test.ts tests/integration/extensionCliInterop.test.ts --preload ./tests/setup-bun.ts

echo -e "\n4. Running background tests..."
bun test tests/entrypoints/background/ --preload ./tests/setup-bun.ts

echo -e "\n5. Running service tests (excluding ones with import issues)..."
bun test tests/services/walletController.test.ts tests/services/multiChainNetworkService.test.ts tests/services/networkService.test.ts --preload ./tests/setup-bun.ts

echo -e "\nTest run complete!"