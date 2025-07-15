use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum InternalCommand {
    // WebSocket commands
    ConnectWebSocket,
    DisconnectWebSocket,
    SendWebSocketMessage(serde_json::Value),
    
    // Session commands
    ProposeSession {
        session_id: String,
        total_participants: u16,
        threshold: u16,
        curve: String,
    },
    AcceptSessionProposal {
        session_id: String,
    },
    RejectSessionProposal {
        session_id: String,
    },
    
    // WebRTC commands
    CreateWebRTCConnection {
        target_device: String,
    },
    SendWebRTCSignal {
        target_device: String,
        signal: WebRTCSignal,
    },
    SendDirect {
        target_device: String,
        message: DirectMessage,
    },
    
    // WebRTC state updates
    UpdateWebRTCState {
        device_id: String,
        state: crate::state::WebRTCConnectionState,
    },
    SendWebSocketRelay {
        target_device: String,
        message: serde_json::Value,
    },
    ProcessDirectMessage {
        from_device: String,
        message: DirectMessage,
    },
    
    // DKG commands
    CheckAndTriggerDkg,
    TriggerDkgRound1,
    TriggerDkgRound2,
    ProcessDkgRound1 {
        from_device: String,
        data: Vec<u8>,
    },
    ProcessDkgRound2 {
        from_device: String,
        data: Vec<u8>,
    },
    
    // Signing commands
    InitiateSigning {
        transaction_data: String,
        blockchain: String,
    },
    ProcessSigningRequest {
        request: SigningRequestMessage,
    },
    AcceptSigning {
        request_id: String,
    },
    RejectSigning {
        request_id: String,
    },
    ProcessSigningCommitment {
        from_device: String,
        request_id: String,
        commitment: Vec<u8>,
    },
    ProcessSigningShare {
        from_device: String,
        request_id: String,
        share: Vec<u8>,
    },
    
    // Keystore commands
    InitKeystore,
    ImportKeystore {
        path: String,
        password: String,
    },
    ExportKeystore {
        path: String,
        password: String,
    },
    
    // UI commands
    UpdateUI,
    ShowSessionProposal {
        invite: crate::state::SessionInvite,
    },
    ShowSigningRequest {
        request: crate::state::SigningRequest,
    },
    ShowWalletList,
    ClosePopup,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebRTCSignal {
    Offer { sdp: String },
    Answer { sdp: String },
    IceCandidate { candidate: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DirectMessage {
    SessionProposal {
        session_id: String,
        total_participants: u16,
        threshold: u16,
        curve: String,
    },
    SessionResponse {
        session_id: String,
        accepted: bool,
    },
    MeshReady {
        session_id: String,
    },
    DkgRound1 {
        data: Vec<u8>,
    },
    DkgRound2 {
        data: Vec<u8>,
    },
    SigningRequest {
        request_id: String,
        transaction_data: String,
        blockchain: String,
    },
    SigningCommitment {
        request_id: String,
        commitment: Vec<u8>,
    },
    SigningShare {
        request_id: String,
        share: Vec<u8>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningRequestMessage {
    pub request_id: String,
    pub from_device: String,
    pub transaction_data: String,
    pub blockchain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    DeviceList {
        devices: Vec<String>,
    },
    Relay {
        from_device: String,
        to_device: String,
        message: serde_json::Value,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    Register {
        device_id: String,
    },
    Relay {
        to_device: String,
        message: serde_json::Value,
    },
}