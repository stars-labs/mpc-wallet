//! Data models for the keystore module.
//!
//! This module defines the data structures used by the keystore, including
//! wallet information, device metadata, and key packages.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::keystore::KEYSTORE_VERSION;

/// Gets the current Unix timestamp in seconds
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}


/// Information about a blockchain supported by a wallet
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BlockchainInfo {
    /// Blockchain identifier (e.g., "ethereum", "bsc", "polygon", "solana")
    pub blockchain: String,
    
    /// Network type (e.g., "mainnet", "testnet", "devnet")
    pub network: String,
    
    /// Chain ID for EVM-compatible chains
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<u64>,
    
    /// Address on this blockchain
    pub address: String,
    
    /// Address format/encoding (e.g., "EIP-55", "base58", "bech32")
    pub address_format: String,
    
    /// Whether this blockchain is actively used
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    
    /// Optional custom RPC endpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rpc_endpoint: Option<String>,
    
    /// Additional metadata specific to this blockchain
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

fn default_enabled() -> bool {
    true
}

/// Information about a wallet stored in the keystore
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WalletInfo {
    /// Unique identifier for this wallet (UUID)
    pub wallet_id: String,

    /// User-friendly name for the wallet
    pub name: String,

    /// Type of cryptographic curve used ("secp256k1" or "ed25519")
    pub curve_type: String,

    /// List of blockchains supported by this wallet
    pub blockchains: Vec<BlockchainInfo>,

    /// Legacy fields for backward compatibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blockchain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_address: Option<String>,

    /// Minimum number of participants required to sign (threshold)
    pub threshold: u16,

    /// Total number of participants in the wallet
    pub total_participants: u16,

    /// Unix timestamp when the wallet was created
    pub created_at: u64,

    /// Serialized group public key for this wallet
    pub group_public_key: String,

    /// Devices that have shares for this wallet
    pub devices: Vec<DeviceInfo>,

    /// User-defined tags for organizing wallets
    pub tags: Vec<String>,

    /// Optional description for the wallet
    pub description: Option<String>,
}

impl WalletInfo {
    /// Creates a new wallet info with multiple blockchain support
    pub fn new_multi_chain(
        wallet_id: String,
        name: String,
        curve_type: String,
        blockchains: Vec<BlockchainInfo>,
        threshold: u16,
        total_participants: u16,
        group_public_key: String,
        tags: Vec<String>,
        description: Option<String>,
    ) -> Self {
        Self {
            wallet_id,
            name,
            curve_type,
            blockchains,
            blockchain: None,
            public_address: None,
            threshold,
            total_participants,
            created_at: current_timestamp(),
            group_public_key,
            devices: Vec::new(),
            tags,
            description,
        }
    }

    /// Creates a new wallet info (legacy single blockchain)
    pub fn new(
        wallet_id: String,
        name: String,
        curve_type: String,
        blockchain: String,
        public_address: String,
        threshold: u16,
        total_participants: u16,
        group_public_key: String,
        tags: Vec<String>,
        description: Option<String>,
    ) -> Self {
        // Create BlockchainInfo from legacy fields
        let blockchain_info = BlockchainInfo {
            blockchain: blockchain.clone(),
            network: "mainnet".to_string(),
            chain_id: if blockchain == "ethereum" { Some(1) } else { None },
            address: public_address,
            address_format: if blockchain == "ethereum" { "EIP-55".to_string() } else { "base58".to_string() },
            enabled: true,
            rpc_endpoint: None,
            metadata: None,
        };

        Self::new_multi_chain(
            wallet_id,
            name,
            curve_type,
            vec![blockchain_info],
            threshold,
            total_participants,
            group_public_key,
            tags,
            description,
        )
    }

    /// Gets the primary blockchain (first enabled blockchain)
    pub fn primary_blockchain(&self) -> Option<&BlockchainInfo> {
        self.blockchains.iter().find(|b| b.enabled)
    }

    /// Gets a blockchain by name
    pub fn get_blockchain(&self, blockchain: &str) -> Option<&BlockchainInfo> {
        self.blockchains.iter().find(|b| b.blockchain == blockchain)
    }

    /// Adds a device to this wallet
    pub fn add_device(&mut self, device: DeviceInfo) {
        // Replace if the device ID already exists, otherwise add
        if let Some(idx) = self
            .devices
            .iter()
            .position(|d| d.device_id == device.device_id)
        {
            self.devices[idx] = device;
        } else {
            self.devices.push(device);
        }
    }
}

/// Information about a device that can participate in signing
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeviceInfo {
    /// Unique identifier for this device
    pub device_id: String,

    /// User-friendly name for the device
    pub name: String,

    /// Serialized FROST identifier
    pub identifier: String,

    /// Last time this device was seen/used
    pub last_seen: u64,
}

impl DeviceInfo {
    /// Creates a new device info
    pub fn new(device_id: String, name: String, identifier: String) -> Self {
        Self {
            device_id,
            name,
            identifier,
            last_seen: current_timestamp(),
        }
    }

}

/// Metadata embedded within each wallet file
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WalletMetadata {
    /// Session ID from DKG that created this wallet
    #[serde(alias = "wallet_id")] // For backward compatibility
    pub session_id: String,
    
    /// Device ID that owns this key share
    pub device_id: String,
    
    /// User-friendly device name (deprecated, use device_id)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_name: Option<String>,
    
    /// Type of cryptographic curve used ("secp256k1" or "ed25519")
    pub curve_type: String,
    
    /// List of blockchains supported by this wallet
    pub blockchains: Vec<BlockchainInfo>,
    
    /// Legacy fields for backward compatibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blockchain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_address: Option<String>,
    
    /// Minimum number of participants required to sign
    pub threshold: u16,
    
    /// Total number of participants
    pub total_participants: u16,
    
    /// This device's participant index (1-based: 1, 2, 3, etc.)
    pub participant_index: u16,
    
    /// This device's identifier (deprecated, use device_id)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,
    
    /// Serialized group public key
    pub group_public_key: String,
    
    /// ISO 8601 timestamp when created
    pub created_at: String,
    
    /// ISO 8601 timestamp when last modified
    pub last_modified: String,
    
    /// User-defined tags (deprecated, redundant with curve_type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    
    /// Optional description (deprecated, redundant with created_at)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Self-contained wallet file format
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct WalletFile {
    /// Format version
    pub version: String,
    
    /// Whether the data is encrypted
    pub encrypted: bool,
    
    /// Encryption algorithm used (e.g., "AES-256-GCM-Argon2id" or "AES-256-GCM-PBKDF2")
    pub algorithm: String,
    
    /// Base64-encoded encrypted data
    pub data: String,
    
    /// Embedded metadata
    pub metadata: WalletMetadata,
}

/// Master index of all wallets and devices (legacy - for migration only)
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct KeystoreIndex {
    /// Keystore format version
    pub version: u8,

    /// List of all wallets
    pub wallets: Vec<WalletInfo>,

    /// List of all devices
    pub devices: Vec<DeviceInfo>,
}

impl KeystoreIndex {
    /// Creates a new, empty keystore index
    pub fn new() -> Self {
        Self {
            version: KEYSTORE_VERSION,
            wallets: Vec::new(),
            devices: Vec::new(),
        }
    }

    /// Adds or updates a wallet in the index
    pub fn add_wallet(&mut self, wallet: WalletInfo) {
        // Replace if wallet ID already exists, otherwise add
        if let Some(idx) = self
            .wallets
            .iter()
            .position(|w| w.wallet_id == wallet.wallet_id)
        {
            self.wallets[idx] = wallet;
        } else {
            self.wallets.push(wallet);
        }
    }

    /// Adds or updates a device in the index
    pub fn add_device(&mut self, device: DeviceInfo) {
        // Replace if device ID already exists, otherwise add
        if let Some(idx) = self
            .devices
            .iter()
            .position(|d| d.device_id == device.device_id)
        {
            self.devices[idx] = device;
        } else {
            self.devices.push(device);
        }
    }

    /// Gets a wallet by ID
    pub fn get_wallet(&self, wallet_id: &str) -> Option<&WalletInfo> {
        self.wallets.iter().find(|w| w.wallet_id == wallet_id)
    }

    /// Gets a device by ID
    pub fn get_device(&self, device_id: &str) -> Option<&DeviceInfo> {
        self.devices.iter().find(|d| d.device_id == device_id)
    }
}
