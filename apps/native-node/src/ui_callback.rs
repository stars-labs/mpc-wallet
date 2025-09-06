//! UICallback implementation for native Slint UI

use async_trait::async_trait;
use slint::{ComponentHandle, ModelRc, VecModel, Weak};
use std::sync::Arc;
use tokio::sync::Mutex;
use tui_node::core::{
    ConnectionInfo, ConnectionStatus, OperationMode, ParticipantInfo, ParticipantStatus,
    SDCardOperation, SessionInfo, SessionStatus, UICallback, WalletInfo,
};

use crate::slint_generatedMainWindow::{
    AppState, ConnectionInfo as SlintConnectionInfo, MainWindow, 
    Participant as SlintParticipant, SDCardOperation as SlintSDCardOperation,
    SessionInfo as SlintSessionInfo, WalletInfo as SlintWalletInfo,
};

/// Native UI callback implementation using Slint
pub struct NativeUICallback {
    window: Weak<MainWindow>,
}

impl NativeUICallback {
    pub fn new(window: Weak<MainWindow>) -> Self {
        Self { window }
    }
    
    /// Convert core ConnectionInfo to Slint ConnectionInfo
    fn to_slint_connection(conn: &ConnectionInfo) -> SlintConnectionInfo {
        SlintConnectionInfo {
            peer_id: conn.peer_id.clone().into(),
            status: match conn.status {
                ConnectionStatus::Connected => "connected".into(),
                ConnectionStatus::Connecting => "connecting".into(),
                ConnectionStatus::Disconnected => "disconnected".into(),
                ConnectionStatus::Failed => "failed".into(),
            },
            latency_ms: conn.latency_ms as i32,
            quality: conn.quality,
        }
    }
    
    /// Convert core WalletInfo to Slint WalletInfo
    fn to_slint_wallet(wallet: &WalletInfo) -> SlintWalletInfo {
        SlintWalletInfo {
            id: wallet.id.clone().into(),
            name: wallet.name.clone().into(),
            address: wallet.address.clone().into(),
            balance: wallet.balance.clone().into(),
            chain: wallet.chain.clone().into(),
            threshold: wallet.threshold.clone().into(),
        }
    }
    
    /// Convert core SessionInfo to Slint SessionInfo
    fn to_slint_session(session: &SessionInfo) -> SlintSessionInfo {
        SlintSessionInfo {
            session_id: session.session_id.clone().into(),
            initiator: session.initiator.clone().into(),
            participants: session.participants.len() as i32,
            threshold: format!("{}/{}", session.threshold.0, session.threshold.1).into(),
            status: match session.status {
                SessionStatus::Waiting => "waiting".into(),
                SessionStatus::InProgress => "in_progress".into(),
                SessionStatus::Completed => "completed".into(),
                SessionStatus::Failed => "failed".into(),
            },
            created_at: session.created_at.clone().into(),
        }
    }
    
    /// Convert core ParticipantInfo to Slint Participant
    fn to_slint_participant(participant: &ParticipantInfo) -> SlintParticipant {
        SlintParticipant {
            id: participant.id.clone().into(),
            name: participant.name.clone().into(),
            status: match participant.status {
                ParticipantStatus::Ready => "ready".into(),
                ParticipantStatus::Processing => "processing".into(),
                ParticipantStatus::Completed => "completed".into(),
                ParticipantStatus::Failed => "failed".into(),
                ParticipantStatus::Offline => "offline".into(),
            },
            round_completed: participant.round_completed as i32,
        }
    }
    
    /// Convert core SDCardOperation to Slint SDCardOperation
    fn to_slint_sd_operation(op: &SDCardOperation) -> SlintSDCardOperation {
        SlintSDCardOperation {
            operation_type: match op.operation_type {
                tui_node::core::SDOperationType::Export => "export".into(),
                tui_node::core::SDOperationType::Import => "import".into(),
            },
            data_type: op.data_type.clone().into(),
            participant: op.participant.clone().into(),
            timestamp: op.timestamp.clone().into(),
        }
    }
}

#[async_trait]
impl UICallback for NativeUICallback {
    async fn update_connection_status(&self, websocket: bool, webrtc: bool) {
        if let Some(window) = self.window.upgrade() {
            slint::invoke_from_event_loop(move || {
                let state = window.global::<AppState>();
                state.set_websocket_connected(websocket);
                state.set_webrtc_connected(webrtc);
            })
            .unwrap();
        }
    }
    
    async fn update_mesh_connections(&self, connections: Vec<ConnectionInfo>) {
        if let Some(window) = self.window.upgrade() {
            let slint_connections: Vec<SlintConnectionInfo> = connections
                .iter()
                .map(Self::to_slint_connection)
                .collect();
            
            slint::invoke_from_event_loop(move || {
                let state = window.global::<AppState>();
                let model = ModelRc::new(VecModel::from(slint_connections));
                state.set_mesh_connections(model);
            })
            .unwrap();
        }
    }
    
    async fn update_operation_mode(&self, mode: OperationMode) {
        if let Some(window) = self.window.upgrade() {
            let mode_str = match mode {
                OperationMode::Online => "online",
                OperationMode::Offline => "offline",
                OperationMode::Hybrid => "hybrid",
            };
            
            slint::invoke_from_event_loop(move || {
                let state = window.global::<AppState>();
                state.set_operation_mode(mode_str.into());
            })
            .unwrap();
        }
    }
    
    async fn update_wallets(&self, wallets: Vec<WalletInfo>) {
        if let Some(window) = self.window.upgrade() {
            let slint_wallets: Vec<SlintWalletInfo> = wallets
                .iter()
                .map(Self::to_slint_wallet)
                .collect();
            
            slint::invoke_from_event_loop(move || {
                let state = window.global::<AppState>();
                let model = ModelRc::new(VecModel::from(slint_wallets));
                state.set_wallets(model);
                state.set_has_keystore(!wallets.is_empty());
            })
            .unwrap();
        }
    }
    
    async fn update_active_wallet(&self, index: usize) {
        if let Some(window) = self.window.upgrade() {
            slint::invoke_from_event_loop(move || {
                let state = window.global::<AppState>();
                state.set_active_wallet_index(index as i32);
            })
            .unwrap();
        }
    }
    
    async fn update_available_sessions(&self, sessions: Vec<SessionInfo>) {
        if let Some(window) = self.window.upgrade() {
            let slint_sessions: Vec<SlintSessionInfo> = sessions
                .iter()
                .map(Self::to_slint_session)
                .collect();
            
            slint::invoke_from_event_loop(move || {
                let state = window.global::<AppState>();
                let model = ModelRc::new(VecModel::from(slint_sessions));
                state.set_available_sessions(model);
            })
            .unwrap();
        }
    }
    
    async fn update_active_session(&self, session: Option<SessionInfo>) {
        if let Some(window) = self.window.upgrade() {
            slint::invoke_from_event_loop(move || {
                let state = window.global::<AppState>();
                
                if let Some(session) = session {
                    let slint_session = Self::to_slint_session(&session);
                    state.set_active_session(slint_session);
                    state.set_has_active_session(true);
                } else {
                    state.set_has_active_session(false);
                }
            })
            .unwrap();
        }
    }
    
    async fn update_dkg_status(&self, active: bool, round: u8, progress: f32) {
        if let Some(window) = self.window.upgrade() {
            slint::invoke_from_event_loop(move || {
                let state = window.global::<AppState>();
                state.set_dkg_active(active);
                state.set_dkg_current_round(round as i32);
                state.set_dkg_progress(progress);
            })
            .unwrap();
        }
    }
    
    async fn update_dkg_participants(&self, participants: Vec<ParticipantInfo>) {
        if let Some(window) = self.window.upgrade() {
            let slint_participants: Vec<SlintParticipant> = participants
                .iter()
                .map(Self::to_slint_participant)
                .collect();
            
            slint::invoke_from_event_loop(move || {
                let state = window.global::<AppState>();
                let model = ModelRc::new(VecModel::from(slint_participants));
                state.set_dkg_participants(model);
            })
            .unwrap();
        }
    }
    
    async fn update_offline_status(&self, enabled: bool, sd_card_detected: bool) {
        if let Some(window) = self.window.upgrade() {
            slint::invoke_from_event_loop(move || {
                let state = window.global::<AppState>();
                state.set_offline_enabled(enabled);
                state.set_sd_card_detected(sd_card_detected);
            })
            .unwrap();
        }
    }
    
    async fn update_sd_operations(&self, operations: Vec<SDCardOperation>) {
        if let Some(window) = self.window.upgrade() {
            let slint_operations: Vec<SlintSDCardOperation> = operations
                .iter()
                .map(Self::to_slint_sd_operation)
                .collect();
            
            slint::invoke_from_event_loop(move || {
                let state = window.global::<AppState>();
                let model = ModelRc::new(VecModel::from(slint_operations));
                state.set_pending_sd_operations(model);
            })
            .unwrap();
        }
    }
    
    async fn show_message(&self, message: String, is_error: bool) {
        if let Some(window) = self.window.upgrade() {
            slint::invoke_from_event_loop(move || {
                let state = window.global::<AppState>();
                
                // Update status message
                state.set_status_message(message.clone().into());
                
                // Add to log messages
                let mut logs = state.get_log_messages().iter().collect::<Vec<_>>();
                let prefix = if is_error { "[ERROR] " } else { "[INFO] " };
                let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
                let log_entry = format!("{} {} {}", timestamp, prefix, message);
                logs.push(log_entry.into());
                
                // Keep only last 100 messages
                if logs.len() > 100 {
                    logs.drain(0..logs.len() - 100);
                }
                
                let model = ModelRc::new(VecModel::from(logs));
                state.set_log_messages(model);
            })
            .unwrap();
        }
    }
    
    async fn show_progress(&self, title: String, progress: f32) {
        if let Some(window) = self.window.upgrade() {
            slint::invoke_from_event_loop(move || {
                let state = window.global::<AppState>();
                state.set_status_message(format!("{}: {:.0}%", title, progress * 100.0).into());
            })
            .unwrap();
        }
    }
    
    async fn request_confirmation(&self, message: String) -> bool {
        // For now, auto-confirm. In a real implementation, this would show a dialog
        // and wait for user response
        true
    }
}