//! Главная CLI утилита для BSL анализатора
//! 
//! Единая точка входа для всех операций анализа BSL кода

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use bsl_analyzer::bsl_parser::{BslAnalyzer, AnalysisConfig};
use bsl_analyzer::unified_index::{UnifiedIndexBuilder, BslApplicationMode};
use bsl_analyzer::cli_common::{self, OutputWriter, OutputFormat, CliCommand};
use colored::Colorize;
use walkdir::WalkDir;

#[derive(Parser)]
#[command(
    name = "bsl-analyzer",
    version = env!("CARGO_PKG_VERSION"),
    about = "BSL Type Safety Analyzer - универсальный анализатор кода 1С:Предприятие",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Уровень детализации вывода
    #[arg(short, long, global = true)]
    verbose: bool,
    
    /// Тихий режим (минимальный вывод)
    #[arg(short, long, global = true)]
    quiet: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Анализирует BSL файлы или директории
    Analyze {
        /// Путь к файлу или директории
        path: PathBuf,
        
        /// Уровень анализа
        #[arg(short, long, default_value = "full")]
        level: String,
        
        /// Формат вывода
        #[arg(short, long, default_value = "text")]
        format: String,
        
        /// Путь к конфигурации (для семантического анализа)
        #[arg(short = 'c', long)]
        config: Option<PathBuf>,
        
        /// Версия платформы
        #[arg(short = 'p', long, default_value = "8.3.25")]
        platform_version: String,
        
        /// Рекурсивный анализ директорий
        #[arg(short, long)]
        recursive: bool,
        
        /// Показать только ошибки
        #[arg(short = 'e', long)]
        errors_only: bool,
        
        /// Максимальное количество ошибок (0 = без ограничений)
        #[arg(long, default_value = "0")]
        max_errors: usize,
    },
    
    /// Проверяет синтаксис BSL файлов
    Check {
        /// Путь к файлу или директории
        path: PathBuf,
        
        /// Формат вывода
        #[arg(short, long, default_value = "text")]
        format: String,
        
        /// Рекурсивный анализ
        #[arg(short, long)]
        recursive: bool,
    },
    
    /// Валидирует конфигурацию и код
    Validate {
        /// Путь к конфигурации
        config: PathBuf,
        
        /// Версия платформы
        #[arg(short = 'p', long, default_value = "8.3.25")]
        platform_version: String,
        
        /// Проверять неиспользуемые объекты
        #[arg(long)]
        check_unused: bool,
        
        /// Проверять циклические зависимости
        #[arg(long)]
        check_cycles: bool,
    },
    
    /// Построить индекс типов
    Index {
        /// Путь к конфигурации
        config: PathBuf,
        
        /// Версия платформы
        #[arg(short = 'p', long, default_value = "8.3.25")]
        platform_version: String,
        
        /// Режим приложения
        #[arg(short = 'm', long, default_value = "managed")]
        mode: String,
        
        /// Показать статистику
        #[arg(short, long)]
        stats: bool,
    },
    
    /// Найти тип в индексе
    Find {
        /// Имя типа для поиска
        name: String,
        
        /// Путь к конфигурации
        #[arg(short = 'c', long)]
        config: Option<PathBuf>,
        
        /// Версия платформы
        #[arg(short = 'p', long, default_value = "8.3.25")]
        platform_version: String,
        
        /// Показать все методы
        #[arg(long)]
        show_methods: bool,
        
        /// Показать все свойства
        #[arg(long)]
        show_properties: bool,
    },
    
    /// Проверить совместимость типов
    Compat {
        /// Исходный тип
        from: String,
        
        /// Целевой тип
        to: String,
        
        /// Путь к конфигурации
        #[arg(short = 'c', long)]
        config: Option<PathBuf>,
        
        /// Версия платформы
        #[arg(short = 'p', long, default_value = "8.3.25")]
        platform_version: String,
    },
    
    /// Показать статистику проекта
    Stats {
        /// Путь к проекту
        path: PathBuf,
        
        /// Формат вывода
        #[arg(short, long, default_value = "table")]
        format: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Инициализация логирования
    if !cli.quiet {
        cli_common::init_logging(cli.verbose)?;
    } else {
        cli_common::init_minimal_logging()?;
    }
    
    // Выполнение команды
    match cli.command {
        Commands::Analyze { 
            path, level, format, config, platform_version, 
            recursive, errors_only, max_errors 
        } => {
            let cmd = AnalyzeCommand {
                path,
                level,
                format,
                config,
                platform_version,
                recursive,
                errors_only,
                max_errors,
            };
            cli_common::run_command(cmd)?;
        }
        
        Commands::Check { path, format, recursive } => {
            let cmd = CheckCommand {
                path,
                format,
                recursive,
            };
            cli_common::run_command(cmd)?;
        }
        
        Commands::Validate { config, platform_version, check_unused, check_cycles } => {
            let cmd = ValidateCommand {
                config,
                platform_version,
                check_unused,
                check_cycles,
            };
            cli_common::run_command(cmd)?;
        }
        
        Commands::Index { config, platform_version, mode, stats } => {
            let cmd = IndexCommand {
                config,
                platform_version,
                mode,
                stats,
            };
            cli_common::run_command(cmd)?;
        }
        
        Commands::Find { name, config, platform_version, show_methods, show_properties } => {
            let cmd = FindCommand {
                name,
                config,
                platform_version,
                show_methods,
                show_properties,
            };
            cli_common::run_command(cmd)?;
        }
        
        Commands::Compat { from, to, config, platform_version } => {
            let cmd = CompatCommand {
                from,
                to,
                config,
                platform_version,
            };
            cli_common::run_command(cmd)?;
        }
        
        Commands::Stats { path, format } => {
            let cmd = StatsCommand {
                path,
                format,
            };
            cli_common::run_command(cmd)?;
        }
    }
    
    Ok(())
}

// ================================================================================
// Команда Analyze - полный анализ с настраиваемым уровнем
// ================================================================================

struct AnalyzeCommand {
    path: PathBuf,
    level: String,
    format: String,
    config: Option<PathBuf>,
    platform_version: String,
    recursive: bool,
    errors_only: bool,
    max_errors: usize,
}

impl CliCommand for AnalyzeCommand {
    fn name(&self) -> &str {
        "analyze"
    }
    
    fn description(&self) -> &str {
        "Comprehensive BSL code analysis"
    }
    
    fn execute(&self) -> Result<()> {
        // Валидация пути
        cli_common::validate_path(&self.path, "Analysis path")?;
        
        // Создание конфигурации анализа
        let mut config = self.create_analysis_config()?;
        config.max_errors = self.max_errors;
        
        // Создание анализатора
        let mut analyzer = if let Some(config_path) = &self.config {
            // С индексом для семантического анализа
            cli_common::print_info("Загрузка индекса типов для семантического анализа...");
            let mut builder = UnifiedIndexBuilder::new()?;
            let index = builder.build_index(
                config_path.to_str().unwrap_or_default(),
                &self.platform_version
            )?;
            BslAnalyzer::with_index_and_config(index, config.clone())?
        } else {
            BslAnalyzer::with_config(config.clone())?
        };
        
        // Анализ файлов
        let mut total_errors = 0;
        let mut total_warnings = 0;
        let mut files_analyzed = 0;
        
        if self.path.is_file() {
            self.analyze_file(&self.path, &mut analyzer, &config)?;
            files_analyzed = 1;
            let (errors, warnings) = analyzer.get_errors_and_warnings();
            total_errors = errors.len();
            total_warnings = warnings.len();
        } else if self.path.is_dir() {
            let (analyzed, errors, warnings) = self.analyze_directory(
                &self.path, 
                &mut analyzer, 
                &config
            )?;
            files_analyzed = analyzed;
            total_errors = errors;
            total_warnings = warnings;
        }
        
        // Вывод статистики
        self.print_summary(files_analyzed, total_errors, total_warnings)?;
        
        // Возвращаем ошибку если были критические проблемы
        if total_errors > 0 {
            std::process::exit(1);
        }
        
        Ok(())
    }
}

impl AnalyzeCommand {
    fn create_analysis_config(&self) -> Result<AnalysisConfig> {
        let config = match self.level.to_lowercase().as_str() {
            "syntax" => AnalysisConfig::syntax_only(),
            "semantic" => AnalysisConfig::semantic(),
            "dataflow" | "data-flow" => AnalysisConfig::data_flow(),
            "full" => AnalysisConfig::full(),
            _ => {
                anyhow::bail!("Invalid analysis level: {}. Use: syntax, semantic, dataflow, or full", self.level);
            }
        };
        Ok(config)
    }
    
    fn analyze_file(
        &self,
        path: &PathBuf,
        analyzer: &mut BslAnalyzer,
        config: &AnalysisConfig,
    ) -> Result<()> {
        analyzer.clear();
        analyzer.analyze_file(path, config)?;
        
        let (errors, warnings) = analyzer.get_errors_and_warnings();
        
        if !errors.is_empty() || (!self.errors_only && !warnings.is_empty()) {
            self.print_diagnostics(path, &errors, &warnings)?;
        }
        
        Ok(())
    }
    
    fn analyze_directory(
        &self,
        path: &PathBuf,
        analyzer: &mut BslAnalyzer,
        config: &AnalysisConfig,
    ) -> Result<(usize, usize, usize)> {
        let mut total_errors = 0;
        let mut total_warnings = 0;
        let mut files_analyzed = 0;
        
        let walker = if self.recursive {
            WalkDir::new(path)
        } else {
            WalkDir::new(path).max_depth(1)
        };
        
        for entry in walker {
            let entry = entry?;
            if entry.file_type().is_file() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "bsl" || ext == "os" {
                        self.analyze_file(&entry.path().to_path_buf(), analyzer, config)?;
                        files_analyzed += 1;
                        
                        let (errors, warnings) = analyzer.get_errors_and_warnings();
                        total_errors += errors.len();
                        total_warnings += warnings.len();
                    }
                }
            }
        }
        
        Ok((files_analyzed, total_errors, total_warnings))
    }
    
    fn print_diagnostics(
        &self,
        path: &PathBuf,
        errors: &[bsl_analyzer::core::errors::AnalysisError],
        warnings: &[bsl_analyzer::core::errors::AnalysisError],
    ) -> Result<()> {
        let format = OutputFormat::from_str(&self.format)?;
        let mut writer = OutputWriter::stdout(format);
        
        writer.write_header(&format!("Diagnostics for: {}", path.display()))?;
        
        for error in errors {
            writer.write_line(&format!(
                "{} [{}:{}] {}",
                "ERROR".red().bold(),
                error.position.line,
                error.position.column,
                error.message
            ))?;
        }
        
        if !self.errors_only {
            for warning in warnings {
                writer.write_line(&format!(
                    "{} [{}:{}] {}",
                    "WARN".yellow().bold(),
                    warning.position.line,
                    warning.position.column,
                    warning.message
                ))?;
            }
        }
        
        writer.flush()?;
        Ok(())
    }
    
    fn print_summary(&self, files: usize, errors: usize, warnings: usize) -> Result<()> {
        let format = OutputFormat::from_str(&self.format)?;
        let mut writer = OutputWriter::stdout(format);
        
        writer.write_header("Analysis Summary")?;
        
        let rows = vec![
            vec!["Files analyzed".to_string(), files.to_string()],
            vec!["Errors found".to_string(), errors.to_string()],
            vec!["Warnings found".to_string(), warnings.to_string()],
            vec!["Analysis level".to_string(), self.level.clone()],
        ];
        
        writer.write_table(&["Metric", "Value"], rows)?;
        writer.flush()?;
        
        if errors > 0 {
            cli_common::print_error(&format!("Found {} errors", errors));
        } else {
            cli_common::print_success("No errors found");
        }
        
        Ok(())
    }
}

// ================================================================================
// Команда Check - быстрая проверка синтаксиса
// ================================================================================

struct CheckCommand {
    path: PathBuf,
    format: String,
    recursive: bool,
}

impl CliCommand for CheckCommand {
    fn name(&self) -> &str {
        "check"
    }
    
    fn description(&self) -> &str {
        "Quick syntax check for BSL files"
    }
    
    fn execute(&self) -> Result<()> {
        // Делегируем в analyze с уровнем syntax
        let cmd = AnalyzeCommand {
            path: self.path.clone(),
            level: "syntax".to_string(),
            format: self.format.clone(),
            config: None,
            platform_version: "8.3.25".to_string(),
            recursive: self.recursive,
            errors_only: false,
            max_errors: 0,
        };
        cmd.execute()
    }
}

// ================================================================================
// Команда Validate - валидация конфигурации
// ================================================================================

struct ValidateCommand {
    config: PathBuf,
    platform_version: String,
    check_unused: bool,
    check_cycles: bool,
}

impl CliCommand for ValidateCommand {
    fn name(&self) -> &str {
        "validate"
    }
    
    fn description(&self) -> &str {
        "Validate configuration and code integrity"
    }
    
    fn execute(&self) -> Result<()> {
        cli_common::validate_path(&self.config, "Configuration path")?;
        
        cli_common::print_info("Загрузка конфигурации...");
        
        // Построение индекса
        let mut builder = UnifiedIndexBuilder::new()?;
        let index = builder.build_index(
            self.config.to_str().unwrap_or_default(),
            &self.platform_version
        )?;
        
        let entity_count = index.get_entity_count();
        cli_common::print_success(&format!("Загружено {} типов", entity_count));
        
        // Проверки
        if self.check_unused {
            cli_common::print_info("Проверка неиспользуемых объектов...");
            // TODO: реализовать проверку неиспользуемых
        }
        
        if self.check_cycles {
            cli_common::print_info("Проверка циклических зависимостей...");
            // TODO: реализовать проверку циклов
        }
        
        cli_common::print_success("Валидация завершена успешно");
        Ok(())
    }
}

// ================================================================================
// Команда Index - построение индекса
// ================================================================================

struct IndexCommand {
    config: PathBuf,
    platform_version: String,
    mode: String,
    stats: bool,
}

impl CliCommand for IndexCommand {
    fn name(&self) -> &str {
        "index"
    }
    
    fn description(&self) -> &str {
        "Build unified BSL type index"
    }
    
    fn execute(&self) -> Result<()> {
        cli_common::validate_path(&self.config, "Configuration path")?;
        
        let app_mode = match self.mode.to_lowercase().as_str() {
            "managed" => BslApplicationMode::ManagedApplication,
            "ordinary" => BslApplicationMode::OrdinaryApplication,
            "mixed" => BslApplicationMode::MixedMode,
            _ => {
                anyhow::bail!("Invalid application mode: {}", self.mode);
            }
        };
        
        cli_common::print_info(&format!("Building index for {} mode...", self.mode));
        
        let start_time = std::time::Instant::now();
        
        let mut builder = UnifiedIndexBuilder::new()?
            .with_application_mode(app_mode);
        
        let index = builder.build_index(
            self.config.to_str().unwrap_or_default(),
            &self.platform_version
        )?;
        
        let elapsed = start_time.elapsed();
        
        if self.stats {
            let mut writer = OutputWriter::stdout(OutputFormat::Table);
            writer.write_header("Index Statistics")?;
            
            let rows = vec![
                vec!["Total entities".to_string(), index.get_entity_count().to_string()],
                vec!["Platform version".to_string(), self.platform_version.clone()],
                vec!["Application mode".to_string(), self.mode.clone()],
                vec!["Build time".to_string(), format!("{:.2?}", elapsed)],
            ];
            
            writer.write_table(&["Metric", "Value"], rows)?;
            writer.flush()?;
        }
        
        cli_common::print_success(&format!(
            "Index built: {} types in {:.2?}",
            index.get_entity_count(),
            elapsed
        ));
        
        Ok(())
    }
}

// ================================================================================
// Команда Find - поиск типа
// ================================================================================

struct FindCommand {
    name: String,
    config: Option<PathBuf>,
    platform_version: String,
    show_methods: bool,
    show_properties: bool,
}

impl CliCommand for FindCommand {
    fn name(&self) -> &str {
        "find"
    }
    
    fn description(&self) -> &str {
        "Find type in BSL index"
    }
    
    fn execute(&self) -> Result<()> {
        // Загружаем индекс
        let index = if let Some(config_path) = &self.config {
            cli_common::validate_path(config_path, "Configuration path")?;
            let mut builder = UnifiedIndexBuilder::new()?;
            builder.build_index(
                config_path.to_str().unwrap_or_default(),
                &self.platform_version
            )?
        } else {
            // Только платформенные типы - используем временную пустую директорию
            cli_common::print_info("Загрузка только платформенных типов...");
            let temp_dir = std::env::temp_dir().join("bsl-analyzer-temp");
            std::fs::create_dir_all(&temp_dir)?;
            let mut builder = UnifiedIndexBuilder::new()?;
            builder.build_index(&temp_dir, &self.platform_version)?
        };
        
        // Ищем тип
        if let Some(entity) = index.find_entity(&self.name) {
            let mut writer = OutputWriter::stdout(OutputFormat::Text);
            
            writer.write_header(&format!("Type: {}", entity.display_name))?;
            writer.write_line(&format!("Qualified name: {}", entity.qualified_name))?;
            writer.write_line(&format!("Type: {:?}", entity.entity_type))?;
            writer.write_line(&format!("Kind: {:?}", entity.entity_kind))?;
            
            if self.show_methods && !entity.interface.methods.is_empty() {
                writer.write_header("Methods")?;
                for (name, _method) in &entity.interface.methods {
                    writer.write_line(&format!("  • {}", name))?;
                }
            }
            
            if self.show_properties && !entity.interface.properties.is_empty() {
                writer.write_header("Properties")?;
                for (name, prop) in &entity.interface.properties {
                    writer.write_line(&format!("  • {} ({})", name, prop.type_name))?;
                }
            }
            
            writer.flush()?;
            cli_common::print_success("Type found");
        } else {
            cli_common::print_warning(&format!("Type '{}' not found", self.name));
        }
        
        Ok(())
    }
}

// ================================================================================
// Команда Compat - проверка совместимости типов
// ================================================================================

struct CompatCommand {
    from: String,
    to: String,
    config: Option<PathBuf>,
    platform_version: String,
}

impl CliCommand for CompatCommand {
    fn name(&self) -> &str {
        "compat"
    }
    
    fn description(&self) -> &str {
        "Check type compatibility"
    }
    
    fn execute(&self) -> Result<()> {
        // Загружаем индекс
        let index = if let Some(config_path) = &self.config {
            cli_common::validate_path(config_path, "Configuration path")?;
            let mut builder = UnifiedIndexBuilder::new()?;
            builder.build_index(
                config_path.to_str().unwrap_or_default(),
                &self.platform_version
            )?
        } else {
            // Только платформенные типы - используем временную пустую директорию
            let temp_dir = std::env::temp_dir().join("bsl-analyzer-temp");
            std::fs::create_dir_all(&temp_dir)?;
            let mut builder = UnifiedIndexBuilder::new()?;
            builder.build_index(&temp_dir, &self.platform_version)?
        };
        
        // Проверяем совместимость
        if index.is_assignable(&self.from, &self.to) {
            cli_common::print_success(&format!(
                "✓ Type '{}' is assignable to '{}'",
                self.from, self.to
            ));
        } else {
            cli_common::print_error(&format!(
                "✗ Type '{}' is NOT assignable to '{}'",
                self.from, self.to
            ));
            std::process::exit(1);
        }
        
        Ok(())
    }
}

// ================================================================================
// Команда Stats - статистика проекта
// ================================================================================

struct StatsCommand {
    path: PathBuf,
    format: String,
}

impl CliCommand for StatsCommand {
    fn name(&self) -> &str {
        "stats"
    }
    
    fn description(&self) -> &str {
        "Show project statistics"
    }
    
    fn execute(&self) -> Result<()> {
        cli_common::validate_path(&self.path, "Project path")?;
        
        let mut file_count = 0;
        let mut line_count = 0;
        let mut module_count = 0;
        
        for entry in WalkDir::new(&self.path) {
            let entry = entry?;
            if entry.file_type().is_file() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "bsl" || ext == "os" {
                        file_count += 1;
                        let content = std::fs::read_to_string(entry.path())?;
                        line_count += content.lines().count();
                        
                        // Простая проверка на модуль
                        if content.contains("Процедура") || content.contains("Функция") {
                            module_count += 1;
                        }
                    }
                }
            }
        }
        
        let format = OutputFormat::from_str(&self.format)?;
        let mut writer = OutputWriter::stdout(format);
        
        writer.write_header("Project Statistics")?;
        
        let rows = vec![
            vec!["BSL files".to_string(), file_count.to_string()],
            vec!["Total lines".to_string(), line_count.to_string()],
            vec!["Modules with code".to_string(), module_count.to_string()],
            vec!["Average lines/file".to_string(), 
                 if file_count > 0 { (line_count / file_count).to_string() } else { "0".to_string() }],
        ];
        
        writer.write_table(&["Metric", "Value"], rows)?;
        writer.flush()?;
        
        Ok(())
    }
}