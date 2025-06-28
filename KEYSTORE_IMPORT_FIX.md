# Keystore Import/Export Format Fix

## Problem
The `import_keystore` function in the WASM code was failing with a WasmError when trying to import keystore data. Investigation revealed a format mismatch between import and export functions.

## Root Cause
1. **Import expected**: `key_package` and `public_key_package` as hex-encoded JSON strings
2. **Export produced**: `key_package` and `public_key_package` as direct JSON strings (not hex-encoded)
3. When trying to hex-decode a plain JSON string, it would fail because JSON contains non-hexadecimal characters like `{`, `}`, `"`, `:`, etc.

## Solution
Modified the `import_keystore` function to handle both formats:

### Import Changes (src/lib.rs)
```rust
// Before: Only handled hex-encoded format
let key_package_bytes = hex::decode(key_package_hex)?;
let key_package: C::KeyPackage = serde_json::from_slice(&key_package_bytes)?;

// After: Handles both hex-encoded and direct JSON formats
let key_package: C::KeyPackage = if key_package_str.chars().all(|c| c.is_ascii_hexdigit()) {
    // Try hex decode first (CLI format)
    console_log!("🔍 import_keystore: Attempting hex decode for key_package");
    let key_package_bytes = hex::decode(key_package_str)?;
    serde_json::from_slice(&key_package_bytes)?
} else {
    // Direct JSON format (extension export format)
    console_log!("🔍 import_keystore: Using direct JSON for key_package");
    serde_json::from_str(key_package_str)?
};
```

### Export Changes (src/lib.rs)
```rust
// Updated to produce hex-encoded JSON for CLI compatibility
"key_package": hex::encode(key_package_json.as_bytes()),
"public_key_package": hex::encode(public_key_package_json.as_bytes()),
```

## Format Examples

### CLI Format (hex-encoded)
```json
{
  "key_package": "7b226865616465..."  // Hex-encoded JSON
  "public_key_package": "7b2276657269..." // Hex-encoded JSON
}
```

### Extension Format (direct JSON)
```json
{
  "key_package": "{\"header\":2,\"version\":0,...}"  // Direct JSON string
  "public_key_package": "{\"verifying_shares\":{...}}" // Direct JSON string
}
```

## Benefits
1. **Backward Compatibility**: Can import keystores exported by older versions
2. **CLI Compatibility**: Maintains compatibility with CLI-generated keystores
3. **Robust Import**: Automatically detects and handles both formats
4. **Clear Export**: Exports in CLI-compatible hex-encoded format

## Testing
Created comprehensive tests in `tests/keystore-format.test.ts` to validate:
- Hex format detection
- Both format imports work correctly
- Invalid formats are handled gracefully
- Round-trip consistency (export → import → export)

## Rebuild Required
After making these changes, the WASM needs to be rebuilt:
```bash
bun run build:wasm
```

This fix ensures seamless keystore import/export between the CLI and Chrome extension, maintaining full compatibility in both directions.