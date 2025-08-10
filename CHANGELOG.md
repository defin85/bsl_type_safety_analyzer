# Changelog

All notable changes to the BSL Type Safety Analyzer project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### � Deprecated
- Legacy semantic analyzer path (pre-arena) marked for removal in next minor release; migrate to arena-based `SemanticArena` API.

### ✨ Added
- Arena semantic: precise spans for New/Binary/Unary, operator token spans normalized.
- Prototype expression type side-table (`expr_types`) enabling future incremental & interning phases.
- Snapshot diagnostics harness and parity tests broadened (control-flow, duplicates).

### 🛠 Changed
- `SemanticArena` now primary; legacy semantic code frozen (no further feature backports).

### 📦 Internal
- Docs: Updated `AST_REDESIGN_SUMMARY.md` to reflect Phase 3 completion criteria and next steps for Phase 4 (interning & symbol table).

### �🔥 Removed
- Legacy `parser` module (`src/parser`) fully deleted (legacy lexer/AST/grammar). The modern tree-sitter based implementation in `bsl_parser` is now the only parser.
- Public re-export of `BslLexer` (legacy shim) removed.

### ✨ Added
- `core::fs_utils::read_bsl_file` – BOM‑aware file reading helper replacing the old `parser::lexer::read_bsl_file`.

### 🛠 Changed
- BOM handling tests now use `core::read_bsl_file` and no longer depend on a legacy lexer.
- `mcp_server` syntax validation now uses a lightweight heuristic placeholder (previous lexer-based quick tokenization removed). A future update will integrate the new parser for richer incremental validation.

### ⚠️ Breaking Changes
- Any external code depending on `bsl_analyzer::parser::*` (especially `BslLexer`) must migrate:
	- Replace `use bsl_analyzer::parser::lexer::read_bsl_file` with `use bsl_analyzer::core::read_bsl_file`.
	- Remove usages of `BslLexer`; if tokenization is still needed, integrate with the new `bsl_parser` APIs (planned public token/AST surface) or implement a project-specific lightweight tokenizer.

### Migration Guide
1. Search your codebase for `bsl_analyzer::parser` imports and update as per above.
2. If you had tests asserting legacy token sequences, refocus them on higher-level parse/semantic behaviors using the new analyzer API.
3. For quick BOM-safe reads: `let content = bsl_analyzer::core::read_bsl_file(path)?;`.

---

## [1.0.0] - 2025-07-25

### 🎉 Initial Production Release

This is the first production-ready release of BSL Type Safety Analyzer, a complete rewrite in Rust of the Python-based BSL analyzer with significant performance improvements and enterprise features.

### ✨ Added

#### Core Analysis Engine
- **Complete BSL parser** with comprehensive token support and AST generation
- **Extended grammar support** including try-except blocks, type annotations, and async/await constructs
- **Semantic analysis** with scope tracking, variable usage patterns, and type inference
- **Method verification** with parameter compatibility and return type checking
- **Configuration-aware analysis** with 1C metadata contract integration
- **Inter-module dependency analysis** with circular dependency detection

#### Enterprise Features
- **Configurable Rules System** with 10 built-in rules (BSL001-BSL010)
- **TOML/YAML configuration** support with rule profiles and inheritance
- **Custom rules support** with regex patterns and advanced conditions
- **Code quality metrics** including cyclomatic complexity, cognitive complexity, and maintainability index
- **Technical debt analysis** with severity levels and estimated fix times
- **Intelligent recommendations** engine based on analysis results

#### LSP Server Integration
- **TCP LSP server** supporting up to 10 concurrent connections
- **STDIO LSP mode** for direct editor integration
- **Incremental analysis** with performance optimizations
- **Real-time diagnostics** with configurable update intervals
- **Intelligent completions** with BSL syntax database integration

#### Reporting & Export
- **SARIF export** for seamless CI/CD integration
- **HTML reports** with interactive metrics and recommendations
- **JSON/Text output** formats for automation and scripting
- **Performance metrics** tracking and optimization insights

#### Performance & Scalability
- **10-20x performance improvement** over Python-based analyzers
- **True parallelism** without GIL limitations
- **Intelligent caching system** for incremental analysis
- **Memory-efficient processing** with zero-copy string operations
- **Configurable threading** with automatic CPU core detection

#### CLI Interface
- **Comprehensive command-line interface** with colored output and progress indicators
- **Analysis commands** with flexible output formats and filtering options
- **Rules management** with listing, configuration generation, and validation
- **Metrics analysis** with focus areas and threshold-based reporting
- **Cache management** with statistics and cleanup utilities

### 🏗️ Architecture

#### Module Structure
```
src/
├── analyzer/           - Semantic analysis engine
├── cache/             - Performance optimization layer
├── configuration/     - 1C metadata and contract parsing
├── core/              - Type system and error handling
├── diagnostics/       - Error reporting and suggestions
├── lsp/               - Language Server Protocol implementation
├── metrics/           - Code quality and technical debt analysis
├── parser/            - Extended BSL lexer and grammar
├── reports/           - Multi-format reporting system
├── rules/             - Configurable rules engine
└── verifiers/         - Method and type verification
```

#### Built-in Rules
- **BSL001**: Unused variable detection
- **BSL002**: Undefined variable analysis
- **BSL003**: Type mismatch identification
- **BSL004**: Unknown method detection
- **BSL005**: Circular dependency analysis
- **BSL006**: Dead code detection
- **BSL007**: Complex function analysis
- **BSL008**: Missing documentation detection
- **BSL009**: Performance anti-pattern identification
- **BSL010**: Security vulnerability detection

#### Metrics Analysis
- **Cyclomatic Complexity**: Traditional complexity measurement
- **Cognitive Complexity**: Human-focused complexity analysis
- **Maintainability Index**: Overall code maintainability score (0-100)
- **Technical Debt**: Estimated fix time with severity categorization
- **Code Duplication**: Duplicate code block detection
- **Documentation Coverage**: Function/module documentation analysis

### 🚀 Performance Benchmarks

- **10-20x faster** than equivalent Python analyzers
- **Parallel processing** utilizing all available CPU cores
- **Memory efficient** with optimized string processing
- **Scalable architecture** supporting large enterprise codebases
- **Intelligent caching** reducing re-analysis overhead by up to 80%

### 📋 Configuration Support

#### Rules Configuration
- **TOML/YAML support** with comprehensive schema validation
- **Rule profiles** (default, strict, lenient) with inheritance
- **Custom severity levels** (error, warning, info, hint)
- **Confidence thresholds** and rule-specific parameters
- **Tag-based filtering** and rule categorization

#### LSP Configuration  
- **TCP/STDIO modes** with configurable endpoints
- **Connection management** with timeout and retry logic
- **Analysis settings** with incremental processing options
- **Performance tuning** with thread and memory limits

### 🔧 Development Tools

#### CLI Commands
```bash
# Analysis
bsl-analyzer analyze ./src --format sarif --output results.sarif
bsl-analyzer metrics ./src --report-format html --output metrics.html

# Rules Management
bsl-analyzer rules list
bsl-analyzer rules generate-config --output bsl-rules.toml

# LSP Server
bsl-analyzer lsp --mode tcp --host 127.0.0.1 --port 9257
bsl-analyzer lsp --mode stdio

# Cache Management
bsl-analyzer cache info
bsl-analyzer cache clean
```

#### Integration Examples
- **GitHub Actions** workflow for automated analysis
- **Jenkins pipeline** integration with SARIF reporting
- **VS Code extension** configuration templates
- **Vim/Neovim LSP** setup documentation

### 📊 Quality Assurance

- **85%+ test coverage** with comprehensive unit and integration tests
- **Memory safety** guaranteed by Rust's type system
- **Thread safety** with careful concurrent programming patterns
- **Error handling** with detailed context and recovery strategies
- **Performance testing** with benchmark suite and regression detection

### 🌟 Notable Features

- **Zero-configuration startup** with sensible defaults
- **Extensive documentation** with examples and best practices  
- **Enterprise-ready** with production deployment guides
- **Community-driven** with open contribution model
- **Backwards compatible** with existing BSL codebases

### 🔧 Technical Specifications

#### System Requirements
- **Rust 1.70+** for compilation
- **4GB RAM minimum** (8GB recommended for large projects)
- **Multi-core CPU** recommended for optimal performance
- **Windows/Linux/macOS** cross-platform support

#### Dependencies
- **Core**: tokio (async runtime), serde (serialization), anyhow (error handling)
- **Parsing**: nom (parser combinators), logos (lexical analysis)
- **LSP**: tower-lsp (Language Server Protocol implementation)
- **CLI**: clap (command-line parsing), console (terminal utilities)
- **Performance**: rayon (data parallelism), num_cpus (CPU detection)

### 🎯 Target Users

- **1C:Enterprise developers** seeking code quality improvements
- **DevOps engineers** integrating static analysis into CI/CD pipelines
- **Technical leads** establishing code quality standards
- **IDE developers** adding BSL language support
- **Open source contributors** extending BSL tooling ecosystem

---

**Full release notes and migration guides available in the [documentation](https://github.com/your-org/bsl-analyzer/wiki).**