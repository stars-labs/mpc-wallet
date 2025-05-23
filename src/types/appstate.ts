export interface SessionInfo {
  session_id: string;
  proposer_id: string;
  total: number; // u16 in Rust, number in TS
  threshold: number; // u16 in Rust, number in TS
  participants: string[];
  accepted_peers: string[];
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

// Add missing WebRTC signal types
export interface SDPInfo {
  sdp: string;
}

export interface CandidateInfo {
  candidate: string;
  sdpMid: string | null;
  sdpMLineIndex: number | null;
}

export type WebRTCSignal =
  | { type: 'Offer'; data: SDPInfo }
  | { type: 'Answer'; data: SDPInfo }
  | { type: 'Candidate'; data: CandidateInfo };

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

export type WebSocketMessagePayload =
  | { websocket_msg_type: 'SessionProposal'; data: SessionProposal }
  | { websocket_msg_type: 'SessionResponse'; data: SessionResponse }
  | { websocket_msg_type: 'WebRTCSignal'; data: WebRTCSignal };

export type WebRTCAppMessage =
  | { webrtc_msg_type: 'ChannelOpen'; peer_id: string }
  | { webrtc_msg_type: 'MeshReady'; session_id: string; peer_id: string }
  | { webrtc_msg_type: 'DkgRound1Package'; package: any }
  | { webrtc_msg_type: 'DkgRound2Package'; package: any };

// Add WebSocket server message types
export interface ServerMsg {
  type: string;
  peers?: string[];
  from?: string;
  data?: any;
}

export interface ClientMsg {
  type: string;
  peer_id?: string;
  to?: string;
  data?: any;
}

