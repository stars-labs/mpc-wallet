import type { SessionInfo, DkgState, MeshStatus, SessionProposal, SessionResponse } from './appstate';
import type { AppState } from './appstate';
import type { WebRTCAppMessage as DataChannelMessage } from './webrtc';
import { ServerMsg, ClientMsg, WebSocketMessagePayload, WebRTCSignal } from './websocket';
// --- Core Message Structure ---
export interface BaseMessage {
    type: string;
    [key: string]: any; // Add index signature for compatibility
}

// --- JSON-RPC Types ---
export interface JsonRpcRequest {
    id: number | string;
    jsonrpc: '2.0';
    method: string;
    params?: unknown;
}

export interface JsonRpcResponse {
    id: number | string;
    jsonrpc: '2.0';
    result?: unknown;
    error?: {
        code: number;
        message: string;
        data?: unknown;
    };
}

// --- Simplified Background Message Types ---
export type BackgroundMessage = BaseMessage & (
    // Core wallet operations
    | { type: 'getState' }
    | { type: 'listPeers' }
    | { type: 'sendDirectMessage'; toPeerId: string; message: string }
    | { type: 'getWebRTCStatus' }
    // session management
    | { type: 'proposeSession'; session_id: string; total: number; threshold: number; participants: string[] }
    | { type: 'acceptSession'; session_id: string; accepted: boolean }

    // Management operations
    | { type: 'createOffscreen' }
    | { type: 'getOffscreenStatus' }
    | { type: 'offscreenReady' }

    // Communication
    | { type: 'relay'; to: string; data: WebSocketMessagePayload }
    | { type: 'fromOffscreen'; payload: OffscreenMessage }

    // RPC operations
    | { type: string; payload: JsonRpcRequest; action?: string; method?: string; params?: unknown[] }
);

// --- Simplified Offscreen Message Types ---
export type OffscreenMessage = BaseMessage & (
    | { type: 'getState' }
    | { type: 'sendDirectMessage'; toPeerId: string; message: string }
    | { type: 'getWebRTCStatus' }
    | { type: 'webrtcStatusUpdate'; peerId: string; status: string }
    | { type: 'sessionUpdate'; sessionInfo: SessionInfo | null; invites: SessionInfo[] }
    | { type: 'peerConnectionStatusUpdate'; peerId: string; connectionState: string }
    | { type: 'dataChannelStatusUpdate'; peerId: string; channelName: string; state: string }
    | { type: 'init'; peerId: string; wsUrl: string }
    | { type: 'relayViaWs'; to: string; data: any }
    | { type: 'webrtcConnectionUpdate'; peerId: string; connected: boolean }
    | { type: 'meshStatusUpdate'; status: MeshStatus }
    | { type: 'dkgStateUpdate'; state: DkgState }
    | { type: 'sessionAccepted'; sessionInfo: SessionInfo; currentPeerId: string }
    | { type: 'sessionAllAccepted'; sessionInfo: SessionInfo; currentPeerId: string }
    | { type: 'sessionResponseUpdate'; sessionInfo: SessionInfo; currentPeerId: string }
);

// Add the missing InitialStateMessage type
export interface InitialStateMessage extends BaseMessage, AppState {
    type: 'initialState';
    peerId: string;
    connectedPeers: string[];
    wsConnected: boolean;
    sessionInfo: SessionInfo | null;
    invites: SessionInfo[];
    meshStatus: { type: number };
    dkgState: number;
    webrtcConnections: Record<string, boolean>; // Add WebRTC connection state
}

// --- Simplified Popup Message Types ---
export type PopupMessage =
    | InitialStateMessage
    | { type: "wsStatus"; connected: boolean } & BaseMessage
    | { type: "wsError"; error: string } & BaseMessage
    | { type: "wsMessage"; message: any } & BaseMessage
    | { type: "peerList"; peers: string[] } & BaseMessage
    | { type: "sessionUpdate"; sessionInfo: SessionInfo | null; invites: SessionInfo[] } & BaseMessage
    | { type: "webrtcConnectionUpdate"; peerId: string; connected: boolean } & BaseMessage
    | { type: "webrtcStatusUpdate"; peerId: string; status: string } & BaseMessage
    | { type: "meshStatusUpdate"; status: MeshStatus } & BaseMessage
    | { type: "dkgStateUpdate"; state: DkgState } & BaseMessage
    | { type: "fromOffscreen"; payload: any } & BaseMessage;

// --- Message to Offscreen Types ---
export type BackgroundToOffscreenMessage = {
    type: 'fromBackground';
    payload: OffscreenMessage;
};

// --- Legacy Support Types (for existing code) ---
export type Account = { address: string;[key: string]: unknown };
export type Network = { id: number | string; name?: string;[key: string]: unknown };

// --- Helper Functions (exported as regular functions, not types) ---
export function isRpcMessage(msg: BackgroundMessage): msg is BackgroundMessage & { payload: JsonRpcRequest } {
    return 'payload' in msg && typeof msg.payload === 'object' && msg.payload !== null && 'jsonrpc' in msg.payload;
}

export function isAccountManagement(msg: BackgroundMessage): boolean {
    return msg.type === 'ACCOUNT_MANAGEMENT';
}

export function isNetworkManagement(msg: BackgroundMessage): boolean {
    return msg.type === 'NETWORK_MANAGEMENT';
}

export function isUIRequest(msg: BackgroundMessage): msg is BackgroundMessage & { payload: { method: string; params: unknown[] } } {
    return msg.type === 'UI_REQUEST' && 'payload' in msg && typeof msg.payload === 'object' && msg.payload !== null && 'method' in msg.payload;
}

// --- Validation Helpers ---
export function validateMessage(msg: unknown): msg is BackgroundMessage {
    return typeof msg === 'object' && msg !== null && 'type' in msg && typeof (msg as any).type === 'string';
}

export function validateSessionProposal(msg: BackgroundMessage): msg is BackgroundMessage & { session_id: string; total: number; threshold: number; participants: string[] } {
    return msg.type === 'proposeSession' &&
        'session_id' in msg && typeof msg.session_id === 'string' &&
        'total' in msg && typeof msg.total === 'number' &&
        'threshold' in msg && typeof msg.threshold === 'number' &&
        'participants' in msg && Array.isArray(msg.participants);
}

export function validateSessionAcceptance(msg: BackgroundMessage): msg is BackgroundMessage & { session_id: string; accepted: boolean } {
    return msg.type === 'acceptSession' &&
        'session_id' in msg && typeof msg.session_id === 'string' &&
        'accepted' in msg && typeof msg.accepted === 'boolean';
}

// --- Legacy Types (kept for compatibility) ---
export type ContentToInjectedMsg = BaseMessage;
export type InjectedToContentMsg = BaseMessage;
export type ContentToBackgroundMsg = BaseMessage;
export type BackgroundToContentMsg = BaseMessage;
export type PopupToBackgroundMsg = BaseMessage;
export type BackgroundToPopupMsg = BaseMessage;
export type BackgroundToOffscreenMsg = BaseMessage;
export type OffscreenToBackgroundMsg = BaseMessage;
export type WebSocketClientMsg = BaseMessage;
export type WebSocketServerMsg = BaseMessage;
export type AnyMessage = BaseMessage;

// --- Message Constants ---
export const MESSAGE_TYPES = {
    GET_STATE: "getState",
    LIST_PEERS: "listPeers",
    PROPOSE_SESSION: "proposeSession",
    ACCEPT_SESSION: "acceptSession",
    RELAY: "relay",
    FROM_OFFSCREEN: "fromOffscreen",
    OFFSCREEN_READY: "offscreenReady",
    CREATE_OFFSCREEN: "createOffscreen",
    GET_OFFSCREEN_STATUS: "getOffscreenStatus",
    GET_WEBRTC_STATE: "getWebRTCState",
    SEND_DIRECT_MESSAGE: "sendDirectMessage",
    GET_WEBRTC_STATUS: "getWebRTCStatus",
    WEBRTC_STATUS_UPDATE: "webrtcStatusUpdate",
    SESSION_UPDATE: "sessionUpdate",
    PEER_CONNECTION_STATUS_UPDATE: "peerConnectionStatusUpdate",
    DATA_CHANNEL_STATUS_UPDATE: "dataChannelStatusUpdate",
    // Legacy support
    ACCOUNT_MANAGEMENT: "ACCOUNT_MANAGEMENT",
    NETWORK_MANAGEMENT: "NETWORK_MANAGEMENT",
    UI_REQUEST: "UI_REQUEST",
} as const;
