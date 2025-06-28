# MPC Wallet Naming Conventions

This document outlines the naming conventions used throughout the MPC Wallet codebase to maintain consistency with the CLI node implementation.

## Field Naming Convention

All field names use **snake_case** (lowercase with underscores) to match the CLI node JSON format.

### Core FROST Fields

| Extension Field (Old) | CLI Field (New) | Description |
|---------------------|-----------------|-------------|
| `identifier` | `participant_index` | Numeric FROST identifier (1-based) |
| `deviceId` | `device_id` | Device identifier string (e.g., "mpc-2") |
| `sessionId` | `session_id` | DKG session identifier |
| `keyPackage` | `key_package` | Serialized FROST KeyPackage |
| `groupPublicKey` | `group_public_key` | Group's public key |
| `publicKeyPackage` | `group_public_key` | Legacy alias for group_public_key |
| `totalParticipants` | `total_participants` | Total number of participants (n) |
| `participantIndex` | `participant_index` | This participant's index (1-based) |

### Timestamp Fields

| Extension Field (Old) | CLI Field (New) | Description |
|---------------------|-----------------|-------------|
| `createdAt` | `created_at` | Creation timestamp |
| `lastUsed` | `last_used` | Last usage timestamp |
| `backupDate` | `backup_date` | Backup timestamp |
| `lastModified` | `last_modified` | Last modification timestamp |

### Address Fields

| Extension Field (Old) | CLI Field (New) | Description |
|---------------------|-----------------|-------------|
| `ethereumAddress` | `ethereum_address` | Ethereum address |
| `solanaAddress` | `solana_address` | Solana address |
| `publicAddress` | `public_address` | Legacy field for address |

### Metadata Fields

| Extension Field (Old) | CLI Field (New) | Description |
|---------------------|-----------------|-------------|
| `walletId` | `wallet_id` | Wallet identifier |
| `deviceName` | `device_name` | User-friendly device name |
| `curveType` | `curve_type` | Cryptographic curve type |

## Implementation Notes

1. **Participant Index vs Device ID**:
   - `participant_index` is the numeric FROST identifier (1, 2, 3, etc.)
   - `device_id` is the string identifier (e.g., "mpc-1", "mpc-2", "mpc-3")
   - The FROST protocol uses participant_index for cryptographic operations

2. **Backward Compatibility**:
   - The WASM import/export functions accept both old and new field names
   - When importing, the code checks for both variants to maintain compatibility
   - When exporting, always use the new CLI-compatible names

3. **Type Consistency**:
   - All TypeScript interfaces use snake_case for consistency
   - The naming convention applies to both data structures and function parameters

## Example Keystore Structure

```json
{
  "version": "1.0.0",
  "encrypted": false,
  "algorithm": "none",
  "data": "",
  "metadata": {
    "session_id": "unique-session-id",
    "device_id": "mpc-2",
    "device_name": "mpc-2",
    "curve_type": "secp256k1",
    "threshold": 2,
    "total_participants": 3,
    "participant_index": 2,
    "group_public_key": "hex-encoded-public-key",
    "created_at": "2024-01-01T00:00:00Z",
    "last_modified": "2024-01-01T00:00:00Z",
    "blockchains": [
      {
        "blockchain": "ethereum",
        "network": "mainnet",
        "chain_id": 1,
        "address": "0x...",
        "address_format": "EIP-55",
        "enabled": true
      }
    ]
  }
}
```

## Migration Guide

When updating code to use the new naming conventions:

1. Update all TypeScript interfaces to use snake_case
2. Update property access in TypeScript files (e.g., `data.deviceId` → `data.device_id`)
3. Update WASM bindings to accept both formats during transition
4. Ensure all new code uses the CLI-compatible naming

This standardization ensures seamless interoperability between the Chrome extension and CLI nodes.