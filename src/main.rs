/*!
# BSL Analyzer CLI

Command-line interface for BSL (1C:Enterprise) static analyzer.
*/

use anyhow::{Context, Result};
use bsl_analyzer::{Configuration, ReportManager, ReportFormat, ReportConfig};
use bsl_analyzer::bsl_parser::BslAnalyzer;
use bsl_analyzer::{RulesManager, RulesConfig, BuiltinRules, CustomRulesManager};
// use bsl_analyzer::analyzer::AnalysisResult; // Удален с analyzer
use bsl_analyzer::core::AnalysisResults;
use bsl_analyzer::metrics::QualityMetricsManager;
use clap::{Parser, Subcommand};
use console::{style, Term};
use indicatif::{ProgressBar, ProgressStyle};
use serde_json;
use std::path::PathBuf;
use std::time::Instant;
use tracing::info;
use tracing_subscriber;

#[derive(Parser)]
#[command(
    name = "bsl-analyzer",
    version = env!("CARGO_PKG_VERSION"),
    author = "BSL Analyzer Team",
    about = "Advanced BSL (1C:Enterprise) static analyzer with type safety checking"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
    
    /// Output format (json, text, sarif, html)
    #[arg(short = 'f', long, default_value = "text")]
    format: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze BSL configuration or files
    Analyze {
        /// Path to configuration directory or BSL files
        #[arg(short, long)]
        path: PathBuf,
        
        /// Output file (stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Number of parallel workers
        #[arg(short, long)]
        workers: Option<usize>,
        
        /// Enable inter-module dependency analysis
        #[arg(long, default_value = "true")]
        inter_module: bool,
        
        /// Include dependency analysis in report
        #[arg(long)]
        include_dependencies: bool,
        
        /// Include performance metrics in report
        #[arg(long)]
        include_performance: bool,
        
        /// Path to rules configuration file
        #[arg(long)]
        rules_config: Option<PathBuf>,
        
        /// Active rules profile to use
        #[arg(long)]
        rules_profile: Option<String>,
        
        /// Enable enhanced semantic analysis with UnifiedBslIndex
        #[arg(long)]
        enable_enhanced_semantics: bool,
        
        /// Platform version for UnifiedBslIndex (default: 8.3.25)
        #[arg(long, default_value = "8.3.25")]
        platform_version: String,
        
        /// Enable method call validation
        #[arg(long)]
        enable_method_validation: bool,
    },
    
    /// Start Language Server Protocol (LSP) server
    Lsp {
        /// LSP server port (stdio if not specified)
        #[arg(short, long)]
        port: Option<u16>,
    },
    
    /// Show configuration information
    Info {
        /// Path to configuration directory
        #[arg(short, long)]
        path: PathBuf,
    },
    
    /// Validate configuration structure
    Validate {
        /// Path to configuration directory
        #[arg(short, long)]
        path: PathBuf,
    },
    
    /// Generate reports in all supported formats
    GenerateReports {
        /// Path to configuration directory or BSL files
        #[arg(short, long)]
        path: PathBuf,
        
        /// Output directory for reports
        #[arg(short, long, default_value = "./reports")]
        output_dir: PathBuf,
        
        /// Include dependency analysis
        #[arg(long)]
        include_dependencies: bool,
        
        /// Include performance metrics
        #[arg(long)]
        include_performance: bool,
    },
    
    /// Rules management commands
    Rules {
        #[command(subcommand)]
        command: RulesCommands,
    },
    
    /// Code quality metrics and analysis
    Metrics {
        #[command(subcommand)]
        command: MetricsCommands,
    },
    
    /// LSP server configuration commands
    LspConfig {
        #[command(subcommand)]
        command: LspConfigCommands,
    },
    
    /// Generate contracts from 1C configuration
    GenerateContracts {
        /// Path to configuration directory
        #[arg(short = 'c', long)]
        config_path: PathBuf,
        
        /// Path to configuration report (optional)
        #[arg(short = 'r', long)]
        report_path: Option<PathBuf>,
        
        /// Output directory for contracts
        #[arg(short, long, default_value = "./contracts")]
        output: PathBuf,
        
        /// Generate metadata contracts
        #[arg(long, default_value = "true")]
        metadata: bool,
        
        /// Generate form contracts
        #[arg(long, default_value = "true")]
        forms: bool,
        
        /// Generate module contracts (stub)
        #[arg(long, default_value = "false")]
        modules: bool,
    },
    
    /// Parse 1C documentation archive (.hbk)
    ParseDocs {
        /// Path to .hbk documentation archive
        #[arg(short, long)]
        hbk_path: PathBuf,
        
        /// Output directory for parsed documentation
        #[arg(short, long, default_value = "./docs")]
        output: PathBuf,
        
        /// Maximum files to process (for testing)
        #[arg(long)]
        max_files: Option<usize>,
    },
}

#[derive(Subcommand)]
enum RulesCommands {
    /// List all available rules
    List {
        /// Show only enabled rules
        #[arg(long)]
        enabled_only: bool,
        
        /// Filter by rule tags
        #[arg(long)]
        tag: Option<String>,
    },
    
    /// Show rule details
    Show {
        /// Rule ID to show details for
        rule_id: String,
    },
    
    /// Create example configuration file
    Init {
        /// Output path for configuration file
        #[arg(short, long, default_value = "bsl-rules.toml")]
        output: PathBuf,
        
        /// Create with strict profile
        #[arg(long)]
        strict: bool,
    },
    
    /// Validate rules configuration
    Validate {
        /// Path to rules configuration file
        #[arg(short, long, default_value = "bsl-rules.toml")]
        config: PathBuf,
    },
    
    /// Export custom rules template
    ExportCustom {
        /// Output path for custom rules file
        #[arg(short, long, default_value = "custom-rules.toml")]
        output: PathBuf,
    },
}

#[derive(Subcommand)]
enum MetricsCommands {
    /// Analyze code quality metrics
    Analyze {
        /// Path to configuration directory or BSL files
        #[arg(short, long)]
        path: PathBuf,
        
        /// Output format (json, text)
        #[arg(short = 'f', long, default_value = "text")]
        format: String,
        
        /// Output file (stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Include detailed function metrics
        #[arg(long)]
        include_functions: bool,
        
        /// Include technical debt analysis
        #[arg(long)]
        include_debt: bool,
    },
    
    /// Generate metrics report
    Report {
        /// Path to configuration directory or BSL files
        #[arg(short, long)]
        path: PathBuf,
        
        /// Output directory for reports
        #[arg(short, long, default_value = "./metrics")]
        output_dir: PathBuf,
    },
    
    /// Test metrics system with sample data
    Test,
}

#[derive(Subcommand)]
enum LspConfigCommands {
    /// Initialize LSP configuration
    Init {
        /// Output path for configuration file
        #[arg(short, long, default_value = "lsp-config.toml")]
        output: PathBuf,
        
        /// Use TCP mode by default
        #[arg(long)]
        tcp: bool,
        
        /// TCP port for server
        #[arg(long, default_value = "8080")]
        port: u16,
    },
    
    /// Validate LSP configuration
    Validate {
        /// Path to LSP configuration file
        #[arg(short, long, default_value = "lsp-config.toml")]
        config: PathBuf,
    },
    
    /// Test LSP server connectivity
    TestConnection {
        /// LSP configuration file
        #[arg(short, long, default_value = "lsp-config.toml")]
        config: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(format!("bsl_analyzer={}", log_level))
        .init();
    
    match cli.command {
        Commands::Analyze { 
            path, 
            output, 
            workers, 
            inter_module,
            include_dependencies,
            include_performance,
            rules_config,
            rules_profile,
            enable_enhanced_semantics,
            platform_version,
            enable_method_validation,
        } => {
            analyze_command(
                path, 
                output, 
                workers, 
                inter_module, 
                include_dependencies, 
                include_performance, 
                &cli.format, 
                rules_config, 
                rules_profile,
                enable_enhanced_semantics,
                platform_version,
                enable_method_validation,
            ).await?;
        }
        
        Commands::Lsp { port } => {
            lsp_command(port).await?;
        }
        
        Commands::Info { path } => {
            info_command(path).await?;
        }
        
        Commands::Validate { path } => {
            validate_command(path).await?;
        }
        
        Commands::GenerateReports { path, output_dir, include_dependencies, include_performance } => {
            generate_reports_command(path, output_dir, include_dependencies, include_performance).await?;
        }
        
        Commands::Rules { command } => {
            rules_command(command).await?;
        }
        
        Commands::Metrics { command } => {
            metrics_command(command).await?;
        }
        
        Commands::LspConfig { command } => {
            lsp_config_command(command).await?;
        }
        
        Commands::GenerateContracts { config_path: _, report_path: _, output: _, metadata: _, forms: _, modules: _ } => {
            eprintln!("❌ Contract generation has been removed. Use UnifiedBslIndex instead.");
            std::process::exit(1);
        }
        
        Commands::ParseDocs { hbk_path, output, max_files } => {
            // For now, default behavior: analyze structure and extract samples
            let analyze_structure = true;
            let extract_samples = max_files.is_some();
            parse_docs_command(hbk_path, output, analyze_structure, extract_samples).await?;
        }
    }
    
    Ok(())
}

async fn rules_command(command: RulesCommands) -> Result<()> {
    let term = Term::stdout();
    
    match command {
        RulesCommands::List { enabled_only, tag } => {
            term.write_line(&format!("🔧 {}", style("Available Rules").bold().cyan()))?;
            
            let rules = BuiltinRules::get_all_rules();
            
            for (rule_id, rule_config) in &rules {
                if enabled_only && !rule_config.enabled {
                    continue;
                }
                
                if let Some(ref filter_tag) = tag {
                    if !rule_config.tags.contains(filter_tag) {
                        continue;
                    }
                }
                
                let status = if rule_config.enabled {
                    style("✅ ENABLED").green()
                } else {
                    style("❌ DISABLED").red()
                };
                
                term.write_line(&format!(
                    "   {} {} [{}] - {}",
                    status,
                    style(rule_id).bold(),
                    style(format!("{:?}", rule_config.severity)).dim(),
                    rule_config.description.as_ref().unwrap_or(&"No description".to_string())
                ))?;
                
                if !rule_config.tags.is_empty() {
                    term.write_line(&format!(
                        "      Tags: {}",
                        style(rule_config.tags.join(", ")).dim()
                    ))?;
                }
            }
        }
        
        RulesCommands::Show { rule_id } => {
            let rules = BuiltinRules::get_all_rules();
            
            if let Some(rule_config) = rules.get(&rule_id) {
                term.write_line(&format!("🔍 {} - {}", style(&rule_id).bold().cyan(), rule_id))?;
                term.write_line(&format!(
                    "   Status: {}",
                    if rule_config.enabled {
                        style("ENABLED").green()
                    } else {
                        style("DISABLED").red()
                    }
                ))?;
                term.write_line(&format!("   Severity: {}", style(format!("{:?}", rule_config.severity)).yellow()))?;
                term.write_line(&format!("   Tags: {}", style(rule_config.tags.join(", ")).dim()))?;
                
                if let Some(ref desc) = rule_config.description {
                    term.write_line(&format!("   Description: {}", desc))?;
                }
                
                if let Some(ref msg) = rule_config.message {
                    term.write_line(&format!("   Message Template: {}", style(msg).dim()))?;
                }
                
                term.write_line(&format!("   Min Confidence: {:.1}%", rule_config.min_confidence * 100.0))?;
                
                // Show recommendations
                let recommendations = BuiltinRules::get_recommendations(&rule_id);
                if !recommendations.is_empty() {
                    term.write_line("   Recommendations:")?;
                    for rec in &recommendations {
                        term.write_line(&format!("     • {}", rec))?;
                    }
                }
            } else {
                term.write_line(&format!("❌ Rule '{}' not found", style(&rule_id).red()))?;
                std::process::exit(1);
            }
        }
        
        RulesCommands::Init { output, strict } => {
            term.write_line(&format!("📄 Creating rules configuration: {}", output.display()))?;
            
            let config = if strict {
                RulesConfig::strict_profile()
            } else {
                RulesConfig::default()
            };
            
            config.export_to_file(&output)?;
            term.write_line(&format!("✅ Rules configuration created: {}", style(output.display()).green()))?;
        }
        
        RulesCommands::Validate { config } => {
            term.write_line(&format!("✅ Validating rules configuration: {}", config.display()))?;
            
            let rules_config = RulesConfig::from_file(&config)
                .with_context(|| format!("Failed to load rules from {}", config.display()))?;
            
            let manager = RulesManager::new_with_config(rules_config);
            let warnings = manager.validate_config()?;
            
            if warnings.is_empty() {
                term.write_line(&format!("   {}", style("Configuration is valid").green()))?;
            } else {
                term.write_line(&format!("   {}", style("Configuration has warnings:").yellow()))?;
                for warning in &warnings {
                    term.write_line(&format!("   - {}", style(warning).yellow()))?;
                }
            }
        }
        
        RulesCommands::ExportCustom { output } => {
            term.write_line(&format!("📄 Creating custom rules template: {}", output.display()))?;
            
            CustomRulesManager::create_example_file(&output)?;
            term.write_line(&format!("✅ Custom rules template created: {}", style(output.display()).green()))?;
            term.write_line("   Edit the file to add your custom rules and load with --custom-rules")?;
        }
    }
    
    Ok(())
}

async fn metrics_command(command: MetricsCommands) -> Result<()> {
    let term = Term::stdout();
    
    match command {
        MetricsCommands::Analyze { path, format, output, include_functions, include_debt } => {
            term.write_line(&format!("📊 {}", style("Code Quality Metrics Analysis").bold().cyan()))?;
            
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.cyan} {msg}")
                    .context("Failed to set progress style")?
            );
            pb.set_message("Loading configuration...");
            
            let config = Configuration::load_from_directory(&path)?;
            pb.set_message("Creating metrics manager...");
            
            let mut metrics_manager = QualityMetricsManager::new();
            
            pb.set_message("Analyzing modules...");
            
            let mut all_metrics = Vec::new();
            
            for module in config.get_modules() {
                if let Ok(content) = std::fs::read_to_string(&module.path) {
                    let metrics = metrics_manager.analyze_file(&module.path, &content)?;
                    all_metrics.push((module.path.clone(), metrics));
                }
            }
            
            pb.finish_and_clear();
            
            // Generate output
            match format.as_str() {
                "json" => {
                    let json_output = serde_json::to_string_pretty(&all_metrics)?;
                    if let Some(output_path) = output {
                        std::fs::write(&output_path, json_output)?;
                        term.write_line(&format!("Results written to {}", output_path.display()))?;
                    } else {
                        println!("{}", json_output);
                    }
                }
                _ => {
                    // Text format
                    for (file_path, metrics) in &all_metrics {
                        term.write_line(&format!("\n📄 {}", style(file_path.display()).bold()))?;
                        term.write_line(&format!("   Quality Score: {:.1}/100", style(metrics.quality_score).green()))?;
                        term.write_line(&format!("   Maintainability Index: {:.1}", metrics.maintainability_index))?;
                        term.write_line(&format!("   Average Complexity: {:.1}", metrics.complexity_metrics.average_cyclomatic_complexity))?;
                        
                        if include_debt && !metrics.technical_debt.debt_items.is_empty() {
                            term.write_line(&format!("   Technical Debt: {} items", metrics.technical_debt.debt_items.len()))?;
                        }
                        
                        if include_functions && !metrics.complexity_metrics.function_metrics.is_empty() {
                            term.write_line("   Functions:")?;
                            for (func_name, func_metrics) in &metrics.complexity_metrics.function_metrics {
                                term.write_line(&format!("     {} (complexity: {})", func_name, func_metrics.cyclomatic_complexity))?;
                            }
                        }
                    }
                }
            }
        }
        
        MetricsCommands::Report { path, output_dir } => {
            term.write_line(&format!("📊 Generating metrics reports in {}", output_dir.display()))?;
            
            std::fs::create_dir_all(&output_dir)?;
            
            let config = Configuration::load_from_directory(&path)?;
            let mut metrics_manager = QualityMetricsManager::new();
            
            let mut all_metrics = Vec::new();
            
            for module in config.get_modules() {
                if let Ok(content) = std::fs::read_to_string(&module.path) {
                    let metrics = metrics_manager.analyze_file(&module.path, &content)?;
                    all_metrics.push((module.path.clone(), metrics));
                }
            }
            
            // Generate JSON report
            let json_report = serde_json::to_string_pretty(&all_metrics)?;
            std::fs::write(output_dir.join("metrics.json"), json_report)?;
            
            term.write_line(&format!("✅ Metrics reports generated in {}", style(output_dir.display()).green()))?;
        }
        
        MetricsCommands::Test => {
            term.write_line(&format!("🧪 {}", style("Testing Metrics System").bold().cyan()))?;
            
            let sample_code = r#"
            Функция СложнаяФункция(Параметр1, Параметр2, Параметр3)
                Если Параметр1 > 0 Тогда
                    Для Счетчик = 1 По 10 Цикл
                        Если Счетчик % 2 = 0 Тогда
                            Возврат Счетчик * Параметр2;
                        КонецЕсли;
                    КонецЦикла;
                ИначеЕсли Параметр2 < 0 Тогда
                    Попытка
                        Результат = Параметр1 / Параметр2;
                    Исключение
                        Возврат Неопределено;
                    КонецПопытки;
                Иначе
                    Возврат Параметр3;
                КонецЕсли;
            КонецФункции
            "#;
            
            let mut metrics_manager = QualityMetricsManager::new();
            let metrics = metrics_manager.analyze_content("test.bsl", sample_code)?;
            
            term.write_line(&format!("📊 Quality Score: {:.1}/100", style(metrics.quality_score).green()))?;
            term.write_line(&format!("📈 Maintainability Index: {:.1}", metrics.maintainability_index))?;
            term.write_line(&format!("🔄 Average Complexity: {:.1}", metrics.complexity_metrics.average_cyclomatic_complexity))?;
            term.write_line(&format!("⚠️  Technical Debt Items: {}", metrics.technical_debt.debt_items.len()))?;
            
            if !metrics.recommendations.is_empty() {
                term.write_line("\n💡 Recommendations:")?;
                for rec in &metrics.recommendations {
                    term.write_line(&format!("   • {}", rec))?;
                }
            }
        }
    }
    
    Ok(())
}

async fn lsp_config_command(command: LspConfigCommands) -> Result<()> {
    let term = Term::stdout();
    
    match command {
        LspConfigCommands::Init { output, tcp, port } => {
            term.write_line(&format!("🔧 Creating LSP configuration: {}", output.display()))?;
            
            let lsp_config = if tcp {
                format!(r#"
# BSL Analyzer LSP Server Configuration

mode = "tcp"

[tcp]
host = "127.0.0.1"
port = {}
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
"#, port)
            } else {
                r#"
# BSL Analyzer LSP Server Configuration

mode = "stdio"

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
"#.to_string()
            };
            
            std::fs::write(&output, lsp_config.trim())?;
            term.write_line(&format!("✅ LSP configuration created: {}", style(output.display()).green()))?;
        }
        
        LspConfigCommands::Validate { config } => {
            term.write_line(&format!("✅ Validating LSP configuration: {}", config.display()))?;
            
            if !config.exists() {
                term.write_line(&format!("❌ Configuration file not found: {}", style(config.display()).red()))?;
                std::process::exit(1);
            }
            
            let content = std::fs::read_to_string(&config)?;
            match toml::from_str::<toml::Value>(&content) {
                Ok(_) => {
                    term.write_line(&format!("   {}", style("Configuration is valid").green()))?;
                }
                Err(e) => {
                    term.write_line(&format!("   {}: {}", style("Configuration error").red(), e))?;
                    std::process::exit(1);
                }
            }
        }
        
        LspConfigCommands::TestConnection { config: _config } => {
            term.write_line(&format!("🔌 {}", style("Testing LSP Server Connection").bold().cyan()))?;
            term.write_line("   This feature will be implemented in the next version")?;
            // TODO: Implement actual connection test
        }
    }
    
    Ok(())
}

async fn analyze_command(
    path: PathBuf,
    output: Option<PathBuf>,
    workers: Option<usize>,
    inter_module: bool,
    include_dependencies: bool,
    include_performance: bool,
    format: &str,
    rules_config: Option<PathBuf>,
    rules_profile: Option<String>,
    enable_enhanced_semantics: bool,
    platform_version: String,
    enable_method_validation: bool,
) -> Result<()> {
    let term = Term::stdout();
    term.write_line(&format!(
        "🦀 {} v{}", 
        style("BSL Analyzer").bold().cyan(),
        env!("CARGO_PKG_VERSION")
    ))?;
    
    let start_time = Instant::now();
    
    // Show progress
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .context("Failed to set progress style")?
    );
    pb.set_message("Loading configuration...");
    
    // Load configuration
    let config = Configuration::load_from_directory(&path)
        .with_context(|| format!("Failed to load configuration from {}", path.display()))?;
    
    pb.set_message(format!(
        "Found {} modules, {} objects", 
        config.modules.len(), 
        config.objects.len()
    ));
    
    // Load rules configuration
    pb.set_message("Loading rules configuration...");
    let mut rules_manager = if let Some(ref rules_path) = rules_config {
        RulesManager::from_file(rules_path)
            .with_context(|| format!("Failed to load rules from {}", rules_path.display()))?
    } else {
        // Use default rules
        RulesManager::new()
    };
    
    // Set active profile if specified
    if let Some(ref profile_name) = rules_profile {
        let mut config = rules_manager.config().clone();
        config.active_profile = profile_name.clone();
        rules_manager.reload_config_from_memory(config)?;
    }
    
    // Validate rules configuration
    let validation_warnings = rules_manager.validate_config()?;
    if !validation_warnings.is_empty() {
        term.write_line(&format!("⚠️  Rules validation warnings:"))?;
        for warning in &validation_warnings {
            term.write_line(&format!("   - {}", style(warning).yellow()))?;
        }
    }
    
    // Create analysis engine with enhanced semantics if requested
    let mut engine = if enable_enhanced_semantics || enable_method_validation {
        pb.set_message("Creating UnifiedBslIndex for enhanced analysis...");
        
        use bsl_analyzer::unified_index::UnifiedIndexBuilder;
        let mut builder = UnifiedIndexBuilder::new()?;
        let index = builder.build_index(&path, &platform_version)
            .context("Failed to create UnifiedBslIndex")?;
        
        term.write_line(&format!(
            "✅ UnifiedBslIndex created: {} entities loaded",
            style(index.get_all_entities().len()).green()
        ))?;
        
        BslAnalyzer::with_index(index)?
    } else {
        BslAnalyzer::new()?
    };
    
    if let Some(workers) = workers {
        engine.set_worker_count(workers);
    }
    engine.set_inter_module_analysis(inter_module);
    
    pb.set_message("Analyzing BSL files...");
    
    // NEW: Use file-by-file analysis instead of analyze_configuration
    let mut total_errors = 0;
    let mut total_warnings = 0;
    let mut analyzed_files = 0;
    let mut all_results = Vec::new();
    
    for module in config.get_modules() {
        if let Ok(content) = std::fs::read_to_string(&module.path) {
            match engine.analyze_code(&content, &module.path.to_string_lossy()) {
                Ok(()) => {
                    analyzed_files += 1;
                    let (errors, warnings) = engine.get_errors_and_warnings();
                    total_errors += errors.len();
                    total_warnings += warnings.len();
                    
                    // Store results for each file
                    all_results.push((module.path.clone(), errors, warnings));
                }
                Err(e) => {
                    term.write_line(&format!(
                        "❌ Failed to analyze {}: {}",
                        module.path.display(),
                        e
                    ))?;
                }
            }
        }
    }
    
    pb.finish_and_clear();
    
    let analysis_time = start_time.elapsed();
    
    // Convert results to AnalysisResults format for compatibility with reports
    let mut combined_results = AnalysisResults::new();
    for (_file_path, errors, warnings) in &all_results {
        for error in errors {
            combined_results.add_error(error.clone());
        }
        for warning in warnings {
            combined_results.add_warning(warning.clone());
        }
    }
    
    // Apply rules to filter and transform results
    let original_errors = total_errors;
    let original_warnings = total_warnings;
    
    if !rules_config.is_none() || rules_profile.is_some() {
        pb.set_message("Applying rules...");
        combined_results = rules_manager.apply_rules(&combined_results)
            .context("Failed to apply rules")?;
        
        let filtered_errors = combined_results.get_errors().len();
        let filtered_warnings = combined_results.get_warnings().len();
        
        // Report rules application results
        if original_errors != filtered_errors || original_warnings != filtered_warnings {
            let rules_summary = rules_manager.get_engine_summary();
            term.write_line(&format!(
                "🔧 Rules applied: {} enabled rules, {} → {} errors, {} → {} warnings",
                rules_summary.enabled_rules,
                original_errors, filtered_errors,
                original_warnings, filtered_warnings
            ))?;
        }
        
        // Update totals after rules application
        total_errors = filtered_errors;
        total_warnings = filtered_warnings;
    }
    
    // Output results
    if let Some(output_path) = output {
        // Generate report using ReportManager
        let report_config = ReportConfig {
            format: format.parse().unwrap_or(ReportFormat::Text),
            output_path: Some(output_path.to_string_lossy().to_string()),
            include_details: true,
            include_performance,
            include_dependencies,
            min_severity: None,
        };
        
        let report_manager = ReportManager::new();
        let report_content = report_manager.generate_with_config(&combined_results, &report_config)
            .context("Failed to generate report")?;
        
        std::fs::write(&output_path, report_content)
            .with_context(|| format!("Failed to write to {}", output_path.display()))?;
        term.write_line(&format!("Results written to {}", output_path.display()))?;
    } else {
        // For text output, use our new custom formatter
        match format {
            "text" => {
                print_new_text_results(&all_results, &term)?;
            }
            _ => {
                let report_config = ReportConfig {
                    format: format.parse().unwrap_or(ReportFormat::Json),
                    output_path: None,
                    include_details: true,
                    include_performance,
                    include_dependencies,
                    min_severity: None,
                };
                
                let report_manager = ReportManager::new();
                let report_content = report_manager.generate_with_config(&combined_results, &report_config)
                    .context("Failed to generate report")?;
                
                println!("{}", report_content);
            }
        }
    }
    
    // Enhanced summary with feature status
    term.write_line("")?;
    term.write_line(&format!("📊 {}", style("Analysis Summary").bold()))?;
    term.write_line(&format!("   Files analyzed: {}", style(analyzed_files).green()))?;
    term.write_line(&format!("   Errors found: {}", 
        if total_errors > 0 { 
            style(total_errors).red() 
        } else { 
            style(total_errors).green() 
        }
    ))?;
    term.write_line(&format!("   Warnings: {}", style(total_warnings).yellow()))?;
    term.write_line(&format!("   Analysis time: {:.2?}", style(analysis_time).dim()))?;
    
    // Show enhanced features status
    if enable_enhanced_semantics || enable_method_validation {
        term.write_line("")?;
        term.write_line(&format!("🚀 {}", style("Enhanced Features").bold().cyan()))?;
        term.write_line(&format!("   Enhanced semantics: {}", 
            if enable_enhanced_semantics { 
                style("✅ ENABLED").green() 
            } else { 
                style("❌ DISABLED").dim() 
            }
        ))?;
        term.write_line(&format!("   Method validation: {}", 
            if enable_method_validation { 
                style("✅ ENABLED").green() 
            } else { 
                style("❌ DISABLED").dim() 
            }
        ))?;
        term.write_line(&format!("   Platform version: {}", style(&platform_version).cyan()))?;
    } else {
        term.write_line("")?;
        term.write_line(&format!("💡 {}", style("Tip").bold().blue()))?;
        term.write_line("   Use --enable-enhanced-semantics for advanced BSL analysis with UnifiedBslIndex")?;
        term.write_line("   Use --enable-method-validation for method call validation")?;
    }
    
    // Exit with error code if issues found (standard behavior for static analyzers)
    if total_errors > 0 {
        std::process::exit(1);
    }
    
    Ok(())
}

async fn lsp_command(port: Option<u16>) -> Result<()> {
    info!("Starting LSP server...");
    
    if let Some(port) = port {
        // TCP server mode
        info!("LSP server listening on port {}", port);
        // TODO: Implement TCP LSP server
        anyhow::bail!("TCP LSP server not yet implemented");
    } else {
        // Stdio mode
        info!("LSP server starting in stdio mode");
        bsl_analyzer::lsp::start_stdio_server().await?;
    }
    
    Ok(())
}

async fn info_command(path: PathBuf) -> Result<()> {
    let term = Term::stdout();
    term.write_line(&format!("📁 {}", style("Configuration Info").bold().cyan()))?;
    
    let config = Configuration::load_from_directory(&path)
        .with_context(|| format!("Failed to load configuration from {}", path.display()))?;
    
    term.write_line(&format!("   Name: {}", style(&config.metadata.name).green()))?;
    term.write_line(&format!("   Version: {}", style(&config.metadata.version).green()))?;
    term.write_line(&format!("   Modules: {}", style(config.modules.len()).yellow()))?;
    term.write_line(&format!("   Objects: {}", style(config.objects.len()).yellow()))?;
    
    // Show module breakdown
    let mut module_types = std::collections::HashMap::new();
    for module in &config.modules {
        *module_types.entry(module.module_type.clone()).or_insert(0) += 1;
    }
    
    term.write_line("")?;
    term.write_line("📋 Module Types:")?;
    for (module_type, count) in module_types {
        term.write_line(&format!("   {}: {}", module_type, style(count).cyan()))?;
    }
    
    Ok(())
}

async fn validate_command(path: PathBuf) -> Result<()> {
    let term = Term::stdout();
    term.write_line(&format!("✅ {}", style("Configuration Validation").bold().green()))?;
    
    let config = Configuration::load_from_directory(&path)
        .with_context(|| format!("Failed to load configuration from {}", path.display()))?;
    
    let validation_result = config.validate()
        .context("Validation failed")?;
    
    if validation_result.is_valid() {
        term.write_line(&format!("   {}", style("Configuration is valid").green()))?;
    } else {
        term.write_line(&format!("   {}", style("Configuration has issues:").red()))?;
        for issue in validation_result.issues() {
            term.write_line(&format!("   - {}", style(issue).red()))?;
        }
        std::process::exit(1);
    }
    
    Ok(())
}

async fn generate_reports_command(
    path: PathBuf,
    output_dir: PathBuf,
    include_dependencies: bool,
    include_performance: bool,
) -> Result<()> {
    let term = Term::stdout();
    term.write_line(&format!(
        "📊 {} v{}", 
        style("BSL Reports Generator").bold().cyan(),
        env!("CARGO_PKG_VERSION")
    ))?;
    
    let start_time = Instant::now();
    
    // Create output directory
    std::fs::create_dir_all(&output_dir)
        .with_context(|| format!("Failed to create output directory: {}", output_dir.display()))?;
    
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .context("Failed to set progress style")?
    );
    pb.set_message("Loading configuration...");
    
    // Load configuration
    let config = Configuration::load_from_directory(&path)
        .with_context(|| format!("Failed to load configuration from {}", path.display()))?;
    
    pb.set_message("Analyzing configuration...");
    
    // Create analysis engine
    let mut engine = BslAnalyzer::new()?;
    engine.set_inter_module_analysis(true);
    
    // Run analysis
    let results = engine.analyze_configuration(&config)
        .context("Analysis failed")?;
    
    pb.set_message("Converting results...");
    
    // Convert engine results to AnalysisResults format
    let mut combined_results = AnalysisResults::new();
    for result in &results {
        for error in result.get_errors() {
            combined_results.add_error(error.clone());
        }
        for warning in result.get_warnings() {
            combined_results.add_warning(warning.clone());
        }
    }
    
    pb.set_message("Generating reports...");
    
    // Create report manager with configuration
    let report_config = ReportConfig {
        format: ReportFormat::Json, // Will be overridden for each format
        output_path: None,
        include_details: true,
        include_performance,
        include_dependencies,
        min_severity: None,
    };
    
    let report_manager = ReportManager::with_config(report_config);
    
    // Generate all format reports
    match report_manager.generate_all_formats(&combined_results, &output_dir) {
        Ok(()) => {
            pb.finish_and_clear();
            
            let generation_time = start_time.elapsed();
            
            // Report success
            term.write_line("")?;
            term.write_line(&format!("✅ {}", style("Reports Generated Successfully").bold().green()))?;
            term.write_line(&format!("   Output directory: {}", style(output_dir.display()).cyan()))?;
            term.write_line(&format!("   SARIF report: {}", style("analysis-results.sarif").yellow()))?;
            term.write_line(&format!("   JSON report: {}", style("analysis-results.json").yellow()))?;
            term.write_line(&format!("   HTML report: {}", style("analysis-results.html").yellow()))?;
            term.write_line(&format!("   Text report: {}", style("analysis-results.txt").yellow()))?;
            term.write_line("")?;
            term.write_line(&format!("📊 Summary:"))?;
            term.write_line(&format!("   Files analyzed: {}", style(results.len()).green()))?;
            term.write_line(&format!("   Errors found: {}", 
                if combined_results.get_errors().len() > 0 { 
                    style(combined_results.get_errors().len()).red() 
                } else { 
                    style(combined_results.get_errors().len()).green() 
                }
            ))?;
            term.write_line(&format!("   Warnings: {}", style(combined_results.get_warnings().len()).yellow()))?;
            term.write_line(&format!("   Generation time: {:.2?}", style(generation_time).dim()))?;
            
            // Special note about SARIF for GitHub Actions
            if combined_results.get_errors().len() > 0 || combined_results.get_warnings().len() > 0 {
                term.write_line("")?;
                term.write_line(&format!("💡 {}", style("GitHub Actions Integration:").bold().blue()))?;
                term.write_line("   Upload the SARIF file to GitHub Security tab:")?;
                term.write_line(&format!("   {} upload-sarif --sarif-file {}/analysis-results.sarif", 
                    style("gh api").dim(),
                    output_dir.display()
                ))?;
            }
        }
        Err(e) => {
            pb.finish_and_clear();
            return Err(e.context("Failed to generate reports"));
        }
    }
    
    Ok(())
}

fn print_new_text_results(
    results: &[(PathBuf, Vec<bsl_analyzer::AnalysisError>, Vec<bsl_analyzer::AnalysisError>)],
    term: &Term
) -> Result<()> {
    for (file_path, errors, warnings) in results {
        if !errors.is_empty() || !warnings.is_empty() {
            term.write_line(&format!(
                "\n📄 {}", 
                style(file_path.display()).bold()
            ))?;
            
            // Print errors
            for error in errors {
                term.write_line(&format!(
                    "   {}:{} {} {}", 
                    style(error.position.line).dim(),
                    style(error.position.column).dim(),
                    style("ERROR").red().bold(),
                    error.message
                ))?;
                
                if let Some(ref code) = error.error_code {
                    term.write_line(&format!("      Code: {}", style(code).dim()))?;
                }
            }
            
            // Print warnings  
            for warning in warnings {
                term.write_line(&format!(
                    "   {}:{} {} {}", 
                    style(warning.position.line).dim(),
                    style(warning.position.column).dim(),
                    style("WARN").yellow().bold(),
                    warning.message
                ))?;
                
                if let Some(ref code) = warning.error_code {
                    term.write_line(&format!("      Code: {}", style(code).dim()))?;
                }
            }
        }
    }
    
    Ok(())
}

#[allow(dead_code)]
fn print_text_results(
    results: &[AnalysisResults], 
    term: &Term
) -> Result<()> {
    for result in results {
        if result.has_issues() {
            term.write_line(&format!(
                "\n📄 {}", 
                style("unknown").bold() // TODO: добавить file_path в AnalysisResults
            ))?;
            
            // Print errors
            for error in result.get_errors() {
                term.write_line(&format!(
                    "   {}:{} {} {}", 
                    style(error.position.line).dim(),
                    style(error.position.column).dim(),
                    style("ERROR").red().bold(),
                    error.message
                ))?;
            }
            
            // Print warnings  
            for warning in result.get_warnings() {
                term.write_line(&format!(
                    "   {}:{} {} {}", 
                    style(warning.position.line).dim(),
                    style(warning.position.column).dim(),
                    style("WARN").yellow().bold(),
                    warning.message
                ))?;
            }
            
            // Print suggestions
            // Временно убираем suggestions пока не реализуем
            // for suggestion in &result.suggestions {
            //     term.write_line(&format!(
            //         "   {}:{} {} {}", 
            //         style(suggestion.line).dim(),
            //         style(suggestion.column).dim(),
            //         style("INFO").blue().bold(),
            //         suggestion.message
            //     ))?;
            // }
        }
    }
    
    Ok(())
}



async fn parse_docs_command(
    hbk_file: PathBuf,
    output_dir: PathBuf,
    analyze_structure: bool,
    extract_samples: bool,
) -> Result<()> {
    let term = Term::stdout();
    term.write_line(&format!(
        "📚 {} v{}", 
        style("Documentation Parser").bold().cyan(),
        env!("CARGO_PKG_VERSION")
    ))?;
    
    // Check if HBK file exists
    if !hbk_file.exists() {
        anyhow::bail!("HBK file not found: {}", hbk_file.display());
    }
    
    term.write_line(&format!("Processing: {}", hbk_file.display()))?;
    
    // Create output directory
    std::fs::create_dir_all(&output_dir)
        .with_context(|| format!("Failed to create output directory: {}", output_dir.display()))?;
    
    let start_time = Instant::now();
    
    // Create and use HBK parser - use the full version
    use bsl_analyzer::docs_integration::hbk_parser_full::HbkArchiveParser;
    
    let mut parser = HbkArchiveParser::new(&hbk_file);
    
    // Open the archive first
    parser.open_archive()
        .context("Failed to open HBK archive")?;
    
    // Analyze structure if requested
    if analyze_structure {
        term.write_line("")?;
        term.write_line(&style("📂 Analyzing archive structure...").yellow().to_string())?;
        
        let structure = parser.analyze_structure()
            .context("Failed to analyze HBK structure")?;
        
        // Save structure to JSON
        let structure_path = output_dir.join("hbk_structure.json");
        let json = serde_json::to_string_pretty(&structure)?;
        std::fs::write(&structure_path, json)?;
        
        term.write_line(&format!(
            "   Structure saved to: {}",
            structure_path.display()
        ))?;
    }
    
    // Extract sample files if requested
    if extract_samples {
        term.write_line("")?;
        term.write_line(&style("📄 Extracting sample files...").yellow().to_string())?;
        
        let samples = parser.extract_sample_files(10)
            .context("Failed to extract sample files")?;
        
        // Save samples to JSON
        let samples_path = output_dir.join("sample_contents.json");
        let json = serde_json::to_string_pretty(&samples)?;
        std::fs::write(&samples_path, json)?;
        
        term.write_line(&format!(
            "   Extracted {} sample files to: {}",
            samples.len(),
            samples_path.display()
        ))?;
    }
    
    // Always extract BSL syntax information
    term.write_line("")?;
    term.write_line(&style("🔍 Extracting BSL syntax information...").yellow().to_string())?;
    
    use bsl_analyzer::docs_integration::BslSyntaxExtractor;
    let extractor = BslSyntaxExtractor::new(&hbk_file);
    
    // List contents to find syntax files
    let contents = parser.list_contents();
    
    let mut syntax_count = 0;
    let mut all_syntax_info = Vec::new();
    
    for filename in contents {
        if filename.ends_with(".htm") || filename.ends_with(".html") {
            match parser.extract_file_content(&filename) {
                Some(content) => {
                    if let Ok(syntax_info) = extractor.extract_syntax_info(&content, &filename) {
                        if !syntax_info.syntax_variants.is_empty() {
                            syntax_count += 1;
                            all_syntax_info.push(syntax_info);
                        }
                    }
                }
                None => {}
            }
        }
    }
    
    // Save syntax information
    if !all_syntax_info.is_empty() {
        let syntax_path = output_dir.join("bsl_syntax.json");
        let json = serde_json::to_string_pretty(&all_syntax_info)?;
        std::fs::write(&syntax_path, json)?;
        
        term.write_line(&format!(
            "   Extracted {} syntax definitions to: {}",
            syntax_count,
            syntax_path.display()
        ))?;
    }
    
    let elapsed = start_time.elapsed();
    
    term.write_line("")?;
    term.write_line(&format!(
        "✅ Documentation parsing completed in {:.2}s",
        elapsed.as_secs_f64()
    ))?;
    
    Ok(())
}
