//! Общий модуль для CLI утилит
//!
//! Содержит общую функциональность, используемую всеми CLI бинарниками:
//! - Инициализация логирования
//! - Обработка ошибок
//! - Форматирование вывода
//! - Общие структуры и трейты

use anyhow::{Context, Result};
use colored::Colorize;
use std::path::{Path, PathBuf};
use tracing_subscriber::EnvFilter;

pub mod args;
pub mod output;
pub mod progress;

pub use args::{CommonArgs, ConfigArgs, PlatformArgs};
pub use output::{OutputFormat, OutputWriter};
pub use progress::{ProgressReporter, ProgressStyle};

/// Инициализирует систему логирования с настройками по умолчанию
pub fn init_logging(verbose: bool) -> Result<()> {
    let level = if verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    let filter = EnvFilter::from_default_env().add_directive(level.into());

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .init();

    Ok(())
}

/// Инициализирует минимальное логирование (только WARN и ERROR)
pub fn init_minimal_logging() -> Result<()> {
    let filter = EnvFilter::from_default_env().add_directive(tracing::Level::WARN.into());

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .init();

    Ok(())
}

/// Выводит заголовок CLI утилиты
pub fn print_header(name: &str, version: &str, description: &str) {
    println!(
        "{} {} - {}",
        "🔧".blue(),
        name.bold().blue(),
        version.dimmed()
    );
    println!("{}\n", description.dimmed());
}

/// Выводит успешное завершение операции
pub fn print_success(message: &str) {
    println!("{} {}", "✅".green(), message.green());
}

/// Выводит предупреждение
pub fn print_warning(message: &str) {
    println!("{} {}", "⚠️".yellow(), message.yellow());
}

/// Выводит ошибку
pub fn print_error(message: &str) {
    eprintln!("{} {}", "❌".red(), message.red());
}

/// Выводит информационное сообщение
pub fn print_info(message: &str) {
    println!("{} {}", "ℹ️".blue(), message);
}

/// Проверяет существование файла или директории
pub fn validate_path(path: &Path, description: &str) -> Result<()> {
    if !path.exists() {
        return Err(anyhow::anyhow!(
            "{} does not exist: {}",
            description,
            path.display()
        ));
    }
    Ok(())
}

/// Возвращает директорию кеша BSL анализатора
pub fn get_cache_dir() -> Result<PathBuf> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
    Ok(home_dir.join(".bsl_analyzer"))
}

/// Создает директорию если она не существует
pub fn ensure_dir_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)
            .with_context(|| format!("Failed to create directory: {}", path.display()))?;
    }
    Ok(())
}

/// Форматирует размер файла в человекочитаемый вид
pub fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

/// Форматирует продолжительность в человекочитаемый вид
pub fn format_duration(duration: std::time::Duration) -> String {
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();

    if secs == 0 {
        format!("{}ms", millis)
    } else if secs < 60 {
        format!("{}.{:03}s", secs, millis)
    } else {
        let mins = secs / 60;
        let secs = secs % 60;
        format!("{}m {}s", mins, secs)
    }
}

/// Trait для CLI команд
pub trait CliCommand {
    /// Выполняет команду
    fn execute(&self) -> Result<()>;

    /// Возвращает имя команды
    fn name(&self) -> &str;

    /// Возвращает описание команды
    fn description(&self) -> &str;
}

/// Запускает CLI команду с обработкой ошибок
pub fn run_command<C: CliCommand>(command: C) -> Result<()> {
    print_header(
        command.name(),
        env!("CARGO_PKG_VERSION"),
        command.description(),
    );

    match command.execute() {
        Ok(()) => {
            print_success(&format!("{} completed successfully", command.name()));
            Ok(())
        }
        Err(e) => {
            print_error(&format!("{} failed: {}", command.name(), e));
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(1023), "1023 B");
        assert_eq!(format_file_size(1024), "1.00 KB");
        assert_eq!(format_file_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_file_size(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_format_duration() {
        use std::time::Duration;

        assert_eq!(format_duration(Duration::from_millis(500)), "500ms");
        assert_eq!(format_duration(Duration::from_secs(1)), "1.000s");
        assert_eq!(format_duration(Duration::from_secs(65)), "1m 5s");
    }
}
