# BSL Type Safety Analyzer v0.0.2-alpha

**Static analyzer for 1C:Enterprise BSL language with integrated metadata parsers**

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)]()
[![Test Coverage](https://img.shields.io/badge/coverage-40%25-yellow)]()
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.70+-orange.svg)]()
[![Development Stage](https://img.shields.io/badge/stage-alpha-orange)]()
[![Parsers](https://img.shields.io/badge/parsers-production%20ready-green)]()

High-performance static analyzer for 1C:Enterprise BSL written in Rust. Currently focuses on comprehensive metadata parsing and BSL documentation integration. **Early development stage** - core BSL code analysis is not yet implemented.

## ⚠️ Project Status: Alpha Development

**Current Version**: v0.0.2-alpha (~25-30% complete)  
**Production Ready**: ❌ Not ready for BSL code analysis  
**Metadata Parsers**: ✅ Production-ready after recent refactoring  
**BSL Documentation**: ✅ Complete integration (4,916 types)  

### What Works Now:
- ✅ **1C Metadata Parsing** - Real configuration reports and XML forms
- ✅ **BSL Documentation Integration** - Complete type system with 4,916 built-in types
- ✅ **HBK Archive Parser** - 1C documentation extraction
- ✅ **Configuration Analysis** - Object structure and relationships
- ✅ **CLI Tools** - Comprehensive command-line interface

### What Doesn't Work Yet:
- ❌ **BSL Code Parsing** - Core grammar parser not implemented
- ❌ **Semantic Analysis** - Code analysis features are stubs
- ❌ **LSP Server** - Limited functionality without parser
- ❌ **Rules System** - Infrastructure only, no real rules

## 🚀 Current Features (Working)

### 1C Metadata Integration
- **MetadataReportParser** - Parses text configuration reports with full type support
- **FormXmlParser** - Extracts form structure from XML files (separate tool)
- **HBK Archive Parser** - Direct 1C documentation processing
- **Hybrid Storage** - Optimized format for BSL type information

### BSL Documentation System
- **Complete Type Database** - 4,916 BSL types with method signatures
- **Multi-language Support** - Russian/English names and descriptions
- **Method Index** - Fast lookup across all types and categories
- **Optimized Storage** - 8 structured files instead of 609 chunks

### CLI Tools
- **Configuration Analysis** - Parse and analyze 1C configurations
- **Documentation Extraction** - Build BSL type database from archives  
- **Metadata Contracts** - Generate typed contracts from real data
- **Forms Extraction** - Parse XML forms from configuration directory

## 📦 Installation

### From Source (Development)
```bash
# Clone the repository (adjust path as needed)
git clone /path/to/bsl_type_safety_analyzer.git
cd bsl_type_safety_analyzer
cargo build --release
```

### Quick Test
```bash
# Test metadata parsing on sample configuration (requires report file)
cargo run --bin parse_metadata_full -- --report "path/to/report.txt"

# Extract BSL documentation (requires 1C help archives)
cargo run --bin extract_hybrid_docs -- --archive "path/to/archive.zip"
```

## 🔧 Quick Start

### 1. Parse 1C Configuration Metadata
```bash
# Parse configuration report to structured format
cargo run --bin parse_metadata_simple -- "path/to/config_report.txt"

# Full parsing with hybrid storage
cargo run --bin parse_metadata_full -- --report "path/to/config_report.txt" --output "./metadata_output"
```

### 2. Extract BSL Documentation
```bash
# Extract complete BSL type system from 1C documentation
cargo run --bin extract_hybrid_docs -- --archive "path/to/hbk_archive.zip" --output "./docs_output"

# Results: ./docs_output/core/builtin_types/*.json
```

### 3. Analyze Configuration Structure
```bash
# Generate contracts from real 1C metadata with detailed type analysis
cargo run --bin analyze_metadata_types -- --report "path/to/config_report.txt"
```

### 4. Parse XML Forms Separately  
```bash
# Extract all forms from 1C configuration directory
cargo run --bin extract_forms -- --config "path/to/config_directory" --output "./forms_output"

# Note: Forms must be parsed separately - they are NOT included in parse_metadata_full
```

## 📋 Real-World Testing

The parsers have been successfully tested on large 1C configurations:

### Metadata Parser Results
- ✅ **14+ metadata objects** parsed from real configuration reports
- ✅ **Complex composite types** - Full support for multi-line type definitions
- ✅ **All register sections** - Measurements, Resources, and Attributes
- ✅ **Type constraints** - String lengths, number precision preserved
- ✅ **UTF-16LE encoding** - Proper handling of 1C report format

### Form Parser Results  
- ✅ **7,220+ XML forms** processed from production configurations
- ✅ **All element types** - Tables, inputs, commands, etc.
- ✅ **Complete structure** - DataPath, events, attributes extracted
- ✅ **Form classification** - ListForm, ItemForm, ObjectForm detection

### BSL Documentation
- ✅ **4,916 BSL types** extracted from official 1C documentation
- ✅ **Complete method signatures** - Parameters, return types, contexts
- ✅ **Multi-language support** - Russian/English method names
- ✅ **Optimized storage** - Fast runtime access to type information

## 🏗️ Architecture Overview

```text
BSL Analyzer v0.0.2-alpha
├── 🟢 Parser (Lexer)     - BSL tokenization (working)
├── 🔴 Parser (Grammar)   - BSL AST construction (NOT IMPLEMENTED)
├── 🟢 Configuration      - 1C metadata parsing (working)
│   ├── MetadataParser    - Text reports → structured data
│   ├── FormParser        - XML forms → contracts (standalone only)
│   └── Dependencies      - Module relationships (stub)
├── 🟢 Docs Integration   - BSL documentation system (working)
│   ├── HBK Parser        - Archive extraction
│   ├── Syntax Extractor  - HTML → BSL signatures
│   └── Hybrid Storage    - Optimized type database
├── 🔴 Analyzer           - Semantic analysis (NOT IMPLEMENTED)
├── 🔴 Rules              - Analysis rules (infrastructure only)
├── 🔴 LSP                - Language server (stub)
└── 🟢 CLI                - Command-line tools (working)
    ├── parse_metadata_full      - Full metadata parsing (reports only)
    ├── parse_metadata_simple    - Quick metadata check  
    ├── analyze_metadata_types   - Detailed type analysis
    └── extract_forms           - Standalone forms extraction
```

**Legend**: 🟢 Working | 🔴 Not Implemented | 🟡 Partial

## 🛠️ Development Commands

### Building and Testing
```bash
# Build project
cargo build

# Run all tests
cargo test

# Format and lint
cargo fmt
cargo clippy
```

### Parser Testing
```bash
# Test metadata parser with sample data
cargo run --bin parse_metadata_simple -- "examples/sample_config_report.txt"

# Test with detailed type analysis
cargo run --bin analyze_metadata_types -- --report "examples/sample_config_report.txt"

# Full integration test
cargo run --bin parse_metadata_full -- --report "examples/sample_config_report.txt" --output "./test_output"

# Test forms extraction
cargo run --bin extract_forms -- --config "path/to/config_directory" --output "./forms_test"
```

### Documentation Extraction
```bash
# Extract BSL documentation to hybrid format
cargo run --bin extract_hybrid_docs -- --archive "path/to/hbk_archive.zip" --output "./docs_output"
```

## 📊 Recent Critical Fixes (2025-07-28)

### MetadataReportParser Improvements ✅
1. **Register Parsing** - Fixed incomplete parsing (now supports Measurements, Resources, Attributes)
2. **Composite Types** - Fixed multi-line type parsing: `СправочникСсылка.Контрагенты, СправочникСсылка.Организации, Строка(10, Переменная)`  
3. **Type Constraints** - Added string length and number precision extraction
4. **Selective Clearing** - Parsers no longer overwrite each other's results  
5. **HybridDocumentationStorage** - Proper architecture implementation
6. **🔒 CRITICAL: Hardcoded Paths Removed** - All parsers now require explicit file paths via CLI parameters

### CLI Architecture Overhaul ✅
**❌ Old (Insecure):**
```bash
cargo run --bin parse_metadata_full              # Used hardcoded paths
cargo run --bin extract_hybrid_docs              # Files location was hidden
```

**✅ New (Secure & Transparent):**
```bash
cargo run --bin parse_metadata_full -- --report "path/to/file.txt" --output "./output"
cargo run --bin extract_hybrid_docs -- --archive "path/to/archive.zip" --output "./docs"
```

**Benefits:**
- 🔒 **Security**: No hidden hardcoded file paths
- 📝 **Transparency**: Explicit source file specification  
- ✅ **Validation**: File existence checks before processing
- 📚 **Help**: Built-in `--help` for all parsers

### Test Results ✅
- **Document "ЗаказНаряды"**: 13 attributes including composite types parsed correctly
- **Register "ТестовыйРегистрСведений"**: All 3 sections (Measurements, Resources, Attributes) extracted
- **Type Constraints**: `Строка(10)`, `Число(10,5)`, `Строка(0)` properly handled
- **Form Preservation**: Selective clearing prevents data loss between parsers
- **CLI Security**: All parsers validate input files and provide clear error messages

## 🎯 Roadmap & Next Steps

### Critical Path (Required for BSL Analysis):
1. **Implement BSL Grammar Parser** (~2-3 weeks)
   - Full BSL language grammar
   - AST construction from tokens  
   - Error recovery and reporting

2. **Basic Semantic Analysis** (~1-2 weeks)
   - Scope resolution
   - Variable tracking
   - Basic type checking

3. **Export/Import Extraction** (~1 week)
   - Parse module exports
   - Build method signatures

### Future Enhancements:
4. **Inter-module Analysis** - Dependency graphs and call validation
5. **Rules System** - Configurable analysis rules
6. **LSP Server** - Real editor integration
7. **SARIF Export** - CI/CD integration

**Realistic Timeline**: MVP with basic BSL analysis in 2-3 months

## 💡 Current Value Proposition

While BSL code analysis is not yet implemented, the project already provides significant value:

1. **Production-Ready Metadata Parsers** - Handle real 1C configuration data
2. **Complete BSL Type System** - 4,916 types with full signatures
3. **Documentation Integration** - Optimized access to 1C help system
4. **Excellent Foundation** - Well-structured Rust codebase for future development
5. **LLM Context Generation** - Generate rich metadata for AI-powered tools

## 🤝 Contributing

This project is in active development. Contributions are welcome, especially:

- BSL grammar parser implementation
- Semantic analysis improvements
- Additional metadata parser features
- Documentation and examples

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 📞 Support & Documentation

- **Architecture Details**: See `CLAUDE.md` for comprehensive development guidance
- **Development Roadmap**: See `ROADMAP.md` for detailed project status
- **Issues**: Contact maintainers for bug reports and feature requests

---

**Note**: This is an alpha release focused on metadata parsing and documentation integration. Full BSL code analysis capabilities are planned for future releases.