// Performance Benchmarking Suite for MPC Wallet TUI
// Run with: cargo bench

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use mpc_wallet_lib::elm::message::Message;
use mpc_wallet_lib::elm::model::Model;
use mpc_wallet_lib::elm::update::update;
use std::time::Duration;
use tokio::runtime::Runtime;

/// Benchmark message processing throughput
fn benchmark_message_processing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("process_single_message", |b| {
        b.iter(|| {
            let mut model = Model::new("test".to_string());
            let msg = Message::Refresh;
            black_box(update(&mut model, msg));
        });
    });
    
    c.bench_function("process_navigation_message", |b| {
        b.iter(|| {
            let mut model = Model::new("test".to_string());
            let msg = Message::NavigateBack;
            black_box(update(&mut model, msg));
        });
    });
    
    // Benchmark batch message processing
    let mut group = c.benchmark_group("message_batch_processing");
    for size in [10, 100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut model = Model::new("test".to_string());
                for _ in 0..size {
                    update(&mut model, Message::Refresh);
                }
            });
        });
    }
    group.finish();
}

/// Benchmark state transitions
fn benchmark_state_transitions(c: &mut Criterion) {
    use mpc_wallet_lib::elm::model::Screen;
    
    c.bench_function("screen_push", |b| {
        b.iter(|| {
            let mut model = Model::new("test".to_string());
            model.push_screen(Screen::MainMenu);
            black_box(&model);
        });
    });
    
    c.bench_function("screen_pop", |b| {
        b.iter(|| {
            let mut model = Model::new("test".to_string());
            model.push_screen(Screen::MainMenu);
            model.push_screen(Screen::ManageWallets);
            model.pop_screen();
            black_box(&model);
        });
    });
    
    // Benchmark deep navigation stack
    c.bench_function("deep_navigation_stack", |b| {
        b.iter(|| {
            let mut model = Model::new("test".to_string());
            for i in 0..20 {
                model.push_screen(Screen::MainMenu);
            }
            for _ in 0..20 {
                model.pop_screen();
            }
            black_box(&model);
        });
    });
}

/// Benchmark keystore operations
fn benchmark_keystore_operations(c: &mut Criterion) {
    use mpc_wallet_lib::keystore::encryption::encrypt_keystore_data;
    use mpc_wallet_lib::keystore::encryption::decrypt_keystore_data;
    
    let test_data = vec![0u8; 1024]; // 1KB test data
    let password = "test_password";
    
    c.bench_function("keystore_encrypt_1kb", |b| {
        b.iter(|| {
            let encrypted = encrypt_keystore_data(&test_data, password).unwrap();
            black_box(encrypted);
        });
    });
    
    // Create encrypted data for decryption benchmark
    let encrypted = encrypt_keystore_data(&test_data, password).unwrap();
    
    c.bench_function("keystore_decrypt_1kb", |b| {
        b.iter(|| {
            let decrypted = decrypt_keystore_data(
                &encrypted.encrypted_data,
                password,
                &encrypted.salt,
                &encrypted.nonce
            ).unwrap();
            black_box(decrypted);
        });
    });
    
    // Benchmark larger payloads
    let large_data = vec![0u8; 100_000]; // 100KB
    
    c.bench_function("keystore_encrypt_100kb", |b| {
        b.iter(|| {
            let encrypted = encrypt_keystore_data(&large_data, password).unwrap();
            black_box(encrypted);
        });
    });
}

/// Benchmark memory allocations
fn benchmark_memory_operations(c: &mut Criterion) {
    use mpc_wallet_lib::elm::model::{WalletInfo, WalletState};
    
    c.bench_function("wallet_list_allocation", |b| {
        b.iter(|| {
            let mut wallets = Vec::new();
            for i in 0..100 {
                wallets.push(WalletInfo {
                    id: format!("wallet_{}", i),
                    name: format!("Wallet {}", i),
                    address: format!("0x{:040x}", i),
                    balance: format!("{} ETH", i),
                    participant_id: i as u16,
                    threshold: 2,
                    total_participants: 3,
                    curve_type: mpc_wallet_lib::elm::model::CurveType::Secp256k1,
                    created_at: chrono::Utc::now(),
                });
            }
            black_box(wallets);
        });
    });
    
    // Benchmark cloning operations
    c.bench_function("model_clone", |b| {
        let model = Model::new("test".to_string());
        b.iter(|| {
            black_box(model.clone());
        });
    });
}

/// Benchmark async operations
fn benchmark_async_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("tokio_channel_send", |b| {
        b.to_async(&rt).iter(|_| async {
            let (tx, mut rx) = tokio::sync::mpsc::channel::<Message>(100);
            tx.send(Message::Refresh).await.unwrap();
            black_box(rx.recv().await);
        });
    });
    
    c.bench_function("unbounded_channel_send", |b| {
        b.to_async(&rt).iter(|_| async {
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Message>();
            tx.send(Message::Refresh).unwrap();
            black_box(rx.recv().await);
        });
    });
}

/// Benchmark WebSocket message parsing
fn benchmark_network_operations(c: &mut Criterion) {
    use mpc_wallet_lib::protocal::signal::WebSocketMessage;
    
    let json_message = r#"{
        "type": "SessionUpdate",
        "session_id": "test-session",
        "accepted_devices": ["device1", "device2"],
        "update_type": "FullSync",
        "timestamp": 1234567890
    }"#;
    
    c.bench_function("websocket_message_parse", |b| {
        b.iter(|| {
            let msg: WebSocketMessage = serde_json::from_str(json_message).unwrap();
            black_box(msg);
        });
    });
    
    c.bench_function("websocket_message_serialize", |b| {
        let msg = WebSocketMessage::Ping;
        b.iter(|| {
            let json = serde_json::to_string(&msg).unwrap();
            black_box(json);
        });
    });
}

/// Benchmark UI rendering components
fn benchmark_ui_operations(c: &mut Criterion) {
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::widgets::{Block, Borders, List, ListItem};
    
    c.bench_function("render_list_100_items", |b| {
        let items: Vec<ListItem> = (0..100)
            .map(|i| ListItem::new(format!("Item {}", i)))
            .collect();
        
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL));
        
        let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 25));
        
        b.iter(|| {
            use ratatui::widgets::Widget;
            list.clone().render(Rect::new(0, 0, 80, 25), &mut buffer);
            black_box(&buffer);
        });
    });
}

/// Benchmark FROST cryptographic operations
fn benchmark_frost_operations(c: &mut Criterion) {
    // Note: These would require actual FROST setup which is complex
    // Placeholder for FROST-specific benchmarks
    
    c.bench_function("frost_nonce_generation", |b| {
        b.iter(|| {
            // Simulate nonce generation
            let nonce = vec![0u8; 32];
            black_box(nonce);
        });
    });
}

/// Create a comprehensive benchmark group
fn comprehensive_benchmarks(c: &mut Criterion) {
    // Memory pressure test
    c.bench_function("memory_pressure_test", |b| {
        b.iter(|| {
            let mut vecs = Vec::new();
            for _ in 0..100 {
                vecs.push(vec![0u8; 10_000]);
            }
            black_box(vecs);
        });
    });
    
    // CPU intensive operation
    c.bench_function("cpu_intensive_hash", |b| {
        use sha2::{Sha256, Digest};
        let data = vec![0u8; 1000];
        
        b.iter(|| {
            let mut hasher = Sha256::new();
            for _ in 0..100 {
                hasher.update(&data);
            }
            black_box(hasher.finalize());
        });
    });
}

criterion_group!(
    benches,
    benchmark_message_processing,
    benchmark_state_transitions,
    benchmark_keystore_operations,
    benchmark_memory_operations,
    benchmark_async_operations,
    benchmark_network_operations,
    benchmark_ui_operations,
    benchmark_frost_operations,
    comprehensive_benchmarks
);

criterion_main!(benches);