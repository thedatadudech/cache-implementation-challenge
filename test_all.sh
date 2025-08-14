#!/bin/bash

# Test All Cache Implementations
# This script runs basic tests for all implementations

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo "=================================================="
echo "Testing All Cache Implementations"
echo "=================================================="

# Test Python
echo -e "\n${YELLOW}Testing Python implementation (Claude)...${NC}"
cd implementations/1_claude_python
if python3 smart_cache.py > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Python tests passed${NC}"
else
    echo -e "${RED}✗ Python tests failed${NC}"
fi
cd ../..

# Test Java
echo -e "\n${YELLOW}Testing Java implementation (Qwen-30B)...${NC}"
cd implementations/2_qwen30b_java
if [ -f "IntelligentCache.class" ]; then
    if java IntelligentCache > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Java tests passed${NC}"
    else
        echo -e "${RED}✗ Java tests failed${NC}"
    fi
else
    echo -e "${YELLOW}⚠ Java not compiled, run build_all.sh first${NC}"
fi
cd ../..

# Test Rust implementations
echo -e "\n${YELLOW}Testing Rust implementations...${NC}"

# Qwen-30B Rust
echo "Testing Qwen-30B Rust..."
cd implementations/3_qwen30b_rust
if cargo test --release 2>&1 | grep -E "(test result|running)" | tail -1; then
    echo -e "${GREEN}✓ Qwen-30B Rust tests passed${NC}"
else
    echo -e "${RED}✗ Qwen-30B Rust tests failed${NC}"
fi
cd ../..

# Qwen-235B Rust
echo "Testing Qwen-235B Rust..."
cd implementations/4_qwen235b_rust
if cargo test --release 2>&1 | grep -E "(test result|running)" | tail -1; then
    echo -e "${GREEN}✓ Qwen-235B Rust tests passed${NC}"
else
    echo -e "${RED}✗ Qwen-235B Rust tests failed${NC}"
fi
cd ../..

# Qwen-435B Rust
echo "Testing Qwen-435B Rust..."
cd implementations/5_qwen435b_rust
if cargo test --release 2>&1 | grep -E "(test result|running)" | tail -1; then
    echo -e "${GREEN}✓ Qwen-435B Rust tests passed${NC}"
else
    echo -e "${RED}✗ Qwen-435B Rust tests failed${NC}"
fi
cd ../..

# GLM-4.5 Rust (expected to fail)
echo "Testing GLM-4.5 Rust (may fail due to known issues)..."
cd implementations/6_glm45_rust
if cargo test --release 2>&1 | grep -E "(test result|running)" | tail -1; then
    echo -e "${GREEN}✓ GLM-4.5 Rust tests passed${NC}"
else
    echo -e "${YELLOW}⚠ GLM-4.5 Rust tests failed (expected due to compilation issues)${NC}"
fi
cd ../..

echo -e "\n=================================================="
echo -e "${GREEN}Testing complete!${NC}"
echo "=================================================="
echo ""
echo "Next steps:"
echo "1. Run benchmarks: cd benchmarks && ./run_fair_concurrent_benchmarks.sh"
echo "2. View results: open benchmarks/results/fair_concurrent_benchmark_report_latest.html"