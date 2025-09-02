# FROST MPC TUI Wallet - DKG Flows

## Table of Contents

1. [Overview](#overview)
2. [Online DKG Flow](#online-dkg-flow)
3. [Offline DKG Flow](#offline-dkg-flow)
4. [Hybrid DKG Flow](#hybrid-dkg-flow)
5. [Recovery Procedures](#recovery-procedures)
6. [Security Considerations](#security-considerations)
7. [Troubleshooting](#troubleshooting)

## Overview

Distributed Key Generation (DKG) is the foundational process for creating MPC wallets. The FROST protocol enables multiple parties to jointly generate a key pair where no single party ever has access to the complete private key. This document details both online and offline DKG procedures.

### Key Concepts

- **Threshold (t)**: Minimum number of participants needed to sign
- **Participants (n)**: Total number of key share holders
- **Key Shares**: Individual pieces of the distributed private key
- **Verification Shares**: Public commitments used to verify operations

### DKG Properties

1. **Distributed Trust**: No single point of failure
2. **Verifiable**: All participants can verify correct execution
3. **Robust**: Can complete even if some parties fail (up to n-t failures)
4. **Secure**: Threshold of parties required to reconstruct private key

## Online DKG Flow

The online DKG process uses WebRTC mesh networking for real-time coordination between participants.

### Prerequisites

- All participants must be online simultaneously
- Stable internet connection
- WebRTC-compatible network (no restrictive firewalls)
- Synchronized system clocks (±5 minutes tolerance)

### Step-by-Step Process

#### 1. Session Initiation

**Coordinator's View:**
```
┌─────────────────────────────────────────────────────┐
│ Create New Wallet - Online DKG                      │
├─────────────────────────────────────────────────────┤
│ Wallet Configuration:                               │
│                                                     │
│ Name: [treasury-wallet_______________]              │
│ Blockchain: [Ethereum (secp256k1)] ▼               │
│ Participants: [3] ▼                                 │
│ Threshold: [2] ▼                                    │
│                                                     │
│ Available Participants (3 online):                  │
│ ☑ alice (coordinator - you)                         │
│ ☑ bob (online - 192.168.1.10)                      │
│ ☑ charlie (online - 192.168.1.11)                  │
│ ☐ dave (offline)                                   │
│                                                     │
│ Network Check:                                      │
│ • Signal Server: ✅ Connected                       │
│ • NAT Type: ✅ Symmetric (WebRTC compatible)       │
│ • Bandwidth: ✅ Sufficient (>1 Mbps)               │
│                                                     │
│ [Start DKG] [Test Connection] [Cancel]             │
└─────────────────────────────────────────────────────┘
```

#### 2. Participant Invitation

**Participant's View:**
```
┌─────────────────────────────────────────────────────┐
│ 🔔 DKG Session Invitation                           │
├─────────────────────────────────────────────────────┤
│ Coordinator: alice                                  │
│ Wallet Name: treasury-wallet                        │
│ Type: 2-of-3 Ethereum Wallet                       │
│                                                     │
│ Your Role: Participant #2                           │
│ Other Participants:                                 │
│ • alice (Coordinator)                               │
│ • charlie (Pending)                                 │
│                                                     │
│ Session Details:                                    │
│ • Created: 2024-01-20 10:30:15                     │
│ • Expires: 2024-01-20 10:45:15 (15 min)           │
│ • Protocol: FROST-secp256k1                        │
│                                                     │
│ ⚠️  Joining will start key generation immediately  │
│                                                     │
│ [Accept & Join] [Decline] [View Details]           │
└─────────────────────────────────────────────────────┘
```

#### 3. WebRTC Mesh Formation

**Connection Status Display:**
```
┌─────────────────────────────────────────────────────┐
│ Establishing Secure Connections                     │
├─────────────────────────────────────────────────────┤
│ Building P2P mesh network...                        │
│                                                     │
│ Connections:                                        │
│ • You → bob     [████████████░░░░] Connecting...   │
│ • You → charlie [████████████████] Connected       │
│ • bob → charlie [████████████████] Connected       │
│                                                     │
│ Network Quality:                                    │
│ • Latency: 12ms average                            │
│ • Packet Loss: 0.0%                                │
│ • Encryption: DTLS 1.3                             │
│                                                     │
│ Status: Waiting for all connections...             │
│                                                     │
│ [Details] [Abort]                                  │
└─────────────────────────────────────────────────────┘
```

#### 4. DKG Protocol Execution

**Round 1 - Commitment Generation:**
```
┌─────────────────────────────────────────────────────┐
│ DKG Progress - Round 1 of 2                         │
├─────────────────────────────────────────────────────┤
│ Generating cryptographic commitments...             │
│                                                     │
│ Local Operations:                                   │
│ ✅ Generated secret polynomial                      │
│ ✅ Computed Feldman commitments                     │
│ ✅ Created proof of knowledge                       │
│                                                     │
│ Broadcast Status:                                   │
│ • To bob:     ✅ Sent (confirmed)                  │
│ • To charlie: ✅ Sent (confirmed)                  │
│                                                     │
│ Received Commitments:                               │
│ • From bob:     ✅ Valid                           │
│ • From charlie: ⏳ Waiting...                      │
│                                                     │
│ Round Progress: ▓▓▓▓▓▓▓▓▓▓░░░░░ 66%               │
│                                                     │
│ [View Technical Details] [Pause]                   │
└─────────────────────────────────────────────────────┘
```

**Round 2 - Share Distribution:**
```
┌─────────────────────────────────────────────────────┐
│ DKG Progress - Round 2 of 2                         │
├─────────────────────────────────────────────────────┤
│ Distributing encrypted shares...                    │
│                                                     │
│ Share Generation:                                   │
│ ✅ Computed shares for each participant             │
│ ✅ Encrypted with participant public keys           │
│ ✅ Generated zero-knowledge proofs                  │
│                                                     │
│ Distribution Status:                                │
│ • To bob:     ✅ Delivered & Acknowledged          │
│ • To charlie: ✅ Delivered & Acknowledged          │
│                                                     │
│ Share Verification:                                 │
│ • From bob:     ✅ Valid share received            │
│ • From charlie: ✅ Valid share received            │
│                                                     │
│ Final Verification:                                 │
│ ✅ All shares consistent with commitments           │
│ ✅ Threshold parameters verified                    │
│                                                     │
│ [Complete DKG] [View Shares]                       │
└─────────────────────────────────────────────────────┘
```

#### 5. Wallet Finalization

**Success Screen:**
```
┌─────────────────────────────────────────────────────┐
│ ✅ Wallet Created Successfully!                     │
├─────────────────────────────────────────────────────┤
│ Wallet Details:                                     │
│ • Name: treasury-wallet                            │
│ • Type: 2-of-3 Ethereum Wallet                     │
│ • Address: 0x742d35Cc6634C0532925a3b844Bc9e7595f │
│                                                     │
│ Your Key Share:                                     │
│ • Share Index: 2                                    │
│ • Public Share: 0x04a8b3...                        │
│ • Status: Encrypted and saved                      │
│                                                     │
│ Other Participants:                                 │
│ • alice: Share 1 ✅                                 │
│ • charlie: Share 3 ✅                               │
│                                                     │
│ Next Steps:                                         │
│ 1. Test wallet with small transaction              │
│ 2. Create secure backup                            │
│ 3. Document participant contacts                   │
│                                                     │
│ [View Wallet] [Create Backup] [Done]               │
└─────────────────────────────────────────────────────┘
```

### Online DKG Sequence Diagram

```
Alice (Coordinator)     Bob (Participant)      Charlie (Participant)
        |                       |                       |
        |---- Create Session -->|                       |
        |                       |                       |
        |<--- Accept -------->  |                       |
        |                       |                       |
        |------ Invite -------->|------- Invite ------->|
        |                       |                       |
        |<---- Accept ----------|<----- Accept ---------|
        |                       |                       |
        |==== WebRTC Setup =====|===== WebRTC Setup ====|
        |                       |                       |
        |---- Round 1 Comm ---->|---- Round 1 Comm ---->|
        |<--- Round 1 Comm -----|<--- Round 1 Comm -----|
        |                       |                       |
        |---- Round 2 Share --->|---- Round 2 Share --->|
        |<--- Round 2 Share ----|<--- Round 2 Share ----|
        |                       |                       |
        |===== Verify ==========|====== Verify =========|
        |                       |                       |
        |---- Complete -------->|---- Complete -------->|
```

## Offline DKG Flow

The offline DKG process enables key generation without network connectivity, using removable media for data exchange.

### Prerequisites

- Dedicated, air-gapped machines for each participant
- Removable media (SD cards, USB drives)
- Secure physical channel for media exchange
- Trusted coordinator for orchestration

### Step-by-Step Process

#### 1. Offline Mode Activation

**Each Participant:**
```
┌─────────────────────────────────────────────────────┐
│ 🔒 Enable Offline Mode                              │
├─────────────────────────────────────────────────────┤
│ Current Status: Online                              │
│                                                     │
│ Offline Mode Checklist:                            │
│ ☑ Network interfaces will be disabled              │
│ ☑ SD card mounted at: /mnt/secure-sd              │
│ ☑ System clock synchronized                        │
│ ☑ Temporary files cleared                          │
│                                                     │
│ Security Verification:                              │
│ • WiFi: Will be disabled                           │
│ • Ethernet: Will be disabled                       │
│ • Bluetooth: Will be disabled                      │
│ • USB: Restricted to storage only                  │
│                                                     │
│ ⚠️  This action cannot be undone without restart   │
│                                                     │
│ [Enable Offline Mode] [Cancel]                     │
└─────────────────────────────────────────────────────┘
```

#### 2. DKG Parameters Exchange

**Coordinator Creates DKG Package:**
```
┌─────────────────────────────────────────────────────┐
│ 🔒 Create Offline DKG Package                       │
├─────────────────────────────────────────────────────┤
│ DKG Configuration:                                  │
│                                                     │
│ Wallet Name: cold-storage                           │
│ Participants: 3                                     │
│ Threshold: 2                                        │
│ Blockchain: Bitcoin (secp256k1)                    │
│                                                     │
│ Participant Information:                            │
│ 1. alice-airgap (Coordinator)                      │
│ 2. bob-airgap                                      │
│ 3. charlie-airgap                                  │
│                                                     │
│ Package Contents:                                   │
│ • DKG parameters                                    │
│ • Participant identifiers                           │
│ • Session metadata                                  │
│ • Expiration: 48 hours                             │
│                                                     │
│ Export Location: /mnt/secure-sd/dkg-init.json      │
│                                                     │
│ [Generate Package] [Cancel]                         │
└─────────────────────────────────────────────────────┘
```

**Participants Import Package:**
```
┌─────────────────────────────────────────────────────┐
│ 🔒 Import DKG Package                               │
├─────────────────────────────────────────────────────┤
│ SD Card Status: Mounted                             │
│ Found DKG package: dkg-init.json                    │
│                                                     │
│ Package Details:                                    │
│ • Created by: alice-airgap                         │
│ • Created at: 2024-01-20 10:00:00                 │
│ • Expires at: 2024-01-22 10:00:00                 │
│ • Signature: ✅ Valid                              │
│                                                     │
│ DKG Parameters:                                     │
│ • Wallet: cold-storage                             │
│ • Your Role: Participant #2 (bob-airgap)          │
│ • Threshold: 2 of 3                                │
│                                                     │
│ [Import & Continue] [Reject] [View Raw]            │
└─────────────────────────────────────────────────────┘
```

#### 3. Round 1 - Commitment Generation

**Each Participant Generates Commitments:**
```
┌─────────────────────────────────────────────────────┐
│ 🔒 Generate DKG Commitments (Offline)               │
├─────────────────────────────────────────────────────┤
│ Round 1 - Local Generation                          │
│                                                     │
│ Operations:                                         │
│ ✅ Generated random polynomial                      │
│ ✅ Computed commitment values                       │
│ ✅ Created cryptographic proofs                     │
│ ✅ Self-verification passed                         │
│                                                     │
│ Commitment Data:                                    │
│ • Size: 2.3 KB                                      │
│ • Format: JSON (signed)                             │
│ • Includes: Public commitments only                │
│                                                     │
│ Ready to export to SD card:                        │
│ /mnt/secure-sd/round1/bob-commitments.json        │
│                                                     │
│ Instructions:                                       │
│ 1. Export your commitments                         │
│ 2. Deliver SD card to coordinator                  │
│ 3. Wait for aggregated commitments                 │
│                                                     │
│ [Export Commitments] [Regenerate]                  │
└─────────────────────────────────────────────────────┘
```

**Coordinator Aggregates Commitments:**
```
┌─────────────────────────────────────────────────────┐
│ 🔒 Aggregate Round 1 Commitments                    │
├─────────────────────────────────────────────────────┤
│ Commitment Collection Status:                       │
│                                                     │
│ Received Commitments:                               │
│ ✅ alice-airgap: alice-commitments.json           │
│ ✅ bob-airgap: bob-commitments.json               │
│ ⏳ charlie-airgap: Waiting...                      │
│                                                     │
│ Verification Results:                               │
│ • alice: ✅ Valid signature & proofs               │
│ • bob: ✅ Valid signature & proofs                 │
│                                                     │
│ [Refresh] [Import from SD] [Verify All]            │
│                                                     │
│ Once all commitments received:                      │
│ [Create Round 1 Package]                           │
└─────────────────────────────────────────────────────┘
```

#### 4. Round 2 - Share Distribution

**Participants Generate Shares:**
```
┌─────────────────────────────────────────────────────┐
│ 🔒 Generate Secret Shares (Offline)                 │
├─────────────────────────────────────────────────────┤
│ Round 2 - Share Generation                          │
│                                                     │
│ Imported Round 1 Package: ✅                        │
│ All commitments verified: ✅                        │
│                                                     │
│ Share Generation:                                   │
│ • For alice-airgap: ✅ Encrypted                   │
│ • For charlie-airgap: ✅ Encrypted                 │
│ • Self share: ✅ Stored locally                    │
│                                                     │
│ Export Package Contents:                            │
│ • Encrypted shares for others                      │
│ • Zero-knowledge proofs                            │
│ • Share commitments                                │
│                                                     │
│ Ready to export:                                    │
│ /mnt/secure-sd/round2/bob-shares.json             │
│                                                     │
│ [Export Shares] [Verify] [Back]                    │
└─────────────────────────────────────────────────────┘
```

**Share Verification:**
```
┌─────────────────────────────────────────────────────┐
│ 🔒 Verify Received Shares                           │
├─────────────────────────────────────────────────────┤
│ Share Import Status:                                │
│                                                     │
│ Received Shares:                                    │
│ • From alice-airgap: ✅ Valid                      │
│ • From charlie-airgap: ✅ Valid                    │
│                                                     │
│ Verification Steps:                                 │
│ ✅ Decrypted shares successfully                    │
│ ✅ Shares match commitment values                   │
│ ✅ Polynomial consistency verified                  │
│ ✅ Zero-knowledge proofs valid                      │
│                                                     │
│ Key Reconstruction Test:                            │
│ ✅ Successfully computed public key                 │
│ ✅ Address derivation successful                    │
│                                                     │
│ Your Key Share: Securely stored                    │
│                                                     │
│ [Complete DKG] [Export Summary]                    │
└─────────────────────────────────────────────────────┘
```

#### 5. Final Verification

**All Participants Confirm:**
```
┌─────────────────────────────────────────────────────┐
│ 🔒 Offline DKG Complete                             │
├─────────────────────────────────────────────────────┤
│ ✅ Cold Storage Wallet Created                      │
│                                                     │
│ Wallet Summary:                                     │
│ • Name: cold-storage                               │
│ • Type: 2-of-3 Bitcoin Wallet                      │
│ • Address: bc1qxy2kgdygjrsqtzq2n0yrf24...         │
│                                                     │
│ Security Verification:                              │
│ ✅ No network activity detected                     │
│ ✅ All operations performed offline                 │
│ ✅ Key material never exposed                       │
│ ✅ Shares encrypted at rest                         │
│                                                     │
│ Backup Reminder:                                    │
│ ⚠️  Create encrypted backup immediately            │
│ ⚠️  Store backup in separate location              │
│ ⚠️  Test recovery procedure                        │
│                                                     │
│ [Create Backup] [View Details] [Exit]              │
└─────────────────────────────────────────────────────┘
```

### Offline DKG Data Flow

```
Coordinator                 Participant 1              Participant 2
     |                           |                           |
     |-- DKG Parameters -------->|                           |
     |         (SD Card)         |-- DKG Parameters -------->|
     |                           |      (SD Card)            |
     |                           |                           |
     |<-- Round 1 Commitments ---|<-- Round 1 Commitments ---|
     |      (SD Card)            |      (SD Card)           |
     |                           |                           |
     |-- Aggregated Commitments->|-- Aggregated Commitments->|
     |      (SD Card)            |      (SD Card)           |
     |                           |                           |
     |<-- Round 2 Shares --------|<-- Round 2 Shares --------|
     |      (SD Card)            |      (SD Card)           |
     |                           |                           |
     |-- Share Packages -------->|-- Share Packages -------->|
     |      (SD Card)            |      (SD Card)           |
     |                           |                           |
     |==== Local Verify =========|==== Local Verify =========|
```

## Hybrid DKG Flow

The hybrid approach combines online coordination with offline key generation for enhanced security.

### Use Cases

1. **High-Value Wallets**: Online coordination, offline key generation
2. **Geographically Distributed Teams**: Mixed online/offline participants
3. **Regulatory Compliance**: Audit trail with air-gapped security

### Process Overview

```
┌─────────────────────────────────────────────────────┐
│ Hybrid DKG Configuration                            │
├─────────────────────────────────────────────────────┤
│ Coordination: Online (WebRTC)                       │
│ Key Generation: Offline (Air-gapped)                │
│                                                     │
│ Participants:                                       │
│ • alice: Online coordination + Offline keygen      │
│ • bob: Fully offline (SD card only)               │
│ • charlie: Online coordination + Offline keygen    │
│                                                     │
│ Workflow:                                           │
│ 1. Online: Establish session parameters            │
│ 2. Offline: Generate commitments                   │
│ 3. Online: Exchange commitments                    │
│ 4. Offline: Generate shares                        │
│ 5. Online: Exchange encrypted shares               │
│ 6. Offline: Verify and store                       │
│                                                     │
│ [Configure Details] [Start Hybrid DKG]             │
└─────────────────────────────────────────────────────┘
```

## Recovery Procedures

### Lost Key Share Recovery

When a participant loses their key share:

```
┌─────────────────────────────────────────────────────┐
│ Key Share Recovery Options                          │
├─────────────────────────────────────────────────────┤
│ Wallet: treasury-wallet (2-of-3)                   │
│ Missing: bob's key share                           │
│                                                     │
│ Recovery Methods:                                   │
│                                                     │
│ 1. Restore from Backup                             │
│    • Requires: Bob's encrypted backup              │
│    • Security: Original password needed            │
│                                                     │
│ 2. Threshold Recovery (Recommended)                │
│    • Requires: 2 other participants               │
│    • Process: Generate new 2-of-3 wallet          │
│    • Result: Bob gets new share                    │
│                                                     │
│ 3. Share Refresh Protocol                          │
│    • Requires: All participants                    │
│    • Process: Redistribute shares                  │
│    • Result: Same wallet, new shares              │
│                                                     │
│ [Select Method] [View Requirements]                │
└─────────────────────────────────────────────────────┘
```

### Emergency Access Procedures

For emergency situations requiring immediate access:

```
┌─────────────────────────────────────────────────────┐
│ ⚠️  Emergency Wallet Access                         │
├─────────────────────────────────────────────────────┤
│ Wallet: critical-operations (3-of-5)                │
│ Available Participants: 2 of 5                      │
│                                                     │
│ Emergency Options:                                  │
│                                                     │
│ 1. Contact Missing Participants                     │
│    • Alice: Last seen 2 hours ago                 │
│    • Dave: Offline mode (check schedule)          │
│    • Eve: Different timezone (sleeping)            │
│                                                     │
│ 2. Use Time-Locked Recovery                        │
│    • Status: Not configured                        │
│    • Recommendation: Set up for future            │
│                                                     │
│ 3. Social Recovery Protocol                        │
│    • Requires: Pre-configured trustees            │
│    • Available: 3 of 4 trustees online            │
│                                                     │
│ [Initiate Social Recovery] [Contact List]          │
└─────────────────────────────────────────────────────┘
```

## Security Considerations

### DKG Security Model

```
┌─────────────────────────────────────────────────────┐
│ Security Properties                                 │
├─────────────────────────────────────────────────────┤
│ ✅ Guaranteed Properties:                           │
│ • No single party has complete key                 │
│ • Threshold parties required for signing           │
│ • Verifiable correct execution                     │
│ • Robust against t-1 malicious parties            │
│                                                     │
│ ⚠️  Assumptions:                                    │
│ • Secure communication channels                    │
│ • Honest majority during DKG                      │
│ • Secure local storage                            │
│ • Trusted execution environment                    │
│                                                     │
│ 🔒 Best Practices:                                  │
│ • Verify participant identities                    │
│ • Use offline DKG for high-value                  │
│ • Regular key share backups                       │
│ • Periodic share refresh                          │
└─────────────────────────────────────────────────────┘
```

### Attack Vectors and Mitigations

| Attack Vector | Impact | Mitigation |
|--------------|--------|------------|
| Malicious participant during DKG | Key compromise | Requires ≥t malicious parties |
| Network eavesdropping | Metadata leak | TLS/DTLS encryption |
| Commitment manipulation | Protocol failure | Cryptographic verification |
| Denial of service | DKG failure | Timeout and retry mechanisms |
| Key share theft | Partial compromise | Encrypted storage, HSM support |
| Replay attacks | Double signing | Nonce tracking, session IDs |

## Troubleshooting

### Common DKG Issues

#### "Timeout during Round 1"
```
┌─────────────────────────────────────────────────────┐
│ ⚠️  DKG Timeout Detected                            │
├─────────────────────────────────────────────────────┤
│ Issue: Round 1 timeout (300s exceeded)              │
│ Missing: charlie's commitments                      │
│                                                     │
│ Diagnostics:                                        │
│ • Network: ✅ Connected                             │
│ • Charlie status: 🔴 Disconnected (180s ago)      │
│ • Partial data: 2 of 3 commitments received       │
│                                                     │
│ Options:                                            │
│ 1. Wait for Charlie (extend timeout)               │
│ 2. Restart with available participants             │
│ 3. Switch to offline DKG                          │
│                                                     │
│ [Extend 5 min] [Restart] [Go Offline]             │
└─────────────────────────────────────────────────────┘
```

#### "Verification Failed"
```
┌─────────────────────────────────────────────────────┐
│ ❌ Share Verification Failed                        │
├─────────────────────────────────────────────────────┤
│ Error: Invalid share from participant 'bob'         │
│                                                     │
│ Details:                                            │
│ • Share doesn't match commitment                   │
│ • Polynomial evaluation incorrect                  │
│ • Possible corruption or attack                    │
│                                                     │
│ Automatic Actions Taken:                            │
│ ✅ Notified other participants                      │
│ ✅ Logged incident for audit                        │
│ ✅ Excluded bob from current round                  │
│                                                     │
│ Next Steps:                                         │
│ • Contact bob to verify software                  │
│ • Restart DKG without bob                         │
│ • Consider alternative participant                 │
│                                                     │
│ [View Technical Details] [Restart] [Abort]         │
└─────────────────────────────────────────────────────┘
```

### DKG Best Practices

1. **Pre-DKG Checklist**
   - Verify all participant identities
   - Test network connections
   - Synchronize clocks
   - Clear previous failed attempts

2. **During DKG**
   - Monitor progress actively
   - Keep stable network connection
   - Don't interrupt the process
   - Save all logs for audit

3. **Post-DKG**
   - Test with small transaction
   - Create immediate backup
   - Document participant info
   - Schedule regular health checks

4. **Security Hygiene**
   - Use dedicated devices for high-value wallets
   - Implement proper access controls
   - Regular security audits
   - Practice recovery procedures