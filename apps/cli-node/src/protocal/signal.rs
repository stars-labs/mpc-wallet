use frost_core::Ciphersuite;
use serde::{Deserialize, Serialize};

use webrtc::ice_transport::ice_candidate::RTCIceCandidateInit;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

/// Session type enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum SessionType {
    /// Distributed Key Generation session
    DKG,
    /// Signing session with existing wallet
    Signing {
        wallet_name: String,
        curve_type: String,
        blockchain: String,
        group_public_key: String,
    },
}
// Import the DKG Package type
// Import round1 and round2 packages

// --- Session Info Struct ---
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct SessionInfo {
    pub session_id: String,
    pub proposer_id: String, // Added field
    pub total: u16,
    pub threshold: u16,
    pub participants: Vec<String>,
    pub accepted_devices: Vec<String>, // List of device_ids that have accepted
    pub session_type: SessionType,
}

// --- WebRTC Signaling Data (sent via Relay) ---
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum WebRTCSignal {
    Offer(SDPInfo),
    Answer(SDPInfo),
    Candidate(CandidateInfo),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SDPInfo {
    pub sdp: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CandidateInfo {
    pub candidate: String,
    #[serde(rename = "sdpMid")]
    pub sdp_mid: Option<String>,
    #[serde(rename = "sdpMLineIndex")]
    pub sdp_mline_index: Option<u16>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "websocket_msg_type")]
pub enum WebSocketMessage {
    // Relay Messages
    /// Session proposal message
    SessionProposal(SessionProposal),
    /// Session response message
    SessionResponse(SessionResponse),
    WebRTCSignal(WebRTCSignal),
}

/// Session proposal information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionProposal {
    pub session_id: String,
    pub total: u16,
    pub threshold: u16,
    pub participants: Vec<String>,
    pub session_type: SessionType,
}

/// Session response information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionResponse {
    pub session_id: String,
    pub accepted: bool,
    pub wallet_status: Option<WalletStatus>,
}

/// Wallet status for signing sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletStatus {
    pub has_wallet: bool,
    pub wallet_valid: bool,
    pub identifier: Option<u16>,
    pub error_reason: Option<String>,
}

// --- Application-Level Messages (sent over established WebRTC Data Channel) ---
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "webrtc_msg_type")]
#[serde(bound(
    serialize = "frost_core::keys::dkg::round1::Package<C>: serde::Serialize, frost_core::keys::dkg::round2::Package<C>: serde::Serialize, frost_core::round1::SigningCommitments<C>: serde::Serialize, frost_core::round2::SignatureShare<C>: serde::Serialize, frost_core::Identifier<C>: serde::Serialize",
    deserialize = "frost_core::keys::dkg::round1::Package<C>: serde::Deserialize<'de>, frost_core::keys::dkg::round2::Package<C>: serde::Deserialize<'de>, frost_core::round1::SigningCommitments<C>: serde::Deserialize<'de>, frost_core::round2::SignatureShare<C>: serde::Deserialize<'de>, frost_core::Identifier<C>: serde::Deserialize<'de>"
))]
pub enum WebRTCMessage<C: Ciphersuite> {
    // DKG Messages
    SimpleMessage {
        text: String,
    },
    DkgRound1Package {
        package: frost_core::keys::dkg::round1::Package<C>,
    },
    // Add other message types as needed (e.g., for signing)
    DkgRound2Package {
        package: frost_core::keys::dkg::round2::Package<C>,
    },
    /// Data channel opened notification
    ChannelOpen {
        device_id: String,
    },
    /// Mesh readiness notification
    MeshReady {
        session_id: String,
        device_id: String,
    },

    // --- Signing Messages ---
    /// Transaction signing request
    SigningRequest {
        signing_id: String,
        transaction_data: String, // Hex-encoded transaction data
        required_signers: usize,
        blockchain: String,       // Blockchain identifier
        chain_id: Option<u64>,    // Chain ID for EVM chains
    },

    /// Acceptance of a signing request
    SigningAcceptance {
        signing_id: String,
        accepted: bool,
    },

    /// Selected signers for threshold signing
    SignerSelection {
        signing_id: String,
        selected_signers: Vec<frost_core::Identifier<C>>,
    },

    /// FROST signing commitments (Round 1)
    SigningCommitment {
        signing_id: String,
        sender_identifier: frost_core::Identifier<C>,
        commitment: frost_core::round1::SigningCommitments<C>,
    },

    /// FROST signature shares (Round 2)
    SignatureShare {
        signing_id: String,
        sender_identifier: frost_core::Identifier<C>,
        share: frost_core::round2::SignatureShare<C>,
    },

    /// Final aggregated signature
    AggregatedSignature {
        signing_id: String,
        signature: Vec<u8>, // The final signature bytes
    },
}

// Helper to convert RTCIceCandidate to CandidateInfo
impl From<RTCIceCandidateInit> for CandidateInfo {
    fn from(init: RTCIceCandidateInit) -> Self {
        CandidateInfo {
            candidate: init.candidate,
            sdp_mid: init.sdp_mid,
            sdp_mline_index: init.sdp_mline_index,
        }
    }
}

// Helper to convert RTCSessionDescription to SDPInfo
impl From<RTCSessionDescription> for SDPInfo {
    fn from(desc: RTCSessionDescription) -> Self {
        SDPInfo { sdp: desc.sdp }
    }
}
