//! Bounded Channel Configuration
//!
//! This module provides configuration for bounded channels to prevent memory leaks
//! and improve backpressure handling throughout the application.

use tokio::sync::mpsc::{Sender, Receiver, channel};

/// Channel bounds configuration
#[derive(Debug, Clone)]
pub struct ChannelConfig {
    /// Maximum messages in the main message queue
    pub message_queue_size: usize,
    /// Maximum events in the session event queue
    pub session_event_queue_size: usize,
    /// Maximum WebSocket messages in the queue
    pub websocket_queue_size: usize,
    /// Maximum internal commands in the queue
    pub internal_command_queue_size: usize,
    /// Maximum batch messages in the queue
    pub batch_queue_size: usize,
}

impl Default for ChannelConfig {
    fn default() -> Self {
        Self {
            message_queue_size: 1000,           // UI messages
            session_event_queue_size: 500,      // Session events
            websocket_queue_size: 200,          // WebSocket messages
            internal_command_queue_size: 100,   // Internal commands
            batch_queue_size: 50,               // Batched messages
        }
    }
}

impl ChannelConfig {
    /// Create a conservative configuration for low-memory environments
    pub fn conservative() -> Self {
        Self {
            message_queue_size: 100,
            session_event_queue_size: 50,
            websocket_queue_size: 20,
            internal_command_queue_size: 10,
            batch_queue_size: 5,
        }
    }
    
    /// Create a performance configuration for high-throughput environments
    pub fn performance() -> Self {
        Self {
            message_queue_size: 5000,
            session_event_queue_size: 2000,
            websocket_queue_size: 1000,
            internal_command_queue_size: 500,
            batch_queue_size: 200,
        }
    }
}

/// Create a bounded channel for messages with backpressure handling
pub fn create_message_channel<T>(size: usize) -> (Sender<T>, Receiver<T>) {
    channel(size)
}

/// Wrapper for bounded sender with metrics
#[derive(Clone)]
pub struct BoundedSender<T> {
    inner: Sender<T>,
    dropped_messages: std::sync::Arc<std::sync::atomic::AtomicU64>,
    channel_name: String,
}

impl<T> BoundedSender<T> {
    /// Create a new bounded sender with metrics
    pub fn new(sender: Sender<T>, channel_name: String) -> Self {
        Self {
            inner: sender,
            dropped_messages: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)),
            channel_name,
        }
    }
    
    /// Try to send a message, dropping it if the channel is full
    pub async fn try_send_lossy(&self, msg: T) -> Result<(), T> {
        match self.inner.try_send(msg) {
            Ok(()) => Ok(()),
            Err(tokio::sync::mpsc::error::TrySendError::Full(msg)) => {
                self.dropped_messages.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                tracing::warn!(
                    "Channel '{}' is full, dropping message (total dropped: {})",
                    self.channel_name,
                    self.dropped_messages.load(std::sync::atomic::Ordering::Relaxed)
                );
                Err(msg)
            }
            Err(tokio::sync::mpsc::error::TrySendError::Closed(msg)) => {
                tracing::error!("Channel '{}' is closed", self.channel_name);
                Err(msg)
            }
        }
    }
    
    /// Send a message, waiting if the channel is full
    pub async fn send(&self, msg: T) -> Result<(), tokio::sync::mpsc::error::SendError<T>> {
        self.inner.send(msg).await
    }
    
    /// Get the number of dropped messages
    pub fn dropped_count(&self) -> u64 {
        self.dropped_messages.load(std::sync::atomic::Ordering::Relaxed)
    }
}

/// Migration helper to convert UnboundedSender usage to bounded
pub type MigrationSender<T> = Sender<T>;
pub type MigrationReceiver<T> = Receiver<T>;

/// Create bounded channels with proper error handling
pub fn create_bounded_channels<T>(config: &ChannelConfig, channel_type: ChannelType) -> (Sender<T>, Receiver<T>) {
    let size = match channel_type {
        ChannelType::Message => config.message_queue_size,
        ChannelType::SessionEvent => config.session_event_queue_size,
        ChannelType::WebSocket => config.websocket_queue_size,
        ChannelType::InternalCommand => config.internal_command_queue_size,
        ChannelType::Batch => config.batch_queue_size,
    };
    
    channel(size)
}

/// Channel type for configuration
#[derive(Debug, Clone, Copy)]
pub enum ChannelType {
    Message,
    SessionEvent,
    WebSocket,
    InternalCommand,
    Batch,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_bounded_sender_lossy() {
        let (tx, mut rx) = channel(2);
        let sender = BoundedSender::new(tx, "test".to_string());
        
        // Send within capacity
        assert!(sender.try_send_lossy(1).await.is_ok());
        assert!(sender.try_send_lossy(2).await.is_ok());
        
        // Channel is full, should drop message
        assert!(sender.try_send_lossy(3).await.is_err());
        assert_eq!(sender.dropped_count(), 1);
        
        // Receive one to make space
        assert_eq!(rx.recv().await, Some(1));
        
        // Should be able to send again
        assert!(sender.try_send_lossy(4).await.is_ok());
    }
    
    #[tokio::test]
    async fn test_channel_config() {
        let config = ChannelConfig::default();
        assert_eq!(config.message_queue_size, 1000);
        
        let conservative = ChannelConfig::conservative();
        assert_eq!(conservative.message_queue_size, 100);
        
        let performance = ChannelConfig::performance();
        assert_eq!(performance.message_queue_size, 5000);
    }
}