use crate::config::AppConfig;
use crate::mpc_manager::MpcManager;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

pub struct MpcWalletApp {
    device_id: String,
    config: AppConfig,
    mpc_manager: Arc<Mutex<MpcManager>>,
    logs: Arc<Mutex<Vec<String>>>,
    websocket_connected: Arc<Mutex<bool>>,
}

impl MpcWalletApp {
    pub async fn new() -> Result<Self> {
        let config = AppConfig::load_or_create().await?;
        let device_id = format!("Device-{}", Uuid::new_v4().to_string()[..8].to_uppercase());
        
        let mpc_manager = Arc::new(Mutex::new(MpcManager::new(&device_id).await?));
        let logs = Arc::new(Mutex::new(Vec::new()));
        let websocket_connected = Arc::new(Mutex::new(false));

        Ok(Self {
            device_id,
            config,
            mpc_manager,
            logs,
            websocket_connected,
        })
    }

    pub fn get_device_id(&self) -> String {
        self.device_id.clone()
    }
    
    pub async fn get_config(&self) -> AppConfig {
        self.config.clone()
    }

    pub async fn add_log(&self, message: String) {
        let mut logs = self.logs.lock().await;
        let timestamp = chrono::Utc::now().format("%H:%M:%S");
        logs.push(format!("[{}] {}", timestamp, message));
        
        // Keep only last 100 log messages
        if logs.len() > 100 {
            logs.remove(0);
        }
    }

    pub async fn get_logs(&self) -> Vec<String> {
        self.logs.lock().await.clone()
    }

    pub async fn is_websocket_connected(&self) -> bool {
        *self.websocket_connected.lock().await
    }

    pub async fn connect_websocket(&self) -> Result<()> {
        self.add_log("Connecting to WebSocket server...".to_string()).await;
        
        let mut manager = self.mpc_manager.lock().await;
        manager.connect_websocket(&self.config.websocket_url).await?;
        
        *self.websocket_connected.lock().await = true;
        self.add_log("WebSocket connection established".to_string()).await;
        
        Ok(())
    }

    pub async fn create_session(&self, session_id: String, total: u16, threshold: u16) -> Result<()> {
        self.add_log(format!(
            "Creating session '{}' with {}/{} participants", 
            session_id, threshold, total
        )).await;
        
        let mut manager = self.mpc_manager.lock().await;
        manager.create_session(session_id, total, threshold).await?;
        
        self.add_log("Session creation request sent".to_string()).await;
        Ok(())
    }

    pub async fn join_session(&self, session_id: String) -> Result<()> {
        self.add_log(format!("Joining session '{}'", session_id)).await;
        
        let mut manager = self.mpc_manager.lock().await;
        manager.join_session(session_id).await?;
        
        self.add_log("Session join request sent".to_string()).await;
        Ok(())
    }

    pub async fn start_dkg(&self) -> Result<()> {
        self.add_log("Starting Distributed Key Generation...".to_string()).await;
        
        let mut manager = self.mpc_manager.lock().await;
        manager.start_dkg().await?;
        
        self.add_log("DKG process initiated".to_string()).await;
        Ok(())
    }

    pub async fn export_keystore(&self) -> Result<String> {
        self.add_log("Exporting keystore...".to_string()).await;
        
        let manager = self.mpc_manager.lock().await;
        let export_path = manager.export_keystore().await?;
        
        self.add_log(format!("Keystore exported successfully")).await;
        Ok(export_path)
    }

    pub async fn initiate_signing(&self, tx_data: String, blockchain: String) -> Result<()> {
        self.add_log(format!(
            "Initiating signing for {} transaction", 
            blockchain
        )).await;
        
        let mut manager = self.mpc_manager.lock().await;
        manager.initiate_signing(tx_data, blockchain).await?;
        
        self.add_log("Signing request sent to participants".to_string()).await;
        Ok(())
    }
}