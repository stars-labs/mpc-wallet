// AppState Compatibility Layer
// Temporary wrapper to help migrate from AppState to StateMachine

use std::sync::Arc;
use tokio::sync::Mutex;
use frost_core::Ciphersuite;
use crate::protocal::signal::SessionInfo;
use super::state::{DkgState, MeshStatus, SigningState};

/// Application state management
/// Central state container for the MPC wallet application
pub struct AppState<C: Ciphersuite> {
    pub device_id: String,
    pub signal_server_url: String,
    pub session: Option<SessionInfo>,
    pub keystore: Option<Arc<crate::keystore::Keystore>>,
    // Legacy fields for compatibility - adding comprehensive set
    pub blockchain_addresses: Vec<crate::keystore::BlockchainInfo>,
    pub solana_public_key: Option<String>,
    pub etherum_public_key: Option<String>,
    pub pending_signatures: usize,
    pub log: Vec<String>,
    pub devices: Vec<String>,
    pub invites: Vec<SessionInfo>,
    pub available_sessions: Vec<crate::protocal::signal::SessionAnnouncement>,
    pub joining_session_id: Option<String>,
    pub current_wallet_id: Option<String>,
    pub device_connections: Arc<tokio::sync::Mutex<std::collections::HashMap<String, Arc<webrtc::peer_connection::RTCPeerConnection>>>>,
    pub data_channels: std::collections::HashMap<String, Arc<webrtc::data_channel::RTCDataChannel>>,
    pub device_statuses: std::collections::HashMap<String, webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState>,
    pub pending_ice_candidates: std::collections::HashMap<String, Vec<webrtc::ice_transport::ice_candidate::RTCIceCandidateInit>>,
    pub making_offer: std::collections::HashMap<String, bool>,
    pub mesh_status: MeshStatus,
    pub dkg_state: DkgState,
    pub received_dkg_packages: std::collections::HashMap<String, Vec<u8>>,
    pub received_dkg_round2_packages: std::collections::HashMap<String, Vec<u8>>,
    pub webrtc_initiation_in_progress: bool,
    pub webrtc_initiation_started_at: Option<std::time::Instant>,
    pub signing_state: SigningState<C>,
    pub pending_signing_requests: Vec<super::state::PendingSigningRequest>,
    pub wallet_creation_progress: Option<crate::handlers::session_handler::WalletCreationProgress>,
    // Additional DKG and other fields
    pub reconnection_tracker: std::collections::HashMap<String, std::time::Instant>,
    pub dkg_part1_public_package: Option<Vec<u8>>,
    pub dkg_part1_secret_package: Option<Vec<u8>>,
    pub dkg_part2_secret_package: Option<Vec<u8>>,
    pub dkg_round1_packages: std::collections::BTreeMap<frost_core::Identifier<C>, frost_core::keys::dkg::round1::Package<C>>,
    pub dkg_round2_packages: std::collections::BTreeMap<frost_core::Identifier<C>, frost_core::keys::dkg::round2::Package<C>>,
    pub key_package: Option<frost_core::keys::KeyPackage<C>>,
    pub group_public_key: Option<frost_core::VerifyingKey<C>>,
    pub public_key_package: Option<frost_core::keys::PublicKeyPackage<C>>,
    pub frost_commitments: std::collections::BTreeMap<frost_core::Identifier<C>, frost_core::round1::SigningCommitments<C>>,
    pub frost_signature_shares: std::collections::BTreeMap<frost_core::Identifier<C>, frost_core::round2::SignatureShare<C>>,
    pub frost_nonces: Option<frost_core::round1::SigningNonces<C>>,
    // More compatibility fields
    pub identifier_map: Option<std::collections::HashMap<String, frost_core::Identifier<C>>>,
    pub offline_sessions: std::collections::HashMap<String, crate::offline::OfflineSession>,
    pub offline_config: Option<crate::offline::OfflineConfig>,
    pub log_scroll: usize,
    pub wallet_creation_config: Option<crate::handlers::session_handler::WalletSessionConfig>,
    pub round2_secret_package: Option<frost_core::keys::dkg::round2::SecretPackage<C>>,
    pub wallet_creation_mode: Option<crate::handlers::session_handler::WalletCreationMode>,
    pub wallet_creation_curve: Option<String>,
    pub pending_mesh_ready_signals: std::collections::HashSet<String>,
    // Additional fields for UI compatibility
    pub websocket_connected: bool,
    pub websocket_connecting: bool,
    pub websocket_reconnecting: bool,
    pub dkg_in_progress: bool, // Prevents duplicate DKG sessions
    pub selected_wallet: Option<String>,
    pub own_mesh_ready_sent: bool,
    pub dkg_mode: Option<crate::protocal::dkg::DkgMode>,
    pub offline_mode: bool,
    pub session_start_time: Option<std::time::Instant>,
    pub webrtc_pending_participants: Vec<String>,
    pub websocket_error: Option<String>,
    pub websocket_internal_cmd_tx: Option<tokio::sync::mpsc::UnboundedSender<super::state::InternalCommand<C>>>,
    // Alternative: string-based channel for WebSocket messages (avoids Send issues)
    pub websocket_msg_tx: Option<tokio::sync::mpsc::UnboundedSender<String>>,
}

impl<C: Ciphersuite + Send + Sync + 'static> AppState<C> 
where
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    pub fn new() -> Self {
        Self {
            device_id: String::new(),
            signal_server_url: String::new(),
            session: None,
            keystore: None,
            blockchain_addresses: Vec::new(),
            solana_public_key: None,
            etherum_public_key: None,
            pending_signatures: 0,
            log: Vec::new(),
            devices: Vec::new(),
            invites: Vec::new(),
            available_sessions: Vec::new(),
            joining_session_id: None,
            current_wallet_id: None,
            device_connections: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
            data_channels: std::collections::HashMap::new(),
            device_statuses: std::collections::HashMap::new(),
            pending_ice_candidates: std::collections::HashMap::new(),
            making_offer: std::collections::HashMap::new(),
            mesh_status: MeshStatus::Incomplete,
            dkg_state: DkgState::Idle,
            received_dkg_packages: std::collections::HashMap::new(),
            received_dkg_round2_packages: std::collections::HashMap::new(),
            webrtc_initiation_in_progress: false,
            webrtc_initiation_started_at: None,
            signing_state: SigningState::Idle,
            pending_signing_requests: Vec::new(),
            wallet_creation_progress: None,
            reconnection_tracker: std::collections::HashMap::new(),
            dkg_part1_public_package: None,
            dkg_part1_secret_package: None,
            dkg_part2_secret_package: None,
            dkg_round1_packages: std::collections::BTreeMap::new(),
            dkg_round2_packages: std::collections::BTreeMap::new(),
            key_package: None,
            group_public_key: None,
            public_key_package: None,
            frost_commitments: std::collections::BTreeMap::new(),
            frost_signature_shares: std::collections::BTreeMap::new(),
            frost_nonces: None,
            identifier_map: None,
            offline_sessions: std::collections::HashMap::new(),
            offline_config: None,
            log_scroll: 0,
            wallet_creation_config: None,
            round2_secret_package: None,
            wallet_creation_mode: None,
            wallet_creation_curve: None,
            pending_mesh_ready_signals: std::collections::HashSet::new(),
            websocket_connected: false,
            websocket_connecting: false,
            websocket_reconnecting: false,
            dkg_in_progress: false,
            selected_wallet: None,
            own_mesh_ready_sent: false,
            dkg_mode: None,
            offline_mode: false,
            session_start_time: None,
            webrtc_pending_participants: Vec::new(),
            websocket_error: None,
            websocket_internal_cmd_tx: None,
            websocket_msg_tx: None,
        }
    }
    
    pub fn with_device_id(device_id: String) -> Self {
        Self::with_device_id_and_server(device_id, String::new())
    }
    
    pub fn with_device_id_and_server(device_id: String, signal_server_url: String) -> Self {
        Self {
            device_id,
            signal_server_url,
            session: None,
            keystore: None,
            blockchain_addresses: Vec::new(),
            solana_public_key: None,
            etherum_public_key: None,
            pending_signatures: 0,
            log: Vec::new(),
            devices: Vec::new(),
            invites: Vec::new(),
            available_sessions: Vec::new(),
            joining_session_id: None,
            current_wallet_id: None,
            device_connections: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
            data_channels: std::collections::HashMap::new(),
            device_statuses: std::collections::HashMap::new(),
            pending_ice_candidates: std::collections::HashMap::new(),
            making_offer: std::collections::HashMap::new(),
            mesh_status: MeshStatus::Incomplete,
            dkg_state: DkgState::Idle,
            received_dkg_packages: std::collections::HashMap::new(),
            received_dkg_round2_packages: std::collections::HashMap::new(),
            webrtc_initiation_in_progress: false,
            webrtc_initiation_started_at: None,
            signing_state: SigningState::Idle,
            pending_signing_requests: Vec::new(),
            wallet_creation_progress: None,
            reconnection_tracker: std::collections::HashMap::new(),
            dkg_part1_public_package: None,
            dkg_part1_secret_package: None,
            dkg_part2_secret_package: None,
            dkg_round1_packages: std::collections::BTreeMap::new(),
            dkg_round2_packages: std::collections::BTreeMap::new(),
            key_package: None,
            group_public_key: None,
            public_key_package: None,
            frost_commitments: std::collections::BTreeMap::new(),
            frost_signature_shares: std::collections::BTreeMap::new(),
            frost_nonces: None,
            identifier_map: None,
            offline_sessions: std::collections::HashMap::new(),
            offline_config: None,
            log_scroll: 0,
            wallet_creation_config: None,
            round2_secret_package: None,
            wallet_creation_mode: None,
            wallet_creation_curve: None,
            pending_mesh_ready_signals: std::collections::HashSet::new(),
            websocket_connected: false,
            websocket_connecting: false,
            websocket_reconnecting: false,
            dkg_in_progress: false,
            selected_wallet: None,
            own_mesh_ready_sent: false,
            dkg_mode: None,
            offline_mode: false,
            session_start_time: None,
            webrtc_pending_participants: Vec::new(),
            websocket_error: None,
            websocket_internal_cmd_tx: None,
            websocket_msg_tx: None,
        }
    }
    
    /// Get DKG state from composite state
    pub async fn get_dkg_state(&self) -> DkgState {
        // Simplified implementation - just return the stored dkg_state
        self.dkg_state.clone()
    }
    
    /// Get mesh status from composite state
    pub async fn get_mesh_status(&self) -> MeshStatus {
        // Simplified implementation - just return the stored mesh_status
        self.mesh_status.clone()
    }
    
    /// Check if can start DKG
    pub async fn can_start_dkg(&self) -> bool {
        // Simple check based on mesh status
        matches!(self.mesh_status, MeshStatus::Ready)
    }
}

/// Create a Mutex-wrapped AppState for compatibility
pub fn create_legacy_appstate<C: Ciphersuite + Send + Sync + 'static>(device_id: String) -> Arc<Mutex<AppState<C>>> 
where
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    Arc::new(Mutex::new(AppState::with_device_id(device_id)))
}