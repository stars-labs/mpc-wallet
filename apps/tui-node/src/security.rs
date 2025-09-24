//! Security constants and configuration for cryptographic operations
//! 
//! This module centralizes all security-related parameters to ensure
//! consistency and make security auditing easier.

use std::env;

/// Minimum recommended PBKDF2 iterations for 2024
/// Based on OWASP recommendations for password storage
pub const MIN_PBKDF2_ITERATIONS: u32 = 100_000;

/// Default PBKDF2 iterations if not configured
/// Using 210,000 for extra security margin
pub const DEFAULT_PBKDF2_ITERATIONS: u32 = 210_000;

/// Maximum PBKDF2 iterations to prevent DoS
pub const MAX_PBKDF2_ITERATIONS: u32 = 1_000_000;

/// Salt length in bytes (128 bits)
pub const SALT_LENGTH: usize = 16;

/// Key length in bytes (256 bits for AES-256)
pub const KEY_LENGTH: usize = 32;

/// Nonce length for AES-GCM (96 bits)
pub const NONCE_LENGTH: usize = 12;

/// Tag length for AES-GCM authentication (128 bits)
pub const TAG_LENGTH: usize = 16;

/// Memory cost for Argon2 (in KiB) - if we migrate to Argon2
pub const ARGON2_MEMORY_COST: u32 = 65536; // 64 MiB

/// Time cost for Argon2 (iterations)
pub const ARGON2_TIME_COST: u32 = 3;

/// Parallelism for Argon2
pub const ARGON2_PARALLELISM: u32 = 4;

/// Maximum notification list size to prevent unbounded growth
pub const MAX_NOTIFICATIONS: usize = 50;

/// Maximum navigation stack depth
pub const MAX_NAVIGATION_DEPTH: usize = 20;

/// Maximum peers in a session
pub const MAX_PEERS: usize = 100;

/// Session timeout in seconds
pub const SESSION_TIMEOUT_SECS: u64 = 300; // 5 minutes

/// Configuration for PBKDF2 iterations
#[derive(Debug, Clone)]
pub struct KdfConfig {
    pub iterations: u32,
    pub salt_length: usize,
    pub key_length: usize,
}

impl KdfConfig {
    /// Create a new KDF configuration with validation
    pub fn new(iterations: u32) -> Result<Self, String> {
        if iterations < MIN_PBKDF2_ITERATIONS {
            return Err(format!(
                "PBKDF2 iterations {} is below minimum recommended {}",
                iterations, MIN_PBKDF2_ITERATIONS
            ));
        }
        
        if iterations > MAX_PBKDF2_ITERATIONS {
            return Err(format!(
                "PBKDF2 iterations {} exceeds maximum {}",
                iterations, MAX_PBKDF2_ITERATIONS
            ));
        }
        
        Ok(Self {
            iterations,
            salt_length: SALT_LENGTH,
            key_length: KEY_LENGTH,
        })
    }
    
    /// Get configuration from environment or use defaults
    pub fn from_env() -> Self {
        let iterations = env::var("PBKDF2_ITERATIONS")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(DEFAULT_PBKDF2_ITERATIONS);
        
        // Clamp to valid range
        let iterations = iterations
            .max(MIN_PBKDF2_ITERATIONS)
            .min(MAX_PBKDF2_ITERATIONS);
        
        Self {
            iterations,
            salt_length: SALT_LENGTH,
            key_length: KEY_LENGTH,
        }
    }
    
    /// Get default secure configuration
    pub fn default_secure() -> Self {
        Self {
            iterations: DEFAULT_PBKDF2_ITERATIONS,
            salt_length: SALT_LENGTH,
            key_length: KEY_LENGTH,
        }
    }
}

impl Default for KdfConfig {
    fn default() -> Self {
        Self::default_secure()
    }
}

/// Validate that a salt has the correct length
pub fn validate_salt(salt: &[u8]) -> Result<(), String> {
    if salt.len() != SALT_LENGTH {
        return Err(format!(
            "Invalid salt length: expected {}, got {}",
            SALT_LENGTH,
            salt.len()
        ));
    }
    Ok(())
}

/// Validate that a key has the correct length
pub fn validate_key(key: &[u8]) -> Result<(), String> {
    if key.len() != KEY_LENGTH {
        return Err(format!(
            "Invalid key length: expected {}, got {}",
            KEY_LENGTH,
            key.len()
        ));
    }
    Ok(())
}

/// Security audit information
pub fn security_info() -> String {
    format!(
        "Security Configuration:
        PBKDF2 Iterations: {} (min: {}, max: {})
        Salt Length: {} bytes
        Key Length: {} bytes
        Nonce Length: {} bytes
        Tag Length: {} bytes
        Max Notifications: {}
        Max Navigation Depth: {}
        Session Timeout: {} seconds",
        KdfConfig::from_env().iterations,
        MIN_PBKDF2_ITERATIONS,
        MAX_PBKDF2_ITERATIONS,
        SALT_LENGTH,
        KEY_LENGTH,
        NONCE_LENGTH,
        TAG_LENGTH,
        MAX_NOTIFICATIONS,
        MAX_NAVIGATION_DEPTH,
        SESSION_TIMEOUT_SECS
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_kdf_config_validation() {
        // Too few iterations
        assert!(KdfConfig::new(1000).is_err());
        
        // Valid iterations
        assert!(KdfConfig::new(100_000).is_ok());
        assert!(KdfConfig::new(210_000).is_ok());
        
        // Too many iterations
        assert!(KdfConfig::new(2_000_000).is_err());
    }
    
    #[test]
    fn test_salt_validation() {
        let valid_salt = vec![0u8; SALT_LENGTH];
        assert!(validate_salt(&valid_salt).is_ok());
        
        let invalid_salt = vec![0u8; 10];
        assert!(validate_salt(&invalid_salt).is_err());
    }
    
    #[test]
    fn test_key_validation() {
        let valid_key = vec![0u8; KEY_LENGTH];
        assert!(validate_key(&valid_key).is_ok());
        
        let invalid_key = vec![0u8; 16];
        assert!(validate_key(&invalid_key).is_err());
    }
}