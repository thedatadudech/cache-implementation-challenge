use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use threadpool::ThreadPool;

// Import working cache implementations
use qwen30b_cache::SmartCache as Cache30B;
use qwen235b_cache::SmartCache as Cache235B;
use qwen435b_cache::SmartCache as Cache435B;
// GLM-4.5 excluded due to compilation errors in the model's code

fn benchmark_single_thread_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_thread");
    group.measurement_time(Duration::from_secs(10));
    
    // Test PUT operations with 1000 operations (matching Python/Java)
    group.bench_function("qwen30b_put_1000", |b| {
        let cache = Cache30B::new(100000);
        let mut i = 0;
        b.iter(|| {
            cache.put(i % 1000, black_box(format!("value_{}", i)), None, 5);
            i += 1;
        });
    });
    
    group.bench_function("qwen235b_put_1000", |b| {
        let cache = Cache235B::new(100000);
        let mut i = 0;
        b.iter(|| {
            cache.put(i % 1000, black_box(format!("value_{}", i)), None, 5);
            i += 1;
        });
    });
    
    group.bench_function("qwen435b_put_1000", |b| {
        let cache = Cache435B::new(100000);
        let mut i = 0;
        b.iter(|| {
            cache.put(i % 1000, black_box(format!("value_{}", i)), None, 5);
            i += 1;
        });
    });
    
    // Benchmark GET HIT operations (keys exist in cache)
    let cache_30b = Cache30B::new(100000);
    let cache_235b = Cache235B::new(100000);
    let cache_435b = Cache435B::new(100000);
    
    // Fill caches
    for i in 0..1000 {
        cache_30b.put(i, format!("value_{}", i), None, 5);
        cache_235b.put(i, format!("value_{}", i), None, 5);
        cache_435b.put(i, format!("value_{}", i), None, 5);
    }
    
    group.bench_function("qwen30b_get_hit", |b| {
        let mut i = 0;
        b.iter(|| {
            black_box(cache_30b.get(&(i % 1000)));
            i += 1;
        });
    });
    
    group.bench_function("qwen235b_get_hit", |b| {
        let mut i = 0;
        b.iter(|| {
            black_box(cache_235b.get(&(i % 1000)));
            i += 1;
        });
    });
    
    group.bench_function("qwen435b_get_hit", |b| {
        let mut i = 0;
        b.iter(|| {
            black_box(cache_435b.get(&(i % 1000)));
            i += 1;
        });
    });
    
    // Benchmark GET MISS operations (keys don't exist)
    group.bench_function("qwen30b_get_miss", |b| {
        let mut i = 1000;
        b.iter(|| {
            black_box(cache_30b.get(&i));
            i += 1;
        });
    });
    
    group.bench_function("qwen235b_get_miss", |b| {
        let mut i = 1000;
        b.iter(|| {
            black_box(cache_235b.get(&i));
            i += 1;
        });
    });
    
    group.bench_function("qwen435b_get_miss", |b| {
        let mut i = 1000;
        b.iter(|| {
            black_box(cache_435b.get(&i));
            i += 1;
        });
    });
    
    group.finish();
}

fn benchmark_concurrent_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent");
    group.sample_size(10);  // Reduce samples for concurrent tests
    group.measurement_time(Duration::from_secs(15));
    
    // Test with 10 threads (matching Python/Java)
    group.bench_function("qwen30b_10_threads", |b| {
        b.iter(|| {
            let cache = Arc::new(Cache30B::new(100000));
            let mut handles = vec![];
            
            for i in 0..10 {
                let cache_clone = Arc::clone(&cache);
                let handle = thread::spawn(move || {
                    for j in 0..100 {
                        let key = i * 100 + j;
                        cache_clone.put(key, format!("value_{}", key), None, 5);
                        black_box(cache_clone.get(&key));
                    }
                });
                handles.push(handle);
            }
            
            for handle in handles {
                handle.join().unwrap();
            }
        });
    });
    
    group.bench_function("qwen235b_10_threads", |b| {
        b.iter(|| {
            let cache = Arc::new(Cache235B::new(100000));
            let mut handles = vec![];
            
            for i in 0..10 {
                let cache_clone = Arc::clone(&cache);
                let handle = thread::spawn(move || {
                    for j in 0..100 {
                        let key = i * 100 + j;
                        cache_clone.put(key, format!("value_{}", key), None, 5);
                        black_box(cache_clone.get(&key));
                    }
                });
                handles.push(handle);
            }
            
            for handle in handles {
                handle.join().unwrap();
            }
        });
    });
    
    group.bench_function("qwen435b_10_threads", |b| {
        b.iter(|| {
            let cache = Arc::new(Cache435B::new(100000));
            let mut handles = vec![];
            
            for i in 0..10 {
                let cache_clone = Arc::clone(&cache);
                let handle = thread::spawn(move || {
                    for j in 0..100 {
                        let key = i * 100 + j;
                        cache_clone.put(key, format!("value_{}", key), None, 5);
                        black_box(cache_clone.get(&key));
                    }
                });
                handles.push(handle);
            }
            
            for handle in handles {
                handle.join().unwrap();
            }
        });
    });
    
    // Test with 100 thread pool (realistic for high-concurrency production)
    group.bench_function("qwen30b_100_thread_pool", |b| {
        b.iter(|| {
            let cache = Arc::new(Cache30B::new(100000));
            let pool = ThreadPool::new(100);
            let (tx, rx) = std::sync::mpsc::channel();
            
            for i in 0..100 {
                let cache_clone = Arc::clone(&cache);
                let tx = tx.clone();
                pool.execute(move || {
                    for j in 0..10 {  // 100 threads * 10 ops = 1000 total
                        let key = i * 10 + j;
                        cache_clone.put(key, format!("value_{}", key), None, 5);
                        black_box(cache_clone.get(&key));
                    }
                    tx.send(()).unwrap();
                });
            }
            
            // Wait for all tasks to complete
            for _ in 0..100 {
                rx.recv().unwrap();
            }
        });
    });
    
    group.bench_function("qwen235b_100_thread_pool", |b| {
        b.iter(|| {
            let cache = Arc::new(Cache235B::new(100000));
            let pool = ThreadPool::new(100);
            let (tx, rx) = std::sync::mpsc::channel();
            
            for i in 0..100 {
                let cache_clone = Arc::clone(&cache);
                let tx = tx.clone();
                pool.execute(move || {
                    for j in 0..20 {
                        let key = i * 10 + j;
                        cache_clone.put(key, format!("value_{}", key), None, 5);
                        black_box(cache_clone.get(&key));
                    }
                    tx.send(()).unwrap();
                });
            }
            
            for _ in 0..100 {
                rx.recv().unwrap();
            }
        });
    });
    
    group.bench_function("qwen435b_100_thread_pool", |b| {
        b.iter(|| {
            let cache = Arc::new(Cache435B::new(100000));
            let pool = ThreadPool::new(100);
            let (tx, rx) = std::sync::mpsc::channel();
            
            for i in 0..100 {
                let cache_clone = Arc::clone(&cache);
                let tx = tx.clone();
                pool.execute(move || {
                    for j in 0..20 {
                        let key = i * 10 + j;
                        cache_clone.put(key, format!("value_{}", key), None, 5);
                        black_box(cache_clone.get(&key));
                    }
                    tx.send(()).unwrap();
                });
            }
            
            for _ in 0..100 {
                rx.recv().unwrap();
            }
        });
    });
    
    group.finish();
}

fn benchmark_eviction_strategies(c: &mut Criterion) {
    let mut group = c.benchmark_group("eviction");
    group.measurement_time(Duration::from_secs(10));
    
    // Test eviction performance with 100 capacity cache (matching Python/Java)
    group.bench_function("qwen30b_eviction_100", |b| {
        b.iter(|| {
            let cache = Cache30B::new(100);
            
            // Fill cache to capacity with varying priorities
            for i in 0..100 {
                cache.put(i, format!("value_{}", i), None, (i % 10) as u8 + 1);
            }
            
            // Force eviction by adding 100 more items
            for i in 100..200 {
                cache.put(i, format!("value_{}", i), None, 5);
            }
            
            // Return evicted count (we expect 100 evictions)
            black_box(100);
        });
    });
    
    group.bench_function("qwen235b_eviction_100", |b| {
        b.iter(|| {
            let cache = Cache235B::new(100);
            
            // Fill cache to capacity with varying priorities
            for i in 0..100 {
                cache.put(i, format!("value_{}", i), None, (i % 10) as u8 + 1);
            }
            
            // Force eviction by adding 100 more items
            for i in 100..200 {
                cache.put(i, format!("value_{}", i), None, 5);
            }
            
            // Return evicted count
            black_box(100);
        });
    });
    
    group.bench_function("qwen435b_eviction_100", |b| {
        b.iter(|| {
            let cache = Cache435B::new(100);
            
            // Fill cache to capacity with varying priorities
            for i in 0..100 {
                cache.put(i, format!("value_{}", i), None, (i % 10) as u8 + 1);
            }
            
            // Force eviction by adding 100 more items
            for i in 100..200 {
                cache.put(i, format!("value_{}", i), None, 5);
            }
            
            // Return evicted count
            black_box(100);
        });
    });
    
    group.finish();
}

fn benchmark_ttl_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("ttl");
    group.measurement_time(Duration::from_secs(10));
    
    // Test TTL expiration (matching Python/Java)
    group.bench_function("qwen30b_ttl_expiry", |b| {
        b.iter(|| {
            let cache = Cache30B::new(200);
            
            // Add 100 items with 1ms TTL
            for i in 0..100 {
                cache.put(
                    i, 
                    format!("value_{}", i), 
                    Some(Duration::from_millis(1)), 
                    5
                );
            }
            
            // Sleep to expire items
            thread::sleep(Duration::from_millis(2));
            
            // Check expired items (should return None)
            let mut expired = 0;
            for i in 0..100 {
                if cache.get(&i).is_none() {
                    expired += 1;
                }
            }
            
            black_box(expired);
        });
    });
    
    group.bench_function("qwen235b_ttl_expiry", |b| {
        b.iter(|| {
            let cache = Cache235B::new(200);
            
            // Add 100 items with 1ms TTL
            for i in 0..100 {
                cache.put(
                    i, 
                    format!("value_{}", i), 
                    Some(Duration::from_millis(1)), 
                    5
                );
            }
            
            // Sleep to expire items
            thread::sleep(Duration::from_millis(2));
            
            // Check expired items
            let mut expired = 0;
            for i in 0..100 {
                if cache.get(&i).is_none() {
                    expired += 1;
                }
            }
            
            black_box(expired);
        });
    });
    
    group.bench_function("qwen435b_ttl_expiry", |b| {
        b.iter(|| {
            let cache = Cache435B::new(200);
            
            // Add 100 items with 1ms TTL
            for i in 0..100 {
                cache.put(
                    i, 
                    format!("value_{}", i), 
                    Some(Duration::from_millis(1)), 
                    5
                );
            }
            
            // Sleep to expire items
            thread::sleep(Duration::from_millis(2));
            
            // Check expired items
            let mut expired = 0;
            for i in 0..100 {
                if cache.get(&i).is_none() {
                    expired += 1;
                }
            }
            
            black_box(expired);
        });
    });
    
    // Test TTL check performance
    group.bench_function("qwen30b_ttl_check", |b| {
        let cache = Cache30B::new(100000);
        
        // Add items with long TTL
        for i in 0..100 {
            cache.put(
                i, 
                format!("value_{}", i), 
                Some(Duration::from_secs(3600)), 
                5
            );
        }
        
        b.iter(|| {
            // Check if items are still valid
            for i in 0..100 {
                black_box(cache.get(&i));
            }
        });
    });
    
    group.bench_function("qwen235b_ttl_check", |b| {
        let cache = Cache235B::new(100000);
        
        // Add items with long TTL
        for i in 0..100 {
            cache.put(
                i, 
                format!("value_{}", i), 
                Some(Duration::from_secs(3600)), 
                5
            );
        }
        
        b.iter(|| {
            // Check if items are still valid
            for i in 0..100 {
                black_box(cache.get(&i));
            }
        });
    });
    
    group.bench_function("qwen435b_ttl_check", |b| {
        let cache = Cache435B::new(100000);
        
        // Add items with long TTL
        for i in 0..100 {
            cache.put(
                i, 
                format!("value_{}", i), 
                Some(Duration::from_secs(3600)), 
                5
            );
        }
        
        b.iter(|| {
            // Check if items are still valid
            for i in 0..100 {
                black_box(cache.get(&i));
            }
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_single_thread_operations,
    benchmark_concurrent_operations,
    benchmark_eviction_strategies,
    benchmark_ttl_operations
);
criterion_main!(benches);