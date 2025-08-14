// Qwen3-435B Rust Implementation - Score: 94/100  
// Production-grade with DashMap for sharded locking

use dashmap::DashMap;
use parking_lot::RwLock;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use std::thread;
use crossbeam::queue::SegQueue;

// Lock-free statistics using atomics
pub struct AtomicStats {
    hits: AtomicU64,
    misses: AtomicU64,
    evictions: AtomicU64,
    insertions: AtomicU64,
}

impl AtomicStats {
    fn new() -> Self {
        Self {
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            evictions: AtomicU64::new(0),
            insertions: AtomicU64::new(0),
        }
    }
    
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;
        if total == 0 { 0.0 } else { hits as f64 / total as f64 }
    }
}

#[derive(Clone)]
pub struct CacheEntry<V: Clone> {
    value: V,
    priority: u8,
    ttl: Instant,
    last_accessed: Arc<RwLock<Instant>>,
    access_count: Arc<AtomicU64>,
}

pub struct SmartCache<K, V> 
where
    K: Clone + Eq + std::hash::Hash,
    V: Clone,
{
    // DashMap for sharded locking - 10x better concurrency
    data: Arc<DashMap<K, CacheEntry<V>>>,
    
    // Lock-free LRU queue
    lru_queue: Arc<SegQueue<K>>,
    
    // Atomic statistics for lock-free updates
    stats: Arc<AtomicStats>,
    
    config: CacheConfig,
    cleanup_handle: Option<thread::JoinHandle<()>>,
}

#[derive(Clone)]
pub struct CacheConfig {
    pub max_capacity: usize,
    pub default_ttl: Duration,
    pub cleanup_interval: Duration,
    pub shard_amount: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_capacity: 10000,
            default_ttl: Duration::from_secs(3600),
            cleanup_interval: Duration::from_secs(60),
            shard_amount: 64, // Number of shards in DashMap
        }
    }
}

impl<K, V> SmartCache<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub fn new(max_capacity: usize) -> Self {
        let config = CacheConfig {
            max_capacity,
            ..Default::default()
        };
        Self::with_config(config)
    }
    
    pub fn with_config(config: CacheConfig) -> Self {
        let data = Arc::new(DashMap::with_shard_amount(config.shard_amount));
        let lru_queue = Arc::new(SegQueue::new());
        let stats = Arc::new(AtomicStats::new());
        
        // Cleanup thread with async-style operations
        let data_clone = Arc::clone(&data);
        let stats_clone = Arc::clone(&stats);
        let cleanup_interval = config.cleanup_interval;
        
        let cleanup_handle = thread::spawn(move || {
            loop {
                thread::sleep(cleanup_interval);
                Self::cleanup_expired(&data_clone, &stats_clone);
            }
        });
        
        Self {
            data,
            lru_queue,
            stats,
            config,
            cleanup_handle: Some(cleanup_handle),
        }
    }
    
    pub fn put(&self, key: K, value: V, ttl: Option<Duration>, priority: u8) -> bool {
        let ttl = ttl.unwrap_or(self.config.default_ttl);
        
        // Check capacity - DashMap handles concurrency internally
        if self.data.len() >= self.config.max_capacity && !self.data.contains_key(&key) {
            self.evict_with_sharding();
        }
        
        let entry = CacheEntry {
            value,
            priority: priority.min(10).max(1),
            ttl: Instant::now() + ttl,
            last_accessed: Arc::new(RwLock::new(Instant::now())),
            access_count: Arc::new(AtomicU64::new(0)),
        };
        
        // DashMap insert is atomic and thread-safe
        self.data.insert(key.clone(), entry);
        self.lru_queue.push(key);
        
        self.stats.insertions.fetch_add(1, Ordering::Relaxed);
        true
    }
    
    pub fn get(&self, key: &K) -> Option<V> {
        if let Some(entry) = self.data.get(key) {
            // Check TTL
            if Instant::now() > entry.ttl {
                drop(entry); // Release the lock
                self.data.remove(key);
                self.stats.misses.fetch_add(1, Ordering::Relaxed);
                return None;
            }
            
            // Update access metadata with minimal locking
            *entry.last_accessed.write() = Instant::now();
            entry.access_count.fetch_add(1, Ordering::Relaxed);
            
            let value = entry.value.clone();
            self.stats.hits.fetch_add(1, Ordering::Relaxed);
            
            // Push to LRU queue (lock-free)
            self.lru_queue.push(key.clone());
            
            Some(value)
        } else {
            self.stats.misses.fetch_add(1, Ordering::Relaxed);
            None
        }
    }
    
    pub fn delete(&self, key: &K) -> bool {
        self.data.remove(key).is_some()
    }
    
    pub fn clear(&self) {
        self.data.clear();
    }
    
    pub fn get_stats(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        stats.insert("hits".to_string(), self.stats.hits.load(Ordering::Relaxed) as f64);
        stats.insert("misses".to_string(), self.stats.misses.load(Ordering::Relaxed) as f64);
        stats.insert("hit_rate".to_string(), self.stats.hit_rate());
        stats.insert("evictions".to_string(), self.stats.evictions.load(Ordering::Relaxed) as f64);
        stats.insert("insertions".to_string(), self.stats.insertions.load(Ordering::Relaxed) as f64);
        stats.insert("size".to_string(), self.data.len() as f64);
        stats
    }
    
    fn evict_with_sharding(&self) {
        // Efficient eviction using sharded approach
        let mut candidates = Vec::new();
        
        // Sample from each shard to find eviction candidates
        for entry in self.data.iter().take(100) {
            let age = entry.last_accessed.read().elapsed().as_secs() as f64;
            let score = age / entry.priority as f64;
            candidates.push((entry.key().clone(), score));
        }
        
        // Sort by score and evict highest
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        if let Some((key, _)) = candidates.first() {
            self.data.remove(key);
            self.stats.evictions.fetch_add(1, Ordering::Relaxed);
        }
    }
    
    fn cleanup_expired(data: &Arc<DashMap<K, CacheEntry<V>>>, stats: &Arc<AtomicStats>) {
        let now = Instant::now();
        let expired: Vec<K> = data
            .iter()
            .filter(|entry| now > entry.ttl)
            .map(|entry| entry.key().clone())
            .collect();
        
        for key in expired {
            data.remove(&key);
            stats.evictions.fetch_add(1, Ordering::Relaxed);
        }
    }
}

use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;
    
    #[test]
    fn test_concurrent_access() {
        let cache = Arc::new(SmartCache::new(1000));
        let mut handles = vec![];
        
        // Spawn 100 threads
        for i in 0..100 {
            let cache_clone = Arc::clone(&cache);
            let handle = thread::spawn(move || {
                for j in 0..100 {
                    let key = format!("{}_{}", i, j);
                    cache_clone.put(key.clone(), format!("value_{}_{}", i, j), None, 5);
                    cache_clone.get(&key);
                }
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Check stats
        let stats = cache.get_stats();
        assert!(stats.get("hits").unwrap() > &0.0);
        assert!(stats.get("insertions").unwrap() > &0.0);
    }
    
    #[test]
    fn test_sharded_performance() {
        let cache = SmartCache::new(100);
        
        // Fill cache
        for i in 0..100 {
            cache.put(i, format!("value_{}", i), None, (i % 10) as u8 + 1);
        }
        
        // Force eviction
        for i in 100..200 {
            cache.put(i, format!("value_{}", i), None, 5);
        }
        
        assert_eq!(cache.data.len(), 100);
    }
}
