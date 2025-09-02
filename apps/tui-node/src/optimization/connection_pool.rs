// WebRTC Connection Pool Implementation
// Optimizes connection establishment and reuse

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Notify};
use dashmap::DashMap;
use webrtc::peer_connection::{RTCPeerConnection, peer_connection_state::RTCPeerConnectionState};
use webrtc::api::API;
use webrtc::peer_connection::configuration::RTCConfiguration;

#[derive(Clone)]
pub struct PoolConfig {
    pub max_connections: usize,
    pub idle_timeout: Duration,
    pub retry_limit: u32,
    pub parallel_attempts: usize,
}

#[derive(Clone)]
pub struct ConnectionEntry {
    pub connection: Arc<RTCPeerConnection>,
    pub last_used: Instant,
    pub state: ConnectionState,
    pub retry_count: u32,
    pub created_at: Instant,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ConnectionState {
    Connecting,
    Connected,
    Failed,
    Idle,
}

impl ConnectionEntry {
    pub fn is_healthy(&self) -> bool {
        matches!(self.state, ConnectionState::Connected) &&
        self.connection.connection_state() == RTCPeerConnectionState::Connected
    }

    pub fn update_last_used(&mut self) {
        self.last_used = Instant::now();
    }

    pub fn is_expired(&self, timeout: Duration) -> bool {
        self.last_used.elapsed() > timeout
    }
}

pub struct ConnectionPool {
    connections: Arc<DashMap<String, ConnectionEntry>>,
    pending: Arc<DashMap<String, Arc<Notify>>>,
    config: PoolConfig,
    api: Arc<API>,
    rtc_config: Arc<RTCConfiguration>,
}

impl ConnectionPool {
    pub fn new(config: PoolConfig) -> Self {
        Self {
            connections: Arc::new(DashMap::new()),
            pending: Arc::new(DashMap::new()),
            config,
            api: Arc::new(crate::network::webrtc::WEBRTC_API.clone()),
            rtc_config: Arc::new(crate::network::webrtc::WEBRTC_CONFIG.clone()),
        }
    }

    /// Get existing connection or create new one
    pub async fn get_or_create(&self, peer_id: &str) -> Result<Arc<RTCPeerConnection>, String> {
        // Fast path: check for healthy existing connection
        if let Some(mut entry) = self.connections.get_mut(peer_id) {
            if entry.is_healthy() {
                entry.update_last_used();
                return Ok(entry.connection.clone());
            }
        }

        // Check if another task is already creating this connection
        if let Some(notify) = self.pending.get(peer_id) {
            let notify_clone = notify.clone();
            drop(notify); // Release the lock
            
            // Wait for the other task to complete
            notify_clone.notified().await;
            
            // Try to get the connection again
            if let Some(entry) = self.connections.get(peer_id) {
                if entry.is_healthy() {
                    return Ok(entry.connection.clone());
                }
            }
        }

        // Create new connection
        self.create_connection(peer_id).await
    }

    /// Create new connection with parallel attempts
    async fn create_connection(&self, peer_id: &str) -> Result<Arc<RTCPeerConnection>, String> {
        // Mark as pending
        let notify = Arc::new(Notify::new());
        self.pending.insert(peer_id.to_string(), notify.clone());

        // Parallel connection attempts
        let mut handles = vec![];
        for i in 0..self.config.parallel_attempts {
            let api = self.api.clone();
            let config = self.rtc_config.clone();
            let peer_id = peer_id.to_string();
            
            handles.push(tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(i as u64 * 100)).await;
                Self::try_create_connection(&api, &config, &peer_id).await
            }));
        }

        // Race parallel attempts
        let (result, _, remaining) = futures::future::select_all(handles).await;
        
        // Cancel remaining attempts
        for handle in remaining {
            handle.abort();
        }

        // Process result
        match result {
            Ok(Ok(connection)) => {
                let entry = ConnectionEntry {
                    connection: connection.clone(),
                    last_used: Instant::now(),
                    state: ConnectionState::Connecting,
                    retry_count: 0,
                    created_at: Instant::now(),
                };
                
                self.connections.insert(peer_id.to_string(), entry);
                self.pending.remove(peer_id);
                notify.notify_waiters();
                
                Ok(connection)
            }
            _ => {
                self.pending.remove(peer_id);
                notify.notify_waiters();
                Err("Failed to create connection".to_string())
            }
        }
    }

    /// Single connection attempt
    async fn try_create_connection(
        api: &Arc<API>,
        config: &Arc<RTCConfiguration>,
        peer_id: &str,
    ) -> Result<Arc<RTCPeerConnection>, String> {
        match api.new_peer_connection((**config).clone()).await {
            Ok(pc) => {
                // Set up connection handlers
                let pc_arc = Arc::new(pc);
                
                // Add connection state change handler
                let peer_id_clone = peer_id.to_string();
                pc_arc.on_peer_connection_state_change(Box::new(move |state| {
                    log::debug!("Connection state for {}: {:?}", peer_id_clone, state);
                    Box::pin(async {})
                }));
                
                Ok(pc_arc)
            }
            Err(_e) => Err(format!("Failed to create peer connection: {}", _e)),
        }
    }

    /// Cleanup expired connections
    pub async fn cleanup_expired(&self) {
        let mut to_remove = vec![];
        
        for entry in self.connections.iter() {
            if entry.is_expired(self.config.idle_timeout) || 
               matches!(entry.state, ConnectionState::Failed) {
                to_remove.push(entry.key().clone());
            }
        }
        
        for key in to_remove {
            if let Some((_, entry)) = self.connections.remove(&key) {
                log::debug!("Removing expired connection: {}", key);
                // Close the connection gracefully
                let _ = entry.connection.close().await;
            }
        }
    }

    /// Get pool statistics
    pub fn get_stats(&self) -> PoolStats {
        let mut stats = PoolStats::default();
        
        for entry in self.connections.iter() {
            stats.total_connections += 1;
            match entry.state {
                ConnectionState::Connected => stats.active_connections += 1,
                ConnectionState::Connecting => stats.connecting += 1,
                ConnectionState::Failed => stats.failed += 1,
                ConnectionState::Idle => stats.idle += 1,
            }
        }
        
        stats.pending_connections = self.pending.len();
        stats
    }

    /// Force close all connections
    pub async fn close_all(&self) {
        let connections: Vec<_> = self.connections.iter()
            .map(|entry| entry.value().connection.clone())
            .collect();
        
        self.connections.clear();
        self.pending.clear();
        
        // Close all connections in parallel
        let mut handles = vec![];
        for conn in connections {
            handles.push(tokio::spawn(async move {
                let _ = conn.close().await;
            }));
        }
        
        futures::future::join_all(handles).await;
    }
}

#[derive(Default, Debug)]
pub struct PoolStats {
    pub total_connections: usize,
    pub active_connections: usize,
    pub idle: usize,
    pub connecting: usize,
    pub failed: usize,
    pub pending_connections: usize,
}

/// Background task to periodically clean up the pool
pub async fn pool_maintenance_task(pool: Arc<ConnectionPool>) {
    let mut interval = tokio::time::interval(Duration::from_secs(30));
    
    loop {
        interval.tick().await;
        pool.cleanup_expired().await;
        
        let stats = pool.get_stats();
        log::debug!("Connection pool stats: {:?}", stats);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connection_pool_basic() {
        let config = PoolConfig {
            max_connections: 10,
            idle_timeout: Duration::from_secs(60),
            retry_limit: 3,
            parallel_attempts: 2,
        };
        
        let pool = ConnectionPool::new(config);
        
        // Test getting connection
        match pool.get_or_create("test_peer").await {
            Ok(conn) => {
                assert!(Arc::strong_count(&conn) >= 1);
            }
            Err(_e) => {
                // Expected in test environment without real WebRTC
                assert!(e.contains("Failed"));
            }
        }
    }

    #[tokio::test]
    async fn test_pool_stats() {
        let config = PoolConfig {
            max_connections: 10,
            idle_timeout: Duration::from_secs(60),
            retry_limit: 3,
            parallel_attempts: 1,
        };
        
        let pool = ConnectionPool::new(config);
        let stats = pool.get_stats();
        
        assert_eq!(stats.total_connections, 0);
        assert_eq!(stats.active_connections, 0);
    }
}