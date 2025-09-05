# Wallet Operations Submenu Wireframes

This document contains detailed wireframes for all wallet operation submenus in the MPC wallet TUI application.

## Table of Contents

1. [Send Transaction](#send-transaction)
2. [Sign Message](#sign-message)
3. [Sign Typed Data](#sign-typed-data)
4. [Multi-Chain Sign](#multi-chain-sign)
5. [Manage Participants](#manage-participants)
6. [Rotate Keys](#rotate-keys)
7. [Lock/Unlock Wallet](#lockunlock-wallet)
8. [View Activity Log](#view-activity-log)
9. [Test Connections](#test-connections)
10. [Export Details](#export-details)
11. [Advanced Settings](#advanced-settings)

---

## Send Transaction

### Transaction Setup Screen

```
┌─ Send Transaction: company_treasury ─────────────────────────────┐
│                                                                  │
│ Transaction Details:                                             │
│ ┌─────────────────────────────────────────────────────────────┐  │
│ │ From Wallet: company_treasury (2-of-3)                     │  │
│ │ Balance: 15.7 ETH ($39,250.00)                            │  │
│ │                                                             │  │
│ │ To Address:                                                 │  │
│ │ [0x742d35Cc6634C0532925a3b844Bc9e7595f2bd__] ✅ Valid    │  │
│ │ ENS: vitalik.eth                                           │  │
│ │                                                             │  │
│ │ Amount to Send:                                             │  │
│ │ [1.5_________] ETH  ≈ $3,750.00 USD                       │  │
│ │ ○ Send all (15.7 ETH)                                      │  │
│ │                                                             │  │
│ │ Network: Ethereum Mainnet                                   │  │
│ │                                                             │  │
│ │ Gas Settings:                                               │  │
│ │ ○ Slow (15 Gwei)    ● Normal (25 Gwei)   ○ Fast (40 Gwei)│  │
│ │ Estimated fee: 0.000525 ETH ($1.31)                       │  │
│ │                                                             │  │
│ │ [ ] Advanced options                                        │  │
│ └─────────────────────────────────────────────────────────────┘  │
│                                                                  │
│ Total: 1.500525 ETH ($3,751.31)                                │
│                                                                  │
│ [Enter] Review transaction  [G] Address book  [Esc] Cancel     │
└──────────────────────────────────────────────────────────────────┘
```

### Transaction Review Screen

```
┌─ Review Transaction ─────────────────────────────────────────────┐
│                                                                  │
│ ⚠️  Please review transaction details carefully:                 │
│                                                                  │
│ ┌─────────────────────────────────────────────────────────────┐  │
│ │ Transaction Summary:                                        │  │
│ │                                                             │  │
│ │ Type: Send ETH                                              │  │
│ │ From: company_treasury (0x742d...2bd)                      │  │
│ │ To: vitalik.eth (0x742d...5f2)                            │  │
│ │                                                             │  │
│ │ Amount: 1.5 ETH                                             │  │
│ │ Value: $3,750.00 USD                                        │  │
│ │                                                             │  │
│ │ Network Fee:                                                │  │
│ │ Gas Limit: 21,000                                           │  │
│ │ Gas Price: 25 Gwei                                          │  │
│ │ Max Fee: 0.000525 ETH ($1.31)                              │  │
│ │                                                             │  │
│ │ Total Cost: 1.500525 ETH ($3,751.31)                      │  │
│ │                                                             │  │
│ │ Wallet Balance After: 14.199475 ETH                        │  │
│ └─────────────────────────────────────────────────────────────┘  │
│                                                                  │
│ Required Signatures: 2 of 3                                      │
│ • Your signature will be provided automatically                 │
│ • Need 1 more signature from: bob, carol                        │
│                                                                  │
│ [S] Sign & Broadcast  [E] Edit  [C] Cancel                     │
└──────────────────────────────────────────────────────────────────┘
```

### Signature Collection Screen

```
┌─ Collecting Signatures ──────────────────────────────────────────┐
│                                                                  │
│ Transaction: Send 1.5 ETH to vitalik.eth                       │
│                                                                  │
│ Signature Progress: [████████████░░░░░░░░░░] 50% (1/2)         │
│                                                                  │
│ ┌─────────────────────────────────────────────────────────────┐  │
│ │ Participants:                                               │  │
│ │                                                             │  │
│ │ ✅ You (alice)          Signed at 14:32:15                 │  │
│ │    Signature: 0x1a2b3c...                                  │  │
│ │                                                             │  │
│ │ ⏳ bob                  Notified, awaiting signature       │  │
│ │    Status: Online       Last seen: 2 min ago               │  │
│ │                                                             │  │
│ │ ⏳ carol                Not required (2-of-3)              │  │
│ │    Status: Offline      Last seen: 1 hour ago              │  │
│ └─────────────────────────────────────────────────────────────┘  │
│                                                                  │
│ Time remaining: 59:32                                            │
│ Auto-timeout in: 1 hour                                         │
│                                                                  │
│ Actions:                                                         │
│ [R] Resend notification  [M] Message participant               │
│ [C] Cancel transaction   [D] Download partial                  │
│                                                                  │
│ 🔔 You'll be notified when all signatures are collected        │
└──────────────────────────────────────────────────────────────────┘
```

---

## Sign Message

### Message Signing Screen

```
┌─ Sign Message: company_treasury ─────────────────────────────────┐
│                                                                  │
│ Message Details:                                                 │
│ ┌─────────────────────────────────────────────────────────────┐  │
│ │ Message to Sign:                                            │  │
│ │ ┌─────────────────────────────────────────────────────┐    │  │
│ │ │ I hereby confirm the ownership of this wallet and    │    │  │
│ │ │ authorize the following operations:                  │    │  │
│ │ │                                                       │    │  │
│ │ │ - Trading on DEX protocols                          │    │  │
│ │ │ - Staking operations                                │    │  │
│ │ │ - Governance participation                          │    │  │
│ │ │                                                       │    │  │
│ │ │ Wallet: 0x742d35Cc6634C0532925a3b844Bc9e7595f2bd   │    │  │
│ │ │ Date: 2025-01-12                                    │    │  │
│ │ └─────────────────────────────────────────────────────┘    │  │
│ │                                                             │  │
│ │ Message Format: ● Plain text  ○ Hex encoded                │  │
│ │                                                             │  │
│ │ Requesting Application: DeFi Protocol v2                   │  │
│ │ Domain: app.defi-protocol.com                              │  │
│ └─────────────────────────────────────────────────────────────┘  │
│                                                                  │
│ ⚠️  Warning: Only sign messages from trusted sources            │
│                                                                  │
│ Required Signatures: 2 of 3                                      │
│                                                                  │
│ [S] Sign message  [V] Verify domain  [C] Cancel                │
└──────────────────────────────────────────────────────────────────┘
```

---

## Sign Typed Data

### EIP-712 Signing Screen

```
┌─ Sign Typed Data (EIP-712) ─────────────────────────────────────┐
│                                                                  │
│ Structured Data Request:                                         │
│ ┌─────────────────────────────────────────────────────────────┐  │
│ │ Domain:                                                     │  │
│ │ • Name: Seaport                                            │  │
│ │ • Version: 1.5                                              │  │
│ │ • Chain ID: 1 (Ethereum Mainnet)                           │  │
│ │ • Verifying Contract: 0x0000...0045                        │  │
│ │                                                             │  │
│ │ Message Type: Order                                         │  │
│ │                                                             │  │
│ │ Order Details:                                              │  │
│ │ ┌─────────────────────────────────────────────────┐        │  │
│ │ │ Offerer: 0x742d35Cc6634C0532925a3b844Bc9e75     │        │  │
│ │ │                                                   │        │  │
│ │ │ Offer:                                            │        │  │
│ │ │ • 1 Bored Ape #5847                              │        │  │
│ │ │                                                   │        │  │
│ │ │ Consideration:                                    │        │  │
│ │ │ • 85 ETH to offerer                              │        │  │
│ │ │ • 2.125 ETH to 0x8De9...f392 (fee)              │        │  │
│ │ │                                                   │        │  │
│ │ │ Expiration: 2025-01-13 14:32:00 UTC              │        │  │
│ │ └─────────────────────────────────────────────────┘        │  │
│ │                                                             │  │
│ │ [V] View raw data  [D] Download JSON                       │  │
│ └─────────────────────────────────────────────────────────────┘  │
│                                                                  │
│ Total Value: 87.125 ETH ($217,812.50)                          │
│                                                                  │
│ [S] Sign order  [R] Reject  [H] What is EIP-712?  [Esc] Cancel │
└──────────────────────────────────────────────────────────────────┘
```

---

## Multi-Chain Sign

### Multi-Chain Transaction Screen

```
┌─ Multi-Chain Sign Operation ─────────────────────────────────────┐
│                                                                  │
│ Cross-Chain Transaction:                                         │
│ ┌─────────────────────────────────────────────────────────────┐  │
│ │ Operation: Bridge ETH to Polygon                           │  │
│ │                                                             │  │
│ │ Source Chain: Ethereum Mainnet                             │  │
│ │ • From: company_treasury (0x742d...2bd)                    │  │
│ │ • Amount: 5.0 ETH                                          │  │
│ │ • Bridge Fee: 0.01 ETH                                     │  │
│ │                                                             │  │
│ │ Destination Chain: Polygon                                  │  │
│ │ • To: 0x742d...2bd (same address)                         │  │
│ │ • Receive: 4.99 ETH (after fees)                          │  │
│ │ • Estimated arrival: 10-15 minutes                         │  │
│ │                                                             │  │
│ │ Bridge Protocol: Official Polygon Bridge                   │  │
│ │ • Contract: 0xA0c6...8c55 ✅ Verified                     │  │
│ │ • Total Value Locked: $2.3B                                │  │
│ │ • Daily Volume: $45M                                       │  │
│ └─────────────────────────────────────────────────────────────┘  │
│                                                                  │
│ Required Operations:                                             │
│ 1. Approve bridge contract (if needed)                          │
│ 2. Initiate bridge transaction                                  │
│ 3. Wait for confirmations (20 blocks)                           │
│ 4. Claim on destination chain                                   │
│                                                                  │
│ [S] Start bridging  [C] Calculate fees  [R] Risks  [Esc] Cancel│
└──────────────────────────────────────────────────────────────────┘
```

---

## Manage Participants

### Participant Management Screen

```
┌─ Manage Participants: company_treasury ──────────────────────────┐
│                                                                  │
│ Current Configuration: 2-of-3 threshold                         │
│                                                                  │
│ Active Participants:                                             │
│ ┌─────────────────────────────────────────────────────────────┐  │
│ │ [1] alice (You)                                             │  │
│ │     Role: Administrator                                     │  │
│ │     Status: 🟢 Online                                       │  │
│ │     Public Key: 0x04a1b2c3d4e5f6...                       │  │
│ │     Added: 2025-01-10                                      │  │
│ │     Permissions: Full access                               │  │
│ │                                                             │  │
│ │ [2] bob                                                     │  │
│ │     Role: Signer                                           │  │
│ │     Status: 🟢 Online (last seen: 5 min ago)              │  │
│ │     Public Key: 0x04d5e6f7a8b9c0...                       │  │
│ │     Added: 2025-01-10                                      │  │
│ │     Permissions: Sign only                                 │  │
│ │                                                             │  │
│ │ [3] carol                                                   │  │
│ │     Role: Signer                                           │  │
│ │     Status: 🔴 Offline (last seen: 2 days ago)            │  │
│ │     Public Key: 0x0489abcdef0123...                       │  │
│ │     Added: 2025-01-10                                      │  │
│ │     Permissions: Sign only                                 │  │
│ └─────────────────────────────────────────────────────────────┘  │
│                                                                  │
│ [A] Add participant  [R] Remove  [E] Edit permissions          │
│ [T] Change threshold  [I] Invite  [Esc] Back                  │
└──────────────────────────────────────────────────────────────────┘
```

### Add Participant Screen

```
┌─ Add New Participant ────────────────────────────────────────────┐
│                                                                  │
│ Add Participant to: company_treasury                            │
│                                                                  │
│ ⚠️  Adding participants requires threshold approval              │
│                                                                  │
│ Participant Information:                                         │
│ ┌─────────────────────────────────────────────────────────────┐  │
│ │ Device ID: [dave-mpc-node________________] (required)      │  │
│ │                                                             │  │
│ │ Display Name: [Dave Johnson______________] (optional)      │  │
│ │                                                             │  │
│ │ Role:                                                       │  │
│ │ ○ Administrator (full access)                              │  │
│ │ ● Signer (sign transactions only)                          │  │
│ │ ○ Observer (view only)                                     │  │
│ │                                                             │  │
│ │ Permissions:                                                │  │
│ │ [✓] Sign transactions                                      │  │
│ │ [ ] Initiate transactions                                  │  │
│ │ [ ] Manage participants                                    │  │
│ │ [ ] Export wallet data                                     │  │
│ │ [ ] Change settings                                        │  │
│ │                                                             │  │
│ │ Invitation Method:                                          │  │
│ │ ● Send invitation link                                     │  │
│ │ ○ Display QR code                                          │  │
│ │ ○ Manual coordination                                      │  │
│ └─────────────────────────────────────────────────────────────┘  │
│                                                                  │
│ New threshold will be: 2-of-4 (unchanged)                       │
│                                                                  │
│ [S] Send invitation  [P] Preview changes  [Esc] Cancel         │
└──────────────────────────────────────────────────────────────────┘
```

---

## Rotate Keys

### Key Rotation Wizard

```
┌─ Key Rotation: company_treasury ─────────────────────────────────┐
│                                                                  │
│ 🔄 Key Rotation Process                    Step 1 of 4          │
│                                                                  │
│ Current Key Information:                                         │
│ ┌─────────────────────────────────────────────────────────────┐  │
│ │ Wallet: company_treasury (2-of-3)                          │  │
│ │ Created: 2025-01-10 (2 days ago)                           │  │
│ │ Last Rotation: Never                                        │  │
│ │ Total Transactions: 47                                      │  │
│ │ Current Participants: alice, bob, carol                    │  │
│ └─────────────────────────────────────────────────────────────┘  │
│                                                                  │
│ Rotation Options:                                                │
│ ┌─────────────────────────────────────────────────────────────┐  │
│ │ ● Refresh Keys Only                                         │  │
│ │   Keep same participants and threshold                     │  │
│ │   Generate new key shares for security                     │  │
│ │                                                             │  │
│ │ ○ Change Participants                                       │  │
│ │   Add or remove signers during rotation                    │  │
│ │                                                             │  │
│ │ ○ Modify Threshold                                          │  │
│ │   Change from 2-of-3 to different scheme                   │  │
│ │                                                             │  │
│ │ ○ Complete Restructure                                      │  │
│ │   Change participants, threshold, and keys                 │  │
│ └─────────────────────────────────────────────────────────────┘  │
│                                                                  │
│ ⚠️  Key rotation requires all participants to be online         │
│                                                                  │
│ [N] Next step  [C] Check participants  [Esc] Cancel            │
└──────────────────────────────────────────────────────────────────┘
```

### Rotation Progress Screen

```
┌─ Key Rotation in Progress ───────────────────────────────────────┐
│                                                                  │
│ Executing Key Rotation...                    Step 3 of 4        │
│                                                                  │
│ Overall Progress: [██████████████░░░░░░░░] 70%                 │
│                                                                  │
│ ┌─────────────────────────────────────────────────────────────┐  │
│ │ Phase 1: Preparation                    ✅ Complete         │  │
│ │ • Participants verified                                     │  │
│ │ • Backup created                                            │  │
│ │ • Session established                                       │  │
│ │                                                             │  │
│ │ Phase 2: Key Generation                 ✅ Complete         │  │
│ │ • New shares generated                                      │  │
│ │ • Commitments exchanged                                     │  │
│ │ • Shares distributed                                        │  │
│ │                                                             │  │
│ │ Phase 3: Verification                   ⏳ In Progress      │  │
│ │ • alice: ✅ Verified                                       │  │
│ │ • bob: ⏳ Verifying share...                               │  │
│ │ • carol: ⏳ Waiting                                        │  │
│ │                                                             │  │
│ │ Phase 4: Activation                     ⏸️  Pending         │  │
│ │ • Update wallet configuration                              │  │
│ │ • Archive old keys                                         │  │
│ │ • Confirm new addresses                                    │  │
│ └─────────────────────────────────────────────────────────────┘  │
│                                                                  │
│ Estimated time remaining: 2 minutes                             │
│                                                                  │
│ [A] Abort (safe)  [L] View logs  [?] Help                      │
└──────────────────────────────────────────────────────────────────┘
```

---

## Lock/Unlock Wallet

### Lock Wallet Screen

```
┌─ Lock Wallet: company_treasury ──────────────────────────────────┐
│                                                                  │
│ 🔒 Lock Wallet                                                   │
│                                                                  │
│ This will temporarily disable all operations on this wallet.    │
│                                                                  │
│ Lock Configuration:                                              │
│ ┌─────────────────────────────────────────────────────────────┐  │
│ │ Lock Duration:                                              │  │
│ │ ● Until manually unlocked                                   │  │
│ │ ○ For specific duration: [___] hours                       │  │
│ │ ○ Until date: [2025-01-15 00:00]                          │  │
│ │                                                             │  │
│ │ Lock Reason: (required for audit)                          │  │
│ │ [Security review - unusual activity detected_____________] │  │
│ │                                                             │  │
│ │ Operations to Disable:                                      │  │
│ │ [✓] Outgoing transactions                                  │  │
│ │ [✓] Message signing                                        │  │
│ │ [✓] Participant changes                                    │  │
│ │ [✓] Key operations                                         │  │
│ │ [ ] View balance (read-only)                               │  │
│ │                                                             │  │
│ │ Unlock Requirements:                                        │  │
│ │ ● Requires 2-of-3 approval (same as threshold)            │  │
│ │ ○ Require all participants                                 │  │
│ │ ○ Time-locked (auto-unlock)                                │  │
│ └─────────────────────────────────────────────────────────────┘  │
│                                                                  │
│ ⚠️  Active operations will be cancelled                         │
│                                                                  │
│ [L] Lock wallet  [C] Cancel                                     │
└──────────────────────────────────────────────────────────────────┘
```

### Unlock Wallet Screen

```
┌─ Unlock Wallet: company_treasury ────────────────────────────────┐
│                                                                  │
│ 🔓 Unlock Wallet Request                                         │
│                                                                  │
│ Wallet Status:                                                   │
│ ┌─────────────────────────────────────────────────────────────┐  │
│ │ Status: 🔒 LOCKED                                           │  │
│ │ Locked by: alice                                            │  │
│ │ Lock time: 2025-01-12 10:30:00 UTC (4 hours ago)          │  │
│ │ Reason: Security review - unusual activity detected        │  │
│ │                                                             │  │
│ │ Lock Type: Manual unlock required                           │  │
│ │ Required approvals: 2 of 3                                 │  │
│ └─────────────────────────────────────────────────────────────┘  │
│                                                                  │
│ Unlock Authorization:                                            │
│ ┌─────────────────────────────────────────────────────────────┐  │
│ │ Current Approvals: 1 of 2 required                         │  │
│ │                                                             │  │
│ │ ✅ alice (You)     Approved at 14:45:00                    │  │
│ │ ⏳ bob            Approval pending (notified)              │  │
│ │ ⏳ carol          Not required                             │  │
│ │                                                             │  │
│ │ Unlock Reason: (required)                                   │  │
│ │ [Security review completed - no issues found_____________] │  │
│ └─────────────────────────────────────────────────────────────┘  │
│                                                                  │
│ [A] Approve unlock  [N] Notify others  [C] Cancel              │
└──────────────────────────────────────────────────────────────────┘
```

---

## View Activity Log

### Activity Log Screen

```
┌─ Activity Log: company_treasury ─────────────────────────────────┐
│                                                                  │
│ Filter: [All Activities ▼] [Last 7 days ▼]    Search: [____]   │
│                                                                  │
│ ┌─────────────────────────────────────────────────────────────┐  │
│ │ 2025-01-12 14:32  Transaction Signed                        │  │
│ │ • Type: Send 1.5 ETH to 0x742d...5f2                      │  │
│ │ • Signers: alice, bob                                      │  │
│ │ • Status: ✅ Confirmed                                     │  │
│ │ • Tx Hash: 0x3f4e5a6b7c8d9e0f...                         │  │
│ │                                                             │  │
│ │ 2025-01-12 13:15  Participant Added                         │  │
│ │ • New participant: dave                                     │  │
│ │ • Added by: alice                                          │  │
│ │ • Approved by: alice, bob                                  │  │
│ │                                                             │  │
│ │ 2025-01-11 16:22  Configuration Change                      │  │
│ │ • Setting: Network timeout                                  │  │
│ │ • Changed from: 30s to 60s                                 │  │
│ │ • Changed by: alice                                        │  │
│ │                                                             │  │
│ │ 2025-01-11 09:45  Failed Transaction                        │  │
│ │ • Type: Send 0.5 ETH                                       │  │
│ │ • Reason: Insufficient signatures (1/2)                    │  │
│ │ • Initiated by: carol                                      │  │
│ └─────────────────────────────────────────────────────────────┘  │
│                                                                  │
│ Showing 4 of 127 activities           Page 1 of 32 [◀][▶]      │
│                                                                  │
│ [E] Export log  [F] Filter  [S] Statistics  [Esc] Back         │
└──────────────────────────────────────────────────────────────────┘
```

---

## Test Connections

### Connection Test Screen

```
┌─ Test Connections: company_treasury ─────────────────────────────┐
│                                                                  │
│ Testing Participant Connections...                               │
│                                                                  │
│ ┌─────────────────────────────────────────────────────────────┐  │
│ │ Participant Test Results:                                   │  │
│ │                                                             │  │
│ │ alice (You)                                                 │  │
│ │ ├─ Device Status: ✅ Online                                │  │
│ │ ├─ Network: ✅ Connected to signaling server              │  │
│ │ ├─ WebRTC: ✅ STUN/TURN accessible                        │  │
│ │ └─ Keys: ✅ Valid and accessible                          │  │
│ │                                                             │  │
│ │ bob                                                         │  │
│ │ ├─ Device Status: ✅ Online                                │  │
│ │ ├─ P2P Connection: ✅ Established (45ms latency)          │  │
│ │ ├─ Last Seen: 2 minutes ago                               │  │
│ │ └─ Signing Ready: ✅ Yes                                  │  │
│ │                                                             │  │
│ │ carol                                                       │  │
│ │ ├─ Device Status: ❌ Offline                               │  │
│ │ ├─ P2P Connection: ❌ Failed (timeout)                    │  │
│ │ ├─ Last Seen: 2 days ago                                  │  │
│ │ └─ Alternative: Email notification available              │  │
│ └─────────────────────────────────────────────────────────────┘  │
│                                                                  │
│ Overall Status: ⚠️  2/3 participants available                  │
│ Threshold operations: ✅ Possible (2-of-3 required)            │
│                                                                  │
│ [R] Re-test  [N] Notify offline  [D] Details  [Esc] Back       │
└──────────────────────────────────────────────────────────────────┘
```

---

## Export Details

### Export Wallet Details Screen

```
┌─ Export Wallet Details: company_treasury ────────────────────────┐
│                                                                  │
│ Select Export Options:                                           │
│ ┌─────────────────────────────────────────────────────────────┐  │
│ │ What to Export:                                             │  │
│ │                                                             │  │
│ │ Key Information:                                            │  │
│ │ [✓] Public addresses (safe to share)                       │  │
│ │ [ ] Public keys (extended)                                 │  │
│ │ [ ] Key shares (SENSITIVE - requires password)             │  │
│ │                                                             │  │
│ │ Configuration:                                              │  │
│ │ [✓] Wallet name and description                           │  │
│ │ [✓] Threshold configuration                                │  │
│ │ [✓] Participant list                                       │  │
│ │ [ ] Network settings                                       │  │
│ │                                                             │  │
│ │ History:                                                    │  │
│ │ [ ] Transaction history                                     │  │
│ │ [ ] Activity logs                                          │  │
│ │ [ ] Audit trail                                            │  │
│ │                                                             │  │
│ │ Export Format:                                              │  │
│ │ ● JSON (structured data)                                   │  │
│ │ ○ PDF (human readable)                                     │  │
│ │ ○ CSV (spreadsheet compatible)                            │  │
│ └─────────────────────────────────────────────────────────────┘  │
│                                                                  │
│ Security Options:                                                │
│ [✓] Encrypt with password                                       │
│ [ ] Split into multiple files                                   │
│                                                                  │
│ [E] Export  [P] Preview  [S] Save preset  [Esc] Cancel         │
└──────────────────────────────────────────────────────────────────┘
```

---

## Advanced Settings

### Advanced Wallet Settings

```
┌─ Advanced Settings: company_treasury ────────────────────────────┐
│                                                                  │
│ Technical Configuration:                                         │
│                                                                  │
│ Protocol Settings:                                               │
│ ┌─────────────────────────────────────────────────────────────┐  │
│ │ DKG Protocol:                                               │  │
│ │ • Round timeout: [300s_____] (5 minutes)                   │  │
│ │ • Max retries: [3__]                                       │  │
│ │ • Commitment scheme: Feldman VSS                           │  │
│ │                                                             │  │
│ │ Signing Protocol:                                           │  │
│ │ • Signing timeout: [3600s___] (1 hour)                     │  │
│ │ • Nonce generation: ● Deterministic  ○ Random             │  │
│ │ • Preprocessing: [ ] Enable (faster signing)               │  │
│ └─────────────────────────────────────────────────────────────┘  │
│                                                                  │
│ Network Optimization:                                            │
│ ┌─────────────────────────────────────────────────────────────┐  │
│ │ P2P Settings:                                               │  │
│ │ • Max peers: [10___]                                       │  │
│ │ • Connection timeout: [30s____]                            │  │
│ │ • [ ] Prefer direct connections                            │  │
│ │ • [✓] Enable connection caching                            │  │
│ └─────────────────────────────────────────────────────────────┘  │
│                                                                  │
│ Developer Options:                                               │
│ [ ] Enable debug logging                                         │
│ [ ] Export raw protocol messages                                 │
│ [ ] Allow experimental features                                  │
│                                                                  │
│ ⚠️  Modifying these settings may affect security                │
│                                                                  │
│ [A] Apply  [R] Reset to defaults  [T] Test config  [Esc] Back  │
└──────────────────────────────────────────────────────────────────┘
```

This comprehensive wallet operations submenu wireframe document provides detailed layouts for all wallet-specific operations, maintaining consistency with the enterprise-grade interface while ensuring all critical wallet management functions are easily accessible and clearly presented.