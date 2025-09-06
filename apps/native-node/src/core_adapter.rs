//! Adapter to integrate TUI node's shared core with native UI

use slint::Weak;
use std::sync::Arc;
use tui_node::core::{
    connection_manager::ConnectionManager,
    dkg_manager::DkgManager,
    offline_manager::OfflineManager,
    session_manager::SessionManager,
    wallet_manager::WalletManager,
    CoreState, UICallback,
};

use crate::slint_generatedMainWindow::MainWindow;
use crate::ui_callback::NativeUICallback;

/// Core adapter that manages all the shared business logic
pub struct CoreAdapter {
    pub state: Arc<CoreState>,
    pub connection_manager: Arc<ConnectionManager>,
    pub session_manager: Arc<SessionManager>,
    pub dkg_manager: Arc<DkgManager>,
    pub wallet_manager: Arc<WalletManager>,
    pub offline_manager: Arc<OfflineManager>,
    ui_callback: Arc<dyn UICallback>,
}

impl CoreAdapter {
    /// Create new core adapter with native UI callback
    pub fn new(window: Weak<MainWindow>) -> Self {
        let state = Arc::new(CoreState::new());
        let ui_callback: Arc<dyn UICallback> = Arc::new(NativeUICallback::new(window));
        
        Self {
            connection_manager: Arc::new(ConnectionManager::new(state.clone(), ui_callback.clone())),
            session_manager: Arc::new(SessionManager::new(state.clone(), ui_callback.clone())),
            dkg_manager: Arc::new(DkgManager::new(state.clone(), ui_callback.clone())),
            wallet_manager: Arc::new(WalletManager::new(state.clone(), ui_callback.clone())),
            offline_manager: Arc::new(OfflineManager::new(state.clone(), ui_callback.clone())),
            state,
            ui_callback,
        }
    }
    
    /// Connect to WebSocket server
    pub async fn connect_websocket(&self, url: String) -> Result<(), String> {
        self.connection_manager
            .connect_websocket(url)
            .await
            .map_err(|e| e.to_string())
    }
    
    /// Create a new wallet
    pub async fn create_wallet(&self) -> Result<(), String> {
        // For demo, create with default parameters
        self.wallet_manager
            .create_wallet(
                "New Wallet".to_string(),
                2,
                vec!["Alice".to_string(), "Bob".to_string(), "Charlie".to_string()],
            )
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    
    /// Import wallet from file
    pub async fn import_wallet(&self) -> Result<(), String> {
        // In a real implementation, this would open a file dialog
        self.ui_callback
            .show_message("Import wallet feature coming soon".to_string(), false)
            .await;
        Ok(())
    }
    
    /// Export wallet to file
    pub async fn export_wallet(&self) -> Result<(), String> {
        // In a real implementation, this would open a save dialog
        self.ui_callback
            .show_message("Export wallet feature coming soon".to_string(), false)
            .await;
        Ok(())
    }
    
    /// Create a new session
    pub async fn create_session(&self) -> Result<(), String> {
        // Get device ID (would be from config in real app)
        let device_id = "native-node-001".to_string();
        
        self.session_manager
            .create_session(device_id, 2, 3)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    
    /// Join an existing session
    pub async fn join_session(&self, session_id: String) -> Result<(), String> {
        let device_id = "native-node-001".to_string();
        
        self.session_manager
            .join_session(session_id, device_id)
            .await
            .map_err(|e| e.to_string())
    }
    
    /// Leave current session
    pub async fn leave_session(&self) -> Result<(), String> {
        let device_id = "native-node-001".to_string();
        
        self.session_manager
            .leave_session(device_id)
            .await
            .map_err(|e| e.to_string())
    }
    
    /// Refresh available sessions
    pub async fn refresh_sessions(&self) -> Result<(), String> {
        self.session_manager
            .refresh_sessions()
            .await
            .map_err(|e| e.to_string())
    }
    
    /// Toggle offline mode
    pub async fn toggle_offline_mode(&self) -> Result<(), String> {
        self.offline_manager
            .toggle_offline_mode()
            .await
            .map_err(|e| e.to_string())
    }
    
    /// Start DKG process
    pub async fn start_dkg(&self) -> Result<(), String> {
        // Get active session
        let session = self.session_manager
            .get_active_session()
            .await
            .ok_or_else(|| "No active session".to_string())?;
        
        // Start DKG with session participants
        self.dkg_manager
            .start_dkg(session.threshold.0, session.participants)
            .await
            .map_err(|e| e.to_string())
    }
    
    /// Abort DKG process
    pub async fn abort_dkg(&self) -> Result<(), String> {
        self.dkg_manager
            .abort_dkg()
            .await
            .map_err(|e| e.to_string())
    }
    
    /// Initialize with demo data
    pub async fn initialize_demo(&self) {
        // Add some demo wallets
        let demo_wallets = vec![
            ("Demo Wallet 1", "0x1234...5678", "1.5 ETH"),
            ("Demo Wallet 2", "0xabcd...ef01", "0.8 ETH"),
        ];
        
        for (name, address, balance) in demo_wallets {
            let wallet = tui_node::core::WalletInfo {
                id: format!("wallet_{}", uuid::Uuid::new_v4()),
                name: name.to_string(),
                address: address.to_string(),
                balance: balance.to_string(),
                chain: "Ethereum".to_string(),
                threshold: "2/3".to_string(),
                participants: vec!["Alice".to_string(), "Bob".to_string(), "Charlie".to_string()],
            };
            
            self.state.wallets.lock().await.push(wallet);
        }
        
        // Update UI with demo data
        self.ui_callback
            .update_wallets(self.state.wallets.lock().await.clone())
            .await;
        
        // Add demo sessions
        let demo_sessions = vec![
            tui_node::core::SessionInfo {
                session_id: "demo-session-1".to_string(),
                initiator: "Alice".to_string(),
                participants: vec!["Alice".to_string()],
                threshold: (2, 3),
                status: tui_node::core::SessionStatus::Waiting,
                created_at: chrono::Utc::now().to_rfc3339(),
            },
            tui_node::core::SessionInfo {
                session_id: "demo-session-2".to_string(),
                initiator: "Bob".to_string(),
                participants: vec!["Bob".to_string(), "Charlie".to_string()],
                threshold: (2, 3),
                status: tui_node::core::SessionStatus::InProgress,
                created_at: chrono::Utc::now().to_rfc3339(),
            },
        ];
        
        *self.state.available_sessions.lock().await = demo_sessions.clone();
        
        self.ui_callback
            .update_available_sessions(demo_sessions)
            .await;
        
        self.ui_callback
            .show_message("Native MPC Wallet initialized with demo data".to_string(), false)
            .await;
    }
}