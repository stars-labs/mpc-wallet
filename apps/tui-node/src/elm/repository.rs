//! Repository Pattern - Data access abstraction
//!
//! This module provides repository interfaces for data access,
//! separating business logic from storage implementation.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

use crate::elm::domain_types::{
    WalletId, SessionId, PeerId, DeviceId, Address, 
    ThresholdConfig, WalletName, ValidationError
};

/// Result type for repository operations
pub type RepositoryResult<T> = Result<T, RepositoryError>;

/// Repository errors
#[derive(Debug, Clone)]
pub enum RepositoryError {
    NotFound(String),
    AlreadyExists(String),
    ValidationError(ValidationError),
    StorageError(String),
    ConcurrencyError(String),
    NetworkError(String),
}

impl std::fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(msg) => write!(f, "Not found: {}", msg),
            Self::AlreadyExists(msg) => write!(f, "Already exists: {}", msg),
            Self::ValidationError(err) => write!(f, "Validation error: {}", err),
            Self::StorageError(msg) => write!(f, "Storage error: {}", msg),
            Self::ConcurrencyError(msg) => write!(f, "Concurrency error: {}", msg),
            Self::NetworkError(msg) => write!(f, "Network error: {}", msg),
        }
    }
}

impl std::error::Error for RepositoryError {}

impl From<ValidationError> for RepositoryError {
    fn from(err: ValidationError) -> Self {
        Self::ValidationError(err)
    }
}

/// Wallet entity
#[derive(Debug, Clone)]
pub struct Wallet {
    pub id: WalletId,
    pub name: WalletName,
    pub threshold_config: ThresholdConfig,
    pub curve: CurveType,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub addresses: HashMap<String, Address>,
    pub metadata: HashMap<String, String>,
}

/// Session entity
#[derive(Debug, Clone)]
pub struct Session {
    pub id: SessionId,
    pub session_type: SessionType,
    pub state: SessionState,
    pub participants: Vec<Participant>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, String>,
}

/// Session types
#[derive(Debug, Clone, PartialEq)]
pub enum SessionType {
    DKG { wallet_config: WalletConfig },
    Signing { wallet_id: WalletId, message: Vec<u8> },
}

/// Session state
#[derive(Debug, Clone, PartialEq)]
pub enum SessionState {
    Pending,
    Active,
    Complete,
    Failed(String),
    Expired,
}

/// Session participant
#[derive(Debug, Clone)]
pub struct Participant {
    pub peer_id: PeerId,
    pub device_id: DeviceId,
    pub status: ParticipantStatus,
    pub joined_at: DateTime<Utc>,
}

/// Participant status
#[derive(Debug, Clone, PartialEq)]
pub enum ParticipantStatus {
    Invited,
    Joined,
    Ready,
    Active,
    Completed,
    Dropped,
}

/// Wallet configuration
#[derive(Debug, Clone, PartialEq)]
pub struct WalletConfig {
    pub name: WalletName,
    pub threshold_config: ThresholdConfig,
    pub curve: CurveType,
}

/// Curve types
#[derive(Debug, Clone, PartialEq)]
pub enum CurveType {
    Secp256k1,
    Ed25519,
}

/// Wallet repository trait
#[async_trait]
pub trait WalletRepository: Send + Sync {
    /// Create a new wallet
    async fn create(&self, wallet: Wallet) -> RepositoryResult<WalletId>;
    
    /// Get wallet by ID
    async fn get(&self, id: &WalletId) -> RepositoryResult<Wallet>;
    
    /// Update wallet
    async fn update(&self, wallet: Wallet) -> RepositoryResult<()>;
    
    /// Delete wallet
    async fn delete(&self, id: &WalletId) -> RepositoryResult<()>;
    
    /// List all wallets
    async fn list(&self) -> RepositoryResult<Vec<Wallet>>;
    
    /// Find wallets by name
    async fn find_by_name(&self, name: &str) -> RepositoryResult<Vec<Wallet>>;
    
    /// Check if wallet exists
    async fn exists(&self, id: &WalletId) -> RepositoryResult<bool>;
}

/// Session repository trait
#[async_trait]
pub trait SessionRepository: Send + Sync {
    /// Create a new session
    async fn create(&self, session: Session) -> RepositoryResult<SessionId>;
    
    /// Get session by ID
    async fn get(&self, id: &SessionId) -> RepositoryResult<Session>;
    
    /// Update session
    async fn update(&self, session: Session) -> RepositoryResult<()>;
    
    /// Delete session
    async fn delete(&self, id: &SessionId) -> RepositoryResult<()>;
    
    /// List active sessions
    async fn list_active(&self) -> RepositoryResult<Vec<Session>>;
    
    /// Find sessions by type
    async fn find_by_type(&self, session_type: &SessionType) -> RepositoryResult<Vec<Session>>;
    
    /// Add participant to session
    async fn add_participant(&self, session_id: &SessionId, participant: Participant) -> RepositoryResult<()>;
    
    /// Update participant status
    async fn update_participant_status(
        &self, 
        session_id: &SessionId, 
        peer_id: &PeerId, 
        status: ParticipantStatus
    ) -> RepositoryResult<()>;
    
    /// Clean up expired sessions
    async fn cleanup_expired(&self) -> RepositoryResult<usize>;
}

/// In-memory wallet repository implementation
pub struct InMemoryWalletRepository {
    storage: Arc<RwLock<HashMap<WalletId, Wallet>>>,
}

impl InMemoryWalletRepository {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl WalletRepository for InMemoryWalletRepository {
    async fn create(&self, wallet: Wallet) -> RepositoryResult<WalletId> {
        let mut storage = self.storage.write().await;
        
        if storage.contains_key(&wallet.id) {
            return Err(RepositoryError::AlreadyExists(
                format!("Wallet {} already exists", wallet.id)
            ));
        }
        
        let id = wallet.id.clone();
        storage.insert(wallet.id.clone(), wallet);
        Ok(id)
    }
    
    async fn get(&self, id: &WalletId) -> RepositoryResult<Wallet> {
        let storage = self.storage.read().await;
        storage.get(id)
            .cloned()
            .ok_or_else(|| RepositoryError::NotFound(format!("Wallet {} not found", id)))
    }
    
    async fn update(&self, wallet: Wallet) -> RepositoryResult<()> {
        let mut storage = self.storage.write().await;
        
        if !storage.contains_key(&wallet.id) {
            return Err(RepositoryError::NotFound(
                format!("Wallet {} not found", wallet.id)
            ));
        }
        
        storage.insert(wallet.id.clone(), wallet);
        Ok(())
    }
    
    async fn delete(&self, id: &WalletId) -> RepositoryResult<()> {
        let mut storage = self.storage.write().await;
        
        if storage.remove(id).is_none() {
            return Err(RepositoryError::NotFound(
                format!("Wallet {} not found", id)
            ));
        }
        
        Ok(())
    }
    
    async fn list(&self) -> RepositoryResult<Vec<Wallet>> {
        let storage = self.storage.read().await;
        Ok(storage.values().cloned().collect())
    }
    
    async fn find_by_name(&self, name: &str) -> RepositoryResult<Vec<Wallet>> {
        let storage = self.storage.read().await;
        let name_lower = name.to_lowercase();
        
        Ok(storage.values()
            .filter(|w| w.name.as_str().to_lowercase().contains(&name_lower))
            .cloned()
            .collect())
    }
    
    async fn exists(&self, id: &WalletId) -> RepositoryResult<bool> {
        let storage = self.storage.read().await;
        Ok(storage.contains_key(id))
    }
}

/// In-memory session repository implementation
pub struct InMemorySessionRepository {
    storage: Arc<RwLock<HashMap<SessionId, Session>>>,
}

impl InMemorySessionRepository {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl SessionRepository for InMemorySessionRepository {
    async fn create(&self, session: Session) -> RepositoryResult<SessionId> {
        let mut storage = self.storage.write().await;
        
        if storage.contains_key(&session.id) {
            return Err(RepositoryError::AlreadyExists(
                format!("Session {} already exists", session.id)
            ));
        }
        
        let id = session.id.clone();
        storage.insert(session.id.clone(), session);
        Ok(id)
    }
    
    async fn get(&self, id: &SessionId) -> RepositoryResult<Session> {
        let storage = self.storage.read().await;
        storage.get(id)
            .cloned()
            .ok_or_else(|| RepositoryError::NotFound(format!("Session {} not found", id)))
    }
    
    async fn update(&self, session: Session) -> RepositoryResult<()> {
        let mut storage = self.storage.write().await;
        
        if !storage.contains_key(&session.id) {
            return Err(RepositoryError::NotFound(
                format!("Session {} not found", session.id)
            ));
        }
        
        storage.insert(session.id.clone(), session);
        Ok(())
    }
    
    async fn delete(&self, id: &SessionId) -> RepositoryResult<()> {
        let mut storage = self.storage.write().await;
        
        if storage.remove(id).is_none() {
            return Err(RepositoryError::NotFound(
                format!("Session {} not found", id)
            ));
        }
        
        Ok(())
    }
    
    async fn list_active(&self) -> RepositoryResult<Vec<Session>> {
        let storage = self.storage.read().await;
        
        Ok(storage.values()
            .filter(|s| matches!(s.state, SessionState::Active | SessionState::Pending))
            .cloned()
            .collect())
    }
    
    async fn find_by_type(&self, session_type: &SessionType) -> RepositoryResult<Vec<Session>> {
        let storage = self.storage.read().await;
        
        Ok(storage.values()
            .filter(|s| std::mem::discriminant(&s.session_type) == std::mem::discriminant(session_type))
            .cloned()
            .collect())
    }
    
    async fn add_participant(&self, session_id: &SessionId, participant: Participant) -> RepositoryResult<()> {
        let mut storage = self.storage.write().await;
        
        let session = storage.get_mut(session_id)
            .ok_or_else(|| RepositoryError::NotFound(format!("Session {} not found", session_id)))?;
        
        // Check if participant already exists
        if session.participants.iter().any(|p| p.peer_id == participant.peer_id) {
            return Err(RepositoryError::AlreadyExists(
                format!("Participant {} already in session", participant.peer_id)
            ));
        }
        
        session.participants.push(participant);
        Ok(())
    }
    
    async fn update_participant_status(
        &self, 
        session_id: &SessionId, 
        peer_id: &PeerId, 
        status: ParticipantStatus
    ) -> RepositoryResult<()> {
        let mut storage = self.storage.write().await;
        
        let session = storage.get_mut(session_id)
            .ok_or_else(|| RepositoryError::NotFound(format!("Session {} not found", session_id)))?;
        
        let participant = session.participants.iter_mut()
            .find(|p| p.peer_id == *peer_id)
            .ok_or_else(|| RepositoryError::NotFound(format!("Participant {} not found", peer_id)))?;
        
        participant.status = status;
        Ok(())
    }
    
    async fn cleanup_expired(&self) -> RepositoryResult<usize> {
        let mut storage = self.storage.write().await;
        let now = Utc::now();
        
        let expired_ids: Vec<SessionId> = storage.iter()
            .filter(|(_, session)| {
                if let Some(expires_at) = session.expires_at {
                    expires_at < now
                } else {
                    false
                }
            })
            .map(|(id, _)| id.clone())
            .collect();
        
        let count = expired_ids.len();
        
        for id in expired_ids {
            storage.remove(&id);
        }
        
        Ok(count)
    }
}

/// Repository manager for dependency injection
pub struct RepositoryManager {
    pub wallets: Arc<dyn WalletRepository>,
    pub sessions: Arc<dyn SessionRepository>,
}

impl RepositoryManager {
    /// Create with in-memory repositories
    pub fn in_memory() -> Self {
        Self {
            wallets: Arc::new(InMemoryWalletRepository::new()),
            sessions: Arc::new(InMemorySessionRepository::new()),
        }
    }
    
    /// Create with custom repositories
    pub fn new(
        wallets: Arc<dyn WalletRepository>,
        sessions: Arc<dyn SessionRepository>,
    ) -> Self {
        Self {
            wallets,
            sessions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_wallet_repository() {
        let repo = InMemoryWalletRepository::new();
        
        let wallet = Wallet {
            id: WalletId::generate(),
            name: WalletName::new("Test Wallet").unwrap(),
            threshold_config: ThresholdConfig::new(2, 3).unwrap(),
            curve: CurveType::Secp256k1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            addresses: HashMap::new(),
            metadata: HashMap::new(),
        };
        
        // Create
        let id = repo.create(wallet.clone()).await.unwrap();
        assert_eq!(id, wallet.id);
        
        // Get
        let retrieved = repo.get(&id).await.unwrap();
        assert_eq!(retrieved.id, wallet.id);
        
        // Exists
        assert!(repo.exists(&id).await.unwrap());
        
        // List
        let wallets = repo.list().await.unwrap();
        assert_eq!(wallets.len(), 1);
        
        // Delete
        repo.delete(&id).await.unwrap();
        assert!(!repo.exists(&id).await.unwrap());
    }
    
    #[tokio::test]
    async fn test_session_repository() {
        let repo = InMemorySessionRepository::new();
        
        let session = Session {
            id: SessionId::generate(),
            session_type: SessionType::DKG {
                wallet_config: WalletConfig {
                    name: WalletName::new("Test").unwrap(),
                    threshold_config: ThresholdConfig::new(2, 3).unwrap(),
                    curve: CurveType::Secp256k1,
                }
            },
            state: SessionState::Active,
            participants: vec![],
            created_at: Utc::now(),
            expires_at: None,
            metadata: HashMap::new(),
        };
        
        // Create
        let id = repo.create(session.clone()).await.unwrap();
        
        // Add participant
        let participant = Participant {
            peer_id: PeerId::new("peer1").unwrap(),
            device_id: DeviceId::new("device1").unwrap(),
            status: ParticipantStatus::Joined,
            joined_at: Utc::now(),
        };
        
        repo.add_participant(&id, participant.clone()).await.unwrap();
        
        // Get and verify
        let retrieved = repo.get(&id).await.unwrap();
        assert_eq!(retrieved.participants.len(), 1);
        
        // Update participant status
        repo.update_participant_status(
            &id,
            &participant.peer_id,
            ParticipantStatus::Ready
        ).await.unwrap();
        
        // List active
        let active = repo.list_active().await.unwrap();
        assert_eq!(active.len(), 1);
    }
}