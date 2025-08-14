// Qwen3-30B Rust Implementation - Score: 85/100
// Basic Rust implementation with RwLock and VecDeque

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock, Mutex};
use std::time::{Duration, Instant};
use std::thread;

#[derive(Debug, Clone)]
pub struct CacheEntry<V: Clone> {
    value: V,
    priority: u8,
    ttl: Instant,
    created_at: Instant,
    last_accessed: Instant,
    access_count: usize,
}

#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub max_capacity: usize,
    pub default_ttl: Duration,
    pub cleanup_interval: Duration,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_capacity: 1000,
            default_ttl: Duration::from_secs(3600),
            cleanup_interval: Duration::from_secs(60),
        }
    }
}

pub struct SmartCache<K, V> 
where
    K: Clone + Eq + std::hash::Hash,
    V: Clone,
{
    data: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    lru_queue: Arc<Mutex<VecDeque<K>>>,
    config: CacheConfig,
    stats: Arc<RwLock<CacheStats>>,
    cleanup_handle: Option<thread::JoinHandle<()>>,
}

#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub insertions: u64,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
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
        let data = Arc::new(RwLock::new(HashMap::new()));
        let lru_queue = Arc::new(Mutex::new(VecDeque::new()));
        let stats = Arc::new(RwLock::new(CacheStats::default()));
        
        // Start cleanup thread
        let data_clone = Arc::clone(&data);
        let lru_clone = Arc::clone(&lru_queue);
        let cleanup_interval = config.cleanup_interval;
        
        let cleanup_handle = thread::spawn(move || {
            loop {
                thread::sleep(cleanup_interval);
                Self::cleanup_expired(&data_clone, &lru_clone);
            }
        });
        
        Self {
            data,
            lru_queue,
            config,
            stats,
            cleanup_handle: Some(cleanup_handle),
        }
    }
    
    pub fn put(&self, key: K, value: V, ttl: Option<Duration>, priority: u8) -> bool {
        let ttl = ttl.unwrap_or(self.config.default_ttl);
        
        let mut data = self.data.write().unwrap();
        let mut lru_queue = self.lru_queue.lock().unwrap();
        
        // Check capacity and evict if necessary
        if !data.contains_key(&key) && data.len() >= self.config.max_capacity {
            self.evict_if_necessary(&mut data, &mut lru_queue);
        }
        
        // Create entry
        let entry = CacheEntry {
            value,
            priority: priority.min(10).max(1),
            ttl: Instant::now() + ttl,
            created_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 0,
        };
        
        // Update data structures
        data.insert(key.clone(), entry);
        lru_queue.retain(|k| k != &key);
        lru_queue.push_back(key);
        
        // Update stats
        self.stats.write().unwrap().insertions += 1;
        
        true
    }
    
    pub fn get(&self, key: &K) -> Option<V> {
        let mut data = self.data.write().unwrap();
        
        if let Some(entry) = data.get_mut(key) {
            // Check TTL
            if Instant::now() > entry.ttl {
                data.remove(key);
                self.lru_queue.lock().unwrap().retain(|k| k != key);
                self.stats.write().unwrap().misses += 1;
                return None;
            }
            
            // Update access metadata
            entry.last_accessed = Instant::now();
            entry.access_count += 1;
            let value = entry.value.clone();
            
            // Update LRU
            let mut lru_queue = self.lru_queue.lock().unwrap();
            lru_queue.retain(|k| k != key);
            lru_queue.push_back(key.clone());
            
            // Update stats
            self.stats.write().unwrap().hits += 1;
            
            Some(value)
        } else {
            self.stats.write().unwrap().misses += 1;
            None
        }
    }
    
    pub fn delete(&self, key: &K) -> bool {
        let mut data = self.data.write().unwrap();
        if data.remove(key).is_some() {
            self.lru_queue.lock().unwrap().retain(|k| k != key);
            true
        } else {
            false
        }
    }
    
    pub fn clear(&self) {
        self.data.write().unwrap().clear();
        self.lru_queue.lock().unwrap().clear();
    }
    
    pub fn get_stats(&self) -> CacheStats {
        self.stats.read().unwrap().clone()
    }
    
    pub fn size(&self) -> usize {
        self.data.read().unwrap().len()
    }
    
    fn evict_if_necessary(&self, data: &mut HashMap<K, CacheEntry<V>>, lru_queue: &mut VecDeque<K>) {
        // Find entry with lowest priority
        let mut eviction_candidate: Option<(K, u8)> = None;
        
        for key in lru_queue.iter() {
            if let Some(entry) = data.get(key) {
                match &eviction_candidate {
                    None => eviction_candidate = Some((key.clone(), entry.priority)),
                    Some((_, priority)) if entry.priority < *priority => {
                        eviction_candidate = Some((key.clone(), entry.priority));
                    }
                    _ => {}
                }
            }
        }
        
        if let Some((key, _)) = eviction_candidate {
            data.remove(&key);
            lru_queue.retain(|k| k != &key);
            self.stats.write().unwrap().evictions += 1;
        }
    }
    
    fn cleanup_expired(data: &Arc<RwLock<HashMap<K, CacheEntry<V>>>>, lru_queue: &Arc<Mutex<VecDeque<K>>>) {
        let mut data = data.write().unwrap();
        let mut lru_queue = lru_queue.lock().unwrap();
        let now = Instant::now();
        
        let expired_keys: Vec<K> = data
            .iter()
            .filter(|(_, entry)| now > entry.ttl)
            .map(|(key, _)| key.clone())
            .collect();
        
        for key in expired_keys {
            data.remove(&key);
            lru_queue.retain(|k| k != &key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_operations() {
        let cache = SmartCache::new(10);
        
        // Test put and get
        assert!(cache.put(1, "value1", None, 5));
        assert_eq!(cache.get(&1), Some("value1"));
        
        // Test miss
        assert_eq!(cache.get(&2), None);
        
        // Test delete
        assert!(cache.delete(&1));
        assert_eq!(cache.get(&1), None);
    }
    
    #[test]
    fn test_capacity_limit() {
        let cache = SmartCache::new(2);
        
        cache.put(1, "value1", None, 1);
        cache.put(2, "value2", None, 5);
        cache.put(3, "value3", None, 10); // Should evict key 1 (lowest priority)
        
        assert_eq!(cache.get(&1), None); // Evicted
        assert_eq!(cache.get(&2), Some("value2"));
        assert_eq!(cache.get(&3), Some("value3"));
    }
    
    #[test]
    fn test_ttl() {
        let cache = SmartCache::new(10);
        
        cache.put(1, "value1", Some(Duration::from_millis(100)), 5);
        assert_eq!(cache.get(&1), Some("value1"));
        
        thread::sleep(Duration::from_millis(150));
        assert_eq!(cache.get(&1), None); // Expired
    }
    
    #[test]
    fn test_stats() {
        let cache = SmartCache::new(10);
        
        cache.put(1, "value1", None, 5);
        cache.get(&1); // Hit
        cache.get(&2); // Miss
        
        let stats = cache.get_stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.insertions, 1);
    }
}
