// Performance benchmarks for MPC wallet optimization
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use tui_node::optimization::*;
use std::time::Duration;
use tokio::runtime::Runtime;

fn bench_message_batching(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("message_batching");
    
    // Test different batch sizes
    for batch_size in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            batch_size,
            |b, &size| {
                b.to_async(&rt).iter(|| async move {
                    let batcher = message_batcher_v2::MessageBatcherV2::<String>::new(
                        size,
                        Duration::from_millis(100),
                    );
                    
                    // Simulate message processing
                    for i in 0..1000 {
                        let msg = format!("message_{}", i);
                        black_box(batcher.add_message(
                            msg,
                            if i % 10 == 0 {
                                message_batcher_v2::Priority::Critical
                            } else {
                                message_batcher_v2::Priority::Normal
                            }
                        ));
                    }
                    
                    // Get batches
                    let mut total = 0;
                    while batcher.has_messages() {
                        let batch = batcher.get_batch();
                        total += batch.len();
                    }
                    black_box(total)
                });
            },
        );
    }
    group.finish();
}

fn bench_lockfree_vs_mutex(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("state_access");
    
    // Benchmark lock-free implementation
    group.bench_function("lockfree_dashmap", |b| {
        b.to_async(&rt).iter(|| async {
            let state = lockfree_state::LockFreeConnectionState::new();
            
            // Simulate concurrent access
            let mut handles = vec![];
            for i in 0..100 {
                let state_clone = state.connections.clone();
                handles.push(tokio::spawn(async move {
                    for j in 0..10 {
                        let device_id = format!("device_{}_{}", i, j);
                        state_clone.get(&device_id);
                    }
                }));
            }
            
            futures::future::join_all(handles).await;
        });
    });
    
    // Benchmark traditional mutex
    group.bench_function("traditional_mutex", |b| {
        use std::collections::HashMap;
        use tokio::sync::Mutex;
        use std::sync::Arc;
        
        b.to_async(&rt).iter(|| async {
            let state = Arc::new(Mutex::new(HashMap::<String, String>::new()));
            
            // Simulate concurrent access
            let mut handles = vec![];
            for i in 0..100 {
                let state_clone = state.clone();
                handles.push(tokio::spawn(async move {
                    for j in 0..10 {
                        let device_id = format!("device_{}_{}", i, j);
                        let guard = state_clone.lock().await;
                        guard.get(&device_id);
                        drop(guard);
                    }
                }));
            }
            
            futures::future::join_all(handles).await;
        });
    });
    
    group.finish();
}

fn bench_bounded_vs_unbounded_channels(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("channels");
    
    // Benchmark bounded channel
    group.bench_function("bounded_channel", |b| {
        b.to_async(&rt).iter(|| async {
            use tokio::sync::mpsc;
            
            let (tx, mut rx) = mpsc::channel::<String>(1000);
            
            // Producer
            let tx_clone = tx.clone();
            tokio::spawn(async move {
                for i in 0..1000 {
                    let _ = tx_clone.send(format!("msg_{}", i)).await;
                }
            });
            
            // Consumer
            let mut count = 0;
            while let Ok(msg) = tokio::time::timeout(
                Duration::from_millis(10),
                rx.recv()
            ).await {
                if msg.is_some() {
                    count += 1;
                }
                if count >= 1000 { break; }
            }
            
            black_box(count)
        });
    });
    
    // Benchmark unbounded channel
    group.bench_function("unbounded_channel", |b| {
        b.to_async(&rt).iter(|| async {
            use tokio::sync::mpsc;
            
            let (tx, mut rx) = mpsc::unbounded_channel::<String>();
            
            // Producer
            for i in 0..1000 {
                let _ = tx.send(format!("msg_{}", i));
            }
            
            // Consumer
            let mut count = 0;
            while let Some(_msg) = rx.recv().await {
                count += 1;
                if count >= 1000 { break; }
            }
            
            black_box(count)
        });
    });
    
    group.finish();
}

fn bench_connection_pool(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("connection_pool");
    
    // Test different pool sizes
    for pool_size in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(pool_size),
            pool_size,
            |b, &size| {
                b.to_async(&rt).iter(|| async move {
                    let pool = connection_pool::ConnectionPool::new(
                        connection_pool::PoolConfig {
                            max_connections: size,
                            idle_timeout: Duration::from_secs(60),
                            retry_limit: 3,
                            parallel_attempts: 3,
                        }
                    );
                    
                    // Simulate connection requests
                    let mut handles = vec![];
                    for i in 0..size {
                        let pool_clone = pool.clone();
                        handles.push(tokio::spawn(async move {
                            pool_clone.get_or_create(&format!("device_{}", i)).await
                        }));
                    }
                    
                    let results = futures::future::join_all(handles).await;
                    black_box(results.len())
                });
            },
        );
    }
    
    group.finish();
}

fn bench_deduplication(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("message_deduplication", |b| {
        b.to_async(&rt).iter(|| async {
            let dedup = deduplicator::MessageDeduplicator::new(
                Duration::from_secs(60),
                10000,
            );
            
            let mut processed = 0;
            
            // Send duplicate messages
            for i in 0..1000 {
                let msg_id = format!("msg_{}", i % 100); // 10x duplicates
                if dedup.should_process(&msg_id).await {
                    processed += 1;
                }
            }
            
            black_box(processed)
        });
    });
}

criterion_group!(
    benches,
    bench_message_batching,
    bench_lockfree_vs_mutex,
    bench_bounded_vs_unbounded_channels,
    bench_connection_pool,
    bench_deduplication
);
criterion_main!(benches);