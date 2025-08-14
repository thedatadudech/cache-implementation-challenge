// Qwen3-235B Rust Implementation - Score: 91/100
// Sophisticated architecture with custom doubly-linked list for O(1) LRU

use std::collections::HashMap;
use std::sync::{Arc, RwLock, Mutex};
use std::time::{Duration, Instant};
use std::thread;

// Custom doubly-linked list for perfect O(1) LRU operations
#[derive(Debug)]
struct LruNode<K: Clone> {
    key: K,
    prev: Option<K>,
    next: Option<K>,
}

struct LruList<K: Clone + Eq + std::hash::Hash> {
    nodes: HashMap<K, LruNode<K>>,
    head: Option<K>,
    tail: Option<K>,
}

impl<K: Clone + Eq + std::hash::Hash> LruList<K> {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            head: None,
            tail: None,
        }
    }
    
    fn push_front(&mut self, key: K) {
        let node = LruNode {
            key: key.clone(),
            prev: None,
            next: self.head.clone(),
        };
        
        if let Some(ref head_key) = self.head {
            if let Some(head_node) = self.nodes.get_mut(head_key) {
                head_node.prev = Some(key.clone());
            }
        }
        
        self.nodes.insert(key.clone(), node);
        self.head = Some(key.clone());
        
        if self.tail.is_none() {
            self.tail = Some(key);
        }
    }
    
    fn remove(&mut self, key: &K) -> bool {
        if let Some(node) = self.nodes.remove(key) {
            // Update prev node's next
            if let Some(ref prev_key) = node.prev {
                if let Some(prev_node) = self.nodes.get_mut(prev_key) {
                    prev_node.next = node.next.clone();
                }
            } else {
                // This was the head
                self.head = node.next.clone();
            }
            
            // Update next node's prev
            if let Some(ref next_key) = node.next {
                if let Some(next_node) = self.nodes.get_mut(next_key) {
                    next_node.prev = node.prev.clone();
                }
            } else {
                // This was the tail
                self.tail = node.prev.clone();
            }
            
            true
        } else {
            false
        }
    }
    
    fn touch(&mut self, key: &K) {
        if self.remove(key) {
            self.push_front(key.clone());
        }
    }
    
    fn pop_back(&mut self) -> Option<K> {
        if let Some(tail_key) = self.tail.clone() {
            self.remove(&tail_key);
            Some(tail_key)
        } else {
            None
        }
    }
    
    fn iter(&self) -> LruIterator<K> {
        LruIterator {
            nodes: &self.nodes,
            current: self.head.clone(),
        }
    }
}

struct LruIterator<'a, K: Clone + Eq + std::hash::Hash> {
    nodes: &'a HashMap<K, LruNode<K>>,
    current: Option<K>,
}

impl<'a, K: Clone + Eq + std::hash::Hash> Iterator for LruIterator<'a, K> {
    type Item = K;
    
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(key) = self.current.clone() {
            if let Some(node) = self.nodes.get(&key) {
                self.current = node.next.clone();
                Some(key)
            } else {
                None
            }
        } else {
            None
        }
    }
}

// Cache entry with metadata
#[derive(Debug, Clone)]
pub struct CacheEntry<V: Clone> {
    value: V,
    priority: u8,
    ttl: Instant,
    created_at: Instant,
    last_accessed: Instant,
    access_count: usize,
}

// Event system with trait-based approach
pub trait CacheCallback<K>: Send + Sync {
    fn on_event(&self, event: CacheEvent<K>);
}

#[derive(Debug, Clone)]
pub enum CacheEvent<K> {
    Hit(K),
    Miss(K),
    Insert(K),
    Eviction(K),
    TTLExpiry(K),
}

// Main cache implementation
pub struct SmartCache<K, V> 
where
    K: Clone + Eq + std::hash::Hash,
    V: Clone,
{
    data: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    lru_list: Arc<Mutex<LruList<K>>>,
    config: CacheConfig,
    stats: Arc<Mutex<CacheStats>>,
    callbacks: Arc<Mutex<Vec<Box<dyn CacheCallback<K>>>>>,
    cleanup_handle: Option<thread::JoinHandle<()>>,
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

#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub insertions: u64,
    pub ttl_expirations: u64,
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
        let lru_list = Arc::new(Mutex::new(LruList::new()));
        let stats = Arc::new(Mutex::new(CacheStats::default()));
        let callbacks = Arc::new(Mutex::new(Vec::new()));
        
        // Start cleanup thread
        let data_clone = Arc::clone(&data);
        let lru_clone = Arc::clone(&lru_list);
        let stats_clone = Arc::clone(&stats);
        let callbacks_clone = Arc::clone(&callbacks);
        let cleanup_interval = config.cleanup_interval;
        
        let cleanup_handle = thread::spawn(move || {
            loop {
                thread::sleep(cleanup_interval);
                Self::cleanup_expired(&data_clone, &lru_clone, &stats_clone, &callbacks_clone);
            }
        });
        
        Self {
            data,
            lru_list,
            config,
            stats,
            callbacks,
            cleanup_handle: Some(cleanup_handle),
        }
    }
    
    pub fn put(&self, key: K, value: V, ttl: Option<Duration>, priority: u8) -> bool {
        let ttl = ttl.unwrap_or(self.config.default_ttl);
        
        // WARNING: Potential deadlock if locks taken in different order!
        let mut data = self.data.write().unwrap();
        let mut lru_list = self.lru_list.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();
        
        // Check capacity and evict if necessary
        if !data.contains_key(&key) && data.len() >= self.config.max_capacity {
            self.evict_lowest_priority(&mut data, &mut lru_list, &mut stats);
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
        lru_list.touch(&key);
        
        stats.insertions += 1;
        
        // Notify callbacks
        self.notify_callbacks(CacheEvent::Insert(key));
        
        true
    }
    
    pub fn get(&self, key: &K) -> Option<V> {
        let mut data = self.data.write().unwrap();
        
        if let Some(entry) = data.get_mut(key) {
            // Check TTL
            if Instant::now() > entry.ttl {
                data.remove(key);
                self.lru_list.lock().unwrap().remove(key);
                
                let mut stats = self.stats.lock().unwrap();
                stats.ttl_expirations += 1;
                stats.misses += 1;
                
                self.notify_callbacks(CacheEvent::TTLExpiry(key.clone()));
                return None;
            }
            
            // Update access metadata
            entry.last_accessed = Instant::now();
            entry.access_count += 1;
            let value = entry.value.clone();
            
            // Update LRU with O(1) operation
            self.lru_list.lock().unwrap().touch(key);
            
            self.stats.lock().unwrap().hits += 1;
            self.notify_callbacks(CacheEvent::Hit(key.clone()));
            
            Some(value)
        } else {
            self.stats.lock().unwrap().misses += 1;
            self.notify_callbacks(CacheEvent::Miss(key.clone()));
            None
        }
    }
    
    pub fn delete(&self, key: &K) -> bool {
        let mut data = self.data.write().unwrap();
        if data.remove(key).is_some() {
            self.lru_list.lock().unwrap().remove(key);
            true
        } else {
            false
        }
    }
    
    pub fn add_callback<C: CacheCallback<K> + 'static>(&self, callback: Box<C>) {
        self.callbacks.lock().unwrap().push(callback);
    }
    
    fn evict_lowest_priority(
        &self,
        data: &mut HashMap<K, CacheEntry<V>>,
        lru_list: &mut LruList<K>,
        stats: &mut CacheStats,
    ) {
        // Find entry with lowest priority score (age / priority)
        let mut eviction_candidate: Option<(K, f64)> = None;
        
        for key in lru_list.iter() {
            if let Some(entry) = data.get(&key) {
                let age = entry.last_accessed.elapsed().as_secs() as f64;
                let score = age / entry.priority as f64;
                
                match &eviction_candidate {
                    None => eviction_candidate = Some((key.clone(), score)),
                    Some((_, best_score)) if score > *best_score => {
                        eviction_candidate = Some((key.clone(), score));
                    }
                    _ => {}
                }
            }
        }
        
        if let Some((key, _)) = eviction_candidate {
            data.remove(&key);
            lru_list.remove(&key);
            stats.evictions += 1;
            self.notify_callbacks(CacheEvent::Eviction(key));
        }
    }
    
    fn cleanup_expired(
        data: &Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
        lru_list: &Arc<Mutex<LruList<K>>>,
        stats: &Arc<Mutex<CacheStats>>,
        callbacks: &Arc<Mutex<Vec<Box<dyn CacheCallback<K>>>>>,
    ) {
        let mut data = data.write().unwrap();
        let mut lru_list = lru_list.lock().unwrap();
        let now = Instant::now();
        
        let expired_keys: Vec<K> = data
            .iter()
            .filter(|(_, entry)| now > entry.ttl)
            .map(|(key, _)| key.clone())
            .collect();
        
        if !expired_keys.is_empty() {
            let mut stats = stats.lock().unwrap();
            for key in expired_keys {
                data.remove(&key);
                lru_list.remove(&key);
                stats.ttl_expirations += 1;
                
                // Notify callbacks
                let callbacks = callbacks.lock().unwrap();
                for callback in callbacks.iter() {
                    callback.on_event(CacheEvent::TTLExpiry(key.clone()));
                }
            }
        }
    }
    
    fn notify_callbacks(&self, event: CacheEvent<K>) {
        let callbacks = self.callbacks.lock().unwrap();
        for callback in callbacks.iter() {
            callback.on_event(event.clone());
        }
    }
    
    pub fn get_stats(&self) -> CacheStats {
        self.stats.lock().unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lru_list() {
        let mut list = LruList::new();
        
        list.push_front(1);
        list.push_front(2);
        list.push_front(3);
        
        // Should be 3, 2, 1
        let items: Vec<i32> = list.iter().collect();
        assert_eq!(items, vec![3, 2, 1]);
        
        // Touch 1, should move to front
        list.touch(&1);
        let items: Vec<i32> = list.iter().collect();
        assert_eq!(items, vec![1, 3, 2]);
        
        // Pop back should return 2
        assert_eq!(list.pop_back(), Some(2));
    }
    
    #[test]
    fn test_priority_eviction() {
        let cache = SmartCache::new(2);
        
        cache.put(1, "low", None, 1);
        cache.put(2, "high", None, 10);
        cache.put(3, "medium", None, 5); // Should evict 1
        
        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&2), Some("high"));
        assert_eq!(cache.get(&3), Some("medium"));
    }
}
