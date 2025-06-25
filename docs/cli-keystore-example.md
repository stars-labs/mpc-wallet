# FROST MPC CLI Keystore Format - Concrete Example

This document shows a concrete example of the CLI keystore format with actual data values.

## Example Scenario

- Wallet Name: "test-wallet-2024"
- 2-of-3 threshold setup
- Three devices: laptop, phone, tablet
- Ethereum/secp256k1 curve

## File Structure

```
~/.frost-mpc-cli/
├── device_id          # Content: "laptop"
├── index.json        
└── wallets/
    └── laptop/
        └── secp256k1/
            └── test-wallet-2024.dat    # Encrypted file
```

## Example `index.json`

```json
{
  "version": 1,
  "wallets": [
    {
      "wallet_id": "test-wallet-2024",
      "name": "Test Wallet 2024",
      "curve_type": "secp256k1",
      "blockchain": "ethereum",
      "public_address": "0x742d35Cc6634C0532925a3b844Bc9e7595f4279",
      "threshold": 2,
      "total_participants": 3,
      "created_at": 1703980800,
      "group_public_key": "{\"curve\":\"secp256k1\",\"point\":\"0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798\"}",
      "devices": [
        {
          "device_id": "laptop",
          "name": "My Laptop",
          "identifier": "1",
          "last_seen": 1703980800
        },
        {
          "device_id": "phone",
          "name": "My Phone",
          "identifier": "2",
          "last_seen": 1703980800
        },
        {
          "device_id": "tablet",
          "name": "My Tablet",
          "identifier": "3",
          "last_seen": 1703980800
        }
      ],
      "tags": ["personal", "main"],
      "description": "My main 2-of-3 wallet"
    }
  ],
  "devices": [
    {
      "device_id": "laptop",
      "name": "My Laptop",
      "identifier": "laptop",
      "last_seen": 1703980800
    },
    {
      "device_id": "phone",
      "name": "My Phone",
      "identifier": "phone",
      "last_seen": 1703980800
    },
    {
      "device_id": "tablet",
      "name": "My Tablet",
      "identifier": "tablet",
      "last_seen": 1703980800
    }
  ]
}
```

## Decrypted Wallet Data (from `test-wallet-2024.dat`)

When decrypted with the password, the wallet file contains:

```json
{
  "key_package": "{\"header\":{\"version\":0,\"ciphersuite\":\"FROST-secp256k1-SHA256-v1\"},\"identifier\":\"0x0000000000000000000000000000000000000000000000000000000000000001\",\"signing_share\":\"0x2b7e151628aed2a6abf7158809cf4f3c762e7160f38b4da56a784d9045190cfef\",\"verifying_share\":\"0x0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798\",\"verifying_key\":\"0x02c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5\",\"min_signers\":2}",
  "group_public_key": "{\"verifying_shares\":{\"0x0000000000000000000000000000000000000000000000000000000000000001\":\"0x0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798\",\"0x0000000000000000000000000000000000000000000000000000000000000002\":\"0x02c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5\",\"0x0000000000000000000000000000000000000000000000000000000000000003\":\"0x02f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f9\"},\"verifying_key\":\"0x02c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5\"}",
  "session_id": "test-wallet-2024",
  "device_id": "laptop"
}
```

## Chrome Extension Export Format

When exported for Chrome extension compatibility:

### Decrypted Extension Format

```json
{
  "keyPackage": "eyJoZWFkZXIiOnsidmVyc2lvbiI6MCwiY2lwaGVyc3VpdGUiOiJGUk9TVC1zZWNwMjU2azEtU0hBMjU2LXYxIn0sImlkZW50aWZpZXIiOiIweDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAxIiwic2lnbmluZ19zaGFyZSI6IjB4MmI3ZTE1MTYyOGFlZDJhNmFiZjcxNTg4MDljZjRmM2M3NjJlNzE2MGYzOGI0ZGE1NmE3ODRkOTA0NTE5MGNmZWYiLCJ2ZXJpZnlpbmdfc2hhcmUiOiIweDAyNzliZTY2N2VmOWRjYmJhYzU1YTA2Mjk1Y2U4NzBiMDcwMjliZmNkYjJkY2UyOGQ5NTlmMjgxNWIxNmY4MTc5OCIsInZlcmlmeWluZ19rZXkiOiIweDAyYzYwNDdmOTQ0MWVkN2Q2ZDMwNDU0MDZlOTVjMDdjZDg1Yzc3OGU0YjhjZWYzY2E3YWJhYzA5Yjk1YzcwOWVlNSIsIm1pbl9zaWduZXJzIjoyfQ==",
  "publicKeyPackage": "eyJ2ZXJpZnlpbmdfc2hhcmVzIjp7IjB4MDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDEiOiIweDAyNzliZTY2N2VmOWRjYmJhYzU1YTA2Mjk1Y2U4NzBiMDcwMjliZmNkYjJkY2UyOGQ5NTlmMjgxNWIxNmY4MTc5OCIsIjB4MDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDIiOiIweDAyYzYwNDdmOTQ0MWVkN2Q2ZDMwNDU0MDZlOTVjMDdjZDg1Yzc3OGU0YjhjZWYzY2E3YWJhYzA5Yjk1YzcwOWVlNSIsIjB4MDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDMiOiIweDAyZjkzMDhhMDE5MjU4YzMxMDQ5MzQ0Zjg1Zjg5ZDUyMjliNTMxYzg0NTgzNmY5OWIwODYwMWYxMTNiY2UwMzZmOSJ9LCJ2ZXJpZnlpbmdrZXkiOiIweDAyYzYwNDdmOTQ0MWVkN2Q2ZDMwNDU0MDZlOTVjMDdjZDg1Yzc3OGU0YjhjZWYzY2E3YWJhYzA5Yjk1YzcwOWVlNSJ9",
  "groupPublicKey": "0x742d35Cc6634C0532925a3b844Bc9e7595f4279",
  "sessionId": "test-wallet-2024",
  "deviceId": "laptop",
  "participantIndex": 1,
  "threshold": 2,
  "totalParticipants": 3,
  "participants": ["laptop", "phone", "tablet"],
  "curve": "secp256k1",
  "ethereumAddress": "0x742d35Cc6634C0532925a3b844Bc9e7595f4279",
  "solanaAddress": null,
  "createdAt": 1703980800000,
  "lastUsed": null,
  "backupDate": 1704067200000
}
```

### Encrypted Extension Backup

```json
{
  "version": "1.0.0",
  "deviceId": "laptop",
  "exportedAt": 1704067200000,
  "wallets": [
    {
      "metadata": {
        "id": "test-wallet-2024",
        "name": "Test Wallet 2024",
        "blockchain": "ethereum",
        "address": "0x742d35Cc6634C0532925a3b844Bc9e7595f4279",
        "sessionId": "test-wallet-2024",
        "isActive": true,
        "hasBackup": true
      },
      "encryptedShare": {
        "walletId": "test-wallet-2024",
        "algorithm": "AES-GCM",
        "salt": "Zm9vYmFyYmF6cXV4MTIzNA==",
        "iv": "cmFuZG9tMTIzNDU2Nzg5MGFi",
        "ciphertext": "U29tZUxvbmdFbmNyeXB0ZWREYXRhSGVyZS4uLg==",
        "authTag": null
      }
    }
  ]
}
```

## Key Points to Note

1. **FROST Identifiers**: The CLI uses hex-encoded identifiers like `0x0000000000000000000000000000000000000000000000000000000000000001`
2. **Serialization**: KeyPackage and PublicKeyPackage are JSON-serialized, then stored as strings in the wallet data
3. **Base64 Encoding**: For extension compatibility, all binary data is base64-encoded
4. **Wallet ID Convention**: The wallet ID matches the session ID and uses the wallet name
5. **Device Organization**: Each device stores only its own key shares in its device-specific directory