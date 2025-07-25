/*!
# Reports Module

Модуль для генерации отчетов анализа в различных форматах.
Поддерживает SARIF для интеграции с CI/CD системами.

## Поддерживаемые форматы:
- **SARIF 2.1.0** - для GitHub Security, Azure DevOps, других CI/CD
- **JSON** - структурированный отчет для API интеграции
- **Text** - человекочитаемый отчет для консоли
- **HTML** - детальный отчет с визуализацией

## Использование:

```rust
use bsl_analyzer::reports::{SarifReporter, ReportFormat};

let results = analyzer.get_results();
let sarif_reporter = SarifReporter::new("BSL Type Safety Analyzer", "1.0.0");
let sarif_output = sarif_reporter.export_results(&results)?;

// Сохранение для GitHub Actions
std::fs::write("analysis-results.sarif", sarif_output)?;
```
*/

pub mod sarif;
pub mod html;
pub mod text;

pub use sarif::{SarifReporter, SarifResult, SarifRule, SarifLocation};
pub use html::HtmlReporter;
pub use text::TextReporter;

use std::path::Path;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use crate::core::AnalysisResults;

/// Формат отчета
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportFormat {
    /// SARIF 2.1.0 для CI/CD интеграции
    Sarif,
    /// JSON для API интеграции
    Json,
    /// HTML для веб-просмотра
    Html,
    /// Текстовый отчет для консоли
    Text,
}

impl std::str::FromStr for ReportFormat {
    type Err = anyhow::Error;
    
    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "sarif" => Ok(ReportFormat::Sarif),
            "json" => Ok(ReportFormat::Json),
            "html" => Ok(ReportFormat::Html),
            "text" => Ok(ReportFormat::Text),
            _ => Err(anyhow::anyhow!("Unknown report format: {}", s))
        }
    }
}

/// Конфигурация отчета
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportConfig {
    /// Формат отчета
    pub format: ReportFormat,
    /// Путь для сохранения отчета
    pub output_path: Option<String>,
    /// Включить детальную информацию
    pub include_details: bool,
    /// Включить метрики производительности
    pub include_performance: bool,
    /// Включить граф зависимостей
    pub include_dependencies: bool,
    /// Фильтровать по серьезности
    pub min_severity: Option<Severity>,
}

/// Серьезность проблемы
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// Информация
    Info,
    /// Предупреждение
    Warning,
    /// Ошибка
    Error,
    /// Критическая ошибка
    Critical,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Info => write!(f, "info"),
            Severity::Warning => write!(f, "warning"),
            Severity::Error => write!(f, "error"),
            Severity::Critical => write!(f, "critical"),
        }
    }
}

/// Трейт для генерации отчетов
pub trait ReportGenerator {
    /// Генерирует отчет на основе результатов анализа
    fn generate_report(&self, results: &AnalysisResults, config: &ReportConfig) -> Result<String>;
    
    /// Возвращает поддерживаемый формат отчета
    fn supported_format() -> ReportFormat;
    
    /// Валидирует конфигурацию отчета
    fn validate_config(config: &ReportConfig) -> Result<()> {
        if let Some(ref path) = config.output_path {
            if let Some(parent) = Path::new(path).parent() {
                if !parent.exists() {
                    return Err(anyhow::anyhow!("Output directory does not exist: {}", parent.display()));
                }
            }
        }
        Ok(())
    }
}

/// Менеджер отчетов для генерации в различных форматах
pub struct ReportManager {
    /// Конфигурация по умолчанию
    default_config: ReportConfig,
}

impl ReportManager {
    /// Создает новый менеджер отчетов
    pub fn new() -> Self {
        Self {
            default_config: ReportConfig {
                format: ReportFormat::Json,
                output_path: None,
                include_details: true,
                include_performance: false,
                include_dependencies: false,
                min_severity: None,
            }
        }
    }
    
    /// Создает менеджер с конфигурацией
    pub fn with_config(config: ReportConfig) -> Self {
        Self {
            default_config: config,
        }
    }
    
    /// Генерирует отчет в указанном формате
    pub fn generate_report(&self, results: &AnalysisResults, format: ReportFormat) -> Result<String> {
        let mut config = self.default_config.clone();
        config.format = format;
        
        match config.format {
            ReportFormat::Sarif => {
                let reporter = SarifReporter::new("BSL Type Safety Analyzer", env!("CARGO_PKG_VERSION"));
                reporter.generate_report(results, &config)
            },
            ReportFormat::Json => {
                let json_output = serde_json::to_string_pretty(results)?;
                Ok(json_output)
            },
            ReportFormat::Html => {
                let reporter = HtmlReporter::new();
                reporter.generate_report(results, &config)
            },
            ReportFormat::Text => {
                let reporter = TextReporter::new();
                reporter.generate_report(results, &config)
            },
        }
    }
    
    /// Генерирует отчет с конфигурацией
    pub fn generate_with_config(&self, results: &AnalysisResults, config: &ReportConfig) -> Result<String> {
        match config.format {
            ReportFormat::Sarif => {
                let reporter = SarifReporter::new("BSL Type Safety Analyzer", env!("CARGO_PKG_VERSION"));
                reporter.generate_report(results, config)
            },
            ReportFormat::Json => {
                let json_output = serde_json::to_string_pretty(results)?;
                Ok(json_output)
            },
            ReportFormat::Html => {
                let reporter = HtmlReporter::new();
                reporter.generate_report(results, config)
            },
            ReportFormat::Text => {
                let reporter = TextReporter::new();
                reporter.generate_report(results, config)
            },
        }
    }
    
    /// Сохраняет отчет в файл
    pub fn save_report<P: AsRef<Path>>(&self, results: &AnalysisResults, 
                                      format: ReportFormat, output_path: P) -> Result<()> {
        let report_content = self.generate_report(results, format)?;
        std::fs::write(output_path, report_content)?;
        Ok(())
    }
    
    /// Генерирует отчеты во всех поддерживаемых форматах
    pub fn generate_all_formats(&self, results: &AnalysisResults, output_dir: &Path) -> Result<()> {
        let formats = [
            (ReportFormat::Sarif, "analysis-results.sarif"),
            (ReportFormat::Json, "analysis-results.json"),
            (ReportFormat::Html, "analysis-results.html"),
            (ReportFormat::Text, "analysis-results.txt"),
        ];
        
        for (format, filename) in &formats {
            let output_path = output_dir.join(filename);
            self.save_report(results, format.clone(), &output_path)
                .map_err(|e| anyhow::anyhow!("Failed to generate {} report: {}", format, e))?;
            
            tracing::info!("Generated {} report: {}", format, output_path.display());
        }
        
        Ok(())
    }
}

impl std::fmt::Display for ReportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReportFormat::Sarif => write!(f, "SARIF"),
            ReportFormat::Json => write!(f, "JSON"),
            ReportFormat::Html => write!(f, "HTML"),
            ReportFormat::Text => write!(f, "Text"),
        }
    }
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            format: ReportFormat::Json,
            output_path: None,
            include_details: true,
            include_performance: false,
            include_dependencies: false,
            min_severity: None,
        }
    }
}

impl Default for ReportManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Вспомогательные функции для создания отчетов
pub mod utils {
    use super::*;
    
    /// Создает быстрый SARIF отчет для CI/CD
    pub fn quick_sarif_report(results: &AnalysisResults) -> Result<String> {
        let manager = ReportManager::new();
        manager.generate_report(results, ReportFormat::Sarif)
    }
    
    /// Создает детальный HTML отчет
    pub fn detailed_html_report(results: &AnalysisResults) -> Result<String> {
        let config = ReportConfig {
            format: ReportFormat::Html,
            include_details: true,
            include_performance: true,
            include_dependencies: true,
            ..Default::default()
        };
        
        let manager = ReportManager::with_config(config);
        manager.generate_with_config(results, &manager.default_config)
    }
    
    /// Создает консольный отчет с фильтрацией по серьезности
    pub fn console_report(results: &AnalysisResults, min_severity: Severity) -> Result<String> {
        let config = ReportConfig {
            format: ReportFormat::Text,
            min_severity: Some(min_severity),
            include_details: false,
            ..Default::default()
        };
        
        let manager = ReportManager::with_config(config);
        manager.generate_with_config(results, &manager.default_config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_report_format_parsing() {
        assert_eq!("sarif".parse::<ReportFormat>().unwrap(), ReportFormat::Sarif);
        assert_eq!("json".parse::<ReportFormat>().unwrap(), ReportFormat::Json);
        assert_eq!("html".parse::<ReportFormat>().unwrap(), ReportFormat::Html);
        assert_eq!("text".parse::<ReportFormat>().unwrap(), ReportFormat::Text);
        
        assert!("invalid".parse::<ReportFormat>().is_err());
    }
    
    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Info < Severity::Warning);
        assert!(Severity::Warning < Severity::Error);
        assert!(Severity::Error < Severity::Critical);
    }
    
    #[test]
    fn test_report_config_default() {
        let config = ReportConfig::default();
        assert_eq!(config.format, ReportFormat::Json);
        assert!(config.include_details);
        assert!(!config.include_performance);
    }
    
    #[test]
    fn test_report_manager_creation() {
        let manager = ReportManager::new();
        assert_eq!(manager.default_config.format, ReportFormat::Json);
        
        let custom_config = ReportConfig {
            format: ReportFormat::Sarif,
            ..Default::default()
        };
        let custom_manager = ReportManager::with_config(custom_config);
        assert_eq!(custom_manager.default_config.format, ReportFormat::Sarif);
    }
}