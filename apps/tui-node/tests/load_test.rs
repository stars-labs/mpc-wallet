// Load Testing Suite for MPC Wallet TUI
// Tests the application under various load conditions

use mpc_wallet_lib::elm::app::ElmApp;
use mpc_wallet_lib::elm::message::Message;
use mpc_wallet_lib::utils::appstate_compat::AppState;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::sleep;

/// Helper to create a test application instance
async fn create_test_app() -> ElmApp<frost_secp256k1::Secp256k1> {
    let device_id = format!("test-device-{}", uuid::Uuid::new_v4());
    let app_state = Arc::new(Mutex::new(AppState::new(device_id.clone())));
    
    ElmApp::new(device_id, app_state).expect("Failed to create app")
}

/// Test: High-frequency message processing
#[tokio::test]
async fn test_high_frequency_messages() {
    let mut app = create_test_app().await;
    let sender = app.get_message_sender();
    
    // Send 1000 messages as fast as possible
    let start = Instant::now();
    let message_count = 1000;
    
    for i in 0..message_count {
        sender.send(Message::Refresh).expect("Failed to send message");
        
        // Every 100 messages, yield to allow processing
        if i % 100 == 0 {
            tokio::task::yield_now().await;
        }
    }
    
    let elapsed = start.elapsed();
    let messages_per_second = message_count as f64 / elapsed.as_secs_f64();
    
    println!("Processed {} messages in {:?}", message_count, elapsed);
    println!("Throughput: {:.2} messages/second", messages_per_second);
    
    // Assert reasonable performance (at least 100 msg/s)
    assert!(
        messages_per_second > 100.0,
        "Message processing too slow: {:.2} msg/s",
        messages_per_second
    );
}

/// Test: Concurrent message senders
#[tokio::test]
async fn test_concurrent_senders() {
    let mut app = create_test_app().await;
    let sender = app.get_message_sender();
    
    // Spawn 10 concurrent senders
    let mut handles = Vec::new();
    let start = Instant::now();
    
    for sender_id in 0..10 {
        let tx = sender.clone();
        let handle = tokio::spawn(async move {
            for i in 0..100 {
                tx.send(Message::Refresh).expect("Failed to send");
                
                // Simulate some processing delay
                if i % 10 == 0 {
                    sleep(Duration::from_millis(1)).await;
                }
            }
            sender_id
        });
        handles.push(handle);
    }
    
    // Wait for all senders to complete
    for handle in handles {
        handle.await.expect("Sender task failed");
    }
    
    let elapsed = start.elapsed();
    println!("10 concurrent senders completed in {:?}", elapsed);
    
    // Should complete within reasonable time (< 5 seconds)
    assert!(
        elapsed < Duration::from_secs(5),
        "Concurrent processing too slow: {:?}",
        elapsed
    );
}

/// Test: Memory usage under load
#[tokio::test]
async fn test_memory_under_load() {
    let mut app = create_test_app().await;
    let sender = app.get_message_sender();
    
    // Get initial memory usage
    let initial_memory = get_process_memory_mb();
    println!("Initial memory: {:.2} MB", initial_memory);
    
    // Send many messages with different types
    for i in 0..5000 {
        let msg = match i % 5 {
            0 => Message::Refresh,
            1 => Message::NavigateBack,
            2 => Message::ListWallets,
            3 => Message::NavigateHome,
            _ => Message::CloseModal,
        };
        
        sender.send(msg).expect("Failed to send");
        
        // Yield periodically
        if i % 500 == 0 {
            tokio::task::yield_now().await;
            
            // Check memory growth
            let current_memory = get_process_memory_mb();
            let growth = current_memory - initial_memory;
            println!("After {} messages: {:.2} MB (growth: {:.2} MB)", 
                     i, current_memory, growth);
        }
    }
    
    // Allow time for processing
    sleep(Duration::from_millis(500)).await;
    
    // Check final memory
    let final_memory = get_process_memory_mb();
    let total_growth = final_memory - initial_memory;
    
    println!("Final memory: {:.2} MB (total growth: {:.2} MB)", 
             final_memory, total_growth);
    
    // Memory growth should be reasonable (< 50MB for 5000 messages)
    assert!(
        total_growth < 50.0,
        "Excessive memory growth: {:.2} MB",
        total_growth
    );
}

/// Test: Navigation stress test
#[tokio::test]
async fn test_navigation_stress() {
    use mpc_wallet_lib::elm::model::Screen;
    
    let mut app = create_test_app().await;
    let sender = app.get_message_sender();
    
    let start = Instant::now();
    
    // Rapidly navigate between screens
    for _ in 0..100 {
        sender.send(Message::Navigate(Screen::MainMenu)).unwrap();
        sender.send(Message::Navigate(Screen::ManageWallets)).unwrap();
        sender.send(Message::NavigateBack).unwrap();
        sender.send(Message::Navigate(Screen::JoinSession)).unwrap();
        sender.send(Message::NavigateHome).unwrap();
    }
    
    let elapsed = start.elapsed();
    println!("500 navigation operations in {:?}", elapsed);
    
    // Should handle rapid navigation (< 2 seconds for 500 ops)
    assert!(
        elapsed < Duration::from_secs(2),
        "Navigation too slow: {:?}",
        elapsed
    );
}

/// Test: WebSocket message flood
#[tokio::test]
async fn test_websocket_message_flood() {
    use mpc_wallet_lib::protocal::signal::{WebSocketMessage, SessionUpdate, SessionUpdateType};
    
    let device_id = "test-device";
    let mut messages = Vec::new();
    
    // Generate 1000 WebSocket messages
    for i in 0..1000 {
        let msg = WebSocketMessage::SessionUpdate(SessionUpdate {
            session_id: format!("session-{}", i),
            accepted_devices: vec![device_id.to_string()],
            update_type: SessionUpdateType::FullSync,
            timestamp: i,
        });
        messages.push(msg);
    }
    
    let start = Instant::now();
    
    // Serialize all messages
    for msg in &messages {
        let _json = serde_json::to_string(msg).unwrap();
    }
    
    let serialization_time = start.elapsed();
    
    // Deserialize all messages
    let start = Instant::now();
    for msg in &messages {
        let json = serde_json::to_string(msg).unwrap();
        let _parsed: WebSocketMessage = serde_json::from_str(&json).unwrap();
    }
    
    let round_trip_time = start.elapsed();
    
    println!("Serialization of 1000 messages: {:?}", serialization_time);
    println!("Round-trip of 1000 messages: {:?}", round_trip_time);
    
    // Should handle 1000 messages quickly (< 1 second)
    assert!(
        round_trip_time < Duration::from_secs(1),
        "Message processing too slow: {:?}",
        round_trip_time
    );
}

/// Test: Keystore operations under load
#[tokio::test]
async fn test_keystore_load() {
    use mpc_wallet_lib::keystore::encryption::{encrypt_keystore_data, decrypt_keystore_data};
    
    let password = "test_password_123";
    let test_data = vec![0u8; 10_000]; // 10KB of data
    
    let start = Instant::now();
    
    // Perform 100 encrypt/decrypt cycles
    for _ in 0..100 {
        let encrypted = encrypt_keystore_data(&test_data, password)
            .expect("Encryption failed");
        
        let _decrypted = decrypt_keystore_data(
            &encrypted.encrypted_data,
            password,
            &encrypted.salt,
            &encrypted.nonce,
        ).expect("Decryption failed");
    }
    
    let elapsed = start.elapsed();
    let ops_per_second = 100.0 / elapsed.as_secs_f64();
    
    println!("100 encrypt/decrypt cycles in {:?}", elapsed);
    println!("Throughput: {:.2} operations/second", ops_per_second);
    
    // Should handle at least 10 ops/second (PBKDF2 is intentionally slow)
    assert!(
        ops_per_second > 10.0,
        "Keystore operations too slow: {:.2} ops/s",
        ops_per_second
    );
}

/// Test: Sustained load over time
#[tokio::test]
async fn test_sustained_load() {
    let mut app = create_test_app().await;
    let sender = app.get_message_sender();
    
    let test_duration = Duration::from_secs(10);
    let start = Instant::now();
    let mut message_count = 0;
    
    // Send messages continuously for 10 seconds
    while start.elapsed() < test_duration {
        sender.send(Message::Refresh).expect("Failed to send");
        message_count += 1;
        
        // Small delay to prevent overwhelming
        if message_count % 100 == 0 {
            sleep(Duration::from_millis(10)).await;
        }
    }
    
    let elapsed = start.elapsed();
    let messages_per_second = message_count as f64 / elapsed.as_secs_f64();
    
    println!("Sustained load test:");
    println!("  Duration: {:?}", elapsed);
    println!("  Messages sent: {}", message_count);
    println!("  Throughput: {:.2} msg/s", messages_per_second);
    
    // Should maintain reasonable throughput over time
    assert!(
        messages_per_second > 50.0,
        "Sustained throughput too low: {:.2} msg/s",
        messages_per_second
    );
}

/// Test: Recovery from overload
#[tokio::test]
async fn test_overload_recovery() {
    let mut app = create_test_app().await;
    let sender = app.get_message_sender();
    
    // Phase 1: Overload with messages
    println!("Phase 1: Creating overload condition...");
    for _ in 0..10000 {
        let _ = sender.send(Message::Refresh); // Ignore errors
    }
    
    // Phase 2: Wait for recovery
    println!("Phase 2: Waiting for recovery...");
    sleep(Duration::from_secs(2)).await;
    
    // Phase 3: Test normal operation
    println!("Phase 3: Testing normal operation...");
    let start = Instant::now();
    
    for _ in 0..100 {
        sender.send(Message::Refresh).expect("Failed to send after recovery");
        sleep(Duration::from_millis(10)).await;
    }
    
    let elapsed = start.elapsed();
    println!("Post-recovery 100 messages in {:?}", elapsed);
    
    // Should recover and process normally
    assert!(
        elapsed < Duration::from_secs(2),
        "Failed to recover from overload: {:?}",
        elapsed
    );
}

// Helper function to get process memory usage in MB
fn get_process_memory_mb() -> f64 {
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        if let Ok(status) = fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    if let Some(kb_str) = line.split_whitespace().nth(1) {
                        if let Ok(kb) = kb_str.parse::<f64>() {
                            return kb / 1024.0;
                        }
                    }
                }
            }
        }
    }
    
    // Fallback: return 0 if we can't determine memory usage
    0.0
}

/// Test: Benchmark report generation
#[tokio::test]
async fn test_performance_report() {
    use mpc_wallet_lib::optimization::performance_monitor::PerformanceMonitor;
    
    let monitor = PerformanceMonitor::new();
    
    // Simulate activity
    for i in 0..1000 {
        monitor.record_message_latency(Duration::from_millis(i % 100));
        
        if i % 100 == 0 {
            monitor.record_memory_snapshot(50_000_000 + i * 1000, 5, 50);
        }
        
        if i % 10 == 0 {
            monitor.update_connection(
                format!("device-{}", i % 5),
                i % 2 == 0,
                Some((i % 50) as f64),
            );
        }
    }
    
    // Record DKG timings
    monitor.record_dkg_round(1, Duration::from_secs(2), 3);
    monitor.record_dkg_round(2, Duration::from_secs(3), 3);
    monitor.record_dkg_round(3, Duration::from_secs(2), 3);
    
    // Get and display report
    let report = monitor.get_report();
    println!("{}", report);
    
    // Export metrics
    let metrics = monitor.export_metrics();
    assert!(!metrics.is_empty(), "Metrics export should not be empty");
    
    // Verify metrics are reasonable
    assert!(report.average_latency_ms > 0.0);
    assert!(report.p95_latency_ms >= report.average_latency_ms);
    assert!(report.p99_latency_ms >= report.p95_latency_ms);
}