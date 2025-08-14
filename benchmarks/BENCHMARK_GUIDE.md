# 📊 Comprehensive Benchmark Guide

This guide explains all benchmark types, methodologies, and how to run and interpret results for the Smart Cache implementations.

## 🎯 Benchmark Philosophy

We use two complementary benchmark approaches:
1. **Statistical Benchmarks**: Isolated performance measurements with confidence intervals
2. **Fair Concurrent Benchmarks**: Real-world workload simulations for cross-language comparison

## 🚀 Quick Start

### Run Everything
```bash
./run_fair_concurrent_benchmarks.sh
# Generates HTML report in results/fair_concurrent_benchmark_report_latest.html
```

### View Results
```bash
open results/fair_concurrent_benchmark_report_latest.html  # macOS
xdg-open results/fair_concurrent_benchmark_report_latest.html  # Linux
```

## ⚖️ Fair Concurrent Benchmarks (Recommended)

These benchmarks ensure fair comparison across languages by using external workloads that all implementations must process.

### Test Types

#### 1. Producer-Consumer Pattern
- **Setup**: 50 producers, 50 consumers
- **Duration**: 5 seconds
- **Workload**: Producers generate data with MD5 hashing, consumers read and verify
- **Measures**: Throughput, hit rate, producer/consumer balance
- **Key Metric**: Operations per second

#### 2. Shared Workload Queue
- **Setup**: 100 workers, 10,000 operations
- **Workload**: Pre-generated queue with 70% writes, 30% reads
- **Measures**: Total throughput, parallelism factor
- **Key Metric**: How much faster than sequential execution

#### 3. I/O Simulation
- **Setup**: 100 workers, 5ms database delays
- **Workload**: Simulates database queries with sleep(5ms)
- **Measures**: Threading benefit for I/O-bound operations
- **Key Metric**: Speedup vs sequential (should be ~100x)

#### 4. Eviction Strategy
- **Setup**: 100 capacity cache, 200 insertions
- **Workload**: Forces 100 evictions to test LRU behavior
- **Measures**: Eviction accuracy and performance
- **Key Metric**: Eviction efficiency percentage

#### 5. TTL Operations
- **Setup**: 100 items with 100ms TTL
- **Workload**: Tests expiration and TTL check overhead
- **Measures**: Expiry accuracy, check performance
- **Key Metric**: Operations per second for TTL checks

### Running Fair Benchmarks

```bash
# All languages at once
./run_fair_concurrent_benchmarks.sh

# Individual languages
python3 fair_concurrent_benchmark.py
java -cp .:../implementations/2_qwen30b_java FairConcurrentBenchmark
cargo run --release --bin fair_concurrent_all
```

### Understanding Results

#### Parallelism Factor
- `< 1.0x`: Worse than sequential (overhead > benefit)
- `1.0x`: No benefit from parallelism
- `> 1.0x`: True parallel speedup
- Python typically shows `0.2-0.3x` due to GIL
- Java/Rust show `2-5x` with true parallelism

#### Hit Rate
- `> 60%`: Good cache effectiveness
- `40-60%`: Moderate, typical for random access
- `< 40%`: Poor, cache too small or bad access pattern

#### Throughput (ops/sec)
- Higher is better
- Compare relative performance between implementations
- Python baseline: ~30,000 ops/sec
- Rust optimized: ~90,000 ops/sec

## 📊 Statistical Benchmarks

Traditional microbenchmarks with statistical analysis for confidence.

### Methodology

1. **Warmup Phase**: 10 iterations to stabilize JIT/CPU
2. **Sample Collection**: 
   - Single-thread: 100 samples
   - Concurrent: 20 samples
3. **Outlier Removal**: IQR method (Q1-1.5×IQR to Q3+1.5×IQR)
4. **Statistics Calculated**:
   - Mean, Median, Standard Deviation
   - 95% Confidence Interval
   - Min/Max after outlier removal

### Test Scenarios

#### Single-Thread Tests
```
PUT: Insert 1000 unique keys
GET_HIT: Retrieve existing keys
GET_MISS: Retrieve non-existent keys
```

#### Concurrent Tests
```
10_threads: Light concurrency
100_thread_pool: Heavy concurrency (thread pool)
```

### Running Statistical Benchmarks

```bash
# Python
python3 benchmark_python_statistical.py

# Java
javac -cp ../implementations/2_qwen30b_java BenchmarkJavaStatistical.java
java -cp .:../implementations/2_qwen30b_java BenchmarkJavaStatistical

# Rust (uses Criterion framework)
cargo bench
```

### Interpreting Statistical Results

```
Single-threaded PUT operations (100000 cache size):
  Samples: 85 (after removing 15 outliers)
  Mean: 2.456 ± 0.234 microseconds [95% CI: 2.222-2.690]
  Median: 2.401 microseconds
  Std Dev: 0.543 microseconds
```

- **Mean ± CI**: Average with confidence interval
- **Median**: Middle value (robust to outliers)
- **Outliers Removed**: Quality of data
- **Std Dev**: Consistency of performance

## 📈 HTML Report Generation

Reports are automatically generated after benchmarks run.

### Report Contents

1. **Comparison Tables**: All metrics side-by-side
2. **Performance Charts**: Visual representation
3. **Key Insights**: Automatic analysis
4. **Implementation Details**: Architecture notes

### Manual Generation

```bash
python3 generate_fair_concurrent_report.py
# Output: results/fair_concurrent_benchmark_report_TIMESTAMP.html
#         results/fair_concurrent_benchmark_report_latest.html
```

## 🔧 Configuration

### Cache Size
All benchmarks use 100,000 capacity (realistic for production).

### Thread Pools
- Python: `ThreadPoolExecutor`
- Java: `FixedThreadPool`
- Rust: `threadpool` crate

### Timing
- Python: `time.perf_counter()`
- Java: `System.nanoTime()`
- Rust: `std::time::Instant`

## 📊 Benchmark File Structure

```
benchmarks/
├── Fair Concurrent Suite
│   ├── fair_concurrent_benchmark.py        # Python implementation
│   ├── FairConcurrentBenchmark.java       # Java implementation
│   └── src/bin/fair_concurrent_all.rs     # Rust (all models)
│
├── Statistical Suite
│   ├── benchmark_python_statistical.py     # Python with CI
│   ├── BenchmarkJavaStatistical.java      # Java with warmup
│   └── benches/benchmark_suite.rs         # Rust Criterion
│
├── Utilities
│   ├── convert_rust_results.py            # Parse Rust output
│   ├── generate_fair_concurrent_report.py # Create HTML report
│   └── run_fair_concurrent_benchmarks.sh  # Automated runner
│
└── Results
    └── results/
        ├── *_fair_concurrent_*.json       # Raw data
        └── *.html                          # Reports
```

## 🎯 Best Practices

### For Accurate Results

1. **Close Other Applications**: Reduce system noise
2. **Consistent Environment**: Same machine, same conditions
3. **Multiple Runs**: Average of 3+ runs for stability
4. **Check Temperature**: Thermal throttling affects results

### For Fair Comparison

1. **Same Workload**: All implementations process identical operations
2. **Thread Pools**: Not native threads (avoids OS limits)
3. **Warmup Period**: Especially important for JVM
4. **Cache Size**: 100k entries (not 1k toy examples)

## 🐛 Troubleshooting

### Common Issues

#### Python Shows Poor Parallelism
- **Expected**: GIL prevents true parallelism
- **Solution**: This is normal, Python excels at I/O-bound tasks

#### Java High Variability
- **Cause**: GC pauses
- **Solution**: Run with `-XX:+UseG1GC` for more consistent latency

#### Rust Compilation Errors
- **GLM-4.5**: Known compilation issues
- **Solution**: Exclude from benchmarks or fix syntax errors

#### Thread Pool Exhaustion
- **Symptom**: "Resource temporarily unavailable"
- **Solution**: Use thread pools, not native threads

### Debug Commands

```bash
# Verbose output
RUST_LOG=debug cargo run --release --bin fair_concurrent_all

# Java with GC logging
java -Xlog:gc -cp . FairConcurrentBenchmark

# Python with profiling
python3 -m cProfile fair_concurrent_benchmark.py
```

## 📊 Performance Expectations

### Relative Performance (normalized to Python = 1.0)

| Operation | Python | Java | Rust-30B | Rust-235B | Rust-435B |
|-----------|--------|------|----------|-----------|-----------|
| PUT | 1.0 | 3.0 | 6.0 | 7.0 | 14.0 |
| GET | 1.0 | 4.0 | 12.0 | 14.0 | 28.0 |
| Concurrent | 1.0 | 1.4 | 1.6 | 1.8 | 2.8 |

### Absolute Performance Targets

- **PUT**: < 10μs (single-thread)
- **GET**: < 5μs (single-thread)
- **Throughput**: > 50k ops/sec (100 threads)
- **Memory**: < 100MB for 100k entries

## 🎓 Understanding Cache Metrics

### Hit Rate
```
Hit Rate = (Cache Hits) / (Total Requests) × 100%
```
- Measures cache effectiveness
- Higher is better (reduces backend load)

### Throughput
```
Throughput = Operations / Time
```
- Measures raw performance
- Limited by lock contention in concurrent scenarios

### Parallelism Factor
```
Parallelism = (Time_Sequential × Ops) / (Time_Parallel × Workers)
```
- Measures scaling efficiency
- Theoretical max = number of cores

### Eviction Efficiency
```
Efficiency = (Items_Evicted) / (Evictions_Forced) × 100%
```
- Measures LRU accuracy
- Should be close to 100% for correct implementation

## 📝 Adding New Benchmarks

To add a new benchmark type:

1. Add to `fair_concurrent_benchmark.py`:
```python
def benchmark_new_test(self, param1, param2):
    # Implementation
    return results_dict
```

2. Add to `FairConcurrentBenchmark.java`:
```java
public BenchmarkResult benchmarkNewTest(int param1, int param2) {
    // Implementation
    return result;
}
```

3. Add to Rust macro in `fair_concurrent_all.rs`:
```rust
pub fn benchmark_new_test(param1: usize, param2: usize) 
    -> HashMap<String, serde_json::Value> {
    // Implementation
}
```

4. Update HTML generator in `generate_fair_concurrent_report.py`

## 📚 References

- [Criterion.rs Documentation](https://github.com/bheisler/criterion.rs)
- [Python timeit Module](https://docs.python.org/3/library/timeit.html)
- [JMH (Java Microbenchmark Harness)](https://openjdk.java.net/projects/code-tools/jmh/)
- [Statistical Analysis in Benchmarking](https://www.brendangregg.com/usemethod.html)

---

*For questions or improvements, please refer to the main README.md or create an issue.*