//! Объединенный BSL анализатор на базе tree-sitter.
//!
//! NOTE: Legacy семантический путь (структуры старого AST) объявлен устаревшим и
//! будет удалён после стабилизации Phase 3 (arena-based semantics + precise spans + snapshot tests).
//! Новый движок: `semantic_arena::SemanticArena` (NodeId/Arena). Все новые правила и улучшения
//! должны добавляться только туда. В этом модуле сохранены только внешние интерфейсы
//! для обратной совместимости API.

use super::{BslParser, DataFlowAnalyzer, Diagnostic, SemanticAnalysisConfig, SemanticAnalyzer};
use super::semantic_arena::SemanticArena; // new arena-based semantic (experimental)
use crate::core::errors::{AnalysisError, ErrorCollector};
// Legacy AstNode no longer used; keep method signature for backward compatibility behind empty type.
// Remove direct import of legacy AST.
use crate::unified_index::UnifiedBslIndex;
use anyhow::Result;

/// Уровни анализа BSL кода
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalysisLevel {
    /// Только синтаксический анализ
    Syntax,
    /// Синтаксис + семантический анализ
    Semantic,
    /// Синтаксис + семантика + анализ потока данных
    DataFlow,
    /// Полный анализ (все проверки)
    Full,
}

impl AnalysisLevel {
    /// Проверяет, включает ли уровень синтаксический анализ
    pub fn includes_syntax(&self) -> bool {
        true // Все уровни включают синтаксис
    }

    /// Проверяет, включает ли уровень семантический анализ
    pub fn includes_semantic(&self) -> bool {
        matches!(
            self,
            AnalysisLevel::Semantic | AnalysisLevel::DataFlow | AnalysisLevel::Full
        )
    }

    /// Проверяет, включает ли уровень анализ потока данных
    pub fn includes_data_flow(&self) -> bool {
        matches!(self, AnalysisLevel::DataFlow | AnalysisLevel::Full)
    }
}

/// Конфигурация анализа BSL кода
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    /// Уровень анализа
    pub level: AnalysisLevel,
    /// Проверять вызовы методов
    pub check_method_calls: bool,
    /// Проверять совместимость типов
    pub check_type_compatibility: bool,
    /// Проверять неиспользуемые переменные
    pub check_unused_variables: bool,
    /// Проверять неинициализированные переменные
    pub check_uninitialized: bool,
    /// Максимальное количество ошибок (0 = без ограничений)
    pub max_errors: usize,
}

impl AnalysisConfig {
    /// Создает конфигурацию только для синтаксической проверки
    pub fn syntax_only() -> Self {
        Self {
            level: AnalysisLevel::Syntax,
            check_method_calls: false,
            check_type_compatibility: false,
            check_unused_variables: false,
            check_uninitialized: false,
            max_errors: 0,
        }
    }

    /// Создает конфигурацию для семантического анализа
    pub fn semantic() -> Self {
        Self {
            level: AnalysisLevel::Semantic,
            check_method_calls: true,
            check_type_compatibility: true,
            check_unused_variables: false,
            check_uninitialized: false,
            max_errors: 0,
        }
    }

    /// Создает конфигурацию для анализа потока данных
    pub fn data_flow() -> Self {
        Self {
            level: AnalysisLevel::DataFlow,
            check_method_calls: true,
            check_type_compatibility: true,
            check_unused_variables: true,
            check_uninitialized: true,
            max_errors: 0,
        }
    }

    /// Создает конфигурацию для полного анализа
    pub fn full() -> Self {
        Self {
            level: AnalysisLevel::Full,
            check_method_calls: true,
            check_type_compatibility: true,
            check_unused_variables: true,
            check_uninitialized: true,
            max_errors: 0,
        }
    }
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self::full()
    }
}

/// Основной BSL анализатор, объединяющий все виды анализа
pub struct BslAnalyzer {
    parser: BslParser,
    semantic_analyzer: SemanticAnalyzer,
    data_flow_analyzer: DataFlowAnalyzer,
    error_collector: ErrorCollector,
    index: Option<UnifiedBslIndex>,
    config: AnalysisConfig,
    // Последний успешно построенный arena-AST (для метрик интернера и потенциально быстрых повторных запросов)
    last_built_arena: Option<crate::ast_core::BuiltAst>,
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
            config: AnalysisConfig::default(),
            last_built_arena: None,
        })
    }

    /// Создает новый анализатор с конфигурацией
    pub fn with_config(config: AnalysisConfig) -> Result<Self> {
        Ok(Self {
            parser: BslParser::new()?,
            semantic_analyzer: SemanticAnalyzer::default(),
            data_flow_analyzer: DataFlowAnalyzer::default(),
            error_collector: ErrorCollector::new(),
            index: None,
            config,
            last_built_arena: None,
        })
    }

    /// Создает новый анализатор с UnifiedBslIndex
    pub fn with_index(index: UnifiedBslIndex) -> Result<Self> {
        let sem_config = SemanticAnalysisConfig {
            check_method_calls: true,
            ..Default::default()
        }; // Включаем проверку методов

        Ok(Self {
            parser: BslParser::new()?,
            semantic_analyzer: SemanticAnalyzer::with_index(sem_config, index.clone()),
            data_flow_analyzer: DataFlowAnalyzer::default(),
            error_collector: ErrorCollector::new(),
            index: Some(index),
            config: AnalysisConfig::default(),
            last_built_arena: None,
        })
    }

    /// Создает новый анализатор с UnifiedBslIndex и конфигурацией
    pub fn with_index_and_config(index: UnifiedBslIndex, config: AnalysisConfig) -> Result<Self> {
        let sem_config = SemanticAnalysisConfig {
            check_method_calls: config.check_method_calls,
            ..Default::default()
        };

        Ok(Self {
            parser: BslParser::new()?,
            semantic_analyzer: SemanticAnalyzer::with_index(sem_config, index.clone()),
            data_flow_analyzer: DataFlowAnalyzer::default(),
            error_collector: ErrorCollector::new(),
            index: Some(index),
            config,
            last_built_arena: None,
        })
    }

    /// Устанавливает UnifiedBslIndex для валидации типов и методов
    pub fn set_index(&mut self, index: UnifiedBslIndex) {
        let config = SemanticAnalysisConfig {
            check_method_calls: true,
            ..Default::default()
        };

        self.semantic_analyzer = SemanticAnalyzer::with_index(config, index.clone());
        self.index = Some(index);
    }

    /// Текущая конфигурация (для вспомогательных вызовов вне основного API)
    pub fn get_config(&self) -> &AnalysisConfig { &self.config }

    /// Доступ к последнему построенному arena AST (для метрик / отладки)
    pub fn last_built_arena(&self) -> Option<&crate::ast_core::BuiltAst> { self.last_built_arena.as_ref() }

    /// Метрики интернера (символы, байты). Возвращает (0,0) если недоступно.
    pub fn get_interner_metrics(&self) -> (usize, usize) {
        if let Some(built) = &self.last_built_arena {
            return (built.interner_symbol_count(), built.interner_total_bytes());
        }
        (0, 0)
    }

    /// Выполняет анализ BSL файла с конфигурацией
    pub fn analyze_file(&mut self, path: &std::path::Path, config: &AnalysisConfig) -> Result<()> {
        let source = std::fs::read_to_string(path)?;
        self.analyze_text(&source, config)
    }

    /// Выполняет анализ BSL текста с конфигурацией
    pub fn analyze_text(&mut self, text: &str, config: &AnalysisConfig) -> Result<()> {
        self.config = config.clone();
        self.analyze_code(text, "<text>")
    }

    /// Выполняет анализ BSL модуля с конфигурацией
    pub fn analyze_module(
        &mut self,
        module_path: &std::path::Path,
        config: &AnalysisConfig,
    ) -> Result<()> {
        self.config = config.clone();
        self.analyze_file(module_path, config)
    }

    /// Выполняет полный анализ BSL кода
    pub fn analyze_code(&mut self, source: &str, file_path: &str) -> Result<()> {
        // Проверяем лимит ошибок
        if self.config.max_errors > 0
            && self.error_collector.get_errors().len() >= self.config.max_errors
        {
            return Ok(());
        }

        // 1. Парсинг (всегда выполняется)
    let mut parse_result = self.parser.parse(source, file_path);
    // Сохраняем последний arena AST для метрик (перемещаем out of parse_result)
    if let Some(built) = parse_result.arena.take() { self.last_built_arena = Some(built); }
    let _arena_ast = self.last_built_arena.as_ref(); // временно не используется (семантика на старом AST)

        // Собираем диагностики парсера
        for diagnostic in parse_result.diagnostics {
            self.add_diagnostic_as_error(&diagnostic);
            if self.config.max_errors > 0
                && self.error_collector.get_errors().len() >= self.config.max_errors
            {
                return Ok(());
            }
        }

        // 2. Семантический анализ (если включен)
    if self.config.level.includes_semantic() {
            if let Some(ast) = parse_result.ast {
                self.semantic_analyzer.analyze(&ast)?;
                let diagnostics = self.semantic_analyzer.get_diagnostics().to_vec();
                for diagnostic in diagnostics {
                    self.add_diagnostic_as_error(&diagnostic);
                    if self.config.max_errors > 0
                        && self.error_collector.get_errors().len() >= self.config.max_errors
                    {
                        return Ok(());
                    }
                }

                // 3. Анализ потоков данных (если включен)
                if self.config.level.includes_data_flow() {
                    self.data_flow_analyzer.analyze(&ast)?;
                    let diagnostics = self.data_flow_analyzer.get_diagnostics().to_vec();
                    for diagnostic in diagnostics {
                        self.add_diagnostic_as_error(&diagnostic);
                        if self.config.max_errors > 0
                            && self.error_collector.get_errors().len() >= self.config.max_errors
                        {
                            return Ok(());
                        }
                    }
                }
            }
            // Experimental arena-based semantic (currently only unused vars) when enabled
            if let Some(built) = &self.last_built_arena {
                // Arena semantic supports: unused, uninitialized, undeclared
                if self.config.check_unused_variables || self.config.check_uninitialized {
                    let mut arena_sem = SemanticArena::new();
                    // Передаем корректное имя файла и line index для точных позиций
                    arena_sem.set_file_name(file_path);
                    arena_sem.set_line_index(crate::core::position::LineIndex::new(source));
                    arena_sem.analyze_with_flags(
                        built,
                        self.config.check_unused_variables,
                        self.config.check_uninitialized,
                        true, // всегда сообщаем об необъявленных пока
                    );
                    for d in arena_sem.diagnostics() {
                        self.add_diagnostic_as_error(d);
                        if self.config.max_errors > 0 && self.error_collector.get_errors().len() >= self.config.max_errors { return Ok(()); }
                    }
                }
            }
        }

        Ok(())
    }

    /// Выполняет анализ AST (для совместимости со старым API)
    pub fn analyze(&mut self, _ast: &()) -> Result<()> {
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
    pub fn analyze_configuration(
        &mut self,
        _config: &crate::configuration::Configuration,
    ) -> Result<Vec<crate::core::results::AnalysisResults>> {
        // TODO: реализовать анализ конфигурации
        Ok(vec![])
    }

    /// Преобразует диагностику в ошибку анализа
    fn add_diagnostic_as_error(&mut self, diagnostic: &Diagnostic) {
    let position = crate::core::position::Position {
            line: diagnostic.location.line,
            column: diagnostic.location.column,
            offset: diagnostic.location.offset,
        };

        let level = match diagnostic.severity {
            super::DiagnosticSeverity::Error => crate::core::errors::ErrorLevel::Error,
            super::DiagnosticSeverity::Warning => crate::core::errors::ErrorLevel::Warning,
            super::DiagnosticSeverity::Info | super::DiagnosticSeverity::Information => {
                crate::core::errors::ErrorLevel::Info
            }
            super::DiagnosticSeverity::Hint => crate::core::errors::ErrorLevel::Hint,
        };

        let error = AnalysisError::new(
            diagnostic.message.clone(),
            diagnostic.location.file.clone().into(),
            position,
            level,
        )
        .with_code(diagnostic.code.clone());

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
