# BSL Type Safety Analyzer - API Documentation

This document provides comprehensive API documentation for the BSL Type Safety Analyzer library.

## Table of Contents

1. [Core API](#core-api)
2. [Analysis API](#analysis-api)
3. [Rules API](#rules-api)
4. [Metrics API](#metrics-api)
5. [Reports API](#reports-api)
6. [LSP API](#lsp-api)
7. [Configuration API](#configuration-api)
8. [Error Handling](#error-handling)

## Core API

### `BslAnalyzer`

The main entry point for BSL analysis functionality.

```rust
use bsl_analyzer::{BslAnalyzer, Configuration};

pub struct BslAnalyzer {
    // Private fields
}

impl BslAnalyzer {
    /// Creates a new analyzer with default configuration
    pub fn new() -> Self;
    
    /// Creates an analyzer with custom configuration
    pub fn with_config(config: Configuration) -> Self;
    
    /// Analyzes a single BSL file
    pub fn analyze_file<P: AsRef<Path>>(
        &mut self, 
        file_path: P
    ) -> anyhow::Result<AnalysisResults>;
    
    /// Analyzes BSL source code directly
    pub fn analyze_code(
        &mut self, 
        code: &str, 
        file_path: Option<&Path>
    ) -> anyhow::Result<AnalysisResults>;
    
    /// Analyzes an entire BSL configuration
    pub fn analyze_configuration<P: AsRef<Path>>(
        &mut self, 
        config_path: P
    ) -> anyhow::Result<ConfigurationResults>;
    
    /// Gets the current analysis results
    pub fn get_results(&self) -> &AnalysisResults;
    
    /// Clears analysis results
    pub fn clear_results(&mut self);
}
```

#### Example

```rust
use bsl_analyzer::BslAnalyzer;

fn main() -> anyhow::Result<()> {
    let mut analyzer = BslAnalyzer::new();
    
    // Analyze a single file
    let results = analyzer.analyze_file("./src/Module1.bsl")?;
    println!("Found {} errors", results.error_count());
    
    // Analyze code directly
    let code = r#"
        Процедура Тест()
            НеопределеннаяПеременная = 1;
        КонецПроцедуры
    "#;
    let results = analyzer.analyze_code(code, None)?;
    println!("Code analysis: {} issues", results.total_issues());
    
    Ok(())
}
```

### `AnalysisResults`

Contains the results of BSL code analysis.

```rust
pub struct AnalysisResults {
    // Private fields
}

impl AnalysisResults {
    /// Creates new empty results
    pub fn new() -> Self;
    
    /// Gets the total number of issues found
    pub fn total_issues(&self) -> usize;
    
    /// Gets the number of errors
    pub fn error_count(&self) -> usize;
    
    /// Gets the number of warnings
    pub fn warning_count(&self) -> usize;
    
    /// Gets the number of info messages
    pub fn info_count(&self) -> usize;
    
    /// Gets the number of hints
    pub fn hint_count(&self) -> usize;
    
    /// Checks if there are any errors
    pub fn has_errors(&self) -> bool;
    
    /// Checks if there are any warnings
    pub fn has_warnings(&self) -> bool;
    
    /// Gets all issues as a vector
    pub fn get_issues(&self) -> &[AnalysisIssue];
    
    /// Gets issues filtered by severity
    pub fn get_issues_by_severity(&self, severity: ErrorLevel) -> Vec<&AnalysisIssue>;
    
    /// Gets issues for a specific file
    pub fn get_issues_for_file(&self, file_path: &Path) -> Vec<&AnalysisIssue>;
}
```

### `AnalysisIssue`

Represents a single analysis issue.

```rust
pub struct AnalysisIssue {
    pub rule_id: String,
    pub severity: ErrorLevel,
    pub message: String,
    pub file_path: PathBuf,
    pub line: u32,
    pub column: u32,
    pub end_line: Option<u32>,
    pub end_column: Option<u32>,
    pub confidence: f64,
    pub tags: Vec<String>,
}

impl AnalysisIssue {
    /// Creates a new analysis issue
    pub fn new(
        rule_id: String,
        severity: ErrorLevel,
        message: String,
        file_path: PathBuf,
        line: u32,
        column: u32,
    ) -> Self;
    
    /// Gets the source location as a formatted string
    pub fn location_string(&self) -> String;
    
    /// Checks if this is a high-confidence issue
    pub fn is_high_confidence(&self) -> bool;
}
```

## Analysis API

### `SemanticAnalyzer`

Performs semantic analysis on BSL AST.

```rust
use bsl_analyzer::analyzer::SemanticAnalyzer;

pub struct SemanticAnalyzer {
    // Private fields
}

impl SemanticAnalyzer {
    /// Creates a new semantic analyzer
    pub fn new() -> Self;
    
    /// Analyzes the given AST
    pub fn analyze(&mut self, ast: &BslAst) -> anyhow::Result<()>;
    
    /// Gets the symbol table
    pub fn get_symbol_table(&self) -> &SymbolTable;
    
    /// Gets detected issues
    pub fn get_issues(&self) -> &[AnalysisIssue];
    
    /// Checks if a variable is defined in the current scope
    pub fn is_variable_defined(&self, name: &str) -> bool;
    
    /// Gets the type of a variable
    pub fn get_variable_type(&self, name: &str) -> Option<&BslType>;
}
```

### `MethodVerifier`

Verifies method calls and parameter compatibility.

```rust
use bsl_analyzer::verifiers::MethodVerifier;

pub struct MethodVerifier {
    // Private fields
}

impl MethodVerifier {
    /// Creates a new method verifier
    pub fn new() -> Self;
    
    /// Verifies a method call
    pub fn verify_method_call(
        &self,
        method_name: &str,
        args: &[BslExpression],
        context: &AnalysisContext,
    ) -> VerificationResult;
    
    /// Checks parameter compatibility
    pub fn check_parameter_compatibility(
        &self,
        expected: &BslType,
        actual: &BslType,
    ) -> bool;
}
```

## Rules API

### `RulesManager`

Manages configurable analysis rules.

```rust
use bsl_analyzer::rules::{RulesManager, RulesConfig};

pub struct RulesManager {
    // Private fields
}

impl RulesManager {
    /// Creates a new rules manager with default configuration
    pub fn new() -> Self;
    
    /// Creates a rules manager with custom configuration
    pub fn new_with_config(config: RulesConfig) -> Self;
    
    /// Creates a rules manager from configuration file
    pub fn from_file<P: AsRef<Path>>(config_path: P) -> anyhow::Result<Self>;
    
    /// Applies rules to analysis results
    pub fn apply_rules(
        &mut self,
        results: &AnalysisResults,
    ) -> anyhow::Result<AnalysisResults>;
    
    /// Gets rule statistics
    pub fn get_stats(&self) -> &HashMap<String, RuleStats>;
    
    /// Gets the current configuration
    pub fn config(&self) -> &RulesConfig;
    
    /// Reloads configuration from file
    pub fn reload_config<P: AsRef<Path>>(&mut self, config_path: P) -> anyhow::Result<()>;
    
    /// Validates the current configuration
    pub fn validate_config(&self) -> anyhow::Result<Vec<String>>;
}
```

### `RulesConfig`

Configuration structure for analysis rules.

```rust
use bsl_analyzer::rules::{RulesConfig, RuleConfig, RuleSeverity};

pub struct RulesConfig {
    pub version: String,
    pub active_profile: String,
    pub profiles: HashMap<String, RuleProfile>,
    pub rules: HashMap<String, RuleConfig>,
    pub settings: GlobalSettings,
}

impl RulesConfig {
    /// Creates default configuration
    pub fn default() -> Self;
    
    /// Creates strict profile configuration
    pub fn strict_profile() -> Self;
    
    /// Loads configuration from TOML file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self>;
    
    /// Saves configuration to TOML file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()>;
    
    /// Loads configuration from YAML file
    pub fn load_from_yaml<P: AsRef<Path>>(path: P) -> anyhow::Result<Self>;
    
    /// Gets the active profile
    pub fn get_active_profile(&self) -> anyhow::Result<&RuleProfile>;
    
    /// Checks if a rule is enabled
    pub fn is_rule_enabled(&self, rule_id: &str) -> anyhow::Result<bool>;
    
    /// Gets effective severity for a rule
    pub fn get_rule_severity(&self, rule_id: &str) -> anyhow::Result<RuleSeverity>;
    
    /// Validates the configuration
    pub fn validate(&self) -> anyhow::Result<Vec<String>>;
}
```

## Metrics API

### `QualityMetricsManager`

Manages code quality metrics analysis.

```rust
use bsl_analyzer::metrics::{QualityMetricsManager, QualityReport};

pub struct QualityMetricsManager {
    // Private fields
}

impl QualityMetricsManager {
    /// Creates a new metrics manager
    pub fn new() -> Self;
    
    /// Analyzes code quality metrics for a file
    pub fn analyze_file<P: AsRef<Path>>(
        &mut self,
        file_path: P,
    ) -> anyhow::Result<QualityReport>;
    
    /// Analyzes metrics for source code
    pub fn analyze_code(&mut self, code: &str) -> anyhow::Result<QualityReport>;
    
    /// Gets comprehensive quality report
    pub fn get_comprehensive_report(&self) -> QualityReport;
    
    /// Exports metrics to JSON
    pub fn export_json(&self) -> anyhow::Result<String>;
}
```

### `QualityReport`

Comprehensive code quality metrics report.

```rust
pub struct QualityReport {
    pub complexity: ComplexityMetrics,
    pub maintainability: MaintainabilityMetrics,
    pub technical_debt: TechnicalDebtAnalysis,
    pub duplication_percentage: f64,
    pub recommendations: Vec<String>,
}

impl QualityReport {
    /// Gets overall quality score (0-100)
    pub fn overall_score(&self) -> f64;
    
    /// Checks if quality meets threshold
    pub fn meets_quality_threshold(&self, threshold: f64) -> bool;
    
    /// Gets priority recommendations
    pub fn get_priority_recommendations(&self) -> Vec<&str>;
}
```

## Reports API

### `ReportManager`

Manages report generation in multiple formats.

```rust
use bsl_analyzer::reports::{ReportManager, ReportFormat, ReportConfig};

pub struct ReportManager {
    // Private fields
}

impl ReportManager {
    /// Creates a new report manager
    pub fn new() -> Self;
    
    /// Creates a report manager with custom configuration
    pub fn with_config(config: ReportConfig) -> Self;
    
    /// Generates a report in the specified format
    pub fn generate_report(
        &self,
        results: &AnalysisResults,
        format: ReportFormat,
    ) -> anyhow::Result<String>;
    
    /// Saves report to file
    pub fn save_report_to_file<P: AsRef<Path>>(
        &self,
        results: &AnalysisResults,
        format: ReportFormat,
        output_path: P,
    ) -> anyhow::Result<()>;
    
    /// Generates SARIF report for CI/CD integration
    pub fn generate_sarif_report(
        &self,
        results: &AnalysisResults,
    ) -> anyhow::Result<String>;
    
    /// Generates HTML report with interactive features
    pub fn generate_html_report(
        &self,
        results: &AnalysisResults,
        metrics: Option<&QualityReport>,
    ) -> anyhow::Result<String>;
}
```

## LSP API

### `LspServer`

Language Server Protocol implementation.

```rust
use bsl_analyzer::lsp::{LspServer, LspConfig};

pub struct LspServer {
    // Private fields
}

impl LspServer {
    /// Creates a new LSP server
    pub fn new(config: LspConfig) -> Self;
    
    /// Starts the LSP server
    pub async fn start(&mut self) -> anyhow::Result<()>;
    
    /// Stops the LSP server
    pub async fn stop(&mut self) -> anyhow::Result<()>;
    
    /// Handles a document change
    pub async fn handle_document_change(
        &mut self,
        uri: &str,
        content: &str,
    ) -> anyhow::Result<Vec<Diagnostic>>;
    
    /// Provides completions for a position
    pub async fn provide_completions(
        &self,
        uri: &str,
        position: Position,
    ) -> anyhow::Result<Vec<CompletionItem>>;
}
```

### `TcpLspServer`

TCP-based LSP server for production environments.

```rust
use bsl_analyzer::lsp::{TcpLspServer, TcpServerConfig};

pub struct TcpLspServer {
    // Private fields
}

impl TcpLspServer {
    /// Creates a new TCP LSP server
    pub fn new(config: TcpServerConfig) -> Self;
    
    /// Starts the TCP server
    pub async fn start(&mut self) -> anyhow::Result<()>;
    
    /// Gets server statistics
    pub fn get_stats(&self) -> ServerStatistics;
    
    /// Gets active connections count
    pub fn active_connections(&self) -> usize;
}
```

## Configuration API

### `Configuration`

1C configuration metadata handling.

```rust
use bsl_analyzer::configuration::Configuration;

pub struct Configuration {
    // Private fields
}

impl Configuration {
    /// Loads configuration from directory
    pub fn load_from_directory<P: AsRef<Path>>(path: P) -> anyhow::Result<Self>;
    
    /// Loads configuration from XML file
    pub fn load_from_xml<P: AsRef<Path>>(path: P) -> anyhow::Result<Self>;
    
    /// Gets all modules in the configuration
    pub fn get_modules(&self) -> &[ModuleInfo];
    
    /// Gets module by name
    pub fn get_module(&self, name: &str) -> Option<&ModuleInfo>;
    
    /// Gets all objects in the configuration
    pub fn get_objects(&self) -> &[ObjectInfo];
    
    /// Gets object by name and type
    pub fn get_object(&self, name: &str, object_type: ObjectType) -> Option<&ObjectInfo>;
    
    /// Validates configuration structure
    pub fn validate(&self) -> anyhow::Result<Vec<String>>;
}
```

## Error Handling

### `AnalysisError`

Main error type for analysis operations.

```rust
use bsl_analyzer::core::AnalysisError;

#[derive(Debug, thiserror::Error)]
pub enum AnalysisError {
    #[error("Parse error: {message}")]
    ParseError { message: String },
    
    #[error("Semantic error: {message}")]
    SemanticError { message: String },
    
    #[error("Configuration error: {message}")]
    ConfigError { message: String },
    
    #[error("IO error: {source}")]
    IoError { #[from] source: std::io::Error },
    
    #[error("Rule error: {message}")]
    RuleError { message: String },
}
```

### `ErrorLevel`

Severity levels for analysis issues.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorLevel {
    /// Critical error that blocks analysis
    Error,
    /// Warning that should be addressed
    Warning,
    /// Informational message
    Info,
    /// Subtle suggestion for improvement
    Hint,
}
```

## Usage Examples

### Basic Analysis

```rust
use bsl_analyzer::{BslAnalyzer, analyze_file};

fn main() -> anyhow::Result<()> {
    // Quick analysis of a single file
    let result = analyze_file("./src/Module1.bsl")?;
    println!("Analysis result: {}", result);
    
    // Full analyzer with custom configuration
    let mut analyzer = BslAnalyzer::new();
    let results = analyzer.analyze_file("./src/Module1.bsl")?;
    
    if results.has_errors() {
        eprintln!("Found {} errors:", results.error_count());
        for issue in results.get_issues_by_severity(ErrorLevel::Error) {
            eprintln!("  {}: {}", issue.location_string(), issue.message);
        }
    }
    
    Ok(())
}
```

### Configuration-based Analysis

```rust
use bsl_analyzer::{BslAnalyzer, Configuration, rules::RulesManager};

fn main() -> anyhow::Result<()> {
    // Load 1C configuration
    let config = Configuration::load_from_directory("./src")?;
    
    // Setup rules
    let mut rules_manager = RulesManager::from_file("bsl-rules.toml")?;
    
    // Analyze all modules
    let mut analyzer = BslAnalyzer::new();
    for module in config.get_modules() {
        let results = analyzer.analyze_file(&module.path)?;
        let filtered_results = rules_manager.apply_rules(&results)?;
        
        if filtered_results.total_issues() > 0 {
            println!("Issues in {}: {}", module.name, filtered_results.total_issues());
        }
    }
    
    Ok(())
}
```

### Metrics Analysis

```rust
use bsl_analyzer::metrics::QualityMetricsManager;

fn main() -> anyhow::Result<()> {
    let mut metrics_manager = QualityMetricsManager::new();
    let report = metrics_manager.analyze_file("./src/ComplexModule.bsl")?;
    
    println!("Quality Score: {:.1}/100", report.overall_score());
    println!("Cyclomatic Complexity: {:.1}", report.complexity.average_cyclomatic_complexity);
    println!("Technical Debt: {} minutes", report.technical_debt.total_debt_minutes);
    
    if !report.meets_quality_threshold(70.0) {
        println!("Recommendations:");
        for rec in report.get_priority_recommendations() {
            println!("  - {}", rec);
        }
    }
    
    Ok(())
}
```

### Report Generation

```rust
use bsl_analyzer::reports::{ReportManager, ReportFormat};

fn main() -> anyhow::Result<()> {
    let mut analyzer = BslAnalyzer::new();
    let results = analyzer.analyze_file("./src/Module1.bsl")?;
    
    let report_manager = ReportManager::new();
    
    // Generate SARIF for CI/CD
    report_manager.save_report_to_file(
        &results,
        ReportFormat::Sarif,
        "analysis-results.sarif",
    )?;
    
    // Generate HTML for developers
    report_manager.save_report_to_file(
        &results,
        ReportFormat::Html,
        "analysis-report.html",
    )?;
    
    Ok(())
}
```

This API documentation covers the main interfaces and usage patterns for the BSL Type Safety Analyzer. For more detailed examples and advanced usage, see the examples directory and integration tests.