# FROST MPC CLI Keystore Format Documentation

## Overview

The FROST MPC CLI node uses a specific format for storing wallet key shares. This document describes the exact format to enable proper import/export functionality in the Chrome extension.

## Directory Structure

```
~/.frost-mpc-cli/
├── device_id          # Contains the device name/ID (plain text)
├── index.json        # Master index of all wallets and devices
└── wallets/
    └── {device-id}/
        ├── secp256k1/    # Ethereum wallets
        │   └── {wallet-id}.dat    # Encrypted wallet file
        └── ed25519/      # Solana wallets
            └── {wallet-id}.dat    # Encrypted wallet file
```

## Index File Format (`index.json`)

```json
{
  "version": 1,
  "wallets": [
    {
      "wallet_id": "wallet-name",
      "name": "Wallet Display Name",
      "curve_type": "secp256k1",
      "blockchain": "ethereum",
      "public_address": "0x742d35Cc6634C0532925a3b844Bc9e7595f4279",
      "threshold": 2,
      "total_participants": 3,
      "created_at": 1234567890,  // Unix timestamp in seconds
      "group_public_key": "serialized-group-public-key",
      "devices": [
        {
          "device_id": "device-1",
          "name": "Device 1",
          "identifier": "1",
          "last_seen": 1234567890
        }
      ],
      "tags": ["tag1", "tag2"],
      "description": "Optional description"
    }
  ],
  "devices": [
    {
      "device_id": "device-1",
      "name": "Device 1",
      "identifier": "device-1",
      "last_seen": 1234567890
    }
  ]
}
```

## Encrypted Wallet File Format

The `.dat` files in the wallet directories are encrypted using the CLI's encryption module. When decrypted, they contain JSON data with the following structure:

```json
{
  "key_package": "{serialized FROST KeyPackage as JSON string}",
  "group_public_key": "{serialized FROST PublicKeyPackage as JSON string}",
  "session_id": "wallet-name",
  "device_id": "device-1"
}
```

### Key Fields Explanation:

- **key_package**: The FROST KeyPackage serialized to JSON, then stored as a string
- **group_public_key**: The FROST PublicKeyPackage serialized to JSON, then stored as a string
- **session_id**: Matches the wallet_id (uses wallet name as convention)
- **device_id**: The device that owns this key share

## Chrome Extension Compatibility Format

For Chrome extension compatibility, the CLI provides an export format:

### Extension Key Share Data Format

```json
{
  "keyPackage": "base64-encoded-serialized-KeyPackage",
  "publicKeyPackage": "base64-encoded-serialized-PublicKeyPackage",
  "groupPublicKey": "hex-encoded-group-public-key",
  "sessionId": "wallet-name",
  "deviceId": "device-1",
  "participantIndex": 1,  // 1-based index
  "threshold": 2,
  "totalParticipants": 3,
  "participants": ["device-1", "device-2", "device-3"],
  "curve": "secp256k1",
  "ethereumAddress": "0x742d35Cc6634C0532925a3b844Bc9e7595f4279",
  "solanaAddress": null,
  "createdAt": 1234567890000,  // Unix timestamp in milliseconds
  "lastUsed": null,
  "backupDate": 1234567890000
}
```

### Extension Encrypted Format

```json
{
  "walletId": "wallet-name",
  "algorithm": "AES-GCM",
  "salt": "base64-encoded-salt",
  "iv": "base64-encoded-iv",
  "ciphertext": "base64-encoded-ciphertext",
  "authTag": null  // Included in ciphertext for AES-GCM
}
```

### Extension Backup Format

```json
{
  "version": "1.0.0",
  "deviceId": "device-1",
  "exportedAt": 1234567890000,
  "wallets": [
    {
      "metadata": {
        "id": "wallet-name",
        "name": "Wallet Display Name",
        "blockchain": "ethereum",
        "address": "0x742d35Cc6634C0532925a3b844Bc9e7595f4279",
        "sessionId": "wallet-name",
        "isActive": true,
        "hasBackup": true
      },
      "encryptedShare": {
        // Extension Encrypted Format (see above)
      }
    }
  ]
}
```

## Encryption Details

### CLI Native Encryption
- Uses Argon2 for key derivation (100,000 iterations)
- AES-256-GCM for encryption
- Stores salt and nonce with encrypted data

### Extension-Compatible Encryption
- Uses PBKDF2-SHA256 for key derivation (100,000 iterations)
- AES-256-GCM for encryption
- Base64 encoding for all binary data
- Compatible with Chrome extension's Web Crypto API

## Import/Export Process

### Exporting from CLI to Extension:
1. Load wallet from keystore using password
2. Deserialize the JSON to get key_package and group_public_key
3. Convert to ExtensionKeyShareData format
4. Encrypt using PBKDF2 + AES-GCM
5. Create ExtensionKeystoreBackup structure
6. Save as JSON file

### Importing from Extension to CLI:
1. Read ExtensionKeystoreBackup JSON
2. Decrypt using PBKDF2 + AES-GCM with password
3. Convert ExtensionKeyShareData to CLI WalletData format
4. Create JSON structure with key_package and group_public_key
5. Encrypt using Argon2 + AES-GCM
6. Save to appropriate wallet directory
7. Update index.json

## Key Differences Between Formats

1. **Timestamps**: CLI uses seconds, Extension uses milliseconds
2. **Participant Index**: CLI uses 0-based, Extension uses 1-based
3. **Key Serialization**: CLI stores as JSON strings, Extension uses base64
4. **Directory Structure**: CLI organizes by device and curve type
5. **Wallet ID**: CLI uses wallet name as ID (session name convention)