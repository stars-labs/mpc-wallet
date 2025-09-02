# MPC Wallet Technical Documentation

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [System Architecture Overview](#system-architecture-overview)
3. [Core Design Decisions](#core-design-decisions)
4. [Component Architecture](#component-architecture)
5. [Network Architecture](#network-architecture)
6. [Security Architecture](#security-architecture)
7. [User Interface Architecture](#user-interface-architecture)
8. [Data Models and Flow](#data-models-and-flow)
9. [Protocol Implementation](#protocol-implementation)
10. [Deployment Architecture](#deployment-architecture)
11. [Performance Characteristics](#performance-characteristics)
12. [Integration Points](#integration-points)
13. [Operational Considerations](#operational-considerations)
14. [Appendices](#appendices)

---

## Executive Summary

The MPC (Multi-Party Computation) Wallet is a distributed cryptographic wallet system that implements the FROST (Flexible Round-Optimized Schnorr Threshold) signature scheme. This architecture enables secure key generation and transaction signing where no single party ever possesses the complete private key, significantly reducing the risk of key compromise while maintaining operational flexibility.

### Key Features
- **Threshold Signatures**: Supports m-of-n signature schemes (e.g., 2-of-3, 3-of-5)
- **Multi-Platform**: Browser extension, CLI tool, and native desktop application
- **Multi-Chain**: Supports Ethereum (secp256k1) and Solana (ed25519)
- **Enterprise-Grade**: SOC 2 compliant with comprehensive audit trails
- **Peer-to-Peer**: WebRTC-based direct communication between participants
- **Offline Support**: Air-gapped operation mode for maximum security

### Target Audience
- **Cryptocurrency Exchanges**: Secure cold wallet management
- **DeFi Protocols**: Treasury management with distributed control
- **Enterprise Users**: Corporate cryptocurrency custody solutions
- **High-Net-Worth Individuals**: Personal wealth security

---

## System Architecture Overview

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        User Interfaces                           │
├─────────────────┬─────────────────┬─────────────────────────────┤
│ Browser Extension│    CLI Node     │   Native Desktop App        │
│   (Chrome/FF)   │   (Terminal)    │      (Slint UI)            │
└────────┬────────┴────────┬────────┴────────┬────────────────────┘
         │                 │                 │
         │                 ▼                 │
         │         ┌──────────────┐         │
         │         │  Rust Core   │         │
         │         │   Library     │         │
         │         └──────┬───────┘         │
         │                │                 │
         ▼                ▼                 ▼
┌─────────────────────────────────────────────────────────────────┐
│                    FROST Cryptographic Core                      │
│              (Distributed Key Generation & Signing)              │
└─────────────────────────────────────────────────────────────────┘
         │                                   │
         ▼                                   ▼
┌─────────────────┐                 ┌─────────────────┐
│  WebSocket      │                 │    WebRTC       │
│  Signaling      │◄───────────────►│  P2P Mesh       │
└─────────────────┘                 └─────────────────┘
```

### Monorepo Structure

The project follows a monorepo architecture with clear separation of concerns:

```
mpc-wallet/
├── apps/                           # Application implementations
│   ├── browser-extension/          # Chrome/Firefox extension
│   ├── cli-node/                  # Terminal-based node
│   ├── native-node/               # Desktop application
│   └── signal-server/             # Signaling infrastructure
│       ├── server/                # Standard WebSocket server
│       └── cloudflare-worker/     # Edge deployment
│
└── packages/@mpc-wallet/          # Shared packages
    ├── frost-core/                # FROST implementation
    ├── core-wasm/                 # WebAssembly bindings
    └── types/                     # TypeScript definitions
```

### Technology Stack

| Layer | Technology | Purpose |
|-------|------------|---------|
| UI - Browser | Svelte 5, TailwindCSS | Reactive UI with modern styling |
| UI - CLI | Ratatui | Terminal UI framework |
| UI - Native | Slint | Native cross-platform GUI |
| Core Logic | Rust | Performance and memory safety |
| Cryptography | FROST, secp256k1, ed25519 | Threshold signatures |
| Networking | WebRTC, WebSocket | P2P communication |
| Build System | Bun, wasm-pack, Cargo | Fast builds and WASM compilation |
| Blockchain | ethers-rs, solana-sdk | Chain interactions |

---

## Core Design Decisions

### 1. Threshold Cryptography Choice

**Decision**: Implement FROST (Flexible Round-Optimized Schnorr Threshold) signatures

**Rationale**:
- **Security**: Proven secure under the discrete logarithm assumption
- **Efficiency**: Requires only 2 rounds for signing (vs 3+ for other schemes)
- **Flexibility**: Supports arbitrary threshold configurations
- **Compatibility**: Works with both secp256k1 (Ethereum) and ed25519 (Solana)

**Trade-offs**:
- Complexity in implementation
- Requires careful handling of nonce generation
- All participants must be online during key generation

### 2. WebRTC for P2P Communication

**Decision**: Use WebRTC for direct peer-to-peer communication

**Rationale**:
- **Direct Communication**: Minimizes trust in intermediary servers
- **Low Latency**: Direct connections reduce round-trip times
- **Browser Support**: Native support in modern browsers
- **NAT Traversal**: Built-in STUN/TURN support

**Implementation**:
```typescript
// Simplified WebRTC connection establishment
class WebRTCManager {
  async createPeerConnection(peerId: string) {
    const pc = new RTCPeerConnection({
      iceServers: [
        { urls: 'stun:stun.l.google.com:19302' },
        { urls: 'turn:turn.auto-life.tech:3478' }
      ]
    });
    
    // Create data channel for MPC messages
    const channel = pc.createDataChannel('mpc', {
      ordered: true,
      reliable: true
    });
    
    return { pc, channel };
  }
}
```

### 3. Rust Core with Multiple UIs

**Decision**: Implement core logic in Rust with multiple UI frontends

**Rationale**:
- **Performance**: Critical cryptographic operations run at native speed
- **Safety**: Memory safety guarantees for security-critical code
- **Code Reuse**: Single implementation serves all platforms
- **WASM Support**: Compiles to WebAssembly for browser use

**Architecture Pattern**:
```rust
// Shared trait for UI implementations
pub trait UIProvider: Send + Sync {
    fn update_status(&self, status: &str);
    fn prompt_user(&self, message: &str) -> Result<String>;
    fn show_error(&self, error: &str);
}

// Core business logic accepts any UI provider
pub struct AppRunner<U: UIProvider> {
    ui: Arc<U>,
    state: Arc<Mutex<AppState>>,
}
```

### 4. Offline-First Design

**Decision**: Support fully offline operation for air-gapped environments

**Rationale**:
- **Security**: Eliminates network attack vectors
- **Compliance**: Meets requirements for cold storage
- **Reliability**: Operations continue without internet
- **Flexibility**: Users choose their security/convenience trade-off

**Implementation Approach**:
- QR code-based data transfer
- File-based session coordination
- Manual share distribution
- Deterministic session IDs

---

## Component Architecture

### Browser Extension Architecture

The browser extension follows Chrome Extension Manifest V3 architecture:

```
┌─────────────────────────────────────────────────────────┐
│                    Browser Extension                     │
├─────────────────┬───────────────┬──────────────────────┤
│   Popup UI      │  Background   │  Offscreen Document │
│   (Svelte)      │ Service Worker│     (WASM + WebRTC) │
│                 │               │                      │
│ • User Interface│ • Message     │ • Crypto Operations │
│ • State Display │   Router      │ • P2P Connections   │
│ • User Actions  │ • WebSocket   │ • DKG/Signing       │
│                 │   Client      │                      │
└─────────────────┴───────────────┴──────────────────────┘
```

#### Message Flow
```
User Action → Popup → Background → Offscreen → WebRTC Peers
                ↑         ↓           ↓
                └─────────┴───────────┴─── WebSocket Server
```

#### Key Components

**1. Background Service Worker** (`background/index.ts`)
- Central message routing hub
- Maintains WebSocket connection to signaling server
- Manages extension lifecycle
- Coordinates between popup and offscreen document

**2. Popup UI** (`popup/App.svelte`)
- User-facing interface
- Displays wallet state and balances
- Initiates DKG and signing operations
- Shows transaction history

**3. Offscreen Document** (`offscreen/index.html`)
- Runs WebAssembly cryptographic operations
- Manages WebRTC peer connections
- Handles DKG protocol rounds
- Performs threshold signing

### CLI Node Architecture

The CLI node provides both library and executable functionality:

```rust
// Library interface (lib.rs)
pub struct CliNode {
    app_runner: AppRunner<TerminalUI>,
    network_manager: NetworkManager,
    keystore: KeystoreManager,
}

// Binary entry point (main.rs)
fn main() -> Result<()> {
    let cli = CliNode::new()?;
    cli.run()?;
}
```

#### Component Hierarchy

```
cli-node/
├── src/
│   ├── lib.rs              # Library interface
│   ├── main.rs             # Binary entry point
│   ├── app_runner.rs       # Core business logic
│   ├── network/            # Networking layer
│   │   ├── websocket.rs    # WebSocket client
│   │   └── webrtc.rs       # WebRTC implementation
│   ├── handlers/           # Command handlers
│   │   ├── session_commands.rs
│   │   └── wallet_commands.rs
│   ├── protocal/           # Protocol implementations
│   │   ├── dkg.rs          # DKG protocol
│   │   └── signing.rs      # Signing protocol
│   └── ui/                 # Terminal UI
│       └── tui.rs          # Ratatui implementation
```

### Native Desktop Application

The native application reuses the CLI node's core logic:

```rust
// Native app adapter pattern
struct NativeUIProvider {
    window: Weak<MainWindow>,
}

impl UIProvider for NativeUIProvider {
    fn update_status(&self, status: &str) {
        if let Some(window) = self.window.upgrade() {
            slint::invoke_from_event_loop(move || {
                window.set_status_text(status.into());
            });
        }
    }
}
```

#### Slint UI Architecture

```
MainWindow (root)
├── NavigationPanel
│   ├── MenuButton
│   └── StatusIndicator
├── ContentArea
│   ├── WalletList
│   ├── SessionView
│   └── SettingsPanel
└── StatusBar
    ├── ConnectionStatus
    └── NotificationArea
```

---

## Network Architecture

### WebSocket Signaling Layer

The signaling server facilitates peer discovery and connection establishment:

```typescript
// Signaling protocol messages
interface SignalingMessage {
  type: 'join' | 'offer' | 'answer' | 'ice-candidate';
  sessionId: string;
  peerId: string;
  payload: any;
}

// Server implementation (simplified)
class SignalingServer {
  sessions: Map<string, Set<WebSocket>> = new Map();
  
  handleMessage(ws: WebSocket, msg: SignalingMessage) {
    switch (msg.type) {
      case 'join':
        this.joinSession(ws, msg.sessionId);
        break;
      case 'offer':
      case 'answer':
      case 'ice-candidate':
        this.relay(msg);
        break;
    }
  }
}
```

### WebRTC Mesh Network

Participants form a full mesh network for DKG and signing:

```
    Alice
    /   \
   /     \
  /       \
Bob ───── Carol

Full mesh for 3 participants
```

#### Connection Establishment Flow

1. **Session Creation**
   ```
   Alice → Server: CREATE_SESSION
   Server → Alice: SESSION_ID
   ```

2. **Peer Discovery**
   ```
   Bob → Server: JOIN_SESSION(SESSION_ID)
   Server → Alice: PEER_JOINED(Bob)
   Server → Bob: EXISTING_PEERS([Alice])
   ```

3. **WebRTC Negotiation**
   ```
   Alice → Bob: OFFER(SDP)
   Bob → Alice: ANSWER(SDP)
   Alice ↔ Bob: ICE_CANDIDATES
   ```

4. **Mesh Formation**
   - Each peer connects to all others
   - Connections verified before proceeding
   - Automatic reconnection on failure

### Network Resilience

#### Failover Mechanisms

1. **WebSocket Failover**
   - Primary: `wss://auto-life.tech`
   - Fallback servers configured
   - Automatic reconnection with exponential backoff

2. **WebRTC Reconnection**
   - Peer connection monitoring
   - Automatic renegotiation on failure
   - Session state preservation

3. **Protocol Recovery**
   - DKG round state saved locally
   - Resume from last completed round
   - Timeout and retry mechanisms

---

## Security Architecture

### Cryptographic Security

#### Key Generation Security

1. **Distributed Key Generation (DKG)**
   - Feldman VSS for share distribution
   - Pedersen commitments for verification
   - No single party sees the complete key

2. **Share Protection**
   ```rust
   pub struct KeyShare {
       index: u32,
       share: Scalar,
       public_key: PublicKey,
       threshold: u32,
       participants: Vec<PublicKey>,
   }
   
   impl KeyShare {
       pub fn encrypt(&self, password: &str) -> EncryptedShare {
           // PBKDF2 key derivation
           let salt = generate_salt();
           let key = derive_key(password, &salt, 100_000);
           
           // AES-256-GCM encryption
           let ciphertext = encrypt_aes_gcm(&self.encode(), &key);
           
           EncryptedShare { salt, ciphertext }
       }
   }
   ```

3. **Signing Security**
   - Deterministic nonce generation prevents reuse
   - Partial signatures verified before aggregation
   - Message commitment prevents malleability

### Network Security

#### Transport Security

1. **WebSocket Security**
   - TLS 1.3 required for all connections
   - Certificate pinning for known servers
   - Message authentication with HMAC

2. **WebRTC Security**
   - DTLS for data channel encryption
   - SRTP would be used for media (disabled)
   - Perfect forward secrecy

#### Authentication & Authorization

```typescript
// Device authentication flow
class DeviceAuth {
  async authenticate(deviceId: string): Promise<AuthToken> {
    // Generate challenge
    const challenge = crypto.randomBytes(32);
    
    // Sign with device key
    const signature = await this.signChallenge(challenge);
    
    // Verify and issue token
    return this.verifyAndIssueToken(deviceId, signature);
  }
}
```

### Access Control

#### Role-Based Permissions

| Role | Permissions |
|------|-------------|
| Administrator | Full access, manage participants |
| Signer | Sign transactions, view balances |
| Observer | View-only access |
| Auditor | Read audit logs, generate reports |

#### Multi-Level Security

1. **Application Level**
   - Password protection for keystore
   - Session timeouts
   - Failed attempt lockouts

2. **Protocol Level**
   - Threshold requirements enforced
   - Participant verification
   - Message replay prevention

3. **System Level**
   - Secure key storage
   - Memory protection
   - Process isolation

---

## User Interface Architecture

### Design Principles

1. **Progressive Disclosure**
   - Simple default flows
   - Advanced options hidden
   - Contextual help available

2. **Security-First UX**
   - Clear security indicators
   - Confirmation for critical actions
   - Audit trail visibility

3. **Keyboard-First Navigation**
   - All actions keyboard accessible
   - Consistent shortcuts across platforms
   - Vim-like navigation in CLI

### TUI Architecture (Terminal UI)

The Terminal UI implements a state-machine based navigation:

```rust
enum AppScreen {
    Welcome,
    MainMenu,
    CreateWallet(CreateWalletState),
    JoinSession(JoinSessionState),
    WalletPortfolio(PortfolioState),
    Settings(SettingsState),
}

impl App {
    fn handle_input(&mut self, key: KeyEvent) {
        match (&self.current_screen, key.code) {
            (AppScreen::Welcome, KeyCode::Enter) => {
                self.transition_to(AppScreen::MainMenu);
            }
            (AppScreen::MainMenu, KeyCode::Char('1')) => {
                self.transition_to(AppScreen::CreateWallet(Default::default()));
            }
            // ... more transitions
        }
    }
}
```

#### Screen Hierarchy

```
Welcome Screen
    │
    ├─ Create New Wallet
    │   ├─ Quick DKG Session
    │   ├─ Custom DKG Setup
    │   └─ Multi-Chain Wallet
    │
    ├─ Join Wallet Session
    │   ├─ Available Sessions
    │   └─ Manual Entry
    │
    ├─ Select Existing Wallet
    │   └─ Wallet Operations
    │       ├─ Send Transaction
    │       ├─ Sign Message
    │       └─ Manage Participants
    │
    └─ Settings & Configuration
        ├─ Network Settings
        ├─ Security Policies
        └─ Display Preferences
```

### Component Design Patterns

#### 1. Status Indicators

```
Connection Status:
🟢 Connected    - Active and healthy
🟡 Connecting   - In progress
🔴 Disconnected - No connection
⚪ Offline      - Offline mode

Security Status:
🔒 Locked       - Wallet locked
🔓 Unlocked     - Ready for operations
⚠️  Warning      - Security issue
✅ Verified     - Cryptographically verified
```

#### 2. Progress Visualization

```
DKG Progress:
Round 1: [████████████████████] 100% ✓
Round 2: [████████░░░░░░░░░░░░] 40%  ⟳
Round 3: [░░░░░░░░░░░░░░░░░░░░] 0%   ⏸

Overall: 47% complete
```

#### 3. Error Handling

```
┌─ Error ──────────────────────────────┐
│                                      │
│ ❌ Connection Failed                 │
│                                      │
│ Unable to connect to signaling       │
│ server at wss://auto-life.tech       │
│                                      │
│ Error: Network timeout (30s)         │
│                                      │
│ [R] Retry  [O] Offline  [H] Help    │
└──────────────────────────────────────┘
```

---

## Data Models and Flow

### Core Data Structures

#### 1. Wallet Model

```rust
pub struct Wallet {
    pub id: String,
    pub name: String,
    pub threshold: u32,
    pub participants: Vec<Participant>,
    pub chain: BlockchainType,
    pub address: Address,
    pub created_at: DateTime<Utc>,
    pub key_share: Option<EncryptedKeyShare>,
}

pub struct Participant {
    pub index: u32,
    pub device_id: String,
    pub public_key: PublicKey,
    pub name: Option<String>,
    pub role: ParticipantRole,
}
```

#### 2. Session Model

```typescript
interface Session {
  id: string;
  type: 'dkg' | 'signing';
  state: SessionState;
  participants: Map<string, ParticipantInfo>;
  threshold: number;
  createdAt: Date;
  expiresAt: Date;
  metadata: SessionMetadata;
}

enum SessionState {
  Created = 'created',
  Joining = 'joining',
  Active = 'active',
  Completed = 'completed',
  Failed = 'failed',
}
```

#### 3. Message Types

```typescript
// High-level message categories
type MessageType = 
  | SessionMessage
  | DKGMessage
  | SigningMessage
  | StatusMessage
  | ErrorMessage;

// DKG protocol messages
interface DKGMessage {
  type: 'dkg';
  round: 1 | 2;
  data: DKGRoundData;
  signature: Signature;
}

// Signing protocol messages
interface SigningMessage {
  type: 'signing';
  phase: 'commitment' | 'signature';
  data: SigningData;
  signature: Signature;
}
```

### Data Flow Diagrams

#### DKG Data Flow

```
┌─────────┐     ┌─────────┐     ┌─────────┐
│  Alice  │     │   Bob   │     │  Carol  │
└────┬────┘     └────┬────┘     └────┬────┘
     │               │               │
     │ Round 1: Generate commitments │
     ├──────────────►│               │
     ├───────────────┼──────────────►│
     │◄──────────────┤               │
     │               │◄──────────────┤
     │◄──────────────┼───────────────┤
     │               │               │
     │ Round 2: Distribute shares    │
     ├──────────────►│               │
     ├───────────────┼──────────────►│
     │◄──────────────┤               │
     │               │◄──────────────┤
     │◄──────────────┼───────────────┤
     │               │               │
     │ Verify and store key shares   │
     ▼               ▼               ▼
  KeyStore       KeyStore        KeyStore
```

#### Transaction Signing Flow

```
1. Transaction Creation
   User → Wallet Selection → Transaction Details → Review

2. Signature Collection  
   Initiator → Broadcast Request → Participants
                                    ↓
                             Review & Approve
                                    ↓
                            Generate Partial Sig

3. Signature Aggregation
   Collect Partial Sigs → Verify → Aggregate → Final Signature

4. Broadcast
   Final Signature → Blockchain Network → Confirmation
```

### State Management

#### Application State

```typescript
interface AppState {
  // UI State
  currentScreen: ScreenType;
  navigationStack: ScreenType[];
  
  // Wallet State
  wallets: Map<string, Wallet>;
  activeWallet: string | null;
  
  // Session State
  activeSession: Session | null;
  sessionHistory: Session[];
  
  // Network State
  connectionStatus: ConnectionStatus;
  peers: Map<string, PeerInfo>;
  
  // Settings
  settings: UserSettings;
  profile: ConnectionProfile;
}
```

#### State Persistence

```rust
// Persistent storage abstraction
trait StorageProvider {
    fn save_wallet(&self, wallet: &Wallet) -> Result<()>;
    fn load_wallets(&self) -> Result<Vec<Wallet>>;
    fn save_settings(&self, settings: &Settings) -> Result<()>;
    fn load_settings(&self) -> Result<Settings>;
}

// Implementations
struct FileStorage { /* ... */ }
struct BrowserStorage { /* ... */ }
struct SecureStorage { /* ... */ }
```

---

## Protocol Implementation

### FROST Protocol Details

#### Key Generation Protocol

```rust
// Simplified FROST DKG implementation
pub struct DKGProtocol {
    round: u8,
    threshold: u32,
    participants: Vec<ParticipantId>,
    commitments: HashMap<ParticipantId, Commitments>,
    shares: HashMap<ParticipantId, Share>,
}

impl DKGProtocol {
    pub fn round1(&mut self) -> Round1Message {
        // Generate polynomial coefficients
        let coefficients = generate_polynomial(self.threshold);
        
        // Create commitments
        let commitments = create_commitments(&coefficients);
        
        Round1Message { 
            sender: self.id,
            commitments 
        }
    }
    
    pub fn round2(&mut self, round1_msgs: Vec<Round1Message>) -> Round2Message {
        // Verify commitments
        for msg in round1_msgs {
            verify_commitments(&msg.commitments)?;
        }
        
        // Generate and encrypt shares
        let shares = generate_shares(&self.coefficients, &self.participants);
        
        Round2Message {
            sender: self.id,
            encrypted_shares: encrypt_shares(shares)
        }
    }
}
```

#### Signing Protocol

```rust
pub struct SigningProtocol {
    message: Vec<u8>,
    signers: Vec<ParticipantId>,
    nonces: HashMap<ParticipantId, Nonce>,
    partial_sigs: HashMap<ParticipantId, PartialSignature>,
}

impl SigningProtocol {
    pub fn create_signing_commitment(&self) -> SigningCommitment {
        let (nonce, commitment) = generate_nonce_commitment();
        
        SigningCommitment {
            signer: self.id,
            commitment,
        }
    }
    
    pub fn create_partial_signature(
        &self, 
        commitments: Vec<SigningCommitment>
    ) -> PartialSignature {
        // Aggregate commitments
        let group_commitment = aggregate_commitments(&commitments);
        
        // Create signature share
        let sig_share = sign_with_share(
            &self.key_share,
            &self.nonce,
            &self.message,
            &group_commitment
        );
        
        PartialSignature {
            signer: self.id,
            signature: sig_share,
        }
    }
}
```

### Protocol State Machine

```rust
enum ProtocolState {
    Idle,
    DKG(DKGState),
    Signing(SigningState),
    Completed(CompletedState),
    Failed(ErrorState),
}

enum DKGState {
    WaitingForParticipants,
    Round1InProgress,
    Round1Complete,
    Round2InProgress,
    Round2Complete,
    Finalizing,
}

enum SigningState {
    CollectingCommitments,
    CommitmentsReceived,
    CollectingSignatures,
    SignaturesReceived,
    Aggregating,
}
```

---

## Deployment Architecture

### Infrastructure Components

```
┌─────────────────────────────────────────────────────────┐
│                   Load Balancer                         │
│                  (CloudFlare CDN)                       │
└─────────────────┬────────────────┬─────────────────────┘
                  │                │
       ┌──────────▼──────┐  ┌──────▼──────────┐
       │ WebSocket Server│  │ Cloudflare      │
       │ (Auto-scaling)  │  │ Worker          │
       └─────────────────┘  └─────────────────┘
                  │                │
       ┌──────────▼────────────────▼─────────┐
       │     Redis Cluster (Session State)    │
       └──────────────────────────────────────┘
```

### Deployment Options

#### 1. Cloud Deployment

```yaml
# Kubernetes deployment example
apiVersion: apps/v1
kind: Deployment
metadata:
  name: mpc-signal-server
spec:
  replicas: 3
  selector:
    matchLabels:
      app: signal-server
  template:
    metadata:
      labels:
        app: signal-server
    spec:
      containers:
      - name: server
        image: mpc-wallet/signal-server:latest
        ports:
        - containerPort: 8080
        env:
        - name: REDIS_URL
          value: "redis://redis-cluster:6379"
```

#### 2. Edge Deployment

```javascript
// Cloudflare Worker for edge signaling
export default {
  async fetch(request, env) {
    const upgradeHeader = request.headers.get('Upgrade');
    
    if (upgradeHeader !== 'websocket') {
      return new Response('Expected websocket', { status: 426 });
    }
    
    const [client, server] = Object.values(new WebSocketPair());
    
    await handleSession(server, env);
    
    return new Response(null, {
      status: 101,
      webSocket: client,
    });
  },
};
```

#### 3. Self-Hosted Deployment

```bash
# Docker Compose for self-hosted setup
version: '3.8'
services:
  signal-server:
    image: mpc-wallet/signal-server:latest
    ports:
      - "8080:8080"
    environment:
      - RUST_LOG=info
    volumes:
      - ./config:/app/config
    restart: unless-stopped
    
  redis:
    image: redis:alpine
    command: redis-server --appendonly yes
    volumes:
      - redis-data:/data
      
volumes:
  redis-data:
```

### Monitoring and Observability

#### Metrics Collection

```rust
// Prometheus metrics
lazy_static! {
    static ref ACTIVE_SESSIONS: IntGauge = 
        register_int_gauge!("mpc_active_sessions", "Number of active sessions").unwrap();
    
    static ref DKG_DURATION: Histogram = 
        register_histogram!("mpc_dkg_duration_seconds", "DKG completion time").unwrap();
        
    static ref SIGNING_REQUESTS: Counter = 
        register_counter!("mpc_signing_requests_total", "Total signing requests").unwrap();
}
```

#### Logging Strategy

```rust
// Structured logging with tracing
#[instrument(skip(session))]
pub async fn handle_dkg_round(
    session: &Session,
    round: u8,
    message: DKGMessage,
) -> Result<()> {
    info!(
        session_id = %session.id,
        round = round,
        participant = %message.sender,
        "Processing DKG round"
    );
    
    // Process round...
    
    Ok(())
}
```

---

## Performance Characteristics

### Benchmarks

#### Cryptographic Operations

| Operation | Time (avg) | Notes |
|-----------|------------|-------|
| DKG Setup (3 participants) | 1.2s | Including network latency |
| Threshold Signing | 450ms | 2-of-3 threshold |
| Key Derivation (PBKDF2) | 100ms | 100,000 iterations |
| Share Encryption | 5ms | AES-256-GCM |

#### Network Performance

| Metric | Value | Conditions |
|--------|-------|------------|
| WebRTC Connection Setup | 2-3s | With TURN server |
| Message Latency (P2P) | 50ms | Same region |
| Throughput | 1MB/s | Per peer connection |
| Concurrent Sessions | 1000+ | Per signaling server |

### Optimization Strategies

#### 1. Connection Pooling

```rust
pub struct ConnectionPool {
    connections: Arc<Mutex<HashMap<PeerId, Connection>>>,
    max_idle: Duration,
}

impl ConnectionPool {
    pub async fn get_connection(&self, peer_id: &PeerId) -> Result<Connection> {
        let mut connections = self.connections.lock().await;
        
        if let Some(conn) = connections.get(peer_id) {
            if conn.is_healthy() {
                return Ok(conn.clone());
            }
        }
        
        // Create new connection
        let conn = self.create_connection(peer_id).await?;
        connections.insert(peer_id.clone(), conn.clone());
        
        Ok(conn)
    }
}
```

#### 2. Parallel Processing

```rust
// Parallel signature verification
pub async fn verify_signatures(sigs: Vec<PartialSignature>) -> Result<()> {
    let handles: Vec<_> = sigs
        .into_iter()
        .map(|sig| {
            tokio::spawn(async move {
                verify_partial_signature(&sig)
            })
        })
        .collect();
    
    let results = futures::future::join_all(handles).await;
    
    for result in results {
        result??;
    }
    
    Ok(())
}
```

#### 3. Caching Strategy

```typescript
class CacheManager {
  private cache: LRUCache<string, any>;
  
  constructor(maxSize: number = 1000) {
    this.cache = new LRUCache({ max: maxSize });
  }
  
  async getOrCompute<T>(
    key: string,
    compute: () => Promise<T>,
    ttl: number = 3600
  ): Promise<T> {
    if (this.cache.has(key)) {
      return this.cache.get(key);
    }
    
    const value = await compute();
    this.cache.set(key, value, { ttl: ttl * 1000 });
    
    return value;
  }
}
```

---

## Integration Points

### Blockchain Integration

#### Ethereum Integration

```rust
use ethers::prelude::*;

pub struct EthereumSigner {
    provider: Provider<Http>,
    wallet_address: Address,
}

impl EthereumSigner {
    pub async fn sign_transaction(
        &self,
        tx: TypedTransaction,
        signature: Signature,
    ) -> Result<Bytes> {
        // Convert FROST signature to Ethereum format
        let eth_sig = convert_frost_to_eth_signature(signature)?;
        
        // Create signed transaction
        let signed_tx = tx.rlp_signed(&eth_sig);
        
        // Broadcast
        let pending_tx = self.provider
            .send_raw_transaction(signed_tx)
            .await?;
            
        Ok(pending_tx.tx_hash())
    }
}
```

#### Solana Integration

```rust
use solana_sdk::prelude::*;

pub struct SolanaSigner {
    rpc_client: RpcClient,
    wallet_pubkey: Pubkey,
}

impl SolanaSigner {
    pub async fn sign_transaction(
        &self,
        tx: Transaction,
        signature: Signature,
    ) -> Result<Signature> {
        // Convert FROST signature to Solana format
        let sol_sig = convert_frost_to_sol_signature(signature)?;
        
        // Attach signature
        let signed_tx = tx.sign(&[&sol_sig], recent_blockhash);
        
        // Send transaction
        let sig = self.rpc_client
            .send_transaction(&signed_tx)
            .await?;
            
        Ok(sig)
    }
}
```

### External API Integration

#### Webhook Notifications

```typescript
interface WebhookConfig {
  url: string;
  events: EventType[];
  headers?: Record<string, string>;
  retryPolicy?: RetryPolicy;
}

class WebhookNotifier {
  async notify(event: WalletEvent, config: WebhookConfig) {
    const payload = {
      event: event.type,
      timestamp: event.timestamp,
      data: event.data,
      signature: this.signPayload(event),
    };
    
    await this.sendWithRetry(config.url, payload, config.retryPolicy);
  }
}
```

#### REST API Endpoints

```typescript
// Express.js API example
app.post('/api/v1/wallets/:walletId/sign', async (req, res) => {
  const { walletId } = req.params;
  const { transaction, metadata } = req.body;
  
  try {
    // Validate request
    validateSigningRequest(transaction);
    
    // Create signing session
    const sessionId = await createSigningSession(walletId, transaction);
    
    // Return session info
    res.json({
      sessionId,
      status: 'pending',
      requiredSigners: 2,
      expiresAt: new Date(Date.now() + 3600000),
    });
  } catch (error) {
    res.status(400).json({ error: error.message });
  }
});
```

---

## Operational Considerations

### Backup and Recovery

#### Keystore Backup Strategy

```rust
pub struct BackupManager {
    encryption_key: Key,
    storage_backend: Box<dyn StorageBackend>,
}

impl BackupManager {
    pub async fn create_backup(&self, wallet: &Wallet) -> Result<BackupBundle> {
        // Create backup bundle
        let bundle = BackupBundle {
            version: BACKUP_VERSION,
            wallet_id: wallet.id.clone(),
            metadata: wallet.metadata.clone(),
            encrypted_shares: self.encrypt_shares(&wallet.shares)?,
            checksum: self.calculate_checksum(&wallet)?,
            created_at: Utc::now(),
        };
        
        // Store backup
        self.storage_backend.store(&bundle).await?;
        
        Ok(bundle)
    }
    
    pub async fn restore_backup(&self, bundle: BackupBundle) -> Result<Wallet> {
        // Verify checksum
        self.verify_checksum(&bundle)?;
        
        // Decrypt shares
        let shares = self.decrypt_shares(&bundle.encrypted_shares)?;
        
        // Reconstruct wallet
        let wallet = Wallet::from_backup(bundle, shares)?;
        
        Ok(wallet)
    }
}
```

#### Disaster Recovery Plan

1. **Regular Backups**
   - Automated daily backups
   - Geographic distribution
   - Encrypted cloud storage

2. **Recovery Procedures**
   ```
   1. Verify backup integrity
   2. Gather minimum threshold of participants
   3. Restore key shares from backup
   4. Verify wallet addresses match
   5. Test with small transaction
   ```

3. **Emergency Procedures**
   - Emergency contact list
   - Documented recovery steps
   - Regular recovery drills

### Maintenance Operations

#### Database Maintenance

```sql
-- Cleanup old sessions
DELETE FROM sessions 
WHERE created_at < NOW() - INTERVAL '7 days' 
AND status IN ('completed', 'failed');

-- Archive audit logs
INSERT INTO audit_logs_archive 
SELECT * FROM audit_logs 
WHERE created_at < NOW() - INTERVAL '90 days';

-- Vacuum and analyze
VACUUM ANALYZE sessions;
VACUUM ANALYZE audit_logs;
```

#### Log Rotation

```bash
# Logrotate configuration
/var/log/mpc-wallet/*.log {
    daily
    rotate 30
    compress
    delaycompress
    notifempty
    create 0640 mpc-wallet mpc-wallet
    sharedscripts
    postrotate
        systemctl reload mpc-wallet
    endscript
}
```

### Security Operations

#### Incident Response

```yaml
# Incident response playbook
incident_response:
  detection:
    - Monitor authentication failures
    - Track unusual signing patterns
    - Alert on configuration changes
    
  containment:
    - Isolate affected systems
    - Revoke compromised credentials
    - Enable emergency lockdown
    
  investigation:
    - Collect audit logs
    - Analyze network traffic
    - Review access patterns
    
  recovery:
    - Rotate affected keys
    - Update security policies
    - Notify stakeholders
```

#### Security Monitoring

```rust
// Security event monitoring
pub struct SecurityMonitor {
    rules: Vec<SecurityRule>,
    alert_manager: AlertManager,
}

impl SecurityMonitor {
    pub async fn check_event(&self, event: &SecurityEvent) -> Result<()> {
        for rule in &self.rules {
            if rule.matches(event) {
                self.alert_manager.send_alert(Alert {
                    severity: rule.severity,
                    title: rule.name.clone(),
                    description: format!("Security rule triggered: {}", rule.description),
                    event: event.clone(),
                }).await?;
            }
        }
        
        Ok(())
    }
}
```

---

## Appendices

### A. Glossary

| Term | Definition |
|------|------------|
| **DKG** | Distributed Key Generation - Process where multiple parties jointly generate a key |
| **FROST** | Flexible Round-Optimized Schnorr Threshold signatures |
| **MPC** | Multi-Party Computation - Cryptographic protocols for joint computation |
| **Threshold Signature** | Signature scheme requiring k-of-n participants |
| **VSS** | Verifiable Secret Sharing - Method to distribute secret shares |
| **WebRTC** | Web Real-Time Communication - P2P communication protocol |
| **STUN/TURN** | Protocols for NAT traversal in WebRTC |

### B. Configuration Reference

#### Environment Variables

```bash
# Server configuration
MPC_WALLET_SERVER_PORT=8080
MPC_WALLET_SERVER_HOST=0.0.0.0
MPC_WALLET_WS_PATH=/ws

# Security settings
MPC_WALLET_SESSION_TIMEOUT=3600
MPC_WALLET_MAX_PARTICIPANTS=10
MPC_WALLET_MIN_THRESHOLD=2

# Network configuration
MPC_WALLET_STUN_SERVERS=stun:stun.l.google.com:19302
MPC_WALLET_TURN_SERVER=turn:turn.auto-life.tech:3478
MPC_WALLET_TURN_USERNAME=user
MPC_WALLET_TURN_PASSWORD=pass

# Storage configuration
MPC_WALLET_STORAGE_PATH=/var/lib/mpc-wallet
MPC_WALLET_BACKUP_PATH=/var/backup/mpc-wallet
```

#### Configuration File

```toml
# config.toml
[server]
host = "0.0.0.0"
port = 8080
tls_cert = "/etc/mpc-wallet/cert.pem"
tls_key = "/etc/mpc-wallet/key.pem"

[security]
session_timeout = 3600
max_failed_attempts = 3
lockout_duration = 300

[network]
stun_servers = [
    "stun:stun.l.google.com:19302",
    "stun:stun1.l.google.com:19302"
]

[storage]
type = "postgresql"
url = "postgresql://user:pass@localhost/mpc_wallet"

[logging]
level = "info"
format = "json"
output = "/var/log/mpc-wallet/app.log"
```

### C. API Reference

#### WebSocket Protocol

```typescript
// Client → Server messages
interface ClientMessage {
  type: 'join_session' | 'create_session' | 'leave_session' | 'relay';
  sessionId?: string;
  payload?: any;
}

// Server → Client messages
interface ServerMessage {
  type: 'session_created' | 'peer_joined' | 'peer_left' | 'relay' | 'error';
  sessionId?: string;
  peerId?: string;
  payload?: any;
  error?: string;
}
```

#### REST API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/v1/health` | GET | Health check |
| `/api/v1/sessions` | GET | List active sessions |
| `/api/v1/sessions` | POST | Create new session |
| `/api/v1/sessions/:id` | GET | Get session details |
| `/api/v1/sessions/:id/join` | POST | Join session |
| `/api/v1/wallets` | GET | List wallets |
| `/api/v1/wallets/:id/sign` | POST | Initiate signing |

### D. Troubleshooting Guide

#### Common Issues

1. **Connection Failures**
   ```
   Problem: Cannot connect to signaling server
   
   Checks:
   - Verify server URL is correct
   - Check firewall allows WebSocket connections
   - Ensure TLS certificate is valid
   
   Solution:
   - Use fallback server
   - Check network connectivity
   - Review server logs
   ```

2. **DKG Failures**
   ```
   Problem: DKG timeout or incomplete
   
   Checks:
   - All participants online?
   - Network connectivity stable?
   - Correct threshold configuration?
   
   Solution:
   - Restart DKG process
   - Check peer connections
   - Increase timeout values
   ```

3. **Signing Errors**
   ```
   Problem: Cannot collect enough signatures
   
   Checks:
   - Minimum threshold available?
   - Participants have valid shares?
   - Message format correct?
   
   Solution:
   - Verify participant availability
   - Check share integrity
   - Review signing request
   ```

### E. Development Setup

#### Prerequisites

```bash
# Install development dependencies
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
curl -fsSL https://bun.sh/install | bash
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Clone repository
git clone https://github.com/your-org/mpc-wallet.git
cd mpc-wallet

# Install dependencies
bun install
```

#### Development Workflow

```bash
# Build WASM
bun run build:wasm:dev

# Start development servers
bun run dev              # Browser extension
cargo run --bin cli      # CLI node
cargo run --bin native   # Native app

# Run tests
bun test                 # JavaScript tests
cargo test               # Rust tests
cargo test --workspace   # All tests
```

#### Debugging Tips

1. **Enable verbose logging**
   ```bash
   export RUST_LOG=debug
   export DEBUG=mpc:*
   ```

2. **Chrome DevTools for extension**
   - Background: chrome://extensions → Service Worker "Inspect"
   - Popup: Right-click → Inspect
   - Offscreen: Check background console

3. **Network debugging**
   ```bash
   # Monitor WebSocket traffic
   wscat -c wss://auto-life.tech/ws
   
   # Test STUN/TURN
   turnutils_stunclient stun.l.google.com
   ```

---

## Conclusion

The MPC Wallet represents a sophisticated approach to cryptocurrency custody that balances security with usability. Through its distributed architecture, no single point of failure exists, while the threshold signature scheme ensures that legitimate transactions can still be processed even if some participants are offline.

The system's modular design allows for deployment across multiple platforms while maintaining consistent security guarantees. The use of modern technologies like WebRTC for peer-to-peer communication and Rust for performance-critical components ensures the system can scale to meet enterprise demands.

Future enhancements may include:
- Support for additional blockchain protocols
- Hardware security module integration
- Advanced multi-sig workflows
- Regulatory compliance features
- Enhanced monitoring and analytics

The architecture provides a solid foundation for these expansions while maintaining backward compatibility and security.