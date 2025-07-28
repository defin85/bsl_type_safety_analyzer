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

# Extract BSL documentation to hybrid format (recommended)
cargo run --bin extract_hybrid_docs

# Extract BSL documentation to chunked format (legacy)
cargo run --bin process_all_docs
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
**COMPLETE**: Full integration of Python `1c-help-parser` functionality:

#### HBK Archive Parser (`src/docs_integration/hbk_parser_full.rs`)
- Complete 1C documentation archive (.hbk/.shcntx_ru) parsing
- ZIP-based archive reading with file extraction
- Multi-encoding support for HTML content
- Handles 51,000+ HTML documentation files

```rust
let mut parser = HbkArchiveParser::new("rebuilt.shcntx_ru.zip");
parser.open_archive()?;
let content = parser.extract_file_content("syntax/array.html");
```

#### BSL Syntax Extractor (`src/docs_integration/bsl_syntax_extractor.rs`) 
- Extracts complete BSL syntax database from documentation
- Parses methods, properties, functions, operators
- Multi-variant syntax support
- Parameter extraction with types and descriptions

```rust
let mut extractor = BslSyntaxExtractor::new(archive_path);
let database = extractor.extract_syntax_database(None)?;
// database contains: objects, methods, properties, functions, operators
```

#### Hybrid Storage Architecture (`src/docs_integration/hybrid_storage.rs`)
**NEW**: Optimized storage format for 4,916 BSL types:
- Groups types by functional categories (Collections, Database, Forms, IO, System, Web)
- Reduces from 609 chunked files to 8 structured files
- Provides fast method/property lookups via indices
- Memory-efficient runtime caching

```rust
let mut storage = HybridDocumentationStorage::new(output_dir);
storage.initialize()?;
// Direct parsing from HBK to hybrid format
extractor.extract_to_hybrid_storage(output_dir, None)?;
```

#### Storage Structure
```
output/hybrid_docs/
├── core/
│   ├── builtin_types/
│   │   ├── collections.json  # Array, Map, ValueList, etc.
│   │   ├── database.json     # Query, QueryResult, etc.
│   │   ├── forms.json        # Form, FormItems, etc.
│   │   ├── io.json          # TextReader, XMLWriter, etc.
│   │   ├── system.json      # 4,894 system types
│   │   └── web.json         # HTTPConnection, etc.
│   └── global_context.json  # Method index and metadata
└── manifest.json           # Version and statistics
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
- `forms: Vec<FormContract>` - parsed form definitions with optimized storage
- **NEW**: `docs_integration: DocsIntegration` - BSL syntax database access
- Helper methods for searching contracts by type
- Statistics tracking for integrated components
- **FIXED**: Eliminated data duplication between parsers (saved 32MB storage)

### BSL Type System Integration
The analyzer now has complete knowledge of:
- **4,916 built-in BSL types** with full method/property signatures
- Parameter types and return values for all methods
- Availability contexts (Client, Server, MobileApp, etc.)
- Deprecated methods and version information
- Multi-language support (Russian/English names)

### Usage Examples

#### Extracting BSL Documentation
```bash
# Extract to hybrid format (recommended)
cargo run --bin extract_hybrid_docs

# Extract to chunked format (legacy)
cargo run --bin process_all_docs
```

#### Accessing Type Information
```rust
// Get type definition
let array_type = storage.get_type("Массив")?;
println!("Methods: {}", array_type.methods.len());

// Find methods by name
let insert_methods = storage.find_methods("Вставить");
// Returns: ["Массив", "СписокЗначений", "ТаблицаЗначений", ...]
```

### Integration Tests
Comprehensive tests in `tests/integration_test.rs` verify:
- Metadata report parsing with realistic 1C object structures (13,872 objects)
- Form XML parsing with proper element extraction (7,227 forms)
- Enhanced Configuration loading with integrated parsers
- BSL documentation extraction and hybrid storage
- Error handling for malformed files and missing reports
- **NEW**: Parser conflict resolution and selective storage clearing

### Parser Architecture Improvements (v1.1)
**CRITICAL FIX**: Resolved parser conflicts that caused data loss:

1. **Problem**: FormXmlParser was clearing all storage data (including metadata from MetadataReportParser)
2. **Solution**: Added `clear_forms_only()` method for selective clearing
3. **Result**: Both parsers can now work sequentially without conflicts
4. **Storage optimization**: Eliminated 123MB data duplication, saved 32MB total

#### Usage Pattern
```rust
// 1. Parse metadata (safe - creates metadata_types/)
let metadata_parser = MetadataReportParser::new()?;
metadata_parser.parse_to_hybrid_storage("report.txt", &mut storage)?;

// 2. Parse forms (safe - only clears forms/, preserves metadata_types/)
let form_parser = FormXmlParser::new();
form_parser.parse_to_hybrid_storage("./config", &mut storage)?;
```

### Example Files
- `examples/sample_config_report.txt` - comprehensive example of 1C configuration report format
- `data/rebuilt.shcntx_ru.zip` - rebuilt 1C documentation archive (required for extraction)
- `docs/HYBRID_STORAGE_ARCHITECTURE.md` - detailed architecture documentation