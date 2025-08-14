#!/usr/bin/env python3
"""
Convert Rust benchmark text output to JSON format
Compatible with the statistical benchmark format
"""

import re
import json
from datetime import datetime

def parse_rust_output(filename):
    """Parse Rust benchmark output and convert to JSON"""
    
    with open(filename, 'r') as f:
        content = f.read()
    
    # Initialize results for each implementation
    results = {
        'qwen30b': {
            'implementation': 'Rust (Qwen30b)',
            'score': 85,
            'benchmarks': {
                'single_thread': {},
                'concurrent': {},
                'eviction': {},
                'ttl': {}
            }
        },
        'qwen235b': {
            'implementation': 'Rust (Qwen235b)', 
            'score': 91,
            'benchmarks': {
                'single_thread': {},
                'concurrent': {},
                'eviction': {},
                'ttl': {}
            }
        },
        'qwen435b': {
            'implementation': 'Rust (Qwen435b)',
            'score': 94,
            'benchmarks': {
                'single_thread': {},
                'concurrent': {},
                'eviction': {},
                'ttl': {}
            }
        }
    }
    
    # Parse single thread operations
    # Pattern: single_thread/qwen30b_put_1000...time: [lower estimate upper]
    pattern = r'single_thread/(\w+)_(put_1000|get_hit|get_miss).*?time:\s+\[([0-9.]+)\s+(\w+)\s+([0-9.]+)\s+(\w+)\s+([0-9.]+)\s+(\w+)\]'
    matches = re.findall(pattern, content, re.DOTALL)
    
    for match in matches:
        impl_name = match[0]
        operation = match[1]
        lower = float(match[2])
        estimate = float(match[4])
        upper = float(match[6])
        unit = match[5]
        
        # Convert to microseconds
        multiplier = {'ns': 0.001, 'µs': 1.0, 'ms': 1000.0, 's': 1000000.0}.get(unit, 1.0)
        
        if impl_name in results:
            if operation == 'put_1000':
                results[impl_name]['benchmarks']['single_thread']['put_microseconds'] = estimate * multiplier
                results[impl_name]['benchmarks']['single_thread']['put_ci'] = [lower * multiplier, upper * multiplier]
            elif operation == 'get_hit':
                results[impl_name]['benchmarks']['single_thread']['get_hit_microseconds'] = estimate * multiplier
                results[impl_name]['benchmarks']['single_thread']['get_hit_ci'] = [lower * multiplier, upper * multiplier]
            elif operation == 'get_miss':
                results[impl_name]['benchmarks']['single_thread']['get_miss_microseconds'] = estimate * multiplier
                results[impl_name]['benchmarks']['single_thread']['get_miss_ci'] = [lower * multiplier, upper * multiplier]
    
    # Parse concurrent operations (both native threads and thread pool)
    # Pattern handles multiline output where benchmark name and time are on separate lines
    pattern = r'concurrent/(\w+)_(10_threads|100_thread_pool)\s*\n\s*time:\s+\[([0-9.]+)\s+(\w+)\s+([0-9.]+)\s+(\w+)\s+([0-9.]+)\s+(\w+)\]'
    matches = re.findall(pattern, content, re.MULTILINE)
    
    for match in matches:
        impl_name = match[0]
        thread_count = match[1]
        lower = float(match[2])
        estimate = float(match[4])
        upper = float(match[6])
        unit = match[5]
        
        # Convert to seconds
        multiplier = {'ns': 0.000000001, 'µs': 0.000001, 'ms': 0.001, 's': 1.0}.get(unit, 1.0)
        
        if impl_name in results:
            if thread_count == '10_threads':
                results[impl_name]['benchmarks']['concurrent']['concurrent_10_threads_seconds'] = estimate * multiplier
                results[impl_name]['benchmarks']['concurrent']['concurrent_10_threads_ci'] = [lower * multiplier, upper * multiplier]
            elif thread_count == '100_thread_pool':
                results[impl_name]['benchmarks']['concurrent']['concurrent_100_threads_seconds'] = estimate * multiplier
                results[impl_name]['benchmarks']['concurrent']['concurrent_100_threads_ci'] = [lower * multiplier, upper * multiplier]
    
    # Parse eviction operations
    pattern = r'eviction/(\w+)_eviction_100.*?time:\s+\[([0-9.]+)\s+(\w+)\s+([0-9.]+)\s+(\w+)\s+([0-9.]+)\s+(\w+)\]'
    matches = re.findall(pattern, content, re.DOTALL)
    
    for match in matches:
        impl_name = match[0]
        lower = float(match[1])
        estimate = float(match[3])
        upper = float(match[5])
        unit = match[4]
        
        # Convert to microseconds
        multiplier = {'ns': 0.001, 'µs': 1.0, 'ms': 1000.0, 's': 1000000.0}.get(unit, 1.0)
        
        if impl_name in results:
            results[impl_name]['benchmarks']['eviction']['eviction_microseconds'] = estimate * multiplier
            results[impl_name]['benchmarks']['eviction']['eviction_ci'] = [lower * multiplier, upper * multiplier]
            results[impl_name]['benchmarks']['eviction']['evictions_count'] = 100
    
    # Parse TTL operations
    pattern = r'ttl/(\w+)_(ttl_expiry|ttl_check).*?time:\s+\[([0-9.]+)\s+(\w+)\s+([0-9.]+)\s+(\w+)\s+([0-9.]+)\s+(\w+)\]'
    matches = re.findall(pattern, content, re.DOTALL)
    
    for match in matches:
        impl_name = match[0]
        ttl_op = match[1]
        lower = float(match[2])
        estimate = float(match[4])
        upper = float(match[6])
        unit = match[5]
        
        # Convert to microseconds
        multiplier = {'ns': 0.001, 'µs': 1.0, 'ms': 1000.0, 's': 1000000.0}.get(unit, 1.0)
        
        if impl_name in results:
            if ttl_op == 'ttl_expiry':
                results[impl_name]['benchmarks']['ttl']['ttl_expiry_microseconds'] = estimate * multiplier
                results[impl_name]['benchmarks']['ttl']['ttl_expiry_ci'] = [lower * multiplier, upper * multiplier]
                results[impl_name]['benchmarks']['ttl']['expired_items'] = 100
            elif ttl_op == 'ttl_check':
                # TTL check is for 100 items, so divide by 100 for per-operation time
                results[impl_name]['benchmarks']['ttl']['ttl_check_microseconds'] = (estimate * multiplier) / 100
                results[impl_name]['benchmarks']['ttl']['ttl_check_ci'] = [(lower * multiplier) / 100, (upper * multiplier) / 100]
    
    return results

def main():
    # Find the latest Rust benchmark output
    import glob
    import os
    
    files = glob.glob('results/rust_bench_output_*.txt')
    if not files:
        print("No Rust benchmark output files found")
        return
    
    latest_file = max(files, key=os.path.getctime)
    print(f"Parsing: {latest_file}")
    
    results = parse_rust_output(latest_file)
    
    # Generate timestamp from filename
    timestamp = latest_file.split('_')[-1].replace('.txt', '')
    
    # Save individual implementation files
    for impl_name, data in results.items():
        filename = f'results/rust_{impl_name}_{timestamp}.json'
        
        # Add metadata
        data['confidence_level'] = 0.95
        data['methodology'] = 'Criterion.rs statistical benchmarking framework'
        
        with open(filename, 'w') as f:
            json.dump(data, f, indent=2)
        print(f"Created: {filename}")
    
    # Save combined file
    combined = {
        'timestamp': timestamp,
        'language': 'Rust',
        'implementations': results
    }
    
    combined_filename = f'results/rust_all_{timestamp}.json'
    with open(combined_filename, 'w') as f:
        json.dump(combined, f, indent=2)
    print(f"Created: {combined_filename}")

if __name__ == "__main__":
    main()