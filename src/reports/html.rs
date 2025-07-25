/*!
# HTML Reporter

Генерация детальных HTML отчетов для веб-просмотра результатов анализа.

## Возможности:
- Интерактивный HTML отчет с навигацией
- Цветовая кодировка проблем по серьезности
- Сортировка и фильтрация результатов
- Встроенный CSS для автономного просмотра
- Статистика анализа и метрики
- Граф зависимостей (если включен)

## Использование:

```rust
use bsl_analyzer::reports::html::HtmlReporter;

let reporter = HtmlReporter::new();
let html_output = reporter.generate_report(&analysis_results, &config)?;

// Сохранение HTML отчета
std::fs::write("analysis-report.html", html_output)?;
```
*/

use anyhow::Result;
use crate::core::AnalysisResults;
use super::{ReportGenerator, ReportConfig, ReportFormat};

/// HTML репортер для создания веб-отчетов
pub struct HtmlReporter {
    /// Включить встроенные стили
    include_inline_css: bool,
    /// Включить JavaScript для интерактивности
    include_javascript: bool,
}

impl HtmlReporter {
    /// Создает новый HTML репортер
    pub fn new() -> Self {
        Self {
            include_inline_css: true,
            include_javascript: true,
        }
    }
    
    /// Создает HTML репортер с конфигурацией
    pub fn with_config(include_css: bool, include_js: bool) -> Self {
        Self {
            include_inline_css: include_css,
            include_javascript: include_js,
        }
    }
    
    /// Генерирует HTML отчет
    fn generate_html_report(&self, results: &AnalysisResults, config: &ReportConfig) -> Result<String> {
        let mut html = String::new();
        
        // HTML заголовок
        html.push_str(&self.generate_html_header());
        
        // CSS стили
        if self.include_inline_css {
            html.push_str(&self.generate_css_styles());
        }
        
        // JavaScript
        if self.include_javascript {
            html.push_str(&self.generate_javascript());
        }
        
        html.push_str("</head>\n<body>\n");
        
        // Заголовок отчета
        html.push_str(&self.generate_report_header(results));
        
        // Сводная статистика
        html.push_str(&self.generate_summary_section(results));
        
        // Результаты анализа
        html.push_str(&self.generate_results_section(results, config));
        
        // Граф зависимостей (если включен)
        if config.include_dependencies {
            html.push_str(&self.generate_dependencies_section(results));
        }
        
        // Метрики производительности (если включены)
        if config.include_performance {
            html.push_str(&self.generate_performance_section(results));
        }
        
        html.push_str("</body>\n</html>");
        
        Ok(html)
    }
    
    /// Генерирует HTML заголовок
    fn generate_html_header(&self) -> String {
        r#"<!DOCTYPE html>
<html lang="ru">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>BSL Analysis Report</title>
"#.to_string()
    }
    
    /// Генерирует CSS стили
    fn generate_css_styles(&self) -> String {
        r#"<style>
body {
    font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
    margin: 0;
    padding: 20px;
    background-color: #f5f5f5;
    color: #333;
}

.container {
    max-width: 1200px;
    margin: 0 auto;
    background: white;
    padding: 30px;
    border-radius: 8px;
    box-shadow: 0 2px 10px rgba(0,0,0,0.1);
}

.header {
    text-align: center;
    margin-bottom: 30px;
    padding-bottom: 20px;
    border-bottom: 2px solid #eee;
}

.header h1 {
    color: #2c3e50;
    margin: 0;
    font-size: 2.5em;
}

.header .subtitle {
    color: #7f8c8d;
    margin-top: 10px;
    font-size: 1.1em;
}

.summary {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 20px;
    margin-bottom: 30px;
}

.summary-card {
    background: #f8f9fa;
    padding: 20px;
    border-radius: 6px;
    text-align: center;
    border-left: 4px solid #3498db;
}

.summary-card.errors {
    border-left-color: #e74c3c;
}

.summary-card.warnings {
    border-left-color: #f39c12;
}

.summary-card.info {
    border-left-color: #3498db;
}

.summary-card h3 {
    margin: 0 0 10px 0;
    color: #2c3e50;
}

.summary-card .number {
    font-size: 2em;
    font-weight: bold;
    color: #2c3e50;
}

.section {
    margin-bottom: 40px;
}

.section h2 {
    color: #2c3e50;
    border-bottom: 2px solid #3498db;
    padding-bottom: 10px;
    margin-bottom: 20px;
}

.results-table {
    width: 100%;
    border-collapse: collapse;
    margin-top: 20px;
}

.results-table th,
.results-table td {
    padding: 12px;
    text-align: left;
    border-bottom: 1px solid #ddd;
}

.results-table th {
    background-color: #3498db;
    color: white;
    font-weight: 600;
}

.results-table tr:hover {
    background-color: #f5f5f5;
}

.severity {
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 0.85em;
    font-weight: 600;
    text-transform: uppercase;
}

.severity.error {
    background-color: #e74c3c;
    color: white;
}

.severity.warning {
    background-color: #f39c12;
    color: white;
}

.severity.info {
    background-color: #3498db;
    color: white;
}

.file-path {
    font-family: 'Courier New', monospace;
    background-color: #f8f9fa;
    padding: 2px 6px;
    border-radius: 3px;
    font-size: 0.9em;
}

.position {
    font-family: 'Courier New', monospace;
    color: #7f8c8d;
    font-size: 0.85em;
}

.message {
    max-width: 400px;
    word-wrap: break-word;
}

.dependencies-graph {
    background: #f8f9fa;
    padding: 20px;
    border-radius: 6px;
    margin-top: 20px;
}

.filter-controls {
    margin-bottom: 20px;
    padding: 15px;
    background: #f8f9fa;
    border-radius: 6px;
}

.filter-controls label {
    margin-right: 15px;
    font-weight: 600;
}

.filter-controls select,
.filter-controls input {
    padding: 5px 10px;
    margin-right: 10px;
    border: 1px solid #ddd;
    border-radius: 4px;
}

.performance-metrics {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 20px;
    margin-top: 20px;
}

.metric-card {
    background: #f8f9fa;
    padding: 15px;
    border-radius: 6px;
    border-left: 4px solid #3498db;
}

.metric-card h4 {
    margin: 0 0 10px 0;
    color: #2c3e50;
}

.metric-value {
    font-size: 1.5em;
    font-weight: bold;
    color: #3498db;
}

.footer {
    margin-top: 40px;
    padding-top: 20px;
    border-top: 1px solid #eee;
    text-align: center;
    color: #7f8c8d;
    font-size: 0.9em;
}
</style>
"#.to_string()
    }
    
    /// Генерирует JavaScript для интерактивности
    fn generate_javascript(&self) -> String {
        r#"<script>
document.addEventListener('DOMContentLoaded', function() {
    // Фильтрация результатов
    const severityFilter = document.getElementById('severity-filter');
    const fileFilter = document.getElementById('file-filter');
    const resultsTable = document.getElementById('results-table');
    
    function filterResults() {
        const severityValue = severityFilter.value;
        const fileValue = fileFilter.value.toLowerCase();
        const rows = resultsTable.querySelectorAll('tbody tr');
        
        rows.forEach(row => {
            const severity = row.querySelector('.severity').textContent;
            const filePath = row.querySelector('.file-path').textContent;
            
            const severityMatch = !severityValue || severity.toLowerCase() === severityValue;
            const fileMatch = !fileValue || filePath.toLowerCase().includes(fileValue);
            
            row.style.display = (severityMatch && fileMatch) ? '' : 'none';
        });
        
        updateVisibleCount();
    }
    
    function updateVisibleCount() {
        const visibleRows = resultsTable.querySelectorAll('tbody tr:not([style*="display: none"])');
        const totalRows = resultsTable.querySelectorAll('tbody tr');
        
        const countDisplay = document.getElementById('results-count');
        if (countDisplay) {
            countDisplay.textContent = `Показано ${visibleRows.length} из ${totalRows.length} результатов`;
        }
    }
    
    if (severityFilter) severityFilter.addEventListener('change', filterResults);
    if (fileFilter) fileFilter.addEventListener('input', filterResults);
    
    // Сортировка таблицы
    const tableHeaders = document.querySelectorAll('.results-table th[data-sortable]');
    tableHeaders.forEach(header => {
        header.style.cursor = 'pointer';
        header.addEventListener('click', () => sortTable(header.dataset.sortable));
    });
    
    let currentSort = { column: null, direction: 'asc' };
    
    function sortTable(column) {
        const tbody = resultsTable.querySelector('tbody');
        const rows = Array.from(tbody.querySelectorAll('tr'));
        
        const direction = (currentSort.column === column && currentSort.direction === 'asc') ? 'desc' : 'asc';
        currentSort = { column, direction };
        
        rows.sort((a, b) => {
            let aValue, bValue;
            
            switch(column) {
                case 'severity':
                    const severityOrder = { 'error': 3, 'warning': 2, 'info': 1 };
                    aValue = severityOrder[a.querySelector('.severity').textContent.toLowerCase()] || 0;
                    bValue = severityOrder[b.querySelector('.severity').textContent.toLowerCase()] || 0;
                    break;
                case 'file':
                    aValue = a.querySelector('.file-path').textContent;
                    bValue = b.querySelector('.file-path').textContent;
                    break;
                case 'line':
                    aValue = parseInt(a.querySelector('.position').textContent.split(':')[0]) || 0;
                    bValue = parseInt(b.querySelector('.position').textContent.split(':')[0]) || 0;
                    break;
                default:
                    aValue = a.cells[getColumnIndex(column)].textContent;
                    bValue = b.cells[getColumnIndex(column)].textContent;
            }
            
            if (direction === 'asc') {
                return aValue > bValue ? 1 : -1;
            } else {
                return aValue < bValue ? 1 : -1;
            }
        });
        
        rows.forEach(row => tbody.appendChild(row));
        
        // Обновляем индикаторы сортировки
        tableHeaders.forEach(h => h.classList.remove('sort-asc', 'sort-desc'));
        header.classList.add(`sort-${direction}`);
    }
    
    function getColumnIndex(column) {
        const headers = Array.from(resultsTable.querySelectorAll('th'));
        return headers.findIndex(h => h.dataset.sortable === column);
    }
    
    // Инициализация счетчика
    updateVisibleCount();
});
</script>
"#.to_string()
    }
    
    /// Генерирует заголовок отчета
    fn generate_report_header(&self, _results: &AnalysisResults) -> String {
        format!(r#"<div class="container">
    <div class="header">
        <h1>🔍 BSL Analysis Report</h1>
        <div class="subtitle">Отчет о статическом анализе BSL кода</div>
        <div class="subtitle">Сгенерирован: {}</div>
    </div>
"#, chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"))
    }
    
    /// Генерирует секцию сводной статистики
    fn generate_summary_section(&self, results: &AnalysisResults) -> String {
        let errors_count = results.get_errors().len();
        let warnings_count = results.get_warnings().len();
        let total_files = self.get_unique_files_count(results);
        
        format!(r#"    <div class="section">
        <h2>📊 Сводная статистика</h2>
        <div class="summary">
            <div class="summary-card errors">
                <h3>Ошибки</h3>
                <div class="number">{}</div>
            </div>
            <div class="summary-card warnings">
                <h3>Предупреждения</h3>
                <div class="number">{}</div>
            </div>
            <div class="summary-card info">
                <h3>Проанализировано файлов</h3>
                <div class="number">{}</div>
            </div>
            <div class="summary-card">
                <h3>Всего проблем</h3>
                <div class="number">{}</div>
            </div>
        </div>
    </div>
"#, errors_count, warnings_count, total_files, errors_count + warnings_count)
    }
    
    /// Генерирует секцию результатов
    fn generate_results_section(&self, results: &AnalysisResults, _config: &ReportConfig) -> String {
        let mut html = String::new();
        
        html.push_str(r#"    <div class="section">
        <h2>🚨 Результаты анализа</h2>
        
        <div class="filter-controls">
            <label for="severity-filter">Фильтр по серьезности:</label>
            <select id="severity-filter">
                <option value="">Все</option>
                <option value="error">Ошибки</option>
                <option value="warning">Предупреждения</option>
                <option value="info">Информация</option>
            </select>
            
            <label for="file-filter">Фильтр по файлу:</label>
            <input type="text" id="file-filter" placeholder="Введите имя файла...">
            
            <span id="results-count" style="margin-left: 20px; color: #7f8c8d;"></span>
        </div>
        
        <table class="results-table" id="results-table">
            <thead>
                <tr>
                    <th data-sortable="severity">Серьезность</th>
                    <th data-sortable="file">Файл</th>
                    <th data-sortable="line">Позиция</th>
                    <th>Код</th>
                    <th>Сообщение</th>
                </tr>
            </thead>
            <tbody>
"#);
        
        // Добавляем ошибки
        for error in results.get_errors() {
            html.push_str(&self.generate_result_row(error, "error"));
        }
        
        // Добавляем предупреждения
        for warning in results.get_warnings() {
            html.push_str(&self.generate_result_row(warning, "warning"));
        }
        
        html.push_str(r#"            </tbody>
        </table>
    </div>
"#);
        
        html
    }
    
    /// Генерирует строку результата
    fn generate_result_row(&self, error: &crate::core::AnalysisError, severity: &str) -> String {
        let error_code = error.error_code.as_deref().unwrap_or("N/A");
        let file_name = error.file_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        
        format!(r#"                <tr>
                    <td><span class="severity {}">{}</span></td>
                    <td><span class="file-path">{}</span></td>
                    <td><span class="position">{}:{}</span></td>
                    <td>{}</td>
                    <td class="message">{}</td>
                </tr>
"#, severity, severity.to_uppercase(), file_name, error.position.line, error.position.column, error_code, html_escape(&error.message))
    }
    
    /// Генерирует секцию зависимостей
    fn generate_dependencies_section(&self, _results: &AnalysisResults) -> String {
        r#"    <div class="section">
        <h2>🔗 Граф зависимостей</h2>
        <div class="dependencies-graph">
            <p>Граф зависимостей будет реализован в следующей версии.</p>
            <p>Эта секция будет содержать интерактивную визуализацию зависимостей между модулями BSL.</p>
        </div>
    </div>
"#.to_string()
    }
    
    /// Генерирует секцию производительности
    fn generate_performance_section(&self, _results: &AnalysisResults) -> String {
        r#"    <div class="section">
        <h2>⚡ Метрики производительности</h2>
        <div class="performance-metrics">
            <div class="metric-card">
                <h4>Время анализа</h4>
                <div class="metric-value">--</div>
            </div>
            <div class="metric-card">
                <h4>Строк кода</h4>
                <div class="metric-value">--</div>
            </div>
            <div class="metric-card">
                <h4>Использование памяти</h4>
                <div class="metric-value">--</div>
            </div>
        </div>
    </div>
"#.to_string()
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

impl ReportGenerator for HtmlReporter {
    fn generate_report(&self, results: &AnalysisResults, config: &ReportConfig) -> Result<String> {
        self.generate_html_report(results, config)
    }
    
    fn supported_format() -> ReportFormat {
        ReportFormat::Html
    }
}

impl Default for HtmlReporter {
    fn default() -> Self {
        Self::new()
    }
}

/// Экранирует HTML спецсимволы
fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
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
        
        results
    }
    
    #[test]
    fn test_html_reporter_creation() {
        let reporter = HtmlReporter::new();
        assert!(reporter.include_inline_css);
        assert!(reporter.include_javascript);
    }
    
    #[test]
    fn test_html_report_generation() {
        let reporter = HtmlReporter::new();
        let results = create_test_results();
        let config = ReportConfig::default();
        
        let html_output = reporter.generate_report(&results, &config).unwrap();
        
        assert!(html_output.contains("<!DOCTYPE html>"));
        assert!(html_output.contains("BSL Analysis Report"));
        assert!(html_output.contains("Тестовая ошибка"));
        assert!(html_output.contains("test.bsl"));
    }
    
    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("test & <script>"), "test &amp; &lt;script&gt;");
        assert_eq!(html_escape("normal text"), "normal text");
    }
    
    #[test]
    fn test_unique_files_count() {
        let reporter = HtmlReporter::new();
        let results = create_test_results();
        
        assert_eq!(reporter.get_unique_files_count(&results), 1);
    }
}