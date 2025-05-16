// This file defines the types for WebSocket messages between client and server

export interface ServerMsg {
    type: "peers" | "relay" | "error";
    peers?: string[];
    from?: string;
    data?: any;
    error?: string;
}

export type ClientMsg =
    | { type: "register"; peer_id: string }
    | { type: "list_peers" }
    | { type: "relay"; to: string; data: any };

export interface SessionInfo {
    session_id: string;
    total: number;
    threshold: number;
    participants: string[];
}

export interface SessionInfo {
    session_id: string;
    proposer_id: string;
    total: number; // u16 in Rust, number in TS
    threshold: number; // u16 in Rust, number in TS
    participants: string[];
    accepted_peers: string[];
  }
  
  // WebRTC Signaling Data (sent via WebSocket Relay)
  export interface SDPInfo {
    sdp: string;
  }
  
  export interface CandidateInfo {
    candidate: string;
    sdpMid?: string | null; // sdp_mid in Rust
    sdpMLineIndex?: number | null; // sdp_mline_index in Rust (u16)
  }
  
  export type WebRTCSignal =
    | { type: 'Offer'; data: SDPInfo }
    | { type: 'Answer'; data: SDPInfo }
    | { type: 'Candidate'; data: CandidateInfo };
  
  // WebSocket Messages (content of Relay data)
  export interface SessionProposal {
    session_id: string;
    total: number; // u16
    threshold: number; // u16
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
  
  // Application-Level Messages (sent over established WebRTC Data Channel)
  // Using 'any' for DKG packages as their structure is complex and not fully defined for TS here.
  export type WebRTCAppMessage =
    | { webrtc_msg_type: 'SimpleMessage'; text: string }
    | { webrtc_msg_type: 'DkgRound1Package'; package: any } // frost_core::keys::dkg::round1::Package<Ed25519Sha512>
    | { webrtc_msg_type: 'DkgRound2Package'; package: any } // frost_core::keys::dkg::round2::Package<Ed25519Sha512>
    | { webrtc_msg_type: 'ChannelOpen'; peer_id: string }
    | { webrtc_msg_type: 'MeshReady'; session_id: string; peer_id: string };
  
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
  
  