//! Split Model Architecture
//!
//! This module provides a cleaner separation of concerns by splitting
//! the monolithic Model into focused sub-models.

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use crate::elm::domain_types::ValidationError;

/// Root application model composed of sub-models
#[derive(Debug, Clone)]
pub struct AppModel {
    /// Core domain models
    pub wallet: WalletModel,
    pub network: NetworkModel,
    pub session: SessionModel,
    
    /// UI-specific models
    pub navigation: NavigationModel,
    pub ui: UIModel,
    
    /// Application metadata
    pub metadata: AppMetadata,
}

impl AppModel {
    /// Create a new application model
    pub fn new(device_id: String) -> Self {
        Self {
            wallet: WalletModel::default(),
            network: NetworkModel::default(),
            session: SessionModel::default(),
            navigation: NavigationModel::default(),
            ui: UIModel::default(),
            metadata: AppMetadata::new(device_id),
        }
    }
}

/// Wallet-related model
#[derive(Debug, Clone, Default)]
pub struct WalletModel {
    /// All wallets
    pub wallets: Vec<Wallet>,
    
    /// Currently selected wallet
    pub selected_wallet_id: Option<String>,
    
    /// Wallet creation in progress
    pub creation_state: Option<WalletCreationState>,
    
    /// Keystore status
    pub keystore: KeystoreState,
}

/// Individual wallet
#[derive(Debug, Clone)]
pub struct Wallet {
    pub id: String,
    pub name: String,
    pub curve: CurveType,
    pub threshold: u16,
    pub participants: u16,
    pub created_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    pub balance: Option<String>, // Balance amount as string for now
    pub addresses: HashMap<String, String>, // chain -> address
}

/// Wallet creation state
#[derive(Debug, Clone)]
pub struct WalletCreationState {
    pub step: CreationStep,
    pub config: WalletConfig,
    pub validation_errors: Vec<ValidationError>,
}

/// Wallet configuration being built
#[derive(Debug, Clone, Default)]
pub struct WalletConfig {
    pub name: Option<String>,
    pub mode: Option<WalletMode>,
    pub curve: Option<CurveType>,
    pub threshold: Option<u16>,
    pub participants: Option<u16>,
}

/// Creation steps
#[derive(Debug, Clone, PartialEq)]
pub enum CreationStep {
    SelectMode,
    SelectCurve,
    ConfigureThreshold,
    SetName,
    Review,
    Processing,
}

/// Keystore state
#[derive(Debug, Clone, Default)]
pub struct KeystoreState {
    pub initialized: bool,
    pub locked: bool,
    pub path: Option<String>,
    pub last_backup: Option<DateTime<Utc>>,
}

/// Network-related model
#[derive(Debug, Clone)]
pub struct NetworkModel {
    /// Connection state
    pub connection: ConnectionState,
    
    /// Connected peers
    pub peers: Vec<Peer>,
    
    /// Network configuration
    pub config: NetworkConfig,
    
    /// Network statistics
    pub stats: NetworkStats,
}

impl Default for NetworkModel {
    fn default() -> Self {
        Self {
            connection: ConnectionState::Disconnected,
            peers: Vec::new(),
            config: NetworkConfig::default(),
            stats: NetworkStats::default(),
        }
    }
}

/// Connection state
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connecting { since: DateTime<Utc> },
    Connected { since: DateTime<Utc> },
    Reconnecting { attempt: u32, next_retry: DateTime<Utc> },
    Failed { error: String, at: DateTime<Utc> },
}

/// Connected peer
#[derive(Debug, Clone)]
pub struct Peer {
    pub id: String,
    pub name: String,
    pub status: PeerStatus,
    pub connected_at: DateTime<Utc>,
    pub latency_ms: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PeerStatus {
    Connected,
    Ready,
    Busy,
    Disconnected,
}

/// Network configuration
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub websocket_url: String,
    pub enable_webrtc: bool,
    pub stun_servers: Vec<String>,
    pub turn_servers: Vec<String>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            websocket_url: "wss://auto-life.tech".to_string(),
            enable_webrtc: true,
            stun_servers: vec!["stun:stun.l.google.com:19302".to_string()],
            turn_servers: Vec::new(),
        }
    }
}

/// Network statistics
#[derive(Debug, Clone, Default)]
pub struct NetworkStats {
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub connection_uptime: Option<Duration>,
    pub reconnection_count: u32,
}

use std::time::Duration;

/// Session-related model
#[derive(Debug, Clone, Default)]
pub struct SessionModel {
    /// Active DKG session
    pub dkg_session: Option<DKGSession>,
    
    /// Active signing sessions
    pub signing_sessions: Vec<SigningSession>,
    
    /// Available sessions to join
    pub available_sessions: Vec<AvailableSession>,
    
    /// Session history
    pub history: Vec<SessionHistoryEntry>,
}

/// DKG session
#[derive(Debug, Clone)]
pub struct DKGSession {
    pub id: String,
    pub state: DKGState,
    pub participants: Vec<Participant>,
    pub config: DKGConfig,
    pub started_at: DateTime<Utc>,
    pub round_timeout: Duration,
}

/// DKG state
#[derive(Debug, Clone, PartialEq)]
pub enum DKGState {
    Initializing,
    Round1 { progress: f32 },
    Round2 { progress: f32 },
    Finalizing,
    Complete { wallet_id: String },
    Failed { error: String },
}

/// Signing session
#[derive(Debug, Clone)]
pub struct SigningSession {
    pub id: String,
    pub wallet_id: String,
    pub message: Vec<u8>,
    pub state: SigningState,
    pub participants_needed: u16,
    pub participants_signed: Vec<String>,
    pub deadline: Option<DateTime<Utc>>,
}

/// Signing state
#[derive(Debug, Clone, PartialEq)]
pub enum SigningState {
    WaitingForParticipants,
    CollectingSignatures { progress: f32 },
    Aggregating,
    Complete { signature: Vec<u8> },
    Failed { error: String },
}

/// Available session to join
#[derive(Debug, Clone)]
pub struct AvailableSession {
    pub id: String,
    pub session_type: SessionType,
    pub coordinator: String,
    pub participants_current: u16,
    pub participants_needed: u16,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SessionType {
    DKG,
    Signing,
}

/// Session history entry
#[derive(Debug, Clone)]
pub struct SessionHistoryEntry {
    pub id: String,
    pub session_type: SessionType,
    pub result: SessionResult,
    pub timestamp: DateTime<Utc>,
    pub duration: Duration,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SessionResult {
    Success,
    Failed { reason: String },
    Cancelled,
}

/// Participant in a session
#[derive(Debug, Clone)]
pub struct Participant {
    pub id: String,
    pub name: String,
    pub status: ParticipantStatus,
    pub share_index: Option<u16>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParticipantStatus {
    Waiting,
    Ready,
    Processing,
    Complete,
    Failed,
    Dropped,
}

/// DKG configuration
#[derive(Debug, Clone)]
pub struct DKGConfig {
    pub threshold: u16,
    pub participants: u16,
    pub curve: CurveType,
    pub mode: SessionMode,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SessionMode {
    Online,
    Offline,
}

/// Navigation model
#[derive(Debug, Clone)]
pub struct NavigationModel {
    /// Current screen
    pub current_screen: Screen,
    
    /// Navigation history stack
    pub history: Vec<Screen>,
    
    /// Maximum history depth
    pub max_history: usize,
    
    /// Breadcrumb trail
    pub breadcrumbs: Vec<Breadcrumb>,
}

impl Default for NavigationModel {
    fn default() -> Self {
        Self {
            current_screen: Screen::Welcome,
            history: Vec::new(),
            max_history: 10,
            breadcrumbs: vec![Breadcrumb::new("Home", Screen::Welcome)],
        }
    }
}

impl NavigationModel {
    /// Navigate to a screen
    pub fn navigate_to(&mut self, screen: Screen) {
        // Push current to history
        if self.history.len() >= self.max_history {
            self.history.remove(0);
        }
        self.history.push(self.current_screen.clone());
        
        // Update current
        self.current_screen = screen.clone();
        
        // Update breadcrumbs
        self.update_breadcrumbs();
    }
    
    /// Go back
    pub fn go_back(&mut self) -> bool {
        if let Some(previous) = self.history.pop() {
            self.current_screen = previous;
            self.update_breadcrumbs();
            true
        } else {
            false
        }
    }
    
    /// Update breadcrumb trail
    fn update_breadcrumbs(&mut self) {
        // Simplified breadcrumb logic
        self.breadcrumbs = vec![
            Breadcrumb::new("Home", Screen::Welcome),
            Breadcrumb::new(&self.current_screen.name(), self.current_screen.clone()),
        ];
    }
}

/// Breadcrumb for navigation trail
#[derive(Debug, Clone)]
pub struct Breadcrumb {
    pub label: String,
    pub screen: Screen,
}

impl Breadcrumb {
    pub fn new(label: impl Into<String>, screen: Screen) -> Self {
        Self {
            label: label.into(),
            screen,
        }
    }
}

/// UI model
#[derive(Debug, Clone, Default)]
pub struct UIModel {
    /// Active modals
    pub modal: Option<Modal>,
    
    /// Notifications
    pub notifications: NotificationQueue,
    
    /// Loading states
    pub loading: LoadingStates,
    
    /// Form states
    pub forms: FormStates,
    
    /// Component focus
    pub focus: FocusState,
    
    /// User preferences
    pub preferences: UIPreferences,
}

/// Modal state
#[derive(Debug, Clone)]
pub enum Modal {
    Confirm {
        title: String,
        message: String,
        on_confirm: ModalAction,
    },
    Error {
        error: String, // Error message
        recoverable: bool,
    },
    Progress {
        operation: String,
        progress: f32,
        cancelable: bool,
    },
    Input {
        prompt: String,
        validator: Option<fn(&str) -> bool>, // Optional validation function
    },
}

/// Modal action
#[derive(Debug, Clone)]
pub enum ModalAction {
    Delete { id: String },
    Confirm { action: String },
    Cancel,
}

/// Notification queue
#[derive(Debug, Clone, Default)]
pub struct NotificationQueue {
    pub items: Vec<Notification>,
    pub max_items: usize,
}

/// Notification
#[derive(Debug, Clone)]
pub struct Notification {
    pub id: String,
    pub level: NotificationLevel,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub auto_dismiss: Option<Duration>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NotificationLevel {
    Info,
    Success,
    Warning,
    Error,
}

/// Loading states for async operations
#[derive(Debug, Clone, Default)]
pub struct LoadingStates {
    pub operations: HashMap<String, LoadingState>,
}

/// Loading state
#[derive(Debug, Clone)]
pub enum LoadingState {
    Loading { since: DateTime<Utc>, message: String },
    Success { at: DateTime<Utc> },
    Failed { at: DateTime<Utc>, error: String },
}

/// Form states
#[derive(Debug, Clone, Default)]
pub struct FormStates {
    pub active_form: Option<String>,
    pub field_values: HashMap<String, String>,
    pub field_errors: HashMap<String, String>,
    pub form_valid: bool,
}

/// Focus state
#[derive(Debug, Clone, Default)]
pub struct FocusState {
    pub component: Option<ComponentId>,
    pub field: Option<String>,
    pub list_index: Option<usize>,
}

/// UI preferences
#[derive(Debug, Clone)]
pub struct UIPreferences {
    pub theme: Theme,
    pub vim_mode: bool,
    pub show_hints: bool,
    pub animations: bool,
}

impl Default for UIPreferences {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            vim_mode: true,
            show_hints: true,
            animations: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Theme {
    Light,
    Dark,
    HighContrast,
}

/// Application metadata
#[derive(Debug, Clone)]
pub struct AppMetadata {
    pub version: String,
    pub device_id: String,
    pub started_at: DateTime<Utc>,
    pub last_saved: Option<DateTime<Utc>>,
    pub session_id: String,
}

impl AppMetadata {
    pub fn new(device_id: String) -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            device_id,
            started_at: Utc::now(),
            last_saved: None,
            session_id: uuid::Uuid::new_v4().to_string(),
        }
    }
}

// Re-export types that are used elsewhere
pub use crate::elm::model::{
    Screen, ComponentId, CurveType, WalletMode,
};

impl Screen {
    /// Get the display name for this screen
    pub fn name(&self) -> String {
        match self {
            Screen::Welcome => "Welcome".to_string(),
            Screen::MainMenu => "Main Menu".to_string(),
            Screen::CreateWallet(_) => "Create Wallet".to_string(),
            Screen::ManageWallets => "Manage Wallets".to_string(),
            Screen::WalletDetail { .. } => "Wallet Detail".to_string(),
            Screen::ImportWallet => "Import Wallet".to_string(),
            Screen::ExportWallet { .. } => "Export Wallet".to_string(),
            Screen::PathSelection => "Path Selection".to_string(),
            Screen::ModeSelection => "Mode Selection".to_string(),
            Screen::CurveSelection => "Curve Selection".to_string(),
            Screen::ThresholdConfig => "Threshold Config".to_string(),
            Screen::TemplateSelection => "Template Selection".to_string(),
            Screen::WalletConfiguration(_) => "Wallet Configuration".to_string(),
            Screen::DKGProgress { .. } => "DKG Progress".to_string(),
            Screen::WalletComplete { .. } => "Wallet Complete".to_string(),
            Screen::JoinSession => "Join Session".to_string(),
            Screen::SessionDetail { .. } => "Session Detail".to_string(),
            Screen::AcceptSession { .. } => "Accept Session".to_string(),
            Screen::SignTransaction { .. } => "Sign Transaction".to_string(),
            Screen::SigningProgress { .. } => "Signing Progress".to_string(),
            Screen::SignatureComplete { .. } => "Signature Complete".to_string(),
            Screen::Settings => "Settings".to_string(),
            Screen::NetworkSettings => "Network Settings".to_string(),
            Screen::SecuritySettings => "Security Settings".to_string(),
            Screen::About => "About".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_navigation_model() {
        let mut nav = NavigationModel::default();
        
        assert_eq!(nav.current_screen, Screen::Welcome);
        
        nav.navigate_to(Screen::MainMenu);
        assert_eq!(nav.current_screen, Screen::MainMenu);
        assert_eq!(nav.history.len(), 1);
        
        nav.navigate_to(Screen::ManageWallets);
        assert_eq!(nav.current_screen, Screen::ManageWallets);
        
        assert!(nav.go_back());
        assert_eq!(nav.current_screen, Screen::MainMenu);
    }
    
    #[test]
    fn test_app_model_creation() {
        let model = AppModel::new("test-device".to_string());
        
        assert_eq!(model.metadata.device_id, "test-device");
        assert!(!model.wallet.keystore.initialized);
        assert_eq!(model.network.connection, ConnectionState::Disconnected);
    }
}