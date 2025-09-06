use frost_core::Ciphersuite;
use serde::{Deserialize, Serialize};

use webrtc::ice_transport::ice_candidate::RTCIceCandidateInit;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

/// Curve type for cryptographic operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CurveType {
    Secp256k1,
    Ed25519,
}

/// Coordination type for session management
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CoordinationType {
    Network,
    Offline,
}

/// Session type enum - represents different types of signing networks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SessionInfo {
    pub session_id: String,
    pub proposer_id: String, // Added field
    pub total: u16,
    pub threshold: u16,
    pub participants: Vec<String>,
    pub accepted_devices: Vec<String>, // List of device_ids that have accepted
    pub session_type: SessionType,
    /// Cryptographic curve type from the proposer
    pub curve_type: String,
    /// Coordination type from the proposer
    #[serde(default = "default_coordination_type")]
    pub coordination_type: String,
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
    /// Session update message (participant list changes)
    SessionUpdate(SessionUpdate),
    /// Session join request (for joining/rejoining)
    SessionJoinRequest(SessionJoinRequest),
    /// Session offer (compatibility with message validator)
    SessionOffer(SessionInfo),
    /// Session accepted (compatibility with message validator)
    SessionAccepted { device_id: String, session_id: String },
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
    /// Device ID of the wallet creator/proposer
    pub proposer_device_id: String,
    /// Cryptographic curve type (secp256k1 or ed25519)
    pub curve_type: String,
    /// Coordination type (network or file)
    #[serde(default = "default_coordination_type")]
    pub coordination_type: String,
}

fn default_coordination_type() -> String {
    "network".to_string()
}

/// Session join request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionJoinRequest {
    pub session_id: String,
    pub device_id: String,
    pub is_rejoin: bool,
}

/// Session announcement for discovery
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionAnnouncement {
    pub session_code: String,
    pub wallet_type: String,
    pub threshold: u16,
    pub total: u16,
    pub curve_type: String,
    pub creator_device: String,
    pub participants_joined: u16,
    pub description: Option<String>,
    pub timestamp: u64,
}

/// Session response information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionResponse {
    pub session_id: String,
    pub from_device_id: String,  // Added to identify sender
    pub accepted: bool,
    pub wallet_status: Option<WalletStatus>,
    pub reason: Option<String>,   // Added for rejoin reason
}

/// Session update information - broadcast when participants join/leave
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionUpdate {
    pub session_id: String,
    pub accepted_devices: Vec<String>,
    pub update_type: SessionUpdateType,
    pub timestamp: u64,  // Added for ordering updates
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionUpdateType {
    ParticipantJoined,
    ParticipantLeft,
    ParticipantRejoined,  // Added for rejoin scenario
    FullSync,
}

/// Wallet status for signing sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletStatus {
    pub has_wallet: bool,
    pub wallet_valid: bool,
    pub identifier: Option<u16>,
    pub error_reason: Option<String>,
}

impl SessionInfo {
    /// Determines the consensus leader using a deterministic algorithm
    /// based on lexicographic ordering of accepted participants.
    /// This ensures all nodes agree on the leader without central coordination.
    pub fn get_consensus_leader(&self) -> String {
        // If we have accepted devices, use the first one lexicographically
        if !self.accepted_devices.is_empty() {
            let mut sorted_devices = self.accepted_devices.clone();
            sorted_devices.sort();
            sorted_devices[0].clone()
        } else {
            // Fallback to proposer if no accepted devices yet
            self.proposer_id.clone()
        }
    }
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
