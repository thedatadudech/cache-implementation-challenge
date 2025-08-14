"""
Claude's Python Implementation - Score: 78/100
Smart Cache with LRU eviction, TTL support, and priority levels
"""

import threading
import time
from collections import OrderedDict
from typing import Any, Optional, Dict, List, Callable
from dataclasses import dataclass
from datetime import datetime, timedelta
import logging

@dataclass
class CacheEntry:
    value: Any
    ttl: float
    priority: int
    created_at: float
    last_accessed: float
    access_count: int

@dataclass
class CacheStats:
    hits: int = 0
    misses: int = 0
    evictions: int = 0
    insertions: int = 0
    
    @property
    def hit_rate(self) -> float:
        total = self.hits + self.misses
        return self.hits / total if total > 0 else 0.0

class SmartCache:
    def __init__(self, max_size: int, default_ttl: int = 3600):
        self._cache = OrderedDict()
        self._metadata: Dict[str, CacheEntry] = {}
        self._max_size = max_size
        self._default_ttl = default_ttl
        self._lock = threading.RLock()
        self._stats = CacheStats()
        self._event_listeners: List[Callable] = []
        self._cleanup_stop_event = threading.Event()
        
        # Start cleanup thread
        self._cleanup_thread = threading.Thread(target=self._cleanup_worker, daemon=True)
        self._cleanup_thread.start()
    
    def put(self, key: str, value: Any, ttl: Optional[int] = None, priority: int = 1) -> bool:
        with self._lock:
            current_time = time.time()
            ttl = ttl or self._default_ttl
            
            # Evict if necessary
            if key not in self._cache and len(self._cache) >= self._max_size:
                self._evict_lowest_priority()
            
            # Store entry
            self._cache[key] = value
            self._metadata[key] = CacheEntry(
                value=value,
                ttl=current_time + ttl,
                priority=max(1, min(10, priority)),
                created_at=current_time,
                last_accessed=current_time,
                access_count=0
            )
            
            # Move to end (most recently used)
            self._cache.move_to_end(key)
            
            self._stats.insertions += 1
            self._notify_listeners("insert", key, value)
            return True
    
    def get(self, key: str) -> Optional[Any]:
        with self._lock:
            if key not in self._cache:
                self._stats.misses += 1
                self._notify_listeners("miss", key)
                return None
            
            entry = self._metadata[key]
            
            # Check TTL
            if time.time() > entry.ttl:
                self._remove_entry(key)
                self._stats.misses += 1
                self._notify_listeners("miss", key)
                return None
            
            # Update metadata
            entry.last_accessed = time.time()
            entry.access_count += 1
            
            # Move to end (most recently used)
            self._cache.move_to_end(key)
            
            self._stats.hits += 1
            self._notify_listeners("hit", key, entry.value)
            return entry.value
    
    def delete(self, key: str) -> bool:
        with self._lock:
            if key in self._cache:
                self._remove_entry(key)
                return True
            return False
    
    def clear(self):
        with self._lock:
            self._cache.clear()
            self._metadata.clear()
    
    def get_stats(self) -> Dict[str, Any]:
        with self._lock:
            return {
                'hits': self._stats.hits,
                'misses': self._stats.misses,
                'hit_rate': self._stats.hit_rate,
                'evictions': self._stats.evictions,
                'insertions': self._stats.insertions,
                'size': len(self._cache),
                'capacity': self._max_size
            }
    
    def get_cache_info(self) -> Dict[str, Any]:
        with self._lock:
            items_info = []
            for key, entry in self._metadata.items():
                items_info.append({
                    'key': key,
                    'priority': entry.priority,
                    'ttl_remaining': max(0, entry.ttl - time.time()),
                    'access_count': entry.access_count
                })
            
            return {
                'items': items_info,
                'stats': self.get_stats()
            }
    
    def add_event_listener(self, callback: Callable):
        self._event_listeners.append(callback)
    
    def _evict_lowest_priority(self):
        if not self._cache:
            return
        
        # Find entry with lowest priority (and oldest if tie)
        min_priority = 11
        evict_key = None
        
        for key, entry in self._metadata.items():
            if entry.priority < min_priority:
                min_priority = entry.priority
                evict_key = key
            elif entry.priority == min_priority and evict_key:
                # Tie-breaker: use LRU
                if list(self._cache.keys()).index(key) < list(self._cache.keys()).index(evict_key):
                    evict_key = key
        
        if evict_key:
            self._remove_entry(evict_key)
            self._stats.evictions += 1
            self._notify_listeners("eviction", evict_key)
    
    def _remove_entry(self, key: str):
        if key in self._cache:
            del self._cache[key]
            del self._metadata[key]
    
    def _cleanup_worker(self):
        while not self._cleanup_stop_event.wait(10):
            self._cleanup_expired()
    
    def _cleanup_expired(self):
        with self._lock:
            current_time = time.time()
            expired_keys = [
                key for key, entry in self._metadata.items()
                if current_time > entry.ttl
            ]
            for key in expired_keys:
                self._remove_entry(key)
    
    def _notify_listeners(self, event_type: str, key: str, value: Any = None):
        for listener in self._event_listeners:
            try:
                listener(event_type, key, value)
            except Exception as e:
                logging.error(f"Event listener error: {e}")
    
    def __len__(self) -> int:
        """Return number of items in cache"""
        return len(self._cache)
    
    def __contains__(self, key: str) -> bool:
        """Check if key exists and is not expired"""
        return self.get(key) is not None
    
    def __del__(self):
        """Cleanup on destruction"""
        if hasattr(self, '_cleanup_stop_event'):
            self._cleanup_stop_event.set()


# Example usage and testing
if __name__ == "__main__":
    import time
    
    # Create cache instance
    cache = SmartCache(max_size=3, default_ttl=60)
    
    # Add event listener for monitoring
    def cache_monitor(event_type: str, key: str, value: Any = None):
        print(f"Cache Event: {event_type} - Key: {key}")
    
    cache.add_event_listener(cache_monitor)
    
    # Test basic operations
    print("=== Basic Operations ===")
    cache.put("user:1", {"name": "Alice", "age": 30})
    cache.put("user:2", {"name": "Bob", "age": 25}, priority=5)
    cache.put("temp", "temporary data", ttl=2)
    
    print(f"Get user:1: {cache.get('user:1')}")
    print(f"Get user:2: {cache.get('user:2')}")
    print(f"Cache stats: {cache.get_stats()}")
    
    # Test capacity limit
    print("\n=== Capacity Testing ===")
    cache.put("user:3", {"name": "Charlie", "age": 35}, priority=1)
    cache.put("user:4", {"name": "Diana", "age": 28}, priority=10)  # Should evict lowest priority
    
    print(f"Cache info: {cache.get_cache_info()}")
    
    # Test TTL expiration
    print("\n=== TTL Testing ===")
    print(f"Temp data before expiry: {cache.get('temp')}")
    time.sleep(3)
    print(f"Temp data after expiry: {cache.get('temp')}")
    
    print(f"\nFinal stats: {cache.get_stats()}")
