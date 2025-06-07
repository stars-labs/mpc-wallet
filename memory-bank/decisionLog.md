# Decision Log - MPC Wallet Chrome Extension

*Last Updated: 2024-12-29*

## Technology Stack Decisions

### Framework Selection
**Decision**: WXT Framework for Chrome Extension
- **Date**: Early 2024
- **Reasoning**: 
  - Type-safe manifest handling
  - Hot reload for development
  - Multi-browser support
  - Better TypeScript integration than standard Chrome extension APIs
- **Alternative Considered**: Standard Chrome Extension APIs
- **Impact**: Simplified development workflow and better developer experience

### Frontend Framework
**Decision**: Svelte for UI Components
- **Date**: Early 2024
- **Reasoning**:
  - Smaller bundle size compared to React/Vue
  - Compile-time optimizations
  - Simple state management
  - Good TypeScript support
- **Alternative Considered**: React, Vue
- **Impact**: Faster load times and smoother user experience

### Crypto Implementation
**Decision**: Rust/WASM for Cryptographic Operations
- **Date**: Early 2024
- **Reasoning**:
  - Performance for complex MPC operations
  - Memory safety for crypto code
  - Reusable across different platforms
  - Access to mature Rust crypto ecosystem
- **Alternative Considered**: Pure JavaScript crypto libraries
- **Impact**: Better security and performance for FROST DKG operations

## Architecture Decisions

### Multi-Context Communication
**Decision**: Message-driven architecture with type-safe interfaces
- **Date**: Mid 2024
- **Reasoning**:
  - Clear separation of concerns
  - Type safety across extension contexts
  - Scalable for future features
  - Easier debugging and maintenance
- **Implementation**: Custom message system in `src/types/messages.ts`
- **Impact**: Reliable communication between popup, background, content, and offscreen contexts

### WebRTC Integration
**Decision**: Offscreen document for WebRTC operations
- **Date**: Late 2024
- **Reasoning**:
  - Service workers don't support WebRTC APIs
  - Need persistent context for P2P connections
  - Better resource management
- **Alternative Considered**: Content script implementation
- **Impact**: Enabled P2P FROST DKG operations

### State Management
**Decision**: Centralized state in background script with message passing
- **Date**: Mid 2024
- **Reasoning**:
  - Single source of truth
  - Persistence across popup sessions
  - Consistent state across all contexts
- **Alternative Considered**: Local storage, IndexedDB
- **Impact**: Reliable state management across extension lifecycle

## Crypto Protocol Decisions

### DKG Protocol Selection
**Decision**: FROST (Flexible Round-Optimized Schnorr Threshold) DKG
- **Date**: Early 2024
- **Reasoning**:
  - Industry standard for threshold signatures
  - Support for multiple curves (Ed25519, Secp256k1)
  - Proven security model
  - Efficient round structure
- **Alternative Considered**: Other threshold signature schemes
- **Impact**: Robust multi-party key generation and signing

### Multi-Curve Support
**Decision**: Support both Ed25519 and Secp256k1 curves
- **Date**: Mid 2024
- **Reasoning**:
  - Ed25519 for Solana compatibility
  - Secp256k1 for Ethereum compatibility
  - Future-proofing for other chains
- **Challenge**: Different identifier serialization formats
- **Impact**: True multi-chain wallet capability

### P2P Communication
**Decision**: WebRTC for peer-to-peer DKG communication
- **Date**: Late 2024
- **Reasoning**:
  - Direct peer communication without central server
  - Real-time bidirectional communication
  - Built-in NAT traversal
  - Lower latency than server-mediated approaches
- **Alternative Considered**: WebSocket server, HTTP polling
- **Impact**: Decentralized key generation process

## Development Decisions

### Testing Strategy
**Decision**: Comprehensive unit tests for FROST DKG operations
- **Date**: December 2024
- **Reasoning**:
  - Complex cryptographic operations need thorough testing
  - Prevent regressions in protocol implementation
  - Validate cross-curve compatibility
- **Implementation**: `webrtc.test.ts` with round-by-round validation
- **Impact**: Higher confidence in crypto implementation

### Error Handling
**Decision**: Structured error handling with detailed logging
- **Date**: December 2024
- **Reasoning**:
  - Complex async operations need clear error tracking
  - Debug information crucial for crypto protocol issues
  - Better user experience with meaningful error messages
- **Implementation**: Enhanced error handling in test suite
- **Impact**: Faster debugging and better reliability

## Pending Decisions

### Key Storage
**Status**: Under consideration
- **Options**: Browser storage, hardware security modules, encrypted local storage
- **Considerations**: Security, accessibility, backup/recovery

### Multi-Party Coordination
**Status**: Research phase
- **Options**: Signaling server, DHT, direct exchange
- **Considerations**: Decentralization, reliability, user experience

### Chain Integration
**Status**: Planned
- **Priority**: Ethereum and Solana integration
- **Considerations**: Chain-specific transaction formatting, gas estimation

## Decision Rationale Template

For future decisions, document:
1. **Context**: What problem are we solving?
2. **Options**: What alternatives were considered?
3. **Criteria**: What factors influenced the decision?
4. **Decision**: What was chosen and why?
5. **Consequences**: What are the implications?
6. **Review Date**: When should this be reconsidered?