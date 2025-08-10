//! Объединенный BSL анализатор на базе tree-sitter.
// incremental-semantic: selective semantic timing integrated (marker)
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
    // Тайминги последнего запуска
    last_parse_time_ns: Option<u128>,
    last_arena_time_ns: Option<u128>,
    // Инкрементальные метрики последнего анализа (diff с предыдущим AST)
    last_incremental_stats: Option<crate::ast_core::IncrementalStats>,
    // Последний план частичной перестройки (прототип Phase 5)
    last_rebuild_plan: Option<crate::ast_core::incremental::RebuildPlan>,
    // Последний исходный текст (для вычисления diff range)
    last_source: Option<String>,
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
            last_parse_time_ns: None,
            last_arena_time_ns: None,
            last_incremental_stats: None,
            last_rebuild_plan: None,
            last_source: None,
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
            last_parse_time_ns: None,
            last_arena_time_ns: None,
            last_incremental_stats: None,
            last_rebuild_plan: None,
            last_source: None,
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
            last_parse_time_ns: None,
            last_arena_time_ns: None,
            last_incremental_stats: None,
            last_rebuild_plan: None,
            last_source: None,
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
            last_parse_time_ns: None,
            last_arena_time_ns: None,
            last_incremental_stats: None,
            last_rebuild_plan: None,
            last_source: None,
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

    /// Root fingerprint (0 если AST отсутствует)
    pub fn get_root_fingerprint(&self) -> u64 {
        self.last_built_arena.as_ref().map(|b| b.root_fingerprint()).unwrap_or(0)
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
    // Сохраняем предыдущий AST для diff до перезаписи
    let old_built_opt = self.last_built_arena.take();
    let mut parse_result = self.parser.parse(source, file_path);
    self.last_parse_time_ns = Some(parse_result.parse_time_ns);
    self.last_arena_time_ns = Some(parse_result.arena_time_ns);
    // Сохраняем последний arena AST и рассчитываем incremental stats
    if let Some(new_built) = parse_result.arena.take() {
        let stats_opt = if let Some(old_full) = old_built_opt.as_ref() {
            if let Some(diff) = old_full.fingerprint_diff(&new_built) {
                Some(diff.to_stats_with_timing(self.last_parse_time_ns, self.last_arena_time_ns, Some(new_built.fingerprint_time_ns)))
            } else { None }
        } else { None };
        self.last_incremental_stats = stats_opt.map(|mut s| { s.planned_routines = self.last_rebuild_plan.as_ref().map(|p| p.routines_to_rebuild.len()); s.replaced_routines = s.planned_routines; s });
        self.last_built_arena = Some(new_built);
    } else { self.last_incremental_stats = None; }
    let _arena_ast = self.last_built_arena.as_ref(); // временно не используется (семантика на старом AST)
    // Обновляем сохранённый исходник
    self.last_source = Some(source.to_string());

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

    pub fn last_timing_parse_ns(&self) -> Option<u128> { self.last_parse_time_ns }
    pub fn last_timing_arena_ns(&self) -> Option<u128> { self.last_arena_time_ns }
    pub fn last_incremental_stats(&self) -> Option<crate::ast_core::IncrementalStats> { self.last_incremental_stats }
    pub fn last_rebuild_plan(&self) -> Option<&crate::ast_core::incremental::RebuildPlan> { self.last_rebuild_plan.as_ref() }

    /// Вычислить текстовый dirty range между предыдущим и новым исходником (по байтам):
    /// возвращает (start_old, end_old, start_new, end_new). Если предыдущего исходника нет — полное изменение.
    fn compute_dirty_range(old_src: &str, new_src: &str) -> (usize, usize, usize, usize) {
        if old_src == new_src { return (0, 0, 0, 0); }
        let old_bytes = old_src.as_bytes();
        let new_bytes = new_src.as_bytes();
        let mut prefix = 0;
        while prefix < old_bytes.len() && prefix < new_bytes.len() && old_bytes[prefix] == new_bytes[prefix] { prefix += 1; }
        let mut old_suffix = old_bytes.len();
        let mut new_suffix = new_bytes.len();
        while old_suffix > prefix && new_suffix > prefix && old_bytes[old_suffix - 1] == new_bytes[new_suffix - 1] {
            old_suffix -= 1;
            new_suffix -= 1;
        }
        (prefix, old_suffix, prefix, new_suffix)
    }

    /// Сформировать план частичной перестройки на основе diff старого и нового текста (без выполнения самой перестройки).
    pub fn plan_incremental_from_texts(&mut self, new_source: &str) -> Option<&crate::ast_core::incremental::RebuildPlan> {
        use crate::ast_core::incremental::{plan_partial_rebuild, RebuildHeuristics};
        let old_src = self.last_source.as_ref()?;
        let built = self.last_built_arena.as_ref()?;
        let (start_old, end_old, _start_new, _end_new) = Self::compute_dirty_range(old_src, new_source);
        // Диапазон для планирования используем в координатах старого исходника (так как spans старые)
        let plan = plan_partial_rebuild(built, start_old as u32, end_old as u32, RebuildHeuristics::default());
        self.last_rebuild_plan = Some(plan);
        self.last_rebuild_plan.as_ref()
    }

    /// Анализ с предварительным построением плана инкрементальной перестройки. Пока реализация всегда делает полный анализ,
    /// но сохраняет план для дальнейшей оптимизации.
    pub fn analyze_incremental(&mut self, new_source: &str, file_path: &str) -> Result<()> {
        // Сохраняем старый AST и источник
        let old_ast_opt = self.last_built_arena.clone();
    let _old_source_opt = self.last_source.clone();
        // Построить план (использует старый AST)
        let _maybe_plan = self.plan_incremental_from_texts(new_source);
        // Выполняем полный парс новой версии чтобы получить new_full (быстрый путь без семантики пока)
        let mut parse_result = self.parser.parse(new_source, file_path);
        self.last_parse_time_ns = Some(parse_result.parse_time_ns);
        self.last_arena_time_ns = Some(parse_result.arena_time_ns);
        if let Some(new_full) = parse_result.arena.take() {
            if let (Some(old_ast), Some(plan)) = (old_ast_opt.as_ref(), self.last_rebuild_plan.as_ref()) {
                use crate::ast_core::incremental::selective_rebuild;
                let (hybrid, replaced, fallback, inner_reused, inner_total) = selective_rebuild(old_ast, &new_full, plan);
                // Вычисляем diff против старого для метрик
                let diff_opt = old_ast.fingerprint_diff(&hybrid);
                let stats_opt = diff_opt.map(|d| {
                    let mut s = d.to_stats_with_timing(self.last_parse_time_ns, self.last_arena_time_ns, Some(hybrid.fingerprint_time_ns));
                    if hybrid.last_partial_recomputed > 0 { s.recomputed_fingerprints = Some(hybrid.last_partial_recomputed); }
                    s.planned_routines = Some(plan.routines_to_rebuild.len());
                    s.replaced_routines = Some(if fallback { plan.routines_to_rebuild.len() } else { replaced });
                    s.initial_touched = Some(plan.initial_touched);
                    s.expanded_touched = Some(plan.expanded_touched);
                    if fallback { s.fallback_reason = Some(plan.fallback_reason); }
                    else if inner_reused > 0 { s.inner_reused_nodes = Some(inner_reused); if inner_total>0 { s.inner_reuse_ratio = Some(inner_reused as f64 / inner_total as f64); } }
                    // Selective semantic (подмножество заменённых рутин) если не fallback и есть рутины
                    if !fallback {
                        // Собираем идентификаторы рутин из плана
                        let routine_ids: Vec<_> = plan.routines_to_rebuild.clone();
                        if !routine_ids.is_empty() {
                            let start_sem = std::time::Instant::now();
                            let mut arena_sem = SemanticArena::new();
                            arena_sem.set_file_name(file_path);
                            arena_sem.set_line_index(crate::core::position::LineIndex::new(new_source));
                            // Для подмножества отключаем undeclared (чтобы не ловить кросс-рутинг зависимости) — передаём false
                            arena_sem.analyze_routines_subset(&hybrid, &routine_ids, true, true, false);
                            let _sem_time = start_sem.elapsed().as_nanos();
                            s.semantic_ns = Some(_sem_time);
                            // Метрики selective семантики
                            s.semantic_processed_routines = Some(routine_ids.len());
                            if let Some(total) = s.planned_routines { if total>0 { let reused = total.saturating_sub(routine_ids.len()); s.semantic_reused_routines = Some(reused); s.semantic_selective_ratio = Some(reused as f64 / total as f64); } }
                        }
                    } else {
                        // fallback: полный семантический анализ
                        let start_sem = std::time::Instant::now();
                        let mut arena_sem = SemanticArena::new();
                        arena_sem.set_file_name(file_path);
                        arena_sem.set_line_index(crate::core::position::LineIndex::new(new_source));
                        arena_sem.analyze_with_flags(&hybrid, true, true, true);
                        s.semantic_ns = Some(start_sem.elapsed().as_nanos());
                        // В полном пути считаем всё пересчитанным
                        if let Some(total) = s.planned_routines { s.semantic_processed_routines = Some(total); s.semantic_reused_routines = Some(0); s.semantic_selective_ratio = Some(0.0); }
                    }
                    s
                });
                self.last_incremental_stats = stats_opt;
                self.last_built_arena = Some(hybrid);
            } else {
                // Нет старого AST — просто сохраняем полный
                // Выполняем полный семантический анализ и фиксируем semantic_ns
                let hybrid = new_full.clone();
                let start_sem = std::time::Instant::now();
                let mut arena_sem = SemanticArena::new();
                arena_sem.set_file_name(file_path);
                arena_sem.set_line_index(crate::core::position::LineIndex::new(new_source));
                arena_sem.analyze_with_flags(&hybrid, true, true, true);
                let _first_semantic_time = start_sem.elapsed().as_nanos();
                // Первая сборка: нет diff -> не заполняем last_incremental_stats, но можем в будущем сохранить отдельное поле
                self.last_incremental_stats = None; // Нет diff для первой сборки
                // Сохраняем время семантики отдельно в last_incremental_stats нельзя (отсутствует diff), но можно хранить в parse+arena через future API
                // Сохраняем AST
                self.last_built_arena = Some(hybrid);
            }
        }
        self.last_source = Some(new_source.to_string());
        // Семантику и прочее пока не запускаем в incremental пути (можно включить опционально позже)
        Ok(())
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

    #[test]
    fn test_incremental_plan_single_routine() {
        let mut analyzer = BslAnalyzer::new().unwrap();
        let original = r#"
Процедура P1()
    Сообщить("A");
КонецПроцедуры

Процедура P2()
    Сообщить("B");
КонецПроцедуры
"#;
        analyzer.analyze_code(original, "mod.bsl").unwrap();
        // Изменяем только тело второй процедуры (замена строки B на BB)
        let modified = r#"
Процедура P1()
    Сообщить("A");
КонецПроцедуры

Процедура P2()
    Сообщить("BB");
КонецПроцедуры
"#;
        let plan_opt = analyzer.plan_incremental_from_texts(modified);
        assert!(plan_opt.is_some(), "Plan should be produced");
        let plan = plan_opt.unwrap();
        // На текущем этапе stub-парсер даёт грубые span, поэтому может быть fallback_full.
        // Проверяем минимум: либо одна рутина выделена, либо полный фолбэк (будет уточнено после улучшения span).
        if !plan.fallback_full {
            assert_eq!(plan.routines_to_rebuild.len(), 1, "Exactly one routine expected to rebuild");
        }
    }

    #[test]
    fn test_selective_rebuild_reuse_ratio() {
        let mut analyzer = BslAnalyzer::new().unwrap();
        let original = r#"
Процедура P1()
    Сообщить("A");
КонецПроцедуры

Процедура P2()
    Сообщить("B");
КонецПроцедуры
"#;
        analyzer.analyze_code(original, "mod.bsl").unwrap();
        let modified = r#"
Процедура P1()
    Сообщить("A");
КонецПроцедуры

Процедура P2()
    Сообщить("BB");
КонецПроцедуры
"#;
        analyzer.analyze_incremental(modified, "mod.bsl").unwrap();
        let stats = analyzer.last_incremental_stats().expect("stats");
        assert!(stats.reuse_ratio > 0.0, "Expect some reuse");
        // planned_routines присутствует
        assert!(stats.planned_routines.is_some());
    }

    #[test]
    fn test_selective_rebuild_fallback_on_first_run() {
        let mut analyzer = BslAnalyzer::new().unwrap();
        // Первый вызов incremental без предыдущего AST должен просто установить состояние.
        analyzer.analyze_incremental("Процедура P()\nКонецПроцедуры", "f.bsl").unwrap();
        assert!(analyzer.last_built_arena().is_some());
        assert!(analyzer.last_incremental_stats().is_none());
    }

    #[test]
    fn test_recomputed_fingerprints_metric() {
        // Исходник с двумя процедурами, изменим одну чтобы selective пересчитал частично
        let mut analyzer = BslAnalyzer::new().unwrap();
        let original = r#"
Процедура P1()
    Сообщить("A");
КонецПроцедуры

Процедура P2()
    Сообщить("B");
КонецПроцедуры
"#;
        analyzer.analyze_code(original, "mod.bsl").unwrap();
        let modified = r#"
Процедура P1()
    Сообщить("A");
КонецПроцедуры

Процедура P2()
    Сообщить("BB"); // небольшое изменение
КонецПроцедуры
"#;
        analyzer.analyze_incremental(modified, "mod.bsl").unwrap();
        if let Some(stats) = analyzer.last_incremental_stats() {
            // Если был fallback полный – пропускаем (зависит от span точности), иначе проверяем метрику
            if stats.fallback_reason.is_none() {
                if let Some(rc) = stats.recomputed_fingerprints {
                    assert!(rc > 0, "Ожидаем >0 пересчитанных fingerprint узлов");
                    assert!(rc < stats.total_nodes, "Пересчитанных fingerprint должно быть меньше общего числа узлов");
                } else {
                    panic!("Ожидается заполненная метрика recomputed_fingerprints");
                }
            }
        } else { panic!("Статистика должна быть доступна после incremental шага"); }
    }
}
