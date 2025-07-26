# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

BSL Type Safety Analyzer is a high-performance static analyzer for 1C:Enterprise BSL (Business Script Language) written in Rust. It provides comprehensive code analysis including lexical/syntactic parsing, semantic analysis, type checking, and method verification for BSL configurations.

## Development Commands

### Building and Running
```bash
# Build the project
cargo build

# Build optimized release version
cargo build --release

# Run CLI analyzer
cargo run -- analyze --path ./test_config

# Run LSP server
cargo run --bin bsl-lsp

# Run specific binary
cargo run --bin bsl-analyzer -- --help

# Generate contracts from 1C configuration
cargo run -- generate-contracts --config-path ./config --report-path ./report.txt --output ./contracts

# Parse 1C documentation archive
cargo run -- parse-docs --hbk-path ./1C_Help.hbk --output ./docs
```

### Testing
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test analyzer::semantic_analyzer_integration_test

# Run specific test
cargo test test_method_verification
```

### Development Tools
```bash
# Check code formatting
cargo fmt --check

# Format code
cargo fmt

# Run linter
cargo clippy

# Check for issues
cargo clippy -- -D warnings
```

## Architecture Overview

The analyzer is structured in several key modules:

### Core Components
- **Parser** (`src/parser/`): Complete BSL lexer and grammar parser with AST generation
- **Analyzer** (`src/analyzer/`): Multi-phase analysis engine including semantic, lexical, and data flow analysis
- **Configuration** (`src/configuration/`): 1C configuration metadata loading and management with integrated parsers
- **Verifiers** (`src/verifiers/`): Method verification and call validation
- **LSP** (`src/lsp/`): Language Server Protocol implementation for editor integration
- **Diagnostics** (`src/diagnostics/`): Error reporting and diagnostic system
- **Documentation Integration** (`src/docs_integration/`): 1C documentation parsing and BSL syntax database

### Key Analysis Phases
1. **Lexical Analysis**: Tokenization and basic syntax validation
2. **Syntax Analysis**: AST construction with grammar validation
3. **Semantic Analysis**: Scope tracking, variable usage, type checking
4. **Method Verification**: Function/procedure call validation and compatibility
5. **Data Flow Analysis**: Variable state tracking and initialization checking

### Configuration Structure
The analyzer understands 1C:Enterprise configuration structure:
- **Modules**: BSL code files with different types (CommonModule, ObjectModule, etc.)
- **Metadata**: Configuration.xml parsing and object relationships
- **Dependencies**: Inter-module dependency tracking and circular dependency detection
- **Enhanced Parsing**: Text configuration reports and XML forms parsing
- **Documentation Integration**: .hbk archives parsing for 1C help system integration

## Development Patterns

### Error Handling
- Use `anyhow::Result<T>` for recoverable errors in most functions
- Use `thiserror` for custom error types in core components
- Collect multiple errors using `ErrorCollector` rather than failing fast

### Testing Approach
- Integration tests are placed alongside modules (e.g., `*_integration_test.rs`)
- Use realistic BSL code samples for testing
- Test both positive and negative cases for analysis rules

### Performance Considerations
- The analyzer uses `rayon` for parallel processing of multiple files
- Parsing uses `nom` combinators for efficient memory usage
- LSP implementation supports incremental parsing for editor responsiveness

### Code Organization
- Each major component is in its own module with a `mod.rs` that re-exports public APIs
- Analysis phases are designed to be composable and can run independently
- Configuration and metadata handling is centralized in the `configuration` module

## Important Implementation Notes

### BSL Language Support
- Supports both Russian and English BSL keywords
- Handles 1C:Enterprise-specific constructs like export procedures, client/server contexts
- Properly parses 1C configuration metadata (XML format)
- **NEW**: BOM (Byte Order Mark) handling for UTF-8, UTF-16LE, UTF-16BE files
- **NEW**: Multi-encoding support for BSL files (UTF-8, UTF-16, Windows-1251)
- **NEW**: Text configuration reports parsing with multi-encoding support (UTF-16, UTF-8, CP1251)
- **NEW**: XML forms parsing with complete element and attribute extraction
- **NEW**: Documentation integration with 1C help system (.hbk archives)

### Multi-threading
- File analysis can run in parallel using the configured worker count
- LSP server uses async/await with tokio runtime
- Shared state is minimized and protected with appropriate synchronization

### Memory Management
- Uses incremental parsing for LSP to avoid re-parsing entire files
- AST nodes use `Rc` for shared references to reduce memory usage
- Parser supports partial parsing for faster completion and validation

## Common Development Tasks

When adding new analysis rules:
1. Add the rule logic to the appropriate analyzer module
2. Update the `AnalysisResult` structure if new diagnostic types are needed
3. Add integration tests with realistic BSL code examples
4. Update CLI output formatting if new diagnostic categories are added

When extending BSL language support:
1. Update the lexer in `src/parser/lexer.rs` for new tokens
2. Extend the grammar in `src/parser/grammar.rs` for new syntax
3. Add corresponding AST node types in `src/parser/ast.rs`
4. Update semantic analysis to handle new constructs

When modifying configuration handling:
1. Update metadata parsing in `src/configuration/metadata.rs`
2. Extend module discovery logic in `src/configuration/modules.rs`
3. Update dependency tracking in `src/configuration/dependencies.rs`
4. Add validation rules in the main `Configuration::validate()` method

## New Integrated Parsers (Phase 1 Complete)

### Configuration Report Parser (`src/configuration/metadata_parser.rs`)
Ported from Python `onec-contract-generator` project:
- Parses text configuration reports (not XML Configuration.xml)
- Multi-encoding support: UTF-16LE, UTF-8, Windows-1251
- Extracts object metadata: directories, documents, registers, reports, etc.
- Generates typed contracts for all configuration objects

```rust
let parser = MetadataReportParser::new()?;
let contracts = parser.parse_report("config_report.txt")?;
```

### Form XML Parser (`src/configuration/form_parser.rs`)
Ported from Python `onec-contract-generator` project:
- Parses XML form files from 1C configuration structure
- Extracts form elements, attributes, commands
- Determines form types (ListForm, ItemForm, ObjectForm, etc.)
- Generates typed form contracts

```rust
let parser = FormXmlParser::new();
let forms = parser.generate_all_contracts("./config")?;
```

### Documentation Integration (`src/docs_integration/`)
Framework for integrating Python `1c-help-parser` functionality:
- HBK archive parser for 1C documentation extraction
- BSL syntax database with methods, objects, functions, properties
- Completion and help system integration (structure ready)

```rust
let mut docs = DocsIntegration::new();
// Future: docs.load_from_hbk_archive("help.hbk")?;
```

### BOM and Encoding Support (`src/parser/lexer.rs`)
Enhanced BSL file reading with proper encoding detection:
- Automatic BOM detection and removal (UTF-8, UTF-16LE, UTF-16BE)
- Multi-encoding support with fallback to Windows-1251
- Safe Unicode character boundary handling

```rust
use bsl_analyzer::parser::read_bsl_file;

// Read BSL file with automatic encoding detection and BOM handling
let content = read_bsl_file("module.bsl")?;
let lexer = BslLexer::new();
let tokens = lexer.tokenize(&content)?; // BOM automatically stripped
```

### Enhanced Configuration Module
The main `Configuration` struct now includes:
- `metadata_contracts: Vec<MetadataContract>` - parsed configuration objects
- `forms: Vec<FormContract>` - parsed form definitions
- Helper methods for searching contracts by type
- Statistics tracking for integrated components

### Integration Tests
Comprehensive tests in `tests/integration_test.rs` verify:
- Metadata report parsing with realistic 1C object structures
- Form XML parsing with proper element extraction
- Enhanced Configuration loading with integrated parsers
- Error handling for malformed files and missing reports

### Example Files
- `examples/sample_config_report.txt` - comprehensive example of 1C configuration report format