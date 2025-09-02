// Message Batching Implementation
// Reduces network overhead by batching multiple messages

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::Mutex;
use tokio::time::{interval, timeout};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchedMessage {
    pub batch_id: String,
    pub target: String,
    pub messages: Vec<serde_json::Value>,
    pub timestamp: SystemTime,
    pub compression: Option<CompressionType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionType {
    None,
    Gzip,
    Lz4,
}

pub struct MessageBatcher {
    pending: Arc<Mutex<HashMap<String, Vec<PendingMessage>>>>,
    batch_size: usize,
    flush_interval: Duration,
    max_batch_bytes: usize,
    sender: Arc<dyn BatchSender>,
}

#[derive(Clone)]
struct PendingMessage {
    content: serde_json::Value,
    timestamp: SystemTime,
    size_bytes: usize,
}

#[async_trait::async_trait]
pub trait BatchSender: Send + Sync {
    async fn send_batch(&self, batch: BatchedMessage) -> Result<(), String>;
}

impl MessageBatcher {
    pub fn new(
        batch_size: usize,
        flush_interval: Duration,
    ) -> Self {
        Self::new_with_sender(
            batch_size,
            flush_interval,
            Arc::new(DefaultBatchSender),
        )
    }

    pub fn new_with_sender(
        batch_size: usize,
        flush_interval: Duration,
        sender: Arc<dyn BatchSender>,
    ) -> Self {
        let batcher = Self {
            pending: Arc::new(Mutex::new(HashMap::new())),
            batch_size,
            flush_interval,
            max_batch_bytes: 1024 * 1024, // 1MB max batch size
            sender,
        };

        // Start background flush task
        let pending_clone = batcher.pending.clone();
        let sender_clone = batcher.sender.clone();
        let flush_interval = batcher.flush_interval;
        
        tokio::spawn(async move {
            Self::flush_task(pending_clone, sender_clone, flush_interval).await;
        });

        batcher
    }

    /// Add a message to the batch
    pub async fn send(&self, target: String, message: serde_json::Value) {
        let size_bytes = message.to_string().len();
        let pending_msg = PendingMessage {
            content: message,
            timestamp: SystemTime::now(),
            size_bytes,
        };

        let mut pending = self.pending.lock().await;
        let batch = pending.entry(target.clone()).or_insert_with(Vec::new);
        
        // Check if adding this message would exceed size limit
        let current_size: usize = batch.iter().map(|m| m.size_bytes).sum();
        if current_size + size_bytes > self.max_batch_bytes && !batch.is_empty() {
            // Flush current batch before adding new message
            let messages_to_send = std::mem::take(batch);
            drop(pending); // Release lock before sending
            
            self.flush_messages(target.clone(), messages_to_send).await;
            
            // Re-acquire lock and add new message
            let mut pending = self.pending.lock().await;
            let batch = pending.entry(target.clone()).or_insert_with(Vec::new);
            batch.push(pending_msg);
        } else {
            batch.push(pending_msg);
            
            // Check if we've reached batch size limit
            if batch.len() >= self.batch_size {
                let messages_to_send = std::mem::take(batch);
                drop(pending); // Release lock before sending
                
                self.flush_messages(target, messages_to_send).await;
            }
        }
    }

    /// Force flush all pending messages
    pub async fn flush_all(&self) {
        let mut pending = self.pending.lock().await;
        let all_batches: Vec<(String, Vec<PendingMessage>)> = 
            pending.drain().collect();
        drop(pending);

        for (target, messages) in all_batches {
            if !messages.is_empty() {
                self.flush_messages(target, messages).await;
            }
        }
    }

    /// Flush a specific target's messages
    async fn flush_messages(&self, target: String, messages: Vec<PendingMessage>) {
        let batch = BatchedMessage {
            batch_id: Uuid::new_v4().to_string(),
            target,
            messages: messages.into_iter().map(|m| m.content).collect(),
            timestamp: SystemTime::now(),
            compression: None, // Can be enhanced to compress large batches
        };

        if let Err(_e) = self.sender.send_batch(batch).await {
            log::error!("Failed to send batch: {}", _e);
        }
    }

    /// Background task to periodically flush batches
    async fn flush_task(
        pending: Arc<Mutex<HashMap<String, Vec<PendingMessage>>>>,
        sender: Arc<dyn BatchSender>,
        flush_interval: Duration,
    ) {
        let mut interval = interval(flush_interval);
        
        loop {
            interval.tick().await;
            
            // Get all pending messages older than flush interval
            let now = SystemTime::now();
            let mut to_flush = Vec::new();
            
            {
                let mut pending_guard = pending.lock().await;
                let mut empty_targets = Vec::new();
                
                for (target, messages) in pending_guard.iter_mut() {
                    let mut to_send = Vec::new();
                    let mut remaining = Vec::new();
                    
                    for msg in messages.drain(..) {
                        if let Ok(age) = now.duration_since(msg.timestamp) {
                            if age >= flush_interval {
                                to_send.push(msg);
                            } else {
                                remaining.push(msg);
                            }
                        } else {
                            remaining.push(msg);
                        }
                    }
                    
                    *messages = remaining;
                    
                    if !to_send.is_empty() {
                        to_flush.push((target.clone(), to_send));
                    }
                    
                    if messages.is_empty() {
                        empty_targets.push(target.clone());
                    }
                }
                
                // Remove empty entries
                for target in empty_targets {
                    pending_guard.remove(&target);
                }
            }
            
            // Send all batches
            for (target, messages) in to_flush {
                let batch = BatchedMessage {
                    batch_id: Uuid::new_v4().to_string(),
                    target,
                    messages: messages.into_iter().map(|m| m.content).collect(),
                    timestamp: SystemTime::now(),
                    compression: None,
                };
                
                if let Err(_e) = sender.send_batch(batch).await {
                    log::error!("Failed to send batch in flush task: {}", _e);
                }
            }
        }
    }

    /// Get current batch statistics
    pub async fn get_stats(&self) -> BatcherStats {
        let pending = self.pending.lock().await;
        let mut stats = BatcherStats::default();
        
        for (target, messages) in pending.iter() {
            stats.pending_targets += 1;
            stats.pending_messages += messages.len();
            stats.pending_bytes += messages.iter().map(|m| m.size_bytes).sum::<usize>();
            
            if messages.len() > stats.largest_batch {
                stats.largest_batch = messages.len();
                stats.largest_batch_target = Some(target.clone());
            }
        }
        
        stats
    }
}

#[derive(Default, Debug)]
pub struct BatcherStats {
    pub pending_targets: usize,
    pub pending_messages: usize,
    pub pending_bytes: usize,
    pub largest_batch: usize,
    pub largest_batch_target: Option<String>,
}

/// Default batch sender implementation
struct DefaultBatchSender;

#[async_trait::async_trait]
impl BatchSender for DefaultBatchSender {
    async fn send_batch(&self, batch: BatchedMessage) -> Result<(), String> {
        log::debug!(
            "Sending batch {} to {} with {} messages",
            batch.batch_id,
            batch.target,
            batch.messages.len()
        );
        // In real implementation, this would send via WebSocket or other transport
        Ok(())
    }
}

/// Optimized batch sender with compression and retries
pub struct OptimizedBatchSender {
    transport: Arc<dyn Transport>,
    enable_compression: bool,
    compression_threshold: usize,
}

#[async_trait::async_trait]
pub trait Transport: Send + Sync {
    async fn send(&self, target: String, data: Vec<u8>) -> Result<(), String>;
}

impl OptimizedBatchSender {
    pub fn new(transport: Arc<dyn Transport>) -> Self {
        Self {
            transport,
            enable_compression: true,
            compression_threshold: 1024, // Compress batches > 1KB
        }
    }

    async fn compress_if_needed(&self, mut batch: BatchedMessage) -> Result<Vec<u8>, String> {
        let serialized = serde_json::to_vec(&batch)
            .map_err(|e| format!("Serialization error: {}", _e))?;
        
        if self.enable_compression && serialized.len() > self.compression_threshold {
            // Use flate2 for gzip compression
            use flate2::write::GzEncoder;
            use flate2::Compression;
            use std::io::Write;
            
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(&serialized)
                .map_err(|e| format!("Compression error: {}", _e))?;
            
            let compressed = encoder.finish()
                .map_err(|e| format!("Compression finish error: {}", _e))?;
            
            // Only use compression if it actually reduces size
            if compressed.len() < serialized.len() {
                batch.compression = Some(CompressionType::Gzip);
                let compressed_batch = serde_json::to_vec(&batch)
                    .map_err(|e| format!("Serialization error: {}", _e))?;
                Ok(compressed_batch)
            } else {
                Ok(serialized)
            }
        } else {
            Ok(serialized)
        }
    }
}

#[async_trait::async_trait]
impl BatchSender for OptimizedBatchSender {
    async fn send_batch(&self, batch: BatchedMessage) -> Result<(), String> {
        let target = batch.target.clone();
        let data = self.compress_if_needed(batch).await?;
        
        // Retry logic with exponential backoff
        let mut retry_count = 0;
        let max_retries = 3;
        
        loop {
            match timeout(Duration::from_secs(5), self.transport.send(target.clone(), data.clone())).await {
                Ok(Ok(())) => return Ok(()),
                Ok(Err(_e)) if retry_count < max_retries => {
                    retry_count += 1;
                    let backoff = Duration::from_millis(100 * (1 << retry_count));
                    log::warn!("Batch send failed, retrying in {:?}: {}", backoff, e);
                    tokio::time::sleep(backoff).await;
                }
                Ok(Err(_e)) => return Err(format!("Failed after {} retries: {}", max_retries, e)),
                Err(_) => {
                    if retry_count < max_retries {
                        retry_count += 1;
                        log::warn!("Batch send timed out, retry {}/{}", retry_count, max_retries);
                    } else {
                        return Err("Batch send timed out".to_string());
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct TestBatchSender {
        sent_count: Arc<AtomicUsize>,
        sent_batches: Arc<Mutex<Vec<BatchedMessage>>>,
    }

    #[async_trait::async_trait]
    impl BatchSender for TestBatchSender {
        async fn send_batch(&self, batch: BatchedMessage) -> Result<(), String> {
            self.sent_count.fetch_add(1, Ordering::SeqCst);
            self.sent_batches.lock().await.push(batch);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_message_batching() {
        let sent_count = Arc::new(AtomicUsize::new(0));
        let sent_batches = Arc::new(Mutex::new(Vec::new()));
        
        let sender = Arc::new(TestBatchSender {
            sent_count: sent_count.clone(),
            sent_batches: sent_batches.clone(),
        });
        
        let batcher = MessageBatcher::new_with_sender(
            3, // batch size
            Duration::from_millis(100),
            sender,
        );
        
        // Send 5 messages - should trigger one batch immediately
        for i in 0..5 {
            batcher.send(
                "target1".to_string(),
                serde_json::json!({ "msg": i }),
            ).await;
        }
        
        // Wait a bit for batch to be sent
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        // Should have sent 1 batch (first 3 messages)
        assert_eq!(sent_count.load(Ordering::SeqCst), 1);
        
        // Wait for flush interval
        tokio::time::sleep(Duration::from_millis(150)).await;
        
        // Should have sent second batch (remaining 2 messages)
        assert_eq!(sent_count.load(Ordering::SeqCst), 2);
        
        let batches = sent_batches.lock().await;
        assert_eq!(batches[0].messages.len(), 3);
        assert_eq!(batches[1].messages.len(), 2);
    }

    #[tokio::test]
    async fn test_flush_all() {
        let sent_count = Arc::new(AtomicUsize::new(0));
        let sent_batches = Arc::new(Mutex::new(Vec::new()));
        
        let sender = Arc::new(TestBatchSender {
            sent_count: sent_count.clone(),
            sent_batches: sent_batches.clone(),
        });
        
        let batcher = MessageBatcher::new_with_sender(
            10, // high batch size
            Duration::from_secs(60), // long flush interval
            sender,
        );
        
        // Send messages to different targets
        for i in 0..3 {
            batcher.send(
                "target1".to_string(),
                serde_json::json!({ "msg": i }),
            ).await;
            
            batcher.send(
                "target2".to_string(),
                serde_json::json!({ "msg": i }),
            ).await;
        }
        
        // No batches sent yet
        assert_eq!(sent_count.load(Ordering::SeqCst), 0);
        
        // Force flush
        batcher.flush_all().await;
        
        // Should have sent 2 batches (one per target)
        assert_eq!(sent_count.load(Ordering::SeqCst), 2);
    }
}