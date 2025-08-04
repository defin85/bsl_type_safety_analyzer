/*!
# Incremental Index Update CLI

CLI инструмент для инкрементального обновления UnifiedBslIndex
без полной перестройки.

## Использование

```bash
# Обновление индекса после изменений в конфигурации
cargo run --bin incremental_update -- --config "path/to/config" --platform-version "8.3.25"

# С детальным выводом
cargo run --bin incremental_update -- --config "path/to/config" --platform-version "8.3.25" --verbose

# Проверка изменений без обновления (dry-run)
cargo run --bin incremental_update -- --config "path/to/config" --platform-version "8.3.25" --dry-run
```

## Производительность

- Полная перестройка: ~500ms для тестовой конфигурации
- Инкрементальное обновление: ~1-20ms для малых изменений
- Целевая производительность: <200ms для Enterprise конфигураций (80,000+ объектов)
*/

use std::path::PathBuf;
use anyhow::{Result, Context};
use clap::{Parser, ValueEnum};
use bsl_analyzer::unified_index::{UnifiedIndexBuilder, BslApplicationMode, ChangeImpact};

#[derive(ValueEnum, Debug, Clone)]
enum ApplicationMode {
    /// Обычное приложение (8.1)
    Ordinary,
    /// Управляемое приложение (8.2+)
    Managed,
    /// Смешанный режим
    Mixed,
}

impl From<ApplicationMode> for BslApplicationMode {
    fn from(mode: ApplicationMode) -> Self {
        match mode {
            ApplicationMode::Ordinary => BslApplicationMode::OrdinaryApplication,
            ApplicationMode::Managed => BslApplicationMode::ManagedApplication,
            ApplicationMode::Mixed => BslApplicationMode::MixedMode,
        }
    }
}

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Incremental update of BSL type index",
    long_about = "Performs incremental updates to UnifiedBslIndex when configuration files change, avoiding full rebuild when possible."
)]
struct Args {
    /// Path to 1C configuration directory
    #[arg(short, long)]
    config: PathBuf,
    
    /// Platform version (e.g., "8.3.25")
    #[arg(short, long)]
    platform_version: String,
    
    /// Application mode
    #[arg(short = 'm', long, value_enum, default_value = "managed")]
    mode: ApplicationMode,
    
    /// Show detailed output about changes
    #[arg(short, long)]
    verbose: bool,
    
    /// Only check for changes, don't update index (dry-run)
    #[arg(short = 'n', long)]
    dry_run: bool,
    
    /// Force incremental update even if full rebuild is recommended
    #[arg(short, long)]
    force_incremental: bool,
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
    
    println!("🔄 BSL Index Incremental Update");
    println!("Configuration: {}", args.config.display());
    println!("Platform version: {}", args.platform_version);
    
    // Create builder
    let mut builder = UnifiedIndexBuilder::new()
        .context("Failed to create index builder")?
        .with_application_mode(args.mode.into());
    
    let start = std::time::Instant::now();
    
    // Check what kind of changes exist
    println!("\n📋 Analyzing configuration changes...");
    
    // First, try to load existing index to see if incremental update is possible
    match builder.check_incremental_update_feasibility(&args.config, &args.platform_version) {
        Ok((change_impact, changed_files)) => {
            println!("Change impact: {:?}", change_impact);
            
            if !changed_files.is_empty() {
                println!("Changed files ({}):", changed_files.len());
                for (file, impact) in &changed_files {
                    if args.verbose {
                        println!("  - {} ({:?})", file.display(), impact);
                    }
                }
                if !args.verbose && changed_files.len() > 5 {
                    println!("  ... and {} more files", changed_files.len() - 5);
                }
            } else {
                println!("✅ No changes detected");
                return Ok(());
            }
            
            if args.dry_run {
                println!("\n🔍 Dry run mode - no changes will be made");
                let recommendation = match change_impact {
                    ChangeImpact::None => "No update needed",
                    ChangeImpact::Minor => "Incremental update (few milliseconds)",
                    ChangeImpact::ModuleUpdate => "Incremental update (~10-50ms)",
                    ChangeImpact::MetadataUpdate => "Incremental update (~50-200ms)",
                    ChangeImpact::FullRebuild => "Full rebuild recommended (~500ms)",
                };
                println!("Recommendation: {}", recommendation);
                return Ok(());
            }
            
            // Perform update based on change impact
            match change_impact {
                ChangeImpact::None => {
                    println!("✅ Index is up to date");
                    return Ok(());
                }
                ChangeImpact::FullRebuild if !args.force_incremental => {
                    println!("⚠️  Full rebuild required due to Configuration.xml changes");
                    println!("   Use --force-incremental to attempt incremental update anyway");
                    
                    let index = builder.build_index(&args.config, &args.platform_version)
                        .context("Failed to rebuild index")?;
                    
                    let elapsed = start.elapsed();
                    println!("✅ Full rebuild completed in {:.2?}", elapsed);
                    println!("Total entities: {}", index.get_entity_count());
                    
                    return Ok(());
                }
                _ => {
                    // Perform incremental update
                    println!("🚀 Performing incremental update...");
                    
                    let update_result = builder.perform_incremental_update(&args.config, &args.platform_version, changed_files)
                        .context("Failed to perform incremental update")?;
                    
                    let elapsed = start.elapsed();
                    
                    if update_result.success {
                        println!("✅ Incremental update completed in {:.2?}", elapsed);
                        println!("Changes: {} added, {} updated, {} removed", 
                            update_result.added_entities.len(),
                            update_result.updated_entities.len(),
                            update_result.removed_entities.len());
                        
                        if args.verbose && update_result.total_changes() > 0 {
                            println!("\nDetailed changes:");
                            for entity in &update_result.added_entities {
                                println!("  + Added: {}", entity);
                            }
                            for entity in &update_result.updated_entities {
                                println!("  ~ Updated: {}", entity);
                            }
                            for entity in &update_result.removed_entities {
                                println!("  - Removed: {}", entity);
                            }
                        }
                    } else {
                        println!("❌ Incremental update failed");
                        return Err(anyhow::anyhow!("Update was not successful"));
                    }
                }
            }
        }
        Err(e) => {
            println!("⚠️  Cannot perform incremental update: {}", e);
            println!("   Falling back to full rebuild...");
            
            let index = builder.build_index(&args.config, &args.platform_version)
                .context("Failed to build index")?;
            
            let elapsed = start.elapsed();
            println!("✅ Full rebuild completed in {:.2?}", elapsed);
            println!("Total entities: {}", index.get_entity_count());
        }
    }
    
    Ok(())
}