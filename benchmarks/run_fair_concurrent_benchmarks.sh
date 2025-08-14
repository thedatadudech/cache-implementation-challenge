#!/bin/bash

# Fair Concurrent Benchmark Script
# Runs comparable benchmarks across Python, Java, and Rust

set -e

BENCHMARK_DIR="$(cd "$(dirname "$0")" && pwd)"
RESULTS_DIR="$BENCHMARK_DIR/results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║           Fair Concurrent Benchmark Suite                     ║${NC}"
echo -e "${BLUE}║         Comparable Metrics Across All Languages               ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""

mkdir -p "$RESULTS_DIR"

# Run Python Fair Concurrent Benchmark
echo -e "\n${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}Running Python Fair Concurrent Benchmark (GIL-limited)${NC}"
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
if python3 fair_concurrent_benchmark.py; then
    echo -e "${GREEN}✓ Python benchmark complete${NC}"
else
    echo -e "${YELLOW}⚠ Python benchmark failed${NC}"
fi

# Run Java Fair Concurrent Benchmark
echo -e "\n${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}Running Java Fair Concurrent Benchmark (True parallelism)${NC}"
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo "Compiling Java benchmark..."
if javac -cp ../implementations/2_qwen30b_java FairConcurrentBenchmark.java; then
    echo "Running Java benchmark..."
    if java -cp .:../implementations/2_qwen30b_java FairConcurrentBenchmark; then
        echo -e "${GREEN}✓ Java benchmark complete${NC}"
    else
        echo -e "${YELLOW}⚠ Java benchmark failed${NC}"
    fi
else
    echo -e "${YELLOW}⚠ Java compilation failed${NC}"
fi

# Run Rust Fair Concurrent Benchmark
echo -e "\n${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}Running Rust Fair Concurrent Benchmark (True parallelism)${NC}"
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo "Building Rust benchmark..."
if cargo build --release --bin fair_concurrent_all 2>&1 | grep -E "(Compiling|Finished)"; then
    echo "Running Rust benchmark (All implementations: Qwen30B, Qwen235B, Qwen435B)..."
    if cargo run --release --bin fair_concurrent_all; then
        echo -e "${GREEN}✓ Rust benchmark complete${NC}"
    else
        echo -e "${YELLOW}⚠ Rust benchmark failed${NC}"
    fi
else
    echo -e "${YELLOW}⚠ Rust build failed${NC}"
fi

# Generate comparison report
echo -e "\n${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}Generating Comparison Report${NC}"
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

# Create Python script to generate comparison
cat > compare_fair_results.py << 'EOF'
import json
import glob
import os
from datetime import datetime

def load_latest_results(pattern):
    files = glob.glob(f'results/{pattern}')
    if not files:
        return None
    latest = max(files, key=os.path.getctime)
    with open(latest, 'r') as f:
        return json.load(f)

# Load results
python_results = load_latest_results('python_fair_concurrent_*.json')
java_results = load_latest_results('java_fair_concurrent_*.json')
rust_results = {
    'qwen30b': load_latest_results('rust_qwen30b_fair_concurrent_*.json'),
    'qwen235b': load_latest_results('rust_qwen235b_fair_concurrent_*.json'),
    'qwen435b': load_latest_results('rust_qwen435b_fair_concurrent_*.json')
}

print("\n" + "="*70)
print("FAIR CONCURRENT BENCHMARK COMPARISON")
print("="*70)

if python_results and 'benchmarks' in python_results:
    p = python_results['benchmarks']
    
    print("\n1. SHARED WORKLOAD (100 workers, 10000 operations)")
    print("-"*70)
    print("Implementation         | Ops/sec | Avg Op Time | Parallelism Factor")
    print("-"*70)
    
    if 'shared_workload' in p:
        sw = p['shared_workload']
        print(f"Python (GIL)          | {sw.get('ops_per_second', 'N/A'):>7} | {sw.get('avg_operation_time_ms', 'N/A'):>11} | {sw.get('parallelism_factor', 'N/A'):>18}")
    
    if java_results and 'benchmarks' in java_results and 'shared_workload' in java_results['benchmarks']:
        sw = java_results['benchmarks']['shared_workload']
        print(f"Java                  | {sw.get('ops_per_second', 'N/A'):>7} | {sw.get('avg_operation_time_ms', 'N/A'):>11} | {sw.get('parallelism_factor', 'N/A'):>18}")
    
    for name, results in rust_results.items():
        if results and 'benchmarks' in results and 'shared_workload' in results['benchmarks']:
            sw = results['benchmarks']['shared_workload']
            print(f"Rust {name:8}     | {sw.get('ops_per_second', 'N/A'):>7} | {sw.get('avg_operation_time_ms', 'N/A'):>11} | {sw.get('parallelism_factor', 'N/A'):>18}")
    
    print("\n2. PRODUCER-CONSUMER PATTERN (50 producers, 50 consumers)")
    print("-"*70)
    print("Implementation         | Total Ops | Ops/sec | Hit Rate")
    print("-"*70)
    
    if 'producer_consumer' in p:
        pc = p['producer_consumer']
        print(f"Python (GIL)          | {pc.get('total_operations', 'N/A'):>9} | {pc.get('ops_per_second', 'N/A'):>7} | {pc.get('hit_rate', 'N/A'):>8}")
    
    if java_results and 'benchmarks' in java_results and 'producer_consumer' in java_results['benchmarks']:
        pc = java_results['benchmarks']['producer_consumer']
        print(f"Java                  | {pc.get('total_operations', 'N/A'):>9} | {pc.get('ops_per_second', 'N/A'):>7} | {pc.get('hit_rate', 'N/A'):>8}")
    
    for name, results in rust_results.items():
        if results and 'benchmarks' in results and 'producer_consumer' in results['benchmarks']:
            pc = results['benchmarks']['producer_consumer']
            print(f"Rust {name:8}     | {pc.get('total_operations', 'N/A'):>9} | {pc.get('ops_per_second', 'N/A'):>7} | {pc.get('hit_rate', 'N/A'):>8}")
    
    print("\n3. I/O SIMULATION (100 workers with 5ms delays)")
    print("-"*70)
    print("Implementation         | Total Ops | Ops/sec | Speedup")
    print("-"*70)
    
    if 'io_simulation' in p:
        io = p['io_simulation']
        print(f"Python (GIL)          | {io.get('total_operations', 'N/A'):>9} | {io.get('ops_per_second', 'N/A'):>7} | {io.get('speedup', 'N/A'):>7}")
    
    if java_results and 'benchmarks' in java_results and 'io_simulation' in java_results['benchmarks']:
        io = java_results['benchmarks']['io_simulation']
        print(f"Java                  | {io.get('total_operations', 'N/A'):>9} | {io.get('ops_per_second', 'N/A'):>7} | {io.get('speedup', 'N/A'):>7}")
    
    for name, results in rust_results.items():
        if results and 'benchmarks' in results and 'io_simulation' in results['benchmarks']:
            io = results['benchmarks']['io_simulation']
            print(f"Rust {name:8}     | {io.get('total_operations', 'N/A'):>9} | {io.get('ops_per_second', 'N/A'):>7} | {io.get('speedup', 'N/A'):>7}")

print("\n" + "="*70)
print("KEY INSIGHTS:")
print("-"*70)
print("• Parallelism Factor: How much faster than sequential execution")
print("• Python's GIL prevents true CPU parallelism (factor ~1.0x)")
print("• Java/Rust show real parallelism with higher factors")
print("• I/O Simulation: All languages benefit from threading with I/O waits")
print("• Hit Rate: Cache effectiveness under concurrent access")
print("="*70)
EOF

python3 compare_fair_results.py

# Generate HTML report
echo -e "\n${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}Generating HTML Report${NC}"
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
if python3 generate_fair_concurrent_report.py; then
    echo -e "${GREEN}✓ HTML report generated${NC}"
else
    echo -e "${YELLOW}⚠ HTML report generation failed${NC}"
fi

echo -e "\n${GREEN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║            Fair Concurrent Benchmarks Complete! 🎉            ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════════════╝${NC}"

# Clean up
rm -f *.class compare_fair_results.py 2>/dev/null

echo -e "\nResults saved in: ${BLUE}$RESULTS_DIR${NC}"
echo "These metrics are directly comparable across all languages!"
echo -e "\nHTML Report: ${BLUE}$RESULTS_DIR/fair_concurrent_benchmark_report_latest.html${NC}"