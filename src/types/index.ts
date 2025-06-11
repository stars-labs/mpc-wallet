// ===================================================================
// TYPES INDEX - CENTRALIZED TYPE EXPORTS
// ===================================================================
//
// This file provides a centralized location to import the most commonly
// used types across the MPC wallet application. This makes it easier
// for developers to find and import types without needing to know
// which specific file contains each type.
//
// Usage Example:
// import { AppState, SessionInfo, DkgState } from '../types';
// 
// Key Concepts for Junior Developers:
// - Index File: A common pattern to re-export types from multiple files
// - Barrel Export: Collecting and re-exporting from a single entry point
// - Type Organization: Grouping related types for easier discovery
// ===================================================================

// Core Application State
export type { AppState, AppStateAction, AppStateEvent } from './appstate';
// Note: INITIAL_APP_STATE is not re-exported to avoid runtime circular dependencies
// Import directly from './appstate' if needed

// Session Management
export type {
    SessionInfo,
    SessionProposal,
    SessionResponse,
    SessionValidation,
    SessionValidator
} from './session';

// DKG (Distributed Key Generation)
export type {
    DkgState,
    DkgPackageInfo,
    DkgStatus,
    DkgEvent
} from './dkg';

// Mesh Network
export type {
    MeshStatusType,
    MeshStatus
} from './mesh';

// WebRTC Communication
export type {
    WebRTCAppMessage,
    DataChannelInfo,
    WebRTCConnectionStatus,
    WebRTCEvent
} from './webrtc';

// WebSocket Signaling
export type {
    SDPInfo,
    CandidateInfo,
    WebRTCSignal,
    WebSocketMessagePayload,
    ServerMsg,
    ClientMsg,
    WebSocketEvent
} from './websocket';

// Account Management
export type {
    Account,
    AccountBalance,
    AccountStorage,
    AccountEvent
} from './account';

// Network Configuration
export type {
    NetworkConfig,
    NetworkEvent
} from './network';

// Message Types (commonly used for inter-component communication)
export type {
    PopupToBackgroundMessage,
    BackgroundToPopupMessage,
    BackgroundToOffscreenMessage,
    OffscreenToBackgroundMessage,
    BackgroundToOffscreenWrapper,
    OffscreenToBackgroundWrapper,
    InitialStateMessage,
    JsonRpcRequest,
    JsonRpcResponse
} from './messages';

// Message validation helpers
export {
    validateMessage,
    validateSessionProposal,
    validateSessionAcceptance,
    isRpcMessage,
    isAccountManagement,
    isNetworkManagement,
    isUIRequest,
    MESSAGE_TYPES
} from './messages';
