//! Объединенный BSL анализатор на базе tree-sitter

use anyhow::Result;
use crate::core::errors::{AnalysisError, ErrorCollector};
use crate::parser::ast::AstNode;
use crate::unified_index::UnifiedBslIndex;
use super::{BslParser, SemanticAnalyzer, SemanticAnalysisConfig, DataFlowAnalyzer, Diagnostic};

/// Основной BSL анализатор, объединяющий все виды анализа
pub struct BslAnalyzer {
    parser: BslParser,
    semantic_analyzer: SemanticAnalyzer,
    data_flow_analyzer: DataFlowAnalyzer,
    error_collector: ErrorCollector,
    index: Option<UnifiedBslIndex>,
}

impl BslAnalyzer {
    /// Создает новый экземпляр анализатора
    pub fn new() -> Result<Self> {
        Ok(Self {
            parser: BslParser::new()?,
            semantic_analyzer: SemanticAnalyzer::default(),
            data_flow_analyzer: DataFlowAnalyzer::default(),
            error_collector: ErrorCollector::new(),
            index: None,
        })
    }
    
    /// Создает новый анализатор с UnifiedBslIndex
    pub fn with_index(index: UnifiedBslIndex) -> Result<Self> {
        let mut config = SemanticAnalysisConfig::default();
        config.check_method_calls = true; // Включаем проверку методов
        
        Ok(Self {
            parser: BslParser::new()?,
            semantic_analyzer: SemanticAnalyzer::with_index(config, index.clone()),
            data_flow_analyzer: DataFlowAnalyzer::default(),
            error_collector: ErrorCollector::new(),
            index: Some(index),
        })
    }
    
    /// Устанавливает UnifiedBslIndex для валидации типов и методов
    pub fn set_index(&mut self, index: UnifiedBslIndex) {
        let mut config = SemanticAnalysisConfig::default();
        config.check_method_calls = true;
        
        self.semantic_analyzer = SemanticAnalyzer::with_index(config, index.clone());
        self.index = Some(index);
    }

    /// Выполняет полный анализ BSL кода
    pub fn analyze_code(&mut self, source: &str, file_path: &str) -> Result<()> {
        // 1. Парсинг
        let parse_result = self.parser.parse(source, file_path);
        
        // Собираем диагностики парсера
        for diagnostic in parse_result.diagnostics {
            self.add_diagnostic_as_error(&diagnostic);
        }
        
        // 2. Семантический анализ
        if let Some(ast) = parse_result.ast {
            self.semantic_analyzer.analyze(&ast)?;
            let diagnostics: Vec<_> = self.semantic_analyzer.get_diagnostics().iter().cloned().collect();
            for diagnostic in diagnostics {
                self.add_diagnostic_as_error(&diagnostic);
            }
            
            // 3. Анализ потоков данных
            self.data_flow_analyzer.analyze(&ast)?;
            let diagnostics: Vec<_> = self.data_flow_analyzer.get_diagnostics().iter().cloned().collect();
            for diagnostic in diagnostics {
                self.add_diagnostic_as_error(&diagnostic);
            }
        }
        
        Ok(())
    }

    /// Выполняет анализ AST (для совместимости со старым API)
    pub fn analyze(&mut self, _ast: &AstNode) -> Result<()> {
        // TODO: конвертировать старый AST в новый формат
        // Пока что возвращаем Ok для совместимости
        Ok(())
    }

    /// Получает результаты анализа
    pub fn get_results(&self) -> &ErrorCollector {
        &self.error_collector
    }

    /// Получает результаты в формате (errors, warnings)
    pub fn get_errors_and_warnings(&self) -> (Vec<AnalysisError>, Vec<AnalysisError>) {
        let (semantic_errors, semantic_warnings) = self.semantic_analyzer.get_results();
        let mut all_errors = semantic_errors;
        let all_warnings = semantic_warnings;
        
        // Добавляем ошибки из error_collector
        for error in self.error_collector.get_errors() {
            all_errors.push(error.clone());
        }
        
        (all_errors, all_warnings)
    }

    /// Очищает результаты предыдущего анализа
    pub fn clear(&mut self) {
        self.error_collector.clear();
    }

    /// Устанавливает количество рабочих потоков (заглушка для совместимости)
    pub fn set_worker_count(&mut self, _workers: usize) {
        // TODO: реализовать многопоточность
    }

    /// Включает межмодульный анализ (заглушка для совместимости)
    pub fn set_inter_module_analysis(&mut self, _enabled: bool) {
        // TODO: реализовать межмодульный анализ
    }

    /// Анализирует конфигурацию (заглушка для совместимости)
    pub fn analyze_configuration(&mut self, _config: &crate::configuration::Configuration) -> Result<Vec<crate::core::results::AnalysisResults>> {
        // TODO: реализовать анализ конфигурации
        Ok(vec![])
    }

    /// Преобразует диагностику в ошибку анализа
    fn add_diagnostic_as_error(&mut self, diagnostic: &Diagnostic) {
        let position = crate::parser::ast::Position {
            line: diagnostic.location.line,
            column: diagnostic.location.column,
            offset: diagnostic.location.offset,
        };
        
        let level = match diagnostic.severity {
            super::DiagnosticSeverity::Error => crate::core::errors::ErrorLevel::Error,
            super::DiagnosticSeverity::Warning => crate::core::errors::ErrorLevel::Warning,
            super::DiagnosticSeverity::Info | super::DiagnosticSeverity::Information => crate::core::errors::ErrorLevel::Info,
            super::DiagnosticSeverity::Hint => crate::core::errors::ErrorLevel::Hint,
        };
        
        let error = AnalysisError::new(
            diagnostic.message.clone(),
            diagnostic.location.file.clone().into(),
            position,
            level,
        ).with_code(diagnostic.code.clone());
        
        self.error_collector.add_error(error);
    }
}

impl Default for BslAnalyzer {
    fn default() -> Self {
        Self::new().expect("Failed to create BSL analyzer")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_creation() {
        let analyzer = BslAnalyzer::new();
        assert!(analyzer.is_ok());
    }

    #[test]
    fn test_analyze_empty_code() {
        let mut analyzer = BslAnalyzer::new().unwrap();
        let result = analyzer.analyze_code("", "test.bsl");
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_simple_procedure() {
        let mut analyzer = BslAnalyzer::new().unwrap();
        let code = r#"
            Процедура Тест()
                Сообщить("Привет мир");
            КонецПроцедуры
        "#;
        
        let result = analyzer.analyze_code(code, "test.bsl");
        assert!(result.is_ok());
    }
}