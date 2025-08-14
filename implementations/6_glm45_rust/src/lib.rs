// GLM-4.5 Rust Implementation - Score: 89/100
// Focus on observability, debugging, and SQL-like queries

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock, Mutex};
use std::time::{Duration, Instant};
use std::thread;
use arc_swap::ArcSwap;
use serde::{Serialize, Deserialize};

// ===== Configuration with Hot Reload =====
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub max_capacity: usize,
    pub default_ttl: Duration,
    pub cleanup_interval: Duration,
    pub enable_trace_log: bool,
    pub trace_log_capacity: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_capacity: 1000,
            default_ttl: Duration::from_secs(3600),
            cleanup_interval: Duration::from_secs(60),
            enable_trace_log: true,
            trace_log_capacity: 10000,
        }
    }
}

// ===== Cache Entry =====
#[derive(Debug, Clone)]
pub struct CacheEntry<V: Clone> {
    value: V,
    priority: u8,
    ttl: Instant,
    created_at: Instant,
    last_accessed: Instant,
    access_count: usize,
}

// ===== Operation Tracing for Debugging =====
#[derive(Debug, Clone, Serialize)]
pub enum CacheOperation {
    Put { key: String, priority: u8, ttl_secs: u64 },
    Get { key: String, hit: bool },
    Delete { key: String },
    Eviction { key: String, reason: EvictionReason },
}

#[derive(Debug, Clone, Serialize)]
pub enum EvictionReason {
    CapacityExceeded { victim_priority: u8 },
    TTLExpired,
    LowPriority { score: f64 },
}

// ===== Eviction Explanation =====
#[derive(Debug, Serialize)]
pub struct EvictionExplanation {
    pub key: String,
    pub would_be_evicted: bool,
    pub reason: String,
    pub priority_score: f64,
    pub lru_position: usize,
    pub ttl_remaining_secs: i64,
}

// ===== Query Engine for SQL-like Interface =====
#[derive(Debug, Serialize)]
pub enum QueryResult {
    Entries(Vec<QueryEntry>),
    Count(usize),
    Stats(HashMap<String, f64>),
}

#[derive(Debug, Serialize)]
pub struct QueryEntry {
    pub key: String,
    pub priority: u8,
    pub access_count: usize,
    pub age_secs: u64,
    pub ttl_remaining_secs: i64,
}

// ===== Circular Buffer for Trace Log =====
pub struct CircularBuffer<T> {
    buffer: Vec<Option<T>>,
    head: usize,
    tail: usize,
    capacity: usize,
}

impl<T: Clone> CircularBuffer<T> {
    fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![None; capacity],
            head: 0,
            tail: 0,
            capacity,
        }
    }
    
    fn push(&mut self, item: T) {
        self.buffer[self.tail] = Some(item);
        self.tail = (self.tail + 1) % self.capacity;
        if self.tail == self.head {
            self.head = (self.head + 1) % self.capacity;
        }
    }
    
    fn to_vec(&self) -> Vec<T> {
        let mut result = Vec::new();
        let mut idx = self.head;
        while idx != self.tail {
            if let Some(ref item) = self.buffer[idx] {
                result.push(item.clone());
            }
            idx = (idx + 1) % self.capacity;
        }
        result
    }
}

// ===== Main Cache Implementation =====
pub struct SmartCache<K, V> 
where
    K: Clone + Eq + std::hash::Hash + ToString,
    V: Clone,
{
    data: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    lru_queue: Arc<Mutex<VecDeque<K>>>,
    
    // Configuration with hot reload
    config: Arc<ArcSwap<CacheConfig>>,
    
    // Advanced debugging features
    trace_log: Arc<Mutex<CircularBuffer<CacheOperation>>>,
    
    // Statistics
    stats: Arc<RwLock<CacheStats>>,
}

#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub insertions: u64,
}

impl<K, V> SmartCache<K, V>
where
    K: Clone + Eq + std::hash::Hash + ToString + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub fn new(max_capacity: usize) -> Self {
        let config = CacheConfig {
            max_capacity,
            ..Default::default()
        };
        
        let trace_log = Arc::new(Mutex::new(CircularBuffer::new(config.trace_log_capacity)));
        let config = Arc::new(ArcSwap::from_pointee(config));
        
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            lru_queue: Arc::new(Mutex::new(VecDeque::new())),
            config,
            trace_log,
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }
    
    pub fn put(&self, key: K, value: V, ttl: Option<Duration>, priority: u8) -> bool {
        let config = self.config.load();
        let ttl = ttl.unwrap_or(config.default_ttl);
        
        // Log operation
        if config.enable_trace_log {
            self.trace_log.lock().unwrap().push(CacheOperation::Put {
                key: key.to_string(),
                priority,
                ttl_secs: ttl.as_secs(),
            });
        }
        
        let mut data = self.data.write().unwrap();
        let mut lru_queue = self.lru_queue.lock().unwrap();
        
        // Check capacity
        if !data.contains_key(&key) && data.len() >= config.max_capacity {
            self.evict_with_explanation(&mut data, &mut lru_queue);
        }
        
        let entry = CacheEntry {
            value,
            priority: priority.min(10).max(1),
            ttl: Instant::now() + ttl,
            created_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 0,
        };
        
        data.insert(key.clone(), entry);
        lru_queue.retain(|k| k != &key);
        lru_queue.push_back(key);
        
        self.stats.write().unwrap().insertions += 1;
        true
    }
    
    pub fn get(&self, key: &K) -> Option<V> {
        let mut data = self.data.write().unwrap();
        
        if let Some(entry) = data.get_mut(key) {
            if Instant::now() > entry.ttl {
                // Log operation
                let config = self.config.load();
                if config.enable_trace_log {
                    self.trace_log.lock().unwrap().push(CacheOperation::Get {
                        key: key.to_string(),
                        hit: false,
                    });
                }
                
                data.remove(key);
                self.lru_queue.lock().unwrap().retain(|k| k != key);
                self.stats.write().unwrap().misses += 1;
                return None;
            }
            
            entry.last_accessed = Instant::now();
            entry.access_count += 1;
            let value = entry.value.clone();
            
            // Update LRU
            let mut lru_queue = self.lru_queue.lock().unwrap();
            lru_queue.retain(|k| k != key);
            lru_queue.push_back(key.clone());
            
            // Log operation
            let config = self.config.load();
            if config.enable_trace_log {
                self.trace_log.lock().unwrap().push(CacheOperation::Get {
                    key: key.to_string(),
                    hit: true,
                });
            }
            
            self.stats.write().unwrap().hits += 1;
            Some(value)
        } else {
            // Log operation
            let config = self.config.load();
            if config.enable_trace_log {
                self.trace_log.lock().unwrap().push(CacheOperation::Get {
                    key: key.to_string(),
                    hit: false,
                });
            }
            
            self.stats.write().unwrap().misses += 1;
            None
        }
    }
    
    // ===== SQL-like Query Interface =====
    pub fn query(&self, sql: &str) -> QueryResult {
        let data = self.data.read().unwrap();
        
        if sql.starts_with("SELECT * FROM cache WHERE priority >") {
            let priority_threshold: u8 = sql
                .split('>')
                .last()
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(5);
            
            let entries: Vec<QueryEntry> = data
                .iter()
                .filter(|(_, entry)| entry.priority > priority_threshold)
                .map(|(key, entry)| QueryEntry {
                    key: key.to_string(),
                    priority: entry.priority,
                    access_count: entry.access_count,
                    age_secs: entry.created_at.elapsed().as_secs(),
                    ttl_remaining_secs: entry.ttl.duration_since(Instant::now())
                        .map(|d| d.as_secs() as i64)
                        .unwrap_or(-1),
                })
                .collect();
            
            QueryResult::Entries(entries)
        } else if sql.starts_with("SELECT COUNT(*) FROM cache") {
            QueryResult::Count(data.len())
        } else {
            let stats = self.stats.read().unwrap();
            let mut stats_map = HashMap::new();
            stats_map.insert("hits".to_string(), stats.hits as f64);
            stats_map.insert("misses".to_string(), stats.misses as f64);
            QueryResult::Stats(stats_map)
        }
    }
    
    // ===== Eviction Explanation =====
    pub fn explain_eviction(&self, key: &K) -> EvictionExplanation {
        let data = self.data.read().unwrap();
        let lru_queue = self.lru_queue.lock().unwrap();
        
        let mut explanation = EvictionExplanation {
            key: key.to_string(),
            would_be_evicted: false,
            reason: String::new(),
            priority_score: 0.0,
            lru_position: 0,
            ttl_remaining_secs: 0,
        };
        
        if let Some(entry) = data.get(key) {
            let age_secs = entry.last_accessed.elapsed().as_secs() as f64;
            explanation.priority_score = age_secs / entry.priority as f64;
            
            explanation.lru_position = lru_queue
                .iter()
                .position(|k| k == key)
                .unwrap_or(usize::MAX);
            
            explanation.ttl_remaining_secs = entry.ttl
                .duration_since(Instant::now())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(-1);
            
            if explanation.ttl_remaining_secs < 0 {
                explanation.would_be_evicted = true;
                explanation.reason = "TTL expired".to_string();
            } else {
                explanation.reason = format!(
                    "Safe from eviction (score: {:.2}, position: {})",
                    explanation.priority_score,
                    explanation.lru_position
                );
            }
        } else {
            explanation.reason = "Key not found in cache".to_string();
        }
        
        explanation
    }
    
    // ===== Operation Replay for Debugging =====
    pub fn get_trace_log(&self) -> Vec<CacheOperation> {
        self.trace_log.lock().unwrap().to_vec()
    }
    
    // ===== Hot Configuration Reload =====
    pub fn reload_config(&self, new_config: CacheConfig) {
        self.config.store(Arc::new(new_config));
    }
    
    pub fn get_stats(&self) -> CacheStats {
        self.stats.read().unwrap().clone()
    }
    
    fn evict_with_explanation(
        &self,
        data: &mut HashMap<K, CacheEntry<V>>,
        lru_queue: &mut VecDeque<K>,
    ) {
        let mut eviction_candidate: Option<(K, f64)> = None;
        
        for key in lru_queue.iter() {
            if let Some(entry) = data.get(key) {
                let age = entry.last_accessed.elapsed().as_secs() as f64;
                let score = age / entry.priority as f64;
                
                match &eviction_candidate {
                    None => eviction_candidate = Some((key.clone(), score)),
                    Some((_, current_score)) if score > *current_score => {
                        eviction_candidate = Some((key.clone(), score));
                    }
                    _ => {}
                }
            }
        }
        
        if let Some((key, score)) = eviction_candidate {
            data.remove(&key);
            lru_queue.retain(|k| k != &key);
            
            let config = self.config.load();
            if config.enable_trace_log {
                self.trace_log.lock().unwrap().push(CacheOperation::Eviction {
                    key: key.to_string(),
                    reason: EvictionReason::LowPriority { score },
                });
            }
            
            self.stats.write().unwrap().evictions += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sql_query() {
        let cache = SmartCache::new(10);
        
        cache.put(1, "high", None, 8);
        cache.put(2, "medium", None, 5);
        cache.put(3, "low", None, 2);
        
        match cache.query("SELECT * FROM cache WHERE priority > 4") {
            QueryResult::Entries(entries) => {
                assert_eq!(entries.len(), 2);
            }
            _ => panic!("Expected entries"),
        }
    }
    
    #[test]
    fn test_eviction_explanation() {
        let cache = SmartCache::new(2);
        
        cache.put(1, "first", None, 5);
        cache.put(2, "second", None, 10);
        
        let explanation = cache.explain_eviction(&1);
        assert!(!explanation.would_be_evicted);
    }
    
    #[test]
    fn test_hot_reload() {
        let cache = SmartCache::new(100);
        
        let mut new_config = CacheConfig::default();
        new_config.max_capacity = 500;
        cache.reload_config(new_config);
        
        assert_eq!(cache.config.load().max_capacity, 500);
    }
}
