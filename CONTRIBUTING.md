# Contributing to BSL Type Safety Analyzer

Thank you for your interest in contributing to BSL Type Safety Analyzer! This document provides guidelines and information for contributors.

## 🚀 Getting Started

### Prerequisites

- **Rust 1.70+** with cargo
- **Git** for version control
- **Basic knowledge** of BSL (1C:Enterprise) language
- **Familiarity** with static analysis concepts

### Development Setup

1. **Fork and Clone**
   ```bash
   git clone https://github.com/your-username/bsl-analyzer.git
   cd bsl-analyzer
   ```

2. **Install Dependencies**
   ```bash
   cargo build
   cargo test
   ```

3. **Verify Installation**
   ```bash
   cargo run -- --help
   cargo run -- analyze examples/
   ```

## 📋 How to Contribute

### Types of Contributions

1. **Bug Reports** - Help us identify and fix issues
2. **Feature Requests** - Suggest new functionality
3. **Code Contributions** - Implement features or fix bugs
4. **Documentation** - Improve guides, examples, and API docs
5. **Testing** - Add test cases and improve coverage
6. **Performance** - Optimize algorithms and data structures

### Reporting Issues

When reporting bugs, please include:

- **BSL Analyzer version** (`bsl-analyzer --version`)
- **Operating system** and version
- **Rust version** (`rustc --version`)
- **Minimal example** that reproduces the issue
- **Expected vs actual behavior**
- **Error messages** or logs if applicable

Use our issue templates:
- [Bug Report](https://github.com/your-org/bsl-analyzer/issues/new?template=bug_report.md)
- [Feature Request](https://github.com/your-org/bsl-analyzer/issues/new?template=feature_request.md)

### Code Contributions

#### Development Workflow

1. **Create Feature Branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make Changes**
   - Follow the coding standards below
   - Add tests for new functionality
   - Update documentation as needed

3. **Test Changes**
   ```bash
   cargo test
   cargo clippy
   cargo fmt --check
   ```

4. **Commit Changes**
   ```bash
   git add .
   git commit -m "feat: add awesome new feature"
   ```

5. **Push and Create PR**
   ```bash
   git push origin feature/your-feature-name
   # Open PR on GitHub
   ```

#### Coding Standards

##### Code Style
- **Rust formatting**: Use `cargo fmt` (rustfmt)
- **Linting**: Pass `cargo clippy` without warnings
- **Documentation**: Document public APIs with `///` comments
- **Error handling**: Use `anyhow::Result` for error propagation
- **Naming**: Follow Rust naming conventions (snake_case, PascalCase)

##### Architecture Principles
- **Modularity**: Keep modules focused and cohesive
- **Performance**: Optimize for common use cases
- **Safety**: Leverage Rust's type system for correctness
- **Testability**: Write testable code with dependency injection
- **Configurability**: Make behavior configurable where appropriate

##### Code Examples

**Good:**
```rust
/// Analyzes BSL code for type safety issues
pub struct BslAnalyzer {
    config: AnalyzerConfig,
    rules: RulesEngine,
}

impl BslAnalyzer {
    /// Creates a new analyzer with the given configuration
    pub fn new(config: AnalyzerConfig) -> Self {
        Self {
            config,
            rules: RulesEngine::new(config.rules.clone()),
        }
    }
    
    /// Analyzes the provided AST and returns analysis results
    pub fn analyze(&mut self, ast: &BslAst) -> anyhow::Result<AnalysisResults> {
        self.rules.apply_rules(ast)
    }
}
```

**Avoid:**
```rust
// No documentation, unclear error handling
pub fn analyze_code(code: String) -> Option<Vec<String>> {
    // Implementation...
}
```

#### Testing Guidelines

##### Test Structure
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_analyze_valid_bsl_code() {
        let analyzer = BslAnalyzer::new(AnalyzerConfig::default());
        let ast = parse_bsl("Процедура Тест() КонецПроцедуры").unwrap();
        
        let results = analyzer.analyze(&ast).unwrap();
        assert_eq!(results.error_count(), 0);
    }
    
    #[test]
    fn test_analyze_invalid_bsl_code() {
        let analyzer = BslAnalyzer::new(AnalyzerConfig::default());
        let ast = parse_bsl("НеопределеннаяПеременная = 1;").unwrap();
        
        let results = analyzer.analyze(&ast).unwrap();
        assert!(results.error_count() > 0);
    }
}
```

##### Test Categories
- **Unit tests**: Test individual functions and methods
- **Integration tests**: Test module interactions
- **Regression tests**: Prevent known bugs from reoccurring
- **Performance tests**: Ensure acceptable performance characteristics

#### Documentation

##### API Documentation
- **Public functions**: Must have `///` documentation
- **Examples**: Include usage examples in doc comments
- **Errors**: Document possible error conditions
- **Safety**: Document unsafe code and invariants

##### README Updates
- Update feature lists for new functionality
- Add new CLI commands and options
- Update performance benchmarks if applicable

##### Changelog
- Follow [Keep a Changelog](https://keepachangelog.com/) format
- Categorize changes: Added, Changed, Deprecated, Removed, Fixed, Security

## 🏗️ Project Structure

```
src/
├── analyzer/           # Semantic analysis engine
│   ├── mod.rs         # Main analyzer interface
│   ├── engine.rs      # Analysis engine implementation
│   └── scope.rs       # Scope tracking
├── cache/             # Performance optimization
├── configuration/     # 1C metadata parsing
├── core/              # Core types and utilities
├── diagnostics/       # Error reporting
├── lsp/               # Language Server Protocol
├── metrics/           # Code quality metrics
├── parser/            # BSL parsing
├── reports/           # Output formatting
├── rules/             # Configurable rules
└── verifiers/         # Type and method verification
```

### Key Components

#### Parser (`src/parser/`)
- **Lexer**: Tokenization of BSL source code
- **Grammar**: BSL language grammar definitions
- **AST**: Abstract Syntax Tree structures

#### Analyzer (`src/analyzer/`)
- **Engine**: Main analysis orchestration
- **Scope**: Variable and function scope tracking
- **Flow**: Data flow analysis

#### Rules (`src/rules/`)
- **Engine**: Rule execution framework
- **Builtin**: Standard BSL rules (BSL001-BSL010)
- **Custom**: User-defined rule support
- **Config**: Rule configuration management

## 🧪 Testing

### Running Tests

```bash
# All tests
cargo test

# Specific module
cargo test analyzer

# With coverage (requires cargo-tarpaulin)
cargo install cargo-tarpaulin
cargo tarpaulin --out html

# Performance tests
cargo test --release performance
```

### Test Data

Test files are located in:
- `tests/fixtures/` - BSL code samples
- `tests/configs/` - Configuration files
- `tests/expected/` - Expected analysis results

### Adding Test Cases

1. **Create test BSL file** in `tests/fixtures/`
2. **Add test function** in appropriate test module
3. **Document test purpose** and expected behavior
4. **Update test counts** if adding integration tests

## 📊 Performance Considerations

### Optimization Guidelines

1. **Profile before optimizing** - Use `cargo flamegraph` or similar tools
2. **Benchmark changes** - Ensure improvements don't regress performance
3. **Memory efficiency** - Minimize allocations in hot paths
4. **Parallelization** - Use `rayon` for CPU-intensive operations
5. **Caching** - Cache expensive computations appropriately

### Performance Testing

```bash
# Benchmark suite
cargo bench

# Memory profiling
cargo run --features dhat-heap -- analyze large-project/

# CPU profiling
cargo flamegraph -- analyze large-project/
```

## 🤝 Community Guidelines

### Code of Conduct

We follow the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct). Please be respectful and inclusive in all interactions.

### Communication Channels

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: General questions and community discussion
- **Pull Requests**: Code review and collaboration

### Review Process

1. **Automated checks** must pass (CI, tests, linting)
2. **Code review** by at least one maintainer
3. **Documentation review** for user-facing changes
4. **Performance review** for performance-critical changes

## 🏆 Recognition

Contributors are recognized in:
- **CHANGELOG.md** - Major contributions
- **README.md** - Long-term contributors
- **GitHub releases** - Per-release contributors

## 📚 Resources

### BSL Language Resources
- [1C:Enterprise Documentation](https://its.1c.ru/db/metod8dev)
- [BSL Language Specification](https://its.1c.ru/db/bsl)
- [1C Community Forum](https://forum.1c-dev.ru/)

### Rust Resources
- [The Rust Programming Language](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Cargo Book](https://doc.rust-lang.org/cargo/)

### Static Analysis Resources
- [SARIF Specification](https://sarifweb.azurewebsites.net/)
- [Language Server Protocol](https://microsoft.github.io/language-server-protocol/)
- [Compiler Design Patterns](https://craftinginterpreters.com/)

## ❓ Getting Help

If you need help contributing:

1. **Check existing issues** and discussions
2. **Review documentation** and examples
3. **Ask questions** in GitHub Discussions
4. **Join community channels** for real-time help

Thank you for helping make BSL Type Safety Analyzer better! 🚀