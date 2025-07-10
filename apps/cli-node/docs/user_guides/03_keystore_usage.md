# FROST MPC Keystore User Manual

## Concepts and Fundamentals

### What is Threshold Cryptography?

Threshold cryptography is a technique that divides a cryptographic key into multiple parts (shares) where:
- Only a subset of shares (the threshold) is needed to sign transactions
- No single party ever has the complete key
- Security is maintained even if some parties are compromised

For example, in a "2-of-3" setup:
- The key is split among 3 participants
- Any 2 participants can collaborate to sign transactions
- If 1 participant is compromised, the attacker still can't sign transactions

### FROST Protocol Explained

FROST (Flexible Round-Optimized Schnorr Threshold signatures) is a modern threshold signature scheme that:
- Requires only two rounds of communication for signing
- Creates a single signature on the blockchain (not multiple signatures)
- Supports both Ethereum (secp256k1) and Solana (ed25519) blockchains
- Provides security against malicious participants

### Understanding Key Terms

- **Share**: A portion of the private key held by one participant
- **Group Public Key**: The single public key/address visible on the blockchain
- **Threshold**: Minimum number of shares needed to sign a transaction
- **DKG (Distributed Key Generation)**: The process of creating shares without anyone having the full key
- **Device**: A computer or hardware that holds one share
- **Wallet**: A collection of shares that control one blockchain address
- **Keystore**: Secure storage system for managing shares

### Keystore Functionality

The keystore provides:
1. **Secure Storage**: Encrypted storage of key shares with password protection
2. **Multi-Wallet Support**: Create and manage multiple wallets for different purposes
3. **Multi-Device Support**: Use different devices to hold shares of the same wallet
4. **Backup & Recovery**: Export and import shares for backup or device replacement
5. **Access Control**: Password protection for each wallet

## Detailed Workflows

### Initial Setup: Creating Your Keystore

Before creating any wallets, you need to initialize your keystore:

1. Launch the FROST MPC CLI node:
   ```bash
   cargo run -p frost-mpc-cli-node -- --device-id my-device-1
   ```

2. Initialize your keystore with a name for this device:
   ```
   /init_keystore ~/.frost-keystore MyLaptop
   ```

   This creates the basic directory structure and assigns a unique device ID.

### Creating a 2-of-3 Ethereum MPC Wallet

This example shows how to create an Ethereum wallet split among three devices where any two can sign.

#### Step 1: Set Up Three Devices

On each device, follow these steps:

**Device 1:**
```bash
cargo run -p frost-mpc-cli-node -- --device-id eth-device-1 --curve secp256k1
```
Then initialize the keystore:
```
/init_keystore ~/.frost-keystore EthDevice1
```

**Device 2:**
```bash
cargo run -p frost-mpc-cli-node -- --device-id eth-device-2 --curve secp256k1
```
Then initialize the keystore:
```
/init_keystore ~/.frost-keystore EthDevice2
```

**Device 3:**
```bash
cargo run -p frost-mpc-cli-node -- --device-id eth-device-3 --curve secp256k1
```
Then initialize the keystore:
```
/init_keystore ~/.frost-keystore EthDevice3
```

#### Step 2: Connect to the Signaling Server

All devices should automatically connect to the signaling server. Verify that each device can see the others with:
```
/list
```

You should see all three device IDs listed.

#### Step 3: Create a Session and Perform DKG

On Device 1, propose a new session with the required parameters:
```
/propose eth-wallet 3 2 eth-device-1,eth-device-2,eth-device-3
```

This command means:
- Create a session named "eth-wallet"
- Total of 3 participants
- Threshold of 2 (any 2 can sign)
- Including the three specified devices

On Devices 2 and 3, accept the invitation by pressing `o` or typing:
```
/accept eth-wallet
```

The system will automatically:
1. Establish WebRTC connections between all devices
2. Wait until the full mesh is ready
3. Begin the DKG process
4. Exchange commitments (Round 1) and shares (Round 2)
5. Finalize the key generation

When DKG completes successfully, each device will display:
```
DKG process completed successfully.
Generated Ethereum Address: 0x...
```

#### Step 4: Save to Keystore

On each device, after DKG completes, save the wallet to the keystore:

**Device 1:**
```
/create_wallet Corporate-ETH my-password "Corporate Ethereum wallet" ethereum,corporate
```

**Device 2:**
```
/create_wallet Corporate-ETH my-password "Corporate Ethereum wallet" ethereum,corporate
```

**Device 3:**
```
/create_wallet Corporate-ETH my-password "Corporate Ethereum wallet" ethereum,corporate
```

#### Step 5: Optional - Backup Your Shares

On each device, export your share to a secure location:
```
/export_share <wallet_id> /secure/backup/eth-share-device1.dat my-password
```

Replace `<wallet_id>` with the UUID returned when you created the wallet.

### Creating a 2-of-3 Solana MPC Wallet

Similarly, you can create a Solana wallet with the same or different devices.

#### Step 1: Set Up Three Devices

On each device:

**Device 1:**
```bash
cargo run -p frost-mpc-cli-node -- --device-id sol-device-1 --curve ed25519
```
Then initialize the keystore:
```
/init_keystore ~/.frost-keystore SolDevice1
```

**Device 2:**
```bash
cargo run -p frost-mpc-cli-node -- --device-id sol-device-2 --curve ed25519
```
Then initialize the keystore:
```
/init_keystore ~/.frost-keystore SolDevice2
```

**Device 3:**
```bash
cargo run -p frost-mpc-cli-node -- --device-id sol-device-3 --curve ed25519
```
Then initialize the keystore:
```
/init_keystore ~/.frost-keystore SolDevice3
```

#### Step 2-4: Complete DKG and Save Wallet

Follow the same process as before, but use different names:

```
/propose sol-wallet 3 2 sol-device-1,sol-device-2,sol-device-3
```

Accept on other devices and save to keystore:
```
/create_wallet Treasury-SOL my-password "Treasury Solana wallet" solana,treasury
```

### Using Your Wallets for Signing

#### Step 1: Load a Previously Created Wallet

To use a wallet you've previously created, first load it:

```
/load_wallet <wallet_id> my-password
```

This will:
- Decrypt your key share
- Load it into memory
- Set the DKG state to "Complete" so you can sign transactions

#### Step 2: Initiate Signing with Only Two Nodes

For a 2-of-3 wallet, you only need two devices to sign. Start two of your devices:

**Device 1:**
```bash
cargo run -p frost-mpc-cli-node -- --device-id eth-device-1 --curve secp256k1
```
Then load your wallet:
```
/load_wallet <wallet_id> my-password
```

**Device 2:**
```bash
cargo run -p frost-mpc-cli-node -- --device-id eth-device-2 --curve secp256k1
```
Then load your wallet:
```
/load_wallet <wallet_id> my-password
```

#### Step 3: Connect Devices and Initiate Signing

Ensure both devices can see each other:
```
/list
```

On Device 1, initiate signing:
```
/sign 0x123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
```

On Device 2, accept the signing request:
```
/acceptSign <signing_id>
```
Where `<signing_id>` is shown in the log after the sign request.

The signing process will automatically:
1. Exchange FROST commitments 
2. Exchange signature shares
3. Aggregate the final signature
4. Display the completed signature

## Real-World Use Cases

### Corporate Treasury Management

**Scenario:** A company wants to secure its treasury funds with multiple approvers.

**Setup:**
1. Create a 2-of-3 Ethereum wallet with:
   - CFO's laptop (Device 1)
   - CEO's laptop (Device 2)
   - Backup device in a secure vault (Device 3)

2. Use descriptive naming:
   ```
   /create_wallet Corporate-Treasury-ETH secure-pwd "Corporate ETH Treasury" ethereum,treasury,corporate
   ```

3. Signing Process:
   - CFO initiates a payment transaction: `/sign 0x...`
   - CEO reviews and approves: `/acceptSign sign_...`
   - Transaction is submitted to Ethereum blockchain

### Personal Wallet Security

**Scenario:** An individual wants to protect their personal crypto assets.

**Setup:**
1. Create a 2-of-3 wallet with:
   - Primary computer (Device 1)
   - Mobile device (Device 2)
   - Backup device at a trusted family member's house (Device 3)

2. Create wallet with personal tags:
   ```
   /create_wallet Personal-Savings personal-pwd "Long-term savings" ethereum,personal,savings
   ```

3. Routine Usage:
   - Load wallet on primary and mobile: `/load_wallet <id> personal-pwd`
   - Sign transactions as needed
   - If either device is lost, recover using the backup

### DAO Governance

**Scenario:** A Decentralized Autonomous Organization needs secure treasury management.

**Setup:**
1. Create multiple wallets with different thresholds:
   - 2-of-3 for operational expenses (lower threshold)
   - 3-of-5 for major investments (higher threshold)

2. Create wallet for operations:
   ```
   /create_wallet DAO-Operations dao-pwd "Daily Operations Wallet" ethereum,dao,operations
   ```

3. Governance Process:
   - Committee members load their shares
   - Proposed transaction is reviewed
   - Required members approve the transaction
   - Funds are transferred according to DAO vote

## Advanced Features

### Managing Multiple Wallets

List all your wallets to see what's available:
```
/list_wallets
```

This shows:
- Wallet IDs
- Names and descriptions
- Blockchain types and addresses
- Threshold configurations

### Adding a New Device to an Existing Wallet

If you need to add a new device to replace one or enhance security:

1. Export a share from an existing device:
   ```
   /export_share <wallet_id> /tmp/share.dat my-password
   ```

2. Transfer the share file securely to the new device

3. On the new device, import the share:
   ```
   /import_share <wallet_id> /tmp/share.dat my-password
   ```

### Recovery Scenarios

#### Device Loss Recovery

If you lose one device in a 2-of-3 setup:

1. Use your remaining two devices normally for signing

2. For better security, create a new wallet with a new third device:
   - Complete DKG with all three devices
   - Move funds from the old wallet to the new wallet

#### Password Recovery

If you forget a wallet password:

1. If you have a backup with the same password:
   - Import from backup using the password you remember

2. If all passwords are lost but you have enough devices:
   - Create a new wallet with a new DKG
   - Move funds from the old wallet to the new wallet

## Security Best Practices

### Password Management

- Use a strong, unique password for each wallet
- Consider a password manager to store complex passwords
- Never store passwords with the device holding the share

### Physical Security

- Keep devices physically secured when not in use
- For high-value wallets, consider keeping devices in different physical locations
- Use full disk encryption on all devices

### Backup Strategy

For each wallet, maintain secure backups:

1. **3-2-1 Backup Rule**:
   - 3 copies of your shares
   - 2 different types of storage media
   - 1 off-site location

2. **Encrypted Backups**:
   - Always keep exported shares encrypted
   - Use strong passwords for backups
   - Test recovery periodically

### Operational Security

- Update software regularly
- Use only trusted networks for signing operations
- Verify transaction details carefully before signing
- For high-value transactions, use additional verification channels

## Troubleshooting

### Connection Issues

**Problem:** Devices cannot see each other in the device list.

**Solutions:**
1. Check internet connectivity on both devices
2. Verify that both devices are connected to the signaling server
3. Use `/list` to refresh the device list
4. Restart the CLI node if connection issues persist

### DKG Failures

**Problem:** DKG process fails or times out.

**Solutions:**
1. Ensure all devices are online and connected
2. Verify that all devices are using the same curve type (secp256k1 or ed25519)
3. Check that the session parameters match on all devices
4. Restart the DKG process from the beginning

### Signing Issues

**Problem:** Signing process fails or stalls.

**Solutions:**
1. Verify that the wallet is properly loaded on all signing devices
2. Check that you have enough participants (meeting threshold)
3. Ensure all signing participants are online and connected
4. Restart the signing process if needed

### Keystore Errors

**Problem:** "Failed to decrypt key file" error.

**Solutions:**
1. Double-check your password
2. Verify the wallet ID is correct
3. Ensure the keystore path hasn't changed
4. Try importing from a backup if available

## Command Reference

| Command | Description | Example |
|---------|-------------|---------|
| `/init_keystore <path> <device_name>` | Initialize keystore | `/init_keystore ~/.frost-keystore MyLaptop` |
| `/list_wallets` | List available wallets | `/list_wallets` |
| `/create_wallet <name> <password> [description] [tags]` | Create new wallet | `/create_wallet Corporate-Treasury secretpass "Treasury wallet" ethereum,corporate` |
| `/load_wallet <wallet_id> <password>` | Load wallet | `/load_wallet 550e8400-e29b-41d4-a716-446655440000 secretpass` |
| `/export_share <wallet_id> <file_path> <password>` | Export share | `/export_share 550e8400-e29b-41d4-a716-446655440000 /tmp/share.dat secretpass` |
| `/import_share <wallet_id> <file_path> <password>` | Import share | `/import_share 550e8400-e29b-41d4-a716-446655440000 /tmp/share.dat secretpass` |
| `/delete_wallet <wallet_id>` | Delete wallet | `/delete_wallet 550e8400-e29b-41d4-a716-446655440000` |
| `/list` | List available devices | `/list` |
| `/propose <session_id> <total> <threshold> <devices>` | Propose DKG session | `/propose eth-wallet 3 2 device1,device2,device3` |
| `/accept <session_id>` | Accept session invitation | `/accept eth-wallet` |
| `/sign <transaction_data>` | Initiate signing | `/sign 0x123456789abcdef` |
| `/acceptSign <signing_id>` | Accept signing request | `/acceptSign sign_device1_1686123456` |

## Glossary

- **DKG**: Distributed Key Generation - process of generating key shares across devices
- **FROST**: Flexible Round-Optimized Schnorr Threshold signatures
- **Key Package**: Contains a device's share of the signing key
- **Group Public Key**: The public key for the wallet, visible on blockchain
- **Share**: A portion of a signing key held by a participant
- **Threshold**: Minimum number of participants needed to sign
- **Wallet**: A collection of key shares that can sign for one blockchain address
- **Keystore**: Secure storage system for managing shares and wallets
- **Session**: A temporary collaboration between devices for DKG or signing
- **WebRTC**: Web Real-Time Communication protocol used for device-to-device communication