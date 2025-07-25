/*!
# Analysis Engine

Core analysis engine that coordinates different analysis phases.
*/

use anyhow::Result;
use crate::diagnostics::Diagnostic;
use super::{AnalysisContext, DataFlowAnalyzer, ValidationManager, AnalysisResult};
use crate::parser::syntax_analyzer::SyntaxAnalyzer;
use crate::verifiers::MethodVerifier;
use crate::verifiers::method_verifier::ArgumentInfo;
use super::semantic::{TypeSystem, SemanticAnalyzer, SemanticAnalysisConfig};
use rayon::prelude::*;

pub struct AnalysisEngine {
    /// Собранные диагностики
    diagnostics: Vec<Diagnostic>,
    /// Менеджер валидации
    validation_manager: ValidationManager,
    /// Анализатор синтаксиса
    syntax_analyzer: SyntaxAnalyzer,
    /// Анализатор потоков данных
    data_flow_analyzer: DataFlowAnalyzer,
    /// Верификатор методов
    method_verifier: MethodVerifier,
    /// Семантический анализатор
    semantic_analyzer: SemanticAnalyzer,
    /// Количество рабочих потоков для параллельной обработки
    worker_count: usize,
}

impl AnalysisEngine {
    /// Создает новый движок анализа
    pub fn new() -> Self {
        let type_system = TypeSystem::new();
        let semantic_config = SemanticAnalysisConfig::default();
        
        Self {
            diagnostics: Vec::new(),
            validation_manager: ValidationManager::new(),
            syntax_analyzer: SyntaxAnalyzer::new(),
            data_flow_analyzer: DataFlowAnalyzer::new(),
            method_verifier: MethodVerifier::new(type_system.clone()),
            semantic_analyzer: SemanticAnalyzer::new(semantic_config),
            worker_count: num_cpus::get(), // Устанавливаем по количеству CPU
        }
    }
    
    /// Выполняет полный анализ BSL кода
    pub fn analyze_code(&mut self, code: &str, file_path: &str, config: &serde_json::Value) -> Result<()> {
        // Очищаем предыдущие результаты
        self.diagnostics.clear();
        
        // 1. Валидация входных данных
        let validation_result = self.validation_manager.validate_analysis_input(code, file_path, config);
        if !validation_result.is_valid {
            let validation_diagnostics = self.validation_manager.to_diagnostics(&validation_result, file_path);
            self.diagnostics.extend(validation_diagnostics);
            return Ok(()); // Прекращаем анализ при критических ошибках валидации
        }
        
        // Добавляем предупреждения валидации
        if validation_result.has_warnings() {
            let validation_diagnostics = self.validation_manager.to_diagnostics(&validation_result, file_path);
            self.diagnostics.extend(validation_diagnostics);
        }
        
        // 2. Создаем контекст анализа
        let mut context = AnalysisContext::new(file_path.to_string(), code.to_string());
        
        // 3. Синтаксический анализ
        if let Ok(ast) = self.syntax_analyzer.parse(code) {
            context.ast = Some(ast);
        } else {
            // Добавляем ошибку парсинга
            self.diagnostics.push(Diagnostic::error(
                "Ошибка синтаксического анализа".to_string(),
                0,
                0
            ));
            return Ok(());
        }
        
        // 4. Анализ потоков данных
        self.data_flow_analyzer.analyze(&mut context)?;
        
        // 5. Семантический анализ
        if let Some(ast) = context.ast.clone() {
            if let Err(e) = self.semantic_analyzer.analyze(&ast) {
                self.diagnostics.push(Diagnostic::error(
                    format!("Ошибка семантического анализа: {}", e),
                    0,
                    0
                ));
            }
            
            // Собираем результаты семантического анализа
            let (semantic_errors, semantic_warnings) = self.semantic_analyzer.get_results();
            
            // Преобразуем ошибки семантического анализа в диагностики
            for error in semantic_errors {
                self.diagnostics.push(Diagnostic::error(
                    error.message.clone(),
                    error.position.line,
                    error.position.column
                ));
            }
            
            for warning in semantic_warnings {
                self.diagnostics.push(Diagnostic::warning(
                    warning.message.clone(),
                    warning.position.line,
                    warning.position.column
                ));
            }
        }
        
        // 6. Верификация методов
        if let Some(ast) = context.ast.clone() {
            self.verify_method_calls(&ast, &mut context);
        }
        
        // 7. Собираем все диагностики
        self.diagnostics.extend(context.diagnostics);
        
        Ok(())
    }
    
    /// Возвращает собранные диагностики
    pub fn get_diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
    
    /// Возвращает количество ошибок
    pub fn get_error_count(&self) -> usize {
        self.diagnostics.iter()
            .filter(|d| matches!(d.level, crate::diagnostics::DiagnosticLevel::Error))
            .count()
    }
    
    /// Возвращает количество предупреждений
    pub fn get_warning_count(&self) -> usize {
        self.diagnostics.iter()
            .filter(|d| matches!(d.level, crate::diagnostics::DiagnosticLevel::Warning))
            .count()
    }
    
    /// Проверяет наличие критических ошибок
    pub fn has_critical_errors(&self) -> bool {
        self.get_error_count() > 0
    }
    
    /// Устанавливает количество рабочих потоков
    pub fn set_worker_count(&mut self, workers: usize) {
        self.worker_count = if workers == 0 { num_cpus::get() } else { workers };
        
        // Настраиваем Rayon thread pool
        rayon::ThreadPoolBuilder::new()
            .num_threads(self.worker_count)
            .build_global()
            .expect("Failed to configure thread pool");
            
        tracing::info!("Configured {} worker threads for parallel analysis", self.worker_count);
    }
    
    /// Включает/выключает межмодульный анализ (заглушка)
    pub fn set_inter_module_analysis(&mut self, _enabled: bool) {
        // TODO: Реализовать межмодульный анализ
        tracing::info!("Inter-module analysis configuration not yet implemented");
    }
    
    /// Анализирует конфигурацию с использованием параллельной обработки
    pub async fn analyze_configuration(&mut self, config: &crate::configuration::Configuration) -> Result<Vec<AnalysisResult>> {
        tracing::info!("Analyzing configuration with {} modules and {} objects using {} workers", 
            config.modules.len(), 
            config.objects.len(),
            self.worker_count
        );
        
        // Собираем все пути модулей для анализа
        let analysis_tasks: Vec<_> = config.modules.iter()
            .map(|module| (module.path.clone(), module.name.clone()))
            .collect();
        
        if analysis_tasks.is_empty() {
            return Ok(vec![]);
        }
        
        // Параллельная обработка модулей
        let results: Result<Vec<_>, _> = analysis_tasks
            .into_par_iter()
            .map(|(file_path, module_name)| {
                self.analyze_module_parallel(&file_path, &module_name)
            })
            .collect();
            
        let module_results = results?;
        
        tracing::info!("Parallel analysis completed: {} modules processed", module_results.len());
        Ok(module_results)
    }
    
    /// Анализирует отдельный модуль (thread-safe)
    fn analyze_module_parallel(&self, file_path: &std::path::PathBuf, module_name: &str) -> Result<AnalysisResult> {
        let _start_time = std::time::Instant::now();
        
        // Читаем содержимое файла
        let content = match std::fs::read_to_string(file_path) {
            Ok(content) => content,
            Err(e) => {
                return Ok(AnalysisResult {
                    file_path: file_path.clone(),
                    errors: vec![crate::core::errors::AnalysisError {
                        message: format!("Failed to read file {}: {}", file_path.display(), e),
                        position: crate::parser::ast::Position { line: 0, column: 0, offset: 0 },
                        level: crate::core::errors::ErrorLevel::Error,
                        error_code: Some("FILE_READ_ERROR".to_string()),
                        related_positions: Vec::new(),
                        suggestion: Some("Check file permissions and path".to_string()),
                    }],
                    warnings: Vec::new(),
                    metrics: super::AnalysisMetrics {
                        lines_analyzed: 0,
                        procedures_count: 0,
                        functions_count: 0,
                        variables_count: 0,
                    },
                });
            }
        };
        
        // Создаем локальные копии анализаторов для потокобезопасности
        let mut local_syntax_analyzer = SyntaxAnalyzer::new();
        let semantic_config = SemanticAnalysisConfig::default();
        let mut local_semantic_analyzer = SemanticAnalyzer::new(semantic_config);
        
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // 1. Синтаксический анализ
        match local_syntax_analyzer.parse(&content) {
            Ok(ast) => {
                // 2. Семантический анализ
                if let Err(e) = local_semantic_analyzer.analyze(&ast) {
                    errors.push(crate::core::errors::AnalysisError {
                        message: format!("Semantic analysis error in {}: {}", module_name, e),
                        position: crate::parser::ast::Position { line: 0, column: 0, offset: 0 },
                        level: crate::core::errors::ErrorLevel::Error,
                        error_code: Some("SEMANTIC_ERROR".to_string()),
                        related_positions: Vec::new(),
                        suggestion: None,
                    });
                }
                
                // Собираем результаты семантического анализа
                let (semantic_errors, semantic_warnings) = local_semantic_analyzer.get_results();
                errors.extend(semantic_errors);
                warnings.extend(semantic_warnings);
            }
            Err(e) => {
                errors.push(crate::core::errors::AnalysisError {
                    message: format!("Syntax error in {}: {}", module_name, e),
                    position: crate::parser::ast::Position { line: 0, column: 0, offset: 0 },
                    level: crate::core::errors::ErrorLevel::Error,
                                                error_code: Some("SYNTAX_ERROR".to_string()),
                    related_positions: Vec::new(),
                    suggestion: None,
                });
            }
        }
        
        let lines_count = content.lines().count();
        
        Ok(AnalysisResult {
            file_path: file_path.clone(),
            errors,
            warnings,
            metrics: super::AnalysisMetrics {
                lines_analyzed: lines_count,
                procedures_count: 0, // TODO: подсчитать из AST
                functions_count: 0,  // TODO: подсчитать из AST  
                variables_count: 0,  // TODO: подсчитать из AST
            },
        })
    }

    /// Получает ссылку на верификатор методов
    pub fn get_method_verifier(&self) -> &MethodVerifier {
        &self.method_verifier
    }
    
    /// Получает мутабельную ссылку на верификатор методов
    pub fn get_method_verifier_mut(&mut self) -> &mut MethodVerifier {
        &mut self.method_verifier
    }
    
    /// Получает ссылку на семантический анализатор
    pub fn get_semantic_analyzer(&self) -> &SemanticAnalyzer {
        &self.semantic_analyzer
    }
    
    /// Получает мутабельную ссылку на семантический анализатор
    pub fn get_semantic_analyzer_mut(&mut self) -> &mut SemanticAnalyzer {
        &mut self.semantic_analyzer
    }
    
    // PRIVATE METHODS
    
    /// Верифицирует вызовы методов в AST
    fn verify_method_calls(&mut self, ast: &crate::parser::ast::AstNode, context: &mut AnalysisContext) {
        // Рекурсивно проходим по всем узлам AST
        self.visit_ast_node_for_method_calls(ast, context);
    }
    
    /// Посещает узел AST для поиска вызовов методов
    fn visit_ast_node_for_method_calls(&mut self, node: &crate::parser::ast::AstNode, context: &mut AnalysisContext) {
        use crate::parser::ast::AstNodeType;
        
        match node.node_type {
            AstNodeType::CallExpression => {
                // Анализируем вызов метода
                if let Some(call_info) = self.extract_method_call_info(node) {
                    let result = self.method_verifier.verify_call(
                        &call_info.object_type,
                        &call_info.method_name,
                        &call_info.arguments,
                        node.position().line
                    );
                    
                    // Создаем диагностику, если есть ошибка
                    if let Some(diagnostic) = self.method_verifier.create_diagnostic(
                        &result, 
                        node.position().line, 
                        node.position().column
                    ) {
                        context.diagnostics.push(diagnostic);
                    }
                }
            }
            AstNodeType::MemberExpression => {
                // Проверяем доступ к членам объекта
                if let Some(member_info) = self.extract_member_access_info(node) {
                    // Создаем пустой список аргументов для проверки существования метода/свойства
                    let empty_args = Vec::new();
                    let result = self.method_verifier.verify_call(
                        &member_info.object_type,
                        &member_info.member_name,
                        &empty_args,
                        node.position().line
                    );
                    
                    if let Some(diagnostic) = self.method_verifier.create_diagnostic(
                        &result,
                        node.position().line,
                        node.position().column
                    ) {
                        context.diagnostics.push(diagnostic);
                    }
                }
            }
            _ => {}
        }
        
        // Рекурсивно проверяем дочерние узлы
        for child in &node.children {
            self.visit_ast_node_for_method_calls(child, context);
        }
    }
    
    /// Извлекает информацию о вызове метода из узла AST
    fn extract_method_call_info(&self, node: &crate::parser::ast::AstNode) -> Option<MethodCallInfo> {
        // Простая реализация - в реальности будет более сложный анализ AST
        if let Some(method_name) = &node.value {
            // Попытка определить тип объекта из контекста
            let object_type = self.infer_object_type_from_context(node);
            
            // Извлекаем аргументы
            let arguments = self.extract_arguments_from_call(node);
            
            Some(MethodCallInfo {
                object_type,
                method_name: method_name.clone(),
                arguments,
            })
        } else {
            None
        }
    }
    
    /// Извлекает информацию о доступе к члену объекта
    fn extract_member_access_info(&self, node: &crate::parser::ast::AstNode) -> Option<MemberAccessInfo> {
        if let Some(member_name) = &node.value {
            let object_type = self.infer_object_type_from_context(node);
            
            Some(MemberAccessInfo {
                object_type,
                member_name: member_name.clone(),
            })
        } else {
            None
        }
    }
    
    /// Выводит тип объекта из контекста AST (упрощенная версия)
    fn infer_object_type_from_context(&self, node: &crate::parser::ast::AstNode) -> String {
        // Простая эвристика - в реальной реализации будет более сложный анализ типов
        
        // Ищем родительский узел, который может содержать информацию о типе
        if let Some(parent_info) = self.find_type_info_in_siblings(node) {
            return parent_info;
        }
        
        // Проверяем, есть ли в имени метода подсказки о типе
        if let Some(method_name) = &node.value {
            if method_name.contains("Таблица") || method_name.contains("Добавить") {
                return "ТаблицаЗначений".to_string();
            }
            if method_name.contains("Запрос") || method_name.contains("Выполнить") {
                return "Запрос".to_string();
            }
            if method_name.contains("Выборка") || method_name.contains("Следующий") {
                return "Выборка".to_string();
            }
        }
        
        // По умолчанию возвращаем универсальный тип
        "Объект".to_string()
    }
    
    /// Ищет информацию о типе в соседних узлах
    fn find_type_info_in_siblings(&self, _node: &crate::parser::ast::AstNode) -> Option<String> {
        // Заглушка - в реальной реализации будет анализ соседних узлов
        // для определения типа объекта из контекста
        None
    }
    
    /// Извлекает аргументы из вызова метода
    fn extract_arguments_from_call(&self, node: &crate::parser::ast::AstNode) -> Vec<ArgumentInfo> {
        use crate::parser::ast::AstNodeType;
        
        let mut arguments = Vec::new();
        
        // Ищем узлы с аргументами среди дочерних элементов
        for (index, child) in node.children.iter().enumerate() {
            match child.node_type {
                AstNodeType::StringLiteral => {
                    arguments.push(ArgumentInfo {
                        arg_type: "Строка".to_string(),
                        value: child.value.clone(),
                        position: index,
                    });
                }
                AstNodeType::NumberLiteral => {
                    arguments.push(ArgumentInfo {
                        arg_type: "Число".to_string(),
                        value: child.value.clone(),
                        position: index,
                    });
                }
                AstNodeType::BooleanLiteral => {
                    arguments.push(ArgumentInfo {
                        arg_type: "Булево".to_string(),
                        value: child.value.clone(),
                        position: index,
                    });
                }
                AstNodeType::Identifier => {
                    // Для идентификаторов пытаемся определить тип
                    let arg_type = self.method_verifier.analyze_expression_type(
                        &child.value.clone().unwrap_or_default()
                    );
                    arguments.push(ArgumentInfo {
                        arg_type,
                        value: child.value.clone(),
                        position: index,
                    });
                }
                _ => {
                    // Для остальных типов используем общий подход
                    arguments.push(ArgumentInfo {
                        arg_type: "Произвольный".to_string(),
                        value: child.value.clone(),
                        position: index,
                    });
                }
            }
        }
        
        arguments
    }
}

/// Информация о вызове метода
#[derive(Debug)]
struct MethodCallInfo {
    object_type: String,
    method_name: String,
    arguments: Vec<ArgumentInfo>,
}

/// Информация о доступе к члену объекта
#[derive(Debug)]  
struct MemberAccessInfo {
    object_type: String,
    member_name: String,
}

impl Default for AnalysisEngine {
    fn default() -> Self {
        Self::new()
    }
}
