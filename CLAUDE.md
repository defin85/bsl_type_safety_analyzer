# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

BSL Type Safety Analyzer is an enterprise-ready static analyzer for 1C:Enterprise BSL (Business Script Language) written in Rust. It features a **Unified BSL Type System** that combines platform types, configuration metadata, and forms into a single queryable index, optimized for large configurations (80,000+ objects).

## Development Commands

### Building and Running

<commands>
  <command-group name="Building">
    <command>
      <description>Build the project</description>
      <code>cargo build</code>
    </command>
    <command>
      <description>Build optimized release version</description>
      <code>cargo build --release</code>
    </command>
  </command-group>

  <command-group name="Unified Index Operations">
    <command>
      <description>Build unified index from configuration (with automatic caching!)</description>
      <code>cargo run --bin build_unified_index -- --config "path/to/config" --platform-version "8.3.25"</code>
      <performance>
        <first-run>~795ms</first-run>
        <cached>~588ms (25% faster)</cached>
      </performance>
    </command>
    
    <command>
      <description>Build with specific application mode</description>
      <variants>
        <variant mode="ordinary">cargo run --bin build_unified_index -- --config "path/to/config" --platform-version "8.3.25" --mode ordinary</variant>
        <variant mode="managed" default="true">cargo run --bin build_unified_index -- --config "path/to/config" --platform-version "8.3.25" --mode managed</variant>
        <variant mode="mixed">cargo run --bin build_unified_index -- --config "path/to/config" --platform-version "8.3.25" --mode mixed</variant>
      </variants>
    </command>
  </command-group>

  <command-group name="Platform Documentation">
    <command>
      <description>Extract platform documentation (one-time per version)</description>
      <code>cargo run --bin extract_platform_docs -- --archive "path/to/1c_v8.3.25.zip" --version "8.3.25"</code>
      <result>~/.bsl_analyzer/platform_cache/v8.3.25.jsonl</result>
    </command>
  </command-group>

  <command-group name="Type Queries">
    <command>
      <description>Query unified index (uses project cache automatically)</description>
      <code>cargo run --bin query_type -- --name "Справочники.Номенклатура" --config "path/to/config" --show-all-methods</code>
    </command>
  </command-group>
</commands>

### Единый индекс BSL типов (v0.0.4) - с автоматическим кешированием!

<examples>
  <example-group name="Index Building">
    <example>
      <description>Построение единого индекса из XML конфигурации (автоматически кешируется)</description>
      <code>cargo run --bin build_unified_index -- --config "C:\Config\MyConfig" --platform-version "8.3.25"</code>
      <performance>
        <first-run>~795ms</first-run>
        <cached>~588ms (25% быстрее)</cached>
      </performance>
    </example>
    
    <example>
      <description>Извлечение платформенных типов (один раз для версии)</description>
      <code>cargo run --bin extract_platform_docs -- --archive "path/to/1c_v8.3.25.zip" --version "8.3.25"</code>
      <result>~/.bsl_analyzer/platform_cache/v8.3.25.jsonl</result>
    </example>
  </example-group>

  <example-group name="Type Queries">
    <example>
      <description>Поиск платформенного типа</description>
      <code>cargo run --bin query_type -- --name "Массив (Array)" --config "path/to/config" --show-methods</code>
      <expected-output>
Найден тип: Массив (Array)
Тип: Platform
Методы: 15
- Вставить(Индекс: Число, Значение: Произвольный)
- Добавить(Значение: Произвольный)
- Найти(Значение: Произвольный): Число
...
      </expected-output>
    </example>
    
    <example>
      <description>Поиск объекта конфигурации со всеми методами</description>
      <code>cargo run --bin query_type -- --name "Справочники.Номенклатура" --config "path/to/config" --show-all-methods</code>
      <expected-output>
Найден тип: Справочники.Номенклатура
Тип: Configuration
Всего методов (включая унаследованные): 45
Собственные методы: 3
Унаследованные методы: 42 (от СправочникОбъект, ОбъектБД)
      </expected-output>
    </example>
    
    <example>
      <description>Проверка совместимости типов</description>
      <code>cargo run --bin check_type -- --from "Справочники.Номенклатура" --to "СправочникСсылка" --config "path/to/config"</code>
      <expected-output>
✓ Тип "Справочники.Номенклатура" совместим с "СправочникСсылка"
Путь наследования: Справочники.Номенклатура → реализует → СправочникСсылка
      </expected-output>
    </example>
  </example-group>
</examples>

### Интеграция с IDE и внешними инструментами
```bash
# MCP сервер для Claude/GPT
cargo run --bin mcp_server

# Базовый LSP сервер (в разработке)
cargo run --bin lsp_server
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

## 🚀 Архитектура BSL Type Safety Analyzer v1.2

### 📋 Актуальная архитектура (основано на docs/)

**Источники:** `docs/CURRENT_DECISIONS.md`, `docs/01-overview/unified-concept.md`  
**Дата:** 2025-08-01 (последние архитектурные решения)

#### Core + Shell - Двухуровневая система
```
┌─────────────────────────────────────────────────────────┐
│                   BSL Analyzer v1.2                      │
├─────────────────────────┬───────────────────────────────┤
│      Core (Heavy)       │        Shell (Light)          │
│  LLM-oriented          │      Developer-oriented       │
├─────────────────────────┼───────────────────────────────┤
│  • UnifiedBslIndex      │  • CLI валидатор              │
│  • 500MB+ в памяти ОК   │  • <50ms старт, <10MB памяти  │
│  • MCP Server           │  • tree-sitter парсер         │
│  • База знаний паттернов│  • Offline режим с кешом      │
│  • Семантический граф   │  • Human-friendly вывод       │
└─────────────────────────┴───────────────────────────────┘
```

### 🔧 Ключевые технологические решения

**1. BSL Parser:** `tree-sitter` (НЕ logos+nom) - решение от 2025-08-01  
**2. Архитектура:** Core + Shell (НЕ монолит)  
**3. Приоритет:** LLM-first подход  
**4. Storage:** UnifiedBslIndex как главное хранилище  
**5. Парсер документации:** HBK архивы → BSL синтаксис база знаний

### UnifiedBslIndex - Единый индекс всех BSL типов
**Революционный подход к анализу BSL с автоматическим кешированием**

**Ключевые компоненты:**
- **BslEntity** - универсальное представление любого BSL типа  
- **ConfigurationXmlParser** - прямой парсинг XML без промежуточных отчетов  
- **PlatformDocsCache** - версионное кеширование платформенных типов  
- **ProjectIndexCache** - автоматическое кеширование проектов  
- **UnifiedIndexBuilder** - объединение всех источников в единый индекс

**Производительность (24,055 объектов):**
- Первая индексация: ~795ms
- Загрузка из кеша: ~588ms (25% быстрее)  
- Поиск типа: <1ms (O(1) HashMap)
- Размер кеша проекта: ~7KB
- Поддержка Enterprise конфигураций: 80,000+ объектов

**Структура кеша v2.0:**
```
~/.bsl_analyzer/
├── platform_cache/           # Переиспользуется между проектами
│   ├── v8.3.25.jsonl        # 24,050 типов платформы (~8.5MB)
│   └── v8.3.26.jsonl
└── project_indices/          # Кеши проектов
    └── ProjectName_<hash>/   # Уникальное имя (хеш пути)
        └── v8.3.25/         # Версия платформы
            ├── config_entities.jsonl  # ~5KB
            └── unified_index.json     # ~1KB
```

**Основные API:**

<api-examples>
  <api-method name="find_entity">
    <description>Поиск любой сущности по имени</description>
    <code lang="rust">
// Поиск платформенного типа
let entity = index.find_entity("Массив")?;
let entity = index.find_entity("Array")?; // английский вариант

// Поиск объекта конфигурации  
let entity = index.find_entity("Справочники.Номенклатура")?;
    </code>
  </api-method>
  
  <api-method name="get_all_methods">
    <description>Получение всех методов объекта (включая унаследованные)</description>
    <code lang="rust">
let methods = index.get_all_methods("Справочники.Номенклатура");
// Возвращает HashMap с 45+ методами от СправочникОбъект, ОбъектБД и т.д.

// Проверка наличия метода
if methods.contains_key("Записать") {
    let method = &methods["Записать"];
    println!("Параметры: {:?}", method.parameters);
}
    </code>
  </api-method>
  
  <api-method name="is_assignable">
    <description>Проверка совместимости типов</description>
    <code lang="rust">
// Проверка через интерфейсы
let ok = index.is_assignable("Справочники.Номенклатура", "СправочникСсылка");
assert!(ok); // true - справочник реализует СправочникСсылка

// Проверка несовместимых типов
let ok = index.is_assignable("Число", "Строка");  
assert!(!ok); // false - типы несовместимы
    </code>
  </api-method>
</api-examples>

### 📚 Парсер синтаксис-помощника и документации

**Источник:** `docs/archive/syntax-helper.md`

#### HBK Archive Parser - извлечение документации 1С
- **Компонент:** `src/docs_integration/hbk_parser_full.rs`
- **Источник данных:** Архивы справки `.hbk` / `rebuilt.shcntx_ru.zip` 
- **Результат:** 24,979 элементов BSL документации
- **Производительность:** 7.85 секунд обработки

#### ⚠️ ВАЖНО: Ограничения размера файлов документации
**Файлы документации 1С огромны и часто превышают лимиты API:**
- Размер файлов: до 28,944+ токенов (лимит API: 25,000 токенов)
- `Global context.html` содержит индекс всех глобальных функций
- Отдельные файлы функций (например, `BegOfYear938.html`) с полной документацией

**Рекомендации для анализа больших файлов:**
```bash
# Использовать Read с offset/limit для чтения по частям
Read(file_path="Global context.html", offset=0, limit=1000)

# Использовать Grep для поиска конкретного содержимого  
Grep(pattern="НачалоГода", file_path="Global context.html")
Grep(pattern="BegOfYear", file_path="Global context.html")

# Анализировать структуру по секциям вместо чтения целиком
```

**Архитектурное обоснование:**
- `HbkArchiveParser` спроектирован для поэтапной обработки больших файлов
- Результаты сохраняются в JSONL формате для эффективной работы с большими объемами данных
- Парсер читает файлы по секциям, а не целиком, что критично для файлов размером 100KB+

#### BSL Syntax Extractor - структурированная база знаний
- **Компонент:** `src/docs_integration/bsl_syntax_extractor.rs`
- **Категории элементов:**
  - objects: 354 элемента (Массив, Справочники, etc.)
  - methods: 15,252 элемента (методы объектов)
  - properties: 326 элементов (свойства объектов) 
  - functions: 2,782 элемента (глобальные функции)
  - operators: 6,265 элементов (операторы языка)

#### Структура извлеченной документации
```
~/.bsl_analyzer/ или output/docs_search/
├── main_index.json          # Главный индекс с item_index
├── objects/                 # 354 объекта в 8 файлах
├── methods/                 # 15,252 метода в 376 файлах  
├── properties/              # 326 свойств в 7 файлах
├── functions/               # 2,782 функции в 72 файлах
└── operators/               # 6,265 операторов в 146 файлах
```

#### Интеграция с UnifiedBslIndex
- Типы из документации становятся `BslEntity` в едином индексе
- Автодополнение и подсказки в LSP сервере
- Верификация вызовов методов в анализаторе
- Поддержка поиска методов и свойств

### Актуальные компоненты v2.0

**1. UnifiedBslIndex** - единое хранилище всех типов BSL с O(1) поиском
**2. ConfigurationXmlParser** - прямой парсинг XML без промежуточных отчетов
**3. PlatformDocsCache** - версионное кеширование в ~/.bsl_analyzer/
**4. BSL Syntax Extractor** - извлечение документации через extract_syntax_database()

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

## 📁 Текущее состояние проекта (roadmap.md)

### ✅ Завершенные компоненты v1.2.0 (2025-08-02)
- **Single Analyzer Architecture** - объединение двух анализаторов в один
- **Tree-sitter Integration** - современный BSL парсер с инкрементальным парсингом
- **Unified Semantic Analyzer** - единая система анализа на базе tree-sitter  
- **API Compatibility** - полная совместимость с существующими компонентами
- **Code Cleanup** - 0 ошибок компиляции, 0 warnings

### 🚧 В разработке (Фаза 1.5 - АКТИВНО)
- **BSL Syntax Parser Validation** - проверка парсера синтаксис-помощника
- **Method Signature Verification** - критический приоритет
- **MCP Server Enhancement** - 60% готов  
- **LSP Server Enhancement** - 40% готов

### 📋 План проверки парсера синтаксис-помощника
```bash
# ЭТАП 1: Базовые категории (День 1-2) - КРИТИЧЕСКИЙ ПРИОРИТЕТ
- Примитивные типы: Строка, Число, Дата, Булево
- Коллекции: Массив ✅, СписокЗначений, ТаблицаЗначений  
- Глобальные функции: Сообщить, Тип, ТипЗнч

# ЭТАП 2: Системные объекты (День 3-4)  
- Файловые операции: ЧтениеXML, ЗаписьXML
- Сетевые: HTTPСоединение, WSПрокси
- База данных: Запрос, РезультатЗапроса

# ЭТАП 3: Специализированные типы (День 5-7)
- Администрирование сервера
- Формы и UI элементы  
- COM-объекты и внешние компоненты
```

## 📁 Команды разработки

### **Основные команды проекта:**
```bash
# Построение единого индекса (автоматическое кеширование)
cargo run --bin build_unified_index -- --config "path/to/config" --platform-version "8.3.25"

# Поиск типов в едином индексе  
cargo run --bin query_type -- --name "Справочники.Номенклатура" --config "path/to/config" --show-all-methods

# Извлечение документации BSL (один раз для версии платформы)
cargo run --bin extract_platform_docs -- --archive "path/to/1c_v8.3.25.zip" --version "8.3.25"

# Извлечение синтаксис-помощника в UnifiedBslIndex
cargo run --bin extract_platform_docs -- --archive "examples/rebuilt.shcntx_ru.zip" --version "8.3.25"
```

### **Дополнительные команды:**
```bash
# MCP сервер для интеграции с LLM
cargo run --bin mcp_server

# Тестирование
cargo test

# Проверка кода  
cargo clippy -- -D warnings
```

## ⚠️ **ВАЖНО: Обязательные параметры (обновлено 2025-07-28)**

**Все парсеры теперь требуют явного указания исходных файлов:**

❌ **Больше НЕ работает:**
```bash
cargo run --bin parse_metadata_full              # ОШИБКА - нет --report
cargo run --bin extract_platform_docs           # ОШИБКА - нет --archive  
```

✅ **Правильное использование:**
```bash
cargo run --bin parse_metadata_full -- --report "файл.txt"
cargo run --bin extract_platform_docs -- --archive "архив.zip" --version "8.3.25"
```

**Преимущества новой архитектуры:**
- 🔒 **Безопасность**: Никаких скрытых хардкодед путей
- 📝 **Прозрачность**: Явно видно, какие файлы используются
- ✅ **Валидация**: Проверка существования файлов перед запуском
- 📚 **Справка**: Команда `--help` для каждого парсера
- 🎯 **Предсказуемость**: Одинаковое поведение всех парсеров

## Исправленные критические проблемы (2025-07-28)

### UnifiedBslIndex - Архитектурные улучшения v2.0 ✅

**1. Единое хранилище типов**
- **Реализовано**: Все типы BSL (платформенные + конфигурационные) в одном индексе
- **Поиск**: O(1) HashMap lookup по имени типа
- **Производительность**: 24,055+ типов, поиск < 1ms
- **Кеширование**: Автоматическое версионное кеширование в ~/.bsl_analyzer/

**2. Прямой парсинг XML конфигурации**
- **Подход**: ConfigurationXmlParser напрямую из XML без промежуточных отчетов
- **Поддержка**: Все объекты конфигурации + формы как BslEntity
- **Скорость**: Первый запуск ~795ms, с кешем ~588ms (ускорение 25%)
- **Надежность**: Нет потери данных при парсинге составных типов

**3. Интеграция с документацией платформы**
- **Источник**: Извлечение из .hbk архивов через extract_syntax_database()
- **Объем**: 4,916 платформенных типов с полными сигнатурами методов
- **Формат**: BslSyntaxDatabase → UnifiedBslIndex
- **API**: Единый интерфейс для поиска любых типов BSL

**4. Core + Shell архитектура**
- **Core (Heavy)**: UnifiedBslIndex + MCP Server для LLM
- **Shell (Light)**: CLI + LSP инструменты для разработчиков  
- **Принцип**: Полный контекст для AI, быстрые инструменты для людей
- **Результат**: Нет компромиссов - каждый компонент оптимизирован под задачу

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

## Modern UnifiedBslIndex Architecture (v2.0)

### Direct XML Configuration Parsing
Modern approach using UnifiedBslIndex without legacy parsers:
- Direct XML parsing through ConfigurationXmlParser
- No need for text configuration reports
- Unified type system with platform documentation integration
- Automatic caching for optimal performance

```rust
// Построение единого индекса из конфигурации
let index = UnifiedIndexBuilder::new()
    .build_index(config_path, "8.3.25")?;

// Поиск типов в едином индексе
let entity = index.find_entity("Справочники.Номенклатура")?;
let methods = index.get_all_methods("Справочники.Номенклатура");
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

#### UnifiedBslIndex Integration (`src/unified_index/`)
**NEW**: Прямая интеграция документации в единый индекс типов:
- Все типы из документации становятся BslEntity
- Единый API для поиска платформенных и конфигурационных типов
- Автоматическое наследование методов и свойств
- Эффективное кеширование и версионирование

```rust
let mut index = UnifiedBslIndex::new();
// Прямое добавление типов из документации
index.add_platform_entities(platform_entities)?;
index.add_config_entities(config_entities)?;
```

### 🔧 Поддержка кодировок и BOM (`src/parser/lexer.rs`)
Улучшенное чтение BSL файлов с автоматическим определением кодировки:
- Автоматическое определение и удаление BOM (UTF-8, UTF-16LE, UTF-16BE)
- Поддержка множественных кодировок с fallback на Windows-1251
- Безопасная обработка границ Unicode символов

```rust
use bsl_analyzer::parser::read_bsl_file;

// Чтение BSL файла с автоматическим определением кодировки и BOM
let content = read_bsl_file("module.bsl")?;
let lexer = BslLexer::new();
let tokens = lexer.tokenize(&content)?; // BOM автоматически удален
```

### 🎯 BSL Parser - Актуальное решение (v1.2.0)
**РЕШЕНИЕ:** `tree-sitter` для BSL Grammar Parser (НЕ logos+nom)

**Источник:** `docs/CURRENT_DECISIONS.md` (2025-08-01)

**Обоснование:**
- Инкрементальный парсинг из коробки
- Готовые binding для множества языков  
- Error recovery для невалидного кода
- Стандарт де-факто для IDE интеграции
- Shell Tools ориентация

**Архитектура:**
- Universal diagnostic output с множественными форматерами (JSON, Human, LSP, SARIF)
- Авто-определение формата вывода по контексту
- Интеграция с UnifiedBslIndex для валидации типов/методов

**⚠️ Устарело:** `logos` + `nom` подход (см. docs/BSL_PARSER_DESIGN.md)

## 📁 Структура примеров и данных

### Тестовые конфигурации
- `examples/ConfTest/` - тестовая конфигурация XML с 5 объектами для UnifiedBslIndex

### Документация и справка
- `examples/rebuilt.shcntx_ru.zip` - восстановленный архив справки 1С (обязателен для извлечения)
- `docs/02-components/unified-index/` - детальная документация архитектуры UnifiedBslIndex
- `docs/archive/syntax-helper.md` - документация парсера синтаксис-помощника