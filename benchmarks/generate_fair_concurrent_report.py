#!/usr/bin/env python3
"""
Generate HTML report from fair concurrent benchmark results
"""

import json
import glob
import os
from datetime import datetime
from pathlib import Path

def load_json_results(pattern):
    """Load all JSON results matching the pattern"""
    results = []
    for file in glob.glob(f"results/{pattern}"):
        try:
            with open(file, 'r') as f:
                data = json.load(f)
                results.append(data)
        except:
            print(f"Warning: Could not load {file}")
    return results

def generate_html_report():
    """Generate comprehensive HTML report from all fair concurrent benchmarks"""
    
    # Load all results
    python_results = load_json_results("python_fair_concurrent_*.json")
    java_results = load_json_results("java_fair_concurrent_*.json")
    rust_30b_results = load_json_results("rust_qwen30b_fair_concurrent_*.json")
    rust_235b_results = load_json_results("rust_qwen235b_fair_concurrent_*.json")
    rust_435b_results = load_json_results("rust_qwen435b_fair_concurrent_*.json")
    rust_glm45_results = load_json_results("rust_glm45_fair_concurrent_*.json")
    
    # Get the latest result from each category
    def get_latest(results):
        if not results:
            return None
        return sorted(results, key=lambda x: x.get('timestamp', ''), reverse=True)[0]
    
    python_data = get_latest(python_results)
    java_data = get_latest(java_results)
    rust_30b_data = get_latest(rust_30b_results)
    rust_235b_data = get_latest(rust_235b_results)
    rust_435b_data = get_latest(rust_435b_results)
    rust_glm45_data = get_latest(rust_glm45_results)
    
    # Generate HTML
    html = """
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Fair Concurrent Benchmark Report</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
            margin: 0;
            padding: 20px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
        }
        .container {
            max-width: 1400px;
            margin: 0 auto;
            background: white;
            border-radius: 12px;
            box-shadow: 0 20px 60px rgba(0,0,0,0.3);
            overflow: hidden;
        }
        .header {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 30px;
            text-align: center;
        }
        h1 {
            margin: 0;
            font-size: 2.5em;
            font-weight: 600;
        }
        .subtitle {
            margin-top: 10px;
            opacity: 0.9;
            font-size: 1.1em;
        }
        .timestamp {
            margin-top: 15px;
            opacity: 0.8;
            font-size: 0.9em;
        }
        .content {
            padding: 30px;
        }
        .section {
            margin-bottom: 40px;
        }
        h2 {
            color: #667eea;
            border-bottom: 2px solid #e0e0e0;
            padding-bottom: 10px;
            margin-bottom: 20px;
        }
        h3 {
            color: #764ba2;
            margin-top: 25px;
            margin-bottom: 15px;
        }
        table {
            width: 100%;
            border-collapse: collapse;
            margin: 20px 0;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        th {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 12px;
            text-align: left;
            font-weight: 600;
        }
        td {
            padding: 10px 12px;
            border-bottom: 1px solid #e0e0e0;
        }
        tr:hover {
            background: #f5f5f5;
        }
        .metric-name {
            font-weight: 500;
            color: #555;
        }
        .best-value {
            background: #4caf50;
            color: white;
            font-weight: 600;
            border-radius: 4px;
            padding: 2px 6px;
        }
        .good-value {
            background: #8bc34a;
            color: white;
            border-radius: 4px;
            padding: 2px 6px;
        }
        .warning {
            background: #fff3cd;
            border: 1px solid #ffc107;
            border-radius: 6px;
            padding: 15px;
            margin: 20px 0;
            color: #856404;
        }
        .info {
            background: #d1ecf1;
            border: 1px solid #17a2b8;
            border-radius: 6px;
            padding: 15px;
            margin: 20px 0;
            color: #0c5460;
        }
        .chart-container {
            margin: 30px 0;
            padding: 20px;
            background: #f9f9f9;
            border-radius: 8px;
        }
        .summary-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 20px;
            margin: 20px 0;
        }
        .summary-card {
            background: linear-gradient(135deg, #f5f5f5 0%, #e0e0e0 100%);
            border-radius: 8px;
            padding: 20px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
        }
        .summary-title {
            font-size: 0.9em;
            color: #666;
            margin-bottom: 5px;
        }
        .summary-value {
            font-size: 1.8em;
            font-weight: 600;
            color: #333;
        }
        .summary-subtitle {
            font-size: 0.85em;
            color: #888;
            margin-top: 5px;
        }
        .impl-badge {
            display: inline-block;
            padding: 4px 8px;
            border-radius: 4px;
            font-size: 0.85em;
            font-weight: 600;
            margin-right: 5px;
        }
        .python-badge { background: #3776ab; color: white; }
        .java-badge { background: #007396; color: white; }
        .rust-badge { background: #ce422b; color: white; }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🚀 Fair Concurrent Benchmark Report</h1>
            <div class="subtitle">Comparing Cache Implementations Across Languages with Realistic Workloads</div>
            <div class="timestamp">Generated: """ + datetime.now().strftime("%Y-%m-%d %H:%M:%S") + """</div>
        </div>
        
        <div class="content">
            <div class="info">
                <strong>ℹ️ About Fair Concurrent Benchmarks:</strong><br>
                These benchmarks measure actual concurrent throughput under realistic conditions, with external workloads
                that ensure fair comparison across languages. Unlike simple concurrent tests, these simulate real-world
                scenarios including I/O operations, producer-consumer patterns, and cache eviction strategies.
            </div>
"""
    
    # Create benchmark comparison tables
    benchmarks = [
        ("Producer-Consumer Pattern", "producer_consumer", [
            ("Total Operations", "total_operations"),
            ("Throughput", "ops_per_second"),
            ("Hit Rate", "hit_rate"),
            ("PUT Throughput", "puts_per_second"),
            ("GET Throughput", "gets_per_second")
        ]),
        ("Shared Workload", "shared_workload", [
            ("Duration", "duration"),
            ("Total Operations", "total_operations"),
            ("Throughput", "ops_per_second"),
            ("Avg Operation Time", "avg_operation_time_ms"),
            ("Parallelism Factor", "parallelism_factor")
        ]),
        ("I/O Simulation", "io_simulation", [
            ("Duration", "duration"),
            ("Total Operations", "total_operations"),
            ("Throughput", "ops_per_second"),
            ("Theoretical Sequential Time", "theoretical_sequential_time"),
            ("Speedup", "speedup")
        ]),
        ("Eviction Strategy", "eviction", [
            ("Cache Size", "cache_size"),
            ("Total Insertions", "total_insertions"),
            ("Evicted Count", "evicted_count"),
            ("Throughput", "ops_per_second"),
            ("Eviction Efficiency", "eviction_efficiency")
        ]),
        ("TTL Operations", "ttl", [
            ("TTL Duration (ms)", "ttl_ms"),
            ("Expired Count", "expired_count"),
            ("Expiry Rate", "expiry_rate"),
            ("Check Performance", "check_ops_per_second"),
            ("Avg Check Time (μs)", "avg_check_time_us")
        ])
    ]
    
    for bench_name, bench_key, metrics in benchmarks:
        html += f"""
            <div class="section">
                <h2>{bench_name}</h2>
                <table>
                    <thead>
                        <tr>
                            <th>Metric</th>
                            <th><span class="impl-badge python-badge">Python</span></th>
                            <th><span class="impl-badge java-badge">Java</span></th>
                            <th><span class="impl-badge rust-badge">Rust 30B</span></th>
                            <th><span class="impl-badge rust-badge">Rust 235B</span></th>
                            <th><span class="impl-badge rust-badge">Rust 435B</span></th>
                            <th><span class="impl-badge rust-badge">GLM-4.5</span></th>
                        </tr>
                    </thead>
                    <tbody>
"""
        
        for metric_name, metric_key in metrics:
            html += f"""
                        <tr>
                            <td class="metric-name">{metric_name}</td>
"""
            
            # Get values for each implementation
            values = []
            for data in [python_data, java_data, rust_30b_data, rust_235b_data, rust_435b_data, rust_glm45_data]:
                if data and 'benchmarks' in data and bench_key in data['benchmarks']:
                    value = data['benchmarks'][bench_key].get(metric_key, 'N/A')
                    values.append(str(value))
                else:
                    values.append('N/A')
            
            # Determine best values for highlighting (for numeric comparisons)
            best_indices = []
            if any(v != 'N/A' for v in values):
                try:
                    # For throughput metrics, higher is better
                    if 'per_second' in metric_key or 'speedup' in metric_key or 'parallelism' in metric_key:
                        numeric_values = []
                        for v in values:
                            try:
                                # Remove units and convert to float
                                cleaned = v.replace('x', '').replace('%', '').replace('ops/sec', '').strip()
                                numeric_values.append(float(cleaned) if v != 'N/A' else -1)
                            except:
                                numeric_values.append(-1)
                        if any(v > 0 for v in numeric_values):
                            max_val = max(v for v in numeric_values if v > 0)
                            best_indices = [i for i, v in enumerate(numeric_values) if v == max_val]
                except:
                    pass
            
            # Output values with highlighting
            for i, value in enumerate(values):
                if i in best_indices and value != 'N/A':
                    html += f'                            <td><span class="best-value">{value}</span></td>\n'
                else:
                    html += f'                            <td>{value}</td>\n'
            
            html += "                        </tr>\n"
        
        html += """
                    </tbody>
                </table>
            </div>
"""
    
    # Add summary section
    html += """
            <div class="section">
                <h2>Key Insights</h2>
                <div class="summary-grid">
"""
    
    # Calculate summary statistics
    if python_data and java_data:
        # Compare Python vs Java throughput
        py_throughput = python_data['benchmarks']['shared_workload'].get('ops_per_second', '0')
        java_throughput = java_data['benchmarks']['shared_workload'].get('ops_per_second', '0')
        
        html += f"""
                    <div class="summary-card">
                        <div class="summary-title">Python Throughput</div>
                        <div class="summary-value">{py_throughput}</div>
                        <div class="summary-subtitle">ops/sec (Shared Workload)</div>
                    </div>
                    <div class="summary-card">
                        <div class="summary-title">Java Throughput</div>
                        <div class="summary-value">{java_throughput}</div>
                        <div class="summary-subtitle">ops/sec (Shared Workload)</div>
                    </div>
"""
    
    # Add Rust throughput if available
    for data, name in [(rust_30b_data, "Rust 30B"), (rust_235b_data, "Rust 235B"), (rust_435b_data, "Rust 435B"), (rust_glm45_data, "GLM-4.5")]:
        if data and 'benchmarks' in data:
            throughput = data['benchmarks']['shared_workload'].get('ops_per_second', '0')
            html += f"""
                    <div class="summary-card">
                        <div class="summary-title">{name} Throughput</div>
                        <div class="summary-value">{throughput}</div>
                        <div class="summary-subtitle">ops/sec (Shared Workload)</div>
                    </div>
"""
    
    html += """
                </div>
                
                <h3>Performance Analysis</h3>
                <ul>
                    <li><strong>Python:</strong> Limited by GIL (Global Interpreter Lock), shows good performance for I/O-bound operations but limited CPU parallelism.</li>
                    <li><strong>Java:</strong> True multi-threading with JVM optimizations, excellent performance for CPU-intensive operations.</li>
                    <li><strong>Rust:</strong> Zero-cost abstractions and true parallelism, minimal overhead with memory safety guarantees.</li>
                    <li><strong>GLM-4.5:</strong> Not included in benchmarks due to compilation errors in the implementation.</li>
                </ul>
                
                <h3>Benchmark Descriptions</h3>
                <ul>
                    <li><strong>Producer-Consumer:</strong> Simulates multiple producers adding items while consumers read them concurrently.</li>
                    <li><strong>Shared Workload:</strong> All workers pull from a common queue ensuring equal work distribution.</li>
                    <li><strong>I/O Simulation:</strong> Simulates database/network delays where threading provides real benefits.</li>
                    <li><strong>Eviction Strategy:</strong> Tests LRU eviction by overfilling the cache.</li>
                    <li><strong>TTL Operations:</strong> Tests time-to-live expiration and performance of TTL checks.</li>
                </ul>
            </div>
            
            <div class="warning">
                <strong>⚠️ Important Notes:</strong><br>
                • Cache size is set to 100,000 entries for all implementations to reflect realistic application scenarios.<br>
                • All benchmarks use thread pools (100 workers) for fair comparison across languages.<br>
                • Python's GIL affects CPU-bound operations but not I/O-bound operations.<br>
                • Results may vary based on system specifications and current load.
            </div>
        </div>
    </div>
</body>
</html>
"""
    
    # Write the HTML file
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    filename = f'results/fair_concurrent_benchmark_report_{timestamp}.html'
    
    # Ensure results directory exists
    Path('results').mkdir(exist_ok=True)
    
    with open(filename, 'w') as f:
        f.write(html)
    
    # Also save a copy with fixed name for easy access
    with open('results/fair_concurrent_benchmark_report_latest.html', 'w') as f:
        f.write(html)
    
    print(f"HTML report generated: {filename}")
    print(f"Latest report also saved as: results/fair_concurrent_benchmark_report_latest.html")
    print(f"Total implementations compared: {sum(1 for d in [python_data, java_data, rust_30b_data, rust_235b_data, rust_435b_data, rust_glm45_data] if d)}")
    
    # List missing data
    missing = []
    if not python_data:
        missing.append("Python")
    if not java_data:
        missing.append("Java")
    if not rust_30b_data:
        missing.append("Rust Qwen30B")
    if not rust_235b_data:
        missing.append("Rust Qwen235B")
    if not rust_435b_data:
        missing.append("Rust Qwen435B")
    if not rust_glm45_data:
        missing.append("GLM-4.5")
    
    if missing:
        print(f"Missing data for: {', '.join(missing)}")
        print("Run the respective benchmarks to include them in the report.")

if __name__ == "__main__":
    generate_html_report()