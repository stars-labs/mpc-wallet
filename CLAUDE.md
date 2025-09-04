# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

FROST MPC TUI Wallet is a professional Terminal User Interface based monorepo that provides enterprise-grade Multi-Party Computation for blockchain wallets. Similar to BitGo's architecture, it implements the FROST (Flexible Round-Optimized Schnorr Threshold) signature scheme for distributed key generation and signing operations where no single party holds the complete private key. The system supports both secp256k1 (Ethereum, Bitcoin) and ed25519 (Solana) curves across multiple blockchains through an intuitive menu-driven interface.

### 🔑 Key Feature: Dual-Mode Operation

The wallet uniquely supports both **online** and **offline** operational modes:

- **🌐 Online/Hot-Wallet Mode**: Uses WebSocket signaling and WebRTC mesh networking for real-time coordination. Ideal for daily operations, trading, and convenient multi-party interactions with full network connectivity.

- **🔒 Offline/Cold-Wallet Mode**: Operates in complete air-gap using SD cards for data exchange between participants. Perfect for high-security environments, cold storage, and regulatory compliance where network isolation is required.

This dual-mode architecture allows organizations to choose their security posture based on specific requirements - from convenient online operations to maximum-security air-gapped setups, all within the same unified system.

### Monorepo Structure

```
apps/
├── browser-extension/    # Chrome/Firefox extension with UI
├── tui-node/            # Terminal UI MPC node (library & binary) - BitGo-like interface
├── native-node/         # Native desktop app with Slint UI (shares TUI node core)
└── signal-server/       # WebRTC signaling infrastructure
    ├── server/          # Standard WebSocket server
    └── cloudflare-worker/ # Edge deployment

packages/@mpc-wallet/
├── frost-core/         # Shared FROST Rust library (NEW)
├── core-wasm/          # FROST WebAssembly bindings (thin wrapper)
└── types/              # Shared TypeScript types
```

## Architecture

The extension follows Chrome Extension Manifest V3 architecture with four main contexts:

1. **Background Service Worker** (`apps/browser-extension/src/entrypoints/background/`): Central message router managing WebSocket connections to a signaling server and coordinating communication between components.
   
2. **Popup Page** (`apps/browser-extension/src/entrypoints/popup/`): User interface for wallet operations built with Svelte.
   
3. **Offscreen Document** (`apps/browser-extension/src/entrypoints/offscreen/`): Handles WebRTC connections for peer-to-peer communication and cryptographic operations using Rust/WebAssembly.
   
4. **Content Script** (`apps/browser-extension/src/entrypoints/content/`): Injects wallet provider API into web pages.

### Key Components

- **WebRTC System** (`apps/browser-extension/src/entrypoints/offscreen/webrtc.ts`): Manages peer-to-peer connections and coordinates the MPC protocol.
  
- **Shared FROST Core** (`packages/@mpc-wallet/frost-core/`): Shared Rust library implementing core FROST cryptographic operations, keystore management, and encryption. Used by WASM, CLI, and native applications to eliminate code duplication.

- **Rust/WebAssembly Core** (`packages/@mpc-wallet/core-wasm/src/lib.rs`): Thin wrapper around frost-core providing WASM bindings for browser usage.

- **TUI Node Library** (`apps/tui-node/src/lib.rs`): The TUI node exposes its core functionality as a Rust library, providing a professional terminal interface like BitGo. Enables the native desktop app to reuse all the WebSocket, WebRTC, DKG, and signing logic without duplication.

- **Native Desktop App** (`apps/native-node/`): Desktop application with Slint UI that uses the TUI node as a library. Features an adapter pattern to bridge UI events to the TUI's command system.
  
- **Shared Types** (`packages/@mpc-wallet/types/`): Centralized TypeScript type definitions shared across all applications. Includes messages, state, session, DKG, keystore, and network types.
  
- **Message System** (`packages/@mpc-wallet/types/src/messages.ts`): Strongly-typed messages for communication between extension components.

- **Keystore Service** (`apps/browser-extension/src/services/keystoreService.ts`): Manages secure storage of FROST key shares with encryption and backup capabilities.

- **DKG Manager** (`apps/browser-extension/src/entrypoints/offscreen/dkgManager.ts`): Handles the complete Distributed Key Generation lifecycle across multiple rounds.

## Build and Development Commands

### Prerequisites

```bash
# Install Bun runtime
curl -fsSL https://bun.sh/install | bash

# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

### Development

```bash
# Install dependencies
bun install

# Build WASM (required before running dev)
bun run build:wasm          # Production WASM build
bun run build:wasm:dev      # Development WASM build (faster, larger)

# Start development server with hot reloading
bun run dev                 # Auto-rebuilds WASM on changes

# Run development for specific browsers
bun run dev:firefox
bun run dev:edge

# Type checking
bun run check               # Svelte type checking

# Clean build artifacts
bun run clean              # Remove dist and .wxt directories
```

### Testing

```bash
# Run all tests with setup preload
bun test

# Run specific test categories
bun run test:unit          # Unit tests only (services, components, config)
bun run test:integration   # Integration tests only
bun run test:services      # Service layer tests
bun run test:webrtc        # WebRTC tests (simple)
bun run test:webrtc:all    # All WebRTC tests
bun run test:dkg           # DKG functionality tests

# Run with options
bun test --watch           # Watch mode
bun test --coverage        # Generate coverage reports

# Test keystore functionality
bun test tests/keystore-import.test.ts         # Keystore import/export
bun test tests/wasm-keystore-import.test.ts    # WASM keystore functions
bun test tests/cli-keystore-decryption.test.ts # CLI format compatibility

# Run a single test file
bun test path/to/test.ts

# Run test scripts
./scripts/test/run-all-tests.sh   # Categorized test suites
./scripts/test/run-tests.sh       # Exclude import issues
./scripts/test-dkg-ui.sh          # Validate DKG UI
```

### Production Build

```bash
# Build for production
bun run build

# Build for specific browsers
bun run build:firefox
bun run build:edge

# Create extension ZIP
bun run zip

# Utility scripts
./scripts/build/fix-all-syntax-errors.sh  # Fix syntax errors
./scripts/build/remove-debug-logs.sh      # Clean debug logs
```

### Performance and Benchmarking

```bash
# Performance monitoring
bun scripts/performance.ts   # Build performance tracking
bun scripts/benchmark.ts     # Runtime benchmarking
```

## Development Workflow

### Browser Extension
1. Start the development server: `bun run dev`
2. Load the extension into Chrome/Edge from the `dist/` directory
3. Monitor logs using the browser's developer console:
   - Background: chrome://extensions → Service Worker "Inspect"
   - Popup: Right-click popup → Inspect
   - Offscreen: Check background console for offscreen logs
4. For changes to the Rust code, the dev server auto-rebuilds WASM

### Native Desktop Application
1. Build and run: `cd apps/native-node && cargo run`
2. Build with logging: `cd apps/native-node && RUST_LOG=info cargo run --bin mpc-wallet-native`
3. The native app shares the TUI node's core functionality via library imports
4. UI is built with Slint framework for native performance
5. Uses the same WebSocket/WebRTC infrastructure as TUI and browser extension
6. Keystore files are compatible between all three applications

### TUI Node Application (Professional Terminal Interface)

The TUI node provides a BitGo-like professional terminal interface for MPC operations with seamless dual-mode support:

1. **Running the Application:**
   ```bash
   # Terminal UI with menu-driven interface (online mode by default)
   cd apps/tui-node && cargo run --bin mpc-wallet-tui
   
   # Start in offline mode (air-gapped)
   cargo run --bin mpc-wallet-tui -- --offline
   
   # With logging (logs to file when TUI is active)
   RUST_LOG=info cargo run --bin mpc-wallet-tui
   
   # With custom device ID
   cargo run --bin mpc-wallet-tui -- --device-id alice
   
   # Headless mode (no TUI)
   cargo run --bin mpc-wallet-tui -- --headless
   ```

2. **Terminal UI Features:**
   - **Menu-Driven Interface**: No commands to memorize - navigate with arrow keys
   - **Visual Workflows**: Progress bars and status indicators for all operations
   - **Dual-Mode Support**: Seamlessly switch between online (WebRTC) and offline (SD card) operations
   - **Online Features**: WebSocket signaling, WebRTC mesh, real-time session discovery
   - **Offline Features**: Air-gap mode, SD card import/export, manual coordination
   - **Session Discovery**: Automatic discovery in online mode, manual import in offline mode
   - **Real-time Updates**: Live status online, progress tracking offline
   - **Professional Grade**: Audit trails, compliance features, enterprise security

3. **Architecture Components:**
   - `app_runner.rs`: Core orchestrator using `&mut self` pattern
   - `ui/tui.rs`: Full Ratatui-based terminal interface with windows and menus
   - `ui/provider.rs`: UIProvider trait for UI abstraction
   - `handlers/`: Modular command handlers for different operations
   - `protocol/`: FROST implementation for DKG and signing
   - `keystore/`: Secure encrypted storage for key shares

4. **Testing Infrastructure:**
   - `TestRunner` helper class for complex test scenarios
   - MockUIProvider for testing UI interactions
   - Comprehensive test suites in `tests/` directory
   - Support for deterministic testing with fixed seeds

5. **Navigation & Usage:**
   ```
   Arrow Keys (↑↓): Navigate menus
   Enter: Select option
   Esc: Go back/Exit
   Tab: Next element
   ?: Context help
   
   Main Menu Options:
   - Create New Wallet: Start DKG with visual progress
   - Join Session: Discover and join available sessions
   - Sign Transaction: Threshold signing with participant status
   - Offline Mode: Air-gapped operations with SD card
   ```

## Message Flow Architecture

```
Popup → Background → Offscreen → WebRTC Peers
  ↑         ↓           ↓
  └─────────┴───────────┴──── WebSocket Server
```

All messages are routed through the background service worker, which acts as the central coordinator. The message types are defined in `src/types/messages.ts` and follow a strict pattern-based routing system.

## Test Organization

```
tests/
├── components/          # UI component tests
├── config/             # Configuration tests
├── entrypoints/        # Extension-specific tests
│   ├── background/     # Service worker tests
│   └── offscreen/      # WebRTC and FROST tests
├── integration/        # End-to-end integration tests
├── services/           # Service layer tests
├── __mocks__/          # Test mocks and stubs
└── legacy/             # Legacy test files (reference only)
```

Test coverage is automatically generated in `coverage/index.html` when running with `--coverage`.

## Configuration Files

- `bunfig.toml` - Bun test and coverage settings
- `wxt.config.ts` - Extension manifest and build configuration
- `tsconfig.json` - TypeScript configuration
- `tailwind.config.js` - Styling configuration with dark mode support
- `flake.nix` - Nix dependency management with graphics libraries for native development
- `Cargo.toml` (workspace) - Rust workspace configuration with shared dependencies
- `build.rs` files - Custom build scripts (Slint UI compilation for native apps)

## Native Application Development

### Slint UI Framework
- Native desktop app uses Slint for cross-platform GUI
- UI files are in `apps/native-node/ui/*.slint` format
- Build process: `build.rs` compiles Slint files into Rust code
- Reactive UI patterns with global state management (`AppState`)
- Thread-safe UI updates using `slint::invoke_from_event_loop`

### TUI-Native Code Sharing Pattern
- TUI node exposes `AppRunner` struct for shared business logic
- Native app implements `UIProvider` trait to handle UI updates
- Eliminates code duplication between terminal UI and native GUI
- Both apps share WebSocket, WebRTC, DKG, and signing implementations

## WebSocket Server

- Default: `wss://auto-life.tech`
- Local development server available in `webrtc-signal-server/`
- Cloudflare Worker implementation in `webrtc-signal-server-cloudflare-worker/`

## Multi-Chain Configuration

Chain support is configured in `src/config/chains.ts`. The extension provides a unified wallet interface across:
- Ethereum (secp256k1 curve)
- Solana (ed25519 curve)

## Keystore Import/Export

The extension supports importing and exporting FROST keystore data for interoperability with CLI nodes:

### Import Keystore from TUI
- Use the "Import Wallet" option in the TUI main menu
- Supports both encrypted (.dat) and unencrypted (.json) keystore files
- Automatically converts between TUI and extension formats
- Maintains participant index mapping and session metadata

### Export Keystore for Backup
- Use the "Export Wallet" option after DKG completion
- Exports in encrypted format for security
- Compatible across TUI, browser extension, and native app
- Supports both secp256k1 (Ethereum) and ed25519 (Solana) curves

### Format Compatibility
- **TUI Format**: Uses PBKDF2-SHA256 with AES-256-GCM encryption
- **Extension Format**: Uses PBKDF2-SHA256 with AES-256-GCM for browser compatibility
- **Automatic Conversion**: WASM handles format detection and bidirectional conversion
- **Participant Indexing**: Maintains consistent 1-based indexing across platforms

## Troubleshooting

### Common Issues

- **WXT Dev Server Issues**: Sometimes the WXT dev server doesn't properly reload after changes. Try stopping and restarting the server.
  
- **Offscreen Document Problems**: If functionality depending on the offscreen document isn't working, check browser console for errors and ensure the document is loaded. Use the "Create Offscreen" button in the popup for debugging.
  
- **WebSocket Connection Errors**: These can occur if the signaling server (`wss://auto-life.tech`) is not reachable. Check connection status in the background service worker logs.

- **"Receiving end does not exist" errors**: Usually indicates the offscreen document hasn't been created yet or has crashed.

- **Keystore Import Issues**: Ensure the keystore file format is correct. Check logs for specific parsing errors. Verify that the TUI and extension are using compatible FROST versions.

- **Signing Failures**: If signing fails after importing a keystore, verify that all participants are using the same key material by comparing exported keystores bit-by-bit.

- **TUI Display Issues**: Ensure terminal supports UTF-8 and has proper TTY. The TUI requires an interactive terminal environment.

### Debugging Tips

- Use the browser's developer tools to debug different extension contexts
- For WebAssembly debugging, use `console.log` calls from the JavaScript side
- Enable verbose logging by checking console output in all contexts
- For keystore debugging, export keystores from both TUI and extension to compare data structures
- Use the test suite to validate keystore functionality: `bun test tests/keystore-*.test.ts`
- Check test coverage reports in `coverage/index.html`
- For TUI debugging, logs are written to file when the interface is active to prevent display corruption

### Native App Debugging
- Use `RUST_LOG=info` or `RUST_LOG=debug` for detailed logging
- Slint UI debugging: Check console for thread synchronization issues
- UIProvider pattern: Verify weak window references are properly upgraded
- Connection status issues: Ensure UI updates use correct window instance (not new windows)

## Technology Stack

- **Runtime**: Bun (fast JavaScript runtime) for web, Rust 2024 edition for native
- **Build System**: Bun + wasm-pack + WXT for web, Cargo for native
- **UI Frameworks**: 
  - Browser Extension: Svelte 5 with TailwindCSS
  - Native Desktop: Slint UI framework (Rust-native GUI)
  - TUI Node: Ratatui for professional terminal interface
- **Extension Framework**: WXT (Web Extension Tools)
- **Cryptography**: FROST threshold signatures (implemented in Rust)
- **P2P Communication**: WebRTC with WebSocket signaling
- **Blockchain Libraries**: viem (Ethereum interactions), ethers-rs, solana-sdk
- **Storage**: Browser extension storage API with AES-256-GCM encryption
- **Key Derivation**: PBKDF2-SHA256 (extension), Argon2id compatibility (CLI import)
- **Testing**: Bun test runner with comprehensive test coverage
- **Development Environment**: NixOS with Nix flake for dependencies
- **Code Sharing**: TUI node exposes lib.rs for reuse in native desktop app

## MPC Protocol Workflow

**IMPORTANT**: The TUI now implements **real FROST DKG** using the exact same cryptographic logic as the `dkg.rs` example. The previous insecure implementation that derived group keys from session IDs has been completely removed and replaced with proper FROST threshold cryptography.

The FROST MPC TUI Wallet supports two distinct operational modes to accommodate different security requirements:

### 🌐 Online/Hot-Wallet Mode

For regular operations with network connectivity, providing real-time coordination and convenience.

#### Features
- **WebSocket Signaling**: Uses WebSocket server (default: `wss://auto-life.tech`) for real-time participant coordination
- **WebRTC Mesh Network**: Direct peer-to-peer connections between participants for secure communication
- **Automatic Discovery**: Real-time session discovery and participant synchronization
- **Live Status Updates**: Visual progress indicators and participant status in the TUI
- **Instant Operations**: Immediate key generation and transaction signing

#### Online DKG Process
1. **Session Creation**: Coordinator creates session with threshold parameters via WebSocket
2. **Participant Discovery**: Nodes automatically discover each other through signaling server
3. **WebRTC Mesh Formation**: Secure P2P connections established between all participants
4. **FROST DKG Rounds**: Real-time execution of multi-round protocol
   - Round 1: Commitment generation and broadcast
   - Round 2: Share distribution and verification
5. **Address Generation**: Immediate derivation of wallet addresses

#### Online Signing Process
1. **Transaction Proposal**: Initiator broadcasts signing request via WebRTC
2. **Real-time Coordination**: Participants receive and approve requests instantly
3. **FROST Signing Rounds**: Multi-round signature generation over WebRTC
4. **Signature Aggregation**: Automatic collection and aggregation
5. **Broadcast**: Direct submission to blockchain

### 🔒 Offline/Cold-Wallet Mode

For maximum security in air-gapped environments, suitable for high-value assets and regulatory compliance.

#### Features
- **True Air-Gap**: No network connectivity required - all network interfaces disabled
- **SD Card Exchange**: Uses removable media for secure data transfer between machines
- **Manual Coordination**: Participants physically exchange data via trusted channels
- **Verifiable Security**: Each step can be independently verified offline
- **Compliance Ready**: Meets requirements for cold storage and regulatory frameworks

#### Offline DKG Process
1. **Offline Mode Activation**: Each participant disables all network interfaces
2. **Parameter Distribution**: Coordinator creates DKG package on SD card
3. **Commitment Round**: 
   - Each participant generates commitments offline
   - Exports to SD card for physical delivery to coordinator
4. **Share Distribution**:
   - Coordinator aggregates and redistributes commitments
   - Participants generate and exchange encrypted shares via SD card
5. **Verification**: Each participant independently verifies all shares offline

#### Offline Signing Process
1. **Request Creation**: Coordinator prepares signing request on SD card
2. **Commitment Collection**: Signers generate commitments offline, transfer via SD card
3. **Package Distribution**: Coordinator creates signing package with aggregated commitments
4. **Share Generation**: Signers produce signature shares offline
5. **Final Assembly**: Coordinator collects shares via SD card and assembles final signature

### Choosing the Right Mode

| Consideration | Online Mode | Offline Mode |
|--------------|-------------|--------------|
| **Security Level** | High - TLS/DTLS encryption | Maximum - Air-gapped |
| **Convenience** | High - Real-time operations | Lower - Manual coordination |
| **Speed** | Instant - Network communication | Slower - Physical media exchange |
| **Use Cases** | Daily operations, trading | Cold storage, treasury management |
| **Compliance** | Standard security practices | Regulatory/audit requirements |
| **Infrastructure** | Internet connection required | Air-gapped machines + SD cards |

### Hybrid Approach

For organizations requiring both security and operational flexibility:
- Use **Offline Mode** for key generation (DKG) to ensure maximum security
- Switch to **Online Mode** for day-to-day signing operations
- Maintain separate hot and cold wallets with different thresholds
- Implement time-based or value-based rules for mode selection

### Security Trade-offs

**Online Mode Security:**
- ✅ TLS 1.3 encryption for WebSocket
- ✅ DTLS 1.3 for WebRTC connections
- ✅ Authenticated signaling server
- ⚠️ Requires trust in network infrastructure
- ⚠️ Vulnerable to network-level attacks

**Offline Mode Security:**
- ✅ Complete air-gap protection
- ✅ No network attack surface
- ✅ Physical security controls
- ⚠️ Requires secure physical channels
- ⚠️ Slower emergency response

Both modes implement the same cryptographic security (FROST protocol) and share the same secure keystore format, allowing seamless transitions between modes based on operational requirements.

## Code Quality Standards

### Compilation Warnings Policy

**IMPORTANT**: All code must compile without warnings. Warnings accumulate over time and hide real issues.

When working with the codebase:
1. **Fix all warnings immediately** after making changes
2. **Use proper prefixes** for intentionally unused variables (prefix with `_`)
3. **Run `cargo build` regularly** to check for new warnings
4. **Use `cargo fix`** for automated fixes where appropriate
5. **Never ignore warnings** - they often indicate bugs or poor practices

Common warning fixes:
- Unused variables: Prefix with underscore `_variable_name`
- Unused imports: Remove them
- Dead code: Remove or mark with `#[allow(dead_code)]` if intentional
- Mutable variables that don't need to be: Remove `mut`

## AI Development Cycle

The project uses a memory bank system for AI assistance. When working with AI tools:

1. **Session Initialization**: Review existing knowledge base and project context
2. **Context Preservation**: Capture architectural decisions and implementation progress
3. **Cross-Session Continuity**: Document outcomes and update task complexity
4. **Project Lifecycle**: Use structured project phases and requirement tracking

Memory categories:
- `architecture` - Core system design decisions
- `implementation` - Technical details and code insights
- `debugging` - Problem resolution patterns
- `performance` - Optimization discoveries
- `integration` - Cross-component interactions
- `testing` - Test strategies and validation

Working directory: `/home/freeman.xiong/Documents/github/hecoinfo/mpc-wallet`

## TUI Documentation

Comprehensive documentation for the TUI wallet is available in `apps/tui-node/docs/`:

- **[User Guide](apps/tui-node/docs/USER_GUIDE.md)**: Complete guide with visual examples
- **[Architecture](apps/tui-node/docs/ARCHITECTURE.md)**: Technical design and components
- **[DKG Flows](apps/tui-node/docs/DKG_FLOWS.md)**: Online and offline key generation procedures
- **[Security Model](apps/tui-node/docs/SECURITY.md)**: Threat model and security measures

## Recent Architectural Changes (2025)

### TUI Node Transformation

The former CLI node has been transformed into a professional Terminal User Interface (TUI) similar to BitGo:

1. **Professional Terminal Interface**: 
   - Menu-driven navigation replacing command-line interface
   - Visual progress indicators and real-time status updates
   - Arrow key navigation with no commands to memorize
   - Context-sensitive help system

2. **Enterprise Features**:
   - **Online DKG**: WebRTC-based distributed key generation with visual mesh formation
   - **Offline DKG**: Air-gapped operations using SD card for high-security environments
   - **Session Discovery**: Automatic discovery of available DKG and signing sessions
   - **Multi-Wallet Management**: Visual wallet list with balance and status tracking
   - **Audit Trails**: Comprehensive logging for compliance

3. **Improved Architecture**:
   - `AppRunner<C>` orchestrator pattern with `&mut self` for clean ownership
   - `UIProvider` trait abstraction for different UI backends
   - `TuiProvider` implementation using Ratatui for rich terminal UI
   - Event-driven command system via `InternalCommand<C>` enum
   - Modular handlers for DKG, signing, and session management

4. **User Experience Enhancements**:
   - No command memorization required
   - Visual feedback for all operations
   - Progress bars for multi-round protocols
   - Real-time participant status updates
   - Intuitive error messages and recovery options

5. **Security & Compliance**:
   - Support for both online and offline modes
   - Encrypted keystore with password protection
   - Session-based access control
   - Complete audit logging
   - Compatible with enterprise security requirements

This transformation positions the wallet as a professional-grade MPC solution comparable to BitGo, making threshold signatures accessible to users without command-line expertise.

## Documentation Structure

The project maintains comprehensive documentation organized in a professional, hierarchical structure:

### Root Documentation
```
/
├── README.md                                  # Main project overview and quick start
├── CONTRIBUTING.md                            # Comprehensive contribution guidelines
├── MPC_WALLET_TECHNICAL_DOCUMENTATION.md     # Complete 1400+ line technical reference
└── CLAUDE.md                                  # This file - AI assistant guidance
```

### Core Documentation Hub (`/docs/`)
```
docs/
├── README.md                    # Documentation navigation hub
├── architecture/               # System design and architectural decisions
│   ├── README.md              # Architecture overview
│   ├── system-design.md       # Core system architecture
│   ├── cryptography.md        # FROST implementation details
│   └── data-flow.md           # Message and data flow patterns
├── security/                   # Security model and best practices
│   ├── README.md              # Security overview
│   ├── threat-model.md        # Threat analysis
│   ├── key-management.md      # Key storage and protection
│   └── audit-compliance.md    # Compliance requirements
├── api/                        # API documentation
│   ├── README.md              # API overview
│   ├── frost-protocol.md      # FROST protocol APIs
│   ├── application-apis.md    # Application interfaces
│   └── message-formats.md     # Protocol message specifications
├── development/                # Development guides
│   ├── README.md              # Development overview
│   ├── setup.md               # Environment setup
│   ├── building.md            # Build instructions
│   └── debugging.md           # Debugging guide
├── deployment/                 # Production deployment
│   ├── README.md              # Deployment overview
│   ├── infrastructure.md      # Infrastructure requirements
│   ├── monitoring.md          # Monitoring and observability
│   └── operations.md          # Operational procedures
├── testing/                    # Testing documentation
│   ├── README.md              # Testing overview
│   ├── unit-testing.md        # Unit test guide
│   ├── integration-testing.md # Integration tests
│   └── e2e-testing.md         # End-to-end testing
├── fixes/                      # Bug fixes and solutions
│   └── README.md              # Fix documentation
└── archive/                    # Historical/obsolete docs
    ├── README.md              # Archive notice
    ├── legacy/                # Old documentation
    ├── migration/             # Migration guides
    └── session-fixes/         # Historical fixes
```

### Application-Specific Documentation

#### Browser Extension (`apps/browser-extension/docs/`)
```
apps/browser-extension/docs/
├── README.md                   # Extension overview and setup
├── architecture/              # Extension architecture
├── api/                       # Extension APIs
├── guides/                    # User guides
└── ui/                        # UI documentation
    ├── README.md              # UI overview
    └── DKG_ADDRESS_UI_IMPLEMENTATION.md
```

#### TUI Node (`apps/tui-node/docs/`)
```
apps/tui-node/docs/
├── README.md                   # TUI overview
├── architecture/              # TUI architecture
│   ├── README.md
│   ├── ARCHITECTURE.md        # System design
│   ├── DKG_FLOWS.md          # DKG procedures
│   ├── SECURITY.md           # Security model
│   └── keystore_*.md         # Keystore docs
├── guides/                    # User guides
│   ├── README.md
│   ├── USER_GUIDE.md         # Complete user guide
│   ├── keystore_sessions_user_guide.md
│   └── offline-mode.md       # Offline operations
├── protocol/                  # Protocol specifications
│   ├── 01_webrtc_signaling.md
│   └── 02_keystore_sessions.md
├── ui/                        # UI wireframes and specs
│   ├── README.md
│   ├── NAVIGATION_FLOW.md
│   ├── SCREEN_SPECIFICATIONS.md
│   └── *_WIREFRAMES.md       # Various UI wireframes
├── api/                       # API documentation
├── DEPLOYMENT_GUIDE.md        # Deployment guide
└── MPC_WALLET_TUI_ARCHITECTURE.md
```

#### Native Node (`apps/native-node/docs/`)
```
apps/native-node/docs/
└── README.md                   # Native app documentation
```

#### Signal Server (`apps/signal-server/docs/`)
```
apps/signal-server/docs/
├── README.md                   # Signal server overview
└── deployment/                # Deployment guides
    └── cloudflare-deployment.md
```

### Documentation Standards

1. **Consistent Format**: All documentation follows markdown best practices with clear headers, code blocks, and navigation
2. **Professional Tone**: Enterprise-grade documentation suitable for serious tech companies
3. **Comprehensive Coverage**: Every component, API, and workflow is documented
4. **Visual Aids**: Wireframes, diagrams, and examples where appropriate
5. **Maintenance**: Obsolete docs archived but preserved for reference
6. **Navigation**: Clear README files at each level with proper linking
7. **Technical Depth**: Detailed technical documentation for developers and architects

### Key Documentation Files

- **Main Technical Reference**: `/MPC_WALLET_TECHNICAL_DOCUMENTATION.md` - Comprehensive 1400+ line document covering all aspects
- **Contributing Guide**: `/CONTRIBUTING.md` - Professional contribution guidelines with code of conduct
- **Documentation Hub**: `/docs/README.md` - Central navigation for all documentation
- **TUI User Guide**: `/apps/tui-node/docs/guides/USER_GUIDE.md` - Complete guide for terminal interface
- **Architecture Overview**: `/docs/architecture/README.md` - System design and decisions

### Documentation Maintenance

When updating the codebase:
1. Update relevant documentation immediately
2. Keep examples and code snippets current
3. Archive obsolete documentation to `docs/archive/`
4. Maintain consistent formatting and style
5. Update navigation README files when adding new documents
6. Ensure all links remain valid

This documentation structure ensures that developers, architects, security teams, and operators can quickly find the information they need, while maintaining professional standards expected of enterprise software.
