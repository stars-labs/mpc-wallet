use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct AppState {
    // Device and connection info
    pub device_id: String,
    pub websocket_connected: bool,
    pub websocket_url: String,
    
    // Session management
    pub current_session: Option<SessionInfo>,
    pub session_invites: Vec<SessionInvite>,
    
    // Connected devices and WebRTC
    pub connected_devices: HashMap<String, DeviceInfo>,
    pub webrtc_connections: HashMap<String, WebRTCConnectionState>,
    
    // DKG state
    pub dkg_state: Option<DkgState>,
    pub dkg_progress: DkgProgress,
    
    // Keystore and wallet
    pub keystore_initialized: bool,
    pub current_wallet: Option<WalletInfo>,
    pub addresses: HashMap<String, String>, // blockchain -> address
    
    // Signing state
    pub signing_requests: Vec<SigningRequest>,
    pub current_signing: Option<SigningState>,
    
    // UI state
    pub log_messages: Vec<String>,
    pub ui_mode: UIMode,
}

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub session_id: String,
    pub total_participants: u16,
    pub threshold: u16,
    pub participants: Vec<String>,
    pub is_creator: bool,
    pub curve: String, // "secp256k1" or "ed25519"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInvite {
    pub session_id: String,
    pub from_device: String,
    pub total_participants: u16,
    pub threshold: u16,
    pub curve: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub device_id: String,
    pub connected: bool,
    pub webrtc_state: WebRTCConnectionState,
    pub last_seen: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WebRTCConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Failed,
}

#[derive(Debug, Clone)]
pub enum DkgState {
    NotStarted,
    Round1 { round1_data: Vec<u8> },
    Round2 { round2_data: Vec<u8> },
    Complete { keystore_data: mpc_wallet_frost_core::KeystoreData },
}

#[derive(Debug, Clone, Default)]
pub struct DkgProgress {
    pub round: u8,
    pub received_from: Vec<String>,
    pub total_expected: usize,
}

#[derive(Debug, Clone)]
pub struct WalletInfo {
    pub wallet_id: String,
    pub name: String,
    pub curve: String, // "secp256k1" or "ed25519"
    pub threshold: u16,
    pub total_participants: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningRequest {
    pub request_id: String,
    pub from_device: String,
    pub transaction_data: String,
    pub blockchain: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone)]
pub struct SigningState {
    pub request_id: String,
    pub phase: SigningPhase,
    pub participants: Vec<String>,
    pub commitments_received: HashMap<String, Vec<u8>>,
    pub shares_received: HashMap<String, Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SigningPhase {
    Commitment,
    Share,
    Complete,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UIMode {
    Normal,
    SessionProposal,
    SigningRequest,
    WalletList,
    Help,
}

impl AppState {
    pub fn new(device_id: String) -> Self {
        Self {
            device_id,
            websocket_connected: false,
            websocket_url: "wss://auto-life.tech".to_string(),
            current_session: None,
            session_invites: Vec::new(),
            connected_devices: HashMap::new(),
            webrtc_connections: HashMap::new(),
            dkg_state: None,
            dkg_progress: DkgProgress::default(),
            keystore_initialized: false,
            current_wallet: None,
            addresses: HashMap::new(),
            signing_requests: Vec::new(),
            current_signing: None,
            log_messages: Vec::new(),
            ui_mode: UIMode::Normal,
        }
    }
    
    pub fn add_log(&mut self, message: String) {
        let timestamp = chrono::Utc::now().format("%H:%M:%S");
        self.log_messages.push(format!("[{}] {}", timestamp, message));
        
        // Keep only last 100 messages
        if self.log_messages.len() > 100 {
            self.log_messages.remove(0);
        }
    }
    
    pub fn update_device_status(&mut self, device_id: String, connected: bool) {
        if let Some(device) = self.connected_devices.get_mut(&device_id) {
            device.connected = connected;
            device.last_seen = chrono::Utc::now().timestamp();
        } else {
            self.connected_devices.insert(
                device_id.clone(),
                DeviceInfo {
                    device_id,
                    connected,
                    webrtc_state: WebRTCConnectionState::Disconnected,
                    last_seen: chrono::Utc::now().timestamp(),
                },
            );
        }
    }
    
    pub fn update_webrtc_state(&mut self, device_id: &str, state: WebRTCConnectionState) {
        self.webrtc_connections.insert(device_id.to_string(), state.clone());
        if let Some(device) = self.connected_devices.get_mut(device_id) {
            device.webrtc_state = state;
        }
    }
    
    pub fn is_mesh_ready(&self) -> bool {
        if let Some(session) = &self.current_session {
            let connected_count = self.webrtc_connections
                .iter()
                .filter(|(_, state)| **state == WebRTCConnectionState::Connected)
                .count();
            
            connected_count + 1 >= session.total_participants as usize
        } else {
            false
        }
    }
}

pub type SharedAppState = Arc<Mutex<AppState>>;