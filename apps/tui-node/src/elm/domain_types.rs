//! Domain Types - Strong typing instead of primitive strings
//!
//! This module provides domain-specific types to replace primitive strings,
//! ensuring type safety and preventing invalid states.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Wallet ID with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WalletId(String);

impl WalletId {
    /// Create a new wallet ID
    pub fn new(id: impl Into<String>) -> Result<Self, ValidationError> {
        let id = id.into();
        if id.is_empty() {
            return Err(ValidationError::Empty("Wallet ID cannot be empty".into()));
        }
        if !id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(ValidationError::InvalidFormat(
                "Wallet ID must contain only alphanumeric characters, hyphens, and underscores".into()
            ));
        }
        Ok(Self(id))
    }
    
    /// Generate a new random wallet ID
    pub fn generate() -> Self {
        Self(format!("wallet_{}", uuid::Uuid::new_v4()))
    }
    
    /// Get the inner string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for WalletId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Session ID with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(String);

impl SessionId {
    /// Create a new session ID
    pub fn new(id: impl Into<String>) -> Result<Self, ValidationError> {
        let id = id.into();
        if id.is_empty() {
            return Err(ValidationError::Empty("Session ID cannot be empty".into()));
        }
        // Session IDs are typically UUIDs or base64
        if id.len() < 8 {
            return Err(ValidationError::TooShort(
                "Session ID must be at least 8 characters".into()
            ));
        }
        Ok(Self(id))
    }
    
    /// Generate a new random session ID
    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
    
    /// Get the inner string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Peer ID for network participants
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PeerId(String);

impl PeerId {
    /// Create a new peer ID
    pub fn new(id: impl Into<String>) -> Result<Self, ValidationError> {
        let id = id.into();
        if id.is_empty() {
            return Err(ValidationError::Empty("Peer ID cannot be empty".into()));
        }
        Ok(Self(id))
    }
    
    /// Get the inner string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Device ID for identifying nodes
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeviceId(String);

impl DeviceId {
    /// Create a new device ID
    pub fn new(id: impl Into<String>) -> Result<Self, ValidationError> {
        let id = id.into();
        if id.is_empty() {
            return Err(ValidationError::Empty("Device ID cannot be empty".into()));
        }
        if id.len() > 64 {
            return Err(ValidationError::TooLong(
                "Device ID must be 64 characters or less".into()
            ));
        }
        Ok(Self(id))
    }
    
    /// Generate from hostname
    pub fn from_hostname() -> Result<Self, ValidationError> {
        let hostname = hostname::get()
            .map_err(|_| ValidationError::SystemError("Failed to get hostname".into()))?
            .to_string_lossy()
            .to_string();
        Self::new(hostname)
    }
    
    /// Get the inner string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DeviceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Blockchain address with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Address {
    chain: Chain,
    value: String,
}

impl Address {
    /// Create a new address
    pub fn new(chain: Chain, value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into();
        
        // Validate based on chain
        match chain {
            Chain::Ethereum => {
                if !value.starts_with("0x") || value.len() != 42 {
                    return Err(ValidationError::InvalidFormat(
                        "Ethereum address must start with 0x and be 42 characters".into()
                    ));
                }
                // Check if it's valid hex
                if !value[2..].chars().all(|c| c.is_ascii_hexdigit()) {
                    return Err(ValidationError::InvalidFormat(
                        "Ethereum address must be valid hexadecimal".into()
                    ));
                }
            }
            Chain::Bitcoin => {
                // Basic Bitcoin address validation (simplified)
                if value.len() < 26 || value.len() > 35 {
                    return Err(ValidationError::InvalidFormat(
                        "Bitcoin address must be between 26 and 35 characters".into()
                    ));
                }
            }
            Chain::Solana => {
                // Solana addresses are base58 encoded
                if value.len() < 32 || value.len() > 44 {
                    return Err(ValidationError::InvalidFormat(
                        "Solana address must be between 32 and 44 characters".into()
                    ));
                }
            }
        }
        
        Ok(Self { chain, value })
    }
    
    /// Get the chain
    pub fn chain(&self) -> &Chain {
        &self.chain
    }
    
    /// Get the address value
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// Supported blockchain chains
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Chain {
    Ethereum,
    Bitcoin,
    Solana,
}

impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Chain::Ethereum => write!(f, "Ethereum"),
            Chain::Bitcoin => write!(f, "Bitcoin"),
            Chain::Solana => write!(f, "Solana"),
        }
    }
}

/// Threshold configuration with validation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThresholdConfig {
    threshold: u16,
    participants: u16,
}

impl ThresholdConfig {
    /// Create a new threshold configuration
    pub fn new(threshold: u16, participants: u16) -> Result<Self, ValidationError> {
        if threshold == 0 {
            return Err(ValidationError::InvalidValue(
                "Threshold must be at least 1".into()
            ));
        }
        if participants == 0 {
            return Err(ValidationError::InvalidValue(
                "Participants must be at least 1".into()
            ));
        }
        if threshold > participants {
            return Err(ValidationError::InvalidValue(
                "Threshold cannot be greater than participants".into()
            ));
        }
        if participants > 100 {
            return Err(ValidationError::TooLarge(
                "Maximum 100 participants supported".into()
            ));
        }
        
        Ok(Self {
            threshold,
            participants,
        })
    }
    
    /// Get the threshold
    pub fn threshold(&self) -> u16 {
        self.threshold
    }
    
    /// Get the number of participants
    pub fn participants(&self) -> u16 {
        self.participants
    }
    
    /// Check if this is a valid signing quorum size
    pub fn is_valid_quorum(&self, quorum_size: usize) -> bool {
        quorum_size >= self.threshold as usize && quorum_size <= self.participants as usize
    }
}

impl fmt::Display for ThresholdConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-of-{}", self.threshold, self.participants)
    }
}

/// WebSocket URL with validation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebSocketUrl(String);

impl WebSocketUrl {
    /// Create a new WebSocket URL
    pub fn new(url: impl Into<String>) -> Result<Self, ValidationError> {
        let url = url.into();
        
        if !url.starts_with("ws://") && !url.starts_with("wss://") {
            return Err(ValidationError::InvalidFormat(
                "WebSocket URL must start with ws:// or wss://".into()
            ));
        }
        
        // Basic URL validation
        if url.len() < 10 {
            return Err(ValidationError::TooShort(
                "WebSocket URL too short".into()
            ));
        }
        
        Ok(Self(url))
    }
    
    /// Get the URL string
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    /// Check if using secure WebSocket
    pub fn is_secure(&self) -> bool {
        self.0.starts_with("wss://")
    }
}

impl fmt::Display for WebSocketUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for WebSocketUrl {
    fn default() -> Self {
        Self("wss://auto-life.tech".to_string())
    }
}

/// Wallet name with validation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalletName(String);

impl WalletName {
    /// Create a new wallet name
    pub fn new(name: impl Into<String>) -> Result<Self, ValidationError> {
        let name = name.into().trim().to_string();
        
        if name.is_empty() {
            return Err(ValidationError::Empty("Wallet name cannot be empty".into()));
        }
        
        if name.len() > 50 {
            return Err(ValidationError::TooLong(
                "Wallet name must be 50 characters or less".into()
            ));
        }
        
        if name.chars().any(|c| c.is_control()) {
            return Err(ValidationError::InvalidFormat(
                "Wallet name cannot contain control characters".into()
            ));
        }
        
        Ok(Self(name))
    }
    
    /// Get the name string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for WalletName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Password with validation and security checks
#[derive(Clone, Serialize, Deserialize)]
pub struct Password(String);

impl Password {
    /// Create a new password with validation
    pub fn new(password: impl Into<String>) -> Result<Self, ValidationError> {
        let password = password.into();
        
        if password.len() < 8 {
            return Err(ValidationError::TooShort(
                "Password must be at least 8 characters".into()
            ));
        }
        
        if password.len() > 128 {
            return Err(ValidationError::TooLong(
                "Password must be 128 characters or less".into()
            ));
        }
        
        // Check password strength
        let has_upper = password.chars().any(|c| c.is_uppercase());
        let has_lower = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        let has_special = password.chars().any(|c| !c.is_alphanumeric());
        
        let strength_score = [has_upper, has_lower, has_digit, has_special]
            .iter()
            .filter(|&&x| x)
            .count();
        
        if strength_score < 3 {
            return Err(ValidationError::Weak(
                "Password must contain at least 3 of: uppercase, lowercase, digit, special character".into()
            ));
        }
        
        Ok(Self(password))
    }
    
    /// Get the password (use carefully)
    pub fn expose_secret(&self) -> &str {
        &self.0
    }
}

// Don't accidentally log passwords
impl fmt::Debug for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Password(***)")
    }
}

// Zero out password on drop
impl Drop for Password {
    fn drop(&mut self) {
        // Clear the password from memory
        let bytes = unsafe {
            self.0.as_bytes_mut()
        };
        for byte in bytes {
            *byte = 0;
        }
    }
}

/// Transaction hash
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TransactionHash(String);

impl TransactionHash {
    /// Create a new transaction hash
    pub fn new(hash: impl Into<String>) -> Result<Self, ValidationError> {
        let hash = hash.into();
        
        // Remove 0x prefix if present
        let hash = if hash.starts_with("0x") {
            &hash[2..]
        } else {
            &hash
        };
        
        // Check if valid hex and correct length (32 bytes = 64 hex chars)
        if hash.len() != 64 {
            return Err(ValidationError::InvalidFormat(
                "Transaction hash must be 32 bytes (64 hex characters)".into()
            ));
        }
        
        if !hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ValidationError::InvalidFormat(
                "Transaction hash must be valid hexadecimal".into()
            ));
        }
        
        Ok(Self(format!("0x{}", hash)))
    }
    
    /// Get the hash string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TransactionHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Validation errors for domain types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationError {
    Empty(String),
    TooShort(String),
    TooLong(String),
    TooLarge(String),
    InvalidFormat(String),
    InvalidValue(String),
    Weak(String),
    SystemError(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty(msg) => write!(f, "Empty value: {}", msg),
            Self::TooShort(msg) => write!(f, "Too short: {}", msg),
            Self::TooLong(msg) => write!(f, "Too long: {}", msg),
            Self::TooLarge(msg) => write!(f, "Too large: {}", msg),
            Self::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            Self::InvalidValue(msg) => write!(f, "Invalid value: {}", msg),
            Self::Weak(msg) => write!(f, "Too weak: {}", msg),
            Self::SystemError(msg) => write!(f, "System error: {}", msg),
        }
    }
}

impl std::error::Error for ValidationError {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wallet_id() {
        assert!(WalletId::new("wallet_123").is_ok());
        assert!(WalletId::new("").is_err());
        assert!(WalletId::new("wallet@123").is_err());
        
        let generated = WalletId::generate();
        assert!(generated.as_str().starts_with("wallet_"));
    }
    
    #[test]
    fn test_threshold_config() {
        assert!(ThresholdConfig::new(2, 3).is_ok());
        assert!(ThresholdConfig::new(0, 3).is_err());
        assert!(ThresholdConfig::new(4, 3).is_err());
        assert!(ThresholdConfig::new(50, 101).is_err());
        
        let config = ThresholdConfig::new(2, 3).unwrap();
        assert_eq!(config.to_string(), "2-of-3");
        assert!(config.is_valid_quorum(2));
        assert!(config.is_valid_quorum(3));
        assert!(!config.is_valid_quorum(1));
        assert!(!config.is_valid_quorum(4));
    }
    
    #[test]
    fn test_address() {
        // Ethereum
        let eth = Address::new(
            Chain::Ethereum,
            "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb4"
        );
        assert!(eth.is_ok());
        
        let bad_eth = Address::new(Chain::Ethereum, "not_an_address");
        assert!(bad_eth.is_err());
        
        // Bitcoin (simplified validation)
        let btc = Address::new(
            Chain::Bitcoin,
            "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"
        );
        assert!(btc.is_ok());
    }
    
    #[test]
    fn test_password() {
        assert!(Password::new("weakpass").is_err());
        assert!(Password::new("Str0ng!Pass").is_ok());
        assert!(Password::new("a").is_err());
        
        let pass = Password::new("MyP@ssw0rd").unwrap();
        assert_eq!(format!("{:?}", pass), "Password(***)");
    }
    
    #[test]
    fn test_websocket_url() {
        assert!(WebSocketUrl::new("wss://example.com").is_ok());
        assert!(WebSocketUrl::new("https://example.com").is_err());
        
        let url = WebSocketUrl::new("wss://secure.com").unwrap();
        assert!(url.is_secure());
        
        let url = WebSocketUrl::new("ws://insecure.com").unwrap();
        assert!(!url.is_secure());
    }
}