// Message Deduplication Implementation
// Prevents duplicate message processing and connection thrashing

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use lru::LruCache;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

pub type MessageId = String;

#[derive(Clone)]
struct SeenMessage {
    timestamp: Instant,
    hash: u64,
    count: u32,
}

pub struct MessageDeduplicator {
    seen: Arc<RwLock<LruCache<MessageId, SeenMessage>>>,
    ttl: Duration,
    max_entries: usize,
    cleanup_interval: Duration,
}

impl MessageDeduplicator {
    pub fn new(ttl: Duration, max_entries: usize) -> Self {
        let dedup = Self {
            seen: Arc::new(RwLock::new(LruCache::new(
                std::num::NonZeroUsize::new(max_entries).unwrap()
            ))),
            ttl,
            max_entries,
            cleanup_interval: Duration::from_secs(60), // Cleanup every minute
        };

        // Start cleanup task
        let seen_clone = dedup.seen.clone();
        let ttl = dedup.ttl;
        let cleanup_interval = dedup.cleanup_interval;
        
        tokio::spawn(async move {
            Self::cleanup_task(seen_clone, ttl, cleanup_interval).await;
        });

        dedup
    }

    /// Check if a message should be processed
    /// Returns true if this is the first time seeing the message
    pub async fn should_process(&self, msg_id: &str) -> bool {
        self.should_process_with_hash(msg_id, 0).await
    }

    /// Check if a message should be processed, with content hash for deeper deduplication
    pub async fn should_process_with_hash(&self, msg_id: &str, content_hash: u64) -> bool {
        let mut cache = self.seen.write().await;
        let now = Instant::now();
        
        // Check if already seen
        if let Some(seen_msg) = cache.get_mut(msg_id) {
            // Check if it's the same content (by hash)
            if content_hash > 0 && seen_msg.hash != content_hash {
                // Different content with same ID - this is suspicious
                log::warn!(
                    "Message ID {} seen with different content hash. Old: {}, New: {}",
                    msg_id, seen_msg.hash, content_hash
                );
            }
            
            // Update timestamp and count
            seen_msg.timestamp = now;
            seen_msg.count += 1;
            
            // Log if we're seeing many duplicates
            if seen_msg.count % 10 == 0 {
                log::warn!(
                    "Message ID {} has been seen {} times",
                    msg_id, seen_msg.count
                );
            }
            
            false
        } else {
            // Add to cache
            cache.put(msg_id.to_string(), SeenMessage {
                timestamp: now,
                hash: content_hash,
                count: 1,
            });
            true
        }
    }

    /// Generate a message ID from components
    pub fn generate_message_id(
        message_type: &str,
        from: &str,
        to: &str,
        sequence: Option<u64>,
    ) -> MessageId {
        if let Some(seq) = sequence {
            format!("{}:{}:{}:{}", message_type, from, to, seq)
        } else {
            // For messages without sequence, include timestamp for uniqueness
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_micros();
            format!("{}:{}:{}:{}", message_type, from, to, timestamp)
        }
    }

    /// Calculate hash of message content for deep deduplication
    pub fn calculate_content_hash<T: Hash>(content: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }

    /// Manual cleanup of expired entries
    pub async fn cleanup(&self) {
        let mut cache = self.seen.write().await;
        let now = Instant::now();
        
        // LRU cache doesn't have retain method, so we need to collect and re-insert
        let mut to_keep = Vec::new();
        
        // Iterate through all entries
        while let Some((key, value)) = cache.pop_lru() {
            if now.duration_since(value.timestamp) < self.ttl {
                to_keep.push((key, value));
            }
        }
        
        // Re-insert non-expired entries
        for (key, value) in to_keep.into_iter().rev() {
            cache.put(key, value);
        }
    }

    /// Background cleanup task
    async fn cleanup_task(
        seen: Arc<RwLock<LruCache<MessageId, SeenMessage>>>,
        ttl: Duration,
        interval: Duration,
    ) {
        let mut interval = tokio::time::interval(interval);
        
        loop {
            interval.tick().await;
            
            let mut cache = seen.write().await;
            let now = Instant::now();
            let mut expired_count = 0;
            
            // Collect expired keys
            let mut expired_keys = Vec::new();
            
            // Peek at all entries without removing
            let entries: Vec<_> = cache.iter()
                .map(|(k, v)| (k.clone(), v.timestamp))
                .collect();
            
            for (key, timestamp) in entries {
                if now.duration_since(timestamp) >= ttl {
                    expired_keys.push(key);
                    expired_count += 1;
                }
            }
            
            // Remove expired entries
            for key in expired_keys {
                cache.pop(&key);
            }
            
            if expired_count > 0 {
                log::debug!("Cleaned up {} expired message IDs", expired_count);
            }
            
            drop(cache); // Explicitly drop to release the lock
        }
    }

    /// Get deduplication statistics
    pub async fn get_stats(&self) -> DeduplicationStats {
        let cache = self.seen.read().await;
        let now = Instant::now();
        
        let mut total_duplicates = 0u64;
        let mut active_entries = 0usize;
        let mut expired_entries = 0usize;
        
        for (_, seen_msg) in cache.iter() {
            if seen_msg.count > 1 {
                total_duplicates += (seen_msg.count - 1) as u64;
            }
            
            if now.duration_since(seen_msg.timestamp) < self.ttl {
                active_entries += 1;
            } else {
                expired_entries += 1;
            }
        }
        
        DeduplicationStats {
            total_entries: cache.len(),
            active_entries,
            expired_entries,
            total_duplicates,
            cache_utilization: (cache.len() as f64 / self.max_entries as f64) * 100.0,
        }
    }
}

#[derive(Debug)]
pub struct DeduplicationStats {
    pub total_entries: usize,
    pub active_entries: usize,
    pub expired_entries: usize,
    pub total_duplicates: u64,
    pub cache_utilization: f64,
}

/// Connection-specific deduplicator to prevent thrashing
pub struct ConnectionDeduplicator {
    dedup: MessageDeduplicator,
}

impl ConnectionDeduplicator {
    pub fn new() -> Self {
        Self {
            dedup: MessageDeduplicator::new(
                Duration::from_secs(30), // Shorter TTL for connections
                1000, // Smaller cache for connection events
            ),
        }
    }

    /// Check if we should attempt a connection
    pub async fn should_attempt_connection(&self, from: &str, to: &str) -> bool {
        let msg_id = format!("conn:{}:{}", from.min(to), from.max(to));
        self.dedup.should_process(&msg_id).await
    }

    /// Check if we should process a WebRTC signal
    pub async fn should_process_signal(
        &self,
        signal_type: &str,
        from: &str,
        to: &str,
    ) -> bool {
        let msg_id = format!("signal:{}:{}:{}", signal_type, from, to);
        self.dedup.should_process(&msg_id).await
    }
}

/// Session-specific deduplicator for session management
pub struct SessionDeduplicator {
    dedup: MessageDeduplicator,
}

impl SessionDeduplicator {
    pub fn new() -> Self {
        Self {
            dedup: MessageDeduplicator::new(
                Duration::from_secs(300), // 5 minute TTL for session messages
                5000, // Larger cache for session events
            ),
        }
    }

    /// Check if we should process a session proposal
    pub async fn should_process_proposal(&self, session_id: &str, from: &str) -> bool {
        let msg_id = format!("proposal:{}:{}", session_id, from);
        self.dedup.should_process(&msg_id).await
    }

    /// Check if we should process a session response
    pub async fn should_process_response(
        &self,
        session_id: &str,
        from: &str,
        accepted: bool,
    ) -> bool {
        let msg_id = format!("response:{}:{}:{}", session_id, from, accepted);
        self.dedup.should_process(&msg_id).await
    }

    /// Check if we should process a session update
    pub async fn should_process_update(
        &self,
        session_id: &str,
        update_type: &str,
        sequence: u64,
    ) -> bool {
        let msg_id = format!("update:{}:{}:{}", session_id, update_type, sequence);
        self.dedup.should_process(&msg_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_deduplication() {
        let dedup = MessageDeduplicator::new(Duration::from_secs(60), 100);
        
        // First time should process
        assert!(dedup.should_process("msg1").await);
        
        // Second time should not process
        assert!(!dedup.should_process("msg1").await);
        
        // Different message should process
        assert!(dedup.should_process("msg2").await);
    }

    #[tokio::test]
    async fn test_content_hash_deduplication() {
        let dedup = MessageDeduplicator::new(Duration::from_secs(60), 100);
        
        let content1 = "Hello World";
        let content2 = "Hello World!";
        
        let hash1 = MessageDeduplicator::calculate_content_hash(&content1);
        let hash2 = MessageDeduplicator::calculate_content_hash(&content2);
        
        // Different content should have different hashes
        assert_ne!(hash1, hash2);
        
        // First message processes
        assert!(dedup.should_process_with_hash("msg1", hash1).await);
        
        // Same ID with same hash doesn't process
        assert!(!dedup.should_process_with_hash("msg1", hash1).await);
        
        // Same ID with different hash still doesn't process (but logs warning)
        assert!(!dedup.should_process_with_hash("msg1", hash2).await);
    }

    #[tokio::test]
    async fn test_connection_deduplicator() {
        let conn_dedup = ConnectionDeduplicator::new();
        
        // First connection attempt should proceed
        assert!(conn_dedup.should_attempt_connection("device1", "device2").await);
        
        // Immediate retry should be blocked
        assert!(!conn_dedup.should_attempt_connection("device1", "device2").await);
        
        // Order shouldn't matter
        assert!(!conn_dedup.should_attempt_connection("device2", "device1").await);
        
        // Different pair should proceed
        assert!(conn_dedup.should_attempt_connection("device1", "device3").await);
    }

    #[tokio::test]
    async fn test_session_deduplicator() {
        let session_dedup = SessionDeduplicator::new();
        
        // First proposal processes
        assert!(session_dedup.should_process_proposal("session1", "device1").await);
        
        // Duplicate blocked
        assert!(!session_dedup.should_process_proposal("session1", "device1").await);
        
        // Different session processes
        assert!(session_dedup.should_process_proposal("session2", "device1").await);
        
        // Response deduplication
        assert!(session_dedup.should_process_response("session1", "device2", true).await);
        assert!(!session_dedup.should_process_response("session1", "device2", true).await);
    }

    #[tokio::test]
    async fn test_statistics() {
        let dedup = MessageDeduplicator::new(Duration::from_secs(60), 100);
        
        // Process some messages
        dedup.should_process("msg1").await;
        dedup.should_process("msg1").await; // Duplicate
        dedup.should_process("msg1").await; // Another duplicate
        dedup.should_process("msg2").await;
        dedup.should_process("msg2").await; // Duplicate
        
        let stats = dedup.get_stats().await;
        
        assert_eq!(stats.total_entries, 2); // msg1 and msg2
        assert_eq!(stats.total_duplicates, 3); // 2 for msg1, 1 for msg2
        assert!(stats.cache_utilization > 0.0);
    }
}