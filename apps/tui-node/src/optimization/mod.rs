// MPC Wallet Performance Optimization Module
// This module contains optimized implementations for critical performance bottlenecks

pub mod connection_pool;
pub mod message_batcher;
pub mod state_manager;
pub mod deduplicator;
pub mod bounded_channel;
pub mod lockfree_state;
pub mod message_batcher_v2;
pub mod performance_monitor;

use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};

/// Performance metrics collector
pub struct PerformanceMetrics {
    session_join_times: Arc<RwLock<Vec<Duration>>>,
    message_counts: Arc<RwLock<HashMap<String, u64>>>,
    connection_attempts: Arc<RwLock<HashMap<String, u32>>>,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            session_join_times: Arc::new(RwLock::new(Vec::new())),
            message_counts: Arc::new(RwLock::new(HashMap::new())),
            connection_attempts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn record_session_join(&self, duration: Duration) {
        let mut times = self.session_join_times.write().await;
        times.push(duration);
        
        // Keep only last 1000 measurements
        if times.len() > 1000 {
            times.remove(0);
        }
    }

    pub async fn increment_message_count(&self, message_type: &str) {
        let mut counts = self.message_counts.write().await;
        *counts.entry(message_type.to_string()).or_insert(0) += 1;
    }

    pub async fn record_connection_attempt(&self, device_id: &str, success: bool) {
        let mut attempts = self.connection_attempts.write().await;
        let entry = attempts.entry(device_id.to_string()).or_insert(0);
        if success {
            *entry = 0; // Reset on success
        } else {
            *entry += 1;
        }
    }

    pub async fn get_p95_session_join_time(&self) -> Option<Duration> {
        let times = self.session_join_times.read().await;
        if times.is_empty() {
            return None;
        }
        
        let mut sorted = times.clone();
        sorted.sort();
        let index = (sorted.len() as f64 * 0.95) as usize;
        sorted.get(index).cloned()
    }
}

/// Optimized session manager with efficient state handling
pub struct OptimizedSessionManager {
    metrics: Arc<PerformanceMetrics>,
    deduplicator: Arc<deduplicator::MessageDeduplicator>,
    batcher: Arc<message_batcher::MessageBatcher>,
    pool: Arc<connection_pool::ConnectionPool>,
}

impl OptimizedSessionManager {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(PerformanceMetrics::new()),
            deduplicator: Arc::new(deduplicator::MessageDeduplicator::new(
                Duration::from_secs(300), // 5 minute TTL
                10000, // Max 10k messages in cache
            )),
            batcher: Arc::new(message_batcher::MessageBatcher::new(
                50, // Batch size
                Duration::from_millis(100), // Flush interval
            )),
            pool: Arc::new(connection_pool::ConnectionPool::new(
                connection_pool::PoolConfig {
                    max_connections: 100,
                    idle_timeout: Duration::from_secs(300),
                    retry_limit: 3,
                    parallel_attempts: 3,
                },
            )),
        }
    }

    /// Optimized session join with parallel operations
    pub async fn join_session_optimized(
        &self,
        session_id: String,
        participants: Vec<String>,
        self_device_id: String,
    ) -> Result<Duration, String> {
        let start = Instant::now();
        
        // Parallel operations using tokio::join!
        let (dedupe_result, connection_results) = tokio::join!(
            // Check for duplicate session join
            async {
                let msg_id = format!("join:{}:{}", session_id, self_device_id);
                self.deduplicator.should_process(&msg_id).await
            },
            // Establish connections in parallel
            async {
                let mut handles = vec![];
                for participant in &participants {
                    if participant != &self_device_id {
                        let pool = self.pool.clone();
                        let participant = participant.clone();
                        handles.push(tokio::spawn(async move {
                            pool.get_or_create(&participant).await
                        }));
                    }
                }
                futures::future::join_all(handles).await
            }
        );

        if !dedupe_result {
            return Err("Duplicate session join detected".to_string());
        }

        // Process connection results
        let mut successful_connections = 0;
        for result in connection_results {
            match result {
                Ok(Ok(_)) => successful_connections += 1,
                _ => {}
            }
        }

        let duration = start.elapsed();
        self.metrics.record_session_join(duration).await;
        
        Ok(duration)
    }

    /// Batch message sending for efficiency
    pub async fn send_message_batched(
        &self,
        target: String,
        message: serde_json::Value,
    ) -> Result<(), String> {
        self.metrics.increment_message_count("batched_send").await;
        self.batcher.send(target, message).await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_metrics() {
        let metrics = PerformanceMetrics::new();
        
        // Record some session join times
        for i in 1..=10 {
            metrics.record_session_join(Duration::from_secs(i)).await;
        }
        
        // Check P95 calculation
        let p95 = metrics.get_p95_session_join_time().await;
        assert!(p95.is_some());
        assert_eq!(p95.unwrap(), Duration::from_secs(10));
    }

    #[tokio::test]
    async fn test_optimized_session_join() {
        let manager = OptimizedSessionManager::new();
        
        let participants = vec![
            "device1".to_string(),
            "device2".to_string(),
            "device3".to_string(),
        ];
        
        let result = manager.join_session_optimized(
            "test_session".to_string(),
            participants,
            "device1".to_string(),
        ).await;
        
        assert!(result.is_ok());
        assert!(result.unwrap() < Duration::from_secs(5));
    }
}