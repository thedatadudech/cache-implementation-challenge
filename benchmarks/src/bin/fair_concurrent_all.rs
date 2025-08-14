use std::sync::{Arc, atomic::{AtomicBool, AtomicUsize, Ordering}};
use std::thread;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use threadpool::ThreadPool;
use crossbeam::channel::{unbounded};
use rand::Rng;
use serde_json;

// Import the cache implementations with concrete types
type Cache30B = qwen30b_cache::SmartCache<String, String>;
type Cache235B = qwen235b_cache::SmartCache<String, String>;
type Cache435B = qwen435b_cache::SmartCache<String, String>;

// Macro to generate benchmark functions for each cache type
macro_rules! impl_benchmarks {
    ($cache_type:ty, $name:expr, $mod_name:ident) => {
        mod $mod_name {
            use super::*;
            
            pub fn benchmark_producer_consumer(num_producers: usize, num_consumers: usize, duration_secs: u64) -> HashMap<String, serde_json::Value> {
                let cache = Arc::new(<$cache_type>::new(100000));
                let stop_flag = Arc::new(AtomicBool::new(false));
                
                let mut producer_counts = Vec::new();
                let mut consumer_hits = Vec::new();
                let mut consumer_misses = Vec::new();
                
                for _ in 0..num_producers {
                    producer_counts.push(Arc::new(AtomicUsize::new(0)));
                }
                for _ in 0..num_consumers {
                    consumer_hits.push(Arc::new(AtomicUsize::new(0)));
                    consumer_misses.push(Arc::new(AtomicUsize::new(0)));
                }
                
                println!("\nRunning Producer-Consumer benchmark ({} producers, {} consumers)...", 
                        num_producers, num_consumers);
                println!("Duration: {} seconds", duration_secs);
                
                let start = Instant::now();
                let pool = ThreadPool::new(num_producers + num_consumers);
                
                // Start producers
                for i in 0..num_producers {
                    let cache = Arc::clone(&cache);
                    let stop = Arc::clone(&stop_flag);
                    let count = Arc::clone(&producer_counts[i]);
                    
                    pool.execute(move || {
                        let mut local_count = 0;
                        let mut rng = rand::thread_rng();
                        
                        while !stop.load(Ordering::Relaxed) {
                            let key = format!("p{}_item_{}", i, local_count % 1000);
                            let value = format!("data_{}_{}", local_count, 
                                std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
                            
                            cache.put(key, value, None, rng.gen_range(1..=10));
                            local_count += 1;
                            count.store(local_count, Ordering::Relaxed);
                            
                            thread::sleep(Duration::from_micros(100));
                        }
                    });
                }
                
                // Start consumers
                for i in 0..num_consumers {
                    let cache = Arc::clone(&cache);
                    let stop = Arc::clone(&stop_flag);
                    let hits = Arc::clone(&consumer_hits[i]);
                    let misses = Arc::clone(&consumer_misses[i]);
                    
                    pool.execute(move || {
                        let mut rng = rand::thread_rng();
                        
                        while !stop.load(Ordering::Relaxed) {
                            let producer_id = rng.gen_range(0..num_producers);
                            let item_id = rng.gen_range(0..1000);
                            let key = format!("p{}_item_{}", producer_id, item_id);
                            
                            if cache.get(&key).is_some() {
                                hits.fetch_add(1, Ordering::Relaxed);
                            } else {
                                misses.fetch_add(1, Ordering::Relaxed);
                            }
                            
                            thread::sleep(Duration::from_micros(100));
                        }
                    });
                }
                
                // Run for specified duration
                thread::sleep(Duration::from_secs(duration_secs));
                stop_flag.store(true, Ordering::Relaxed);
                
                // Wait for completion
                drop(pool);
                
                let elapsed = start.elapsed();
                
                // Calculate statistics
                let total_puts: usize = producer_counts.iter()
                    .map(|c| c.load(Ordering::Relaxed))
                    .sum();
                    
                let total_hits: usize = consumer_hits.iter()
                    .map(|c| c.load(Ordering::Relaxed))
                    .sum();
                    
                let total_misses: usize = consumer_misses.iter()
                    .map(|c| c.load(Ordering::Relaxed))
                    .sum();
                    
                let total_gets = total_hits + total_misses;
                let hit_rate = if total_gets > 0 { 
                    total_hits as f64 / total_gets as f64 
                } else { 
                    0.0 
                };
                
                let mut result = HashMap::new();
                result.insert("duration".to_string(), serde_json::json!(elapsed.as_secs_f64()));
                result.insert("total_puts".to_string(), serde_json::json!(total_puts));
                result.insert("total_gets".to_string(), serde_json::json!(total_gets));
                result.insert("total_operations".to_string(), serde_json::json!(total_puts + total_gets));
                result.insert("puts_per_second".to_string(), serde_json::json!(format!("{:.2}", total_puts as f64 / elapsed.as_secs_f64())));
                result.insert("gets_per_second".to_string(), serde_json::json!(format!("{:.2}", total_gets as f64 / elapsed.as_secs_f64())));
                result.insert("ops_per_second".to_string(), serde_json::json!(format!("{:.2}", (total_puts + total_gets) as f64 / elapsed.as_secs_f64())));
                result.insert("hit_rate".to_string(), serde_json::json!(format!("{:.1}%", hit_rate * 100.0)));
                result.insert("total_hits".to_string(), serde_json::json!(total_hits));
                result.insert("total_misses".to_string(), serde_json::json!(total_misses));
                
                result
            }
            
            pub fn benchmark_shared_workload(num_workers: usize, num_operations: usize) -> HashMap<String, serde_json::Value> {
                let cache = Arc::new(<$cache_type>::new(100000));
                
                // Create work queue
                let (tx, rx) = unbounded();
                let mut rng = rand::thread_rng();
                
                // Fill with mixed operations
                for i in 0..num_operations {
                    if rng.gen::<f64>() < 0.7 { // 70% writes
                        tx.send(("PUT", 
                                format!("key_{}", i % 1000),
                                format!("value_{}", i),
                                rng.gen_range(1..=10)))
                            .unwrap();
                    } else { // 30% reads
                        tx.send(("GET", 
                                format!("key_{}", rng.gen_range(0..1000)),
                                String::new(),
                                0))
                            .unwrap();
                    }
                }
                drop(tx); // Close sender
                
                let operation_times = Arc::new(parking_lot::Mutex::new(Vec::new()));
                
                println!("\nRunning Shared Workload benchmark ({} workers, {} operations)...", 
                        num_workers, num_operations);
                
                let start = Instant::now();
                let pool = ThreadPool::new(num_workers);
                let (done_tx, done_rx) = unbounded();
                
                // Start workers
                for _ in 0..num_workers {
                    let cache = Arc::clone(&cache);
                    let rx = rx.clone();
                    let times = Arc::clone(&operation_times);
                    let done = done_tx.clone();
                    
                    pool.execute(move || {
                        let mut local_times = Vec::new();
                        
                        while let Ok((op, key, value, priority)) = rx.recv() {
                            let op_start = Instant::now();
                            
                            match op {
                                "PUT" => {
                                    cache.put(key, value, None, priority);
                                },
                                "GET" => {
                                    let _ = cache.get(&key);
                                },
                                _ => {}
                            }
                            
                            local_times.push(op_start.elapsed());
                        }
                        
                        times.lock().extend(local_times);
                        done.send(()).unwrap();
                    });
                }
                
                drop(done_tx);
                // Wait for all workers to complete
                for _ in 0..num_workers {
                    done_rx.recv().unwrap();
                }
                
                let elapsed = start.elapsed();
                
                // Calculate statistics
                let times = operation_times.lock();
                let avg_op_time = if !times.is_empty() {
                    let sum: Duration = times.iter().sum();
                    sum.as_secs_f64() / times.len() as f64 * 1000.0 // Convert to ms
                } else {
                    0.0
                };
                
                let parallelism_factor = if elapsed.as_secs_f64() > 0.0 {
                    (avg_op_time * num_operations as f64 / 1000.0) / elapsed.as_secs_f64()
                } else {
                    1.0
                };
                
                let mut result = HashMap::new();
                result.insert("duration".to_string(), serde_json::json!(format!("{:.3}", elapsed.as_secs_f64())));
                result.insert("num_workers".to_string(), serde_json::json!(num_workers));
                result.insert("total_operations".to_string(), serde_json::json!(num_operations));
                result.insert("ops_per_second".to_string(), serde_json::json!(format!("{:.2}", num_operations as f64 / elapsed.as_secs_f64())));
                result.insert("avg_operation_time_ms".to_string(), serde_json::json!(format!("{:.3}", avg_op_time)));
                result.insert("parallelism_factor".to_string(), serde_json::json!(format!("{:.2}x", parallelism_factor)));
                
                result
            }
            
            pub fn benchmark_eviction_strategy(cache_size: usize, total_insertions: usize) -> HashMap<String, serde_json::Value> {
                let cache = Arc::new(<$cache_type>::new(cache_size));
                
                println!("\nRunning Eviction Strategy benchmark (cache size: {}, insertions: {})...", 
                        cache_size, total_insertions);
                
                let start = Instant::now();
                
                // Fill cache to capacity with varying priorities
                for i in 0..cache_size {
                    cache.put(
                        format!("key_{}", i), 
                        format!("value_{}", i), 
                        None, 
                        (i % 10 + 1) as u8
                    );
                }
                
                // Force evictions by adding more items than capacity
                let evictions_forced = total_insertions - cache_size;
                for i in cache_size..total_insertions {
                    cache.put(
                        format!("key_{}", i), 
                        format!("value_{}", i), 
                        None, 
                        5
                    );
                }
                
                let elapsed = start.elapsed();
                
                // Check which original items were evicted
                let mut original_items_remaining = 0;
                for i in 0..cache_size {
                    if cache.get(&format!("key_{}", i)).is_some() {
                        original_items_remaining += 1;
                    }
                }
                
                let evicted_count = cache_size - original_items_remaining;
                let eviction_efficiency = if evictions_forced > 0 {
                    (evicted_count as f64 / evictions_forced as f64 * 100.0)
                } else {
                    0.0
                };
                
                let mut result = HashMap::new();
                result.insert("duration".to_string(), serde_json::json!(format!("{:.3}", elapsed.as_secs_f64())));
                result.insert("cache_size".to_string(), serde_json::json!(cache_size));
                result.insert("total_insertions".to_string(), serde_json::json!(total_insertions));
                result.insert("evictions_forced".to_string(), serde_json::json!(evictions_forced));
                result.insert("evicted_count".to_string(), serde_json::json!(evicted_count));
                result.insert("ops_per_second".to_string(), serde_json::json!(format!("{:.2}", total_insertions as f64 / elapsed.as_secs_f64())));
                result.insert("eviction_efficiency".to_string(), serde_json::json!(format!("{:.1}%", eviction_efficiency)));
                
                result
            }
            
            pub fn benchmark_ttl_operations(num_items: usize, ttl_ms: u64) -> HashMap<String, serde_json::Value> {
                let cache = Arc::new(<$cache_type>::new(10000));
                
                println!("\nRunning TTL Operations benchmark ({} items with {}ms TTL)...", 
                        num_items, ttl_ms);
                
                // Part 1: TTL Expiry Test
                let start = Instant::now();
                
                // Add items with short TTL
                for i in 0..num_items {
                    cache.put(
                        format!("ttl_key_{}", i), 
                        format!("value_{}", i), 
                        Some(Duration::from_millis(ttl_ms)), 
                        5
                    );
                }
                
                // Wait for expiration
                thread::sleep(Duration::from_millis(ttl_ms + 10));
                
                // Check expired items
                let mut expired_count = 0;
                for i in 0..num_items {
                    if cache.get(&format!("ttl_key_{}", i)).is_none() {
                        expired_count += 1;
                    }
                }
                
                let expiry_elapsed = start.elapsed();
                
                // Part 2: TTL Check Performance (with valid items)
                // Add items with long TTL
                for i in 0..num_items {
                    cache.put(
                        format!("valid_key_{}", i), 
                        format!("value_{}", i), 
                        Some(Duration::from_secs(3600)), 
                        5
                    );
                }
                
                // Measure time to check all items
                let check_start = Instant::now();
                let mut valid_count = 0;
                for i in 0..num_items {
                    if cache.get(&format!("valid_key_{}", i)).is_some() {
                        valid_count += 1;
                    }
                }
                let check_elapsed = check_start.elapsed();
                
                let expiry_rate = if num_items > 0 {
                    (expired_count as f64 / num_items as f64 * 100.0)
                } else {
                    0.0
                };
                
                let check_ops_per_second = if check_elapsed.as_secs_f64() > 0.0 {
                    num_items as f64 / check_elapsed.as_secs_f64()
                } else {
                    0.0
                };
                
                let avg_check_time_us = if num_items > 0 {
                    (check_elapsed.as_secs_f64() * 1_000_000.0 / num_items as f64)
                } else {
                    0.0
                };
                
                let mut result = HashMap::new();
                result.insert("ttl_expiry_duration".to_string(), serde_json::json!(format!("{:.3}", expiry_elapsed.as_secs_f64())));
                result.insert("ttl_check_duration".to_string(), serde_json::json!(format!("{:.3}", check_elapsed.as_secs_f64())));
                result.insert("num_items".to_string(), serde_json::json!(num_items));
                result.insert("ttl_ms".to_string(), serde_json::json!(ttl_ms));
                result.insert("expired_count".to_string(), serde_json::json!(expired_count));
                result.insert("expiry_rate".to_string(), serde_json::json!(format!("{:.1}%", expiry_rate)));
                result.insert("valid_count".to_string(), serde_json::json!(valid_count));
                result.insert("check_ops_per_second".to_string(), serde_json::json!(format!("{:.2}", check_ops_per_second)));
                result.insert("avg_check_time_us".to_string(), serde_json::json!(format!("{:.2}", avg_check_time_us)));
                
                result
            }
            
            pub fn benchmark_io_simulation(num_workers: usize, duration_secs: u64) -> HashMap<String, serde_json::Value> {
                let cache = Arc::new(<$cache_type>::new(100000));
                let stop_flag = Arc::new(AtomicBool::new(false));
                
                let mut operation_counts = Vec::new();
                for _ in 0..num_workers {
                    operation_counts.push(Arc::new(AtomicUsize::new(0)));
                }
                
                println!("\nRunning I/O Simulation benchmark ({} workers)...", num_workers);
                println!("Simulating database/network delays where threading helps...");
                
                let start = Instant::now();
                let pool = ThreadPool::new(num_workers);
                
                // Start workers
                for i in 0..num_workers {
                    let cache = Arc::clone(&cache);
                    let stop = Arc::clone(&stop_flag);
                    let count = Arc::clone(&operation_counts[i]);
                    
                    pool.execute(move || {
                        let mut local_count = 0;
                        let mut rng = rand::thread_rng();
                        
                        while !stop.load(Ordering::Relaxed) {
                            // Simulate database query
                            thread::sleep(Duration::from_millis(5));
                            
                            let key = format!("worker_{}_item_{}", i, local_count % 100);
                            let value = format!("data_{}_{}", local_count, 
                                              std::time::SystemTime::now()
                                              .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
                            
                            // Cache the result
                            cache.put(key, value, None, 5);
                            
                            // Try to read some other worker's data
                            let other_worker = (i + rng.gen_range(1..num_workers)) % num_workers;
                            let other_key = format!("worker_{}_item_{}", other_worker, rng.gen_range(0..100));
                            
                            if cache.get(&other_key).is_some() {
                                // Simulate processing
                                thread::sleep(Duration::from_millis(1));
                            }
                            
                            local_count += 2; // PUT + GET
                            count.store(local_count, Ordering::Relaxed);
                        }
                    });
                }
                
                // Run for specified duration
                thread::sleep(Duration::from_secs(duration_secs));
                stop_flag.store(true, Ordering::Relaxed);
                
                // Wait for completion
                drop(pool);
                
                let elapsed = start.elapsed();
                
                // Calculate statistics
                let total_operations: usize = operation_counts.iter()
                    .map(|c| c.load(Ordering::Relaxed))
                    .sum();
                    
                let theoretical_sequential_time = total_operations as f64 * 0.006; // 6ms per op
                let speedup = theoretical_sequential_time / elapsed.as_secs_f64();
                
                let mut result = HashMap::new();
                result.insert("duration".to_string(), serde_json::json!(format!("{:.2}", elapsed.as_secs_f64())));
                result.insert("num_workers".to_string(), serde_json::json!(num_workers));
                result.insert("total_operations".to_string(), serde_json::json!(total_operations));
                result.insert("ops_per_second".to_string(), serde_json::json!(format!("{:.2}", total_operations as f64 / elapsed.as_secs_f64())));
                result.insert("ops_per_worker".to_string(), serde_json::json!(total_operations / num_workers));
                result.insert("theoretical_sequential_time".to_string(), serde_json::json!(format!("{:.2}", theoretical_sequential_time)));
                result.insert("speedup".to_string(), serde_json::json!(format!("{:.2}x", speedup)));
                
                result
            }
        }
    };
}

// Generate benchmarks for all three cache implementations
impl_benchmarks!(Cache30B, "Qwen30B", qwen30b);
impl_benchmarks!(Cache235B, "Qwen235B", qwen235b);
impl_benchmarks!(Cache435B, "Qwen435B", qwen435b);

fn run_all_benchmarks(name: &str, module: &str) -> HashMap<String, HashMap<String, serde_json::Value>> {
    println!("\n{}", "=".repeat(60));
    println!("Testing: {} Rust Implementation", name);
    println!("{}", "=".repeat(60));
    
    let mut all_results = HashMap::new();
    
    // Run benchmarks based on module
    let (pc_result, sw_result, io_result, evict_result, ttl_result) = match module {
        "qwen30b" => (
            qwen30b::benchmark_producer_consumer(50, 50, 5),
            qwen30b::benchmark_shared_workload(100, 10000),
            qwen30b::benchmark_io_simulation(100, 5),
            qwen30b::benchmark_eviction_strategy(100, 200),
            qwen30b::benchmark_ttl_operations(100, 100),
        ),
        "qwen235b" => (
            qwen235b::benchmark_producer_consumer(50, 50, 5),
            qwen235b::benchmark_shared_workload(100, 10000),
            qwen235b::benchmark_io_simulation(100, 5),
            qwen235b::benchmark_eviction_strategy(100, 200),
            qwen235b::benchmark_ttl_operations(100, 100),
        ),
        "qwen435b" => (
            qwen435b::benchmark_producer_consumer(50, 50, 5),
            qwen435b::benchmark_shared_workload(100, 10000),
            qwen435b::benchmark_io_simulation(100, 5),
            qwen435b::benchmark_eviction_strategy(100, 200),
            qwen435b::benchmark_ttl_operations(100, 100),
        ),
        _ => panic!("Unknown module"),
    };
    
    // Test 1: Producer-Consumer Pattern
    println!("\n1. Producer-Consumer Pattern");
    println!("{}", "-".repeat(40));
    all_results.insert("producer_consumer".to_string(), pc_result.clone());
    println!("\nResults:");
    for (key, value) in &pc_result {
        println!("  {}: {}", key, value);
    }
    
    // Test 2: Shared Workload
    println!("\n2. Shared Workload (Fair Comparison)");
    println!("{}", "-".repeat(40));
    all_results.insert("shared_workload".to_string(), sw_result.clone());
    println!("\nResults:");
    for (key, value) in &sw_result {
        println!("  {}: {}", key, value);
    }
    
    // Test 3: I/O Simulation
    println!("\n3. I/O-Bound Simulation");
    println!("{}", "-".repeat(40));
    all_results.insert("io_simulation".to_string(), io_result.clone());
    println!("\nResults:");
    for (key, value) in &io_result {
        println!("  {}: {}", key, value);
    }
    
    // Test 4: Eviction Strategy
    println!("\n4. Eviction Strategy");
    println!("{}", "-".repeat(40));
    all_results.insert("eviction".to_string(), evict_result.clone());
    println!("\nResults:");
    for (key, value) in &evict_result {
        println!("  {}: {}", key, value);
    }
    
    // Test 5: TTL Operations
    println!("\n5. TTL Operations");
    println!("{}", "-".repeat(40));
    all_results.insert("ttl".to_string(), ttl_result.clone());
    println!("\nResults:");
    for (key, value) in &ttl_result {
        println!("  {}: {}", key, value);
    }
    
    all_results
}

fn main() {
    println!("{}", "=".repeat(60));
    println!("Fair Concurrent Benchmark Suite");
    println!("Rust Implementation with True Parallelism");
    println!("{}", "=".repeat(60));
    
    // Test all three implementations
    let implementations = vec![
        ("Qwen30B", "qwen30b"),
        ("Qwen235B", "qwen235b"),
        ("Qwen435B", "qwen435b"),
    ];
    
    for (name, module) in implementations {
        let all_results = run_all_benchmarks(name, module);
        
        // Save results
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
        let output = serde_json::json!({
            "implementation": format!("Rust {} (Fair Concurrent)", name),
            "timestamp": timestamp,
            "cache_size": 100000,
            "benchmarks": all_results,
            "notes": {
                "parallelism": "Rust has true parallelism with lock contention",
                "io_benefit": "Threading provides significant speedup for I/O operations",
                "comparison": "These metrics are directly comparable across languages"
            }
        });
        
        let filename = format!("results/rust_{}_fair_concurrent_{}.json", name.to_lowercase(), timestamp);
        std::fs::create_dir_all("results").unwrap();
        std::fs::write(&filename, serde_json::to_string_pretty(&output).unwrap()).unwrap();
        
        println!("\n{}", "=".repeat(60));
        println!("Results saved to: {}", filename);
    }
    
    println!("\n{}", "=".repeat(60));
    println!("All Rust benchmarks complete!");
    println!("{}", "=".repeat(60));
}