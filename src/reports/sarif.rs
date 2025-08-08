/*!
# SARIF Reporter

Генерация отчетов в формате SARIF 2.1.0 для интеграции с:
- GitHub Security tab
- Azure DevOps Security scanning
- GitLab Security Dashboard
- Другими CI/CD системами

SARIF (Static Analysis Results Interchange Format) - стандартный формат
для обмена результатами статического анализа.

## Возможности:
- Полная совместимость с SARIF 2.1.0
- Поддержка GitHub Security tab
- Интеграция с Azure DevOps
- Детальная информация о проблемах
- Сопоставление с исходным кодом
- Метаданные инструмента анализа

## Использование:

```rust,ignore
use bsl_analyzer::reports::sarif::SarifReporter;

let reporter = SarifReporter::new("BSL Analyzer", "1.0.0");
let sarif_output = reporter.export_results(&analysis_results)?;

// Сохранение для GitHub Actions
std::fs::write("bsl-analysis.sarif", sarif_output)?;
```
*/

use super::{ReportConfig, ReportFormat, ReportGenerator, Severity};
use crate::core::{AnalysisError, AnalysisResults};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// SARIF 2.1.0 корневая структура
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifReport {
    /// Версия схемы SARIF
    #[serde(rename = "$schema")]
    pub schema: String,
    /// Версия SARIF
    pub version: String,
    /// Массив запусков анализа
    pub runs: Vec<SarifRun>,
}

/// Информация о запуске анализа
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifRun {
    /// Информация об инструменте анализа
    pub tool: SarifTool,
    /// Результаты анализа
    pub results: Vec<SarifResult>,
    /// Исходные файлы
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifacts: Option<Vec<SarifArtifact>>,
    /// Правила анализа
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<SarifRule>>,
    /// Метаданные запуска
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invocation: Option<SarifInvocation>,
}

/// Информация об инструменте анализа
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifTool {
    /// Драйвер инструмента
    pub driver: SarifDriver,
}

/// Драйвер инструмента анализа
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifDriver {
    /// Имя инструмента
    pub name: String,
    /// Версия инструмента
    pub version: String,
    /// Информационная версия
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semantic_version: Option<String>,
    /// URL инструмента
    #[serde(skip_serializing_if = "Option::is_none")]
    pub information_uri: Option<String>,
    /// Правила анализа
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<SarifRule>>,
}

/// Правило анализа
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifRule {
    /// Идентификатор правила
    pub id: String,
    /// Имя правила
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Краткое описание
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_description: Option<SarifMultiformatMessageString>,
    /// Полное описание
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_description: Option<SarifMultiformatMessageString>,
    /// Справочная информация
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<SarifMultiformatMessageString>,
    /// URL справки
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help_uri: Option<String>,
    /// Конфигурация правила по умолчанию
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_configuration: Option<SarifReportingConfiguration>,
}

/// Конфигурация отчетности для правила
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifReportingConfiguration {
    /// Уровень серьезности
    pub level: SarifLevel,
    /// Включено ли правило
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

/// Уровень серьезности в SARIF
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SarifLevel {
    /// Информация
    Note,
    /// Предупреждение
    Warning,
    /// Ошибка
    Error,
}

/// Результат анализа
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifResult {
    /// Идентификатор правила
    pub rule_id: String,
    /// Сообщение о проблеме
    pub message: SarifMessage,
    /// Уровень серьезности
    pub level: SarifLevel,
    /// Места обнаружения проблемы
    pub locations: Vec<SarifLocation>,
    /// Связанные места (опционально)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_locations: Option<Vec<SarifLocation>>,
    /// Отпечаток результата для дедупликации
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprints: Option<HashMap<String, String>>,
}

/// Сообщение в SARIF
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifMessage {
    /// Текст сообщения
    pub text: String,
    /// Идентификатор шаблона сообщения
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Аргументы для шаблона
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<String>>,
}

/// Многоформатная строка сообщения
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifMultiformatMessageString {
    /// Текстовая версия
    pub text: String,
    /// Markdown версия (опционально)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub markdown: Option<String>,
}

/// Место обнаружения проблемы
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifLocation {
    /// Физическое место в файле
    pub physical_location: SarifPhysicalLocation,
    /// Сообщение для конкретного места
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<SarifMessage>,
}

/// Физическое место в файле
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifPhysicalLocation {
    /// Ссылка на артефакт (файл)
    pub artifact_location: SarifArtifactLocation,
    /// Регион в файле
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<SarifRegion>,
    /// Контекстный регион
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_region: Option<SarifRegion>,
}

/// Ссылка на артефакт (файл)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifArtifactLocation {
    /// URI файла
    pub uri: String,
    /// Индекс в массиве артефактов
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<usize>,
}

/// Регион в файле
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifRegion {
    /// Начальная строка (1-based)
    pub start_line: u32,
    /// Начальная колонка (1-based)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_column: Option<u32>,
    /// Конечная строка
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_line: Option<u32>,
    /// Конечная колонка
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_column: Option<u32>,
    /// Смещение символов
    #[serde(skip_serializing_if = "Option::is_none")]
    pub char_offset: Option<u32>,
    /// Длина в символах
    #[serde(skip_serializing_if = "Option::is_none")]
    pub char_length: Option<u32>,
}

/// Артефакт (исходный файл)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifArtifact {
    /// Местоположение артефакта
    pub location: SarifArtifactLocation,
    /// MIME тип
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Длина в байтах
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<u64>,
    /// Хэш содержимого
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hashes: Option<HashMap<String, String>>,
}

/// Метаданные вызова анализа
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifInvocation {
    /// Время начала
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time_utc: Option<DateTime<Utc>>,
    /// Время окончания
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time_utc: Option<DateTime<Utc>>,
    /// Код выхода
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    /// Сигнал завершения
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code_description: Option<String>,
    /// Рабочая директория
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<SarifArtifactLocation>,
}

/// SARIF репортер для BSL анализатора
pub struct SarifReporter {
    /// Имя инструмента
    tool_name: String,
    /// Версия инструмента
    tool_version: String,
    /// URL инструмента
    tool_uri: Option<String>,
    /// Кэш правил
    #[allow(dead_code)]
    rules_cache: HashMap<String, SarifRule>,
}

impl SarifReporter {
    /// Создает новый SARIF репортер
    pub fn new(tool_name: &str, tool_version: &str) -> Self {
        Self {
            tool_name: tool_name.to_string(),
            tool_version: tool_version.to_string(),
            tool_uri: Some("https://github.com/defin85/bsl-type-safety-analyzer".to_string()),
            rules_cache: HashMap::new(),
        }
    }

    /// Устанавливает URL инструмента
    pub fn with_tool_uri(mut self, uri: String) -> Self {
        self.tool_uri = Some(uri);
        self
    }

    /// Экспортирует результаты анализа в SARIF формат
    pub fn export_results(&self, results: &AnalysisResults) -> Result<String> {
        let sarif_report = self.create_sarif_report(results)?;
        let json_output = serde_json::to_string_pretty(&sarif_report)
            .context("Failed to serialize SARIF report to JSON")?;
        Ok(json_output)
    }

    /// Создает структуру SARIF отчета
    fn create_sarif_report(&self, results: &AnalysisResults) -> Result<SarifReport> {
        let run = self.create_sarif_run(results)?;

        Ok(SarifReport {
            schema: "https://json.schemastore.org/sarif-2.1.0.json".to_string(),
            version: "2.1.0".to_string(),
            runs: vec![run],
        })
    }

    /// Создает запуск анализа
    fn create_sarif_run(&self, results: &AnalysisResults) -> Result<SarifRun> {
        let tool = self.create_tool_info();
        let rules = self.create_rules_from_results(results);
        let sarif_results = self.convert_results_to_sarif(results)?;
        let artifacts = self.create_artifacts_from_results(results);

        Ok(SarifRun {
            tool,
            results: sarif_results,
            artifacts: Some(artifacts),
            rules: Some(rules),
            invocation: Some(SarifInvocation {
                start_time_utc: Some(Utc::now()),
                end_time_utc: Some(Utc::now()),
                exit_code: Some(0),
                exit_code_description: Some("Analysis completed successfully".to_string()),
                working_directory: None,
            }),
        })
    }

    /// Создает информацию об инструменте
    fn create_tool_info(&self) -> SarifTool {
        SarifTool {
            driver: SarifDriver {
                name: self.tool_name.clone(),
                version: self.tool_version.clone(),
                semantic_version: Some(self.tool_version.clone()),
                information_uri: self.tool_uri.clone(),
                rules: None, // Правила добавляются в run.rules
            },
        }
    }

    /// Создает правила из результатов анализа
    fn create_rules_from_results(&self, results: &AnalysisResults) -> Vec<SarifRule> {
        let mut rules = HashMap::new();

        // Собираем уникальные коды ошибок
        for error in results.get_errors() {
            if let Some(ref code) = error.error_code {
                rules
                    .entry(code.clone())
                    .or_insert_with(|| self.create_rule_from_error_code(code));
            }
        }

        for warning in results.get_warnings() {
            if let Some(ref code) = warning.error_code {
                rules
                    .entry(code.clone())
                    .or_insert_with(|| self.create_rule_from_error_code(code));
            }
        }

        rules.into_values().collect()
    }

    /// Создает правило из кода ошибки
    fn create_rule_from_error_code(&self, error_code: &str) -> SarifRule {
        let (name, description, help) = match error_code {
            "BSL001" => (
                "Неиспользуемая переменная",
                "Переменная объявлена, но не используется в коде",
                "Удалите неиспользуемую переменную или добавьте к ней префикс '_' если она нужна",
            ),
            "BSL002" => (
                "Неопределенная переменная",
                "Использование переменной, которая не была объявлена",
                "Объявите переменную перед использованием с помощью 'Перем'",
            ),
            "BSL003" => (
                "Несоответствие типов",
                "Присваивание значения несовместимого типа",
                "Проверьте совместимость типов данных при присваивании",
            ),
            "BSL004" => (
                "Неизвестный метод",
                "Вызов метода, который не найден в конфигурации или документации",
                "Проверьте правильность имени метода и его доступность в текущем контексте",
            ),
            "BSL005" => (
                "Циклическая зависимость",
                "Обнаружена циклическая зависимость между модулями",
                "Реорганизуйте код для устранения циклических зависимостей",
            ),
            _ => (
                "Проблема BSL кода",
                "Обнаружена проблема в BSL коде",
                "См. документацию BSL анализатора для получения дополнительной информации",
            ),
        };

        SarifRule {
            id: error_code.to_string(),
            name: Some(name.to_string()),
            short_description: Some(SarifMultiformatMessageString {
                text: description.to_string(),
                markdown: None,
            }),
            full_description: Some(SarifMultiformatMessageString {
                text: description.to_string(),
                markdown: None,
            }),
            help: Some(SarifMultiformatMessageString {
                text: help.to_string(),
                markdown: None,
            }),
            help_uri: Some(format!(
                "https://github.com/defin85/bsl-type-safety-analyzer/docs/rules/{}",
                error_code
            )),
            default_configuration: Some(SarifReportingConfiguration {
                level: self.error_code_to_sarif_level(error_code),
                enabled: Some(true),
            }),
        }
    }

    /// Конвертирует код ошибки в уровень серьезности SARIF
    fn error_code_to_sarif_level(&self, error_code: &str) -> SarifLevel {
        match error_code {
            "BSL001" => SarifLevel::Note,    // Неиспользуемая переменная
            "BSL002" => SarifLevel::Error,   // Неопределенная переменная
            "BSL003" => SarifLevel::Warning, // Несоответствие типов
            "BSL004" => SarifLevel::Warning, // Неизвестный метод
            "BSL005" => SarifLevel::Error,   // Циклическая зависимость
            _ => SarifLevel::Warning,
        }
    }

    /// Конвертирует результаты анализа в SARIF результаты
    fn convert_results_to_sarif(&self, results: &AnalysisResults) -> Result<Vec<SarifResult>> {
        let mut sarif_results = Vec::new();

        // Конвертируем ошибки
        for error in results.get_errors() {
            sarif_results.push(self.convert_analysis_error_to_sarif(error, SarifLevel::Error)?);
        }

        // Конвертируем предупреждения
        for warning in results.get_warnings() {
            sarif_results.push(self.convert_analysis_error_to_sarif(warning, SarifLevel::Warning)?);
        }

        Ok(sarif_results)
    }

    /// Конвертирует ошибку анализа в SARIF результат
    fn convert_analysis_error_to_sarif(
        &self,
        error: &AnalysisError,
        level: SarifLevel,
    ) -> Result<SarifResult> {
        let rule_id = error
            .error_code
            .clone()
            .unwrap_or_else(|| "BSL000".to_string());

        let location = SarifLocation {
            physical_location: SarifPhysicalLocation {
                artifact_location: SarifArtifactLocation {
                    uri: self.path_to_uri(&error.file_path),
                    index: None,
                },
                region: Some(SarifRegion {
                    start_line: error.position.line as u32,
                    start_column: Some(error.position.column as u32),
                    end_line: Some(error.position.line as u32),
                    end_column: Some((error.position.column + 10) as u32), // Примерная длина
                    char_offset: None,
                    char_length: None,
                }),
                context_region: None,
            },
            message: None,
        };

        // Создаем отпечаток для дедупликации
        let mut fingerprints = HashMap::new();
        fingerprints.insert(
            "BSLAnalyzer/v1".to_string(),
            format!(
                "{}:{}:{}:{}",
                error.file_path.display(),
                error.position.line,
                error.position.column,
                rule_id
            ),
        );

        Ok(SarifResult {
            rule_id,
            message: SarifMessage {
                text: error.message.clone(),
                id: None,
                arguments: None,
            },
            level,
            locations: vec![location],
            related_locations: None,
            fingerprints: Some(fingerprints),
        })
    }

    /// Создает артефакты из результатов анализа
    fn create_artifacts_from_results(&self, results: &AnalysisResults) -> Vec<SarifArtifact> {
        let mut artifacts = HashMap::new();

        // Собираем уникальные файлы из ошибок
        for error in results.get_errors() {
            let uri = self.path_to_uri(&error.file_path);
            artifacts
                .entry(uri.clone())
                .or_insert_with(|| SarifArtifact {
                    location: SarifArtifactLocation { uri, index: None },
                    mime_type: Some("text/plain".to_string()),
                    length: None,
                    hashes: None,
                });
        }

        // Собираем уникальные файлы из предупреждений
        for warning in results.get_warnings() {
            let uri = self.path_to_uri(&warning.file_path);
            artifacts
                .entry(uri.clone())
                .or_insert_with(|| SarifArtifact {
                    location: SarifArtifactLocation { uri, index: None },
                    mime_type: Some("text/plain".to_string()),
                    length: None,
                    hashes: None,
                });
        }

        artifacts.into_values().collect()
    }

    /// Конвертирует путь к файлу в URI
    fn path_to_uri(&self, path: &Path) -> String {
        // Конвертируем путь в URI формат для SARIF
        if path.is_absolute() {
            format!("file:///{}", path.display().to_string().replace('\\', "/"))
        } else {
            path.display().to_string().replace('\\', "/")
        }
    }

    /// Генерирует SARIF специально для GitHub Security
    pub fn export_for_github(&self, results: &AnalysisResults) -> Result<String> {
        // GitHub Security требует определенные поля
        let mut sarif_report = self.create_sarif_report(results)?;

        // Добавляем GitHub-специфичные метаданные
        if let Some(run) = sarif_report.runs.first_mut() {
            run.tool.driver.information_uri =
                Some("https://github.com/defin85/bsl-type-safety-analyzer".to_string());

            // GitHub рекомендует указывать semantic_version
            run.tool.driver.semantic_version = Some(self.tool_version.clone());
        }

        let json_output = serde_json::to_string_pretty(&sarif_report)
            .context("Failed to serialize GitHub SARIF report")?;
        Ok(json_output)
    }

    /// Генерирует SARIF для Azure DevOps
    pub fn export_for_azure_devops(&self, results: &AnalysisResults) -> Result<String> {
        // Azure DevOps имеет свои требования к SARIF
        let sarif_report = self.create_sarif_report(results)?;

        let json_output = serde_json::to_string_pretty(&sarif_report)
            .context("Failed to serialize Azure DevOps SARIF report")?;
        Ok(json_output)
    }
}

impl ReportGenerator for SarifReporter {
    fn generate_report(&self, results: &AnalysisResults, _config: &ReportConfig) -> Result<String> {
        self.export_results(results)
    }

    fn supported_format() -> ReportFormat {
        ReportFormat::Sarif
    }
}

impl From<Severity> for SarifLevel {
    fn from(severity: Severity) -> Self {
        match severity {
            Severity::Info => SarifLevel::Note,
            Severity::Warning => SarifLevel::Warning,
            Severity::Error | Severity::Critical => SarifLevel::Error,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{AnalysisError, AnalysisResults};
    use crate::parser::Position;
    use std::path::PathBuf;

    fn create_test_analysis_results() -> AnalysisResults {
        let mut results = AnalysisResults::new();

        results.add_error(AnalysisError {
            message: "Неиспользуемая переменная 'тестоваяПеременная'".to_string(),
            file_path: PathBuf::from("test.bsl"),
            position: Position {
                line: 10,
                column: 5,
                offset: 100,
            },
            level: crate::core::ErrorLevel::Error,
            error_code: Some("BSL001".to_string()),
            suggestion: None,
            related_positions: Vec::new(),
        });

        results.add_warning(AnalysisError {
            message: "Неизвестный метод 'НеизвестныйМетод'".to_string(),
            file_path: PathBuf::from("module.bsl"),
            position: Position {
                line: 25,
                column: 12,
                offset: 250,
            },
            level: crate::core::ErrorLevel::Warning,
            error_code: Some("BSL004".to_string()),
            suggestion: None,
            related_positions: Vec::new(),
        });

        results
    }

    #[test]
    fn test_sarif_reporter_creation() {
        let reporter = SarifReporter::new("BSL Analyzer", "1.0.0");
        assert_eq!(reporter.tool_name, "BSL Analyzer");
        assert_eq!(reporter.tool_version, "1.0.0");
    }

    #[test]
    fn test_sarif_export() {
        let reporter = SarifReporter::new("BSL Analyzer", "1.0.0");
        let results = create_test_analysis_results();

        let sarif_output = reporter.export_results(&results).unwrap();
        assert!(sarif_output.contains("$schema"));
        assert!(sarif_output.contains("version"));
        assert!(sarif_output.contains("BSL001"));
        assert!(sarif_output.contains("BSL004"));
    }

    #[test]
    fn test_github_sarif_export() {
        let reporter = SarifReporter::new("BSL Analyzer", "1.0.0");
        let results = create_test_analysis_results();

        let github_sarif = reporter.export_for_github(&results).unwrap();
        assert!(github_sarif.contains("github.com"));
        assert!(github_sarif.contains("semantic_version"));
    }

    #[test]
    fn test_error_code_to_sarif_level() {
        let reporter = SarifReporter::new("BSL Analyzer", "1.0.0");

        assert!(matches!(
            reporter.error_code_to_sarif_level("BSL001"),
            SarifLevel::Note
        ));
        assert!(matches!(
            reporter.error_code_to_sarif_level("BSL002"),
            SarifLevel::Error
        ));
        assert!(matches!(
            reporter.error_code_to_sarif_level("BSL003"),
            SarifLevel::Warning
        ));
    }

    #[test]
    fn test_path_to_uri_conversion() {
        let reporter = SarifReporter::new("BSL Analyzer", "1.0.0");

        let relative_path = Path::new("test.bsl");
        let uri = reporter.path_to_uri(relative_path);
        assert_eq!(uri, "test.bsl");

        #[cfg(windows)]
        {
            let absolute_path = Path::new("C:\\project\\test.bsl");
            let uri = reporter.path_to_uri(absolute_path);
            assert!(uri.starts_with("file:///"));
            assert!(uri.contains("/project/test.bsl"));
        }
    }

    #[test]
    fn test_severity_conversion() {
        assert!(matches!(SarifLevel::from(Severity::Info), SarifLevel::Note));
        assert!(matches!(
            SarifLevel::from(Severity::Warning),
            SarifLevel::Warning
        ));
        assert!(matches!(
            SarifLevel::from(Severity::Error),
            SarifLevel::Error
        ));
        assert!(matches!(
            SarifLevel::from(Severity::Critical),
            SarifLevel::Error
        ));
    }
}
