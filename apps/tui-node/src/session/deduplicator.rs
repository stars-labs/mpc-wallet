use std::time::{Duration, Instant};
use lru::LruCache;
use std::num::NonZeroUsize;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Message identifier for deduplication
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct MessageId {
    /// Source of the message
    pub from: String,
    /// Type of message
    pub msg_type: String,
    /// Hash of message content
    pub content_hash: u64,
    /// Optional sequence number
    pub sequence: Option<u64>,
}

impl MessageId {
    /// Create a new message ID from components
    pub fn new(from: String, msg_type: String, content: &[u8]) -> Self {
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        let content_hash = hasher.finish();
        
        Self {
            from,
            msg_type,
            content_hash,
            sequence: None,
        }
    }
    
    /// Create a message ID with sequence number
    pub fn with_sequence(mut self, seq: u64) -> Self {
        self.sequence = Some(seq);
        self
    }
}

/// Entry in the deduplication cache
#[derive(Debug, Clone)]
struct DeduplicationEntry {
    /// When the message was first seen
    seen_at: Instant,
    /// Number of times we've seen this message
    count: u32,
}

/// Message deduplicator to prevent processing duplicate messages
pub struct MessageDeduplicator {
    /// Cache of recently seen messages
    seen: LruCache<MessageId, DeduplicationEntry>,
    /// Time-to-live for entries
    ttl: Duration,
    /// Statistics
    stats: DeduplicationStats,
}

impl MessageDeduplicator {
    pub fn new(ttl: Duration) -> Self {
        // Default cache size of 10,000 messages
        Self::with_capacity(10_000, ttl)
    }
    
    pub fn with_capacity(capacity: usize, ttl: Duration) -> Self {
        let cache_size = NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::new(1).unwrap());
        
        Self {
            seen: LruCache::new(cache_size),
            ttl,
            stats: DeduplicationStats::default(),
        }
    }
    
    /// Check if we should process a message (returns false for duplicates)
    pub fn should_process(&mut self, msg_id: &MessageId) -> bool {
        self.stats.total_checked += 1;
        
        // Check if we've seen this message
        if let Some(entry) = self.seen.get_mut(msg_id) {
            // Check if entry is still valid
            if entry.seen_at.elapsed() < self.ttl {
                entry.count += 1;
                self.stats.duplicates_blocked += 1;
                tracing::debug!(
                    "Blocking duplicate message: {:?} (seen {} times)",
                    msg_id, entry.count
                );
                return false; // Duplicate
            } else {
                // Entry expired, update it
                entry.seen_at = Instant::now();
                entry.count = 1;
                self.stats.unique_processed += 1;
                return true;
            }
        }
        
        // New message, add to cache
        self.seen.put(
            msg_id.clone(),
            DeduplicationEntry {
                seen_at: Instant::now(),
                count: 1,
            },
        );
        
        self.stats.unique_processed += 1;
        true
    }
    
    /// Check if we should process a proposal (special handling)
    pub fn should_process_proposal(&mut self, session_id: &str, proposer: &str) -> bool {
        let msg_id = MessageId {
            from: proposer.to_string(),
            msg_type: "SessionProposal".to_string(),
            content_hash: {
                let mut hasher = DefaultHasher::new();
                session_id.hash(&mut hasher);
                hasher.finish()
            },
            sequence: None,
        };
        
        self.should_process(&msg_id)
    }
    
    /// Check if we should process a response
    pub fn should_process_response(&mut self, session_id: &str, device_id: &str) -> bool {
        let msg_id = MessageId {
            from: device_id.to_string(),
            msg_type: "SessionResponse".to_string(),
            content_hash: {
                let mut hasher = DefaultHasher::new();
                session_id.hash(&mut hasher);
                hasher.finish()
            },
            sequence: None,
        };
        
        self.should_process(&msg_id)
    }
    
    /// Check if we should process a WebRTC signal
    pub fn should_process_signal(&mut self, from: &str, to: &str, signal_type: &str) -> bool {
        let msg_id = MessageId {
            from: from.to_string(),
            msg_type: format!("WebRTC-{}", signal_type),
            content_hash: {
                let mut hasher = DefaultHasher::new();
                to.hash(&mut hasher);
                hasher.finish()
            },
            sequence: None,
        };
        
        self.should_process(&msg_id)
    }
    
    /// Clean up expired entries
    pub fn cleanup(&mut self) {
        let now = Instant::now();
        let mut expired = Vec::new();
        
        // Find expired entries
        for (key, entry) in self.seen.iter() {
            if now.duration_since(entry.seen_at) > self.ttl {
                expired.push(key.clone());
            }
        }
        
        // Remove expired entries
        for key in expired {
            self.seen.pop(&key);
            self.stats.entries_expired += 1;
        }
    }
    
    /// Clear all entries
    pub fn clear(&mut self) {
        self.seen.clear();
        self.stats.entries_cleared += self.seen.len();
    }
    
    /// Get deduplication statistics
    pub fn get_stats(&self) -> DeduplicationStats {
        let mut stats = self.stats.clone();
        stats.cache_size = self.seen.len();
        stats
    }
    
    /// Get cache utilization percentage
    pub fn utilization(&self) -> f64 {
        (self.seen.len() as f64 / self.seen.cap().get() as f64) * 100.0
    }
}

/// Statistics about deduplication
#[derive(Debug, Clone, Default)]
pub struct DeduplicationStats {
    /// Total messages checked
    pub total_checked: u64,
    /// Unique messages processed
    pub unique_processed: u64,
    /// Duplicate messages blocked
    pub duplicates_blocked: u64,
    /// Entries that expired
    pub entries_expired: usize,
    /// Entries explicitly cleared
    pub entries_cleared: usize,
    /// Current cache size
    pub cache_size: usize,
}

impl DeduplicationStats {
    /// Get the duplicate rate as a percentage
    pub fn duplicate_rate(&self) -> f64 {
        if self.total_checked == 0 {
            return 0.0;
        }
        (self.duplicates_blocked as f64 / self.total_checked as f64) * 100.0
    }
    
    /// Get the cache hit rate
    pub fn hit_rate(&self) -> f64 {
        if self.total_checked == 0 {
            return 0.0;
        }
        let hits = self.duplicates_blocked + self.unique_processed;
        (hits as f64 / self.total_checked as f64) * 100.0
    }
}

/// Sequence tracker for ordered message processing
pub struct SequenceTracker {
    /// Expected sequence numbers per source
    sequences: std::collections::HashMap<String, u64>,
    /// Out-of-order messages buffer
    buffer: std::collections::HashMap<String, Vec<(u64, serde_json::Value)>>,
    /// Maximum buffer size per source
    max_buffer_size: usize,
}

impl SequenceTracker {
    pub fn new() -> Self {
        Self::with_buffer_size(100)
    }
    
    pub fn with_buffer_size(max_buffer_size: usize) -> Self {
        Self {
            sequences: std::collections::HashMap::new(),
            buffer: std::collections::HashMap::new(),
            max_buffer_size,
        }
    }
    
    /// Check if a message with given sequence should be processed
    pub fn should_process_sequence(&mut self, source: &str, sequence: u64) -> bool {
        let expected = self.sequences.get(source).copied().unwrap_or(0);
        
        if sequence == expected {
            // This is the expected sequence
            self.sequences.insert(source.to_string(), sequence + 1);
            true
        } else if sequence < expected {
            // Old message, already processed
            tracing::debug!("Dropping old message from {} (seq {} < expected {})", 
                       source, sequence, expected);
            false
        } else {
            // Future message, buffer it
            tracing::debug!("Buffering future message from {} (seq {} > expected {})", 
                       source, sequence, expected);
            false
        }
    }
    
    /// Buffer an out-of-order message
    pub fn buffer_message(&mut self, source: String, sequence: u64, message: serde_json::Value) {
        let buffer = self.buffer.entry(source.clone()).or_insert_with(Vec::new);
        
        // Add to buffer if not full
        if buffer.len() < self.max_buffer_size {
            buffer.push((sequence, message));
            buffer.sort_by_key(|&(seq, _)| seq);
        } else {
            tracing::warn!("Buffer full for source {}, dropping message", source);
        }
    }
    
    /// Get any buffered messages that can now be processed
    pub fn get_ready_messages(&mut self, source: &str) -> Vec<serde_json::Value> {
        let expected = self.sequences.get(source).copied().unwrap_or(0);
        let mut ready = Vec::new();
        
        if let Some(buffer) = self.buffer.get_mut(source) {
            while !buffer.is_empty() && buffer[0].0 == expected + ready.len() as u64 {
                let (seq, msg) = buffer.remove(0);
                ready.push(msg);
                self.sequences.insert(source.to_string(), seq + 1);
            }
        }
        
        ready
    }
    
    /// Reset sequence tracking for a source
    pub fn reset_source(&mut self, source: &str) {
        self.sequences.remove(source);
        self.buffer.remove(source);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_deduplication() {
        let mut dedup = MessageDeduplicator::new(Duration::from_secs(60));
        
        let msg_id = MessageId::new(
            "peer1".to_string(),
            "test".to_string(),
            b"content",
        );
        
        // First time should process
        assert!(dedup.should_process(&msg_id));
        
        // Second time should not process (duplicate)
        assert!(!dedup.should_process(&msg_id));
        
        // Check stats
        let stats = dedup.get_stats();
        assert_eq!(stats.total_checked, 2);
        assert_eq!(stats.unique_processed, 1);
        assert_eq!(stats.duplicates_blocked, 1);
    }
    
    #[test]
    fn test_ttl_expiration() {
        let mut dedup = MessageDeduplicator::new(Duration::from_millis(100));
        
        let msg_id = MessageId::new(
            "peer1".to_string(),
            "test".to_string(),
            b"content",
        );
        
        // First time should process
        assert!(dedup.should_process(&msg_id));
        
        // Wait for TTL to expire
        std::thread::sleep(Duration::from_millis(150));
        
        // Should process again after TTL
        assert!(dedup.should_process(&msg_id));
    }
    
    #[test]
    fn test_proposal_deduplication() {
        let mut dedup = MessageDeduplicator::new(Duration::from_secs(60));
        
        // First proposal should process
        assert!(dedup.should_process_proposal("session1", "proposer1"));
        
        // Duplicate proposal should not process
        assert!(!dedup.should_process_proposal("session1", "proposer1"));
        
        // Different session should process
        assert!(dedup.should_process_proposal("session2", "proposer1"));
        
        // Different proposer, same session should process
        assert!(dedup.should_process_proposal("session1", "proposer2"));
    }
    
    #[test]
    fn test_sequence_tracker() {
        let mut tracker = SequenceTracker::new();
        
        // Messages in order
        assert!(tracker.should_process_sequence("peer1", 0));
        assert!(tracker.should_process_sequence("peer1", 1));
        assert!(tracker.should_process_sequence("peer1", 2));
        
        // Old message (already processed)
        assert!(!tracker.should_process_sequence("peer1", 1));
        
        // Future message (should buffer)
        assert!(!tracker.should_process_sequence("peer1", 5));
        
        // Different peer starts at 0
        assert!(tracker.should_process_sequence("peer2", 0));
    }
    
    #[test]
    fn test_buffered_messages() {
        let mut tracker = SequenceTracker::new();
        
        // Process sequence 0
        assert!(tracker.should_process_sequence("peer1", 0));
        
        // Buffer future messages (out of order) - leave a gap at 5
        tracker.buffer_message("peer1".to_string(), 3, serde_json::json!({"msg": 3}));
        tracker.buffer_message("peer1".to_string(), 2, serde_json::json!({"msg": 2}));
        tracker.buffer_message("peer1".to_string(), 5, serde_json::json!({"msg": 5})); // Gap at 4
        
        // Check buffer contents
        let buffer = tracker.buffer.get("peer1").unwrap();
        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer[0].0, 2); // Should be sorted
        assert_eq!(buffer[1].0, 3);
        assert_eq!(buffer[2].0, 5); // Gap at 4
        
        // Process sequence 1, which updates expected to 2
        assert!(tracker.should_process_sequence("peer1", 1));
        
        // Now check if any buffered messages are ready
        // 2 and 3 are consecutive and should be ready
        // 5 should stay buffered because there's a gap at 4
        let ready = tracker.get_ready_messages("peer1");
        assert_eq!(ready.len(), 2); // Messages 2 and 3 should be ready
        
        // Verify 5 is still buffered
        let buffer = tracker.buffer.get("peer1").unwrap();
        assert_eq!(buffer.len(), 1);
        assert_eq!(buffer[0].0, 5);
    }
    
    #[test]
    fn test_cache_utilization() {
        let mut dedup = MessageDeduplicator::with_capacity(10, Duration::from_secs(60));
        
        // Add messages up to capacity
        for i in 0..5 {
            let msg_id = MessageId::new(
                format!("peer{}", i),
                "test".to_string(),
                format!("content{}", i).as_bytes(),
            );
            dedup.should_process(&msg_id);
        }
        
        assert_eq!(dedup.utilization(), 50.0);
    }
}