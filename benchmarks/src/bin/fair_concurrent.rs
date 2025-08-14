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

fn benchmark_shared_workload_30b(num_workers: usize, num_operations: usize) -> HashMap<String, serde_json::Value> {
    let cache = Arc::new(Cache30B::new(100000));
    
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
    
    println!("\nRunning Shared Workload benchmark ({} workers, {} operations)...", 
            num_workers, num_operations);
    
    let start = Instant::now();
    let pool = ThreadPool::new(num_workers);
    let (done_tx, done_rx) = unbounded();
    
    // Start workers
    for _ in 0..num_workers {
        let cache = Arc::clone(&cache);
        let rx = rx.clone();
        let done = done_tx.clone();
        
        pool.execute(move || {
            while let Ok((op, key, value, priority)) = rx.recv() {
                match op {
                    "PUT" => {
                        cache.put(key, value, None, priority);
                    },
                    "GET" => {
                        let _ = cache.get(&key);
                    },
                    _ => {}
                }
            }
            done.send(()).unwrap();
        });
    }
    
    drop(done_tx);
    // Wait for all workers to complete
    for _ in 0..num_workers {
        done_rx.recv().unwrap();
    }
    
    let elapsed = start.elapsed();
    
    let mut result = HashMap::new();
    result.insert("duration".to_string(), serde_json::json!(format!("{:.3}", elapsed.as_secs_f64())));
    result.insert("num_workers".to_string(), serde_json::json!(num_workers));
    result.insert("total_operations".to_string(), serde_json::json!(num_operations));
    result.insert("ops_per_second".to_string(), serde_json::json!(format!("{:.2}", num_operations as f64 / elapsed.as_secs_f64())));
    
    result
}

fn benchmark_shared_workload_235b(num_workers: usize, num_operations: usize) -> HashMap<String, serde_json::Value> {
    let cache = Arc::new(Cache235B::new(100000));
    
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
    
    println!("\nRunning Shared Workload benchmark ({} workers, {} operations)...", 
            num_workers, num_operations);
    
    let start = Instant::now();
    let pool = ThreadPool::new(num_workers);
    let (done_tx, done_rx) = unbounded();
    
    // Start workers
    for _ in 0..num_workers {
        let cache = Arc::clone(&cache);
        let rx = rx.clone();
        let done = done_tx.clone();
        
        pool.execute(move || {
            while let Ok((op, key, value, priority)) = rx.recv() {
                match op {
                    "PUT" => {
                        cache.put(key, value, None, priority);
                    },
                    "GET" => {
                        let _ = cache.get(&key);
                    },
                    _ => {}
                }
            }
            done.send(()).unwrap();
        });
    }
    
    drop(done_tx);
    // Wait for all workers to complete
    for _ in 0..num_workers {
        done_rx.recv().unwrap();
    }
    
    let elapsed = start.elapsed();
    
    let mut result = HashMap::new();
    result.insert("duration".to_string(), serde_json::json!(format!("{:.3}", elapsed.as_secs_f64())));
    result.insert("num_workers".to_string(), serde_json::json!(num_workers));
    result.insert("total_operations".to_string(), serde_json::json!(num_operations));
    result.insert("ops_per_second".to_string(), serde_json::json!(format!("{:.2}", num_operations as f64 / elapsed.as_secs_f64())));
    
    result
}

fn benchmark_shared_workload_435b(num_workers: usize, num_operations: usize) -> HashMap<String, serde_json::Value> {
    let cache = Arc::new(Cache435B::new(100000));
    
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
    
    println!("\nRunning Shared Workload benchmark ({} workers, {} operations)...", 
            num_workers, num_operations);
    
    let start = Instant::now();
    let pool = ThreadPool::new(num_workers);
    let (done_tx, done_rx) = unbounded();
    
    // Start workers
    for _ in 0..num_workers {
        let cache = Arc::clone(&cache);
        let rx = rx.clone();
        let done = done_tx.clone();
        
        pool.execute(move || {
            while let Ok((op, key, value, priority)) = rx.recv() {
                match op {
                    "PUT" => {
                        cache.put(key, value, None, priority);
                    },
                    "GET" => {
                        let _ = cache.get(&key);
                    },
                    _ => {}
                }
            }
            done.send(()).unwrap();
        });
    }
    
    drop(done_tx);
    // Wait for all workers to complete
    for _ in 0..num_workers {
        done_rx.recv().unwrap();
    }
    
    let elapsed = start.elapsed();
    
    let mut result = HashMap::new();
    result.insert("duration".to_string(), serde_json::json!(format!("{:.3}", elapsed.as_secs_f64())));
    result.insert("num_workers".to_string(), serde_json::json!(num_workers));
    result.insert("total_operations".to_string(), serde_json::json!(num_operations));
    result.insert("ops_per_second".to_string(), serde_json::json!(format!("{:.2}", num_operations as f64 / elapsed.as_secs_f64())));
    
    result
}

fn main() {
    println!("{}", "=".repeat(60));
    println!("Fair Concurrent Benchmark Suite");
    println!("Rust Implementation with True Parallelism");
    println!("{}", "=".repeat(60));
    
    // Test each implementation
    type BenchmarkFn = fn(usize, usize) -> HashMap<String, serde_json::Value>;
    let implementations: Vec<(&str, BenchmarkFn)> = vec![
        ("Qwen30B", benchmark_shared_workload_30b as BenchmarkFn),
        ("Qwen235B", benchmark_shared_workload_235b as BenchmarkFn),
        ("Qwen435B", benchmark_shared_workload_435b as BenchmarkFn),
    ];
    
    for (name, benchmark_fn) in implementations {
        println!("\n{}", "=".repeat(60));
        println!("Testing: {} Rust Implementation", name);
        println!("{}", "=".repeat(60));
        
        println!("\nShared Workload (Fair Comparison)");
        println!("{}", "-".repeat(40));
        
        let result = benchmark_fn(100, 10000);
        
        println!("\nResults:");
        for (key, value) in &result {
            println!("  {}: {}", key, value);
        }
        
        // Save results
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
        let output = serde_json::json!({
            "implementation": format!("Rust {} (Fair Concurrent)", name),
            "timestamp": timestamp,
            "cache_size": 100000,
            "benchmarks": {
                "shared_workload": result
            },
            "notes": {
                "parallelism": "Rust has true parallelism with lock contention",
                "comparison": "These metrics are directly comparable across languages"
            }
        });
        
        let filename = format!("results/rust_{}_fair_concurrent_{}.json", name.to_lowercase(), timestamp);
        std::fs::create_dir_all("results").unwrap();
        std::fs::write(&filename, serde_json::to_string_pretty(&output).unwrap()).unwrap();
        
        println!("\nResults saved to: {}", filename);
    }
    
    println!("\n{}", "=".repeat(60));
    println!("All Rust benchmarks complete!");
    println!("{}", "=".repeat(60));
}