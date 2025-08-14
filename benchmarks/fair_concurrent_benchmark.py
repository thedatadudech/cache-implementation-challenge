#!/usr/bin/env python3
"""
Fair Concurrent Benchmark with External Workload
Measures actual throughput under realistic conditions
"""

import sys
import time
import threading
import random
import hashlib
import json
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime
from typing import Dict, Tuple
import queue

sys.path.append('../implementations/1_claude_python')
from smart_cache import SmartCache

class FairConcurrentBenchmark:
    """Benchmark that measures actual concurrent throughput"""
    
    def __init__(self, cache_size: int = 100000):
        self.cache_size = cache_size
        
    def benchmark_producer_consumer(self, num_producers: int = 50, num_consumers: int = 50, 
                                   duration_seconds: int = 10) -> Dict:
        """
        Producer-Consumer pattern benchmark
        Producers add items, consumers read items
        Measures throughput and hit rate
        """
        cache = SmartCache(max_size=self.cache_size, default_ttl=3600)
        
        stop_flag = threading.Event()
        producer_counts = [0] * num_producers
        consumer_stats = [[0, 0] for _ in range(num_consumers)]  # [hits, misses]
        
        def producer(producer_id: int):
            count = 0
            while not stop_flag.is_set():
                # Simulate data generation with some CPU work
                key = f"p{producer_id}_item_{count % 1000}"
                value = hashlib.md5(f"{producer_id}_{count}_{time.time()}".encode()).hexdigest()
                
                cache.put(key, value, priority=random.randint(1, 10))
                count += 1
                producer_counts[producer_id] = count
                
                # Small delay to simulate real data generation
                time.sleep(0.0001)
        
        def consumer(consumer_id: int):
            hits = 0
            misses = 0
            while not stop_flag.is_set():
                # Try to read random items
                producer_id = random.randint(0, num_producers - 1)
                item_id = random.randint(0, 999)
                key = f"p{producer_id}_item_{item_id}"
                
                result = cache.get(key)
                if result:
                    # Simulate processing the cached data
                    _ = hashlib.md5(result.encode()).hexdigest()
                    hits += 1
                else:
                    misses += 1
                
                consumer_stats[consumer_id] = [hits, misses]
                
                # Small delay to simulate processing
                time.sleep(0.0001)
        
        print(f"\nRunning Producer-Consumer benchmark ({num_producers} producers, {num_consumers} consumers)...")
        print(f"Duration: {duration_seconds} seconds")
        
        start_time = time.perf_counter()
        
        with ThreadPoolExecutor(max_workers=num_producers + num_consumers) as executor:
            # Start producers
            producer_futures = [executor.submit(producer, i) for i in range(num_producers)]
            
            # Start consumers  
            consumer_futures = [executor.submit(consumer, i) for i in range(num_consumers)]
            
            # Run for specified duration
            time.sleep(duration_seconds)
            stop_flag.set()
            
            # Wait for all to complete
            for f in producer_futures + consumer_futures:
                f.result()
        
        elapsed = time.perf_counter() - start_time
        
        # Calculate statistics
        total_puts = sum(producer_counts)
        total_gets = sum(h + m for h, m in consumer_stats)
        total_hits = sum(h for h, m in consumer_stats)
        total_misses = sum(m for h, m in consumer_stats)
        
        hit_rate = total_hits / total_gets if total_gets > 0 else 0
        
        return {
            'duration': elapsed,
            'total_puts': total_puts,
            'total_gets': total_gets,
            'total_operations': total_puts + total_gets,
            'puts_per_second': f"{total_puts / elapsed:.2f}",
            'gets_per_second': f"{total_gets / elapsed:.2f}",
            'ops_per_second': f"{(total_puts + total_gets) / elapsed:.2f}",
            'hit_rate': f"{hit_rate * 100:.1f}%",
            'total_hits': total_hits,
            'total_misses': total_misses
        }
    
    def benchmark_shared_workload(self, num_workers: int = 100, num_operations: int = 10000) -> Dict:
        """
        Shared workload benchmark
        All workers pull from a common queue of operations
        Ensures everyone does exactly the same amount of work
        """
        cache = SmartCache(max_size=self.cache_size, default_ttl=3600)
        
        # Create work queue
        work_queue = queue.Queue()
        
        # Fill with mixed operations
        for i in range(num_operations):
            if random.random() < 0.7:  # 70% writes
                work_queue.put(('PUT', f'key_{i % 1000}', f'value_{i}', random.randint(1, 10)))
            else:  # 30% reads
                work_queue.put(('GET', f'key_{random.randint(0, 999)}'))
        
        completed_operations = threading.atomic = 0
        operation_times = []
        lock = threading.Lock()
        
        def worker():
            local_times = []
            while True:
                try:
                    task = work_queue.get_nowait()
                except queue.Empty:
                    break
                
                start = time.perf_counter()
                
                if task[0] == 'PUT':
                    cache.put(task[1], task[2], priority=task[3])
                elif task[0] == 'GET':
                    cache.get(task[1])
                
                elapsed = time.perf_counter() - start
                local_times.append(elapsed)
                
                work_queue.task_done()
            
            with lock:
                operation_times.extend(local_times)
        
        print(f"\nRunning Shared Workload benchmark ({num_workers} workers, {num_operations} operations)...")
        
        start_time = time.perf_counter()
        
        with ThreadPoolExecutor(max_workers=num_workers) as executor:
            futures = [executor.submit(worker) for _ in range(num_workers)]
            for f in futures:
                f.result()
        
        elapsed = time.perf_counter() - start_time
        
        # Calculate statistics
        avg_op_time = sum(operation_times) / len(operation_times) if operation_times else 0
        
        return {
            'duration': f"{elapsed:.3f}",
            'num_workers': num_workers,
            'total_operations': num_operations,
            'ops_per_second': f"{num_operations / elapsed:.2f}",
            'avg_operation_time_ms': f"{avg_op_time * 1000:.3f}",
            'parallelism_factor': f"{(avg_op_time * num_operations) / elapsed if elapsed > 0 else 1:.2f}x"
        }
    
    def benchmark_eviction_strategy(self, cache_size: int = 100, total_insertions: int = 200) -> Dict:
        """
        Eviction strategy benchmark
        Tests LRU eviction by filling cache beyond capacity
        """
        cache = SmartCache(max_size=cache_size, default_ttl=3600)
        
        print(f"\nRunning Eviction Strategy benchmark (cache size: {cache_size}, insertions: {total_insertions})...")
        
        start_time = time.perf_counter()
        
        # Fill cache to capacity with varying priorities
        for i in range(cache_size):
            cache.put(f"key_{i}", f"value_{i}", priority=i % 10 + 1)
        
        # Force evictions by adding more items than capacity
        evictions_forced = total_insertions - cache_size
        for i in range(cache_size, total_insertions):
            cache.put(f"key_{i}", f"value_{i}", priority=5)
        
        elapsed = time.perf_counter() - start_time
        
        # Check which original items were evicted
        original_items_remaining = 0
        for i in range(cache_size):
            if cache.get(f"key_{i}") is not None:
                original_items_remaining += 1
        
        evicted_count = cache_size - original_items_remaining
        
        return {
            'duration': f"{elapsed:.3f}",
            'cache_size': cache_size,
            'total_insertions': total_insertions,
            'evictions_forced': evictions_forced,
            'evicted_count': evicted_count,
            'ops_per_second': f"{total_insertions / elapsed:.2f}",
            'eviction_efficiency': f"{(evicted_count / evictions_forced * 100):.1f}%" if evictions_forced > 0 else "N/A"
        }
    
    def benchmark_ttl_operations(self, num_items: int = 100, ttl_ms: int = 100) -> Dict:
        """
        TTL operations benchmark
        Tests TTL expiry and performance of TTL checks
        """
        cache = SmartCache(max_size=10000, default_ttl=3600)
        
        print(f"\nRunning TTL Operations benchmark ({num_items} items with {ttl_ms}ms TTL)...")
        
        # Part 1: TTL Expiry Test
        start_time = time.perf_counter()
        
        # Add items with short TTL
        for i in range(num_items):
            cache.put(f"ttl_key_{i}", f"value_{i}", priority=5, ttl=ttl_ms/1000.0)  # Convert ms to seconds
        
        # Wait for expiration
        time.sleep((ttl_ms + 10) / 1000.0)  # Wait slightly longer than TTL
        
        # Check expired items
        expired_count = 0
        for i in range(num_items):
            if cache.get(f"ttl_key_{i}") is None:
                expired_count += 1
        
        expiry_elapsed = time.perf_counter() - start_time
        
        # Part 2: TTL Check Performance (with valid items)
        cache.clear() if hasattr(cache, 'clear') else None
        
        # Add items with long TTL
        for i in range(num_items):
            cache.put(f"valid_key_{i}", f"value_{i}", priority=5, ttl=3600)  # 1 hour TTL
        
        # Measure time to check all items
        check_start = time.perf_counter()
        valid_count = 0
        for i in range(num_items):
            if cache.get(f"valid_key_{i}") is not None:
                valid_count += 1
        check_elapsed = time.perf_counter() - check_start
        
        return {
            'ttl_expiry_duration': f"{expiry_elapsed:.3f}",
            'ttl_check_duration': f"{check_elapsed:.3f}",
            'num_items': num_items,
            'ttl_ms': ttl_ms,
            'expired_count': expired_count,
            'expiry_rate': f"{(expired_count / num_items * 100):.1f}%" if num_items > 0 else "N/A",
            'valid_count': valid_count,
            'check_ops_per_second': f"{num_items / check_elapsed:.2f}" if check_elapsed > 0 else "N/A",
            'avg_check_time_us': f"{(check_elapsed * 1_000_000 / num_items):.2f}" if num_items > 0 else "N/A"
        }
    
    def benchmark_io_simulation(self, num_workers: int = 100, duration_seconds: int = 10) -> Dict:
        """
        I/O-bound simulation benchmark
        Simulates network/database delays where threading actually helps
        """
        cache = SmartCache(max_size=self.cache_size, default_ttl=3600)
        
        stop_flag = threading.Event()
        operation_counts = [0] * num_workers
        
        def worker(worker_id: int):
            count = 0
            while not stop_flag.is_set():
                # Simulate fetching from database (I/O wait)
                time.sleep(0.005)  # 5ms "database query"
                
                key = f"worker_{worker_id}_item_{count % 100}"
                value = f"data_{count}_{time.time()}"
                
                # Cache the result
                cache.put(key, value, priority=5)
                
                # Try to read some other worker's data
                other_worker = (worker_id + random.randint(1, num_workers - 1)) % num_workers
                other_key = f"worker_{other_worker}_item_{random.randint(0, 99)}"
                cached = cache.get(other_key)
                
                if cached:
                    # Simulate processing cached data
                    time.sleep(0.001)  # 1ms processing
                
                count += 2  # PUT + GET
                operation_counts[worker_id] = count
        
        print(f"\nRunning I/O Simulation benchmark ({num_workers} workers)...")
        print("Simulating database/network delays where threading helps...")
        
        start_time = time.perf_counter()
        
        with ThreadPoolExecutor(max_workers=num_workers) as executor:
            futures = [executor.submit(worker, i) for i in range(num_workers)]
            
            time.sleep(duration_seconds)
            stop_flag.set()
            
            for f in futures:
                f.result()
        
        elapsed = time.perf_counter() - start_time
        
        total_operations = sum(operation_counts)
        
        return {
            'duration': f"{elapsed:.2f}",
            'num_workers': num_workers,
            'total_operations': total_operations,
            'ops_per_second': f"{total_operations / elapsed:.2f}",
            'ops_per_worker': total_operations / num_workers,
            'theoretical_sequential_time': f"{total_operations * 0.006:.2f}",  # 6ms per op
            'speedup': f"{(total_operations * 0.006) / elapsed:.2f}x"
        }

def main():
    print("=" * 60)
    print("Fair Concurrent Benchmark Suite")
    print("Python Implementation with GIL considerations")
    print("=" * 60)
    
    benchmark = FairConcurrentBenchmark()
    results = {}
    
    # Test 1: Producer-Consumer Pattern
    print("\n1. Producer-Consumer Pattern")
    print("-" * 40)
    pc_result = benchmark.benchmark_producer_consumer(
        num_producers=50, 
        num_consumers=50, 
        duration_seconds=5
    )
    results['producer_consumer'] = pc_result
    
    print(f"\nResults:")
    print(f"  Total Operations: {pc_result['total_operations']:,}")
    print(f"  Throughput: {pc_result['ops_per_second']} ops/sec")
    print(f"  Cache Hit Rate: {pc_result['hit_rate']}")
    print(f"  PUT throughput: {pc_result['puts_per_second']} ops/sec")
    print(f"  GET throughput: {pc_result['gets_per_second']} ops/sec")
    
    # Test 2: Shared Workload
    print("\n2. Shared Workload (Fair Comparison)")
    print("-" * 40)
    sw_result = benchmark.benchmark_shared_workload(
        num_workers=100,
        num_operations=10000
    )
    results['shared_workload'] = sw_result
    
    print(f"\nResults:")
    print(f"  Total Operations: {sw_result['total_operations']:,}")
    print(f"  Duration: {sw_result['duration']} seconds")
    print(f"  Throughput: {sw_result['ops_per_second']} ops/sec")
    print(f"  Avg Operation Time: {sw_result['avg_operation_time_ms']} ms")
    print(f"  Parallelism Factor: {sw_result['parallelism_factor']}")
    
    # Test 3: I/O Simulation
    print("\n3. I/O-Bound Simulation")
    print("-" * 40)
    io_result = benchmark.benchmark_io_simulation(
        num_workers=100,
        duration_seconds=5
    )
    results['io_simulation'] = io_result
    
    print(f"\nResults:")
    print(f"  Total Operations: {io_result['total_operations']:,}")
    print(f"  Throughput: {io_result['ops_per_second']} ops/sec")
    print(f"  Theoretical Sequential Time: {io_result['theoretical_sequential_time']} seconds")
    print(f"  Actual Time: {io_result['duration']} seconds")
    print(f"  Speedup from Threading: {io_result['speedup']}")
    
    # Test 4: Eviction Strategy
    print("\n4. Eviction Strategy")
    print("-" * 40)
    evict_result = benchmark.benchmark_eviction_strategy(
        cache_size=100,
        total_insertions=200
    )
    results['eviction'] = evict_result
    
    print(f"\nResults:")
    print(f"  Cache Size: {evict_result['cache_size']}")
    print(f"  Total Insertions: {evict_result['total_insertions']}")
    print(f"  Evicted Count: {evict_result['evicted_count']}")
    print(f"  Throughput: {evict_result['ops_per_second']} ops/sec")
    print(f"  Eviction Efficiency: {evict_result['eviction_efficiency']}")
    
    # Test 5: TTL Operations
    print("\n5. TTL Operations")
    print("-" * 40)
    ttl_result = benchmark.benchmark_ttl_operations(
        num_items=100,
        ttl_ms=100
    )
    results['ttl'] = ttl_result
    
    print(f"\nResults:")
    print(f"  Items with TTL: {ttl_result['num_items']}")
    print(f"  TTL Duration: {ttl_result['ttl_ms']}ms")
    print(f"  Expired Count: {ttl_result['expired_count']}")
    print(f"  Expiry Rate: {ttl_result['expiry_rate']}")
    print(f"  Check Performance: {ttl_result['check_ops_per_second']} ops/sec")
    print(f"  Avg Check Time: {ttl_result['avg_check_time_us']} μs")
    
    # Save results
    timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
    output = {
        'implementation': 'Python (Fair Concurrent)',
        'timestamp': timestamp,
        'cache_size': 100000,
        'benchmarks': results,
        'notes': {
            'gil_impact': 'Python GIL limits true CPU parallelism',
            'io_benefit': 'Threading helps with I/O-bound operations',
            'comparison': 'These metrics are comparable across languages'
        }
    }
    
    filename = f'results/python_fair_concurrent_{timestamp}.json'
    with open(filename, 'w') as f:
        json.dump(output, f, indent=2)
    
    print(f"\n" + "=" * 60)
    print(f"Results saved to: {filename}")
    print("=" * 60)
    
    return results

if __name__ == "__main__":
    main()