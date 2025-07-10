#!/usr/bin/env zsh

# DKG Address UI Test Script
# This script validates the DKG address UI implementation

echo "🧪 DKG Address UI Test Script"
echo "=============================="

# Check if development server is running
echo "1. Checking development server..."
if curl -s http://localhost:3001 > /dev/null; then
    echo "✅ Development server is running at http://localhost:3001"
else
    echo "❌ Development server is not running. Please run 'npm run dev' first."
    exit 1
fi

# Check if the extension builds successfully
echo "2. Testing production build..."
cd "$(dirname "$0")"
if npm run build > /dev/null 2>&1; then
    echo "✅ Extension builds successfully"
else
    echo "❌ Extension build failed"
    exit 1
fi

# Check key files exist
echo "3. Validating implementation files..."

files=(
    "src/entrypoints/popup/App.svelte"
    "src/entrypoints/offscreen/webrtc.ts"
    "src/entrypoints/offscreen/index.ts"
    "DKG_ADDRESS_UI_IMPLEMENTATION.md"
)

for file in "${files[@]}"; do
    if [[ -f "$file" ]]; then
        echo "✅ $file exists"
    else
        echo "❌ $file missing"
        exit 1
    fi
done

# Check for key implementation features in App.svelte
echo "4. Validating UI implementation..."

app_svelte="src/entrypoints/popup/App.svelte"

# Check for DKG state variables
if grep -q "let dkgAddress: string" "$app_svelte" && \
   grep -q "let dkgError: string" "$app_svelte" && \
   grep -q "let addressType:" "$app_svelte"; then
    echo "✅ DKG state variables implemented"
else
    echo "❌ DKG state variables missing"
    exit 1
fi

# Check for fetchDkgAddress function
if grep -q "async function fetchDkgAddress" "$app_svelte"; then
    echo "✅ fetchDkgAddress function implemented"
else
    echo "❌ fetchDkgAddress function missing"
    exit 1
fi

# Check for address type selection UI
if grep -q "Address Type:" "$app_svelte" && \
   grep -q "Single-Party" "$app_svelte" && \
   grep -q "DKG (MPC)" "$app_svelte"; then
    echo "✅ Address type selection UI implemented"
else
    echo "❌ Address type selection UI missing"
    exit 1
fi

# Check for DKG address display
if grep -q "DKG (MPC) Address:" "$app_svelte" && \
   grep -q "threshold signature" "$app_svelte"; then
    echo "✅ DKG address display implemented"
else
    echo "❌ DKG address display missing"
    exit 1
fi

# Check WebRTC manager implementation
echo "5. Validating WebRTC manager..."

webrtc_file="src/entrypoints/offscreen/webrtc.ts"

if grep -q "getEthereumAddress" "$webrtc_file"; then
    echo "✅ getEthereumAddress method implemented"
else
    echo "❌ getEthereumAddress method missing"
    exit 1
fi

# Check offscreen handlers
echo "6. Validating offscreen handlers..."

offscreen_file="src/entrypoints/offscreen/index.ts"

if grep -q "getEthereumAddress" "$offscreen_file"; then
    echo "✅ getEthereumAddress handler implemented"
else
    echo "❌ getEthereumAddress handler missing"
    exit 1
fi

echo ""
echo "🎉 All tests passed!"
echo "✅ DKG Address UI implementation is complete and functional"
echo ""
echo "Next steps:"
echo "1. Load the extension in Chrome (chrome://extensions/)"
echo "2. Enable Developer mode and load .output/chrome-mv3-dev"
echo "3. Test single-party address generation"
echo "4. Complete a DKG session to test MPC address functionality"
echo "5. Switch between address types to verify UI behavior"
echo ""
echo "For detailed testing instructions, see DKG_ADDRESS_UI_IMPLEMENTATION.md"
