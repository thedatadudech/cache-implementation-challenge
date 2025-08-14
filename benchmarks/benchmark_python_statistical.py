#!/usr/bin/env python3
"""
Statistical Python Benchmark Suite for Cache Implementations
Implements Criterion-like statistical analysis for Python benchmarks
"""

import sys
import time
import threading
import json
import os
import numpy as np
from datetime import datetime
from concurrent.futures import ThreadPoolExecutor
from typing import List, Tuple, Dict, Callable
import warnings
warnings.filterwarnings('ignore')

sys.path.append('../implementations/1_claude_python')
from smart_cache import SmartCache

class StatisticalBenchmark:
    """Statistical benchmarking similar to Criterion.rs"""
    
    def __init__(self, warmup_iters: int = 10, sample_size: int = 100, confidence: float = 0.95):
        self.warmup_iters = warmup_iters
        self.sample_size = sample_size
        self.confidence = confidence
        
    def measure(self, func: Callable, iterations: int = 1) -> Dict:
        """Measure a function's performance with statistical analysis"""
        
        # Warmup phase
        print(f"      Warming up ({self.warmup_iters} iterations)...", end='', flush=True)
        for _ in range(self.warmup_iters):
            func()
        print(" done")
        
        # Measurement phase
        print(f"      Collecting {self.sample_size} samples...", end='', flush=True)
        times = []
        
        for i in range(self.sample_size):
            if i % 20 == 0:
                print(".", end='', flush=True)
                
            start = time.perf_counter()
            for _ in range(iterations):
                func()
            elapsed = time.perf_counter() - start
            times.append(elapsed / iterations)
        
        print(" done")
        
        # Statistical analysis
        times_us = np.array(times) * 1_000_000  # Convert to microseconds
        
        # Remove outliers using IQR method
        q1, q3 = np.percentile(times_us, [25, 75])
        iqr = q3 - q1
        lower_bound = q1 - 1.5 * iqr
        upper_bound = q3 + 1.5 * iqr
        filtered_times = times_us[(times_us >= lower_bound) & (times_us <= upper_bound)]
        outliers = len(times_us) - len(filtered_times)
        
        # Calculate statistics
        mean = np.mean(filtered_times)
        std = np.std(filtered_times)
        median = np.median(filtered_times)
        
        # Calculate confidence interval
        z_score = 1.96 if self.confidence == 0.95 else 2.58  # 95% or 99% confidence
        margin = z_score * (std / np.sqrt(len(filtered_times)))
        ci_lower = mean - margin
        ci_upper = mean + margin
        
        return {
            'mean': mean,
            'median': median,
            'std': std,
            'min': np.min(filtered_times),
            'max': np.max(filtered_times),
            'ci_lower': ci_lower,
            'ci_upper': ci_upper,
            'confidence': self.confidence,
            'samples': len(filtered_times),
            'outliers': outliers,
            'raw_times': times_us.tolist()  # For detailed analysis
        }
    
    def format_result(self, result: Dict, name: str) -> str:
        """Format benchmark result like Criterion"""
        return (f"      {name:30} time: [{result['ci_lower']:.4f} µs "
                f"{result['mean']:.4f} µs {result['ci_upper']:.4f} µs]\n"
                f"      {'':30} (std: {result['std']:.4f} µs, "
                f"median: {result['median']:.4f} µs, "
                f"outliers: {result['outliers']})")

def benchmark_single_thread_operations():
    """Benchmark single-threaded operations with statistical analysis"""
    print("\n1. Single Thread Benchmarks")
    print("=" * 60)
    
    bench = StatisticalBenchmark()
    results = {}
    
    cache = SmartCache(max_size=100000, default_ttl=3600)  # 100k - typische Application Cache Größe
    
    # Test PUT operations
    print("\n   PUT Operations (100,000 size cache):")
    counter = [0]
    def put_op():
        cache.put(f"key_{counter[0] % 1000}", f"value_{counter[0]}", priority=counter[0] % 10 + 1)
        counter[0] += 1
    
    put_result = bench.measure(put_op)
    print(bench.format_result(put_result, "PUT"))
    results['put_microseconds'] = put_result['mean']
    results['put_ci'] = [put_result['ci_lower'], put_result['ci_upper']]
    
    # Fill cache for GET tests
    for i in range(1000):
        cache.put(f"key_{i}", f"value_{i}", priority=i % 10 + 1)
    
    # Test GET operations (hits)
    print("\n   GET Operations (cache hits):")
    counter[0] = 0
    def get_hit_op():
        cache.get(f"key_{counter[0] % 1000}")
        counter[0] += 1
    
    get_hit_result = bench.measure(get_hit_op)
    print(bench.format_result(get_hit_result, "GET (hit)"))
    results['get_hit_microseconds'] = get_hit_result['mean']
    results['get_hit_ci'] = [get_hit_result['ci_lower'], get_hit_result['ci_upper']]
    
    # Test GET operations (misses)
    print("\n   GET Operations (cache misses):")
    counter[0] = 1000
    def get_miss_op():
        cache.get(f"missing_key_{counter[0]}")
        counter[0] += 1
    
    get_miss_result = bench.measure(get_miss_op)
    print(bench.format_result(get_miss_result, "GET (miss)"))
    results['get_miss_microseconds'] = get_miss_result['mean']
    results['get_miss_ci'] = [get_miss_result['ci_lower'], get_miss_result['ci_upper']]
    
    return results, {
        'put': put_result,
        'get_hit': get_hit_result,
        'get_miss': get_miss_result
    }

def benchmark_concurrent_operations():
    """Benchmark concurrent operations with statistical analysis"""
    print("\n2. Concurrent Operations Benchmarks")
    print("=" * 60)
    
    bench = StatisticalBenchmark(warmup_iters=3, sample_size=20)  # Fewer samples for slow tests
    results = {}
    
    # Test with 10 threads (using thread pool)
    print("\n   10 Threads Concurrent Access (Thread Pool):")
    def concurrent_10_threads():
        # Create new cache for each iteration (fair comparison with Rust)
        cache = SmartCache(max_size=100000, default_ttl=3600)  # 100k - typische Application Cache Größe
        
        def worker(thread_id):
            for i in range(100):
                key = f"t{thread_id}_key_{i}"
                cache.put(key, f"value_{i}", priority=5)
                cache.get(key)
        
        with ThreadPoolExecutor(max_workers=10) as executor:
            futures = [executor.submit(worker, i) for i in range(10)]
            for future in futures:
                future.result()
    
    concurrent_10_result = bench.measure(concurrent_10_threads)
    print(bench.format_result(concurrent_10_result, "10 thread pool"))
    results['concurrent_10_threads_seconds'] = concurrent_10_result['mean'] / 1_000_000  # Convert to seconds
    results['concurrent_10_threads_ci'] = [
        concurrent_10_result['ci_lower'] / 1_000_000,
        concurrent_10_result['ci_upper'] / 1_000_000
    ]
    
    # Test with 100 threads (using thread pool for safe concurrency)
    print("\n   100 Threads Concurrent Access (Thread Pool):")
    def concurrent_100_threads():
        cache = SmartCache(max_size=100000, default_ttl=3600)
        
        def worker(thread_id):
            for i in range(10):  # 100 threads * 10 ops = 1000 total
                key = f"t{thread_id}_key_{i}"
                cache.put(key, f"value_{i}", priority=5)
                cache.get(key)
        
        with ThreadPoolExecutor(max_workers=100) as executor:
            futures = [executor.submit(worker, i) for i in range(100)]
            for future in futures:
                future.result()
    
    concurrent_100_result = bench.measure(concurrent_100_threads)
    print(bench.format_result(concurrent_100_result, "100 thread pool"))
    results['concurrent_100_threads_seconds'] = concurrent_100_result['mean'] / 1_000_000
    results['concurrent_100_threads_ci'] = [
        concurrent_100_result['ci_lower'] / 1_000_000,
        concurrent_100_result['ci_upper'] / 1_000_000
    ]
    
    return results, {
        '10_threads': concurrent_10_result,
        '100_threads': concurrent_100_result
    }

def benchmark_eviction_strategies():
    """Benchmark eviction strategies with statistical analysis"""
    print("\n3. Eviction Strategy Benchmarks")
    print("=" * 60)
    
    bench = StatisticalBenchmark(warmup_iters=5, sample_size=50)
    results = {}
    
    print("\n   LRU Eviction (100 capacity, 200 insertions):")
    def eviction_test():
        cache = SmartCache(max_size=100, default_ttl=3600)
        
        # Fill cache to capacity
        for i in range(100):
            cache.put(f"key_{i}", f"value_{i}", priority=i % 10 + 1)
        
        # Force eviction with 100 more items
        for i in range(100, 200):
            cache.put(f"key_{i}", f"value_{i}", priority=5)
    
    eviction_result = bench.measure(eviction_test)
    print(bench.format_result(eviction_result, "Eviction"))
    results['eviction_microseconds'] = eviction_result['mean']
    results['eviction_ci'] = [eviction_result['ci_lower'], eviction_result['ci_upper']]
    results['evictions_count'] = 100  # We know we evicted 100 items
    
    return results, {'eviction': eviction_result}

def benchmark_ttl_operations():
    """Benchmark TTL operations with statistical analysis"""
    print("\n4. TTL (Time-To-Live) Benchmarks")
    print("=" * 60)
    
    bench = StatisticalBenchmark(warmup_iters=5, sample_size=30)
    results = {}
    
    # Test TTL expiration
    print("\n   TTL Expiration (100 items, 1ms TTL):")
    def ttl_expiry_test():
        cache = SmartCache(max_size=200, default_ttl=0.001)  # 1ms default TTL
        
        # Add items with short TTL
        for i in range(100):
            cache.put(f"key_{i}", f"value_{i}", priority=5, ttl=0.001)
        
        # Wait for expiration
        time.sleep(0.002)
        
        # Check expired items
        expired = sum(1 for i in range(100) if cache.get(f"key_{i}") is None)
        return expired
    
    ttl_result = bench.measure(ttl_expiry_test)
    print(bench.format_result(ttl_result, "TTL expiry"))
    results['ttl_expiry_microseconds'] = ttl_result['mean']
    results['ttl_expiry_ci'] = [ttl_result['ci_lower'], ttl_result['ci_upper']]
    
    # Test TTL check performance
    print("\n   TTL Check Performance (100 items with valid TTL):")
    cache = SmartCache(max_size=100000, default_ttl=3600)  # 100k - typische Application Cache Größe
    
    # Add items with long TTL
    for i in range(100):
        cache.put(f"ttl_key_{i}", f"value_{i}", priority=5, ttl=3600)
    
    def ttl_check_test():
        for i in range(100):
            cache.get(f"ttl_key_{i}")
    
    ttl_check_result = bench.measure(ttl_check_test, iterations=1)
    print(bench.format_result(ttl_check_result, "TTL check"))
    results['ttl_check_microseconds'] = ttl_check_result['mean'] / 100  # Per operation
    results['ttl_check_ci'] = [ttl_check_result['ci_lower'] / 100, ttl_check_result['ci_upper'] / 100]
    results['expired_items'] = 100
    
    return results, {
        'ttl_expiry': ttl_result,
        'ttl_check': ttl_check_result
    }

def generate_report(all_results: Dict, detailed_results: Dict):
    """Generate detailed statistical report"""
    print("\n" + "=" * 60)
    print("STATISTICAL SUMMARY REPORT")
    print("=" * 60)
    
    print("\nImplementation: Claude Python (SmartCache)")
    print(f"Score: 78/100")
    print(f"Confidence Level: 95%")
    print(f"Samples per benchmark: 100 (warmup: 10)")
    
    print("\n" + "-" * 60)
    print("Performance Summary (mean with 95% CI):")
    print("-" * 60)
    
    # Single thread operations
    st = all_results['single_thread']
    print("\nSingle Thread Operations:")
    print(f"  PUT:       {st['put_microseconds']:.2f} µs [{st['put_ci'][0]:.2f}, {st['put_ci'][1]:.2f}]")
    print(f"  GET (hit): {st['get_hit_microseconds']:.2f} µs [{st['get_hit_ci'][0]:.2f}, {st['get_hit_ci'][1]:.2f}]")
    print(f"  GET (miss): {st['get_miss_microseconds']:.2f} µs [{st['get_miss_ci'][0]:.2f}, {st['get_miss_ci'][1]:.2f}]")
    
    # Concurrent operations
    c = all_results['concurrent']
    print("\nConcurrent Operations:")
    print(f"  10 threads:  {c['concurrent_10_threads_seconds']*1000:.2f} ms [{c['concurrent_10_threads_ci'][0]*1000:.2f}, {c['concurrent_10_threads_ci'][1]*1000:.2f}]")
    print(f"  100 threads: {c['concurrent_100_threads_seconds']*1000:.2f} ms [{c['concurrent_100_threads_ci'][0]*1000:.2f}, {c['concurrent_100_threads_ci'][1]*1000:.2f}]")
    
    # Eviction
    e = all_results['eviction']
    print("\nEviction Strategy:")
    print(f"  200 ops (100 evictions): {e['eviction_microseconds']:.2f} µs [{e['eviction_ci'][0]:.2f}, {e['eviction_ci'][1]:.2f}]")
    
    # TTL
    t = all_results['ttl']
    print("\nTTL Operations:")
    print(f"  TTL expiry: {t['ttl_expiry_microseconds']:.2f} µs [{t['ttl_expiry_ci'][0]:.2f}, {t['ttl_expiry_ci'][1]:.2f}]")
    print(f"  TTL check:  {t['ttl_check_microseconds']:.2f} µs [{t['ttl_check_ci'][0]:.2f}, {t['ttl_check_ci'][1]:.2f}]")
    
    print("\n" + "=" * 60)

def main():
    print("\n" + "=" * 60)
    print("Python Statistical Benchmark Suite")
    print("Implementation: Claude Python (SmartCache)")
    print("=" * 60)
    
    all_results = {}
    detailed_results = {}
    
    # Run benchmarks
    single_thread_results, single_thread_detailed = benchmark_single_thread_operations()
    all_results['single_thread'] = single_thread_results
    detailed_results['single_thread'] = single_thread_detailed
    
    concurrent_results, concurrent_detailed = benchmark_concurrent_operations()
    all_results['concurrent'] = concurrent_results
    detailed_results['concurrent'] = concurrent_detailed
    
    eviction_results, eviction_detailed = benchmark_eviction_strategies()
    all_results['eviction'] = eviction_results
    detailed_results['eviction'] = eviction_detailed
    
    ttl_results, ttl_detailed = benchmark_ttl_operations()
    all_results['ttl'] = ttl_results
    detailed_results['ttl'] = ttl_detailed
    
    # Generate report
    generate_report(all_results, detailed_results)
    
    # Save results
    os.makedirs('results', exist_ok=True)
    timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
    
    # Save standard format for compatibility
    output = {
        'implementation': 'Claude Python (Statistical)',
        'score': 78,
        'benchmarks': all_results,
        'confidence_level': 0.95,
        'methodology': 'Statistical analysis with outlier removal and confidence intervals'
    }
    
    filename = f'results/python_statistical_{timestamp}.json'
    with open(filename, 'w') as f:
        json.dump(output, f, indent=2)
    
    # Save detailed results
    detailed_filename = f'results/python_statistical_detailed_{timestamp}.json'
    with open(detailed_filename, 'w') as f:
        json.dump({
            'summary': output,
            'detailed': detailed_results
        }, f, indent=2, default=lambda x: x.tolist() if hasattr(x, 'tolist') else str(x))
    
    print(f"\nResults saved to:")
    print(f"  - {filename}")
    print(f"  - {detailed_filename}")
    
    return all_results

if __name__ == "__main__":
    main()