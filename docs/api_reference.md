# BSL Analyzer API Reference

## Overview

The BSL Analyzer provides a comprehensive API for BSL (1C:Enterprise) static code analysis. This document covers all public APIs, configuration options, and integration patterns.

## Core Components

### Analyzer Engine

The main analysis engine that processes BSL code and detects issues.

```rust
use bsl_analyzer::{AnalysisEngine, Configuration, AnalysisResult};

// Create and configure engine
let mut engine = AnalysisEngine::new();
engine.set_worker_count(4);
engine.set_inter_module_analysis(true);

// Analyze configuration
let config = Configuration::load_from_directory("./src")?;
let results = engine.analyze_configuration(&config).await?;
```

#### Methods

- `new() -> Self` - Create new analysis engine
- `set_worker_count(count: usize)` - Set number of parallel workers
- `set_inter_module_analysis(enabled: bool)` - Enable inter-module dependency analysis
- `analyze_configuration(config: &Configuration) -> Result<Vec<AnalysisResult>>` - Analyze entire configuration
- `analyze_file(path: &Path) -> Result<AnalysisResult>` - Analyze single file

### Parser Components

#### BslParser

Main parser for BSL source code.

```rust
use bsl_analyzer::{BslParser, BslLexer};

let parser = BslParser::new();
let ast = parser.parse_file("module.bsl")?;
```

#### BslLexer

Tokenizer for BSL source code.

```rust
let lexer = BslLexer::new();
let tokens = lexer.tokenize("Процедура Тест() КонецПроцедуры")?;
```

#### Extended Parser (Phase 6)

Extended parser supporting modern BSL constructs.

```rust
use bsl_analyzer::parser::{ExtendedBslParser, ExtendedBslLexer, IntegratedParser};

let parser = IntegratedParser::new();
let ast = parser.parse_with_extensions("code with try-except")?;
```

### Configuration System

Load and manage 1C:Enterprise configuration metadata.

```rust
use bsl_analyzer::Configuration;

// Load from directory
let config = Configuration::load_from_directory("./src")?;

// Access modules and objects
for module in config.get_modules() {
    println!("Module: {}", module.name);
}

for object in config.get_objects() {
    println!("Object: {} ({})", object.name, object.object_type);
}
```

#### Configuration Structure

- `metadata: ConfigurationMetadata` - Configuration information
- `modules: Vec<ModuleInfo>` - All modules in configuration
- `objects: Vec<ObjectInfo>` - All configuration objects
- `forms: Vec<FormInfo>` - Form definitions
- `dependencies: Vec<DependencyInfo>` - Inter-module dependencies

### Rules System (Phase 4)

Configurable rules for code analysis and quality checking.

#### RulesManager

Main manager for rules configuration and application.

```rust
use bsl_analyzer::{RulesManager, RulesConfig};

// Create with default configuration
let mut manager = RulesManager::new();

// Load from file
let manager = RulesManager::from_file("bsl-rules.toml")?;

// Apply rules to results
let filtered_results = manager.apply_rules(&analysis_results)?;
```

#### Built-in Rules

10 standard rules for BSL analysis:

- **BSL001** - Unused Variable Detection
- **BSL002** - Undefined Variable Detection  
- **BSL003** - Type Mismatch Warning
- **BSL004** - Unknown Method Detection
- **BSL005** - Circular Dependency Detection
- **BSL006** - Dead Code Detection
- **BSL007** - Complex Function Warning
- **BSL008** - Missing Documentation
- **BSL009** - Performance Warnings
- **BSL010** - Security Warnings

#### Custom Rules

```rust
use bsl_analyzer::rules::{CustomRule, CustomRulesManager, PatternType, RuleTarget};

let rule = CustomRule::new(
    "CUSTOM001".to_string(),
    "Russian Comments Only".to_string(),
    PatternType::Regex,
    r"//\s*[a-zA-Z]".to_string(),
    RuleSeverity::Info,
    "Comment should be in Russian: {original_message}".to_string(),
    RuleTarget::SourceCode,
)?;

let mut manager = CustomRulesManager::new();
manager.add_rule(rule)?;
```

### LSP Server (Phase 5)

Language Server Protocol implementation for IDE integration.

#### TCP Server

```rust
use bsl_analyzer::lsp::{TcpLspServer, TcpServerConfig};

let config = TcpServerConfig {
    host: "127.0.0.1".to_string(),
    port: 8080,
    max_connections: 10,
    connection_timeout: Duration::from_secs(30),
    request_timeout: Duration::from_secs(10),
};

let server = TcpLspServer::new(config);
server.start().await?;
```

#### STDIO Server

```rust
use bsl_analyzer::lsp;

// Start STDIO LSP server
lsp::start_stdio_server().await?;
```

### Extended Grammar (Phase 6)

Support for modern BSL constructs and annotations.

#### Extended AST Nodes

```rust
use bsl_analyzer::parser::extended_grammar::{
    ExtendedStatement, TryExceptStatement, AnnotatedVariableDeclaration
};

// Try-except blocks
let try_stmt = TryExceptStatement {
    try_block: block,
    except_clauses: vec![except_clause],
    finally_block: Some(finally_block),
};

// Type annotations
let annotated_var = AnnotatedVariableDeclaration {
    name: "Переменная".to_string(),
    type_annotation: Some("Строка".to_string()),
    value: None,
};
```

#### Supported Extensions

- Try-except-finally blocks
- Type annotations with `?` operator
- Documentation comments with `///`
- Async/await constructs
- Advanced control flow

### Metrics System (Phase 7)

Code quality metrics and technical debt analysis.

#### QualityMetricsManager

```rust
use bsl_analyzer::metrics::QualityMetricsManager;

let mut manager = QualityMetricsManager::new();
let metrics = manager.analyze_file(&path, &content)?;

println!("Quality Score: {:.1}/100", metrics.quality_score);
println!("Maintainability Index: {:.1}", metrics.maintainability_index);
```

#### Metrics Types

1. **Complexity Metrics**
   - Cyclomatic complexity
   - Cognitive complexity
   - Function-level metrics

2. **Maintainability Metrics**
   - Maintainability Index
   - Code duplication analysis
   - SLOC counts

3. **Technical Debt Analysis**
   - 5 debt categories (Design, Code Quality, Performance, Security, Documentation)
   - Severity levels (Critical, High, Medium, Low, Info)
   - Prioritization and recommendations

### Report Generation

Multi-format report generation with SARIF support.

```rust
use bsl_analyzer::{ReportManager, ReportFormat, ReportConfig};

let config = ReportConfig {
    format: ReportFormat::Sarif,
    output_path: Some("results.sarif".to_string()),
    include_details: true,
    include_performance: true,
    include_dependencies: false,
    min_severity: None,
};

let manager = ReportManager::with_config(config);
let report = manager.generate(&analysis_results)?;
```

#### Supported Formats

- **SARIF** - GitHub Actions integration
- **JSON** - Machine-readable format
- **HTML** - Rich web reports
- **Text** - Console-friendly output

## CLI Interface

### Main Commands

```bash
# Analyze configuration
bsl-analyzer analyze --path ./src --format sarif --output results.sarif

# Start LSP server
bsl-analyzer lsp --port 8080

# Generate all reports
bsl-analyzer generate-reports --path ./src --output-dir ./reports

# Show configuration info
bsl-analyzer info --path ./src

# Validate configuration
bsl-analyzer validate --path ./src
```

### Rules Management

```bash
# List all rules
bsl-analyzer rules list

# Show rule details
bsl-analyzer rules show BSL001

# Create rules configuration
bsl-analyzer rules init --output bsl-rules.toml

# Validate rules configuration
bsl-analyzer rules validate --config bsl-rules.toml

# Export custom rules template
bsl-analyzer rules export-custom --output custom-rules.toml
```

### Metrics Commands

```bash
# Analyze metrics
bsl-analyzer metrics analyze --path ./src --include-functions --include-debt

# Generate metrics report
bsl-analyzer metrics report --path ./src --output-dir ./metrics

# Test metrics system
bsl-analyzer metrics test
```

### LSP Configuration

```bash
# Initialize LSP configuration
bsl-analyzer lsp-config init --tcp --port 8080

# Validate LSP configuration
bsl-analyzer lsp-config validate --config lsp-config.toml

# Test LSP connection
bsl-analyzer lsp-config test-connection --config lsp-config.toml
```

## Configuration Files

### Rules Configuration (TOML)

```toml
# BSL Analyzer Rules Configuration
active_profile = "default"

[settings]
max_errors = 1000
parallel_processing = true
cache_enabled = true

[profiles.default]
description = "Default rule profile for BSL analysis"
enabled_rules = ["BSL001", "BSL002", "BSL003", "BSL004", "BSL005"]

[profiles.strict]
description = "Strict rule profile with all rules enabled"
enabled_rules = ["BSL001", "BSL002", "BSL003", "BSL004", "BSL005", "BSL006", "BSL007", "BSL008", "BSL009", "BSL010"]

[rules.BSL001]
enabled = true
severity = "Warning"
min_confidence = 0.9
description = "Variable is declared but never used"
tags = ["unused", "cleanup"]

[rules.BSL001.config]
check_parameters = true
ignore_underscore = true
```

### LSP Configuration (TOML)

```toml
# BSL Analyzer LSP Server Configuration
mode = "tcp"

[tcp]
host = "127.0.0.1"
port = 8080
max_connections = 10
connection_timeout = 30000
request_timeout = 10000

[analysis]
enable_real_time = true
enable_auto_save = true
max_file_size = 1048576
enable_diagnostics = true
enable_completion = true
enable_hover = true

[logging]
level = "info"
file = "lsp-server.log"
```

### Custom Rules (TOML)

```toml
[CUSTOM001]
name = "Russian Comments Only"
description = "Ensure comments are written in Russian"
pattern_type = "regex"
pattern = "//\\s*[a-zA-Z]"
severity = "Info"
message_template = "Comment should be in Russian: {original_message}"
applies_to = "source_code"
enabled = true
confidence = 0.8

[CUSTOM001.config]
suggestion = "Write comments in Russian for better code readability"
```

## Integration Patterns

### CI/CD Integration

#### GitHub Actions

```yaml
name: BSL Analysis
on: [push, pull_request]

jobs:
  analyze:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install BSL Analyzer
        run: cargo install bsl-analyzer
        
      - name: Run Analysis
        run: bsl-analyzer analyze --path ./src --format sarif --output results.sarif
        
      - name: Upload SARIF
        uses: github/codeql-action/upload-sarif@v2
        with:
          sarif_file: results.sarif
```

### VS Code Extension

```typescript
// VS Code extension integration
import { LanguageClient } from 'vscode-languageclient/node';

const client = new LanguageClient(
    'bsl-analyzer',
    'BSL Analyzer',
    {
        command: 'bsl-analyzer',
        args: ['lsp', '--port', '8080']
    },
    {
        documentSelector: [{ scheme: 'file', language: 'bsl' }]
    }
);

client.start();
```

### Library Integration

```rust
use bsl_analyzer::*;

async fn analyze_project(project_path: &str) -> Result<()> {
    // Load configuration
    let config = Configuration::load_from_directory(project_path)?;
    
    // Create analysis engine
    let mut engine = AnalysisEngine::new();
    engine.set_worker_count(num_cpus::get());
    
    // Load rules
    let rules_manager = RulesManager::from_file("bsl-rules.toml")?;
    
    // Run analysis
    let results = engine.analyze_configuration(&config).await?;
    
    // Convert to unified format
    let mut combined_results = AnalysisResults::new();
    for result in &results {
        for error in &result.errors {
            combined_results.add_error(error.clone());
        }
    }
    
    // Apply rules
    let filtered_results = rules_manager.apply_rules(&combined_results)?;
    
    // Generate metrics
    let mut metrics_manager = QualityMetricsManager::new();
    let quality_metrics = metrics_manager.analyze_results(&filtered_results)?;
    
    // Generate reports
    let report_config = ReportConfig {
        format: ReportFormat::Html,
        include_details: true,
        include_performance: true,
        include_dependencies: true,
        ..Default::default()
    };
    
    let report_manager = ReportManager::with_config(report_config);
    report_manager.generate_all_formats(&filtered_results, "./reports")?;
    
    Ok(())
}
```

## Error Handling

### Error Types

```rust
use bsl_analyzer::core::{AnalysisError, ErrorLevel};

pub enum AnalyzerError {
    ParseError(String),
    ConfigurationError(String),
    AnalysisError(String),
    IoError(std::io::Error),
    SerializationError(serde_json::Error),
}
```

### Error Levels

- **Error** - Critical issues that prevent compilation
- **Warning** - Issues that should be addressed
- **Info** - Informational messages
- **Hint** - Suggestions for improvement

## Performance Considerations

### Parallel Processing

The analyzer uses Rust's native threading for parallel processing:

```rust
// Configure worker threads
engine.set_worker_count(num_cpus::get());

// Enable parallel rules processing
rules_config.settings.parallel_processing = true;
```

### Caching

Enable caching for improved performance:

```rust
use bsl_analyzer::cache::CacheManager;

let cache_manager = CacheManager::new("./cache")?;
engine.set_cache_manager(cache_manager);
```

### Memory Management

- Zero-copy string processing where possible
- Streaming analysis for large files
- Incremental parsing support

## Version Compatibility

- **Rust**: 1.70+
- **1C:Enterprise**: 8.3+
- **BSL Language**: All standard constructs + extensions
- **LSP Protocol**: 3.17

## API Stability

- **Core APIs** - Stable, following semantic versioning
- **Extended Grammar** - Experimental, may change
- **Metrics System** - Stable
- **Rules System** - Stable

## Support and Documentation

- **GitHub**: https://github.com/yourorg/bsl-analyzer
- **Issues**: Report bugs and feature requests
- **Discussions**: Community support and questions
- **Wiki**: Extended documentation and examples