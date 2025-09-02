// Lock-free state management using DashMap for high-concurrency access
use dashmap::DashMap;
use std::sync::Arc;
use webrtc::{
    peer_connection::RTCPeerConnection,
    peer_connection::peer_connection_state::RTCPeerConnectionState,
    data_channel::RTCDataChannel,
    ice_transport::ice_candidate::RTCIceCandidateInit,
};
use std::time::Instant;
use tracing::{debug, warn};

/// Lock-free connection state manager
pub struct LockFreeConnectionState {
    /// Device connections without locks
    pub connections: Arc<DashMap<String, ConnectionInfo>>,
    /// Device statuses without locks
    pub statuses: Arc<DashMap<String, RTCPeerConnectionState>>,
    /// Data channels without locks
    pub data_channels: Arc<DashMap<String, Arc<RTCDataChannel>>>,
    /// Pending ICE candidates
    pub pending_candidates: Arc<DashMap<String, Vec<RTCIceCandidateInit>>>,
    /// Connection health tracking
    health_tracker: Arc<DashMap<String, HealthInfo>>,
}

#[derive(Clone)]
pub struct ConnectionInfo {
    pub connection: Arc<RTCPeerConnection>,
    pub created_at: Instant,
    pub last_active: Instant,
}

#[derive(Clone)]
pub struct HealthInfo {
    pub last_ping: Instant,
    pub last_pong: Instant,
    pub consecutive_failures: u32,
    pub average_latency_ms: f64,
}

impl LockFreeConnectionState {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(DashMap::new()),
            statuses: Arc::new(DashMap::new()),
            data_channels: Arc::new(DashMap::new()),
            pending_candidates: Arc::new(DashMap::new()),
            health_tracker: Arc::new(DashMap::new()),
        }
    }
    
    /// Add or update a connection
    pub fn upsert_connection(&self, device_id: String, connection: Arc<RTCPeerConnection>) {
        let now = Instant::now();
        let info = ConnectionInfo {
            connection,
            created_at: now,
            last_active: now,
        };
        
        self.connections.insert(device_id.clone(), info);
        self.statuses.insert(device_id.clone(), RTCPeerConnectionState::New);
        
        // Initialize health tracking
        let health = HealthInfo {
            last_ping: now,
            last_pong: now,
            consecutive_failures: 0,
            average_latency_ms: 0.0,
        };
        self.health_tracker.insert(device_id, health);
        
        debug!("Connection added for device: {}", device_id);
    }
    
    /// Get connection without locking
    pub fn get_connection(&self, device_id: &str) -> Option<Arc<RTCPeerConnection>> {
        self.connections.get(device_id)
            .map(|entry| {
                // Update last active time
                let mut info = entry.clone();
                info.last_active = Instant::now();
                drop(entry); // Release the reference before re-inserting
                self.connections.insert(device_id.to_string(), info.clone());
                info.connection
            })
    }
    
    /// Update connection status
    pub fn update_status(&self, device_id: String, status: RTCPeerConnectionState) {
        self.statuses.insert(device_id.clone(), status);
        
        // Update health based on status
        if matches!(status, RTCPeerConnectionState::Failed | RTCPeerConnectionState::Disconnected) {
            if let Some(mut health) = self.health_tracker.get_mut(&device_id) {
                health.consecutive_failures += 1;
            }
        } else if matches!(status, RTCPeerConnectionState::Connected) {
            if let Some(mut health) = self.health_tracker.get_mut(&device_id) {
                health.consecutive_failures = 0;
            }
        }
    }
    
    /// Add pending ICE candidate
    pub fn add_pending_candidate(&self, device_id: String, candidate: RTCIceCandidateInit) {
        self.pending_candidates
            .entry(device_id)
            .and_modify(|candidates| candidates.push(candidate.clone()))
            .or_insert_with(|| vec![candidate]);
    }
    
    /// Take and clear pending candidates
    pub fn take_pending_candidates(&self, device_id: &str) -> Vec<RTCIceCandidateInit> {
        self.pending_candidates
            .remove(device_id)
            .map(|(_, candidates)| candidates)
            .unwrap_or_default()
    }
    
    /// Clean up stale connections
    pub fn cleanup_stale_connections(&self, max_age: std::time::Duration) {
        let now = Instant::now();
        let mut to_remove = Vec::new();
        
        for entry in self.connections.iter() {
            let device_id = entry.key();
            let info = entry.value();
            
            if now.duration_since(info.last_active) > max_age {
                to_remove.push(device_id.clone());
            }
        }
        
        for device_id in to_remove {
            self.remove_connection(&device_id);
            warn!("Removed stale connection for device: {}", device_id);
        }
    }
    
    /// Remove a connection and all associated data
    pub fn remove_connection(&self, device_id: &str) {
        self.connections.remove(device_id);
        self.statuses.remove(device_id);
        self.data_channels.remove(device_id);
        self.pending_candidates.remove(device_id);
        self.health_tracker.remove(device_id);
    }
    
    /// Get all connected devices
    pub fn get_connected_devices(&self) -> Vec<String> {
        self.statuses
            .iter()
            .filter(|entry| matches!(entry.value(), RTCPeerConnectionState::Connected))
            .map(|entry| entry.key().clone())
            .collect()
    }
    
    /// Update health metrics
    pub fn update_health(&self, device_id: &str, latency_ms: f64) {
        if let Some(mut health) = self.health_tracker.get_mut(device_id) {
            let alpha = 0.1; // Exponential moving average factor
            health.average_latency_ms = alpha * latency_ms + (1.0 - alpha) * health.average_latency_ms;
            health.last_pong = Instant::now();
        }
    }
    
    /// Check if a device is healthy
    pub fn is_healthy(&self, device_id: &str, max_failures: u32, max_latency_ms: f64) -> bool {
        self.health_tracker
            .get(device_id)
            .map(|health| {
                health.consecutive_failures < max_failures && 
                health.average_latency_ms < max_latency_ms
            })
            .unwrap_or(false)
    }
    
    /// Get statistics for monitoring
    pub fn get_stats(&self) -> ConnectionStats {
        let total = self.connections.len();
        let connected = self.get_connected_devices().len();
        let with_channels = self.data_channels.len();
        
        let avg_latency = if !self.health_tracker.is_empty() {
            let sum: f64 = self.health_tracker
                .iter()
                .map(|entry| entry.value().average_latency_ms)
                .sum();
            sum / self.health_tracker.len() as f64
        } else {
            0.0
        };
        
        ConnectionStats {
            total_connections: total,
            connected_count: connected,
            with_data_channels: with_channels,
            average_latency_ms: avg_latency,
        }
    }
}

#[derive(Debug)]
pub struct ConnectionStats {
    pub total_connections: usize,
    pub connected_count: usize,
    pub with_data_channels: usize,
    pub average_latency_ms: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lockfree_operations() {
        let state = LockFreeConnectionState::new();
        
        // Test concurrent access (would deadlock with regular mutex)
        let state1 = state.connections.clone();
        let state2 = state.connections.clone();
        
        // These can run concurrently without locks
        std::thread::scope(|s| {
            s.spawn(|| {
                for i in 0..100 {
                    let _ = state1.get(&format!("device_{}", i));
                }
            });
            
            s.spawn(|| {
                for i in 0..100 {
                    let _ = state2.get(&format!("device_{}", i));
                }
            });
        });
    }
    
    #[test]
    fn test_health_tracking() {
        let state = LockFreeConnectionState::new();
        
        // Simulate connection lifecycle
        let device_id = "test_device".to_string();
        state.update_status(device_id.clone(), RTCPeerConnectionState::Connected);
        
        // Update health metrics
        state.update_health(&device_id, 25.0);
        state.update_health(&device_id, 30.0);
        
        // Check health
        assert!(state.is_healthy(&device_id, 5, 50.0));
        
        // Simulate failures
        state.update_status(device_id.clone(), RTCPeerConnectionState::Failed);
        assert!(!state.is_healthy(&device_id, 1, 50.0));
    }
}