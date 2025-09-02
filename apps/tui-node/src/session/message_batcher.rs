use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use serde_json::Value;
use anyhow::Result;

/// Message to be batched
#[derive(Debug, Clone)]
pub struct Message {
    pub id: String,
    pub to: String,
    pub data: Value,
    pub queued_at: Instant,
}

impl Message {
    pub fn new(to: String, data: Value) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            to,
            data,
            queued_at: Instant::now(),
        }
    }
}

/// Batched messages ready to send
#[derive(Debug, Clone)]
pub struct MessageBatch {
    pub messages: Vec<Message>,
    pub created_at: Instant,
}

/// Message batcher for efficient network utilization
pub struct MessageBatcher {
    /// Messages waiting to be sent
    batch: Vec<Message>,
    /// Maximum batch size
    max_size: usize,
    /// Flush interval
    flush_interval: Duration,
    /// Last flush time
    last_flush: Instant,
    /// Channel to send batched messages
    batch_tx: Option<mpsc::UnboundedSender<MessageBatch>>,
    /// Flush timer handle
    flush_handle: Option<tokio::task::JoinHandle<()>>,
}

impl MessageBatcher {
    pub fn new(max_size: usize, flush_interval: Duration) -> Self {
        Self {
            batch: Vec::with_capacity(max_size),
            max_size,
            flush_interval,
            last_flush: Instant::now(),
            batch_tx: None,
            flush_handle: None,
        }
    }
    
    /// Set the channel for sending batched messages
    pub fn set_batch_channel(&mut self, tx: mpsc::UnboundedSender<MessageBatch>) {
        self.batch_tx = Some(tx);
    }
    
    /// Add a message to the batch
    pub async fn add(&mut self, message: Message) -> Result<()> {
        tracing::trace!("Adding message to batch: {}", message.id);
        
        self.batch.push(message);
        
        // Check if we should flush
        if self.batch.len() >= self.max_size {
            tracing::debug!("Batch size {} reached max {}, flushing", self.batch.len(), self.max_size);
            self.flush().await?;
        } else {
            // Schedule flush if not already scheduled
            self.schedule_flush();
        }
        
        Ok(())
    }
    
    /// Add multiple messages at once
    pub async fn add_batch(&mut self, messages: Vec<Message>) -> Result<()> {
        for message in messages {
            self.batch.push(message);
            
            if self.batch.len() >= self.max_size {
                self.flush().await?;
            }
        }
        
        if !self.batch.is_empty() {
            self.schedule_flush();
        }
        
        Ok(())
    }
    
    /// Flush the current batch
    pub async fn flush(&mut self) -> Result<()> {
        if self.batch.is_empty() {
            return Ok(());
        }
        
        tracing::debug!("Flushing batch of {} messages", self.batch.len());
        
        let batch = std::mem::take(&mut self.batch);
        self.last_flush = Instant::now();
        
        // Cancel any pending flush timer
        if let Some(handle) = self.flush_handle.take() {
            handle.abort();
        }
        
        // Send the batch
        if let Some(tx) = &self.batch_tx {
            let message_batch = MessageBatch {
                messages: batch,
                created_at: Instant::now(),
            };
            
            tx.send(message_batch)
                .map_err(|e| anyhow::anyhow!("Failed to send batch: {}", e))?;
        } else {
            // If no channel set, process messages individually
            for msg in batch {
                self.process_single_message(msg).await?;
            }
        }
        
        Ok(())
    }
    
    /// Schedule a flush after the flush interval
    fn schedule_flush(&mut self) {
        // Don't schedule if we already have one pending
        if self.flush_handle.is_some() {
            return;
        }
        
        let interval = self.flush_interval;
        let batch_tx = self.batch_tx.clone();
        
        self.flush_handle = Some(tokio::spawn(async move {
            tokio::time::sleep(interval).await;
            
            // Send flush signal
            if let Some(tx) = batch_tx {
                // Send empty batch as flush signal
                let _ = tx.send(MessageBatch {
                    messages: vec![],
                    created_at: Instant::now(),
                });
            }
        }));
    }
    
    /// Process a single message (fallback when no batch channel)
    async fn process_single_message(&self, message: Message) -> Result<()> {
        tracing::debug!("Processing single message: {} -> {}", message.id, message.to);
        // This would be implemented based on your actual message sending logic
        Ok(())
    }
    
    /// Get current batch size
    pub fn batch_size(&self) -> usize {
        self.batch.len()
    }
    
    /// Check if batch is empty
    pub fn is_empty(&self) -> bool {
        self.batch.is_empty()
    }
    
    /// Get time since last flush
    pub fn time_since_flush(&self) -> Duration {
        self.last_flush.elapsed()
    }
    
    /// Force flush if interval has passed
    pub async fn check_and_flush(&mut self) -> Result<()> {
        if !self.batch.is_empty() && self.time_since_flush() >= self.flush_interval {
            self.flush().await?;
        }
        Ok(())
    }
    
    /// Get statistics about the batcher
    pub fn get_stats(&self) -> BatcherStats {
        BatcherStats {
            current_batch_size: self.batch.len(),
            max_batch_size: self.max_size,
            time_since_flush: self.last_flush.elapsed(),
            flush_interval: self.flush_interval,
        }
    }
}

/// Statistics about the message batcher
#[derive(Debug, Clone)]
pub struct BatcherStats {
    pub current_batch_size: usize,
    pub max_batch_size: usize,
    pub time_since_flush: Duration,
    pub flush_interval: Duration,
}

/// Optimized message combiner for reducing redundant messages
pub struct MessageCombiner {
    /// Messages grouped by destination
    pending: std::collections::HashMap<String, Vec<Message>>,
}

impl MessageCombiner {
    pub fn new() -> Self {
        Self {
            pending: std::collections::HashMap::new(),
        }
    }
    
    /// Add a message and combine if possible
    pub fn add(&mut self, message: Message) {
        self.pending
            .entry(message.to.clone())
            .or_insert_with(Vec::new)
            .push(message);
    }
    
    /// Combine messages to the same destination
    pub fn combine(&mut self) -> Vec<Message> {
        let mut combined = Vec::new();
        
        for (to, messages) in self.pending.drain() {
            if messages.len() == 1 {
                // Single message, send as-is
                combined.extend(messages);
            } else {
                // Multiple messages to same destination, combine
                let data = Value::Array(
                    messages.into_iter()
                        .map(|m| m.data)
                        .collect()
                );
                
                combined.push(Message::new(to, data));
            }
        }
        
        combined
    }
    
    /// Get the number of pending destinations
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[tokio::test]
    async fn test_batch_size_trigger() {
        let mut batcher = MessageBatcher::new(3, Duration::from_secs(1));
        
        // Add messages up to max size
        for i in 0..3 {
            let msg = Message::new(
                format!("peer{}", i),
                json!({"data": i}),
            );
            batcher.add(msg).await.unwrap();
        }
        
        // Batch should be flushed
        assert_eq!(batcher.batch_size(), 0);
    }
    
    #[tokio::test]
    async fn test_manual_flush() {
        let mut batcher = MessageBatcher::new(10, Duration::from_secs(1));
        
        // Add some messages
        for i in 0..5 {
            let msg = Message::new(
                format!("peer{}", i),
                json!({"data": i}),
            );
            batcher.add(msg).await.unwrap();
        }
        
        assert_eq!(batcher.batch_size(), 5);
        
        // Manual flush
        batcher.flush().await.unwrap();
        assert_eq!(batcher.batch_size(), 0);
    }
    
    #[tokio::test]
    async fn test_message_combiner() {
        let mut combiner = MessageCombiner::new();
        
        // Add multiple messages to same destination
        combiner.add(Message::new("peer1".to_string(), json!({"a": 1})));
        combiner.add(Message::new("peer1".to_string(), json!({"b": 2})));
        combiner.add(Message::new("peer2".to_string(), json!({"c": 3})));
        
        let combined = combiner.combine();
        
        // Should have 2 messages (peer1 combined, peer2 single)
        assert_eq!(combined.len(), 2);
        
        // Check that peer1 messages were combined into array
        let peer1_msg = combined.iter().find(|m| m.to == "peer1").unwrap();
        assert!(peer1_msg.data.is_array());
    }
    
    #[tokio::test]
    async fn test_time_based_flush() {
        let mut batcher = MessageBatcher::new(10, Duration::from_millis(100));
        
        // Add a message
        let msg = Message::new("peer1".to_string(), json!({"data": 1}));
        batcher.add(msg).await.unwrap();
        
        assert_eq!(batcher.batch_size(), 1);
        
        // Wait for flush interval
        tokio::time::sleep(Duration::from_millis(150)).await;
        
        // Check and flush based on time
        batcher.check_and_flush().await.unwrap();
        assert_eq!(batcher.batch_size(), 0);
    }
}