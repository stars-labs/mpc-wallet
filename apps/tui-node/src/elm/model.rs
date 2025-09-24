//! Model - The application state
//!
//! The Model represents the complete state of the application following
//! the Elm Architecture pattern. All state is centralized here.

use crate::keystore::{Keystore, WalletMetadata};
use crate::protocal::signal::SessionInfo;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The complete application state
#[derive(Debug, Clone)]
pub struct Model {
    /// Core application state
    pub wallet_state: WalletState,
    pub network_state: NetworkState,
    pub ui_state: UIState,
    
    /// Navigation
    pub navigation_stack: Vec<Screen>,
    pub current_screen: Screen,
    
    /// Session management
    pub active_session: Option<SessionInfo>,
    pub pending_operations: Vec<Operation>,
    pub session_invites: Vec<SessionInfo>,
    
    /// User context
    pub selected_wallet: Option<String>,
    pub device_id: String,
    
    /// Application metadata
    pub app_version: String,
    pub last_saved: Option<DateTime<Utc>>,
}

impl Model {
    pub fn new(device_id: String) -> Self {
        Self {
            wallet_state: WalletState::default(),
            network_state: NetworkState::default(),
            ui_state: UIState::default(),
            navigation_stack: Vec::new(),
            current_screen: Screen::Welcome,
            active_session: None,
            pending_operations: Vec::new(),
            session_invites: Vec::new(),
            selected_wallet: None,
            device_id,
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            last_saved: None,
        }
    }
    
    /// Push a screen to the navigation stack
    pub fn push_screen(&mut self, screen: Screen) {
        self.navigation_stack.push(self.current_screen.clone());
        self.current_screen = screen;
    }
    
    /// Pop a screen from the navigation stack
    pub fn pop_screen(&mut self) -> bool {
        if let Some(prev_screen) = self.navigation_stack.pop() {
            self.current_screen = prev_screen;
            true
        } else {
            false
        }
    }
    
    /// Clear navigation stack and go to main menu
    pub fn go_home(&mut self) {
        self.navigation_stack.clear();
        self.current_screen = Screen::MainMenu;
    }
}

/// Wallet-related state
#[derive(Clone, Default)]
pub struct WalletState {
    pub wallets: Vec<WalletMetadata>,
    pub keystore_initialized: bool,
    pub keystore_path: String,
    pub keystore: Option<std::sync::Arc<Keystore>>,
    pub selected_wallet: Option<String>,
    pub creating_wallet: Option<CreateWalletState>,
}

// Manual Debug implementation for WalletState
impl std::fmt::Debug for WalletState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WalletState")
            .field("wallets", &self.wallets)
            .field("keystore_initialized", &self.keystore_initialized)
            .field("keystore_path", &self.keystore_path)
            .field("keystore", &self.keystore.is_some())  // Just show if present
            .field("selected_wallet", &self.selected_wallet)
            .field("creating_wallet", &self.creating_wallet)
            .finish()
    }
}

/// Network-related state
#[derive(Debug, Clone)]
pub struct NetworkState {
    pub connected: bool,
    pub peers: Vec<String>,
    pub websocket_url: String,
    pub connection_status: ConnectionStatus,
    pub last_ping: Option<DateTime<Utc>>,
    pub reconnect_attempts: u32,
    pub participant_webrtc_status: std::collections::HashMap<String, (bool, bool)>, // (webrtc_connected, data_channel_open)
}

impl Default for NetworkState {
    fn default() -> Self {
        Self {
            connected: false,
            peers: Vec::new(),
            websocket_url: "wss://auto-life.tech".to_string(),
            connection_status: ConnectionStatus::Disconnected,
            last_ping: None,
            reconnect_attempts: 0,
            participant_webrtc_status: std::collections::HashMap::new(),
        }
    }
}

/// UI-related state
#[derive(Debug, Clone)]
pub struct UIState {
    pub focus: ComponentId,
    pub modal: Option<Modal>,
    pub notifications: Vec<Notification>,
    pub input_buffer: String,
    pub scroll_position: u16,
    pub selected_indices: HashMap<ComponentId, usize>,
    pub error_message: Option<String>,
    pub success_message: Option<String>,
    pub is_busy: bool,
    pub progress: Option<ProgressInfo>,
    pub join_session_tab: usize, // 0 = DKG, 1 = Signing
}

impl Default for UIState {
    fn default() -> Self {
        Self {
            focus: ComponentId::MainMenu,
            modal: None,
            notifications: Vec::new(),
            input_buffer: String::new(),
            scroll_position: 0,
            selected_indices: HashMap::new(),
            error_message: None,
            success_message: None,
            is_busy: false,
            progress: None,
            join_session_tab: 0, // Default to DKG tab
        }
    }
}

/// Represents different screens in the application
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Screen {
    Welcome,
    MainMenu,
    
    // Wallet management
    CreateWallet(CreateWalletState),
    ManageWallets,
    WalletDetail { wallet_id: String },
    ImportWallet,
    ExportWallet { wallet_id: String },
    
    // DKG flow
    PathSelection,
    ModeSelection,
    CurveSelection,
    ThresholdConfig,
    TemplateSelection,
    WalletConfiguration(WalletConfig),
    DKGProgress { session_id: String },
    WalletComplete { wallet_id: String },
    
    // Session management
    JoinSession,
    SessionDetail { session_id: String },
    AcceptSession { sessions: Vec<SessionInfo> },
    
    // Signing flow
    SignTransaction { wallet_id: String },
    SigningProgress { request_id: String },
    SignatureComplete { signature: String },
    
    // Settings
    Settings,
    NetworkSettings,
    SecuritySettings,
    About,
}

/// State for wallet creation flow
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CreateWalletState {
    pub mode: Option<WalletMode>,
    pub curve: Option<CurveType>,
    pub template: Option<WalletTemplate>,
    pub custom_config: Option<WalletConfig>,
}

/// Wallet creation mode
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum WalletMode {
    #[default]
    Online,
    Offline,
    Hybrid,
}

/// Supported curve types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CurveType {
    #[default]
    Secp256k1,
    Ed25519,
}

/// Wallet templates
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalletTemplate {
    pub name: String,
    pub description: String,
    pub total_participants: u16,
    pub threshold: u16,
    pub security_level: String,
    pub use_case: String,
}

/// Wallet configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WalletConfig {
    pub name: String,
    pub total_participants: u16,
    pub threshold: u16,
    pub curve: CurveType,
    pub mode: WalletMode,
}

/// Component identifiers for focus management
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ComponentId {
    MainMenu,
    WalletList,
    SessionList,
    InputField,
    Modal,
    Notification,
    CreateWallet,
    ModeSelection,
    CurveSelection,
    ThresholdConfig,
    JoinSession,
    DKGProgress,
    Custom(String),
}

/// Modal dialog types
#[derive(Debug, Clone)]
pub enum Modal {
    Confirm {
        title: String,
        message: String,
        on_confirm: Box<Message>,
        on_cancel: Box<Message>,
    },
    Progress {
        title: String,
        message: String,
        progress: f32,
    },
    Error {
        title: String,
        message: String,
    },
    Success {
        title: String,
        message: String,
    },
    Input {
        title: String,
        prompt: String,
        default_value: String,
        on_submit: Box<fn(String) -> Message>,
    },
}

// Manual PartialEq implementation for Modal (ignoring function pointers and comparing f32)
impl PartialEq for Modal {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Modal::Confirm { title: t1, message: m1, .. }, 
             Modal::Confirm { title: t2, message: m2, .. }) => t1 == t2 && m1 == m2,
            (Modal::Progress { title: t1, message: m1, progress: p1 }, 
             Modal::Progress { title: t2, message: m2, progress: p2 }) => t1 == t2 && m1 == m2 && (p1 - p2).abs() < f32::EPSILON,
            (Modal::Error { title: t1, message: m1 }, 
             Modal::Error { title: t2, message: m2 }) => t1 == t2 && m1 == m2,
            (Modal::Success { title: t1, message: m1 }, 
             Modal::Success { title: t2, message: m2 }) => t1 == t2 && m1 == m2,
            (Modal::Input { title: t1, prompt: p1, default_value: d1, .. }, 
             Modal::Input { title: t2, prompt: p2, default_value: d2, .. }) => t1 == t2 && p1 == p2 && d1 == d2,
            _ => false,
        }
    }
}

/// Notification types
#[derive(Debug, Clone)]
pub struct Notification {
    pub id: String,
    pub text: String,
    pub kind: NotificationKind,
    pub timestamp: DateTime<Utc>,
    pub dismissible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotificationKind {
    Info,
    Success,
    Warning,
    Error,
}

/// Progress information for long-running operations
#[derive(Debug, Clone)]
pub struct ProgressInfo {
    pub operation: String,
    pub progress: f32,
    pub message: String,
    pub started_at: DateTime<Utc>,
    pub estimated_completion: Option<DateTime<Utc>>,
}

/// Connection status
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Failed(String),
}

/// Pending operations
#[derive(Debug, Clone)]
pub enum Operation {
    CreateWallet(WalletConfig),
    ImportWallet { path: String },
    ExportWallet { wallet_id: String, path: String },
    DeleteWallet { wallet_id: String },
    StartDKG { config: WalletConfig },
    JoinDKG { session_id: String },
    SignTransaction { wallet_id: String, data: Vec<u8> },
}

use crate::elm::message::Message;

/// Persistent state that can be saved/loaded
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentState {
    pub device_id: String,
    pub websocket_url: String,
    pub selected_wallet: Option<String>,
    pub keystore_path: String,
    pub last_screen: Screen,
}

impl Model {
    /// Convert to persistent state for saving
    pub fn to_persistent(&self) -> PersistentState {
        PersistentState {
            device_id: self.device_id.clone(),
            websocket_url: self.network_state.websocket_url.clone(),
            selected_wallet: self.selected_wallet.clone(),
            keystore_path: self.wallet_state.keystore_path.clone(),
            last_screen: self.current_screen.clone(),
        }
    }
    
    /// Create from persistent state
    pub fn from_persistent(state: PersistentState) -> Self {
        let mut model = Self::new(state.device_id);
        model.network_state.websocket_url = state.websocket_url;
        model.selected_wallet = state.selected_wallet;
        model.wallet_state.keystore_path = state.keystore_path;
        model.current_screen = Screen::MainMenu; // Always start at main menu
        model
    }
}