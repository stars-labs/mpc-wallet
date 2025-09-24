// Optimized Event Loop Implementation
// Reduces CPU usage and improves responsiveness

use std::time::{Duration, Instant};
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::interval;
use tracing::{debug, info, warn};
use anyhow::Result;

/// Performance-optimized event loop with adaptive polling
pub struct OptimizedEventLoop<M> {
    /// Last time we saw user activity
    last_activity: Instant,
    
    /// Whether a render is needed
    render_needed: bool,
    
    /// Current polling interval (adaptive)
    poll_interval: Duration,
    
    /// Minimum polling interval (when active)
    min_interval: Duration,
    
    /// Maximum polling interval (when idle)
    max_interval: Duration,
    
    /// Time before considering the app idle
    idle_threshold: Duration,
    
    /// Message receiver
    message_rx: Receiver<M>,
    
    /// Performance metrics
    metrics: LoopMetrics,
    
    /// Render callback
    render_fn: Box<dyn FnMut() -> Result<()>>,
    
    /// Event handler callback
    event_handler: Box<dyn FnMut(CrosstermEvent) -> Result<Option<M>>>,
    
    /// Message handler callback
    message_handler: Box<dyn FnMut(M) -> Result<bool>>, // Returns true if render needed
}

/// Performance metrics for the event loop
#[derive(Debug, Default)]
pub struct LoopMetrics {
    /// Total events processed
    pub events_processed: u64,
    
    /// Total messages processed
    pub messages_processed: u64,
    
    /// Total renders performed
    pub renders_performed: u64,
    
    /// Average time between renders
    pub avg_render_interval_ms: f64,
    
    /// Current polling interval
    pub current_poll_interval_ms: u64,
    
    /// Time spent idle (percentage)
    pub idle_percentage: f64,
    
    /// Last render time
    last_render: Option<Instant>,
}

impl<M> OptimizedEventLoop<M> {
    pub fn new(
        message_rx: Receiver<M>,
        render_fn: impl FnMut() -> Result<()> + 'static,
        event_handler: impl FnMut(CrosstermEvent) -> Result<Option<M>> + 'static,
        message_handler: impl FnMut(M) -> Result<bool> + 'static,
    ) -> Self {
        Self {
            last_activity: Instant::now(),
            render_needed: true, // Initial render
            poll_interval: Duration::from_millis(10),
            min_interval: Duration::from_millis(5),
            max_interval: Duration::from_millis(200),
            idle_threshold: Duration::from_secs(2),
            message_rx,
            metrics: LoopMetrics::default(),
            render_fn: Box::new(render_fn),
            event_handler: Box::new(event_handler),
            message_handler: Box::new(message_handler),
        }
    }
    
    /// Configure polling intervals
    pub fn with_intervals(
        mut self,
        min_ms: u64,
        max_ms: u64,
        idle_threshold_secs: u64,
    ) -> Self {
        self.min_interval = Duration::from_millis(min_ms);
        self.max_interval = Duration::from_millis(max_ms);
        self.idle_threshold = Duration::from_secs(idle_threshold_secs);
        self.poll_interval = self.min_interval;
        self
    }
    
    /// Main event loop with optimizations
    pub async fn run(&mut self) -> Result<()> {
        info!("Starting optimized event loop");
        
        // Use deadline-based polling for better efficiency
        let mut poll_timer = interval(self.poll_interval);
        poll_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        
        // Track idle time for metrics
        let loop_start = Instant::now();
        let mut idle_time = Duration::ZERO;
        
        loop {
            let iteration_start = Instant::now();
            
            tokio::select! {
                // Terminal events with adaptive polling
                _ = poll_timer.tick() => {
                    let mut had_events = false;
                    
                    // Process all pending terminal events in batch
                    while event::poll(Duration::ZERO)? {
                        let event = event::read()?;
                        
                        // Handle resize immediately without going through message system
                        if let CrosstermEvent::Resize(_, _) = event {
                            self.render_needed = true;
                            debug!("Terminal resized");
                        } else {
                            // Process other events
                            if let Some(msg) = (self.event_handler)(event)? {
                                self.render_needed |= (self.message_handler)(msg)?;
                                self.metrics.messages_processed += 1;
                            }
                        }
                        
                        self.metrics.events_processed += 1;
                        self.last_activity = Instant::now();
                        had_events = true;
                    }
                    
                    if had_events {
                        self.render_needed = true;
                    }
                    
                    // Perform render if needed
                    if self.render_needed {
                        self.perform_render()?;
                    }
                    
                    // Update polling interval based on activity
                    self.update_poll_interval();
                    poll_timer = interval(self.poll_interval);
                    poll_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
                }
                
                // Handle async messages
                Some(msg) = self.message_rx.recv() => {
                    self.render_needed |= (self.message_handler)(msg)?;
                    self.metrics.messages_processed += 1;
                    self.last_activity = Instant::now();
                    
                    // Immediately render for messages
                    if self.render_needed {
                        self.perform_render()?;
                    }
                }
            }
            
            // Track idle time for metrics
            let iteration_time = iteration_start.elapsed();
            if iteration_time < self.poll_interval {
                idle_time += self.poll_interval - iteration_time;
            }
            
            // Update idle percentage periodically
            if self.metrics.events_processed % 100 == 0 {
                let total_time = loop_start.elapsed();
                self.metrics.idle_percentage = 
                    (idle_time.as_secs_f64() / total_time.as_secs_f64()) * 100.0;
            }
        }
    }
    
    /// Perform render and update metrics
    fn perform_render(&mut self) -> Result<()> {
        let render_start = Instant::now();
        
        (self.render_fn)()?;
        
        self.render_needed = false;
        self.metrics.renders_performed += 1;
        
        // Update average render interval
        if let Some(last) = self.metrics.last_render {
            let interval = last.elapsed().as_millis() as f64;
            let alpha = 0.1; // Exponential moving average factor
            self.metrics.avg_render_interval_ms = 
                alpha * interval + (1.0 - alpha) * self.metrics.avg_render_interval_ms;
        }
        self.metrics.last_render = Some(Instant::now());
        
        let render_time = render_start.elapsed();
        if render_time > Duration::from_millis(16) {
            warn!("Slow render: {:?}", render_time);
        }
        
        Ok(())
    }
    
    /// Update polling interval based on activity patterns
    fn update_poll_interval(&mut self) {
        let idle_duration = self.last_activity.elapsed();
        
        let new_interval = if idle_duration < Duration::from_millis(100) {
            // Very active - minimum interval
            self.min_interval
        } else if idle_duration < Duration::from_secs(1) {
            // Active - scale linearly
            let factor = idle_duration.as_millis() as f64 / 1000.0;
            Duration::from_millis(
                (self.min_interval.as_millis() as f64 * (1.0 + factor * 4.0)) as u64
            )
        } else if idle_duration < self.idle_threshold {
            // Semi-idle - medium interval
            Duration::from_millis(50)
        } else {
            // Idle - maximum interval
            self.max_interval
        };
        
        // Apply smoothing to prevent jumpy behavior
        if new_interval != self.poll_interval {
            let diff = if new_interval > self.poll_interval {
                Duration::from_millis(10) // Increase slowly
            } else {
                new_interval // Decrease immediately for responsiveness
            };
            
            self.poll_interval = if new_interval > self.poll_interval {
                (self.poll_interval + diff).min(new_interval)
            } else {
                new_interval
            };
            
            self.metrics.current_poll_interval_ms = self.poll_interval.as_millis() as u64;
            
            debug!(
                "Polling interval adjusted to {}ms (idle for {:?})",
                self.poll_interval.as_millis(),
                idle_duration
            );
        }
    }
    
    /// Get current performance metrics
    pub fn metrics(&self) -> &LoopMetrics {
        &self.metrics
    }
    
    /// Force an immediate render
    pub fn request_render(&mut self) {
        self.render_needed = true;
    }
}

/// Builder pattern for easier configuration
pub struct EventLoopBuilder<M> {
    min_interval_ms: u64,
    max_interval_ms: u64,
    idle_threshold_secs: u64,
    _phantom: std::marker::PhantomData<M>,
}

impl<M> Default for EventLoopBuilder<M> {
    fn default() -> Self {
        Self {
            min_interval_ms: 5,
            max_interval_ms: 200,
            idle_threshold_secs: 2,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<M> EventLoopBuilder<M> {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn min_interval_ms(mut self, ms: u64) -> Self {
        self.min_interval_ms = ms;
        self
    }
    
    pub fn max_interval_ms(mut self, ms: u64) -> Self {
        self.max_interval_ms = ms;
        self
    }
    
    pub fn idle_threshold_secs(mut self, secs: u64) -> Self {
        self.idle_threshold_secs = secs;
        self
    }
    
    pub fn build(
        self,
        message_rx: Receiver<M>,
        render_fn: impl FnMut() -> Result<()> + 'static,
        event_handler: impl FnMut(CrosstermEvent) -> Result<Option<M>> + 'static,
        message_handler: impl FnMut(M) -> Result<bool> + 'static,
    ) -> OptimizedEventLoop<M> {
        OptimizedEventLoop::new(message_rx, render_fn, event_handler, message_handler)
            .with_intervals(
                self.min_interval_ms,
                self.max_interval_ms,
                self.idle_threshold_secs,
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;
    
    #[tokio::test]
    async fn test_adaptive_polling() {
        let (tx, rx) = mpsc::channel(100);
        
        let mut loop_instance = OptimizedEventLoop::new(
            rx,
            || Ok(()),
            |_| Ok(None),
            |_| Ok(false),
        );
        
        // Initially should be at minimum interval
        assert_eq!(loop_instance.poll_interval, Duration::from_millis(5));
        
        // Simulate idle time
        loop_instance.last_activity = Instant::now() - Duration::from_secs(3);
        loop_instance.update_poll_interval();
        
        // Should be at maximum interval when idle
        assert_eq!(loop_instance.poll_interval, Duration::from_millis(200));
    }
}