# Contributing to Cache Implementation Challenge

Thank you for your interest in contributing to this AI model comparison project!

## 🎯 Purpose

This repository serves as a benchmark and comparison of AI-generated code. Contributions should maintain this educational focus.

## 🤝 How to Contribute

### 1. Adding New AI Model Implementations

If you have access to other AI models, you can add their implementations:

1. Create a new directory: `implementations/7_modelname_language/`
2. Implement the same cache requirements
3. Document the model details in the implementation
4. Add benchmarks for the new implementation

### 2. Improving Benchmarks

- Add new benchmark scenarios in `benchmarks/`
- Improve statistical analysis methods
- Add visualization tools
- Fix bugs in existing benchmarks

### 3. Fixing Compilation Issues

GLM-4.5 currently has compilation errors. Fixes are welcome:
- Maintain the original algorithm/approach
- Document what was fixed
- Keep the innovative features intact

### 4. Documentation

- Improve clarity in guides
- Add examples
- Translate documentation
- Fix typos

## 📋 Contribution Process

1. **Fork the Repository**
   ```bash
   git clone https://github.com/[your-username]/cache-implementation-challenge
   ```

2. **Create a Feature Branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Make Your Changes**
   - Follow existing code style
   - Add tests if applicable
   - Update documentation

4. **Run Tests**
   ```bash
   # Python tests
   cd implementations/1_claude_python
   python3 smart_cache.py
   
   # Java tests
   cd implementations/2_qwen30b_java
   javac IntelligentCache.java && java IntelligentCache
   
   # Rust tests
   cd implementations/[3-6]_*_rust
   cargo test
   ```

5. **Run Benchmarks**
   ```bash
   cd benchmarks
   ./run_fair_concurrent_benchmarks.sh
   ```

6. **Commit Your Changes**
   ```bash
   git add .
   git commit -m "feat: description of your changes"
   ```

7. **Push and Create PR**
   ```bash
   git push origin feature/your-feature-name
   ```
   Then create a Pull Request on GitHub.

## 🔧 Development Setup

### Prerequisites

- Python 3.8+
- Java 11+
- Rust 1.70+
- Git

### Install Dependencies

```bash
# Python dependencies
pip install -r requirements.txt

# Rust dependencies (handled by Cargo)
cd benchmarks
cargo build --release
```

## 📊 Benchmark Guidelines

When adding or modifying benchmarks:

1. **Fairness**: Ensure all implementations process identical workloads
2. **Reproducibility**: Use fixed seeds for random operations
3. **Statistical Rigor**: Include confidence intervals and outlier detection
4. **Documentation**: Explain what each benchmark measures

## 🐛 Reporting Issues

Please use GitHub Issues to report:
- Benchmark failures
- Compilation errors
- Documentation problems
- Performance anomalies

Include:
- System specifications
- Error messages
- Steps to reproduce

## 💡 Improvement Ideas

Areas where contributions are especially welcome:

1. **Async Implementations**: None of the models used async/await
2. **Distributed Cache**: Extend to multi-node scenarios
3. **Additional Languages**: Go, C++, Julia implementations
4. **Visualization**: Better charts and graphs for results
5. **CI/CD**: Automated testing and benchmarking

## 📝 Code Style

### Python
- Follow PEP 8
- Use type hints
- Document with docstrings

### Java
- Follow Oracle Java conventions
- Use meaningful variable names
- Comment complex logic

### Rust
- Follow Rust API guidelines
- Use `cargo fmt` and `cargo clippy`
- Document public APIs

## ⚖️ Legal

By contributing, you agree that your contributions will be licensed under the MIT License.

## 🙏 Acknowledgments

Contributors will be acknowledged in the README and in the Git history.

## 📧 Contact

For questions about contributing, please open a GitHub Issue.

---

*Remember: This project showcases AI capabilities. Keep the original AI-generated structure intact when possible, documenting any necessary fixes.*