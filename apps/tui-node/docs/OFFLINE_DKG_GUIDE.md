# 🔒 Offline DKG Complete Guide

## Executive Summary

The **Offline Distributed Key Generation (DKG)** process enables creation of MPC wallets in completely air-gapped environments, providing maximum security for high-value assets. This guide details every step of the manual coordination process using SD cards for data exchange.

## Table of Contents

1. [Overview](#overview)
2. [Security Requirements](#security-requirements)
3. [Participant Roles](#participant-roles)
4. [Equipment Checklist](#equipment-checklist)
5. [Step-by-Step Process](#step-by-step-process)
6. [Data Formats](#data-formats)
7. [Verification Procedures](#verification-procedures)
8. [Troubleshooting](#troubleshooting)
9. [Security Best Practices](#security-best-practices)

---

## Overview

### What is Offline DKG?

Offline DKG is a process where multiple parties generate cryptographic key shares without any network connectivity. Data exchange happens exclusively through physical media (SD cards/USB drives), ensuring complete air-gap security.

### Why Use Offline DKG?

- **Maximum Security**: No network attack surface
- **Regulatory Compliance**: Meets strict air-gap requirements
- **Verifiable Process**: Every step can be independently verified
- **Physical Control**: Complete control over data flow

### Process Timeline

| Phase | Coordinator Time | Participant Time | Total Time |
|-------|------------------|------------------|------------|
| Setup | 30 minutes | 15 minutes | 30 minutes |
| Round 1 | 45 minutes | 30 minutes | 45 minutes |
| Round 2 | 60 minutes | 45 minutes | 60 minutes |
| Finalization | 30 minutes | 20 minutes | 30 minutes |
| **Total** | **2.5-3 hours** | **2 hours** | **3-4 hours** |

---

## Security Requirements

### Mandatory Requirements

✅ **Air-Gapped Machines**
- All network interfaces MUST be disabled
- WiFi, Ethernet, Bluetooth - all OFF
- Ideally use dedicated offline machines

✅ **Secure Physical Environment**
- Controlled access room
- No unauthorized personnel
- No recording devices

✅ **Verified Software**
- Pre-installed MPC wallet software
- Verified checksums
- No internet access after installation

### Recommended Security Measures

🔒 **Hardware Security**
- Use write-protected SD cards when possible
- Dedicated SD card reader
- Hardware security modules (HSM) for key storage

🔒 **Operational Security**
- Two-person control for coordinator
- Witness present during key operations
- Document chain of custody for SD cards

---

## Participant Roles

### 📋 Coordinator

The coordinator manages the DKG ceremony and handles data distribution.

**Responsibilities:**
- Create initial session parameters
- Collect and redistribute participant data
- Verify data integrity at each step
- Maintain ceremony log
- Destroy temporary data securely

**Required Skills:**
- Understanding of threshold cryptography
- Ability to verify cryptographic proofs
- Strong operational security practices

### 👤 Participants

Participants generate their key shares and exchange cryptographic material.

**Responsibilities:**
- Generate commitment and shares
- Verify received data
- Maintain security of their key share
- Report any anomalies immediately

**Required Skills:**
- Basic understanding of MPC
- Ability to follow security procedures
- Careful attention to detail

---

## Equipment Checklist

### Per Participant

- [ ] Air-gapped computer with MPC wallet installed
- [ ] 2+ SD cards (minimum 8GB each)
- [ ] SD card reader
- [ ] Secure storage for key share backup
- [ ] Ceremony checklist printout

### For Coordinator (Additional)

- [ ] SD cards for all participants (2x number of participants)
- [ ] Label maker or permanent markers
- [ ] Secure transport container for SD cards
- [ ] Ceremony log book
- [ ] File shredding software

### SD Card Preparation

```bash
# Format SD card (Linux/Mac)
sudo diskutil eraseDisk FAT32 DKG_TRANSFER /dev/disk2

# Verify SD card is empty
ls -la /Volumes/DKG_TRANSFER/

# Create ceremony directory structure
mkdir -p /Volumes/DKG_TRANSFER/dkg_ceremony/round1
mkdir -p /Volumes/DKG_TRANSFER/dkg_ceremony/round2
mkdir -p /Volumes/DKG_TRANSFER/dkg_ceremony/final
```

---

## Step-by-Step Process

### 📍 Phase 0: Pre-Ceremony Preparation

**All Participants:**

1. **Verify Air-Gap**
   ```bash
   # Disable all network interfaces
   sudo ifconfig en0 down        # Ethernet
   sudo ifconfig en1 down        # WiFi
   sudo systemctl stop bluetooth # Bluetooth
   
   # Verify disconnection
   ifconfig | grep "status"       # Should show "inactive"
   ```

2. **Launch MPC Wallet in Offline Mode**
   ```bash
   cd /path/to/mpc-wallet-tui
   ./mpc-wallet-tui --offline --device-id participant_1
   ```

3. **Navigate to Offline DKG**
   - Main Menu → Create New Wallet
   - Select "Offline Mode"
   - Choose role: Coordinator or Participant

---

### 📍 Phase 1: Setup & Parameter Distribution

#### Coordinator Actions:

1. **Create DKG Session**
   ```
   Session Parameters:
   - Session ID: DKG-2025-001-OFFLINE
   - Threshold: 2-of-3
   - Curve: Secp256k1
   - Participants: 3
   ```

2. **Generate Participant IDs**
   ```
   Participant 1: P1_Alice_7f3a
   Participant 2: P2_Bob_9b2c
   Participant 3: P3_Charlie_4d8e
   ```

3. **Export to SD Card**
   - Press `E` to export
   - Select `/media/sdcard/dkg_ceremony/session_params.json`
   - Verify file creation

4. **Distribute SD Cards**
   - Copy session file to each participant's SD card
   - Physically deliver to participants
   - Record delivery in ceremony log

#### Participant Actions:

1. **Import Session Parameters**
   - Insert coordinator's SD card
   - Press `I` to import
   - Select `session_params.json`
   - Verify parameters displayed correctly

2. **Confirm Participation**
   - Review threshold settings
   - Note your participant ID
   - Press `Enter` to confirm

**Verification Checkpoint:**
- [ ] All participants have same session ID
- [ ] Participant count matches expected
- [ ] Threshold parameters confirmed
- [ ] All machines remain air-gapped

---

### 📍 Phase 2: Round 1 - Commitment Exchange

#### All Participants (Including Coordinator):

1. **Generate Commitment**
   ```
   Generating Round 1 Commitment...
   Polynomial degree: 1 (for 2-of-3)
   Commitment points: 2
   Generating proof of knowledge...
   ✅ Commitment generated successfully
   ```

2. **Export Commitment**
   - File: `round1_P1_commitment.json`
   - Size: ~4-5 KB
   - Contains: Public commitment points
   - Sign with participant key

3. **Deliver to Coordinator**
   - Save to SD card
   - Physically deliver to coordinator
   - Wait for aggregated response

#### Coordinator Additional Actions:

1. **Collect All Commitments**
   ```
   Received files:
   ✅ round1_P1_commitment.json (Alice)
   ✅ round1_P2_commitment.json (Bob)
   ✅ round1_P3_commitment.json (Charlie)
   ```

2. **Verify Commitments**
   - Check cryptographic validity
   - Verify participant signatures
   - Ensure no duplicates

3. **Create Aggregated Package**
   ```json
   {
     "round": 1,
     "session_id": "DKG-2025-001-OFFLINE",
     "commitments": {
       "P1": {...},
       "P2": {...},
       "P3": {...}
     },
     "timestamp": "2025-01-05T15:00:00Z",
     "coordinator_signature": "..."
   }
   ```

4. **Distribute Back to Participants**
   - Copy to each participant's SD card
   - Include verification checksums
   - Deliver physically

#### All Participants:

1. **Import Aggregated Commitments**
   - Load from SD card
   - Verify all commitments present
   - Check coordinator signature

**Verification Checkpoint:**
- [ ] All commitments collected
- [ ] Signatures verified
- [ ] No missing participants
- [ ] Ready for Round 2

---

### 📍 Phase 3: Round 2 - Share Distribution

#### All Participants:

1. **Generate Encrypted Shares**
   ```
   Generating shares for other participants...
   For P2: Encrypting with P2's public key...
   For P3: Encrypting with P3's public key...
   ✅ Shares generated and encrypted
   ```

2. **Export Shares**
   ```
   Files created:
   - round2_P1_shares_for_P2.enc (8.1 KB)
   - round2_P1_shares_for_P3.enc (8.1 KB)
   ```

3. **Deliver to Coordinator**

#### Coordinator:

1. **Collect All Shares**
   ```
   Share matrix:
   From P1: [→P2, →P3]
   From P2: [→P1, →P3]
   From P3: [→P1, →P2]
   Total files: 6
   ```

2. **Organize by Recipient**
   ```
   For P1:
   - round2_P2_shares_for_P1.enc
   - round2_P3_shares_for_P1.enc
   
   For P2:
   - round2_P1_shares_for_P2.enc
   - round2_P3_shares_for_P2.enc
   
   For P3:
   - round2_P1_shares_for_P3.enc
   - round2_P2_shares_for_P3.enc
   ```

3. **Create Personalized SD Cards**
   - Each participant gets only their shares
   - Include share verification data
   - Add integrity checksums

#### All Participants:

1. **Import Your Shares**
   ```
   Importing encrypted shares...
   Decrypting with private key...
   ✅ Share from P2 decrypted
   ✅ Share from P3 decrypted
   ```

2. **Verify Shares**
   ```
   Verifying shares against commitments...
   Share from P2: ✅ Valid
   Share from P3: ✅ Valid
   All shares verified successfully
   ```

3. **Compute Final Key Share**
   ```
   Computing final key share...
   Combining received shares...
   ✅ Key share generated
   ```

**Verification Checkpoint:**
- [ ] All shares distributed correctly
- [ ] Decryption successful
- [ ] Share verification passed
- [ ] No complaints registered

---

### 📍 Phase 4: Finalization

#### All Participants:

1. **Compute Public Key**
   ```
   Deriving group public key...
   Public key: 0x04a7b8c9d2e3f4...
   
   Generating wallet addresses:
   ETH: 0x742d35Cc6634C053...
   BTC: bc1qxy2kgdygjrsqtzq...
   ```

2. **Create Verification Proof**
   ```
   Generating proof of correct share...
   Proof size: 256 bytes
   ✅ Proof generated
   ```

3. **Export Public Data**
   - File: `final_public_data_P1.json`
   - Contains: Public key, addresses, proof
   - Does NOT contain private share

#### Coordinator:

1. **Collect Verification Data**
   ```
   Collected:
   ✅ final_public_data_P1.json
   ✅ final_public_data_P2.json
   ✅ final_public_data_P3.json
   ```

2. **Verify Consistency**
   ```
   Checking public keys match...
   P1 public key: ✅ Match
   P2 public key: ✅ Match
   P3 public key: ✅ Match
   
   All participants derived same addresses ✅
   ```

3. **Create Final Wallet Package**
   ```json
   {
     "wallet_id": "MPC_WALLET_2025_001",
     "creation_date": "2025-01-05",
     "threshold": "2-of-3",
     "participants": ["P1_Alice", "P2_Bob", "P3_Charlie"],
     "public_key": "0x04a7b8c9d2e3f4...",
     "addresses": {
       "ethereum": "0x742d35Cc6634C053...",
       "bitcoin": "bc1qxy2kgdygjrsqtzq..."
     },
     "ceremony_log_hash": "sha256:7f3a9b2c4d8e..."
   }
   ```

4. **Distribute Final Package**
   - Copy to all participants
   - Include ceremony completion certificate
   - Provide secure storage instructions

**Final Verification:**
- [ ] All participants confirmed success
- [ ] Public keys match across all parties
- [ ] Addresses recorded and verified
- [ ] Key shares securely stored
- [ ] SD cards securely wiped

---

## Data Formats

### Session Parameters
```json
{
  "version": "1.0",
  "session_id": "DKG-2025-001-OFFLINE",
  "threshold": 2,
  "participants": 3,
  "curve": "secp256k1",
  "participant_ids": ["P1", "P2", "P3"],
  "created_at": "2025-01-05T14:00:00Z",
  "coordinator": "P1"
}
```

### Round 1 Commitment
```json
{
  "round": 1,
  "participant_id": "P1",
  "session_id": "DKG-2025-001-OFFLINE",
  "commitment": {
    "points": ["0x04abc...", "0x04def..."],
    "proof_of_knowledge": "0x7f3a9b..."
  },
  "signature": "0x3045022100..."
}
```

### Round 2 Encrypted Share
```json
{
  "round": 2,
  "from": "P1",
  "to": "P2",
  "session_id": "DKG-2025-001-OFFLINE",
  "encrypted_share": "AES256-GCM:0x8b3f...",
  "share_commitment": "0x04abc...",
  "signature": "0x3045022100..."
}
```

---

## Verification Procedures

### At Each Round

1. **File Integrity**
   ```bash
   # Generate checksum
   sha256sum round1_P1_commitment.json > checksums.txt
   
   # Verify checksum
   sha256sum -c checksums.txt
   ```

2. **Signature Verification**
   - Each file should be signed by sender
   - Verify using participant's public key
   - Reject unsigned or invalid signatures

3. **Cryptographic Validation**
   - Commitments match expected format
   - Shares decrypt successfully
   - Proofs verify correctly

### Final Verification

1. **Test Signature Generation**
   ```
   Test message: "Test DKG completion"
   Participants needed: 2 of 3
   
   P1 signature share: ✅
   P3 signature share: ✅
   Combined signature: ✅ Valid
   ```

2. **Address Derivation**
   - All participants should derive same addresses
   - Verify against expected derivation path
   - Check on blockchain (if previously used)

---

## Troubleshooting

### Common Issues

#### Issue: SD Card Not Detected
**Solution:**
- Check card reader connection
- Try different USB port
- Verify card format (FAT32)
- Test with different SD card

#### Issue: Share Verification Fails
**Solution:**
- Verify correct round 1 commitments imported
- Check participant ID mapping
- Ensure shares from correct round
- Re-export and try again

#### Issue: Different Public Keys Derived
**Critical:** Stop immediately
- Review all imported data
- Check for missing shares
- Verify participant ordering
- May need to restart ceremony

### Recovery Procedures

#### Participant Unavailable
- If before Round 1: Restart with fewer participants
- If after Round 1: Cannot proceed, must restart
- Consider (t, n-1) threshold if possible

#### Corrupted Data
- Request re-export from source
- Verify checksums before importing
- Use backup SD card if available

---

## Security Best Practices

### During Ceremony

✅ **Physical Security**
- Never leave SD cards unattended
- Use tamper-evident seals
- Maintain visual contact during transfers
- Two-person rule for coordinator

✅ **Data Hygiene**
- Wipe SD cards before use
- Secure deletion after ceremony
- No copies on hard drives
- No cloud backups ever

✅ **Operational Security**
- No phones in ceremony room
- No network devices present
- Document everything
- Multiple witnesses preferred

### After Ceremony

🔒 **Key Share Storage**
- Encrypt with strong password
- Store in secure location
- Consider hardware security module
- Never store on networked device

🔒 **Backup Procedures**
- Create encrypted backups
- Store in separate location
- Test recovery procedure
- Document recovery process

🔒 **Ongoing Security**
- Regular key share verification
- Scheduled signing ceremonies
- Maintain air-gap discipline
- Update security procedures

---

## Signing Operations (Post-DKG)

### Offline Signing Process

The same SD card exchange process is used for signing:

1. **Transaction Creation**
   - Coordinator creates transaction
   - Exports to SD card
   - Distributes to signers

2. **Share Generation**
   - Each signer generates signature share
   - Exports share to SD card
   - Returns to coordinator

3. **Signature Assembly**
   - Coordinator combines shares
   - Verifies final signature
   - Broadcasts transaction (if going online)

### Timeline
- Simple signing: 1-2 hours
- Complex transactions: 2-4 hours
- Emergency signing: Have predetermined fast-track process

---

## Compliance & Documentation

### Required Documentation

- [ ] Ceremony attendance log
- [ ] Participant identity verification
- [ ] Timeline of all operations
- [ ] Hash of all exchanged files
- [ ] Final wallet configuration
- [ ] Participant contact information
- [ ] Recovery procedures

### Audit Trail

Maintain records of:
- Every SD card exchange
- All file operations
- Verification results
- Any anomalies or issues
- Resolution of problems
- Final success confirmation

---

## Conclusion

Offline DKG provides maximum security for MPC wallet creation at the cost of operational complexity. By following this guide carefully and maintaining strict security discipline, you can create highly secure distributed wallets suitable for protecting high-value assets.

**Remember:**
- Security is only as strong as the weakest link
- Take your time - rushing leads to mistakes
- When in doubt, stop and verify
- Document everything for audit purposes

For additional support, consult the technical documentation or contact your security team.

---

*Last Updated: January 2025*
*Version: 1.0*
*Classification: Public*