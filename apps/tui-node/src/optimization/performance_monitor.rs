// Real-time performance monitoring for the MPC wallet
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::VecDeque;
use dashmap::DashMap;
use parking_lot::RwLock;

/// Performance monitoring dashboard
pub struct PerformanceMonitor {
    /// Message processing latencies (rolling window)
    message_latencies: Arc<RwLock<VecDeque<Duration>>>,
    /// Memory usage snapshots
    memory_snapshots: Arc<RwLock<VecDeque<MemorySnapshot>>>,
    /// Connection metrics by device
    connection_metrics: Arc<DashMap<String, ConnectionMetrics>>,
    /// DKG round timings
    dkg_timings: Arc<RwLock<DkgTimings>>,
    /// Global stats
    global_stats: Arc<RwLock<GlobalStats>>,
    /// Start time
    start_time: Instant,
}

#[derive(Clone, Debug)]
pub struct MemorySnapshot {
    pub timestamp: Instant,
    pub heap_size: usize,
    pub active_connections: usize,
    pub pending_messages: usize,
}

#[derive(Clone, Debug)]
pub struct ConnectionMetrics {
    pub established_at: Instant,
    pub last_activity: Instant,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub average_latency_ms: f64,
    pub reconnection_count: u32,
}

#[derive(Clone, Debug, Default)]
pub struct DkgTimings {
    pub round1_duration: Option<Duration>,
    pub round2_duration: Option<Duration>,
    pub round3_duration: Option<Duration>,
    pub total_duration: Option<Duration>,
    pub participant_count: usize,
}

#[derive(Clone, Debug, Default)]
pub struct GlobalStats {
    pub total_messages: u64,
    pub messages_per_second: f64,
    pub peak_memory_mb: f64,
    pub average_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub uptime: Duration,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            message_latencies: Arc::new(RwLock::new(VecDeque::with_capacity(10000))),
            memory_snapshots: Arc::new(RwLock::new(VecDeque::with_capacity(1000))),
            connection_metrics: Arc::new(DashMap::new()),
            dkg_timings: Arc::new(RwLock::new(DkgTimings::default())),
            global_stats: Arc::new(RwLock::new(GlobalStats::default())),
            start_time: Instant::now(),
        }
    }
    
    /// Record message processing latency
    pub fn record_message_latency(&self, latency: Duration) {
        let mut latencies = self.message_latencies.write();
        latencies.push_back(latency);
        
        // Keep only last 10000 measurements
        if latencies.len() > 10000 {
            latencies.pop_front();
        }
        
        // Update global stats
        self.update_latency_stats();
    }
    
    /// Record memory usage
    pub fn record_memory_snapshot(&self, heap_size: usize, active_connections: usize, pending_messages: usize) {
        let snapshot = MemorySnapshot {
            timestamp: Instant::now(),
            heap_size,
            active_connections,
            pending_messages,
        };
        
        let mut snapshots = self.memory_snapshots.write();
        snapshots.push_back(snapshot);
        
        // Keep only last 1000 snapshots
        if snapshots.len() > 1000 {
            snapshots.pop_front();
        }
        
        // Update peak memory
        let mut stats = self.global_stats.write();
        let heap_mb = heap_size as f64 / 1_048_576.0;
        if heap_mb > stats.peak_memory_mb {
            stats.peak_memory_mb = heap_mb;
        }
    }
    
    /// Update connection metrics
    pub fn update_connection(&self, device_id: String, sent: bool, latency_ms: Option<f64>) {
        self.connection_metrics
            .entry(device_id)
            .and_modify(|metrics| {
                metrics.last_activity = Instant::now();
                if sent {
                    metrics.messages_sent += 1;
                } else {
                    metrics.messages_received += 1;
                }
                
                if let Some(lat) = latency_ms {
                    // Exponential moving average for latency
                    let alpha = 0.1;
                    metrics.average_latency_ms = 
                        alpha * lat + (1.0 - alpha) * metrics.average_latency_ms;
                }
            })
            .or_insert_with(|| ConnectionMetrics {
                established_at: Instant::now(),
                last_activity: Instant::now(),
                messages_sent: if sent { 1 } else { 0 },
                messages_received: if sent { 0 } else { 1 },
                average_latency_ms: latency_ms.unwrap_or(0.0),
                reconnection_count: 0,
            });
    }
    
    /// Record DKG round timing
    pub fn record_dkg_round(&self, round: u8, duration: Duration, participant_count: usize) {
        let mut timings = self.dkg_timings.write();
        timings.participant_count = participant_count;
        
        match round {
            1 => timings.round1_duration = Some(duration),
            2 => timings.round2_duration = Some(duration),
            3 => timings.round3_duration = Some(duration),
            _ => {}
        }
        
        // Calculate total if all rounds complete
        if timings.round1_duration.is_some() && 
           timings.round2_duration.is_some() && 
           timings.round3_duration.is_some() {
            timings.total_duration = Some(
                timings.round1_duration.unwrap() +
                timings.round2_duration.unwrap() +
                timings.round3_duration.unwrap()
            );
        }
    }
    
    /// Update latency statistics
    fn update_latency_stats(&self) {
        let latencies = self.message_latencies.read();
        if latencies.is_empty() {
            return;
        }
        
        // Convert to sorted vector for percentile calculations
        let mut sorted: Vec<Duration> = latencies.iter().cloned().collect();
        sorted.sort();
        
        let len = sorted.len();
        let avg_ms = sorted.iter()
            .map(|d| d.as_secs_f64() * 1000.0)
            .sum::<f64>() / len as f64;
        
        let p95_idx = (len as f64 * 0.95) as usize;
        let p99_idx = (len as f64 * 0.99) as usize;
        
        let p95_ms = sorted.get(p95_idx)
            .map(|d| d.as_secs_f64() * 1000.0)
            .unwrap_or(0.0);
        
        let p99_ms = sorted.get(p99_idx)
            .map(|d| d.as_secs_f64() * 1000.0)
            .unwrap_or(0.0);
        
        let mut stats = self.global_stats.write();
        stats.average_latency_ms = avg_ms;
        stats.p95_latency_ms = p95_ms;
        stats.p99_latency_ms = p99_ms;
    }
    
    /// Get current performance report
    pub fn get_report(&self) -> PerformanceReport {
        let stats = self.global_stats.read().clone();
        let uptime = self.start_time.elapsed();
        
        // Calculate messages per second
        let latencies = self.message_latencies.read();
        let total_messages = latencies.len() as u64;
        let messages_per_second = if uptime.as_secs() > 0 {
            total_messages as f64 / uptime.as_secs_f64()
        } else {
            0.0
        };
        
        // Get active connections
        let active_connections = self.connection_metrics
            .iter()
            .filter(|entry| {
                entry.value().last_activity.elapsed() < Duration::from_secs(60)
            })
            .count();
        
        // Get latest memory snapshot
        let memory_snapshots = self.memory_snapshots.read();
        let latest_memory = memory_snapshots.back().cloned();
        
        PerformanceReport {
            uptime,
            total_messages,
            messages_per_second,
            active_connections,
            average_latency_ms: stats.average_latency_ms,
            p95_latency_ms: stats.p95_latency_ms,
            p99_latency_ms: stats.p99_latency_ms,
            peak_memory_mb: stats.peak_memory_mb,
            current_memory_mb: latest_memory
                .map(|s| s.heap_size as f64 / 1_048_576.0)
                .unwrap_or(0.0),
            dkg_timings: self.dkg_timings.read().clone(),
        }
    }
    
    /// Export metrics for external monitoring (Prometheus format)
    pub fn export_metrics(&self) -> String {
        let report = self.get_report();
        
        format!(
            r#"# HELP mpc_wallet_uptime_seconds Uptime in seconds
# TYPE mpc_wallet_uptime_seconds gauge
mpc_wallet_uptime_seconds {}

# HELP mpc_wallet_messages_total Total messages processed
# TYPE mpc_wallet_messages_total counter
mpc_wallet_messages_total {}

# HELP mpc_wallet_messages_per_second Messages processed per second
# TYPE mpc_wallet_messages_per_second gauge
mpc_wallet_messages_per_second {}

# HELP mpc_wallet_active_connections Number of active connections
# TYPE mpc_wallet_active_connections gauge
mpc_wallet_active_connections {}

# HELP mpc_wallet_latency_ms Message processing latency in milliseconds
# TYPE mpc_wallet_latency_ms summary
mpc_wallet_latency_ms{{quantile="0.5"}} {}
mpc_wallet_latency_ms{{quantile="0.95"}} {}
mpc_wallet_latency_ms{{quantile="0.99"}} {}

# HELP mpc_wallet_memory_mb Memory usage in megabytes
# TYPE mpc_wallet_memory_mb gauge
mpc_wallet_memory_mb{{type="current"}} {}
mpc_wallet_memory_mb{{type="peak"}} {}
"#,
            report.uptime.as_secs(),
            report.total_messages,
            report.messages_per_second,
            report.active_connections,
            report.average_latency_ms,
            report.p95_latency_ms,
            report.p99_latency_ms,
            report.current_memory_mb,
            report.peak_memory_mb,
        )
    }
}

#[derive(Clone, Debug)]
pub struct PerformanceReport {
    pub uptime: Duration,
    pub total_messages: u64,
    pub messages_per_second: f64,
    pub active_connections: usize,
    pub average_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub peak_memory_mb: f64,
    pub current_memory_mb: f64,
    pub dkg_timings: DkgTimings,
}

impl std::fmt::Display for PerformanceReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, 
            r#"
╔══════════════════════════════════════════════════════════════╗
║               MPC Wallet Performance Report                  ║
╠══════════════════════════════════════════════════════════════╣
║ Uptime:              {:>40} ║
║ Total Messages:      {:>40} ║
║ Messages/sec:        {:>40.2} ║
║ Active Connections:  {:>40} ║
╠══════════════════════════════════════════════════════════════╣
║                     Latency Statistics                       ║
╠══════════════════════════════════════════════════════════════╣
║ Average:             {:>38.2} ms ║
║ P95:                 {:>38.2} ms ║
║ P99:                 {:>38.2} ms ║
╠══════════════════════════════════════════════════════════════╣
║                     Memory Usage                             ║
╠══════════════════════════════════════════════════════════════╣
║ Current:             {:>38.2} MB ║
║ Peak:                {:>38.2} MB ║
╚══════════════════════════════════════════════════════════════╝
"#,
            format!("{:?}", self.uptime),
            self.total_messages,
            self.messages_per_second,
            self.active_connections,
            self.average_latency_ms,
            self.p95_latency_ms,
            self.p99_latency_ms,
            self.current_memory_mb,
            self.peak_memory_mb,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_performance_monitoring() {
        let monitor = PerformanceMonitor::new();
        
        // Record some latencies
        for i in 1..=100 {
            monitor.record_message_latency(Duration::from_millis(i));
        }
        
        // Record memory
        monitor.record_memory_snapshot(100_000_000, 5, 50);
        
        // Update connections
        monitor.update_connection("device1".to_string(), true, Some(25.0));
        monitor.update_connection("device2".to_string(), false, Some(30.0));
        
        // Get report
        let report = monitor.get_report();
        assert_eq!(report.total_messages, 100);
        assert!(report.average_latency_ms > 0.0);
        assert!(report.p95_latency_ms >= report.average_latency_ms);
    }
}