# FROST MPC Keystore Documentation

## Overview

The FROST MPC keystore is a secure system for managing threshold signing keys across multiple devices and wallets. This document provides technical details about the keystore architecture and instructions for end users.

## Architecture

The keystore is designed to support:

1. Multiple wallets (accounts) per user
2. Multiple devices per user
3. Secure sharing of key material between devices
4. Password-protected encryption
5. Metadata management for wallets and devices

### Key Components

#### Keystore Structure

```
keystore/
├── index.json           # Master index of wallets and devices
├── device_id            # Unique identifier for this device
└── wallets/
    ├── wallet1.key      # Device's key share for wallet1
    ├── wallet1_dev2.share  # Imported share from another device
    ├── wallet2.key      # Device's key share for wallet2
    └── ...
```

#### File Formats

1. **index.json**: Contains metadata about all wallets and devices.
2. **wallet files**: Encrypted files containing key material for each wallet.
3. **share files**: Encrypted files containing shares from other devices.

#### Security Model

- All sensitive key material is encrypted with AES-256-GCM
- Keys are derived from passwords using Argon2id with salting
- Each device has a unique identifier
- Each wallet has a unique identifier (UUID)

## User Manual

### Keystore Commands

| Command | Description | Example |
|---------|-------------|---------|
| `/init_keystore <path> <device_name>` | Initialize keystore | `/init_keystore ~/.mpc-keystore MyLaptop` |
| `/list_wallets` | List available wallets | `/list_wallets` |
| `/create_wallet <name> <password> [description] [tags]` | Create new wallet | `/create_wallet Corporate-Treasury secretpass "Treasury wallet" ethereum,corporate` |
| `/load_wallet <wallet_id> <password>` | Load wallet | `/load_wallet 550e8400-e29b-41d4-a716-446655440000 secretpass` |
| `/export_share <wallet_id> <file_path> <password>` | Export share | `/export_share 550e8400-e29b-41d4-a716-446655440000 /tmp/share.dat secretpass` |
| `/import_share <wallet_id> <file_path> <password>` | Import share | `/import_share 550e8400-e29b-41d4-a716-446655440000 /tmp/share.dat secretpass` |
| `/delete_wallet <wallet_id>` | Delete wallet | `/delete_wallet 550e8400-e29b-41d4-a716-446655440000` |

### Usage Workflows

#### 1. Setting up a new wallet

1. Initialize the keystore on each device:
   ```
   /init_keystore ~/.mpc-keystore MyLaptop
   ```

2. Complete DKG with all participating devices (using the standard FROST DKG process)

3. After DKG completes, save the wallet to the keystore on each device:
   ```
   /create_wallet TeamWallet secretpass "Team's shared wallet" team,ethereum
   ```

#### 2. Backing up keys

1. Export your share of a wallet:
   ```
   /export_share 550e8400-e29b-41d4-a716-446655440000 /secure/backup/wallet-share.dat secretpass
   ```

2. Store this backup securely (e.g., in an encrypted USB drive or secure vault)

#### 3. Setting up a new device

1. Initialize the keystore on the new device:
   ```
   /init_keystore ~/.mpc-keystore NewDevice
   ```

2. Import an existing share from another device or backup:
   ```
   /import_share 550e8400-e29b-41d4-a716-446655440000 /tmp/share.dat secretpass
   ```

3. The new device can now participate in signing operations

#### 4. Using a wallet for signing

1. Load a wallet:
   ```
   /load_wallet 550e8400-e29b-41d4-a716-446655440000 secretpass
   ```

2. Use standard FROST signing commands to sign transactions

## Security Best Practices

1. **Password Management**:
   - Use strong, unique passwords for each wallet
   - Use a password manager to store complex passwords
   - Never reuse passwords across different wallets

2. **Backup Protection**:
   - Store backups in multiple secure locations
   - Consider using physical vault storage for critical wallets
   - Encrypt all backup media

3. **Device Security**:
   - Use only trusted devices for key generation and signing
   - Ensure devices have up-to-date security patches
   - Use full-disk encryption on all devices with key material

4. **Threshold Management**:
   - Choose appropriate thresholds based on security needs
   - For high-value wallets, consider higher thresholds
   - Maintain separation between devices (don't keep multiple shares on one device)

## Technical Details

### Encryption Details

- **Algorithm**: AES-256-GCM (Authenticated Encryption with Associated Data)
- **Key Derivation**: Argon2id (memory-hard function)
- **Salt**: Unique 16-byte salt per encrypted file
- **Nonce**: Unique 12-byte nonce per encrypted file
- **Format**: `salt (16 bytes) + nonce (12 bytes) + ciphertext`

### Key Material Format

The keystore uses a structured format for key material:

```json
{
  "version": 1,
  "wallet_id": "550e8400-e29b-41d4-a716-446655440000",
  "device_id": "d9e1f2a3-b4c5-6d7e-8f9a-0b1c2d3e4f5a",
  "key_package": "...", // Serialized FROST key package
  "identifier_map": {}, // Map of device IDs to identifiers
  "created_at": 1686123456,
  "last_modified": 1686123456,
  "metadata": {} // Custom metadata
}
```

### Wallet Index Format

The index file contains metadata about all wallets:

```json
{
  "version": 1,
  "wallets": [
    {
      "wallet_id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "Corporate Treasury",
      "curve_type": "secp256k1",
      "blockchain": "ethereum",
      "public_address": "0x1234...",
      "threshold": 2,
      "total_participants": 3,
      "created_at": 1686123456,
      "group_public_key": "...",
      "devices": [
        {
          "device_id": "d9e1f2a3-b4c5-6d7e-8f9a-0b1c2d3e4f5a",
          "name": "Alice's Laptop",
          "device_id": "mpc-1",
          "identifier": "...",
          "last_seen": 1686123456
        },
        ...
      ],
      "tags": ["ethereum", "corporate"],
      "description": "Corporate Treasury wallet"
    },
    ...
  ],
  "devices": [...]
}
```

## Error Handling

| Error | Description | Solution |
|-------|-------------|----------|
| "Failed to decrypt key file" | Invalid password or corrupted file | Check password or restore from backup |
| "Wallet not found in index" | Wallet ID doesn't exist | Check wallet ID or import wallet |
| "Share belongs to a different wallet" | Trying to import a share for wrong wallet | Verify wallet ID matches |
| "No key package found" | Wallet not properly loaded | Use `/load_wallet` command first |

## Recovery Procedures

### Standard Recovery

If a device is lost but you have a backup:
1. Initialize keystore on a new device
2. Import your share from backup
3. Continue participating in signing operations

### Threshold Recovery

If you lose access to some devices but maintain the threshold number:
1. Complete enough share imports to meet the threshold
2. You can now sign transactions
3. Consider creating a new wallet with fresh DKG if security is compromised

### Complete Loss Recovery

If you lose more devices than the threshold allows:
1. The wallet is non-recoverable by design (threshold security)
2. You'll need to create a new wallet and transfer any assets

## Future Enhancements

The keystore architecture supports these planned enhancements:

1. **Key rotation**: Ability to refresh shares while maintaining the same public key
2. **Hierarchical wallets**: Support for HD wallet structures with derived keys
3. **Hardware security module integration**: Support for HSM storage of shares
4. **Multi-party recovery**: Advanced protocols for secure recovery options
5. **Auditing and compliance tools**: Logging and verification for enterprise use

## Troubleshooting

### Common Issues

1. **"Cannot load wallet"**: 
   - Verify the wallet ID is correct
   - Check that the password is correct
   - Ensure the keystore path is correct

2. **"Failed to sign transaction"**:
   - Ensure the wallet is loaded
   - Verify you have the correct key share
   - Check that enough participants are online

3. **"Encryption failed"**:
   - Check disk space
   - Verify file permissions
   - Check for system encryption policy conflicts

### Diagnostics

Use these commands for diagnostics:

```
/status keystore  # Check keystore status
/verify <wallet_id> <password>  # Verify wallet integrity
/debug keystore  # Output diagnostic information
```

## Glossary

- **DKG**: Distributed Key Generation - the process of generating key shares across devices
- **FROST**: Flexible Round-Optimized Schnorr Threshold signatures
- **Key Package**: Contains a device's share of the signing key
- **Group Public Key**: The public key for the wallet, visible on blockchain
- **Share**: A portion of a signing key held by a participant
- **Threshold**: Minimum number of participants needed to sign
- **Wallet**: A collection of key shares that can sign for one blockchain address