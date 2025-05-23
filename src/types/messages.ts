import type { SessionInfo, DkgState, MeshStatus, SessionProposal, SessionResponse, WebSocketMessagePayload, WebRTCSignal, ServerMsg } from './appstate';

// --- Core Message Structure ---
export interface BaseMessage {
    type: string;
    [key: string]: unknown;
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
    | { type: 'log'; message: string }
    | { type: 'relayViaWs'; to: string; data: WebRTCSignal }
    | { type: 'init'; peerId: string; wsUrl: string }
    | { type: 'relayMessage'; fromPeerId: string; data: WebSocketMessagePayload }
    | { type: 'proposeSession'; session_id: string; total: number; threshold: number; participants: string[] }
    | { type: 'acceptSession'; session_id: string; accepted: boolean }
);

// Add the missing InitialStateMessage type
export interface InitialStateMessage {
    type: 'initialState';
    peerId: string;
    connectedPeers: string[];
    wsConnected: boolean;
    sessionInfo: SessionInfo | null;
    invites: SessionInfo[];
    meshStatus: { type: number };
    dkgState: number;
}

// --- Simplified Popup Message Types ---
export type PopupMessage = BaseMessage & (
    | { type: 'wsStatus'; connected: boolean; reason?: string }
    | { type: 'wsMessage'; message: ServerMsg }
    | { type: 'peerList'; peers: string[] }
    | { type: 'wsError'; error: string }
    | { type: 'fromOffscreen'; payload: OffscreenMessage }
    | InitialStateMessage
);

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
    GET_STATE: 'getState',
    LIST_PEERS: 'listPeers',
    PROPOSE_SESSION: 'proposeSession',
    ACCEPT_SESSION: 'acceptSession',
    RELAY: 'relay',
    FROM_OFFSCREEN: 'fromOffscreen',
    OFFSCREEN_READY: 'offscreenReady',
    CREATE_OFFSCREEN: 'createOffscreen',
    GET_OFFSCREEN_STATUS: 'getOffscreenStatus',

    // Legacy support
    ACCOUNT_MANAGEMENT: 'ACCOUNT_MANAGEMENT',
    NETWORK_MANAGEMENT: 'NETWORK_MANAGEMENT',
    UI_REQUEST: 'UI_REQUEST',
} as const;
