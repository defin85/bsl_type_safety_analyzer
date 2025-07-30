use std::path::PathBuf;
use anyhow::{Result, Context};
use clap::Parser;
use bsl_analyzer::unified_index::{UnifiedIndexBuilder};

#[derive(Parser, Debug)]
#[command(author, version, about = "Build unified BSL type index from 1C configuration", long_about = None)]
struct Args {
    /// Path to 1C configuration directory
    #[arg(short, long)]
    config: PathBuf,
    
    /// Platform version (e.g., "8.3.25")
    #[arg(short, long)]
    platform_version: String,
    
    /// Output directory for the index (optional)
    #[arg(short, long)]
    output: Option<PathBuf>,
    
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Initialize logging
    if args.verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }
    
    // Validate config path
    if !args.config.exists() {
        return Err(anyhow::anyhow!(
            "Configuration path does not exist: {}",
            args.config.display()
        ));
    }
    
    if !args.config.is_dir() {
        return Err(anyhow::anyhow!(
            "Configuration path must be a directory: {}",
            args.config.display()
        ));
    }
    
    println!("Building unified BSL index...");
    println!("Configuration: {}", args.config.display());
    println!("Platform version: {}", args.platform_version);
    
    // Create builder
    let builder = UnifiedIndexBuilder::new()
        .context("Failed to create index builder")?;
    
    // Build index
    let start = std::time::Instant::now();
    let index = builder.build_index(&args.config, &args.platform_version)
        .context("Failed to build unified index")?;
    let elapsed = start.elapsed();
    
    println!("\n✅ Index built successfully!");
    println!("Total entities: {}", index.get_entity_count());
    println!("Build time: {:.2?}", elapsed);
    
    // Show statistics by type
    println!("\nEntity statistics:");
    use bsl_analyzer::unified_index::{BslEntityType, BslEntityKind};
    
    let platform_count = index.get_entities_by_type(&BslEntityType::Platform).len();
    let config_count = index.get_entities_by_type(&BslEntityType::Configuration).len();
    let form_count = index.get_entities_by_type(&BslEntityType::Form).len();
    let module_count = index.get_entities_by_type(&BslEntityType::Module).len();
    
    println!("  Platform types: {}", platform_count);
    println!("  Configuration objects: {}", config_count);
    println!("  Forms: {}", form_count);
    println!("  Modules: {}", module_count);
    
    // Show statistics by kind
    println!("\nConfiguration objects by type:");
    let kinds = [
        (BslEntityKind::Catalog, "Catalogs"),
        (BslEntityKind::Document, "Documents"),
        (BslEntityKind::InformationRegister, "Information registers"),
        (BslEntityKind::AccumulationRegister, "Accumulation registers"),
        (BslEntityKind::ChartOfAccounts, "Charts of accounts"),
        (BslEntityKind::ChartOfCharacteristicTypes, "Charts of characteristic types"),
        (BslEntityKind::ChartOfCalculationTypes, "Charts of calculation types"),
        (BslEntityKind::BusinessProcess, "Business processes"),
        (BslEntityKind::Task, "Tasks"),
        (BslEntityKind::ExchangePlan, "Exchange plans"),
        (BslEntityKind::CommonModule, "Common modules"),
    ];
    
    for (kind, name) in &kinds {
        let count = index.get_entities_by_kind(kind).len();
        if count > 0 {
            println!("  {}: {}", name, count);
        }
    }
    
    // Save index if output specified
    if let Some(output_dir) = args.output {
        println!("\nSaving index to: {}", output_dir.display());
        
        std::fs::create_dir_all(&output_dir)
            .context("Failed to create output directory")?;
            
        let index_file = output_dir.join("unified_index.json");
        let index_json = serde_json::to_string_pretty(&index)
            .context("Failed to serialize index")?;
            
        std::fs::write(&index_file, index_json)
            .context("Failed to write index file")?;
            
        println!("✅ Index saved to: {}", index_file.display());
    }
    
    // Example queries
    if args.verbose {
        println!("\n=== Example queries ===");
        
        // Find a specific type
        if let Some(array_type) = index.find_entity("Массив") {
            println!("\nFound type 'Массив':");
            println!("  Methods: {}", array_type.interface.methods.len());
            println!("  Properties: {}", array_type.interface.properties.len());
        }
        
        // Find types with specific method
        let types_with_insert = index.find_types_with_method("Вставить");
        if !types_with_insert.is_empty() {
            println!("\nTypes with method 'Вставить':");
            for entity in types_with_insert.iter().take(5) {
                println!("  - {}", entity.qualified_name);
            }
            if types_with_insert.len() > 5 {
                println!("  ... and {} more", types_with_insert.len() - 5);
            }
        }
    }
    
    Ok(())
}