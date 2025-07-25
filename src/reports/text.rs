/*!
# Text Reporter

Генерация консольных текстовых отчетов для BSL анализатора.

## Возможности:
- Цветной вывод в консоль (с поддержкой ANSI)
- Группировка по файлам и серьезности
- Сводная статистика
- Фильтрация по уровню серьезности
- Краткий и детальный формат
- Совместимость с CI/CD (без цветов)

## Использование:

```rust
use bsl_analyzer::reports::text::TextReporter;

let reporter = TextReporter::new();
let text_output = reporter.generate_report(&analysis_results, &config)?;
println!("{}", text_output);
```
*/

use std::collections::HashMap;
use anyhow::Result;
use crate::core::AnalysisResults;
use super::{ReportGenerator, ReportConfig, ReportFormat};

/// Текстовый репортер для консольного вывода
pub struct TextReporter {
    /// Использовать цветной вывод
    use_colors: bool,
    /// Детальный формат
    detailed_format: bool,
    /// Группировать по файлам
    group_by_files: bool,
}

/// Цвета для ANSI вывода
struct Colors;

impl Colors {
    const RESET: &'static str = "\x1b[0m";
    const BOLD: &'static str = "\x1b[1m";
    const RED: &'static str = "\x1b[31m";
    const YELLOW: &'static str = "\x1b[33m";
    const BLUE: &'static str = "\x1b[34m";
    const GREEN: &'static str = "\x1b[32m";
    const CYAN: &'static str = "\x1b[36m";
    const GRAY: &'static str = "\x1b[90m";
}

impl TextReporter {
    /// Создает новый текстовый репортер
    pub fn new() -> Self {
        Self {
            use_colors: Self::supports_colors(),
            detailed_format: true,
            group_by_files: true,
        }
    }
    
    /// Создает репортер с конфигурацией
    pub fn with_config(use_colors: bool, detailed: bool, group_by_files: bool) -> Self {
        Self {
            use_colors,
            detailed_format: detailed,
            group_by_files,
        }
    }
    
    /// Создает краткий репортер для CI/CD
    pub fn brief() -> Self {
        Self {
            use_colors: false,
            detailed_format: false,
            group_by_files: false,
        }
    }
    
    /// Проверяет поддержку цветов в терминале
    fn supports_colors() -> bool {
        // Проверяем переменные окружения
        if std::env::var("NO_COLOR").is_ok() {
            return false;
        }
        
        if std::env::var("FORCE_COLOR").is_ok() {
            return true;
        }
        
        // Проверяем TERM
        if let Ok(term) = std::env::var("TERM") {
            if term == "dumb" || term.is_empty() {
                return false;
            }
        }
        
        // В Windows проверяем поддержку ANSI
        #[cfg(windows)]
        {
            
            unsafe {
                let console_mode = winapi::um::wincon::GetConsoleMode;
                let std_output_handle = winapi::um::winbase::GetStdHandle;
                let enable_virtual_terminal = winapi::um::wincon::ENABLE_VIRTUAL_TERMINAL_PROCESSING;
                
                let handle = std_output_handle(winapi::um::winbase::STD_OUTPUT_HANDLE);
                if handle != winapi::um::handleapi::INVALID_HANDLE_VALUE {
                    let mut mode = 0;
                    if console_mode(handle, &mut mode) != 0 {
                        return (mode & enable_virtual_terminal) != 0;
                    }
                }
            }
            
            false
        }
        
        #[cfg(not(windows))]
        {
            true // На Unix системах обычно поддерживаются цвета
        }
    }
    
    /// Генерирует текстовый отчет
    fn generate_text_report(&self, results: &AnalysisResults, config: &ReportConfig) -> Result<String> {
        let mut output = String::new();
        
        // Заголовок отчета
        output.push_str(&self.generate_header());
        
        // Сводная статистика
        output.push_str(&self.generate_summary(results));
        
        // Результаты анализа
        if self.group_by_files {
            output.push_str(&self.generate_results_by_files(results, config));
        } else {
            output.push_str(&self.generate_results_list(results, config));
        }
        
        // Зависимости (если включены)
        if config.include_dependencies {
            output.push_str(&self.generate_dependencies_summary(results));
        }
        
        // Производительность (если включена)
        if config.include_performance {
            output.push_str(&self.generate_performance_summary(results));
        }
        
        Ok(output)
    }
    
    /// Генерирует заголовок отчета
    fn generate_header(&self) -> String {
        let title = if self.use_colors {
            format!("{}{}🔍 BSL Analysis Report{}\n", Colors::BOLD, Colors::CYAN, Colors::RESET)
        } else {
            "BSL Analysis Report\n".to_string()
        };
        
        let separator = if self.use_colors {
            format!("{}{}{}\n", Colors::GRAY, "=".repeat(50), Colors::RESET)
        } else {
            format!("{}\n", "=".repeat(50))
        };
        
        format!("{}{}\n", title, separator)
    }
    
    /// Генерирует сводную статистику
    fn generate_summary(&self, results: &AnalysisResults) -> String {
        let errors_count = results.get_errors().len();
        let warnings_count = results.get_warnings().len();
        let total_files = self.get_unique_files_count(results);
        
        let mut summary = String::new();
        
        if self.use_colors {
            summary.push_str(&format!("{}Summary:{}\n", Colors::BOLD, Colors::RESET));
            summary.push_str(&format!("  {}Errors:{} {}{}{}\n", 
                Colors::BOLD, Colors::RESET, Colors::RED, errors_count, Colors::RESET));
            summary.push_str(&format!("  {}Warnings:{} {}{}{}\n", 
                Colors::BOLD, Colors::RESET, Colors::YELLOW, warnings_count, Colors::RESET));
            summary.push_str(&format!("  {}Files analyzed:{} {}{}{}\n", 
                Colors::BOLD, Colors::RESET, Colors::BLUE, total_files, Colors::RESET));
        } else {
            summary.push_str("Summary:\n");
            summary.push_str(&format!("  Errors: {}\n", errors_count));
            summary.push_str(&format!("  Warnings: {}\n", warnings_count));
            summary.push_str(&format!("  Files analyzed: {}\n", total_files));
        }
        
        summary.push('\n');
        summary
    }
    
    /// Генерирует результаты, сгруппированные по файлам
    fn generate_results_by_files(&self, results: &AnalysisResults, config: &ReportConfig) -> String {
        let mut output = String::new();
        let mut files_map: HashMap<String, Vec<&crate::core::AnalysisError>> = HashMap::new();
        
        // Группируем ошибки по файлам
        for error in results.get_errors() {
            let file_path = error.file_path.display().to_string();
            files_map.entry(file_path).or_default().push(error);
        }
        
        // Группируем предупреждения по файлам
        for warning in results.get_warnings() {
            let file_path = warning.file_path.display().to_string();
            files_map.entry(file_path).or_default().push(warning);
        }
        
        if files_map.is_empty() {
            output.push_str(&self.colorize("✅ No issues found!\n", Colors::GREEN));
            return output;
        }
        
        output.push_str(&self.colorize("Issues by files:\n", Colors::BOLD));
        output.push('\n');
        
        // Сортируем файлы по количеству проблем
        let mut sorted_files: Vec<_> = files_map.iter().collect();
        sorted_files.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
        
        for (file_path, errors) in sorted_files {
            // Заголовок файла
            let file_name = std::path::Path::new(file_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(file_path);
            
            output.push_str(&self.colorize(&format!("📁 {} ({} issues)\n", file_name, errors.len()), Colors::BOLD));
            
            if self.use_colors {
                output.push_str(&format!("   {}{}{}\n", Colors::GRAY, file_path, Colors::RESET));
            } else {
                output.push_str(&format!("   {}\n", file_path));
            }
            
            // Сортируем ошибки по позиции
            let mut sorted_errors = errors.clone();
            sorted_errors.sort_by(|a, b| a.position.line.cmp(&b.position.line));
            
            for error in sorted_errors {
                output.push_str(&self.format_error_line(error, config, true));
            }
            
            output.push('\n');
        }
        
        output
    }
    
    /// Генерирует список результатов
    fn generate_results_list(&self, results: &AnalysisResults, config: &ReportConfig) -> String {
        let mut output = String::new();
        
        let total_issues = results.get_errors().len() + results.get_warnings().len();
        if total_issues == 0 {
            output.push_str(&self.colorize("✅ No issues found!\n", Colors::GREEN));
            return output;
        }
        
        output.push_str(&self.colorize("Issues found:\n", Colors::BOLD));
        output.push('\n');
        
        // Ошибки
        if !results.get_errors().is_empty() {
            output.push_str(&self.colorize("Errors:\n", Colors::RED));
            for error in results.get_errors() {
                output.push_str(&self.format_error_line(error, config, true));
            }
            output.push('\n');
        }
        
        // Предупреждения
        if !results.get_warnings().is_empty() {
            output.push_str(&self.colorize("Warnings:\n", Colors::YELLOW));
            for warning in results.get_warnings() {
                output.push_str(&self.format_error_line(warning, config, false));
            }
        }
        
        output
    }
    
    /// Форматирует строку ошибки
    fn format_error_line(&self, error: &crate::core::AnalysisError, _config: &ReportConfig, is_error: bool) -> String {
        let mut line = String::new();
        
        // Определяем серьезность по типу
        let (severity_symbol, severity_color) = if is_error {
            ("❌", Colors::RED)
        } else {
            ("⚠️", Colors::YELLOW)
        };
        
        if self.detailed_format {
            // Детальный формат
            let file_name = error.file_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            
            let position = format!("{}:{}", error.position.line, error.position.column);
            let error_code = error.error_code.as_deref().unwrap_or("N/A");
            
            if self.use_colors {
                line.push_str(&format!("  {} {}[{}]{} {}{}:{}{} {}{}{}\n",
                    severity_symbol,
                    severity_color, error_code, Colors::RESET,
                    Colors::CYAN, file_name, Colors::RESET,
                    Colors::GRAY, position, Colors::RESET,
                    error.message
                ));
            } else {
                line.push_str(&format!("  [{}] {}:{} {}\n", 
                    error_code, file_name, position, error.message));
            }
        } else {
            // Краткий формат
            if self.use_colors {
                line.push_str(&format!("  {} {}\n", severity_symbol, error.message));
            } else {
                line.push_str(&format!("  {}\n", error.message));
            }
        }
        
        line
    }
    
    /// Генерирует сводку зависимостей
    fn generate_dependencies_summary(&self, _results: &AnalysisResults) -> String {
        let mut output = String::new();
        output.push_str(&self.colorize("Dependencies Analysis:\n", Colors::BOLD));
        output.push_str("  Dependencies analysis will be available in the next version.\n");
        output.push('\n');
        output
    }
    
    /// Генерирует сводку производительности
    fn generate_performance_summary(&self, _results: &AnalysisResults) -> String {
        let mut output = String::new();
        output.push_str(&self.colorize("Performance Metrics:\n", Colors::BOLD));
        output.push_str("  Performance metrics will be available in the next version.\n");
        output.push('\n');
        output
    }
    
    /// Применяет цвет к тексту если включены цвета
    fn colorize(&self, text: &str, color: &str) -> String {
        if self.use_colors {
            format!("{}{}{}", color, text, Colors::RESET)
        } else {
            text.to_string()
        }
    }
    
    /// Получает количество уникальных файлов
    fn get_unique_files_count(&self, results: &AnalysisResults) -> usize {
        let mut files = std::collections::HashSet::new();
        
        for error in results.get_errors() {
            files.insert(&error.file_path);
        }
        
        for warning in results.get_warnings() {
            files.insert(&warning.file_path);
        }
        
        files.len()
    }
}

impl ReportGenerator for TextReporter {
    fn generate_report(&self, results: &AnalysisResults, config: &ReportConfig) -> Result<String> {
        self.generate_text_report(results, config)
    }
    
    fn supported_format() -> ReportFormat {
        ReportFormat::Text
    }
}

impl Default for TextReporter {
    fn default() -> Self {
        Self::new()
    }
}

// Зависимости для Windows API (условная компиляция)
#[cfg(windows)]
mod winapi {
    pub mod um {
        pub mod wincon {
            pub const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32 = 0x0004;
            
            extern "system" {
                pub fn GetConsoleMode(
                    hConsoleHandle: *mut std::ffi::c_void,
                    lpMode: *mut u32,
                ) -> i32;
            }
        }
        
        pub mod winbase {
            pub const STD_OUTPUT_HANDLE: u32 = 0xFFFFFFF5;
            
            extern "system" {
                pub fn GetStdHandle(nStdHandle: u32) -> *mut std::ffi::c_void;
            }
        }
        
        pub mod handleapi {
            pub const INVALID_HANDLE_VALUE: *mut std::ffi::c_void = -1isize as *mut std::ffi::c_void;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use crate::core::{AnalysisResults, AnalysisError};
    use crate::parser::Position;
    
    fn create_test_results() -> AnalysisResults {
        let mut results = AnalysisResults::new();
        
        results.add_error(AnalysisError {
            message: "Тестовая ошибка".to_string(),
            file_path: PathBuf::from("test.bsl"),
            position: Position { line: 10, column: 5, offset: 100 },
            level: crate::core::ErrorLevel::Error,
            error_code: Some("BSL001".to_string()),
            suggestion: None,
            related_positions: Vec::new(),
        });
        
        results.add_warning(AnalysisError {
            message: "Тестовое предупреждение".to_string(),
            file_path: PathBuf::from("module.bsl"),
            position: Position { line: 25, column: 12, offset: 250 },
            level: crate::core::ErrorLevel::Warning,
            error_code: Some("BSL004".to_string()),
            suggestion: None,
            related_positions: Vec::new(),
        });
        
        results
    }
    
    #[test]
    fn test_text_reporter_creation() {
        let reporter = TextReporter::new();
        assert!(reporter.detailed_format);
        assert!(reporter.group_by_files);
    }
    
    #[test]
    fn test_brief_reporter() {
        let reporter = TextReporter::brief();
        assert!(!reporter.use_colors);
        assert!(!reporter.detailed_format);
        assert!(!reporter.group_by_files);
    }
    
    #[test]
    fn test_text_report_generation() {
        let reporter = TextReporter::with_config(false, true, false); // Без цветов для стабильного теста
        let results = create_test_results();
        let config = ReportConfig::default();
        
        let text_output = reporter.generate_report(&results, &config).unwrap();
        
        assert!(text_output.contains("BSL Analysis Report"));
        assert!(text_output.contains("Тестовая ошибка"));
        assert!(text_output.contains("Тестовое предупреждение"));
        assert!(text_output.contains("Errors: 1"));
        assert!(text_output.contains("Warnings: 1"));
    }
    
    #[test]
    fn test_colorize() {
        let reporter_with_colors = TextReporter::with_config(true, true, false);
        let reporter_no_colors = TextReporter::with_config(false, true, false);
        
        let colored_text = reporter_with_colors.colorize("test", Colors::RED);
        let plain_text = reporter_no_colors.colorize("test", Colors::RED);
        
        assert!(colored_text.contains("\x1b[31m")); // ANSI красный цвет
        assert_eq!(plain_text, "test");
    }
    
    #[test]
    fn test_unique_files_count() {
        let reporter = TextReporter::new();
        let results = create_test_results();
        
        assert_eq!(reporter.get_unique_files_count(&results), 2); // test.bsl и module.bsl
    }
}