#!/bin/bash

# Script to reorganize and archive obsolete documentation
# This moves old fix-related docs to archive while preserving important current docs

DOCS_DIR="/home/freeman.xiong/Documents/github/hecoinfo/mpc-wallet/docs"
ARCHIVE_DIR="$DOCS_DIR/archive"

echo "Starting documentation reorganization..."

# Create archive directories
mkdir -p "$ARCHIVE_DIR/fixes"
mkdir -p "$ARCHIVE_DIR/legacy"
mkdir -p "$ARCHIVE_DIR/migration"
mkdir -p "$ARCHIVE_DIR/session-fixes"

# Move fix-related documents to archive
echo "Archiving fix documentation..."
mv "$DOCS_DIR"/*FIX*.md "$ARCHIVE_DIR/fixes/" 2>/dev/null
mv "$DOCS_DIR"/*fix*.md "$ARCHIVE_DIR/fixes/" 2>/dev/null
mv "$DOCS_DIR"/*REJOIN*.md "$ARCHIVE_DIR/fixes/" 2>/dev/null
mv "$DOCS_DIR"/SESSION_*.md "$ARCHIVE_DIR/session-fixes/" 2>/dev/null
mv "$DOCS_DIR"/COMPLETE_*.md "$ARCHIVE_DIR/session-fixes/" 2>/dev/null
mv "$DOCS_DIR"/FINAL_*.md "$ARCHIVE_DIR/session-fixes/" 2>/dev/null

# Move legacy/obsolete documentation
echo "Archiving legacy documentation..."
mv "$DOCS_DIR"/CLAUDE.md "$ARCHIVE_DIR/legacy/" 2>/dev/null
mv "$DOCS_DIR"/MIGRATION_*.md "$ARCHIVE_DIR/migration/" 2>/dev/null
mv "$DOCS_DIR"/MPC2_*.md "$ARCHIVE_DIR/fixes/" 2>/dev/null
mv "$DOCS_DIR"/MPC3_*.md "$ARCHIVE_DIR/fixes/" 2>/dev/null
mv "$DOCS_DIR"/WEBRTC_*.md "$ARCHIVE_DIR/fixes/" 2>/dev/null
mv "$DOCS_DIR"/DEVICE_*.md "$ARCHIVE_DIR/legacy/" 2>/dev/null
mv "$DOCS_DIR"/STATE_*.md "$ARCHIVE_DIR/legacy/" 2>/dev/null
mv "$DOCS_DIR"/STATELESS_*.md "$ARCHIVE_DIR/legacy/" 2>/dev/null
mv "$DOCS_DIR"/SIMPLIFIED_*.md "$ARCHIVE_DIR/legacy/" 2>/dev/null
mv "$DOCS_DIR"/SECURITY_FIX*.md "$ARCHIVE_DIR/fixes/" 2>/dev/null
mv "$DOCS_DIR"/PERFORMANCE_*.md "$ARCHIVE_DIR/legacy/" 2>/dev/null
mv "$DOCS_DIR"/USER_INSTRUCTIONS.md "$ARCHIVE_DIR/legacy/" 2>/dev/null
mv "$DOCS_DIR"/IMPLEMENTATION_GUIDE.md "$ARCHIVE_DIR/legacy/" 2>/dev/null
mv "$DOCS_DIR"/CODE_VS_*.md "$ARCHIVE_DIR/legacy/" 2>/dev/null
mv "$DOCS_DIR"/DKG_PARTICIPANT_*.md "$ARCHIVE_DIR/fixes/" 2>/dev/null
mv "$DOCS_DIR"/DKG_FLOW_*.md "$ARCHIVE_DIR/legacy/" 2>/dev/null
mv "$DOCS_DIR"/DOCUMENTATION_REORGANIZATION.md "$ARCHIVE_DIR/legacy/" 2>/dev/null
mv "$DOCS_DIR"/KEYSTORE_IMPORT_FIX.md "$ARCHIVE_DIR/fixes/" 2>/dev/null
mv "$DOCS_DIR"/PERFECT_*.md "$ARCHIVE_DIR/fixes/" 2>/dev/null
mv "$DOCS_DIR"/UNIFIED_*.md "$ARCHIVE_DIR/legacy/" 2>/dev/null
mv "$DOCS_DIR"/WALLET_CREATION_TEST_*.md "$ARCHIVE_DIR/legacy/" 2>/dev/null

# Move old CLI documentation (now in tui-node)
echo "Moving CLI documentation to TUI node..."
mv "$DOCS_DIR"/cli-*.md "$ARCHIVE_DIR/legacy/" 2>/dev/null

# Keep important current documentation in place
echo "Preserving current documentation..."
# These files stay in the main docs directory:
# - README.md (already updated)
# - architecture/ (current architecture docs)
# - security/ (security documentation)
# - api/ (API documentation)
# - development/ (development guide)
# - deployment/ (deployment guide)
# - testing/ (current testing docs)
# - implementation/ (current implementation docs)
# - fixes/ (organized fix documentation)

# Create index for archived documents
echo "Creating archive index..."
cat > "$ARCHIVE_DIR/README.md" << 'EOF'
# Archived Documentation

This directory contains historical documentation that is no longer current but may be useful for reference.

## Categories

### Fixes (`fixes/`)
Historical bug fixes and solutions that have been implemented and merged.

### Legacy (`legacy/`)
Outdated documentation from previous versions or deprecated features.

### Migration (`migration/`)
Old migration guides from previous versions.

### Session Fixes (`session-fixes/`)
Historical session-related fixes and improvements.

## Note

These documents are preserved for historical reference only. For current documentation, please refer to the main documentation directories.
EOF

echo "Documentation reorganization complete!"
echo ""
echo "Summary:"
echo "- Archived obsolete fix documentation to: $ARCHIVE_DIR/fixes/"
echo "- Archived legacy documentation to: $ARCHIVE_DIR/legacy/"
echo "- Preserved current documentation in main directories"
echo ""
echo "Please review the changes and commit when ready."