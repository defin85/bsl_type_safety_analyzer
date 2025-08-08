//! CLI утилита для инкрементального обновления UnifiedBslIndex

use std::path::PathBuf;
use anyhow::{Result, Context};
use clap::{Parser as ClapParser, ValueEnum};
use bsl_analyzer::unified_index::{UnifiedIndexBuilder, BslApplicationMode, ChangeImpact};
use bsl_analyzer::cli_common::{self, OutputWriter, OutputFormat, CliCommand, ProgressReporter};
use serde::{Serialize, Deserialize};

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

#[derive(ClapParser, Debug)]
#[command(
    name = "incremental_update",
    about = "Инкрементальное обновление индекса BSL типов",
    long_about = "Выполняет инкрементальное обновление UnifiedBslIndex при изменении файлов конфигурации"
)]
struct Args {
    /// Путь к директории конфигурации 1С
    #[arg(short, long)]
    config: PathBuf,
    
    /// Версия платформы (например, "8.3.25")
    #[arg(short, long)]
    platform_version: String,
    
    /// Режим приложения
    #[arg(short = 'm', long, value_enum, default_value = "managed")]
    mode: ApplicationMode,
    
    /// Формат вывода (text, json, table)
    #[arg(short, long, default_value = "text")]
    format: String,
    
    /// Подробный вывод об изменениях
    #[arg(short, long)]
    verbose: bool,
    
    /// Только проверка изменений без обновления (dry-run)
    #[arg(short = 'n', long)]
    dry_run: bool,
    
    /// Принудительное инкрементальное обновление
    #[arg(short = 'f', long)]
    force_incremental: bool,
    
    /// Тихий режим
    #[arg(short, long)]
    quiet: bool,
    
    /// Показать детальную статистику
    #[arg(short = 's', long)]
    show_stats: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct UpdateResult {
    config_path: String,
    platform_version: String,
    change_impact: String,
    changed_files_count: usize,
    update_type: String,
    success: bool,
    statistics: UpdateStatistics,
    changes: ChangesSummary,
}

#[derive(Debug, Serialize, Deserialize)]
struct UpdateStatistics {
    elapsed_ms: u128,
    total_entities: usize,
    cache_hit: bool,
    performance_ratio: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChangesSummary {
    added_entities: Vec<String>,
    updated_entities: Vec<String>,
    removed_entities: Vec<String>,
    total_changes: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct FileChange {
    path: String,
    impact: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Initialize logging
    if !args.quiet {
        cli_common::init_logging(args.verbose)?;
    } else {
        cli_common::init_minimal_logging()?;
    }
    
    // Create command and run
    let command = IncrementalUpdateCommand::new(args);
    cli_common::run_command(command)
}

struct IncrementalUpdateCommand {
    args: Args,
}

impl IncrementalUpdateCommand {
    fn new(args: Args) -> Self {
        Self { args }
    }
}

impl CliCommand for IncrementalUpdateCommand {
    fn name(&self) -> &str {
        "incremental_update"
    }
    
    fn description(&self) -> &str {
        "Incremental update of BSL type index"
    }
    
    fn execute(&self) -> Result<()> {
        self.run_update()
    }
}

impl IncrementalUpdateCommand {
    fn run_update(&self) -> Result<()> {
        // Validate config path
        cli_common::validate_path(&self.args.config, "Configuration directory")?;
        
        if !self.args.config.is_dir() {
            return Err(anyhow::anyhow!(
                "Configuration path must be a directory: {}",
                self.args.config.display()
            ));
        }
        
        if !self.args.quiet {
            cli_common::print_info("🔄 Инкрементальное обновление индекса BSL");
        }
        
        // Create builder
        let mut builder = UnifiedIndexBuilder::new()
            .context("Failed to create index builder")?
            .with_application_mode(self.args.mode.clone().into());
        
        let start = std::time::Instant::now();
        
        // Set up progress reporting
        let progress = if !self.args.quiet && !self.args.dry_run {
            Some(ProgressReporter::new(100, "Анализ изменений"))
        } else {
            None
        };
        
        // Check feasibility
        let result = match builder.check_incremental_update_feasibility(
            &self.args.config, 
            &self.args.platform_version
        ) {
            Ok((change_impact, changed_files)) => {
                if let Some(p) = &progress {
                    p.update(50);
                }
                
                self.handle_changes(
                    &mut builder, 
                    change_impact, 
                    changed_files, 
                    start
                )?
            }
            Err(e) => {
                if !self.args.quiet {
                    cli_common::print_warning(&format!(
                        "Невозможно выполнить инкрементальное обновление: {}", e
                    ));
                }
                
                self.perform_full_rebuild(&mut builder, start)?
            }
        };
        
        if let Some(p) = progress {
            p.finish();
        }
        
        // Display results
        self.display_results(&result)?;
        
        Ok(())
    }
    
    fn handle_changes(
        &self,
        builder: &mut UnifiedIndexBuilder,
        change_impact: ChangeImpact,
        changed_files: Vec<(PathBuf, ChangeImpact)>,
        start: std::time::Instant,
    ) -> Result<UpdateResult> {
        let mut result = UpdateResult {
            config_path: self.args.config.display().to_string(),
            platform_version: self.args.platform_version.clone(),
            change_impact: format!("{:?}", change_impact),
            changed_files_count: changed_files.len(),
            update_type: String::new(),
            success: false,
            statistics: UpdateStatistics {
                elapsed_ms: 0,
                total_entities: 0,
                cache_hit: false,
                performance_ratio: 0.0,
            },
            changes: ChangesSummary {
                added_entities: Vec::new(),
                updated_entities: Vec::new(),
                removed_entities: Vec::new(),
                total_changes: 0,
            },
        };
        
        if changed_files.is_empty() {
            result.update_type = "None".to_string();
            result.success = true;
            result.statistics.elapsed_ms = start.elapsed().as_millis();
            
            if !self.args.quiet {
                cli_common::print_success("Индекс актуален, изменений не обнаружено");
            }
            return Ok(result);
        }
        
        if self.args.dry_run {
            result.update_type = "DryRun".to_string();
            result.success = true;
            result.statistics.elapsed_ms = start.elapsed().as_millis();
            
            let recommendation = self.get_recommendation(&change_impact);
            if !self.args.quiet {
                cli_common::print_info(&format!("🔍 Режим проверки - изменения не применены"));
                cli_common::print_info(&format!("Рекомендация: {}", recommendation));
            }
            
            return Ok(result);
        }
        
        // Perform actual update
        match change_impact {
            ChangeImpact::None => {
                result.update_type = "None".to_string();
                result.success = true;
            }
            ChangeImpact::FullRebuild if !self.args.force_incremental => {
                if !self.args.quiet {
                    cli_common::print_warning(
                        "Требуется полная перестройка из-за изменений в Configuration.xml"
                    );
                }
                return self.perform_full_rebuild(builder, start);
            }
            _ => {
                // Perform incremental update
                if !self.args.quiet {
                    cli_common::print_info("🚀 Выполняется инкрементальное обновление...");
                }
                
                let update_result = builder.perform_incremental_update(
                    &self.args.config,
                    &self.args.platform_version,
                    changed_files
                ).context("Failed to perform incremental update")?;
                
                result.update_type = "Incremental".to_string();
                result.success = update_result.success;
                result.changes = ChangesSummary {
                    added_entities: update_result.added_entities.clone(),
                    updated_entities: update_result.updated_entities.clone(),
                    removed_entities: update_result.removed_entities.clone(),
                    total_changes: update_result.total_changes(),
                };
            }
        }
        
        result.statistics.elapsed_ms = start.elapsed().as_millis();
        result.statistics.cache_hit = true;
        result.statistics.performance_ratio = if result.statistics.elapsed_ms > 0 {
            500.0 / result.statistics.elapsed_ms as f64  // Compare to 500ms baseline
        } else {
            1.0
        };
        
        Ok(result)
    }
    
    fn perform_full_rebuild(
        &self,
        builder: &mut UnifiedIndexBuilder,
        start: std::time::Instant,
    ) -> Result<UpdateResult> {
        if !self.args.quiet {
            cli_common::print_info("Выполняется полная перестройка индекса...");
        }
        
        let index = builder.build_index(&self.args.config, &self.args.platform_version)
            .context("Failed to build index")?;
        
        let elapsed = start.elapsed();
        
        Ok(UpdateResult {
            config_path: self.args.config.display().to_string(),
            platform_version: self.args.platform_version.clone(),
            change_impact: "FullRebuild".to_string(),
            changed_files_count: 0,
            update_type: "FullRebuild".to_string(),
            success: true,
            statistics: UpdateStatistics {
                elapsed_ms: elapsed.as_millis(),
                total_entities: index.get_entity_count(),
                cache_hit: false,
                performance_ratio: 1.0,
            },
            changes: ChangesSummary {
                added_entities: Vec::new(),
                updated_entities: Vec::new(),
                removed_entities: Vec::new(),
                total_changes: 0,
            },
        })
    }
    
    fn get_recommendation(&self, impact: &ChangeImpact) -> &'static str {
        match impact {
            ChangeImpact::None => "Обновление не требуется",
            ChangeImpact::Minor => "Инкрементальное обновление (несколько миллисекунд)",
            ChangeImpact::ModuleUpdate => "Инкрементальное обновление (~10-50мс)",
            ChangeImpact::MetadataUpdate => "Инкрементальное обновление (~50-200мс)",
            ChangeImpact::FullRebuild => "Рекомендуется полная перестройка (~500мс)",
        }
    }
    
    fn display_results(&self, result: &UpdateResult) -> Result<()> {
        let format = OutputFormat::from_str(&self.args.format)?;
        let mut writer = OutputWriter::stdout(format);
        
        if !self.args.quiet {
            writer.write_header("Результаты обновления индекса")?;
            
            // Basic info
            let info_rows = vec![
                vec!["Конфигурация".to_string(), result.config_path.clone()],
                vec!["Версия платформы".to_string(), result.platform_version.clone()],
                vec!["Тип обновления".to_string(), result.update_type.clone()],
                vec!["Статус".to_string(), if result.success { "✅ Успешно" } else { "❌ Ошибка" }.to_string()],
                vec!["Время выполнения".to_string(), format!("{:.2?}мс", result.statistics.elapsed_ms)],
            ];
            
            writer.write_table(&["Параметр", "Значение"], info_rows)?;
            
            // Changes summary
            if result.changes.total_changes > 0 {
                writer.write_header("Сводка изменений")?;
                
                let changes_rows = vec![
                    vec!["Добавлено".to_string(), result.changes.added_entities.len().to_string()],
                    vec!["Обновлено".to_string(), result.changes.updated_entities.len().to_string()],
                    vec!["Удалено".to_string(), result.changes.removed_entities.len().to_string()],
                    vec!["Всего изменений".to_string(), result.changes.total_changes.to_string()],
                ];
                
                writer.write_table(&["Тип изменения", "Количество"], changes_rows)?;
                
                // Detailed changes if verbose
                if self.args.verbose && result.changes.total_changes > 0 {
                    writer.write_header("Детали изменений")?;
                    
                    if !result.changes.added_entities.is_empty() {
                        writer.write_line("➕ Добавлено:")?;
                        for entity in &result.changes.added_entities {
                            writer.write_line(&format!("   • {}", entity))?;
                        }
                    }
                    
                    if !result.changes.updated_entities.is_empty() {
                        writer.write_line("🔄 Обновлено:")?;
                        for entity in &result.changes.updated_entities {
                            writer.write_line(&format!("   • {}", entity))?;
                        }
                    }
                    
                    if !result.changes.removed_entities.is_empty() {
                        writer.write_line("➖ Удалено:")?;
                        for entity in &result.changes.removed_entities {
                            writer.write_line(&format!("   • {}", entity))?;
                        }
                    }
                }
            }
            
            // Performance statistics
            if self.args.show_stats {
                writer.write_header("Статистика производительности")?;
                
                let stats_rows = vec![
                    vec!["Всего сущностей".to_string(), result.statistics.total_entities.to_string()],
                    vec!["Использован кеш".to_string(), if result.statistics.cache_hit { "Да" } else { "Нет" }.to_string()],
                    vec!["Коэффициент производительности".to_string(), 
                         format!("{:.2}x", result.statistics.performance_ratio)],
                    vec!["Измененных файлов".to_string(), result.changed_files_count.to_string()],
                ];
                
                writer.write_table(&["Метрика", "Значение"], stats_rows)?;
            }
            
            // Success message
            if result.success {
                cli_common::print_success(&format!(
                    "Обновление завершено за {:.2?}мс", 
                    result.statistics.elapsed_ms
                ));
            }
        }
        
        writer.flush()?;
        Ok(())
    }
}