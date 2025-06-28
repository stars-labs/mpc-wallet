#!/bin/sh

# Script to remove debug console logs while keeping essential ones

echo "🧹 Starting console log cleanup..."

# Create backup
echo "📦 Creating backup..."
cp -r src src.backup.$(date +%Y%m%d_%H%M%S)

# Files with the most debug logs to clean
FILES_TO_CLEAN=(
    "src/entrypoints/offscreen/webrtc.ts"
    "src/entrypoints/offscreen/webrtcConnection.ts"
    "src/entrypoints/background/messageHandlers.ts"
    "src/entrypoints/offscreen/messageRouter.ts"
    "src/entrypoints/background/stateManager.ts"
    "src/entrypoints/offscreen/wasmInitializer.ts"
    "src/entrypoints/background/patternRouter.ts"
    "src/entrypoints/content/provider.ts"
    "src/entrypoints/background/index.ts"
    "src/entrypoints/offscreen/index.ts"
    "src/utils/messageHandler.ts"
    "src/utils/sessionActions.ts"
    "src/entrypoints/background/webSocketManager.ts"
    "src/entrypoints/popup/App.svelte"
    "src/components/Settings.svelte"
    "src/components/AccountManager.svelte"
    "src/utils/uiState.ts"
)

# Function to comment out debug logs
comment_debug_logs() {
    local file=$1
    echo "🔧 Processing: $file"
    
    # Create temp file
    local temp_file="${file}.tmp"
    
    # Process the file
    awk '
    # Skip already commented lines
    /^[[:space:]]*\/\// { print; next }
    
    # Keep error logs
    /console\.error/ { print; next }
    
    # Keep specific security/audit logs
    /Permission (granted|revoked)/ { print; next }
    /Signature (approved|rejected)/ { print; next }
    /Account created/ { print; next }
    /Network added/ { print; next }
    
    # Comment out debug logs with specific patterns
    /console\.log.*\[.*DEBUG.*\]/ { print "// " $0; next }
    /console\.log.*"🔍/ { print "// " $0; next }
    /console\.log.*"🟡/ { print "// " $0; next }
    /console\.log.*"📊/ { print "// " $0; next }
    /console\.log.*"📡/ { print "// " $0; next }
    /console\.log.*"🔧/ { print "// " $0; next }
    /console\.log.*"✅/ { print "// " $0; next }
    /console\.log.*"🔄/ { print "// " $0; next }
    /console\.log.*"📤/ { print "// " $0; next }
    /console\.log.*"📨/ { print "// " $0; next }
    /console\.log.*"🎯/ { print "// " $0; next }
    /console\.log.*"🖥️/ { print "// " $0; next }
    /console\.log.*"🔌/ { print "// " $0; next }
    /console\.log.*"🚀/ { print "// " $0; next }
    /console\.log.*"🎉/ { print "// " $0; next }
    /console\.log.*"🔗/ { print "// " $0; next }
    /console\.log.*"🧊/ { print "// " $0; next }
    /console\.log.*"💥/ { print "// " $0; next }
    
    # Comment out message routing logs
    /console\.log.*Processing.*message/ { print "// " $0; next }
    /console\.log.*Message.*received/ { print "// " $0; next }
    /console\.log.*Routing.*to/ { print "// " $0; next }
    /console\.log.*Forwarding.*to/ { print "// " $0; next }
    
    # Comment out state update logs
    /console\.log.*State.*update/ { print "// " $0; next }
    /console\.log.*Updating.*state/ { print "// " $0; next }
    /console\.log.*UI preferences/ { print "// " $0; next }
    
    # Comment out WebRTC connection logs
    /console\.log.*connection state:/ { print "// " $0; next }
    /console\.log.*Data channel/ { print "// " $0; next }
    /console\.log.*ICE candidate/ { print "// " $0; next }
    /console\.log.*Handling.*from/ { print "// " $0; next }
    
    # Comment out WASM debug logs
    /console\.log.*WASM.*modules/ { print "// " $0; next }
    /console\.log.*typeof.*Frost/ { print "// " $0; next }
    /console\.log.*FROST DKG INIT/ { print "// " $0; next }
    
    # Comment out decorative logs
    /console\.log.*"[│┌└─]/ { print "// " $0; next }
    
    # Default: keep the line as is
    { print }
    ' "$file" > "$temp_file"
    
    # Replace original file
    mv "$temp_file" "$file"
}

# Process each file
for file in "${FILES_TO_CLEAN[@]}"; do
    if [ -f "$file" ]; then
        comment_debug_logs "$file"
    else
        echo "⚠️  File not found: $file"
    fi
done

echo "✅ Console log cleanup complete!"
echo "📁 Backup created in: src.backup.*"
echo ""
echo "🔍 Remaining console statements:"
grep -r "console\." src --include="*.ts" --include="*.js" --include="*.svelte" | grep -v "^[[:space:]]*\/\/" | wc -l