//! Извлекает документацию платформы из архива синтаксис-помощника
//!
//! Использование:
//! ```bash
//! cargo run --bin extract_platform_docs -- --archive "path/to/rebuilt.shcntx_ru.zip" --version "8.3.25"
//! ```

use anyhow::{Context, Result};
use bsl_analyzer::cli_common::{self, CliCommand, OutputFormat, OutputWriter, ProgressReporter};
use bsl_analyzer::docs_integration::BslSyntaxExtractor;
use bsl_analyzer::unified_index::{converters::SyntaxDbConverter, PlatformDocsCache};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "extract_platform_docs")]
#[command(about = "Extract platform documentation from 1C help archives")]
struct Args {
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Output format (text, json, table)
    #[arg(long, default_value = "text")]
    format: String,
    /// Path to 1C documentation archive (e.g., rebuilt.shcntx_ru.zip)
    #[arg(short, long)]
    archive: PathBuf,

    /// Platform version
    #[arg(short = 'p', long = "platform-version")]
    version: String,

    /// Force re-extraction even if cache exists
    #[arg(short, long)]
    force: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    cli_common::init_logging(args.verbose)?;

    // Create command and run
    let command = ExtractDocsCommand::new(args);
    cli_common::run_command(command)
}

struct ExtractDocsCommand {
    args: Args,
}

impl ExtractDocsCommand {
    fn new(args: Args) -> Self {
        Self { args }
    }
}

impl CliCommand for ExtractDocsCommand {
    fn name(&self) -> &str {
        "extract-platform-docs"
    }

    fn description(&self) -> &str {
        "Extract platform documentation from 1C help archives"
    }

    fn execute(&self) -> Result<()> {
        self.extract_docs()
    }
}

impl ExtractDocsCommand {
    fn extract_docs(&self) -> Result<()> {
        // Create output writer
        let format = OutputFormat::parse_output_format(&self.args.format)?;
        let mut writer = OutputWriter::stdout(format);

        writer.write_header("BSL Platform Documentation Extractor")?;
        writer.write_line(&format!("Archive: {}", self.args.archive.display()))?;
        writer.write_line(&format!("Platform version: {}", self.args.version))?;

        // Check if archive exists
        cli_common::validate_path(&self.args.archive, "Archive file")?;

        // Initialize cache
        let cache = PlatformDocsCache::new()?;

        // Check if already cached (unless force flag is set)
        if !self.args.force {
            let cache_file = cache_dir()?.join(format!("{}.jsonl", self.args.version));
            if cache_file.exists() {
                cli_common::print_success(&format!(
                    "Platform documentation already cached for version {}",
                    self.args.version
                ));
                writer.write_line(&format!("Cache file: {}", cache_file.display()))?;
                writer.write_line("Use --force to re-extract")?;

                // Load and show statistics
                let entities = cache.get_or_create(&self.args.version)?;
                self.show_statistics(&mut writer, &entities)?;
                return Ok(());
            }
        }

        // Extract syntax database from archive
        writer.write_line("")?;
        writer.write_header("Extracting platform documentation from archive...")?;

        let progress = if self.args.verbose {
            Some(ProgressReporter::new(100, "Extracting documentation"))
        } else {
            None
        };

        let mut extractor = BslSyntaxExtractor::new(&self.args.archive);
        let syntax_db = extractor
            .extract_syntax_database(None)
            .context("Failed to extract BSL syntax database")?;

        if let Some(p) = progress {
            p.finish();
        }

        // Show extraction statistics
        writer.write_header("Extracted syntax database")?;
        let headers = vec!["Category", "Count"];
        let rows = vec![
            vec!["Objects".to_string(), syntax_db.objects.len().to_string()],
            vec!["Methods".to_string(), syntax_db.methods.len().to_string()],
            vec![
                "Properties".to_string(),
                syntax_db.properties.len().to_string(),
            ],
            vec![
                "Functions".to_string(),
                syntax_db.functions.len().to_string(),
            ],
            vec![
                "Operators".to_string(),
                syntax_db.operators.len().to_string(),
            ],
            vec!["Keywords".to_string(), syntax_db.keywords.len().to_string()],
        ];
        writer.write_table(&headers, rows)?;

        // Convert to BSL entities using unified converter
        writer.write_header("Converting to unified BSL entities...")?;
        let entities = convert_syntax_db_to_entities(&syntax_db, &self.args.version)?;

        cli_common::print_success(&format!("Converted {} platform entities", entities.len()));

        // Save to cache
        writer.write_header("Saving to platform cache...")?;
        cache.save_to_cache(&self.args.version, &entities)?;

        let cache_file = cache_dir()?.join(format!("{}.jsonl", self.args.version));
        cli_common::print_success("Platform documentation cached successfully!");
        writer.write_line(&format!("Cache file: {}", cache_file.display()))?;
        writer.write_line(&format!("Total entities: {}", entities.len()))?;

        // Show detailed statistics
        self.show_statistics(&mut writer, &entities)?;

        writer.flush()?;
        Ok(())
    }

    fn show_statistics(
        &self,
        writer: &mut OutputWriter,
        entities: &[bsl_analyzer::unified_index::BslEntity],
    ) -> Result<()> {
        use bsl_analyzer::unified_index::BslEntityKind;
        use std::collections::HashMap;

        let mut categories: HashMap<String, usize> = HashMap::new();

        for entity in entities {
            let category = match entity.entity_kind {
                BslEntityKind::Primitive => "Primitive types",
                BslEntityKind::Array => "Arrays",
                BslEntityKind::Structure => "Structures",
                BslEntityKind::Map => "Maps",
                BslEntityKind::ValueList => "Value lists",
                BslEntityKind::ValueTable => "Value tables",
                BslEntityKind::ValueTree => "Value trees",
                BslEntityKind::Global => "Global context",
                BslEntityKind::System => "System types",
                _ => "Other",
            };
            *categories.entry(category.to_string()).or_insert(0) += 1;
        }

        if !categories.is_empty() {
            writer.write_header("Entity statistics by category")?;
            let mut rows = Vec::new();
            for (category, count) in categories {
                rows.push(vec![category, count.to_string()]);
            }
            rows.sort_by(|a, b| {
                b[1].parse::<usize>()
                    .unwrap()
                    .cmp(&a[1].parse::<usize>().unwrap())
            });
            writer.write_table(&["Category", "Count"], rows)?;
        }

        Ok(())
    }
}

fn convert_syntax_db_to_entities(
    syntax_db: &bsl_analyzer::docs_integration::BslSyntaxDatabase,
    version: &str,
) -> Result<Vec<bsl_analyzer::unified_index::BslEntity>> {
    let converter = SyntaxDbConverter::new(version);
    converter.convert(syntax_db)
}

fn cache_dir() -> Result<PathBuf> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
    Ok(home_dir.join(".bsl_analyzer").join("platform_cache"))
}
