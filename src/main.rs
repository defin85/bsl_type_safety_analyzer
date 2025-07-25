/*!
# BSL Analyzer CLI

Command-line interface for BSL (1C:Enterprise) static analyzer.
*/

use anyhow::{Context, Result};
use bsl_analyzer::{Configuration, AnalysisEngine};
use bsl_analyzer::analyzer::AnalysisResult;
use clap::{Parser, Subcommand};
use console::{style, Term};
use indicatif::{ProgressBar, ProgressStyle};
use serde_json;
use std::path::PathBuf;
use std::time::Instant;
use tracing::{info, warn};
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
    
    /// Output format (json, text, sarif)
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
            inter_module 
        } => {
            analyze_command(path, output, workers, inter_module, &cli.format).await?;
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
    }
    
    Ok(())
}

async fn analyze_command(
    path: PathBuf,
    output: Option<PathBuf>,
    workers: Option<usize>,
    inter_module: bool,
    format: &str,
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
    
    // Create analysis engine
    let mut engine = AnalysisEngine::new();
    if let Some(workers) = workers {
        engine.set_worker_count(workers);
    }
    engine.set_inter_module_analysis(inter_module);
    
    pb.set_message("Analyzing configuration...");
    
    // Run analysis
    let results = engine.analyze_configuration(&config)
        .await
        .context("Analysis failed")?;
    
    pb.finish_and_clear();
    
    let analysis_time = start_time.elapsed();
    
    // Collect statistics
    let total_files = results.len();
    let total_errors: usize = results.iter().map(|r| r.errors.len()).sum();
    let total_warnings: usize = results.iter().map(|r| r.warnings.len()).sum();
    let total_suggestions: usize = 0; // results.iter().map(|r| r.suggestions.len()).sum();
    
    // Output results
    match format {
        "json" => {
            let json_output = serde_json::to_string_pretty(&results)
                .context("Failed to serialize results to JSON")?;
                
            if let Some(output_path) = output {
                std::fs::write(&output_path, json_output)
                    .with_context(|| format!("Failed to write to {}", output_path.display()))?;
                term.write_line(&format!("Results written to {}", output_path.display()))?;
            } else {
                println!("{}", json_output);
            }
        }
        
        "text" => {
            print_text_results(&results, &term)?;
        }
        
        "sarif" => {
            // TODO: Implement SARIF format
            warn!("SARIF format not yet implemented, falling back to JSON");
            let json_output = serde_json::to_string_pretty(&results)
                .context("Failed to serialize results to JSON")?;
            println!("{}", json_output);
        }
        
        _ => {
            anyhow::bail!("Unsupported format: {}. Use json, text, or sarif", format);
        }
    }
    
    // Print summary
    term.write_line("")?;
    term.write_line(&format!("📊 {}", style("Analysis Summary").bold()))?;
    term.write_line(&format!("   Files analyzed: {}", style(total_files).green()))?;
    term.write_line(&format!("   Errors found: {}", 
        if total_errors > 0 { 
            style(total_errors).red() 
        } else { 
            style(total_errors).green() 
        }
    ))?;
    term.write_line(&format!("   Warnings: {}", style(total_warnings).yellow()))?;
    term.write_line(&format!("   Suggestions: {}", style(total_suggestions).blue()))?;
    term.write_line(&format!("   Analysis time: {:.2?}", style(analysis_time).dim()))?;
    
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

fn print_text_results(
    results: &[AnalysisResult], 
    term: &Term
) -> Result<()> {
    for result in results {
        if result.has_issues() {
            term.write_line(&format!(
                "\n📄 {}", 
                style(result.file_path.display()).bold()
            ))?;
            
            // Print errors
            for error in &result.errors {
                term.write_line(&format!(
                    "   {}:{} {} {}", 
                    style(error.position.line).dim(),
                    style(error.position.column).dim(),
                    style("ERROR").red().bold(),
                    error.message
                ))?;
            }
            
            // Print warnings  
            for warning in &result.warnings {
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
