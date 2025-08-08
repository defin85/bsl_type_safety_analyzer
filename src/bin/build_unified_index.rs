use anyhow::{Context, Result};
use bsl_analyzer::cli_common::{self, CliCommand, OutputFormat, OutputWriter, ProgressReporter};
use bsl_analyzer::unified_index::{
    BslApplicationMode, BslEntityKind, BslEntityType, UnifiedIndexBuilder,
};
use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(ValueEnum, Debug, Clone)]
enum ApplicationMode {
    /// Обычное приложение (8.1) - обычные формы, нет директив компиляции
    Ordinary,
    /// Управляемое приложение (8.2+) - управляемые формы, директивы &НаСервере и т.д.
    Managed,
    /// Смешанный режим - поддержка обоих типов форм
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
#[command(author, version, about = "Build unified BSL type index from 1C configuration", long_about = None)]
struct Args {
    /// Path to 1C configuration directory
    #[arg(short, long)]
    config: PathBuf,

    /// Platform version (e.g., "8.3.25")
    #[arg(short, long)]
    platform_version: String,

    /// Application mode (ordinary/managed/mixed)
    #[arg(short = 'm', long, value_enum, default_value = "managed")]
    mode: ApplicationMode,

    /// Output directory for the index (optional)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Path to platform documentation archive (optional)
    #[arg(long)]
    platform_docs_archive: Option<PathBuf>,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Output format (text, json, table)
    #[arg(long, default_value = "text")]
    format: String,

    /// Show detailed statistics
    #[arg(long)]
    detailed: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    cli_common::init_logging(args.verbose)?;

    // Create command and run
    let command = BuildIndexCommand::new(args);
    cli_common::run_command(command)
}

struct BuildIndexCommand {
    args: Args,
}

impl BuildIndexCommand {
    fn new(args: Args) -> Self {
        Self { args }
    }
}

impl CliCommand for BuildIndexCommand {
    fn name(&self) -> &str {
        "build-unified-index"
    }

    fn description(&self) -> &str {
        "Build unified BSL type index from 1C configuration"
    }

    fn execute(&self) -> Result<()> {
        self.build_index()
    }
}

impl BuildIndexCommand {
    fn build_index(&self) -> Result<()> {
        // Validate config path
        cli_common::validate_path(&self.args.config, "Configuration directory")?;

        if !self.args.config.is_dir() {
            return Err(anyhow::anyhow!(
                "Configuration path must be a directory: {}",
                self.args.config.display()
            ));
        }

        // Create output writer
        let format = OutputFormat::parse_output_format(&self.args.format)?;
        let mut writer = if let Some(output_path) = &self.args.output {
            cli_common::ensure_dir_exists(output_path)?;
            let output_file = output_path.join("index_stats.txt");
            OutputWriter::file(&output_file, format)?
        } else {
            OutputWriter::stdout(format)
        };

        writer.write_header("Building Unified BSL Index")?;
        writer.write_line(&format!("Configuration: {}", self.args.config.display()))?;
        writer.write_line(&format!("Platform version: {}", self.args.platform_version))?;
        writer.write_line(&format!("Application mode: {:?}", self.args.mode))?;

        // Create builder with application mode and optional archive
        let mut builder = UnifiedIndexBuilder::new()
            .context("Failed to create index builder")?
            .with_application_mode(self.args.mode.clone().into())
            .with_platform_docs_archive(self.args.platform_docs_archive.clone());

        // Build index with progress reporting
        let start = std::time::Instant::now();

        let progress = if self.args.verbose {
            Some(ProgressReporter::new(100, "Building index"))
        } else {
            None
        };

        let index = builder
            .build_index(&self.args.config, &self.args.platform_version)
            .context("Failed to build unified index")?;

        if let Some(p) = progress {
            p.finish();
        }

        let elapsed = start.elapsed();

        // Write success message
        writer.write_line("")?;
        cli_common::print_success(&format!(
            "Index built successfully! {} entities in {}",
            index.get_entity_count(),
            cli_common::format_duration(elapsed)
        ));

        // Show statistics
        self.write_statistics(&mut writer, &index)?;

        // Save index if output specified
        if let Some(output_dir) = &self.args.output {
            self.save_index(output_dir, &index)?;
        }

        // Show example queries in verbose mode
        if self.args.verbose {
            self.show_example_queries(&mut writer, &index)?;
        }

        writer.flush()?;
        Ok(())
    }

    fn write_statistics(
        &self,
        writer: &mut OutputWriter,
        index: &bsl_analyzer::unified_index::UnifiedBslIndex,
    ) -> Result<()> {
        writer.write_header("Entity Statistics")?;

        // Statistics by type
        let platform_count = index.get_entities_by_type(&BslEntityType::Platform).len();
        let config_count = index
            .get_entities_by_type(&BslEntityType::Configuration)
            .len();
        let form_count = index.get_entities_by_type(&BslEntityType::Form).len();
        let module_count = index.get_entities_by_type(&BslEntityType::Module).len();

        // Create table for statistics
        let headers = vec!["Entity Type", "Count"];
        let rows = vec![
            vec!["Platform types".to_string(), platform_count.to_string()],
            vec![
                "Configuration objects".to_string(),
                config_count.to_string(),
            ],
            vec!["Forms".to_string(), form_count.to_string()],
            vec!["Modules".to_string(), module_count.to_string()],
        ];

        writer.write_table(&headers, rows)?;

        // Detailed statistics by kind
        if self.args.detailed {
            writer.write_header("Configuration Objects by Type")?;

            let kinds = [
                (BslEntityKind::Catalog, "Catalogs"),
                (BslEntityKind::Document, "Documents"),
                (BslEntityKind::InformationRegister, "Information registers"),
                (
                    BslEntityKind::AccumulationRegister,
                    "Accumulation registers",
                ),
                (BslEntityKind::ChartOfAccounts, "Charts of accounts"),
                (
                    BslEntityKind::ChartOfCharacteristicTypes,
                    "Charts of characteristic types",
                ),
                (
                    BslEntityKind::ChartOfCalculationTypes,
                    "Charts of calculation types",
                ),
                (BslEntityKind::BusinessProcess, "Business processes"),
                (BslEntityKind::Task, "Tasks"),
                (BslEntityKind::ExchangePlan, "Exchange plans"),
                (BslEntityKind::Constant, "Constants"),
                (BslEntityKind::Enum, "Enums"),
                (BslEntityKind::Report, "Reports"),
                (BslEntityKind::DataProcessor, "Data processors"),
                (BslEntityKind::DocumentJournal, "Document journals"),
                (BslEntityKind::CommonModule, "Common modules"),
            ];

            let mut detail_rows = Vec::new();
            for (kind, name) in &kinds {
                let count = index.get_entities_by_kind(kind).len();
                if count > 0 {
                    detail_rows.push(vec![name.to_string(), count.to_string()]);
                }
            }

            if !detail_rows.is_empty() {
                writer.write_table(&["Object Type", "Count"], detail_rows)?;
            }
        }

        Ok(())
    }

    fn save_index(
        &self,
        output_dir: &PathBuf,
        index: &bsl_analyzer::unified_index::UnifiedBslIndex,
    ) -> Result<()> {
        cli_common::print_warning(&format!("Saving index to: {}", output_dir.display()));

        std::fs::create_dir_all(output_dir).context("Failed to create output directory")?;

        let index_file = output_dir.join("unified_index.json");
        let index_json =
            serde_json::to_string_pretty(index).context("Failed to serialize index")?;

        let json_size = index_json.len() as u64;
        std::fs::write(&index_file, index_json).context("Failed to write index file")?;

        cli_common::print_success(&format!(
            "Index saved to: {} ({})",
            index_file.display(),
            cli_common::format_file_size(json_size)
        ));

        Ok(())
    }

    fn show_example_queries(
        &self,
        writer: &mut OutputWriter,
        index: &bsl_analyzer::unified_index::UnifiedBslIndex,
    ) -> Result<()> {
        writer.write_header("Example Queries")?;

        // Find a specific type
        if let Some(array_type) = index.find_entity("Массив") {
            writer.write_line("\nFound type 'Массив':")?;
            writer.write_list_item(&format!("Methods: {}", array_type.interface.methods.len()))?;
            writer.write_list_item(&format!(
                "Properties: {}",
                array_type.interface.properties.len()
            ))?;
        }

        // Find types with specific method
        let types_with_insert = index.find_types_with_method("Вставить");
        if !types_with_insert.is_empty() {
            writer.write_line("\nTypes with method 'Вставить':")?;
            for entity in types_with_insert.iter().take(5) {
                writer.write_list_item(&entity.qualified_name)?;
            }
            if types_with_insert.len() > 5 {
                writer.write_line(&format!("... and {} more", types_with_insert.len() - 5))?;
            }
        }

        Ok(())
    }
}
