// Enhanced message batching with priority queue and deduplication
use tokio::sync::mpsc;
use tokio::time::{Duration, interval};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tracing::{debug, warn};
use serde::{Serialize, Deserialize};

/// Message priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Critical = 0,  // DKG messages, connection setup
    High = 1,      // Signing requests
    Normal = 2,    // Status updates
    Low = 3,       // Logging, metrics
}

/// Wrapper for messages with priority and metadata
#[derive(Debug, Clone)]
pub struct PrioritizedMessage<T> {
    pub priority: Priority,
    pub message: T,
    pub timestamp: std::time::Instant,
    pub retry_count: u32,
}

/// Advanced message batcher with priority queue and deduplication
pub struct MessageBatcherV2<T: Clone + Send + 'static> {
    batch_queues: HashMap<Priority, VecDeque<PrioritizedMessage<T>>>,
    max_batch_size: usize,
    flush_interval: Duration,
    dedup_window: Duration,
    recent_hashes: HashMap<u64, std::time::Instant>,
    metrics: BatcherMetrics,
}

#[derive(Default, Debug)]
pub struct BatcherMetrics {
    pub messages_processed: u64,
    pub batches_sent: u64,
    pub messages_deduplicated: u64,
    pub messages_dropped: u64,
    pub average_batch_size: f64,
}

impl<T> MessageBatcherV2<T> 
where 
    T: Clone + Send + Sync + 'static + std::hash::Hash,
{
    pub fn new(max_batch_size: usize, flush_interval: Duration) -> Self {
        let mut batch_queues = HashMap::new();
        batch_queues.insert(Priority::Critical, VecDeque::new());
        batch_queues.insert(Priority::High, VecDeque::new());
        batch_queues.insert(Priority::Normal, VecDeque::new());
        batch_queues.insert(Priority::Low, VecDeque::new());
        
        Self {
            batch_queues,
            max_batch_size,
            flush_interval,
            dedup_window: Duration::from_secs(5),
            recent_hashes: HashMap::new(),
            metrics: BatcherMetrics::default(),
        }
    }
    
    /// Add a message to the batcher
    pub fn add_message(&mut self, message: T, priority: Priority) -> bool {
        let hash = self.calculate_hash(&message);
        
        // Check for duplicates
        if let Some(last_seen) = self.recent_hashes.get(&hash) {
            if last_seen.elapsed() < self.dedup_window {
                self.metrics.messages_deduplicated += 1;
                debug!("Deduplicated message with hash: {}", hash);
                return false;
            }
        }
        
        // Add to recent hashes for deduplication
        self.recent_hashes.insert(hash, std::time::Instant::now());
        
        // Add to appropriate priority queue
        let prioritized = PrioritizedMessage {
            priority,
            message,
            timestamp: std::time::Instant::now(),
            retry_count: 0,
        };
        
        if let Some(queue) = self.batch_queues.get_mut(&priority) {
            queue.push_back(prioritized);
            self.metrics.messages_processed += 1;
            true
        } else {
            false
        }
    }
    
    /// Get next batch to process (respecting priorities)
    pub fn get_batch(&mut self) -> Vec<T> {
        let mut batch = Vec::with_capacity(self.max_batch_size);
        let mut remaining = self.max_batch_size;
        
        // Process in priority order
        for priority in [Priority::Critical, Priority::High, Priority::Normal, Priority::Low] {
            if remaining == 0 { break; }
            
            if let Some(queue) = self.batch_queues.get_mut(&priority) {
                let take_count = remaining.min(queue.len());
                for _ in 0..take_count {
                    if let Some(msg) = queue.pop_front() {
                        batch.push(msg.message);
                        remaining -= 1;
                    }
                }
            }
        }
        
        // Update metrics
        if !batch.is_empty() {
            self.metrics.batches_sent += 1;
            let batch_size = batch.len() as f64;
            self.metrics.average_batch_size = 
                (self.metrics.average_batch_size * (self.metrics.batches_sent - 1) as f64 + batch_size) 
                / self.metrics.batches_sent as f64;
        }
        
        // Clean old dedup entries
        self.cleanup_old_hashes();
        
        batch
    }
    
    /// Check if any messages are ready
    pub fn has_messages(&self) -> bool {
        self.batch_queues.values().any(|q| !q.is_empty())
    }
    
    /// Get total pending message count
    pub fn pending_count(&self) -> usize {
        self.batch_queues.values().map(|q| q.len()).sum()
    }
    
    /// Calculate hash for deduplication
    fn calculate_hash(&self, message: &T) -> u64 {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        message.hash(&mut hasher);
        hasher.finish()
    }
    
    /// Clean up old deduplication entries
    fn cleanup_old_hashes(&mut self) {
        let now = std::time::Instant::now();
        self.recent_hashes.retain(|_, timestamp| {
            now.duration_since(*timestamp) < self.dedup_window * 2
        });
    }
    
    /// Get current metrics
    pub fn metrics(&self) -> &BatcherMetrics {
        &self.metrics
    }
}

/// Async batcher that runs in background
pub struct AsyncMessageBatcher<T: Clone + Send + Sync + 'static + std::hash::Hash> {
    batcher: Arc<tokio::sync::Mutex<MessageBatcherV2<T>>>,
    sender: mpsc::Sender<(T, Priority)>,
}

impl<T> AsyncMessageBatcher<T> 
where 
    T: Clone + Send + Sync + 'static + std::hash::Hash,
{
    pub fn new(
        max_batch_size: usize,
        flush_interval: Duration,
        process_fn: impl Fn(Vec<T>) + Send + 'static,
    ) -> Self {
        let (sender, mut receiver) = mpsc::channel::<(T, Priority)>(1000);
        let batcher = Arc::new(tokio::sync::Mutex::new(
            MessageBatcherV2::new(max_batch_size, flush_interval)
        ));
        
        let batcher_clone = batcher.clone();
        tokio::spawn(async move {
            let mut interval = interval(flush_interval);
            
            loop {
                tokio::select! {
                    Some((msg, priority)) = receiver.recv() => {
                        let mut b = batcher_clone.lock().await;
                        b.add_message(msg, priority);
                        
                        // Check if we should flush immediately
                        if b.pending_count() >= max_batch_size {
                            let batch = b.get_batch();
                            if !batch.is_empty() {
                                process_fn(batch);
                            }
                        }
                    }
                    _ = interval.tick() => {
                        let mut b = batcher_clone.lock().await;
                        if b.has_messages() {
                            let batch = b.get_batch();
                            if !batch.is_empty() {
                                process_fn(batch);
                            }
                        }
                    }
                }
            }
        });
        
        Self { batcher, sender }
    }
    
    /// Send a message to be batched
    pub async fn send(&self, message: T, priority: Priority) -> Result<(), mpsc::error::SendError<(T, Priority)>> {
        self.sender.send((message, priority)).await
    }
    
    /// Get current metrics
    pub async fn metrics(&self) -> BatcherMetrics {
        let b = self.batcher.lock().await;
        b.metrics().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[derive(Clone, Hash, Debug, PartialEq)]
    struct TestMessage {
        id: u32,
        content: String,
    }
    
    #[tokio::test]
    async fn test_priority_batching() {
        let mut batcher = MessageBatcherV2::<TestMessage>::new(5, Duration::from_millis(100));
        
        // Add messages with different priorities
        batcher.add_message(TestMessage { id: 1, content: "low".into() }, Priority::Low);
        batcher.add_message(TestMessage { id: 2, content: "critical".into() }, Priority::Critical);
        batcher.add_message(TestMessage { id: 3, content: "normal".into() }, Priority::Normal);
        batcher.add_message(TestMessage { id: 4, content: "high".into() }, Priority::High);
        
        let batch = batcher.get_batch();
        
        // Critical should come first
        assert_eq!(batch[0].content, "critical");
        assert_eq!(batch[1].content, "high");
        assert_eq!(batch[2].content, "normal");
        assert_eq!(batch[3].content, "low");
    }
    
    #[tokio::test]
    async fn test_deduplication() {
        let mut batcher = MessageBatcherV2::<TestMessage>::new(5, Duration::from_millis(100));
        
        let msg = TestMessage { id: 1, content: "test".into() };
        
        // Add same message twice
        assert!(batcher.add_message(msg.clone(), Priority::Normal));
        assert!(!batcher.add_message(msg.clone(), Priority::Normal)); // Should be deduplicated
        
        assert_eq!(batcher.metrics.messages_deduplicated, 1);
    }
}