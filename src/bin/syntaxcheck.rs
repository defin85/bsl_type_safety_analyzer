//! CLI утилита для проверки синтаксиса BSL файлов

use anyhow::Result;
use clap::Parser as ClapParser;
use std::path::{Path, PathBuf};
use bsl_analyzer::bsl_parser::{BslAnalyzer, AnalysisConfig};
use bsl_analyzer::core::errors::AnalysisError;
use bsl_analyzer::cli_common::{self, OutputWriter, OutputFormat, CliCommand, ProgressReporter};
use walkdir::WalkDir;
use colored::Colorize;

#[derive(ClapParser, Debug)]
#[command(
    name = "syntaxcheck",
    about = "Проверяет синтаксис BSL файлов",
    long_about = "Утилита для быстрой проверки синтаксиса BSL файлов с использованием tree-sitter парсера"
)]
struct Args {
    /// Путь к файлу или директории для проверки
    #[arg(help = "Файл .bsl или директория с файлами")]
    path: PathBuf,

    /// Формат вывода (text, json, lsp)
    #[arg(short, long, default_value = "text")]
    format: String,

    /// Вывести только ошибки (без предупреждений)
    #[arg(short, long)]
    errors_only: bool,

    /// Рекурсивная проверка директорий
    #[arg(short, long, default_value = "true")]
    recursive: bool,

    /// Показать статистику
    #[arg(short, long)]
    stats: bool,

    /// Тихий режим (только код возврата)
    #[arg(short, long)]
    quiet: bool,
    
    /// Подробный вывод
    #[arg(short, long)]
    verbose: bool,
    
    /// Уровень анализа (syntax, semantic, dataflow, full)
    #[arg(short = 'l', long, default_value = "syntax")]
    level: String,
}

#[derive(Debug, Clone)]
struct CheckStats {
    files_checked: usize,
    files_with_errors: usize,
    total_errors: usize,
    total_warnings: usize,
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
    let command = SyntaxCheckCommand::new(args);
    cli_common::run_command(command)
}

struct SyntaxCheckCommand {
    args: Args,
}

impl SyntaxCheckCommand {
    fn new(args: Args) -> Self {
        Self { args }
    }
}

impl CliCommand for SyntaxCheckCommand {
    fn name(&self) -> &str {
        "syntaxcheck"
    }
    
    fn description(&self) -> &str {
        "Check BSL files syntax using tree-sitter parser"
    }
    
    fn execute(&self) -> Result<()> {
        self.run_check()
    }
}

impl SyntaxCheckCommand {
    fn run_check(&self) -> Result<()> {
        // Validate path
        cli_common::validate_path(&self.args.path, "Input path")?;
        
        let mut stats = CheckStats {
            files_checked: 0,
            files_with_errors: 0,
            total_errors: 0,
            total_warnings: 0,
        };
        
        // Создаем конфигурацию анализа на основе параметров
        let config = self.create_analysis_config()?;
        let mut analyzer = BslAnalyzer::with_config(config.clone())?;
        
        // Count total files for progress
        let total_files = if self.args.path.is_dir() {
            self.count_bsl_files(&self.args.path)?
        } else {
            1
        };
        
        let progress = if !self.args.quiet && total_files > 1 {
            Some(ProgressReporter::new(total_files, "Checking files"))
        } else {
            None
        };
        
        // Check files
        if self.args.path.is_file() {
            self.check_file(&self.args.path, &mut analyzer, &mut stats, &config)?;
        } else if self.args.path.is_dir() {
            self.check_directory(&self.args.path, &mut analyzer, &mut stats, &progress, &config)?;
        }
        
        if let Some(p) = progress {
            p.finish();
        }
        
        // Show statistics
        if self.args.stats && !self.args.quiet {
            self.print_statistics(&stats)?;
        }
        
        // Return non-zero exit code if there were errors
        if stats.files_with_errors > 0 {
            std::process::exit(1);
        }
        
        Ok(())
    }
    
    fn create_analysis_config(&self) -> Result<AnalysisConfig> {
        let config = match self.args.level.to_lowercase().as_str() {
            "syntax" => AnalysisConfig::syntax_only(),
            "semantic" => AnalysisConfig::semantic(),
            "dataflow" | "data-flow" => AnalysisConfig::data_flow(),
            "full" => AnalysisConfig::full(),
            _ => {
                anyhow::bail!("Invalid analysis level: {}. Use: syntax, semantic, dataflow, or full", self.args.level);
            }
        };
        Ok(config)
    }
    
    fn count_bsl_files(&self, path: &Path) -> Result<usize> {
        let walker = if self.args.recursive {
            WalkDir::new(path)
        } else {
            WalkDir::new(path).max_depth(1)
        };
        
        Ok(walker
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("bsl"))
            .count())
    }
    
    fn check_file(
        &self,
        path: &Path,
        analyzer: &mut BslAnalyzer,
        stats: &mut CheckStats,
        config: &AnalysisConfig,
    ) -> Result<()> {
        if path.extension().and_then(|e| e.to_str()) != Some("bsl") {
            return Ok(());
        }
        
        // Очищаем предыдущие результаты
        analyzer.clear();
        
        // Анализируем файл с заданной конфигурацией
        analyzer.analyze_file(path, config)?;
        
        stats.files_checked += 1;
        
        // Получаем результаты анализа
        let (errors, warnings) = analyzer.get_errors_and_warnings();
        
        stats.total_errors += errors.len();
        stats.total_warnings += warnings.len();
        
        if !errors.is_empty() {
            stats.files_with_errors += 1;
        }
        
        if !self.args.quiet && (!errors.is_empty() || !warnings.is_empty()) {
            self.print_analysis_diagnostics(path, &errors, &warnings)?;
        }
        
        Ok(())
    }
    
    fn check_directory(
        &self,
        path: &Path,
        analyzer: &mut BslAnalyzer,
        stats: &mut CheckStats,
        progress: &Option<ProgressReporter>,
        config: &AnalysisConfig,
    ) -> Result<()> {
        let walker = if self.args.recursive {
            WalkDir::new(path)
        } else {
            WalkDir::new(path).max_depth(1)
        };
        
        for entry in walker {
            let entry = entry?;
            if entry.file_type().is_file() {
                self.check_file(entry.path(), analyzer, stats, config)?;
                if let Some(p) = progress {
                    p.inc();
                }
            }
        }
        
        Ok(())
    }
    
    fn print_analysis_diagnostics(
        &self,
        path: &Path,
        errors: &[AnalysisError],
        warnings: &[AnalysisError],
    ) -> Result<()> {
        let format = OutputFormat::from_str(&self.args.format)?;
        let mut writer = OutputWriter::stdout(format);
        
        match format {
            OutputFormat::Text => self.print_human_format(&mut writer, path, errors, warnings)?,
            OutputFormat::Json => self.print_json_format(&mut writer, path, errors, warnings)?,
            _ => self.print_lsp_format(&mut writer, path, errors, warnings)?,
        }
        
        writer.flush()?;
        Ok(())
    }
    
    fn print_human_format(
        &self,
        writer: &mut OutputWriter,
        path: &Path,
        errors: &[AnalysisError],
        warnings: &[AnalysisError],
    ) -> Result<()> {
        // Печатаем ошибки
        for error in errors {
            let severity_label = "error".red().bold();
            
            writer.write_line(&format!(
                "{}: {}",
                severity_label,
                error.message
            ))?;
            
            writer.write_line(&format!(
                "  --> {}:{}:{}",
                path.display(),
                error.position.line,
                error.position.column
            ))?;
            
            if let Some(code) = &error.error_code {
                writer.write_line(&format!("  Код: {}", code.dimmed()))?;
            }
            
            writer.write_line("")?;
        }
        
        // Печатаем предупреждения, если не установлен флаг errors_only
        if !self.args.errors_only {
            for warning in warnings {
                let severity_label = "warning".yellow().bold();
                
                writer.write_line(&format!(
                    "{}: {}",
                    severity_label,
                    warning.message
                ))?;
                
                writer.write_line(&format!(
                    "  --> {}:{}:{}",
                    path.display(),
                    warning.position.line,
                    warning.position.column
                ))?;
                
                if let Some(code) = &warning.error_code {
                    writer.write_line(&format!("  Код: {}", code.dimmed()))?;
                }
                
                writer.write_line("")?;
            }
        }
        
        Ok(())
    }
    
    fn print_json_format(
        &self,
        writer: &mut OutputWriter,
        path: &Path,
        errors: &[AnalysisError],
        warnings: &[AnalysisError],
    ) -> Result<()> {
        let diagnostics: Vec<_> = errors.iter().map(|e| {
            serde_json::json!({
                "severity": "error",
                "message": e.message,
                "line": e.position.line,
                "column": e.position.column,
                "code": e.error_code
            })
        }).chain(warnings.iter().filter(|_| !self.args.errors_only).map(|w| {
            serde_json::json!({
                "severity": "warning",
                "message": w.message,
                "line": w.position.line,
                "column": w.position.column,
                "code": w.error_code
            })
        })).collect();
        
        let output = serde_json::json!({
            "file": path.to_str().unwrap_or("<unknown>"),
            "diagnostics": diagnostics,
            "hasErrors": !errors.is_empty(),
            "errorCount": errors.len(),
            "warningCount": warnings.len()
        });
        
        writer.write_object(&output)?;
        Ok(())
    }
    
    fn print_lsp_format(
        &self,
        writer: &mut OutputWriter,
        path: &Path,
        errors: &[AnalysisError],
        warnings: &[AnalysisError],
    ) -> Result<()> {
        let diagnostics: Vec<_> = errors.iter().map(|e| {
            serde_json::json!({
                "range": {
                    "start": {
                        "line": e.position.line.saturating_sub(1),
                        "character": e.position.column.saturating_sub(1)
                    },
                    "end": {
                        "line": e.position.line.saturating_sub(1),
                        "character": e.position.column + 20
                    }
                },
                "severity": 1, // Error
                "code": e.error_code,
                "message": e.message,
                "source": "bsl-analyzer"
            })
        }).chain(warnings.iter().filter(|_| !self.args.errors_only).map(|w| {
            serde_json::json!({
                "range": {
                    "start": {
                        "line": w.position.line.saturating_sub(1),
                        "character": w.position.column.saturating_sub(1)
                    },
                    "end": {
                        "line": w.position.line.saturating_sub(1),
                        "character": w.position.column + 20
                    }
                },
                "severity": 2, // Warning
                "code": w.error_code,
                "message": w.message,
                "source": "bsl-analyzer"
            })
        })).collect();
        
        let output = serde_json::json!({
            "uri": format!("file://{}", path.display()),
            "diagnostics": diagnostics
        });
        
        writer.write_object(&output)?;
        Ok(())
    }
    
    fn print_statistics(&self, stats: &CheckStats) -> Result<()> {
        let format = OutputFormat::from_str(&self.args.format)?;
        let mut writer = OutputWriter::stdout(format);
        
        writer.write_header("Статистика проверки")?;
        
        let rows = vec![
            vec!["Проверено файлов".to_string(), stats.files_checked.to_string()],
            vec!["Файлов с ошибками".to_string(), stats.files_with_errors.to_string()],
            vec!["Всего ошибок".to_string(), stats.total_errors.to_string()],
            vec!["Всего предупреждений".to_string(), stats.total_warnings.to_string()],
        ];
        
        writer.write_table(&["Параметр", "Значение"], rows)?;
        
        if stats.total_errors > 0 {
            cli_common::print_error(&format!("Обнаружено {} ошибок", stats.total_errors));
        } else {
            cli_common::print_success("Ошибок не обнаружено");
        }
        
        writer.flush()?;
        Ok(())
    }
}