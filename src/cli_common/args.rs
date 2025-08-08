//! Общие аргументы командной строки для CLI утилит

use clap::Parser;
use std::path::PathBuf;

/// Общие аргументы для всех CLI команд
#[derive(Parser, Debug, Clone)]
pub struct CommonArgs {
    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Output format (json, text, table)
    #[arg(long, default_value = "text")]
    pub format: String,

    /// Suppress all output except errors
    #[arg(short, long)]
    pub quiet: bool,
}

/// Аргументы для работы с платформой 1С
#[derive(Parser, Debug, Clone)]
pub struct PlatformArgs {
    /// Platform version (e.g., "8.3.25")
    #[arg(short = 'p', long = "platform-version", default_value = "8.3.25")]
    pub platform_version: String,

    /// Path to platform documentation archive (optional)
    #[arg(long = "platform-docs-archive")]
    pub platform_docs_archive: Option<PathBuf>,

    /// Application mode (ordinary, managed, mixed)
    #[arg(long, default_value = "managed")]
    pub mode: String,
}

/// Аргументы для работы с конфигурацией
#[derive(Parser, Debug, Clone)]
pub struct ConfigArgs {
    /// Path to 1C configuration directory
    #[arg(short, long)]
    pub config: PathBuf,

    /// Force rebuild index even if cache exists
    #[arg(short, long)]
    pub force: bool,

    /// Skip cache and always rebuild
    #[arg(long)]
    pub no_cache: bool,
}

/// Аргументы для работы с выводом
#[derive(Parser, Debug, Clone)]
pub struct OutputArgs {
    /// Output file path (stdout if not specified)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Pretty print JSON output
    #[arg(long)]
    pub pretty: bool,

    /// Include additional details in output
    #[arg(long)]
    pub detailed: bool,
}

/// Объединенные аргументы для утилит работающих с индексом
#[derive(Parser, Debug)]
pub struct IndexArgs {
    #[clap(flatten)]
    pub common: CommonArgs,

    #[clap(flatten)]
    pub platform: PlatformArgs,

    #[clap(flatten)]
    pub config: ConfigArgs,
}

/// Объединенные аргументы для утилит экспорта
#[derive(Parser, Debug)]
pub struct ExportArgs {
    #[clap(flatten)]
    pub common: CommonArgs,

    #[clap(flatten)]
    pub output: OutputArgs,
}

impl CommonArgs {
    /// Определяет уровень логирования на основе флагов
    pub fn log_level(&self) -> tracing::Level {
        if self.quiet {
            tracing::Level::ERROR
        } else if self.verbose {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        }
    }

    /// Проверяет, нужно ли выводить информацию
    pub fn should_print(&self) -> bool {
        !self.quiet
    }
}

impl PlatformArgs {
    /// Нормализует версию платформы (убирает префикс v если есть)
    pub fn normalized_version(&self) -> String {
        self.platform_version
            .strip_prefix("v")
            .unwrap_or(&self.platform_version)
            .to_string()
    }

    /// Валидирует режим приложения
    pub fn validate_mode(&self) -> Result<(), String> {
        match self.mode.as_str() {
            "ordinary" | "managed" | "mixed" => Ok(()),
            _ => Err(format!(
                "Invalid application mode: {}. Must be one of: ordinary, managed, mixed",
                self.mode
            )),
        }
    }
}

impl ConfigArgs {
    /// Проверяет существование конфигурации
    pub fn validate(&self) -> Result<(), String> {
        if !self.config.exists() {
            return Err(format!(
                "Configuration directory does not exist: {}",
                self.config.display()
            ));
        }

        if !self.config.is_dir() {
            return Err(format!(
                "Configuration path is not a directory: {}",
                self.config.display()
            ));
        }

        Ok(())
    }

    /// Определяет, нужно ли использовать кеш
    pub fn use_cache(&self) -> bool {
        !self.no_cache && !self.force
    }
}
