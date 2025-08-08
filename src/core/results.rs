/*!
# Analysis Results

Структуры для хранения и управления результатами анализа BSL кода.
Используется репортерами для генерации различных форматов отчетов.
*/

use super::AnalysisError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Результаты анализа BSL кода
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResults {
    /// Список ошибок
    errors: Vec<AnalysisError>,
    /// Список предупреждений
    warnings: Vec<AnalysisError>,
    /// Метаданные анализа
    metadata: AnalysisMetadata,
}

/// Метаданные анализа
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisMetadata {
    /// Время начала анализа
    pub start_time: Option<std::time::SystemTime>,
    /// Время окончания анализа
    pub end_time: Option<std::time::SystemTime>,
    /// Версия анализатора
    pub analyzer_version: String,
    /// Количество проанализированных файлов
    pub files_analyzed: usize,
    /// Дополнительные метрики
    pub metrics: HashMap<String, String>,
}

impl AnalysisResults {
    /// Создает новые пустые результаты анализа
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            metadata: AnalysisMetadata {
                start_time: Some(std::time::SystemTime::now()),
                end_time: None,
                analyzer_version: env!("CARGO_PKG_VERSION").to_string(),
                files_analyzed: 0,
                metrics: HashMap::new(),
            },
        }
    }

    /// Добавляет ошибку к результатам
    pub fn add_error(&mut self, error: AnalysisError) {
        self.errors.push(error);
    }

    /// Добавляет предупреждение к результатам
    pub fn add_warning(&mut self, warning: AnalysisError) {
        self.warnings.push(warning);
    }

    /// Возвращает список ошибок
    pub fn get_errors(&self) -> &[AnalysisError] {
        &self.errors
    }

    /// Возвращает список предупреждений
    pub fn get_warnings(&self) -> &[AnalysisError] {
        &self.warnings
    }

    /// Возвращает общее количество проблем
    pub fn total_issues(&self) -> usize {
        self.errors.len() + self.warnings.len()
    }

    /// Проверяет, есть ли ошибки
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Проверяет, есть ли предупреждения
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Проверяет, есть ли любые проблемы
    pub fn has_issues(&self) -> bool {
        self.has_errors() || self.has_warnings()
    }

    /// Возвращает количество ошибок
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Возвращает количество предупреждений
    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }

    /// Объединяет результаты с другими результатами
    pub fn merge(&mut self, other: AnalysisResults) {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
        self.metadata.files_analyzed += other.metadata.files_analyzed;

        // Объединяем метрики
        for (key, value) in other.metadata.metrics {
            self.metadata.metrics.insert(key, value);
        }
    }

    /// Фильтрует результаты по файлу
    pub fn filter_by_file(&self, file_path: &std::path::Path) -> AnalysisResults {
        let mut filtered = AnalysisResults::new();

        for error in &self.errors {
            if error.file_path == file_path {
                filtered.add_error(error.clone());
            }
        }

        for warning in &self.warnings {
            if warning.file_path == file_path {
                filtered.add_warning(warning.clone());
            }
        }

        filtered
    }

    /// Сортирует результаты по файлу и позиции
    pub fn sort_by_location(&mut self) {
        self.errors.sort_by(|a, b| {
            a.file_path
                .cmp(&b.file_path)
                .then_with(|| a.position.line.cmp(&b.position.line))
                .then_with(|| a.position.column.cmp(&b.position.column))
        });

        self.warnings.sort_by(|a, b| {
            a.file_path
                .cmp(&b.file_path)
                .then_with(|| a.position.line.cmp(&b.position.line))
                .then_with(|| a.position.column.cmp(&b.position.column))
        });
    }

    /// Устанавливает время окончания анализа
    pub fn set_end_time(&mut self) {
        self.metadata.end_time = Some(std::time::SystemTime::now());
    }

    /// Устанавливает количество проанализированных файлов
    pub fn set_files_analyzed(&mut self, count: usize) {
        self.metadata.files_analyzed = count;
    }

    /// Добавляет метрику
    pub fn add_metric(&mut self, key: String, value: String) {
        self.metadata.metrics.insert(key, value);
    }

    /// Возвращает время анализа
    pub fn analysis_duration(&self) -> Option<std::time::Duration> {
        if let (Some(start), Some(end)) = (self.metadata.start_time, self.metadata.end_time) {
            end.duration_since(start).ok()
        } else {
            None
        }
    }

    /// Возвращает метаданные
    pub fn metadata(&self) -> &AnalysisMetadata {
        &self.metadata
    }
}

impl Default for AnalysisResults {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for AnalysisResults {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Analysis Results:")?;
        writeln!(f, "  Errors: {}", self.error_count())?;
        writeln!(f, "  Warnings: {}", self.warning_count())?;
        writeln!(f, "  Files analyzed: {}", self.metadata.files_analyzed)?;

        if let Some(duration) = self.analysis_duration() {
            writeln!(f, "  Analysis time: {:.2?}", duration)?;
        }

        if !self.errors.is_empty() {
            writeln!(f, "\nErrors:")?;
            for error in &self.errors {
                writeln!(f, "  {}", error)?;
            }
        }

        if !self.warnings.is_empty() {
            writeln!(f, "\nWarnings:")?;
            for warning in &self.warnings {
                writeln!(f, "  {}", warning)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Position;
    use std::path::PathBuf;

    fn create_test_error() -> AnalysisError {
        AnalysisError {
            message: "Test error".to_string(),
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
        }
    }

    fn create_test_warning() -> AnalysisError {
        AnalysisError {
            message: "Test warning".to_string(),
            file_path: PathBuf::from("test.bsl"),
            position: Position {
                line: 15,
                column: 3,
                offset: 150,
            },
            level: crate::core::ErrorLevel::Warning,
            error_code: Some("BSL002".to_string()),
            suggestion: None,
            related_positions: Vec::new(),
        }
    }

    #[test]
    fn test_analysis_results_creation() {
        let results = AnalysisResults::new();
        assert_eq!(results.error_count(), 0);
        assert_eq!(results.warning_count(), 0);
        assert!(!results.has_errors());
        assert!(!results.has_warnings());
    }

    #[test]
    fn test_add_error_and_warning() {
        let mut results = AnalysisResults::new();

        results.add_error(create_test_error());
        results.add_warning(create_test_warning());

        assert_eq!(results.error_count(), 1);
        assert_eq!(results.warning_count(), 1);
        assert!(results.has_errors());
        assert!(results.has_warnings());
        assert!(results.has_issues());
    }

    #[test]
    fn test_merge_results() {
        let mut results1 = AnalysisResults::new();
        results1.add_error(create_test_error());

        let mut results2 = AnalysisResults::new();
        results2.add_warning(create_test_warning());

        results1.merge(results2);

        assert_eq!(results1.error_count(), 1);
        assert_eq!(results1.warning_count(), 1);
    }

    #[test]
    fn test_filter_by_file() {
        let mut results = AnalysisResults::new();

        let mut error1 = create_test_error();
        error1.file_path = PathBuf::from("file1.bsl");

        let mut error2 = create_test_error();
        error2.file_path = PathBuf::from("file2.bsl");

        results.add_error(error1);
        results.add_error(error2);

        let filtered = results.filter_by_file(&PathBuf::from("file1.bsl"));

        assert_eq!(filtered.error_count(), 1);
        assert_eq!(
            filtered.get_errors()[0].file_path,
            PathBuf::from("file1.bsl")
        );
    }

    #[test]
    fn test_sort_by_location() {
        let mut results = AnalysisResults::new();

        let mut error1 = create_test_error();
        error1.position = Position {
            line: 20,
            column: 5,
            offset: 200,
        };

        let mut error2 = create_test_error();
        error2.position = Position {
            line: 10,
            column: 3,
            offset: 100,
        };

        results.add_error(error1);
        results.add_error(error2);

        results.sort_by_location();

        assert_eq!(results.get_errors()[0].position.line, 10);
        assert_eq!(results.get_errors()[1].position.line, 20);
    }

    #[test]
    fn test_metrics() {
        let mut results = AnalysisResults::new();
        results.add_metric("test_metric".to_string(), "test_value".to_string());

        assert_eq!(
            results.metadata().metrics.get("test_metric"),
            Some(&"test_value".to_string())
        );
    }
}
