use crate::protocal::signal::SessionInfo;
use frost_core::{
    Ciphersuite, Identifier,
    keys::{
        KeyPackage,
        PublicKeyPackage,
        dkg::{round1, round2}, // Import the specific DKG types
    },
};

use std::sync::Arc; // Use TokioMutex for device_connections
use std::time::{Duration, Instant}; // Import Duration and Instant
use tokio::sync::Mutex; // Use TokioMutex for async WebRTC state
use webrtc::{
    data_channel::RTCDataChannel, ice_transport::ice_candidate::RTCIceCandidateInit,
    peer_connection::RTCPeerConnection,
    peer_connection::peer_connection_state::RTCPeerConnectionState,
}; // Keep SessionInfo import

use std::{
    collections::{BTreeMap, HashMap, HashSet}, // Keep BTreeMap
                                               // Remove Arc import from here if only used for device_connections
};

use webrtc_signal_server::ClientMsg as SharedClientMsg;
// Add this import

use crate::protocal::signal::SessionResponse;

#[derive(Debug, Clone)]
pub struct PendingSigningRequest {
    pub signing_id: String,
    pub from_device: String,
    pub transaction_data: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum InternalCommand<C: Ciphersuite> {
    // --- Keystore Commands ---
    /// Initialize a keystore
    InitKeystore {
        path: String,
        device_name: String,
    },

    /// List available wallets
    ListWallets,

    /// Create a new wallet from DKG results
    CreateWallet {
        name: String,
        description: Option<String>,
        password: String,
        tags: Vec<String>,
    },
    
    /// Show wallet file location for direct sharing
    LocateWallet {
        wallet_id: String,
    },
    /// Send a message to the signaling server
    SendToServer(SharedClientMsg),

    /// Send a direct WebRTC message to a device
    SendDirect {
        to: String,
        message: String,
    },

    /// Propose a new MPC session (replacing the old CreateSession)
    ProposeSession {
        session_id: String,
        total: u16,
        threshold: u16,
        participants: Vec<String>,
    },

    /// Accept a session proposal by session ID
    AcceptSessionProposal(String),

    /// Process a session response from a device
    ProcessSessionResponse {
        from_device_id: String,
        response: SessionResponse,
    },

    /// Report that a data channel has been opened with a device
    ReportChannelOpen {
        device_id: String,
    },

    // MeshReady, // This variant is redundant and has been removed. Use SendOwnMeshReadySignal.
    SendOwnMeshReadySignal,
    /// Process mesh ready notification from a device
    ProcessMeshReady {
        device_id: String,
    },

    /// Check if conditions are met to trigger DKG and do so if appropriate
    CheckAndTriggerDkg,

    /// Trigger DKG Round 1 (Commitments)
    TriggerDkgRound1,

    /// Process DKG Round 1 data from a device
    ProcessDkgRound1 {
        from_device_id: String,
        package: round1::Package<C>,
    },

    /// Trigger DKG Round 2 (Shares)
    TriggerDkgRound2,

    /// Process DKG Round 2 data from a device
    ProcessDkgRound2 {
        from_device_id: String,
        package: round2::Package<C>,
    },

    /// Finalize the DKG process
    FinalizeDkg,

    // --- Signing Commands ---
    /// Initiate a signing process with transaction data
    InitiateSigning {
        transaction_data: String, // Hex-encoded transaction data
        blockchain: String,       // Blockchain identifier (e.g., "ethereum", "solana")
        chain_id: Option<u64>,    // Chain ID for EVM chains
    },

    /// Accept a signing request
    AcceptSigning {
        signing_id: String,
    },

    /// Process a signing request from a device
    ProcessSigningRequest {
        from_device_id: String,
        signing_id: String,
        transaction_data: String,
        timestamp: String,
        blockchain: String,
        chain_id: Option<u64>,
    },

    /// Process signing acceptance from a device
    ProcessSigningAcceptance {
        from_device_id: String,
        signing_id: String,
        timestamp: String,
    },

    /// Process signing commitment from a device (FROST Round 1)
    ProcessSigningCommitment {
        from_device_id: String,
        signing_id: String,
        commitment: frost_core::round1::SigningCommitments<C>,
    },

    /// Process signature share from a device (FROST Round 2)
    ProcessSignatureShare {
        from_device_id: String,
        signing_id: String,
        share: frost_core::round2::SignatureShare<C>,
    },

    /// Process aggregated signature result
    ProcessAggregatedSignature {
        from_device_id: String,
        signing_id: String,
        signature: Vec<u8>, // The final signature bytes
    },

    /// Process signer selection message
    ProcessSignerSelection {
        from_device_id: String,
        signing_id: String,
        selected_signers: Vec<Identifier<C>>,
    },

    /// Initiate FROST Round 1 commitment generation
    InitiateFrostRound1 {
        signing_id: String,
        transaction_data: String,
        selected_signers: Vec<Identifier<C>>,
    },
    
    // --- Offline Mode Commands --- (temporarily disabled for browser compatibility focus)
    // /// Toggle offline mode
    // OfflineMode {
    //     enabled: bool,
    // },
    
    // /// Create a signing request for offline distribution
    // CreateSigningRequest {
    //     wallet_id: String,
    //     message: String,
    //     transaction_hex: String,
    // },
    
    // /// Export signing request to file/SD card
    // ExportSigningRequest {
    //     session_id: String,
    //     output_path: String,
    // },
    
    // /// Import signing request from file/SD card
    // ImportSigningRequest {
    //     input_path: String,
    // },
    
    // /// Review a signing request
    // ReviewSigningRequest {
    //     session_id: String,
    // },
    
    // /// List offline sessions
    // ListOfflineSessions,
}

/// DKG status tracking enum
#[derive(Debug, PartialEq, Clone)]
pub enum DkgState {
    Idle,
    Round1InProgress, // Same as CommitmentsInProgress but with naming used in other files
    Round1Complete,   // All Round 1 packages received
    Round2InProgress, // Same as SharesInProgress but with naming used in other files
    Round2Complete,   // All Round 2 packages received
    Finalizing,
    Complete,
    Failed(String),
}

/// Mesh status tracking enum
#[derive(Debug, PartialEq, Clone)]
pub enum MeshStatus {
    Incomplete,
    PartiallyReady {
        ready_devices: HashSet<String>,
        total_devices: usize,
    },
    Ready,
}

/// Signing status tracking enum
#[derive(Debug, PartialEq, Clone)]
pub enum SigningState<C: Ciphersuite> {
    Idle,
    AwaitingAcceptance {
        signing_id: String,
        transaction_data: String,
        initiator: String,
        required_signers: usize,
        accepted_signers: HashSet<String>,
        blockchain: String,
        chain_id: Option<u64>,
    },
    CommitmentPhase {
        signing_id: String,
        transaction_data: String,
        selected_signers: Vec<Identifier<C>>,
        commitments: BTreeMap<Identifier<C>, frost_core::round1::SigningCommitments<C>>,
        own_commitment: Option<frost_core::round1::SigningCommitments<C>>,
        nonces: Option<frost_core::round1::SigningNonces<C>>,
        blockchain: String,
        chain_id: Option<u64>,
    },
    SharePhase {
        signing_id: String,
        transaction_data: String,
        selected_signers: Vec<Identifier<C>>,
        signing_package: Option<frost_core::SigningPackage<C>>,
        shares: BTreeMap<Identifier<C>, frost_core::round2::SignatureShare<C>>,
        own_share: Option<frost_core::round2::SignatureShare<C>>,
        blockchain: String,
        chain_id: Option<u64>,
    },
    Complete {
        signing_id: String,
        signature: Vec<u8>,
    },
    Failed {
        signing_id: String,
        reason: String,
    },
}

impl<C: Ciphersuite> SigningState<C> {
    pub fn display_status(&self) -> String {
        match self {
            SigningState::Idle => "Idle".to_string(),
            SigningState::AwaitingAcceptance {
                signing_id,
                required_signers,
                accepted_signers,
                ..
            } => {
                format!(
                    "Awaiting Acceptance ({}): {}/{} signers",
                    signing_id,
                    accepted_signers.len(),
                    required_signers
                )
            }
            SigningState::CommitmentPhase {
                signing_id,
                commitments,
                selected_signers,
                ..
            } => {
                format!(
                    "Commitment Phase ({}): {}/{} commitments",
                    signing_id,
                    commitments.len(),
                    selected_signers.len()
                )
            }
            SigningState::SharePhase {
                signing_id,
                shares,
                selected_signers,
                ..
            } => {
                format!(
                    "Share Phase ({}): {}/{} shares",
                    signing_id,
                    shares.len(),
                    selected_signers.len()
                )
            }
            SigningState::Complete { signing_id, .. } => {
                format!("Complete ({})", signing_id)
            }
            SigningState::Failed { signing_id, reason } => {
                format!("Failed ({}): {}", signing_id, reason)
            }
        }
    }

    /// Check if a signing process is currently active (not idle or complete)
    pub fn is_active(&self) -> bool {
        match self {
            SigningState::Idle => false,
            SigningState::Complete { .. } => false,
            SigningState::Failed { .. } => false,
            _ => true,
        }
    }

    pub fn get_signing_id(&self) -> Option<&str> {
        match self {
            SigningState::Idle => None,
            SigningState::AwaitingAcceptance { signing_id, .. }
            | SigningState::CommitmentPhase { signing_id, .. }
            | SigningState::SharePhase { signing_id, .. }
            | SigningState::Complete { signing_id, .. }
            | SigningState::Failed { signing_id, .. } => Some(signing_id),
        }
    }
}

// DkgStateDisplay trait - defines display behavior for DkgState
pub trait DkgStateDisplay {
    fn display_status(&self) -> String;
    fn is_active(&self) -> bool;
    fn is_completed(&self) -> bool;
}

// Export DkgStateDisplay as a type alias for use in other modules
pub type DkgStateDisplayString = String;

// Implement the trait for the imported DkgState
impl DkgStateDisplay for DkgState {
    fn display_status(&self) -> String {
        match self {
            DkgState::Idle => "Idle".to_string(),
            DkgState::Round1InProgress => "Round 1 In Progress".to_string(),
            DkgState::Round1Complete => "Round 1 Complete".to_string(),
            DkgState::Round2InProgress => "Round 2 In Progress".to_string(),
            DkgState::Round2Complete => "Round 2 Complete".to_string(),
            DkgState::Finalizing => "Finalizing".to_string(),
            DkgState::Complete => "DKG Complete".to_string(),
            DkgState::Failed(reason) => format!("Failed: {}", reason),
        }
    }

    fn is_active(&self) -> bool {
        matches!(
            self,
            DkgState::Round1InProgress
                | DkgState::Round1Complete
                | DkgState::Round2InProgress
                | DkgState::Round2Complete
                | DkgState::Finalizing
        )
    }

    fn is_completed(&self) -> bool {
        matches!(self, DkgState::Complete)
    }
}

// --- AppState Struct ---
#[derive(Clone)]
pub struct AppState<C: Ciphersuite> {
    pub device_id: String,
    pub devices: Vec<String>,
    pub log: Vec<String>,
    pub log_scroll: u16, // Add scroll state for the log
    pub session: Option<SessionInfo>,
    pub invites: Vec<SessionInfo>, // Store full SessionInfo for invites
    // WebRTC related state (needs TokioMutex for async access)
    pub device_connections: Arc<Mutex<HashMap<String, Arc<RTCPeerConnection>>>>,
    // TUI related state (can use StdMutex)
    pub device_statuses: HashMap<String, RTCPeerConnectionState>,
    pub reconnection_tracker: ReconnectionTracker,
    // Perfect Negotiation Flags
    pub making_offer: HashMap<String, bool>,
    pub pending_ice_candidates: HashMap<String, Vec<RTCIceCandidateInit>>,
    // --- DKG State ---
    pub dkg_state: DkgState,
    pub identifier_map: Option<BTreeMap<String, Identifier<C>>>, // device_id -> FROST Identifier
    // pub identifier_to_index_map: Option<BTreeMap<Identifier<C>, u16>>, // Removed field
    // Fix: Use proper round1::SecretPackage and round1::Package types
    pub dkg_part1_public_package: Option<round1::Package<C>>,
    pub dkg_part1_secret_package: Option<round1::SecretPackage<C>>,
    // Fix: Store received round1 packages with correct type
    pub received_dkg_packages: BTreeMap<Identifier<C>, round1::Package<C>>,
    pub round2_secret_package: Option<round2::SecretPackage<C>>, // Secret needed for Part 3
    // Fix: Store received round2 packages with correct type
    pub received_dkg_round2_packages: BTreeMap<Identifier<C>, round2::Package<C>>,
    pub key_package: Option<KeyPackage<C>>,
    pub group_public_key: Option<PublicKeyPackage<C>>, // Use PublicKeyPackage from frost_core
    // Add data channels mapping
    pub data_channels: HashMap<String, Arc<RTCDataChannel>>,
    // Legacy fields for backward compatibility
    pub solana_public_key: Option<String>,
    pub etherum_public_key: Option<String>,
    
    // Multi-blockchain support
    pub blockchain_addresses: Vec<crate::keystore::BlockchainInfo>,
    pub mesh_status: MeshStatus,
    // Buffer for MeshReady signals received before session is active
    pub pending_mesh_ready_signals: Vec<String>,
    // Explicit flag to track if THIS node has sent its own mesh ready signal
    pub own_mesh_ready_sent: bool,

    // --- Keystore State ---
    pub keystore: Option<Arc<crate::keystore::Keystore>>,
    pub current_wallet_id: Option<String>,

    // --- Signing State ---
    pub signing_state: SigningState<C>,
    
    // Track all pending signing requests (not just the active one)
    pub pending_signing_requests: Vec<PendingSigningRequest>,
    
    // --- Offline Mode --- (temporarily disabled for browser compatibility focus)
    pub offline_mode: bool,
    // pub offline_config: crate::offline::OfflineConfig,
    // pub offline_sessions: HashMap<String, crate::offline::OfflineSession>,
}

impl<C: Ciphersuite> AppState<C> {
    pub fn new() -> Self {
        Self {
            device_id: String::new(),
            devices: Vec::new(),
            log: Vec::new(),
            log_scroll: 0,
            session: None,
            invites: Vec::new(),
            device_connections: Arc::new(Mutex::new(HashMap::new())),
            device_statuses: HashMap::new(),
            reconnection_tracker: ReconnectionTracker::new(),
            making_offer: HashMap::new(),
            pending_ice_candidates: HashMap::new(),
            dkg_state: DkgState::Idle,
            identifier_map: None,
            dkg_part1_public_package: None,
            dkg_part1_secret_package: None,
            received_dkg_packages: BTreeMap::new(),
            round2_secret_package: None,
            received_dkg_round2_packages: BTreeMap::new(),
            key_package: None,
            group_public_key: None,
            data_channels: HashMap::new(),
            solana_public_key: None,
            etherum_public_key: None,
            blockchain_addresses: Vec::new(),
            mesh_status: MeshStatus::Incomplete,
            pending_mesh_ready_signals: Vec::new(),
            own_mesh_ready_sent: false,
            keystore: None,
            current_wallet_id: None,
            signing_state: SigningState::Idle,
            pending_signing_requests: Vec::new(),
            offline_mode: false,
        }
    }
}

// --- Reconnection Tracker ---
#[derive(Debug, Clone)]
pub struct ReconnectionTracker {
    attempts: HashMap<String, usize>,
    last_attempt: HashMap<String, Instant>,
    cooldown: Duration,
    max_attempts: usize,
}

impl ReconnectionTracker {
    pub fn new() -> Self {
        ReconnectionTracker {
            attempts: HashMap::new(),
            last_attempt: HashMap::new(),
            cooldown: Duration::from_secs(5), // Reduced from 10 to 5 seconds for faster recovery
            max_attempts: 10, // Increased from 5 to 10 for more persistent reconnection
        }
    }

    pub fn should_attempt(&mut self, device_id: &str) -> bool {
        let now = Instant::now();
        let attempts = self.attempts.entry(device_id.to_string()).or_insert(0);
        let last = self
            .last_attempt
            .entry(device_id.to_string())
            .or_insert_with(|| now - self.cooldown * 2); // Ensure first attempt is allowed

        // For first few attempts, retry quickly
        if *attempts < 3 {
            // Almost no cooldown for the first few attempts
            if now.duration_since(*last) < Duration::from_millis(500) {
                return false;
            }
        } else if *attempts >= self.max_attempts {
            // Use exponential backoff with a cap after max attempts
            let backoff = self
                .cooldown
                .mul_f32(1.5_f32.powi(*attempts as i32 - self.max_attempts as i32));
            let capped_backoff = std::cmp::min(backoff, Duration::from_secs(60)); // Cap at 1 minute

            if now.duration_since(*last) < capped_backoff {
                return false; // Still in cooldown
            }
        } else {
            // Linear backoff between the first few attempts and max attempts
            if now.duration_since(*last) < self.cooldown.mul_f32(*attempts as f32 / 2.0) {
                return false; // Still in cooldown
            }
        }

        *attempts += 1;
        *last = now;
        true
    }

    pub fn record_success(&mut self, device_id: &str) {
        self.attempts.remove(device_id);
        self.last_attempt.remove(device_id);
    }
}
