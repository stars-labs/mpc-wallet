// Bounded channel implementation with backpressure
use tokio::sync::mpsc;
use std::sync::Arc;
use tracing::{warn, debug};
use crate::utils::state::InternalCommand;
use frost_core::Ciphersuite;

const DEFAULT_CHANNEL_SIZE: usize = 1000;
const WARNING_THRESHOLD: usize = 800;

/// Bounded command channel with backpressure and monitoring
pub struct BoundedCommandChannel<C: Ciphersuite> {
    sender: mpsc::Sender<InternalCommand<C>>,
    receiver: Option<mpsc::Receiver<InternalCommand<C>>>,
    metrics: Arc<ChannelMetrics>,
}

#[derive(Default)]
pub struct ChannelMetrics {
    messages_sent: std::sync::atomic::AtomicU64,
    messages_dropped: std::sync::atomic::AtomicU64,
    backpressure_events: std::sync::atomic::AtomicU64,
}

impl<C: Ciphersuite> BoundedCommandChannel<C> {
    /// Create a new bounded channel with default size
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CHANNEL_SIZE)
    }
    
    /// Create a new bounded channel with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        let (sender, receiver) = mpsc::channel(capacity);
        Self {
            sender,
            receiver: Some(receiver),
            metrics: Arc::new(ChannelMetrics::default()),
        }
    }
    
    /// Get the sender half of the channel
    pub fn sender(&self) -> BoundedSender<C> {
        BoundedSender {
            inner: self.sender.clone(),
            metrics: self.metrics.clone(),
        }
    }
    
    /// Take the receiver (can only be called once)
    pub fn take_receiver(&mut self) -> Option<mpsc::Receiver<InternalCommand<C>>> {
        self.receiver.take()
    }
    
    /// Get current channel metrics
    pub fn metrics(&self) -> ChannelStats {
        ChannelStats {
            messages_sent: self.metrics.messages_sent.load(std::sync::atomic::Ordering::Relaxed),
            messages_dropped: self.metrics.messages_dropped.load(std::sync::atomic::Ordering::Relaxed),
            backpressure_events: self.metrics.backpressure_events.load(std::sync::atomic::Ordering::Relaxed),
            capacity: self.sender.capacity(),
            available: self.sender.capacity() - self.sender.max_capacity() + self.sender.capacity(),
        }
    }
}

/// Sender with automatic metrics and backpressure handling
pub struct BoundedSender<C: Ciphersuite> {
    inner: mpsc::Sender<InternalCommand<C>>,
    metrics: Arc<ChannelMetrics>,
}

impl<C: Ciphersuite> BoundedSender<C> {
    /// Send a command with backpressure handling
    pub async fn send(&self, cmd: InternalCommand<C>) -> Result<(), SendError> {
        // Check capacity before sending
        let capacity = self.inner.capacity();
        if capacity < (DEFAULT_CHANNEL_SIZE - WARNING_THRESHOLD) {
            warn!("Channel approaching capacity: {} slots remaining", capacity);
            self.metrics.backpressure_events.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        
        match self.inner.send(cmd).await {
            Ok(()) => {
                self.metrics.messages_sent.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                Ok(())
            }
            Err(_) => {
                self.metrics.messages_dropped.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                Err(SendError::ChannelClosed)
            }
        }
    }
    
    /// Try to send without blocking
    pub fn try_send(&self, cmd: InternalCommand<C>) -> Result<(), SendError> {
        match self.inner.try_send(cmd) {
            Ok(()) => {
                self.metrics.messages_sent.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                Ok(())
            }
            Err(mpsc::error::TrySendError::Full(_)) => {
                self.metrics.backpressure_events.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                Err(SendError::ChannelFull)
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                self.metrics.messages_dropped.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                Err(SendError::ChannelClosed)
            }
        }
    }
    
    /// Send with timeout
    pub async fn send_timeout(&self, cmd: InternalCommand<C>, timeout: std::time::Duration) -> Result<(), SendError> {
        match tokio::time::timeout(timeout, self.send(cmd)).await {
            Ok(result) => result,
            Err(_) => {
                self.metrics.backpressure_events.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                Err(SendError::Timeout)
            }
        }
    }
}

impl<C: Ciphersuite> Clone for BoundedSender<C> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            metrics: self.metrics.clone(),
        }
    }
}

#[derive(Debug)]
pub enum SendError {
    ChannelClosed,
    ChannelFull,
    Timeout,
}

#[derive(Debug)]
pub struct ChannelStats {
    pub messages_sent: u64,
    pub messages_dropped: u64,
    pub backpressure_events: u64,
    pub capacity: usize,
    pub available: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_bounded_channel_backpressure() {
        let mut channel = BoundedCommandChannel::<frost_secp256k1::Secp256K1Sha256>::with_capacity(10);
        let sender = channel.sender();
        
        // Fill the channel
        for i in 0..10 {
            let cmd = InternalCommand::ListWallets;
            assert!(sender.try_send(cmd).is_ok());
        }
        
        // Next send should fail with ChannelFull
        let cmd = InternalCommand::ListWallets;
        match sender.try_send(cmd) {
            Err(SendError::ChannelFull) => {},
            _ => panic!("Expected ChannelFull error"),
        }
        
        // Check metrics
        let stats = channel.metrics();
        assert_eq!(stats.messages_sent, 10);
        assert_eq!(stats.backpressure_events, 1);
    }
}