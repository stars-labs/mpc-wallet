//! Standardized Error Handling
//!
//! This module provides a unified error handling system for the entire application,
//! ensuring consistent error types, propagation, and user feedback.

use std::fmt;
use thiserror::Error;

/// Main application error type
#[derive(Error, Debug)]
pub enum AppError {
    /// Domain validation errors
    #[error("Validation error: {0}")]
    Validation(#[from] crate::elm::domain_types::ValidationError),
    
    /// Repository errors
    #[error("Repository error: {0}")]
    Repository(#[from] crate::elm::repository::RepositoryError),
    
    /// Network-related errors
    #[error("Network error: {0}")]
    Network(NetworkError),
    
    /// Cryptographic errors
    #[error("Cryptographic error: {0}")]
    Crypto(CryptoError),
    
    /// Storage/IO errors
    #[error("Storage error: {0}")]
    Storage(StorageError),
    
    /// Session-related errors
    #[error("Session error: {0}")]
    Session(SessionError),
    
    /// Configuration errors
    #[error("Configuration error: {0}")]
    Configuration(ConfigError),
    
    /// UI/Interaction errors
    #[error("UI error: {0}")]
    UI(UIError),
    
    /// System errors
    #[error("System error: {0}")]
    System(SystemError),
    
    /// Unknown/unexpected errors
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Network-related errors
#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("Connection failed: {reason}")]
    ConnectionFailed { reason: String },
    
    #[error("Connection timeout after {seconds} seconds")]
    ConnectionTimeout { seconds: u64 },
    
    #[error("WebSocket error: {0}")]
    WebSocket(String),
    
    #[error("WebRTC error: {0}")]
    WebRTC(String),
    
    #[error("Peer disconnected: {peer_id}")]
    PeerDisconnected { peer_id: String },
    
    #[error("Invalid message format")]
    InvalidMessage,
    
    #[error("Network unavailable")]
    Unavailable,
}

/// Cryptographic errors
#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Invalid key material")]
    InvalidKeyMaterial,
    
    #[error("Signature verification failed")]
    SignatureVerificationFailed,
    
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    
    #[error("Invalid threshold: {threshold} of {participants}")]
    InvalidThreshold { threshold: u16, participants: u16 },
    
    #[error("Insufficient signatures: got {got}, need {need}")]
    InsufficientSignatures { got: usize, need: usize },
    
    #[error("DKG round {round} failed: {reason}")]
    DKGFailed { round: u8, reason: String },
    
    #[error("FROST protocol error: {0}")]
    FrostError(String),
}

/// Storage/IO errors
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("File not found: {path}")]
    FileNotFound { path: String },
    
    #[error("Permission denied: {path}")]
    PermissionDenied { path: String },
    
    #[error("Disk full")]
    DiskFull,
    
    #[error("Corrupted data: {details}")]
    CorruptedData { details: String },
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Keystore locked")]
    KeystoreLocked,
}

/// Session-related errors
#[derive(Error, Debug)]
pub enum SessionError {
    #[error("Session not found: {id}")]
    NotFound { id: String },
    
    #[error("Session expired")]
    Expired,
    
    #[error("Session full: {current}/{max} participants")]
    Full { current: usize, max: usize },
    
    #[error("Already in session")]
    AlreadyInSession,
    
    #[error("Not authorized for session")]
    Unauthorized,
    
    #[error("Invalid session state: expected {expected}, got {actual}")]
    InvalidState { expected: String, actual: String },
    
    #[error("Participant dropped: {peer_id}")]
    ParticipantDropped { peer_id: String },
}

/// Configuration errors
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Missing required field: {field}")]
    MissingField { field: String },
    
    #[error("Invalid value for {field}: {value}")]
    InvalidValue { field: String, value: String },
    
    #[error("Configuration file not found: {path}")]
    FileNotFound { path: String },
    
    #[error("Failed to parse configuration: {0}")]
    ParseError(String),
    
    #[error("Incompatible version: requires {required}, got {actual}")]
    IncompatibleVersion { required: String, actual: String },
}

/// UI/Interaction errors
#[derive(Error, Debug)]
pub enum UIError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Operation cancelled by user")]
    Cancelled,
    
    #[error("Component not found: {id}")]
    ComponentNotFound { id: String },
    
    #[error("Invalid navigation: {reason}")]
    InvalidNavigation { reason: String },
    
    #[error("Rendering error: {0}")]
    RenderError(String),
}

/// System errors
#[derive(Error, Debug)]
pub enum SystemError {
    #[error("Out of memory")]
    OutOfMemory,
    
    #[error("Thread panic: {0}")]
    ThreadPanic(String),
    
    #[error("Signal received: {0}")]
    Signal(String),
    
    #[error("Environment variable not set: {0}")]
    EnvVarNotSet(String),
    
    #[error("System call failed: {0}")]
    SystemCall(String),
}

/// Result type alias for the application
pub type AppResult<T> = Result<T, AppError>;

/// Error context for adding additional information
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub operation: String,
    pub details: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub retry_count: u32,
}

impl ErrorContext {
    /// Create a new error context
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            details: None,
            timestamp: chrono::Utc::now(),
            retry_count: 0,
        }
    }
    
    /// Add details to the context
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
    
    /// Set retry count
    pub fn with_retry_count(mut self, count: u32) -> Self {
        self.retry_count = count;
        self
    }
}

/// Error with context
#[derive(Debug)]
pub struct ContextualError {
    pub error: AppError,
    pub context: ErrorContext,
}

impl fmt::Display for ContextualError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.context.operation, self.error)?;
        if let Some(ref details) = self.context.details {
            write!(f, " ({})", details)?;
        }
        if self.context.retry_count > 0 {
            write!(f, " [retry {}]", self.context.retry_count)?;
        }
        Ok(())
    }
}

impl std::error::Error for ContextualError {}

/// Extension trait for adding context to errors
pub trait ErrorExt: Sized {
    /// Add context to the error
    fn with_context(self, context: ErrorContext) -> ContextualError;
    
    /// Add operation context
    fn with_operation(self, operation: impl Into<String>) -> ContextualError;
}

impl ErrorExt for AppError {
    fn with_context(self, context: ErrorContext) -> ContextualError {
        ContextualError {
            error: self,
            context,
        }
    }
    
    fn with_operation(self, operation: impl Into<String>) -> ContextualError {
        self.with_context(ErrorContext::new(operation))
    }
}

/// Error recovery strategies
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// Retry the operation
    Retry {
        max_attempts: u32,
        backoff: BackoffStrategy,
    },
    
    /// Fallback to alternative
    Fallback {
        alternative: String,
    },
    
    /// Skip and continue
    Skip,
    
    /// Abort the operation
    Abort,
    
    /// Ask user for input
    AskUser {
        prompt: String,
        options: Vec<String>,
    },
}

/// Backoff strategies for retries
#[derive(Debug, Clone)]
pub enum BackoffStrategy {
    /// Fixed delay between retries
    Fixed { delay_ms: u64 },
    
    /// Exponential backoff
    Exponential { 
        initial_ms: u64,
        max_ms: u64,
        factor: f64,
    },
    
    /// Linear backoff
    Linear {
        initial_ms: u64,
        increment_ms: u64,
    },
}

impl BackoffStrategy {
    /// Calculate delay for given attempt number (0-indexed)
    pub fn delay(&self, attempt: u32) -> std::time::Duration {
        let ms = match self {
            Self::Fixed { delay_ms } => *delay_ms,
            
            Self::Exponential { initial_ms, max_ms, factor } => {
                let delay = (*initial_ms as f64) * factor.powi(attempt as i32);
                delay.min(*max_ms as f64) as u64
            }
            
            Self::Linear { initial_ms, increment_ms } => {
                initial_ms + (increment_ms * attempt as u64)
            }
        };
        
        std::time::Duration::from_millis(ms)
    }
}

/// Error handler for determining recovery strategies
pub struct ErrorHandler;

impl ErrorHandler {
    /// Determine recovery strategy for an error
    pub fn recovery_strategy(error: &AppError) -> RecoveryStrategy {
        match error {
            // Network errors usually benefit from retry
            AppError::Network(NetworkError::ConnectionFailed { .. }) |
            AppError::Network(NetworkError::ConnectionTimeout { .. }) => {
                RecoveryStrategy::Retry {
                    max_attempts: 3,
                    backoff: BackoffStrategy::Exponential {
                        initial_ms: 1000,
                        max_ms: 30000,
                        factor: 2.0,
                    },
                }
            }
            
            // Storage errors might be recoverable
            AppError::Storage(StorageError::PermissionDenied { .. }) => {
                RecoveryStrategy::AskUser {
                    prompt: "Permission denied. How would you like to proceed?".to_string(),
                    options: vec![
                        "Retry with different permissions".to_string(),
                        "Choose different location".to_string(),
                        "Cancel operation".to_string(),
                    ],
                }
            }
            
            // Validation errors need user correction
            AppError::Validation(_) => RecoveryStrategy::AskUser {
                prompt: "Invalid input detected.".to_string(),
                options: vec![
                    "Correct input".to_string(),
                    "Cancel".to_string(),
                ],
            },
            
            // Session errors might be retryable
            AppError::Session(SessionError::Expired) => RecoveryStrategy::Retry {
                max_attempts: 1,
                backoff: BackoffStrategy::Fixed { delay_ms: 0 },
            },
            
            // Critical errors should abort
            AppError::Crypto(_) |
            AppError::System(_) => RecoveryStrategy::Abort,
            
            // Default to skip for unknown errors
            _ => RecoveryStrategy::Skip,
        }
    }
    
    /// Check if error is retryable
    pub fn is_retryable(error: &AppError) -> bool {
        matches!(
            error,
            AppError::Network(_) | 
            AppError::Session(SessionError::Expired) |
            AppError::Storage(StorageError::Io(_))
        )
    }
    
    /// Check if error is critical
    pub fn is_critical(error: &AppError) -> bool {
        matches!(
            error,
            AppError::Crypto(_) |
            AppError::System(_) |
            AppError::Storage(StorageError::CorruptedData { .. })
        )
    }
}

/// Macro for easy error creation with context
#[macro_export]
macro_rules! app_error {
    ($error:expr) => {
        AppError::from($error)
    };
    
    ($error:expr, $operation:expr) => {
        AppError::from($error).with_operation($operation)
    };
    
    ($error:expr, $operation:expr, $details:expr) => {
        AppError::from($error).with_context(
            ErrorContext::new($operation).with_details($details)
        )
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_display() {
        let error = AppError::Network(NetworkError::ConnectionFailed {
            reason: "Host unreachable".to_string(),
        });
        
        assert!(error.to_string().contains("Connection failed"));
        assert!(error.to_string().contains("Host unreachable"));
    }
    
    #[test]
    fn test_error_context() {
        let error = AppError::Validation(
            crate::elm::domain_types::ValidationError::Empty("Field required".to_string())
        );
        
        let contextual = error.with_context(
            ErrorContext::new("Creating wallet")
                .with_details("Name field")
                .with_retry_count(2)
        );
        
        let display = contextual.to_string();
        assert!(display.contains("Creating wallet"));
        assert!(display.contains("Name field"));
        assert!(display.contains("retry 2"));
    }
    
    #[test]
    fn test_backoff_strategies() {
        let fixed = BackoffStrategy::Fixed { delay_ms: 1000 };
        assert_eq!(fixed.delay(0).as_millis(), 1000);
        assert_eq!(fixed.delay(5).as_millis(), 1000);
        
        let exponential = BackoffStrategy::Exponential {
            initial_ms: 100,
            max_ms: 10000,
            factor: 2.0,
        };
        assert_eq!(exponential.delay(0).as_millis(), 100);
        assert_eq!(exponential.delay(1).as_millis(), 200);
        assert_eq!(exponential.delay(2).as_millis(), 400);
        assert_eq!(exponential.delay(10).as_millis(), 10000); // Capped at max
        
        let linear = BackoffStrategy::Linear {
            initial_ms: 100,
            increment_ms: 50,
        };
        assert_eq!(linear.delay(0).as_millis(), 100);
        assert_eq!(linear.delay(1).as_millis(), 150);
        assert_eq!(linear.delay(2).as_millis(), 200);
    }
    
    #[test]
    fn test_error_handler() {
        let network_error = AppError::Network(NetworkError::ConnectionFailed {
            reason: "test".to_string(),
        });
        assert!(ErrorHandler::is_retryable(&network_error));
        assert!(!ErrorHandler::is_critical(&network_error));
        
        let crypto_error = AppError::Crypto(CryptoError::InvalidKeyMaterial);
        assert!(!ErrorHandler::is_retryable(&crypto_error));
        assert!(ErrorHandler::is_critical(&crypto_error));
    }
}