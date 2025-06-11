export interface SessionInfo {
  session_id: string;
  proposer_id: string;
  total: number; // u16 in Rust, number in TS
  threshold: number; // u16 in Rust, number in TS
  participants: string[];
  accepted_peers: string[];
  status?: string; // Add optional status field
}


// --- Enums for State Management ---
export enum DkgState {
  Idle,
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
  | { type: MeshStatusType.PartiallyReady; ready_peers: Set<string>; total_peers: number }
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
  | { webrtc_msg_type: 'ChannelOpen'; peer_id: string }
  | { webrtc_msg_type: 'MeshReady'; session_id: string; peer_id: string }
  | { webrtc_msg_type: 'SimpleMessage'; text: string }
  | { webrtc_msg_type: 'DkgRound1Package'; package: any }
  | { webrtc_msg_type: 'DkgRound2Package'; package: any };

// Define the main application state interface
export interface AppState {
  peerId: string;
  connectedPeers: string[];
  wsConnected: boolean;
  sessionInfo: SessionInfo | null;
  invites: SessionInfo[];
  meshStatus: MeshStatus;
  dkgState: DkgState;
  webrtcConnections: Record<string, boolean>;
  blockchain?: "ethereum" | "solana"; // Current blockchain selection for the active session
}