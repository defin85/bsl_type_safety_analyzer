//! CLI утилита для проверки синтаксиса BSL файлов

use anyhow::Result;
use clap::Parser as ClapParser;
use std::path::{Path, PathBuf};
use std::io::{self, Write};
use bsl_analyzer::bsl_parser::{BslParser, ParseResult, DiagnosticSeverity};
use walkdir::WalkDir;
use console::style;

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

    /// Формат вывода
    #[arg(short, long, value_enum, default_value = "human")]
    format: OutputFormat,

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
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum OutputFormat {
    Human,
    Json,
    Lsp,
}

struct CheckStats {
    files_checked: usize,
    files_with_errors: usize,
    total_errors: usize,
    total_warnings: usize,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    let mut stats = CheckStats {
        files_checked: 0,
        files_with_errors: 0,
        total_errors: 0,
        total_warnings: 0,
    };

    let mut parser = BslParser::new()?;
    
    if args.path.is_file() {
        check_file(&args.path, &mut parser, &args, &mut stats)?;
    } else if args.path.is_dir() {
        check_directory(&args.path, &mut parser, &args, &mut stats)?;
    } else {
        eprintln!("Ошибка: {} не является файлом или директорией", args.path.display());
        std::process::exit(1);
    }

    if args.stats && !args.quiet {
        print_stats(&stats);
    }

    // Возвращаем ненулевой код, если были ошибки
    if stats.files_with_errors > 0 {
        std::process::exit(1);
    }

    Ok(())
}

fn check_file(
    path: &Path,
    parser: &mut BslParser,
    args: &Args,
    stats: &mut CheckStats,
) -> Result<()> {
    if path.extension().and_then(|e| e.to_str()) != Some("bsl") {
        return Ok(());
    }

    let content = std::fs::read_to_string(path)?;
    let result = parser.parse(&content, path.to_str().unwrap_or("<unknown>"));
    
    stats.files_checked += 1;
    
    let errors: Vec<_> = result.diagnostics.iter()
        .filter(|d| d.severity == DiagnosticSeverity::Error)
        .collect();
    
    let warnings: Vec<_> = result.diagnostics.iter()
        .filter(|d| d.severity == DiagnosticSeverity::Warning)
        .collect();
    
    stats.total_errors += errors.len();
    stats.total_warnings += warnings.len();
    
    if !errors.is_empty() {
        stats.files_with_errors += 1;
    }
    
    if args.quiet {
        return Ok(());
    }
    
    match args.format {
        OutputFormat::Human => print_human_format(&result, path, args.errors_only),
        OutputFormat::Json => print_json_format(&result, path),
        OutputFormat::Lsp => print_lsp_format(&result, path),
    }
    
    Ok(())
}

fn check_directory(
    path: &Path,
    parser: &mut BslParser,
    args: &Args,
    stats: &mut CheckStats,
) -> Result<()> {
    let walker = if args.recursive {
        WalkDir::new(path)
    } else {
        WalkDir::new(path).max_depth(1)
    };
    
    for entry in walker {
        let entry = entry?;
        if entry.file_type().is_file() {
            check_file(entry.path(), parser, args, stats)?;
        }
    }
    
    Ok(())
}

fn print_human_format(result: &ParseResult, path: &Path, errors_only: bool) {
    let mut has_output = false;
    
    for diag in &result.diagnostics {
        if errors_only && diag.severity != DiagnosticSeverity::Error {
            continue;
        }
        
        has_output = true;
        
        let severity_style = match diag.severity {
            DiagnosticSeverity::Error => style("error").red().bold(),
            DiagnosticSeverity::Warning => style("warning").yellow().bold(),
            DiagnosticSeverity::Information => style("info").blue(),
            DiagnosticSeverity::Hint => style("hint").cyan(),
        };
        
        println!(
            "{}: {} ({})",
            severity_style,
            diag.message,
            diag.code
        );
        
        println!(
            "  --> {}:{}:{}",
            path.display(),
            diag.location.line,
            diag.location.column
        );
        
        if let Some(found) = &diag.details.found {
            println!("  Найдено: {}", style(found).dim());
        }
        
        if let Some(expected) = &diag.details.expected {
            println!("  Ожидалось: {}", style(expected).green());
        }
        
        if let Some(suggestion) = &diag.details.suggestion {
            println!("  Предложение: {}", style(suggestion).cyan());
        }
        
        println!();
    }
    
    if has_output {
        io::stdout().flush().unwrap();
    }
}

fn print_json_format(result: &ParseResult, path: &Path) {
    let output = serde_json::json!({
        "file": path.to_str().unwrap_or("<unknown>"),
        "diagnostics": result.diagnostics,
        "hasErrors": result.diagnostics.iter().any(|d| d.severity == DiagnosticSeverity::Error)
    });
    
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

fn print_lsp_format(result: &ParseResult, path: &Path) {
    // LSP-совместимый формат диагностики
    let diagnostics: Vec<_> = result.diagnostics.iter().map(|diag| {
        serde_json::json!({
            "range": {
                "start": {
                    "line": diag.location.line - 1,
                    "character": diag.location.column - 1
                },
                "end": {
                    "line": diag.location.line - 1,
                    "character": diag.location.column - 1 + diag.location.length
                }
            },
            "severity": match diag.severity {
                DiagnosticSeverity::Error => 1,
                DiagnosticSeverity::Warning => 2,
                DiagnosticSeverity::Information => 3,
                DiagnosticSeverity::Hint => 4,
            },
            "code": diag.code,
            "message": diag.message,
            "data": diag.details
        })
    }).collect();
    
    let output = serde_json::json!({
        "uri": format!("file://{}", path.display()),
        "diagnostics": diagnostics
    });
    
    println!("{}", serde_json::to_string(&output).unwrap());
}

fn print_stats(stats: &CheckStats) {
    println!();
    println!("{}", style("=== Статистика проверки ===").bold());
    println!("Проверено файлов: {}", stats.files_checked);
    println!("Файлов с ошибками: {}", stats.files_with_errors);
    println!(
        "Всего ошибок: {}",
        if stats.total_errors > 0 {
            style(stats.total_errors).red()
        } else {
            style(stats.total_errors).green()
        }
    );
    println!(
        "Всего предупреждений: {}",
        if stats.total_warnings > 0 {
            style(stats.total_warnings).yellow()
        } else {
            style(stats.total_warnings).green()
        }
    );
}