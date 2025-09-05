//! Message - All possible events in the application
//!
//! Messages represent all user actions, system events, and state transitions
//! that can occur in the application. They are the only way to trigger state changes.

use crate::elm::model::{Screen, WalletConfig, WalletMode, CurveType, WalletTemplate, Modal, NotificationKind, ComponentId};
use crate::protocal::signal::SessionInfo;
use crate::utils::state::PendingSigningRequest;

/// All possible messages in the application
#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    // Navigation messages
    Navigate(Screen),
    NavigateBack,
    NavigateHome,
    PushScreen(Screen),
    PopScreen,
    
    // Wallet management messages
    CreateWallet { config: WalletConfig },
    SelectWallet { wallet_id: String },
    ListWallets,
    WalletsLoaded { wallets: Vec<crate::keystore::WalletMetadata> },
    DeleteWallet { wallet_id: String },
    WalletDeleted { wallet_id: String },
    ExportWallet { wallet_id: String },
    WalletExported { wallet_id: String, path: String },
    ImportWallet { data: Vec<u8> },
    WalletImported { wallet_id: String },
    
    // Wallet creation flow
    SelectMode(WalletMode),
    SelectCurve(CurveType),
    SelectTemplate(WalletTemplate),
    SetWalletName(String),
    SetThreshold(u16),
    SetTotalParticipants(u16),
    ConfirmWalletCreation,
    
    // DKG operations
    InitiateDKG { params: DKGParams },
    JoinSession { session_id: String },
    SessionsLoaded { sessions: Vec<SessionInfo> },
    UpdateDKGProgress { round: DKGRound, progress: f32 },
    DKGComplete { result: DKGResult },
    DKGFailed { error: String },
    CancelDKG,
    
    // Signing operations
    InitiateSigning { request: SigningRequest },
    SigningRequestsLoaded { requests: Vec<PendingSigningRequest> },
    ApproveSignature { request_id: String },
    RejectSignature { request_id: String },
    UpdateSigningProgress { request_id: String, progress: f32 },
    SigningComplete { request_id: String, signature: Vec<u8> },
    SigningFailed { request_id: String, error: String },
    
    // Network events
    WebSocketConnected,
    WebSocketDisconnected,
    WebSocketError { error: String },
    PeerDiscovered { peer_id: String },
    PeerDisconnected { peer_id: String },
    NetworkMessage { from: String, data: Vec<u8> },
    ConnectionStatusChanged { connected: bool },
    
    // Keystore events
    KeystoreInitialized { path: String },
    KeystoreError { error: String },
    KeystoreLocked,
    KeystoreUnlocked,
    
    // UI events
    KeyPressed(crossterm::event::KeyEvent),
    FocusChanged { component: ComponentId },
    InputChanged { value: String },
    ScrollUp,
    ScrollDown,
    ScrollTo { position: u16 },
    SelectItem { index: usize },
    
    // Modal management
    ShowModal(Modal),
    CloseModal,
    ConfirmModal,
    CancelModal,
    ModalInputSubmitted { value: String },
    
    // Notifications
    ShowNotification { text: String, kind: NotificationKind },
    ClearNotification { id: String },
    ClearAllNotifications,
    
    // Progress updates
    StartProgress { operation: String, message: String },
    UpdateProgress { progress: f32, message: Option<String> },
    CompleteProgress,
    
    // Settings
    UpdateWebSocketUrl { url: String },
    UpdateDeviceId { device_id: String },
    SaveSettings,
    LoadSettings,
    SettingsLoaded { websocket_url: String, device_id: String },
    
    // System messages
    Initialize,
    Shutdown,
    Quit,
    Refresh,
    Error { message: String },
    Success { message: String },
    Warning { message: String },
    Info { message: String },
    
    // Command execution results
    CommandCompleted { command: String },
    CommandFailed { command: String, error: String },
    
    // Time-based events
    Tick,
    Heartbeat,
    
    // No operation
    None,
}

/// DKG parameters
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DKGParams {
    pub wallet_config: WalletConfig,
    pub session_id: Option<String>,
    pub coordinator: bool,
}

/// DKG round information
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DKGRound {
    Initialization,
    Round1,
    Round2,
    Finalization,
}

/// DKG result
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DKGResult {
    pub wallet_id: String,
    pub group_public_key: String,
    pub participant_index: u16,
    pub addresses: Vec<(String, String)>, // (chain, address)
}

/// Signing request
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SigningRequest {
    pub wallet_id: String,
    pub transaction_data: Vec<u8>,
    pub chain: String,
    pub metadata: Option<String>,
}

impl Message {
    /// Create a key pressed message from a key event
    pub fn from_key_event(key: crossterm::event::KeyEvent) -> Self {
        Message::KeyPressed(key)
    }
    
    /// Check if this is a navigation message
    pub fn is_navigation(&self) -> bool {
        matches!(
            self,
            Message::Navigate(_)
            | Message::NavigateBack
            | Message::NavigateHome
            | Message::PushScreen(_)
            | Message::PopScreen
        )
    }
    
    /// Check if this is an error message
    pub fn is_error(&self) -> bool {
        matches!(
            self,
            Message::Error { .. }
            | Message::DKGFailed { .. }
            | Message::SigningFailed { .. }
            | Message::WebSocketError { .. }
            | Message::KeystoreError { .. }
            | Message::CommandFailed { .. }
        )
    }
    
    /// Check if this is a success message
    pub fn is_success(&self) -> bool {
        matches!(
            self,
            Message::Success { .. }
            | Message::DKGComplete { .. }
            | Message::SigningComplete { .. }
            | Message::WalletImported { .. }
            | Message::WalletExported { .. }
            | Message::CommandCompleted { .. }
        )
    }
    
    // Removed from_global_key - using direct key handling in app.rs instead (KISS)
}

impl From<crossterm::event::KeyEvent> for Message {
    fn from(key: crossterm::event::KeyEvent) -> Self {
        Message::KeyPressed(key)
    }
}

impl Default for Message {
    fn default() -> Self {
        Message::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_message_is_navigation() {
        assert!(Message::NavigateBack.is_navigation());
        assert!(Message::Navigate(Screen::MainMenu).is_navigation());
        assert!(!Message::Quit.is_navigation());
        println!("✅ Navigation message detection works");
    }
    
    #[test]
    fn test_message_is_error() {
        assert!(Message::Error { message: "test".to_string() }.is_error());
        assert!(!Message::Success { message: "test".to_string() }.is_error());
        println!("✅ Error message detection works");
    }
    
    #[test]
    fn test_message_is_success() {
        assert!(Message::Success { message: "test".to_string() }.is_success());
        assert!(!Message::Error { message: "test".to_string() }.is_success());
        println!("✅ Success message detection works");
    }
}