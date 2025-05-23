// WebRTC Signaling Data (sent via WebSocket Relay)
export interface SDPInfo {
  sdp: string;
}

export interface CandidateInfo {
  candidate: string;
  sdpMid?: string | null; // sdp_mid in Rust
  sdpMLineIndex?: number | null; // sdp_mline_index in Rust (u16)
}

// Updated WebRTCSignal to match Rust's externally tagged enum serialization
export type WebRTCSignal =
  | { Offer: SDPInfo }
  | { Answer: SDPInfo }
  | { Candidate: CandidateInfo };

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

// Updated WebSocketMessagePayload to model Rust's WebSocketMessage enum
export type WebSocketMessagePayload =
  | ({ websocket_msg_type: 'SessionProposal' } & SessionProposal)
  | ({ websocket_msg_type: 'SessionResponse' } & SessionResponse)
  | ({ websocket_msg_type: 'WebRTCSignal' } & WebRTCSignal);

// Updated ServerMsg to match Rust's internally tagged enum serialization
// Rust: #[serde(tag = "type", rename_all = "snake_case")]
export type ServerMsg =
  | { type: "peers"; peers: string[] }
  | { type: "relay"; from: string; data: WebSocketMessagePayload }
  | { type: "error"; error: string };

export type ClientMsg =
  | { type: "register"; peer_id: string }
  | { type: "list_peers" }
  | { type: "relay"; to: string; data: WebSocketMessagePayload };