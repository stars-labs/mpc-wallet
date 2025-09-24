// Optimized State Management Implementation
// Provides efficient state storage with O(1) lookups and bounded memory usage

use std::collections::{HashMap, HashSet, BTreeSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use dashmap::DashMap;
use lru::LruCache;
use frost_core::Ciphersuite;
use crate::protocal::signal::{SessionInfo, SessionAnnouncement};

/// Optimized application state with efficient data structures
pub struct OptimizedAppState<C: Ciphersuite> {
    /// Primary session storage with O(1) lookup
    pub sessions: DashMap<String, SessionData>,
    
    /// Invites with automatic expiration
    pub invites: Arc<RwLock<LruCache<String, InviteData>>>,
    
    /// Device to sessions mapping for quick queries
    pub device_sessions: DashMap<String, HashSet<String>>,
    
    /// Session participants with sorted structure for consistent ordering
    pub session_participants: DashMap<String, BTreeSet<String>>,
    
    /// Active connections tracking
    pub active_connections: DashMap<String, ConnectionInfo>,
    
    /// Performance metrics
    pub metrics: Arc<StateMetrics>,
    
    /// Cleanup configuration
    cleanup_config: CleanupConfig,
    
    /// Phantom data for generic type
    _phantom: std::marker::PhantomData<C>,
}

#[derive(Clone)]
pub struct SessionData {
    pub info: SessionInfo,
    pub created_at: Instant,
    pub last_activity: Instant,
    pub state: SessionState,
    pub metadata: HashMap<String, String>,
}

#[derive(Clone)]
pub struct InviteData {
    pub info: SessionInfo,
    pub received_at: Instant,
    pub expires_at: Instant,
    pub from_device: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SessionState {
    Pending,
    Active,
    Completing,
    Completed,
    Failed(String),
}

#[derive(Clone)]
pub struct ConnectionInfo {
    pub device_id: String,
    pub established_at: Instant,
    pub last_message: Instant,
    pub message_count: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

#[derive(Clone)]
pub struct CleanupConfig {
    pub invite_ttl: Duration,
    pub session_ttl: Duration,
    pub connection_idle_timeout: Duration,
    pub cleanup_interval: Duration,
}

impl Default for CleanupConfig {
    fn default() -> Self {
        Self {
            invite_ttl: Duration::from_secs(300), // 5 minutes
            session_ttl: Duration::from_secs(3600), // 1 hour
            connection_idle_timeout: Duration::from_secs(600), // 10 minutes
            cleanup_interval: Duration::from_secs(60), // Run cleanup every minute
        }
    }
}

pub struct StateMetrics {
    pub session_count: Arc<RwLock<u64>>,
    pub invite_count: Arc<RwLock<u64>>,
    pub connection_count: Arc<RwLock<u64>>,
    pub cleanup_runs: Arc<RwLock<u64>>,
    pub memory_usage: Arc<RwLock<u64>>,
}

impl<C: Ciphersuite> OptimizedAppState<C> {
    pub fn new() -> Self {
        Self::with_config(CleanupConfig::default())
    }

    pub fn with_config(config: CleanupConfig) -> Self {
        let state = Self {
            sessions: DashMap::new(),
            invites: Arc::new(RwLock::new(LruCache::new(
                std::num::NonZeroUsize::new(1000).unwrap() // Max 1000 invites
            ))),
            device_sessions: DashMap::new(),
            session_participants: DashMap::new(),
            active_connections: DashMap::new(),
            metrics: Arc::new(StateMetrics {
                session_count: Arc::new(RwLock::new(0)),
                invite_count: Arc::new(RwLock::new(0)),
                connection_count: Arc::new(RwLock::new(0)),
                cleanup_runs: Arc::new(RwLock::new(0)),
                memory_usage: Arc::new(RwLock::new(0)),
            }),
            cleanup_config: config.clone(),
            _phantom: std::marker::PhantomData,
        };

        // Start cleanup task
        let state_weak = Arc::downgrade(&Arc::new(()));
        let cleanup_interval = config.cleanup_interval;
        let sessions = state.sessions.clone();
        let invites = state.invites.clone();
        let connections = state.active_connections.clone();
        let metrics = state.metrics.clone();
        let cleanup_config = config;
        
        tokio::spawn(async move {
            Self::cleanup_task(
                state_weak,
                sessions,
                invites,
                connections,
                metrics,
                cleanup_config,
                cleanup_interval,
            ).await;
        });

        state
    }

    /// Add or update a session
    pub async fn upsert_session(&self, session_id: String, info: SessionInfo) {
        let now = Instant::now();
        
        // Update session data
        let session_data = SessionData {
            info: info.clone(),
            created_at: now,
            last_activity: now,
            state: SessionState::Pending,
            metadata: HashMap::new(),
        };
        
        self.sessions.insert(session_id.clone(), session_data);
        
        // Update indices
        for participant in &info.participants {
            self.device_sessions
                .entry(participant.clone())
                .or_insert_with(HashSet::new)
                .insert(session_id.clone());
        }
        
        self.session_participants.insert(
            session_id.clone(),
            info.participants.iter().cloned().collect(),
        );
        
        // Update metrics
        *self.metrics.session_count.write().await = self.sessions.len() as u64;
    }

    /// Get session by ID with O(1) lookup
    pub fn get_session(&self, session_id: &str) -> Option<SessionInfo> {
        self.sessions.get(session_id).map(|entry| entry.info.clone())
    }

    /// Get all sessions for a device
    pub fn get_device_sessions(&self, device_id: &str) -> Vec<String> {
        self.device_sessions
            .get(device_id)
            .map(|set| set.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Add an invite with automatic expiration
    pub async fn add_invite(&self, session_id: String, info: SessionInfo, from_device: String) {
        let now = Instant::now();
        let invite = InviteData {
            info,
            received_at: now,
            expires_at: now + self.cleanup_config.invite_ttl,
            from_device,
        };
        
        self.invites.write().await.put(session_id, invite);
        *self.metrics.invite_count.write().await = self.invites.read().await.len() as u64;
    }

    /// Get and remove an invite
    pub async fn take_invite(&self, session_id: &str) -> Option<SessionInfo> {
        let mut invites = self.invites.write().await;
        let invite = invites.pop(session_id)?;
        
        // Check if expired
        if Instant::now() > invite.expires_at {
            return None;
        }
        
        *self.metrics.invite_count.write().await = invites.len() as u64;
        Some(invite.info)
    }

    /// Update session state
    pub fn update_session_state(&self, session_id: &str, state: SessionState) {
        if let Some(mut session) = self.sessions.get_mut(session_id) {
            session.state = state;
            session.last_activity = Instant::now();
        }
    }

    /// Record connection activity
    pub fn record_connection_activity(&self, device_id: &str, bytes_sent: u64, bytes_received: u64) {
        let now = Instant::now();
        
        self.active_connections
            .entry(device_id.to_string())
            .and_modify(|conn| {
                conn.last_message = now;
                conn.message_count += 1;
                conn.bytes_sent += bytes_sent;
                conn.bytes_received += bytes_received;
            })
            .or_insert_with(|| ConnectionInfo {
                device_id: device_id.to_string(),
                established_at: now,
                last_message: now,
                message_count: 1,
                bytes_sent,
                bytes_received,
            });
    }

    /// Get state statistics
    pub async fn get_stats(&self) -> StateStats {
        let memory_usage = self.estimate_memory_usage().await;
        *self.metrics.memory_usage.write().await = memory_usage;
        
        StateStats {
            total_sessions: self.sessions.len(),
            active_sessions: self.sessions.iter()
                .filter(|entry| matches!(entry.state, SessionState::Active))
                .count(),
            total_invites: self.invites.read().await.len(),
            total_connections: self.active_connections.len(),
            memory_usage_bytes: memory_usage,
            cleanup_runs: *self.metrics.cleanup_runs.read().await,
        }
    }

    /// Estimate memory usage
    async fn estimate_memory_usage(&self) -> u64 {
        let session_size = 500; // Estimated bytes per session
        let invite_size = 400; // Estimated bytes per invite
        let connection_size = 200; // Estimated bytes per connection
        
        let sessions_memory = self.sessions.len() as u64 * session_size;
        let invites_memory = self.invites.read().await.len() as u64 * invite_size;
        let connections_memory = self.active_connections.len() as u64 * connection_size;
        
        sessions_memory + invites_memory + connections_memory
    }

    /// Background cleanup task
    async fn cleanup_task(
        _state_weak: std::sync::Weak<()>,
        sessions: DashMap<String, SessionData>,
        invites: Arc<RwLock<LruCache<String, InviteData>>>,
        connections: DashMap<String, ConnectionInfo>,
        metrics: Arc<StateMetrics>,
        config: CleanupConfig,
        interval: Duration,
    ) {
        let mut interval = tokio::time::interval(interval);
        
        loop {
            interval.tick().await;
            
            let now = Instant::now();
            let mut cleaned = CleanupStats::default();
            
            // Cleanup expired sessions
            let mut expired_sessions = Vec::new();
            for entry in sessions.iter() {
                let age = now.duration_since(entry.last_activity);
                if age > config.session_ttl && matches!(entry.state, SessionState::Completed | SessionState::Failed(_)) {
                    expired_sessions.push(entry.key().clone());
                }
            }
            
            for session_id in expired_sessions {
                sessions.remove(&session_id);
                cleaned.sessions_removed += 1;
            }
            
            // Cleanup expired invites (LRU handles most of this)
            {
                let invites_guard = invites.read().await;
                cleaned.invites_checked = invites_guard.len();
                drop(invites_guard);
            }
            
            // Cleanup idle connections
            let mut idle_connections = Vec::new();
            for entry in connections.iter() {
                if now.duration_since(entry.last_message) > config.connection_idle_timeout {
                    idle_connections.push(entry.key().clone());
                }
            }
            
            for device_id in idle_connections {
                connections.remove(&device_id);
                cleaned.connections_removed += 1;
            }
            
            // Update metrics
            *metrics.cleanup_runs.write().await += 1;
            *metrics.session_count.write().await = sessions.len() as u64;
            *metrics.connection_count.write().await = connections.len() as u64;
            
            if cleaned.sessions_removed > 0 || cleaned.connections_removed > 0 {
                log::debug!(
                    "State cleanup: removed {} sessions, {} connections",
                    cleaned.sessions_removed,
                    cleaned.connections_removed
                );
            }
        }
    }
}

#[derive(Debug)]
pub struct StateStats {
    pub total_sessions: usize,
    pub active_sessions: usize,
    pub total_invites: usize,
    pub total_connections: usize,
    pub memory_usage_bytes: u64,
    pub cleanup_runs: u64,
}

#[derive(Default)]
struct CleanupStats {
    pub sessions_removed: usize,
    pub invites_checked: usize,
    pub connections_removed: usize,
}

/// Fast session lookup table for discovery
pub struct SessionLookupTable {
    /// Session code to session announcement mapping
    by_code: DashMap<String, SessionAnnouncement>,
    
    /// Device to session codes mapping
    by_device: DashMap<String, HashSet<String>>,
    
    /// Wallet type to session codes mapping
    by_wallet_type: DashMap<String, HashSet<String>>,
}

impl SessionLookupTable {
    pub fn new() -> Self {
        Self {
            by_code: DashMap::new(),
            by_device: DashMap::new(),
            by_wallet_type: DashMap::new(),
        }
    }

    /// Add or update a session announcement
    pub fn upsert(&self, announcement: SessionAnnouncement) {
        let code = announcement.session_code.clone();
        let device = announcement.creator_device.clone();
        let wallet_type = announcement.wallet_type.clone();
        
        // Update primary index
        self.by_code.insert(code.clone(), announcement);
        
        // Update secondary indices
        self.by_device
            .entry(device)
            .or_insert_with(HashSet::new)
            .insert(code.clone());
            
        self.by_wallet_type
            .entry(wallet_type)
            .or_insert_with(HashSet::new)
            .insert(code);
    }

    /// Get session by code
    pub fn get_by_code(&self, code: &str) -> Option<SessionAnnouncement> {
        self.by_code.get(code).map(|entry| entry.value().clone())
    }

    /// Get all sessions by device
    pub fn get_by_device(&self, device: &str) -> Vec<SessionAnnouncement> {
        self.by_device
            .get(device)
            .map(|codes| {
                codes.iter()
                    .filter_map(|code| self.get_by_code(code))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all sessions by wallet type
    pub fn get_by_wallet_type(&self, wallet_type: &str) -> Vec<SessionAnnouncement> {
        self.by_wallet_type
            .get(wallet_type)
            .map(|codes| {
                codes.iter()
                    .filter_map(|code| self.get_by_code(code))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Remove expired sessions
    pub fn cleanup_expired(&self, max_age: Duration) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        let mut to_remove = Vec::new();
        
        for entry in self.by_code.iter() {
            let age = now.saturating_sub(entry.timestamp);
            if age > max_age.as_secs() {
                to_remove.push(entry.key().clone());
            }
        }
        
        for code in to_remove {
            if let Some((_, announcement)) = self.by_code.remove(&code) {
                // Clean up secondary indices
                if let Some(mut device_codes) = self.by_device.get_mut(&announcement.creator_device) {
                    device_codes.remove(&code);
                }
                
                if let Some(mut wallet_codes) = self.by_wallet_type.get_mut(&announcement.wallet_type) {
                    wallet_codes.remove(&code);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocal::signal::SessionType;

    #[tokio::test]
    async fn test_optimized_state_operations() {
        let state = OptimizedAppState::<frost_secp256k1::Secp256K1Sha256>::new();
        
        // Test session operations
        let session_info = SessionInfo {
            session_id: "test_session".to_string(),
            proposer_id: "device1".to_string(),
            total: 3,
            threshold: 2,
            participants: vec!["device1".to_string(), "device2".to_string(), "device3".to_string()],
            participants: vec!["device1".to_string()],
            session_type: SessionType::DKG,
            curve_type: "secp256k1".to_string(),
            coordination_type: "network".to_string(),
        };
        
        state.upsert_session("test_session".to_string(), session_info.clone()).await;
        
        // Test O(1) lookup
        let retrieved = state.get_session("test_session");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().session_id, "test_session");
        
        // Test device sessions lookup
        let device_sessions = state.get_device_sessions("device1");
        assert_eq!(device_sessions.len(), 1);
        assert!(device_sessions.contains(&"test_session".to_string()));
    }

    #[tokio::test]
    async fn test_invite_expiration() {
        let mut config = CleanupConfig::default();
        config.invite_ttl = Duration::from_millis(100); // Short TTL for testing
        
        let state = OptimizedAppState::<frost_secp256k1::Secp256K1Sha256>::with_config(config);
        
        let session_info = SessionInfo {
            session_id: "test_invite".to_string(),
            proposer_id: "device1".to_string(),
            total: 2,
            threshold: 2,
            participants: vec!["device1".to_string(), "device2".to_string()],
            participants: vec![],
            session_type: SessionType::DKG,
            curve_type: "secp256k1".to_string(),
            coordination_type: "network".to_string(),
        };
        
        state.add_invite("test_invite".to_string(), session_info, "device1".to_string()).await;
        
        // Invite should be available immediately
        assert!(state.take_invite("test_invite").await.is_some());
        
        // Add another invite and wait for expiration
        state.add_invite("test_invite2".to_string(), session_info, "device1".to_string()).await;
        tokio::time::sleep(Duration::from_millis(150)).await;
        
        // Expired invite should not be returned
        assert!(state.take_invite("test_invite2").await.is_none());
    }

    #[tokio::test]
    async fn test_session_lookup_table() {
        let table = SessionLookupTable::new();
        
        let announcement = SessionAnnouncement {
            session_code: "ABC123".to_string(),
            wallet_type: "ethereum".to_string(),
            threshold: 2,
            total: 3,
            curve_type: "secp256k1".to_string(),
            creator_device: "device1".to_string(),
            participants_joined: 1,
            description: Some("Test wallet".to_string()),
            timestamp: 1234567890,
        };
        
        table.upsert(announcement.clone());
        
        // Test lookups
        assert!(table.get_by_code("ABC123").is_some());
        assert_eq!(table.get_by_device("device1").len(), 1);
        assert_eq!(table.get_by_wallet_type("ethereum").len(), 1);
        
        // Test cleanup
        table.cleanup_expired(Duration::from_secs(0)); // Expire everything
        assert!(table.get_by_code("ABC123").is_none());
    }
}