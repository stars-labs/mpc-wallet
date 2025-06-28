export interface SessionInfo {
  session_id: string;
  proposer_id: string;
  total: number; // u16 in Rust, number in TS
  threshold: number; // u16 in Rust, number in TS
  participants: string[];
  accepted_devices: string[];
  status?: string; // Add optional status field
}


// --- Enums for State Management ---
export enum DkgState {
  Idle,
  Initializing,
  Round1InProgress,
  Round1Complete,
  Round2InProgress,
  Round2Complete,
  Finalizing,
  Complete,
  Failed,
}

export enum MeshStatusType {
  Incomplete,
  PartiallyReady,
  Ready,
}

export type MeshStatus =
  | { type: MeshStatusType.Incomplete }
  | { type: MeshStatusType.PartiallyReady; ready_devices: Set<string>; total_devices: number }
  | { type: MeshStatusType.Ready };

// --- WebRTC Signaling ---
export interface SessionProposal {
  session_id: string;
  total: number;
  threshold: number;
  participants: string[];
}

export interface SessionResponse {
  session_id: string;
  accepted: boolean;
}

export type WebRTCAppMessage =
  | { webrtc_msg_type: 'ChannelOpen'; device_id: string }
  | { webrtc_msg_type: 'MeshReady'; session_id: string; device_id: string }
  | { webrtc_msg_type: 'SimpleMessage'; text: string }
  | { webrtc_msg_type: 'DkgRound1Package'; package: any }
  | { webrtc_msg_type: 'DkgRound2Package'; package: any };

// Define the main application state interface
export interface AppState {
  deviceId: string;
  connecteddevices: string[];
  wsConnected: boolean;
  sessionInfo: SessionInfo | null;
  invites: SessionInfo[];
  meshStatus: MeshStatus;
  dkgState: DkgState;
  webrtcConnections: Record<string, boolean>;
  blockchain?: "ethereum" | "solana"; // Current blockchain selection for the active session
}

// Initial state for the application
export const INITIAL_APP_STATE: AppState = {
  deviceId: '',
  connecteddevices: [],
  wsConnected: false,
  sessionInfo: null,
  invites: [],
  meshStatus: { type: MeshStatusType.Incomplete },
  dkgState: DkgState.Idle,
  webrtcConnections: {},
  blockchain: "ethereum"
};

// Supported chains type
export type SupportedChain = 'ethereum' | 'solana';

// Curve compatibility mapping
export const CURVE_COMPATIBLE_CHAINS: Record<string, SupportedChain[]> = {
  'secp256k1': ['ethereum'],
  'ed25519': ['solana']
};

// Get compatible chains for a curve type
export function getCompatibleChains(curveType: string): SupportedChain[] {
  return CURVE_COMPATIBLE_CHAINS[curveType] || [];
}

// Get required curve for a chain
export function getRequiredCurve(chain: SupportedChain): string {
  for (const [curve, chains] of Object.entries(CURVE_COMPATIBLE_CHAINS)) {
    if (chains.includes(chain)) {
      return curve;
    }
  }
  return 'secp256k1'; // Default to secp256k1
}