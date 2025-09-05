# Hybrid Mode E2E Test Design

## Overview

This document outlines the hybrid operational mode where some MPC participants operate online (via WebSocket/WebRTC) while others remain offline (air-gapped with SD card exchange). This reflects real-world scenarios where high-security keys are kept offline while convenience signers operate online.

## Architecture

```
┌─────────────────┐        WebSocket         ┌─────────────────┐
│   Online Node   │◄────────────────────────►│   Online Node   │
│   (Alice - P1)  │                          │   (Bob - P2)    │
└────────┬────────┘        WebRTC            └────────┬────────┘
         │         ◄──────────────────────────►        │
         │                                              │
         │              SD Card Exchange               │
         └──────────────────────────────────────────────┘
                               │
                               ▼
                    ┌─────────────────┐
                    │  Offline Node   │
                    │ (Charlie - P3)  │
                    │  (Air-gapped)   │
                    └─────────────────┘
```

## Test Scenarios

### 🌐 Scenario 1: Hybrid DKG (2 Online + 1 Offline)

**Setup:**
- Alice (P1): Online coordinator
- Bob (P2): Online participant  
- Charlie (P3): Offline participant
- Threshold: 2-of-3
- Curves: Both secp256k1 (Ethereum) and ed25519 (Solana)

**DKG Flow:**

1. **Round 1 - Commitment Generation**
   - Alice & Bob: Exchange commitments via WebRTC
   - Charlie: Generates commitment offline, exports to SD card
   - Alice: Collects Charlie's commitment from SD card

2. **Round 2 - Share Distribution**
   - Alice & Bob: Exchange shares via encrypted WebRTC
   - Charlie: Receives aggregated data via SD card
   - Charlie: Generates shares, exports to SD card
   - Alice & Bob: Import Charlie's shares from SD card

3. **Round 3 - Finalization**
   - All parties finalize locally
   - Group public keys verified across all participants

### 💰 Scenario 2: Hybrid Ethereum Transaction Signing

**Transaction:** 
- Type: ETH Transfer
- Amount: 2.5 ETH
- To: 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7

**Signing Participants:** Alice (online) + Charlie (offline)

**Flow:**
1. Alice initiates transaction online
2. Alice generates commitment, broadcasts via WebSocket
3. Charlie receives transaction via SD card
4. Charlie generates commitment offline, exports to SD card
5. Alice imports Charlie's commitment
6. Both generate signature shares
7. Alice aggregates and broadcasts

### ☀️ Scenario 3: Hybrid Solana Transaction Signing

**Transaction:**
- Type: SOL Transfer
- Amount: 100 SOL
- To: 9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM

**Signing Participants:** Bob (online) + Charlie (offline)

**Flow:**
1. Bob creates Solana transaction
2. Bob's commitment sent via WebSocket
3. SD card exchange for Charlie
4. Signature aggregation
5. Transaction submission to Solana

### 🪙 Scenario 4: SPL Token Transfer (Solana)

**Transaction:**
- Token: USDC (SPL)
- Amount: 500 USDC
- Program: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA

**Signing Participants:** Alice (online) + Bob (online)
- Charlie remains offline but could participate if needed

### 🔄 Scenario 5: Emergency Signing (All Offline)

**Situation:** Network compromise detected, all nodes switch to offline mode

**Flow:**
1. All nodes disconnect from network
2. Transaction created offline
3. SD card round-robin for commitments
4. SD card round-robin for shares
5. Final signature assembly offline

## Implementation Components

### 1. Network Simulator
```rust
struct NetworkSimulator {
    online_nodes: HashMap<ParticipantId, OnlineNode>,
    offline_nodes: HashMap<ParticipantId, OfflineNode>,
    websocket_hub: WebSocketHub,
    webrtc_mesh: WebRTCMesh,
    sd_card: MockSDCard,
}
```

### 2. Solana Transaction Builder
```rust
struct SolanaTransaction {
    instructions: Vec<Instruction>,
    recent_blockhash: Hash,
    fee_payer: Pubkey,
}

impl SolanaTransaction {
    fn transfer_sol(from: &Pubkey, to: &Pubkey, lamports: u64) -> Self;
    fn transfer_spl_token(token: &Pubkey, from: &Pubkey, to: &Pubkey, amount: u64) -> Self;
    fn create_associated_token_account(wallet: &Pubkey, mint: &Pubkey) -> Self;
}
```

### 3. Hybrid Coordinator
```rust
struct HybridCoordinator {
    online_transport: OnlineTransport,
    offline_transport: OfflineTransport,
    message_queue: MessageQueue,
}

impl HybridCoordinator {
    async fn coordinate_dkg(&mut self) -> Result<GroupKey>;
    async fn coordinate_signing(&mut self, tx: Transaction) -> Result<Signature>;
    fn bridge_online_offline(&mut self) -> Result<()>;
}
```

## Test Execution Plan

### Phase 1: Setup
1. Initialize 3 participants with mixed online/offline status
2. Establish WebSocket connections for online nodes
3. Setup WebRTC data channels
4. Initialize SD card simulation for offline node

### Phase 2: Hybrid DKG
1. Execute DKG with online nodes communicating via WebRTC
2. Bridge offline node via SD card exchanges
3. Verify all nodes derive same group keys
4. Save keystores for all participants

### Phase 3: Ethereum Signing
1. Create ETH transaction
2. Sign with Alice (online) + Charlie (offline)
3. Verify signature
4. Test with different participant combinations

### Phase 4: Solana Signing
1. Create SOL transfer transaction
2. Sign with Bob (online) + Charlie (offline)
3. Create SPL token transfer
4. Sign with Alice + Bob (both online)
5. Verify ed25519 signatures

### Phase 5: Stress Testing
1. Simulate network failures
2. Test offline fallback
3. Verify signature consistency
4. Test concurrent transactions

## Security Considerations

### Online Nodes
- TLS 1.3 for WebSocket
- DTLS for WebRTC
- Authenticated channels
- Rate limiting

### Offline Node
- Air-gap enforcement
- SD card encryption
- Physical security
- Audit logging

### Bridge Security
- One-way data flow enforcement
- Sanitization of SD card data
- Verification of all imported data
- Time-based validity windows

## Success Criteria

1. **DKG Success**: All nodes derive identical group keys
2. **Signing Success**: Valid signatures from any 2-of-3 combination
3. **Hybrid Operation**: Seamless online/offline coordination
4. **Multi-Chain**: Both Ethereum and Solana transactions work
5. **Security**: No key material leakage between online/offline
6. **Performance**: < 5 seconds for online, < 30 seconds for hybrid

## Expected Output

```
🚀 Hybrid Mode E2E Test
========================

Phase 1: Setup
✅ Alice (P1): Online - WebSocket connected
✅ Bob (P2): Online - WebRTC ready
✅ Charlie (P3): Offline - SD card initialized

Phase 2: Hybrid DKG
✅ Online nodes exchanged via WebRTC
✅ Offline node bridged via SD card
✅ Group keys match across all nodes
  Ethereum: 0x1234...
  Solana: 9WzDX...

Phase 3: Ethereum Transactions
✅ ETH transfer signed (Alice + Charlie)
✅ ERC20 transfer signed (Bob + Charlie)
✅ Signatures verified

Phase 4: Solana Transactions
✅ SOL transfer signed (Bob + Charlie)
✅ SPL token transfer signed (Alice + Bob)
✅ Ed25519 signatures valid

Phase 5: Stress Tests
✅ Network failure handled
✅ Offline fallback successful
✅ Concurrent signing works

Summary: All tests passed!
```

## Implementation Files

```
apps/tui-node/
├── src/
│   ├── hybrid/
│   │   ├── mod.rs
│   │   ├── coordinator.rs
│   │   ├── online_transport.rs
│   │   └── offline_transport.rs
│   └── solana/
│       ├── mod.rs
│       ├── transaction.rs
│       └── spl_token.rs
└── examples/
    └── hybrid_mode_e2e_test.rs
```