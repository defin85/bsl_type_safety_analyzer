/// Проверка совместимости BSL типов
///
/// Утилита для проверки совместимости типов в рамках системы типов BSL.
/// Использует UnifiedBslIndex для определения возможности присваивания одного типа другому.
///
/// Пример использования:
/// ```bash
/// cargo run --bin check_type_compatibility -- --from "Справочники.Номенклатура" --to "СправочникСсылка" --config "path/to/config"
/// ```
use anyhow::Result;
use bsl_analyzer::unified_index::UnifiedIndexBuilder;
use bsl_analyzer::cli_common::{self, OutputWriter, OutputFormat, CliCommand};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "check_type_compatibility")]
#[command(about = "Проверка совместимости BSL типов")]
#[command(
    long_about = "Утилита для проверки совместимости типов в системе типов BSL. \
Проверяет возможность присваивания значения одного типа переменной другого типа."
)]
struct Args {
    /// Исходный тип (от какого типа преобразуем)
    #[arg(long, help = "Исходный тип для проверки")]
    from: String,

    /// Целевой тип (к какому типу преобразуем)
    #[arg(long, help = "Целевой тип для проверки")]
    to: String,

    /// Путь к конфигурации 1С
    #[arg(long, help = "Путь к конфигурации 1С")]
    config: PathBuf,

    /// Версия платформы 1С
    #[arg(long, default_value = "8.3.25", help = "Версия платформы 1С")]
    platform_version: String,

    /// Подробный вывод с объяснением совместимости
    #[arg(short, long, help = "Подробный анализ совместимости")]
    verbose: bool,

    /// Показать путь наследования
    #[arg(long, help = "Показать путь наследования между типами")]
    show_inheritance_path: bool,
    
    /// Output format (text, json, table)
    #[arg(long, default_value = "text")]
    format: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Initialize logging
    cli_common::init_logging(args.verbose)?;
    
    // Create command and run
    let command = CheckCompatibilityCommand::new(args);
    cli_common::run_command(command)
}

struct CheckCompatibilityCommand {
    args: Args,
}

impl CheckCompatibilityCommand {
    fn new(args: Args) -> Self {
        Self { args }
    }
}

impl CliCommand for CheckCompatibilityCommand {
    fn name(&self) -> &str {
        "check-type-compatibility"
    }
    
    fn description(&self) -> &str {
        "Check BSL type compatibility and inheritance"
    }
    
    fn execute(&self) -> Result<()> {
        self.check_compatibility()
    }
}

impl CheckCompatibilityCommand {
    fn check_compatibility(&self) -> Result<()> {
        // Validate configuration path
        cli_common::validate_path(&self.args.config, "Configuration directory")?;
        
        // Create output writer
        let format = OutputFormat::from_str(&self.args.format)?;
        let mut writer = OutputWriter::stdout(format);
        
        writer.write_header("Type Compatibility Check")?;
        writer.write_line(&format!("From: {}", self.args.from))?;
        writer.write_line(&format!("To: {}", self.args.to))?;
        writer.write_line("")?;
        
        // Build index
        let mut builder = UnifiedIndexBuilder::new()?;
        let index = builder.build_index(&self.args.config, &self.args.platform_version)?;
        
        // Check compatibility
        let is_compatible = index.is_assignable(&self.args.from, &self.args.to);
        
        // Find entities
        let from_entity = index.find_entity(&self.args.from).cloned();
        let to_entity = index.find_entity(&self.args.to).cloned();
        
        // Write results
        if self.args.verbose {
            self.write_detailed_analysis(&mut writer, is_compatible, &from_entity, &to_entity)?;
        } else {
            self.write_brief_result(&mut writer, is_compatible)?;
        }
        
        writer.flush()?;
        
        // Set exit code for scripts
        if !is_compatible {
            std::process::exit(1);
        }
        
        Ok(())
    }
    
    fn write_detailed_analysis(
        &self,
        writer: &mut OutputWriter,
        is_compatible: bool,
        from_entity: &Option<bsl_analyzer::unified_index::BslEntity>,
        to_entity: &Option<bsl_analyzer::unified_index::BslEntity>,
    ) -> Result<()> {
        writer.write_header("Analysis Results")?;
        
        // Source type info
        writer.write_line(&format!("Source type: {}", self.args.from))?;
        match from_entity {
            Some(entity) => {
                writer.write_list_item(&format!(
                    "Found: {} ({:?})",
                    entity.display_name, entity.entity_type
                ))?;
            }
            None => {
                writer.write_list_item("Not found in index")?;
            }
        }
        
        // Target type info
        writer.write_line(&format!("\nTarget type: {}", self.args.to))?;
        match to_entity {
            Some(entity) => {
                writer.write_list_item(&format!(
                    "Found: {} ({:?})",
                    entity.display_name, entity.entity_type
                ))?;
            }
            None => {
                writer.write_list_item("Not found in index")?;
            }
        }
        
        // Compatibility result
        writer.write_header("Compatibility Result")?;
        if is_compatible {
            cli_common::print_success(&format!("COMPATIBLE: '{}' can be assigned to '{}'", self.args.from, self.args.to));
            writer.write_line(&format!("✅ Type '{}' can be assigned to variable of type '{}'", self.args.from, self.args.to))?;
        } else {
            cli_common::print_error(&format!("NOT COMPATIBLE: '{}' cannot be assigned to '{}'", self.args.from, self.args.to));
            writer.write_line(&format!("❌ Type '{}' CANNOT be assigned to variable of type '{}'", self.args.from, self.args.to))?;
        }
        
        // Show inheritance path if requested
        if self.args.show_inheritance_path && is_compatible && self.args.from != self.args.to {
            self.write_inheritance_path(writer, from_entity)?;
        }
        
        // Additional type information
        if from_entity.is_some() || to_entity.is_some() {
            writer.write_header("Type Information")?;
            
            if let Some(entity) = from_entity {
                if !entity.constraints.parent_types.is_empty() {
                    writer.write_line(&format!(
                        "{} inherits from: {}",
                        self.args.from,
                        entity.constraints.parent_types.join(", ")
                    ))?;
                }
                if !entity.constraints.implements.is_empty() {
                    writer.write_line(&format!(
                        "{} implements: {}",
                        self.args.from,
                        entity.constraints.implements.join(", ")
                    ))?;
                }
            }
            
            if let Some(entity) = to_entity {
                if !entity.constraints.parent_types.is_empty() {
                    writer.write_line(&format!(
                        "{} inherits from: {}",
                        self.args.to,
                        entity.constraints.parent_types.join(", ")
                    ))?;
                }
                if !entity.constraints.implements.is_empty() {
                    writer.write_line(&format!(
                        "{} implements: {}",
                        self.args.to,
                        entity.constraints.implements.join(", ")
                    ))?;
                }
            }
        }
        
        Ok(())
    }
    
    fn write_brief_result(&self, _writer: &mut OutputWriter, is_compatible: bool) -> Result<()> {
        if is_compatible {
            cli_common::print_success(&format!("COMPATIBLE: {} -> {}", self.args.from, self.args.to));
        } else {
            cli_common::print_error(&format!("NOT COMPATIBLE: {} -> {}", self.args.from, self.args.to));
        }
        Ok(())
    }
    
    fn write_inheritance_path(
        &self,
        writer: &mut OutputWriter,
        from_entity: &Option<bsl_analyzer::unified_index::BslEntity>,
    ) -> Result<()> {
        writer.write_header("Compatibility Path")?;
        
        if let Some(entity) = from_entity {
            if entity.constraints.parent_types.contains(&self.args.to) {
                writer.write_line(&format!("{} → inherits → {}", self.args.from, self.args.to))?;
            } else if entity.constraints.implements.contains(&self.args.to) {
                writer.write_line(&format!("{} → implements → {}", self.args.from, self.args.to))?;
            } else if self.args.from == self.args.to {
                writer.write_line(&format!("{} ≡ {} (identical types)", self.args.from, self.args.to))?;
            } else {
                writer.write_line("Compatibility through BSL type system")?;
            }
        }
        
        Ok(())
    }
}