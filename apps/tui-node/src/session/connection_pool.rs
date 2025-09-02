use std::sync::Arc;
use std::time::{Duration, Instant};
use dashmap::DashMap;
use anyhow::{Result, Context};
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::api::APIBuilder;
use webrtc::data_channel::RTCDataChannel;
use futures::future::join_all;

/// Connection state tracking
#[derive(Clone)]
pub struct ConnectionInfo {
    pub peer: String,
    pub connection: Arc<RTCPeerConnection>,
    pub data_channel: Option<Arc<RTCDataChannel>>,
    pub created_at: Instant,
    pub last_used: Instant,
    pub is_healthy: bool,
}

/// Connection pool for WebRTC connections
pub struct ConnectionPool {
    /// Active connections mapped by peer ID
    connections: Arc<DashMap<String, ConnectionInfo>>,
    /// Idle timeout for connections
    idle_timeout: Duration,
    /// WebRTC API
    api: Arc<webrtc::api::API>,
}

impl ConnectionPool {
    pub fn new() -> Self {
        // Create WebRTC API
        let api = APIBuilder::new().build();
        
        Self {
            connections: Arc::new(DashMap::new()),
            idle_timeout: Duration::from_secs(300), // 5 minutes
            api: Arc::new(api),
        }
    }
    
    /// Get or create a connection to a peer
    pub async fn get_or_create(&self, peer: &str) -> Result<Arc<RTCPeerConnection>> {
        // Check if we have an existing healthy connection
        if let Some(mut conn_info) = self.connections.get_mut(peer) {
            if conn_info.is_healthy && conn_info.last_used.elapsed() < self.idle_timeout {
                tracing::debug!("Reusing existing connection to {}", peer);
                conn_info.last_used = Instant::now();
                return Ok(conn_info.connection.clone());
            } else {
                tracing::debug!("Connection to {} is stale, creating new one", peer);
                // Remove stale connection
                drop(conn_info);
                self.connections.remove(peer);
            }
        }
        
        // Create new connection
        tracing::debug!("Creating new connection to {}", peer);
        self.create_connection(peer).await
    }
    
    /// Create a new WebRTC connection
    async fn create_connection(&self, peer: &str) -> Result<Arc<RTCPeerConnection>> {
        // Configure ICE servers
        let config = RTCConfiguration {
            ice_servers: vec![
                RTCIceServer {
                    urls: vec!["stun:stun.l.google.com:19302".to_string()],
                    ..Default::default()
                },
                RTCIceServer {
                    urls: vec!["stun:stun1.l.google.com:19302".to_string()],
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        
        // Create peer connection
        let pc = Arc::new(
            self.api
                .new_peer_connection(config)
                .await
                .context("Failed to create peer connection")?
        );
        
        // Create data channel for MPC messages
        let dc = pc.create_data_channel("mpc", None).await
            .context("Failed to create data channel")?;
        
        // Store connection info
        let conn_info = ConnectionInfo {
            peer: peer.to_string(),
            connection: pc.clone(),
            data_channel: Some(dc),
            created_at: Instant::now(),
            last_used: Instant::now(),
            is_healthy: true,
        };
        
        self.connections.insert(peer.to_string(), conn_info);
        
        // Set up connection monitoring
        self.monitor_connection(peer.to_string(), pc.clone());
        
        Ok(pc)
    }
    
    /// Monitor connection health
    fn monitor_connection(&self, peer: String, connection: Arc<RTCPeerConnection>) {
        let connections = self.connections.clone();
        let idle_timeout = self.idle_timeout;
        
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(30)).await;
                
                // Check connection state
                let state = connection.connection_state();
                let is_healthy = matches!(
                    state,
                    webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState::Connected
                );
                
                // Update health status
                if let Some(mut conn_info) = connections.get_mut(&peer) {
                    conn_info.is_healthy = is_healthy;
                    
                    // Remove if idle for too long
                    if conn_info.last_used.elapsed() > idle_timeout {
                        tracing::debug!("Removing idle connection to {}", peer);
                        drop(conn_info);
                        connections.remove(&peer);
                        break;
                    }
                    
                    // Remove if unhealthy
                    if !is_healthy {
                        tracing::debug!("Removing unhealthy connection to {}", peer);
                        drop(conn_info);
                        connections.remove(&peer);
                        break;
                    }
                } else {
                    // Connection was removed
                    break;
                }
            }
        });
    }
    
    /// Establish mesh connections with multiple peers in parallel
    pub async fn establish_mesh(&self, peers: Vec<String>) -> Result<()> {
        if peers.is_empty() {
            return Ok(());
        }
        
        tracing::info!("Establishing mesh with {} peers", peers.len());
        
        // Create connections in parallel
        let futures: Vec<_> = peers
            .iter()
            .map(|peer| self.get_or_create(peer))
            .collect();
        
        let results = join_all(futures).await;
        
        // Check for errors
        for (i, result) in results.into_iter().enumerate() {
            if let Err(e) = result {
                tracing::error!("Failed to connect to {}: {}", peers[i], e);
                // Continue with other connections rather than failing entirely
            }
        }
        
        tracing::info!("Mesh establishment complete");
        Ok(())
    }
    
    /// Get connection info for a peer
    pub fn get_connection_info(&self, peer: &str) -> Option<ConnectionInfo> {
        self.connections.get(peer).map(|entry| entry.clone())
    }
    
    /// Check if we have a healthy connection to a peer
    pub fn is_connected(&self, peer: &str) -> bool {
        self.connections
            .get(peer)
            .map(|conn| conn.is_healthy)
            .unwrap_or(false)
    }
    
    /// Get all connected peers
    pub fn get_connected_peers(&self) -> Vec<String> {
        self.connections
            .iter()
            .filter(|entry| entry.is_healthy)
            .map(|entry| entry.key().clone())
            .collect()
    }
    
    /// Remove a connection
    pub fn remove_connection(&self, peer: &str) {
        if let Some((_, conn_info)) = self.connections.remove(peer) {
            tracing::debug!("Removed connection to {}", peer);
            // Close the connection
            if let Some(dc) = conn_info.data_channel {
                let _ = dc.close();
            }
            let _ = conn_info.connection.close();
        }
    }
    
    /// Clean up all connections
    pub async fn cleanup(&self) {
        tracing::info!("Cleaning up connection pool");
        
        // Close all connections
        for entry in self.connections.iter() {
            let conn_info = entry.value();
            if let Some(dc) = &conn_info.data_channel {
                let _ = dc.close().await;
            }
            let _ = conn_info.connection.close().await;
        }
        
        // Clear the pool
        self.connections.clear();
    }
    
    /// Get pool statistics
    pub fn get_stats(&self) -> ConnectionPoolStats {
        let total = self.connections.len();
        let healthy = self.connections
            .iter()
            .filter(|entry| entry.is_healthy)
            .count();
        
        ConnectionPoolStats {
            total_connections: total,
            healthy_connections: healthy,
            unhealthy_connections: total - healthy,
        }
    }
}

/// Statistics about the connection pool
#[derive(Clone)]
pub struct ConnectionPoolStats {
    pub total_connections: usize,
    pub healthy_connections: usize,
    pub unhealthy_connections: usize,
}

/// Extension trait for RTCPeerConnection
#[async_trait::async_trait]
pub trait ConnectionExt {
    async fn create_offer(&self) -> Result<RTCSessionDescription>;
    async fn create_answer(&self) -> Result<RTCSessionDescription>;
    async fn set_remote_description(&self, desc: RTCSessionDescription) -> Result<()>;
    async fn set_local_description(&self, desc: RTCSessionDescription) -> Result<()>;
}

#[async_trait::async_trait]
impl ConnectionExt for RTCPeerConnection {
    async fn create_offer(&self) -> Result<RTCSessionDescription> {
        RTCPeerConnection::create_offer(self, None)
            .await
            .context("Failed to create offer")
    }
    
    async fn create_answer(&self) -> Result<RTCSessionDescription> {
        self.create_answer(None)
            .await
            .context("Failed to create answer")
    }
    
    async fn set_remote_description(&self, desc: RTCSessionDescription) -> Result<()> {
        RTCPeerConnection::set_remote_description(self, desc)
            .await
            .context("Failed to set remote description")
    }
    
    async fn set_local_description(&self, desc: RTCSessionDescription) -> Result<()> {
        RTCPeerConnection::set_local_description(self, desc)
            .await
            .context("Failed to set local description")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_connection_pool_creation() {
        let pool = ConnectionPool::new();
        assert_eq!(pool.get_stats().total_connections, 0);
    }
    
    #[tokio::test]
    async fn test_connection_reuse() {
        let pool = ConnectionPool::new();
        
        // Create first connection
        let conn1 = pool.get_or_create("peer1").await;
        assert!(conn1.is_ok());
        assert_eq!(pool.get_stats().total_connections, 1);
        
        // Should reuse existing connection
        let conn2 = pool.get_or_create("peer1").await;
        assert!(conn2.is_ok());
        assert_eq!(pool.get_stats().total_connections, 1);
    }
    
    #[tokio::test]
    async fn test_parallel_mesh_establishment() {
        let pool = ConnectionPool::new();
        
        let peers = vec![
            "peer1".to_string(),
            "peer2".to_string(),
            "peer3".to_string(),
        ];
        
        let result = pool.establish_mesh(peers).await;
        assert!(result.is_ok());
        
        // Should have created 3 connections
        assert_eq!(pool.get_stats().total_connections, 3);
    }
    
    #[tokio::test]
    async fn test_connection_removal() {
        let pool = ConnectionPool::new();
        
        // Create connection
        let _ = pool.get_or_create("peer1").await;
        assert_eq!(pool.get_stats().total_connections, 1);
        
        // Remove connection
        pool.remove_connection("peer1");
        assert_eq!(pool.get_stats().total_connections, 0);
    }
    
    #[tokio::test]
    async fn test_cleanup() {
        let pool = ConnectionPool::new();
        
        // Create multiple connections
        let _ = pool.get_or_create("peer1").await;
        let _ = pool.get_or_create("peer2").await;
        assert_eq!(pool.get_stats().total_connections, 2);
        
        // Cleanup should remove all
        pool.cleanup().await;
        assert_eq!(pool.get_stats().total_connections, 0);
    }
}