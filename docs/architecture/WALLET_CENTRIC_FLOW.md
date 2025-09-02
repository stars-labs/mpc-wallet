# Wallet-Centric User Flow Design

## Core Principle
Users care about **wallets**, not technical implementation details. The UI should focus on wallet management and signing operations, with coordination methods (hot/cold) being a secondary choice made at the right moment.

## Main Flow

### 1. Initial Screen (Wallet Management)
```
┌─ MPC Wallet ────────────────────────────────────────────────────────┐
│                                                                      │
│                     🔐 Threshold Wallet Manager                      │
│                                                                      │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │  📁 Existing Wallets                                       │    │
│  │                                                            │    │
│  │  • Business Wallet (2/3)         Balance: 12.5 ETH       │    │
│  │  • Personal Wallet (2/2)         Balance: 5.2 ETH        │    │
│  │  • Treasury Wallet (3/5)         Balance: 45.1 ETH       │    │
│  └────────────────────────────────────────────────────────────┘    │
│                                                                      │
│  What would you like to do?                                        │
│                                                                      │
│  [1] Select Wallet                                                  │
│  [2] Create New Wallet                                              │
│  [3] Import Wallet                                                  │
│  [4] Exit                                                           │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### 2. Create Wallet Flow

#### 2.1 Wallet Type Selection
```
┌─ Create New Wallet ─────────────────────────────────────────────────┐
│                                                                      │
│  Choose wallet configuration:                                       │
│                                                                      │
│  [1] 🏢 Business (2-of-3)                                          │
│      Recommended for company operations                             │
│                                                                      │
│  [2] 👥 Personal (2-of-2)                                          │
│      Enhanced security for personal funds                           │
│                                                                      │
│  [3] 🏛️ Treasury (3-of-5)                                          │
│      High security for large holdings                               │
│                                                                      │
│  [4] 🛠️ Custom                                                      │
│      Define your own threshold                                      │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

#### 2.2 Coordination Method Selection
```
┌─ Setup Coordination ─────────────────────────────────────────────────┐
│                                                                      │
│  How will participants coordinate?                                  │
│                                                                      │
│  [1] 🌐 Network (Recommended)                                      │
│      • Real-time coordination via internet                          │
│      • Participants join via session code                           │
│      • Fastest setup and signing                                    │
│                                                                      │
│  [2] 📁 File Transfer (Air-Gapped)                                 │
│      • No network required                                          │
│      • Exchange data via USB/SD cards                               │
│      • Maximum security for cold storage                            │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

#### 2.3A Network Coordination Flow
```
┌─ Network Setup ──────────────────────────────────────────────────────┐
│                                                                      │
│  Session Code: wallet-2of3-abc123                                   │
│                                                                      │
│  Share this code with participants:                                 │
│  ┌──────────────────────────────────────────────────────────┐      │
│  │                    [QR CODE HERE]                         │      │
│  └──────────────────────────────────────────────────────────┘      │
│                                                                      │
│  Participants (2/3):                                                │
│  ✓ You (Device-1)              Connected                           │
│  ⏳ Waiting...                                                      │
│  ⏳ Waiting...                                                      │
│                                                                      │
│  [Start Key Generation] (enabled when all connected)                │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

#### 2.3B File Coordination Flow
```
┌─ File-Based Setup ───────────────────────────────────────────────────┐
│                                                                      │
│  Step 1: Initialize Your Key Share                                  │
│                                                                      │
│  [Generate Key Share]                                               │
│                                                                      │
│  Step 2: Exchange Setup Files                                       │
│                                                                      │
│  📤 Export your setup file to: /media/usb/wallet_setup_device1.json │
│  📥 Import setup files from other participants                      │
│                                                                      │
│  Files collected (1/3):                                             │
│  ✓ wallet_setup_device1.json (You)                                 │
│  ⏳ Waiting for 2 more files...                                     │
│                                                                      │
│  [Complete Setup] (enabled when all files imported)                 │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### 3. Wallet Operations Screen
```
┌─ Business Wallet (2/3) ──────────────────────────────────────────────┐
│                                                                      │
│  Balance: 12.5 ETH ($31,250)                                       │
│  Address: 0x742d...9636                                            │
│                                                                      │
│  Recent Transactions:                                               │
│  • Sent 0.5 ETH to 0x123...abc (2 hours ago)                      │
│  • Received 2.1 ETH from 0x456...def (Yesterday)                   │
│                                                                      │
│  [1] Send Transaction                                               │
│  [2] Receive (Show Address)                                         │
│  [3] Transaction History                                            │
│  [4] Export Wallet                                                  │
│  [5] Back to Wallet List                                           │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### 4. Signing Flow

#### 4.1 Coordination Method Selection (Per Transaction)
```
┌─ Send Transaction ───────────────────────────────────────────────────┐
│                                                                      │
│  To: 0x9876...5432                                                 │
│  Amount: 5.0 ETH                                                    │
│  Fee: 0.002 ETH                                                    │
│                                                                      │
│  How would you like to coordinate signing?                         │
│                                                                      │
│  [1] 🌐 Online Signing                                             │
│      • Instant if signers are online                               │
│      • Requires network connection                                  │
│                                                                      │
│  [2] 📁 Offline Signing                                            │
│      • Export signing request to file                              │
│      • Collect signatures via USB/SD                               │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

#### 4.2A Online Signing
```
┌─ Online Signing ─────────────────────────────────────────────────────┐
│                                                                      │
│  Requesting signatures for 5.0 ETH to 0x9876...5432                │
│                                                                      │
│  Required: 2 of 3 signatures                                        │
│                                                                      │
│  ✓ Your signature                    (Just now)                     │
│  ⏳ Device-2                         Notified                       │
│  ⏳ Device-3                         Notified                       │
│                                                                      │
│  ⚡ Transaction will be sent automatically when threshold is met    │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

#### 4.2B Offline Signing
```
┌─ Offline Signing ────────────────────────────────────────────────────┐
│                                                                      │
│  Step 1: Export Signing Request                                     │
│                                                                      │
│  📤 Exported to: /media/usb/tx_5eth_request.json                   │
│                                                                      │
│  Step 2: Collect Signatures                                         │
│                                                                      │
│  Share the request file with other signers.                        │
│  Import their signature files here:                                 │
│                                                                      │
│  Signatures (1/2):                                                  │
│  ✓ Your signature                                                  │
│  📥 [Import Signature File]                                         │
│                                                                      │
│  [Broadcast Transaction] (enabled when threshold met)               │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

## Key Design Principles

### 1. **Wallet-First Navigation**
- Everything starts from wallet selection
- Technical details are hidden unless necessary
- Clear visual hierarchy focused on user tasks

### 2. **Just-In-Time Choices**
- Don't ask about hot/cold at startup
- Ask about coordination method when it matters:
  - During wallet creation
  - When initiating a transaction
  
### 3. **Progressive Disclosure**
- Show only what's needed for the current task
- Advanced options available but not prominent
- Technical details in tooltips/help, not main UI

### 4. **Unified Experience**
- Same wallet works with both coordination methods
- User can choose per-operation
- No separate "modes" to manage

### 5. **Clear Status Communication**
- Visual progress indicators
- Plain language explanations
- Next steps always clear

## Implementation Notes

### Remove from UI:
- Log window (write to file: `~/.frost_keystore/logs/`)
- Device connection details (unless relevant)
- Session/mesh status (internal state)
- DKG technical status
- Complex commands

### Add to UI:
- Wallet balance display
- Transaction history
- Clear progress bars
- Contextual help
- File operation status

### File Structure:
```
~/.frost_keystore/
├── wallets/
│   ├── wallet_abc123.json
│   └── wallet_def456.json
├── coordination/
│   ├── pending/
│   └── completed/
└── logs/
    └── frost_wallet.log
```