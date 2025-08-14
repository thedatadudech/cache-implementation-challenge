#!/bin/bash

# Build All Cache Implementations
# This script builds all implementations and prepares them for benchmarking

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo "=================================================="
echo "Building All Cache Implementations"
echo "=================================================="

# Check prerequisites
echo -e "\n${YELLOW}Checking prerequisites...${NC}"

# Check Python
if command -v python3 &> /dev/null; then
    echo -e "${GREEN}✓ Python3 found: $(python3 --version)${NC}"
else
    echo -e "${RED}✗ Python3 not found${NC}"
    exit 1
fi

# Check Java
if command -v javac &> /dev/null; then
    echo -e "${GREEN}✓ Java found: $(javac -version 2>&1 | head -n1)${NC}"
else
    echo -e "${RED}✗ Java compiler not found${NC}"
    exit 1
fi

# Check Rust
if command -v cargo &> /dev/null; then
    echo -e "${GREEN}✓ Rust found: $(rustc --version)${NC}"
else
    echo -e "${RED}✗ Rust/Cargo not found${NC}"
    exit 1
fi

# Build Python (nothing to build, just check)
echo -e "\n${YELLOW}Checking Python implementation...${NC}"
if [ -f "implementations/1_claude_python/smart_cache.py" ]; then
    echo -e "${GREEN}✓ Python implementation ready${NC}"
else
    echo -e "${RED}✗ Python implementation not found${NC}"
fi

# Build Java
echo -e "\n${YELLOW}Building Java implementation...${NC}"
cd implementations/2_qwen30b_java
if javac IntelligentCache.java 2>/dev/null; then
    echo -e "${GREEN}✓ Java implementation built${NC}"
else
    echo -e "${RED}✗ Java build failed${NC}"
fi
cd ../..

# Build Rust implementations
echo -e "\n${YELLOW}Building Rust implementations...${NC}"

# Qwen-30B Rust
echo "Building Qwen-30B Rust..."
cd implementations/3_qwen30b_rust
if cargo build --release 2>&1 | grep -E "(Compiling|Finished)" | tail -1; then
    echo -e "${GREEN}✓ Qwen-30B Rust built${NC}"
else
    echo -e "${RED}✗ Qwen-30B Rust build failed${NC}"
fi
cd ../..

# Qwen-235B Rust
echo "Building Qwen-235B Rust..."
cd implementations/4_qwen235b_rust
if cargo build --release 2>&1 | grep -E "(Compiling|Finished)" | tail -1; then
    echo -e "${GREEN}✓ Qwen-235B Rust built${NC}"
else
    echo -e "${RED}✗ Qwen-235B Rust build failed${NC}"
fi
cd ../..

# Qwen-435B Rust
echo "Building Qwen-435B Rust..."
cd implementations/5_qwen435b_rust
if cargo build --release 2>&1 | grep -E "(Compiling|Finished)" | tail -1; then
    echo -e "${GREEN}✓ Qwen-435B Rust built${NC}"
else
    echo -e "${RED}✗ Qwen-435B Rust build failed${NC}"
fi
cd ../..

# GLM-4.5 Rust (expected to fail)
echo "Building GLM-4.5 Rust (may fail due to known issues)..."
cd implementations/6_glm45_rust
if cargo build --release 2>&1 | grep -E "(Compiling|Finished)" | tail -1; then
    echo -e "${GREEN}✓ GLM-4.5 Rust built${NC}"
else
    echo -e "${YELLOW}⚠ GLM-4.5 Rust build failed (expected)${NC}"
fi
cd ../..

# Build benchmark suite
echo -e "\n${YELLOW}Building benchmark suite...${NC}"
cd benchmarks
if cargo build --release --all 2>&1 | grep -E "(Compiling|Finished)" | tail -1; then
    echo -e "${GREEN}✓ Benchmark suite built${NC}"
else
    echo -e "${RED}✗ Benchmark build failed${NC}"
fi
cd ..

echo -e "\n=================================================="
echo -e "${GREEN}Build process complete!${NC}"
echo "=================================================="
echo ""
echo "Next steps:"
echo "1. Run tests: ./test_all.sh"
echo "2. Run benchmarks: cd benchmarks && ./run_fair_concurrent_benchmarks.sh"
echo "3. View results: open benchmarks/results/fair_concurrent_benchmark_report_latest.html"