use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub websocket_url: String,
    pub data_dir: PathBuf,
    pub log_level: String,
    pub auto_connect: bool,
    pub default_threshold: u16,
    pub default_participants: u16,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            websocket_url: "wss://auto-life.tech".to_string(),
            data_dir: dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("mpc-wallet"),
            log_level: "info".to_string(),
            auto_connect: false,
            default_threshold: 2,
            default_participants: 3,
        }
    }
}

impl AppConfig {
    pub async fn load_or_create() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        if config_path.exists() {
            let config_content = tokio::fs::read_to_string(&config_path).await?;
            let config: AppConfig = toml::from_str(&config_content)?;
            Ok(config)
        } else {
            let config = AppConfig::default();
            config.save().await?;
            Ok(config)
        }
    }
    
    pub async fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        
        // Ensure the config directory exists
        if let Some(parent) = config_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        
        let config_content = toml::to_string_pretty(self)?;
        tokio::fs::write(&config_path, config_content).await?;
        
        Ok(())
    }
    
    fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
        
        Ok(config_dir.join("mpc-wallet").join("native-node.toml"))
    }
    
    pub fn ensure_data_dir(&self) -> Result<()> {
        std::fs::create_dir_all(&self.data_dir)?;
        Ok(())
    }
}