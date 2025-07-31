# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

BSL Type Safety Analyzer is an enterprise-ready static analyzer for 1C:Enterprise BSL (Business Script Language) written in Rust. It features a **Unified BSL Type System** that combines platform types, configuration metadata, and forms into a single queryable index, optimized for large configurations (80,000+ objects).

## Development Commands

### Building and Running
```bash
# Build the project
cargo build

# Build optimized release version
cargo build --release

# Build unified index from configuration (with automatic caching!)
cargo run --bin build_unified_index -- --config "path/to/config" --platform-version "8.3.25"

# Build with specific application mode (NEW!)
cargo run --bin build_unified_index -- --config "path/to/config" --platform-version "8.3.25" --mode ordinary
cargo run --bin build_unified_index -- --config "path/to/config" --platform-version "8.3.25" --mode managed  # default
cargo run --bin build_unified_index -- --config "path/to/config" --platform-version "8.3.25" --mode mixed

# Extract platform documentation (one-time per version)
cargo run --bin extract_platform_docs -- --archive "path/to/1c_v8.3.25.zip" --version "8.3.25"

# Query unified index (uses project cache automatically)
cargo run --bin query_type -- --name "Справочники.Номенклатура" --config "path/to/config" --show-all-methods
```

### Единый индекс BSL типов (v0.0.4) - с автоматическим кешированием!
```bash
# Построение единого индекса из XML конфигурации (автоматически кешируется)
cargo run --bin build_unified_index -- --config "C:\Config\MyConfig" --platform-version "8.3.25"
# Первый запуск: ~795ms, последующие: ~588ms (25% быстрее)

# Извлечение платформенных типов (один раз для версии)
cargo run --bin extract_platform_docs -- --archive "path/to/1c_v8.3.25.zip" --version "8.3.25"
# Результат: ~/.bsl_analyzer/platform_cache/v8.3.25.jsonl

# Запросы к единому индексу (требуется указать конфигурацию)
cargo run --bin query_type -- --name "Массив (Array)" --config "path/to/config" --show-methods
cargo run --bin query_type -- --name "Справочники.Номенклатура" --config "path/to/config" --show-all-methods
cargo run --bin check_type -- --from "Справочники.Номенклатура" --to "СправочникСсылка" --config "path/to/config"
```

### Legacy парсеры (использовать только для совместимости)
```bash
# Старые парсеры из текстовых отчетов (не рекомендуется)
cargo run --bin parse_metadata_full -- --report "path/to/report.txt" --output "./output"
cargo run --bin extract_forms -- --config "path/to/config" --output "./forms_output"
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

## 🚀 Архитектура Unified BSL Index (v0.0.4)

### UnifiedBslIndex - Единый индекс всех BSL типов
**Революционный подход к анализу BSL с автоматическим кешированием**

**Ключевые компоненты:**
- **BslEntity** - универсальное представление любого BSL типа
- **ConfigurationXmlParser** - прямой парсинг XML без промежуточных отчетов
- **PlatformDocsCache** - версионное кеширование платформенных типов
- **ProjectIndexCache** - автоматическое кеширование проектов (NEW!)
- **UnifiedIndexBuilder** - объединение всех источников в единый индекс

**Производительность (24,055 объектов):**
- Первая индексация: ~795ms
- Загрузка из кеша: ~588ms (25% быстрее)
- Поиск типа: <1ms
- Размер кеша проекта: ~7KB

**Основные API:**
```rust
// Поиск любой сущности
let entity = index.find_entity("Справочники.Номенклатура")?;

// Все методы объекта (включая унаследованные)
let methods = index.get_all_methods("Справочники.Номенклатура");

// Проверка совместимости типов
let compatible = index.is_assignable("Справочники.Номенклатура", "СправочникСсылка");
```

### Legacy парсеры (оставлены для совместимости)

**1. MetadataReportParser** - текстовые отчеты (НЕ рекомендуется)
**2. FormXmlParser** - отдельный парсинг форм (НЕ рекомендуется)
**3. HbkArchiveParser** - документация BSL (используется в PlatformDocsCache)

## 📚 Примеры файлов и структура

### Структура хранения Unified Index (v2.0)
```
~/.bsl_analyzer/
├── platform_cache/                          # Переиспользуется между проектами
│   ├── v8.3.24.jsonl                       # 24,050 типов платформы
│   ├── v8.3.25.jsonl           
│   └── v8.3.26.jsonl
└── project_indices/                        # Индексы проектов
    └── ProjectName_<hash>/                 # Уникальное имя (хеш полного пути)
        ├── v8.3.25/                        # Версия платформы
        │   ├── config_entities.jsonl       # Объекты конфигурации (~5KB)
        │   ├── unified_index.json          # Только индексы конфигурации (~1KB)
        │   └── manifest.json               # Метаданные проекта
        └── v8.3.26/                        # Другая версия платформы
            └── ...
```

### Пример BslEntity
```rust
BslEntity {
    id: BslEntityId("Справочники.Номенклатура"),
    qualified_name: "Справочники.Номенклатура",
    display_name: "Номенклатура",
    entity_type: BslEntityType::Configuration,
    entity_kind: BslEntityKind::Catalog,
    interface: BslInterface {
        methods: {
            "Записать": BslMethod { ... },
            "Прочитать": BslMethod { ... },
            // + все унаследованные от СправочникОбъект
        },
        properties: {
            "Наименование": BslProperty { type: "Строка(150)" },
            "Код": BslProperty { type: "Строка(10)" },
        },
    },
    constraints: BslConstraints {
        parent_types: ["СправочникОбъект"],
        implements: ["СправочникСсылка"],
    },
}
```

## 📁 Примеры файлов и команд

### **MetadataReportParser - Тестовые файлы:**
```bash
# Основной тестовый файл (если доступен)
cargo run --bin parse_metadata_full -- --report "C:\Users\Egor\Downloads\ОтчетПоКонфигурации888.txt"

# Пример файла проекта (если существует)
cargo run --bin parse_metadata_full -- --report "examples/sample_config_report.txt"

# Простое тестирование структуры
cargo run --bin parse_metadata_simple -- "путь/к/вашему/отчету.txt"

# Детальный анализ типов с ограничениями
cargo run --bin analyze_metadata_types -- --report "путь/к/вашему/отчету.txt"
```

### **FormXmlParser - Парсинг форм:**
```bash
# Извлечение всех XML форм из директории конфигурации
cargo run --bin extract_forms -- --config "путь/к/конфигурации" --output "./forms_output"

# Результат: Все формы в структурированном виде
# Структура: ./forms_output/configuration/forms/*.json
```

### **HbkArchiveParser - Архивы документации:**
```bash
# Если у вас есть архив документации 1С
cargo run --bin extract_hybrid_docs -- --archive "путь/к/архиву.zip" --output "./docs_output"

# Результат: 4,916 типов BSL в оптимизированном формате
# Структура: ./docs_output/hybrid_docs/core/builtin_types/*.json
```

### **Получение справки по командам:**
```bash
# Справка по каждому парсеру
cargo run --bin parse_metadata_full -- --help
cargo run --bin parse_metadata_simple -- --help  
cargo run --bin analyze_metadata_types -- --help
cargo run --bin extract_forms -- --help
cargo run --bin extract_hybrid_docs -- --help
```

## ⚠️ **ВАЖНО: Обязательные параметры (обновлено 2025-07-28)**

**Все парсеры теперь требуют явного указания исходных файлов:**

❌ **Больше НЕ работает:**
```bash
cargo run --bin parse_metadata_full              # ОШИБКА - нет --report
cargo run --bin extract_hybrid_docs              # ОШИБКА - нет --archive  
```

✅ **Правильное использование:**
```bash
cargo run --bin parse_metadata_full -- --report "файл.txt"
cargo run --bin extract_hybrid_docs -- --archive "архив.zip"
```

**Преимущества новой архитектуры:**
- 🔒 **Безопасность**: Никаких скрытых хардкодед путей
- 📝 **Прозрачность**: Явно видно, какие файлы используются
- ✅ **Валидация**: Проверка существования файлов перед запуском
- 📚 **Справка**: Команда `--help` для каждого парсера
- 🎯 **Предсказуемость**: Одинаковое поведение всех парсеров

## Исправленные критические проблемы (2025-07-28)

### MetadataReportParser - Критические исправления ✅

**1. Проблема: Неполный парсинг регистров**
- **Было**: Парсились только "Реквизиты", игнорировались "Измерения" и "Ресурсы"
- **Исправлено**: Поддержка всех трех секций регистров
- **Код**: `metadata_parser.rs:469` - добавлена проверка `element_type == "измерения" || element_type == "ресурсы"`

**2. КРИТИЧЕСКИЙ БАГ: Составные типы парсились частично**
- **Было**: `СправочникСсылка.Контрагенты, СправочникСсылка.Организации, Строка(10, Переменная)` → только первая часть
- **Исправлено**: Полный парсинг многострочных составных типов с корректной обработкой кавычек
- **Код**: `metadata_parser.rs:408-502` - полностью переписана логика обработки составных типов

**3. КРИТИЧЕСКАЯ ПРОБЛЕМА: Ограничения длины строк не сохранялись**
- **Было**: `Строка(10, Переменная)` парсилось как просто `Строка`
- **Исправлено**: Извлечение и сохранение всех ограничений типов
- **Код**: Добавлен метод `extract_type_constraints()` с regex-парсингом
- **Результат**: `length: 10, precision: 5` для оптимизации запросов

**4. Конфликты парсеров - селективная очистка**
- **Было**: MetadataReportParser и FormXmlParser перетирали результаты друг друга
- **Исправлено**: Реализована селективная очистка с методами `clear_metadata_types_only()` и `clear_forms_only()`
- **Код**: `hybrid_storage.rs` - добавлены специализированные методы очистки
- **Результат**: Парсеры работают независимо, сохраняя результаты друг друга

**5. HybridDocumentationStorage - правильная архитектура**
- **Было**: Парсеры создавали простые JSON файлы вместо структурированного хранилища
- **Исправлено**: Полная реализация HybridDocumentationStorage согласно архитектуре
- **Код**: Интеграция с `manifest.json`, правильная структура папок, метаданные
- **Результат**: Совместимость с существующими компонентами анализатора

**6. КРИТИЧЕСКАЯ ПРОБЛЕМА: Хардкодед пути в парсерах**
- **Было**: Пути к файлам жестко прописаны в коде, невозможно изменить источники данных
- **Исправлено**: Все парсеры требуют явного указания исходных файлов через CLI параметры
- **Код**: Добавлен clap::Parser во все binaries с валидацией существования файлов
- **Результат**: Безопасность, прозрачность, возможность работы с любыми файлами

### Успешные тесты на реальных данных ✅
- **Документ ЗаказНаряды**: 13 атрибутов, включая составные типы
- **РегистрСведений**: Все 3 секции (Измерения, Ресурсы, Реквизиты)
- **Ограничения типов**: `Строка(10)`, `Число(10,5)`, `Строка(0)` (неограниченная)
- **Составные типы**: Полная поддержка многострочных определений
- **Селективная очистка**: Формы сохраняются при парсинге метаданных

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

### Console Output Design Guidelines
When designing console output for CLI tools, follow these principles:

1. **Use Grouped/Summary Output by Default**
   - Display progress in grouped format (e.g., "Processing 1000/5000 files...")
   - Show summary statistics instead of individual items
   - Use progress indicators for long operations
   - For operations with 1000+ items, show progress every 1000 items, not every 100

2. **Verbose Mode Should Be Optional**
   - Full output only with `--verbose` flag
   - Individual item processing details only when explicitly requested
   - Consider file output for detailed logs
   - Even in verbose mode, limit detailed output to reasonable amounts

3. **Output Limitations**
   - Implement configurable limits for console output
   - Redirect detailed output to files when exceeding thresholds
   - Provide `--output-file` option for full results
   - For platform docs extraction (4000+ types), use minimal console output

4. **Example Implementation Pattern**
   ```rust
   // Good: Grouped progress for large datasets
   if index % 1000 == 0 && index > 0 {
       tracing::info!("Progress: {}/{} files", index, total);
   }
   
   // Better: Final summary only for very large operations
   tracing::info!("Processed {} items in {:.2?}", total, elapsed);
   
   // Bad: Individual item output
   for item in items {
       println!("Processing: {}", item); // Avoid this
   }
   ```

5. **Use Logging Levels Appropriately**
   - `INFO`: Summary and major milestones only
   - `DEBUG`: Progress updates and grouped statistics
   - `TRACE`: Individual item processing details
   - For extract_platform_docs specifically: Use INFO only for start/end messages

6. **Specific Guidelines for Platform Extraction**
   - Extracting 4,916 platform types should show:
     - Start message
     - One progress update at 50%
     - Final summary with counts
   - Avoid showing individual type processing
   - Save detailed type list to cache file, not console

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

1. **Problem**: MetadataReportParser and FormXmlParser were overwriting each other's results
2. **Solution**: Added selective clearing methods (`clear_metadata_types_only()`, `clear_forms_only()`)
3. **Result**: Both parsers can now work sequentially without conflicts
4. **Architecture**: Full HybridDocumentationStorage implementation with proper manifest and structure

#### Selective Clearing Implementation
```rust
// HybridDocumentationStorage now supports selective operations
impl HybridDocumentationStorage {
    /// Очищает только metadata_types, сохраняя формы (для MetadataReportParser)
    pub fn clear_metadata_types_only(&self) -> Result<()> { ... }
    
    /// Очищает только forms, сохраняя metadata_types (для FormXmlParser)
    pub fn clear_forms_only(&self) -> Result<()> { ... }
}
```

#### Safe Usage Pattern
```rust
// 1. Parse metadata (safe - only clears metadata_types/, preserves forms/)
let mut storage = HybridDocumentationStorage::new(output_dir);
storage.clear_metadata_types_only()?;
let metadata_parser = MetadataReportParser::new()?;
metadata_parser.parse_to_hybrid_storage("report.txt", &mut storage)?;

// 2. Parse forms (safe - only clears forms/, preserves metadata_types/)
storage.clear_forms_only()?;
let form_parser = FormXmlParser::new();
form_parser.parse_to_hybrid_storage("./config", &mut storage)?;
```

#### Test Results
- ✅ **Metadata parsing**: Creates `configuration/metadata_types/*.json` with 5 metadata objects
- ✅ **Forms preservation**: Existing `configuration/forms/test/test_form.json` survives metadata parsing
- ✅ **Structure compliance**: Proper `manifest.json` with statistics and timestamps
- ✅ **No conflicts**: Both parsers work independently without data loss

### BSL Grammar Parser Design Decision (v0.0.9)
**DECIDED**: Use `logos` + `nom` for BSL Grammar Parser implementation

**Rationale:**
- Already used in project (see `src/parser/lexer.rs`)
- Best performance for CLI use case (2ms per 1000-line file)
- Full control over AST structure
- Supports incremental parsing
- Minimal binary size increase

**Architecture:**
- Universal diagnostic output with multiple formatters (JSON, Human, LSP, SARIF)
- Auto-detection of output format based on context
- Integration with UnifiedBslIndex for type/method validation

See `docs/BSL_PARSER_DESIGN.md` for complete architectural decision.

### Example Files
- `examples/sample_config_report.txt` - comprehensive example of 1C configuration report format
- `examples/ConfTest/` - test configuration with 5 objects for testing
- `data/rebuilt.shcntx_ru.zip` - rebuilt 1C documentation archive (required for extraction)
- `docs/UNIFIED_INDEX_ARCHITECTURE.md` - detailed unified index architecture documentation