//! Adaptive Event Loop with Dynamic Polling Intervals
//!
//! This module provides an optimized event loop that adjusts polling intervals
//! based on user activity to reduce CPU usage while maintaining responsiveness.

use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Configuration for adaptive polling intervals
#[derive(Debug, Clone)]
pub struct AdaptiveConfig {
    /// Minimum polling interval during active use (milliseconds)
    pub min_interval_ms: u64,
    /// Maximum polling interval when idle (milliseconds)
    pub max_interval_ms: u64,
    /// How quickly to increase interval when idle (multiplier)
    pub idle_growth_factor: f32,
    /// How quickly to decrease interval when active (multiplier)
    pub active_decay_factor: f32,
    /// Time without events before considering idle (seconds)
    pub idle_timeout_secs: u64,
}

impl Default for AdaptiveConfig {
    fn default() -> Self {
        Self {
            min_interval_ms: 5,      // 5ms for responsive UI
            max_interval_ms: 200,    // 200ms when idle (5 FPS)
            idle_growth_factor: 1.5, // Grow by 50% each iteration
            active_decay_factor: 0.1, // Drop to 10% immediately on activity
            idle_timeout_secs: 2,    // Consider idle after 2 seconds
        }
    }
}

/// Adaptive event loop that reduces CPU usage when idle
pub struct AdaptiveEventLoop {
    config: AdaptiveConfig,
    current_interval_ms: u64,
    last_activity: Instant,
    is_idle: bool,
    /// Performance metrics
    pub total_polls: u64,
    pub idle_polls: u64,
    pub active_polls: u64,
}

impl AdaptiveEventLoop {
    /// Create a new adaptive event loop with default configuration
    pub fn new() -> Self {
        Self::with_config(AdaptiveConfig::default())
    }
    
    /// Create a new adaptive event loop with custom configuration
    pub fn with_config(config: AdaptiveConfig) -> Self {
        Self {
            current_interval_ms: config.min_interval_ms,
            config,
            last_activity: Instant::now(),
            is_idle: false,
            total_polls: 0,
            idle_polls: 0,
            active_polls: 0,
        }
    }
    
    /// Register activity (user input, network event, etc.)
    pub fn register_activity(&mut self) {
        self.last_activity = Instant::now();
        
        if self.is_idle {
            // Rapidly decrease interval when becoming active
            self.current_interval_ms = (self.current_interval_ms as f32 * self.config.active_decay_factor) as u64;
            self.current_interval_ms = self.current_interval_ms.max(self.config.min_interval_ms);
            self.is_idle = false;
            tracing::debug!("⚡ Switching to active mode, interval: {}ms", self.current_interval_ms);
        }
    }
    
    /// Update the polling interval based on activity
    fn update_interval(&mut self) {
        let time_since_activity = self.last_activity.elapsed();
        
        if time_since_activity > Duration::from_secs(self.config.idle_timeout_secs) {
            // We're idle, gradually increase the interval
            if !self.is_idle {
                self.is_idle = true;
                tracing::debug!("😴 Switching to idle mode");
            }
            
            self.current_interval_ms = (self.current_interval_ms as f32 * self.config.idle_growth_factor) as u64;
            self.current_interval_ms = self.current_interval_ms.min(self.config.max_interval_ms);
            
            self.idle_polls += 1;
        } else {
            // We're active, keep interval low
            self.current_interval_ms = self.config.min_interval_ms;
            self.active_polls += 1;
        }
        
        self.total_polls += 1;
    }
    
    /// Get the current polling interval
    pub fn get_interval(&self) -> Duration {
        Duration::from_millis(self.current_interval_ms)
    }
    
    /// Wait for the next poll interval
    pub async fn wait_next(&mut self) {
        self.update_interval();
        sleep(self.get_interval()).await;
    }
    
    /// Get performance statistics
    pub fn get_stats(&self) -> EventLoopStats {
        EventLoopStats {
            total_polls: self.total_polls,
            idle_polls: self.idle_polls,
            active_polls: self.active_polls,
            current_interval_ms: self.current_interval_ms,
            is_idle: self.is_idle,
            idle_percentage: if self.total_polls > 0 {
                (self.idle_polls as f64 / self.total_polls as f64) * 100.0
            } else {
                0.0
            },
        }
    }
    
    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.total_polls = 0;
        self.idle_polls = 0;
        self.active_polls = 0;
    }
}

/// Statistics about event loop performance
#[derive(Debug, Clone)]
pub struct EventLoopStats {
    pub total_polls: u64,
    pub idle_polls: u64,
    pub active_polls: u64,
    pub current_interval_ms: u64,
    pub is_idle: bool,
    pub idle_percentage: f64,
}

impl std::fmt::Display for EventLoopStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "EventLoop Stats: {} total polls ({:.1}% idle), current interval: {}ms, state: {}",
            self.total_polls,
            self.idle_percentage,
            self.current_interval_ms,
            if self.is_idle { "IDLE" } else { "ACTIVE" }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_adaptive_intervals() {
        let mut loop_controller = AdaptiveEventLoop::new();
        
        // Initially should be at minimum interval
        assert_eq!(loop_controller.get_interval(), Duration::from_millis(5));
        
        // Simulate idle time
        loop_controller.last_activity = Instant::now() - Duration::from_secs(3);
        loop_controller.update_interval();
        
        // Should have increased interval
        assert!(loop_controller.current_interval_ms > 5);
        assert!(loop_controller.is_idle);
        
        // Register activity
        loop_controller.register_activity();
        
        // Should rapidly decrease interval
        assert_eq!(loop_controller.current_interval_ms, 5);
        assert!(!loop_controller.is_idle);
    }
    
    #[test]
    fn test_stats_tracking() {
        let mut loop_controller = AdaptiveEventLoop::new();
        
        // Simulate some polls
        for _ in 0..10 {
            loop_controller.update_interval();
        }
        
        let stats = loop_controller.get_stats();
        assert_eq!(stats.total_polls, 10);
        assert_eq!(stats.active_polls, 10);
        assert_eq!(stats.idle_polls, 0);
        
        // Simulate idle
        loop_controller.last_activity = Instant::now() - Duration::from_secs(3);
        for _ in 0..5 {
            loop_controller.update_interval();
        }
        
        let stats = loop_controller.get_stats();
        assert_eq!(stats.total_polls, 15);
        assert_eq!(stats.idle_polls, 5);
    }
}