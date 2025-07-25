/*!
# BSL Semantic Analyzer

Complete semantic analyzer for BSL with scope tracking, type checking,
and variable usage analysis. Ported from Python implementation.
*/

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::parser::ast::{AstNode, AstNodeType, Position};
use crate::core::errors::{AnalysisError, ErrorLevel};
use anyhow::Result;

/// Информация о типе
#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub name: String,
    pub description: Option<String>,
    pub parent: Option<String>,
    pub methods: HashMap<String, String>,
}

/// Variable information in scope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableInfo {
    pub name: String,
    pub var_type: Option<String>,
    pub position: Position,
    pub used: bool,
    pub is_parameter: bool,
    pub is_export: bool,
}

impl VariableInfo {
    pub fn new(name: String, position: Position) -> Self {
        Self {
            name,
            var_type: None,
            position,
            used: false,
            is_parameter: false,
            is_export: false,
        }
    }
    
    pub fn with_type(mut self, var_type: String) -> Self {
        self.var_type = Some(var_type);
        self
    }
    
    pub fn as_parameter(mut self) -> Self {
        self.is_parameter = true;
        self
    }
    
    pub fn as_export(mut self) -> Self {
        self.is_export = true;
        self
    }
    
    pub fn mark_used(&mut self) {
        self.used = true;
    }
}

/// Scope for variable visibility tracking
#[derive(Debug, Clone)]
pub struct Scope {
    pub name: String,
    pub variables: HashMap<String, VariableInfo>,
    pub parent: Option<Box<Scope>>,
    pub children: Vec<Scope>,
    pub scope_type: ScopeType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeType {
    Global,
    Procedure,
    Function,
    Block,
}

impl Scope {
    pub fn new(name: String, scope_type: ScopeType) -> Self {
        Self {
            name,
            variables: HashMap::new(),
            parent: None,
            children: Vec::new(),
            scope_type,
        }
    }
    
    pub fn with_parent(name: String, scope_type: ScopeType, parent: Box<Scope>) -> Self {
        Self {
            name,
            variables: HashMap::new(),
            parent: Some(parent),
            children: Vec::new(),
            scope_type,
        }
    }
    
    /// Add variable to current scope
    pub fn add_variable(&mut self, var_info: VariableInfo) {
        self.variables.insert(var_info.name.clone(), var_info);
    }
    
    /// Get variable from current or parent scopes
    pub fn get_variable(&self, name: &str) -> Option<&VariableInfo> {
        if let Some(var_info) = self.variables.get(name) {
            Some(var_info)
        } else if let Some(parent) = &self.parent {
            parent.get_variable(name)
        } else {
            None
        }
    }
    
    /// Get mutable variable reference
    pub fn get_variable_mut(&mut self, name: &str) -> Option<&mut VariableInfo> {
        if self.variables.contains_key(name) {
            self.variables.get_mut(name)
        } else if let Some(parent) = &mut self.parent {
            parent.get_variable_mut(name)
        } else {
            None
        }
    }
    
    /// Mark variable as used
    pub fn mark_variable_as_used(&mut self, name: &str) {
        if let Some(var_info) = self.get_variable_mut(name) {
            var_info.mark_used();
        }
    }
    
    /// Get unused variables in current scope
    pub fn get_unused_variables(&self) -> Vec<&VariableInfo> {
        self.variables
            .values()
            .filter(|var| !var.used && !var.is_parameter)
            .collect()
    }
}

/// BSL Type system for advanced type checking
#[derive(Debug, Clone)]
pub struct TypeSystem {
    pub builtin_types: HashMap<String, String>,
    pub known_objects: HashMap<String, String>,
    pub method_cache: HashMap<String, Vec<MethodInfo>>,
    pub global_functions: HashMap<String, FunctionInfo>,
}

/// Information about a method
#[derive(Debug, Clone)]
pub struct MethodInfo {
    pub name: String,
    pub parameters: Vec<ParameterInfo>,
    pub return_type: Option<String>,
    pub description: Option<String>,
    pub is_procedure: bool,
}

/// Information about a parameter
#[derive(Debug, Clone)]
pub struct ParameterInfo {
    pub name: String,
    pub param_type: Option<String>,
    pub is_optional: bool,
    pub default_value: Option<String>,
}

/// Information about a function
#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub parameters: Vec<ParameterInfo>,
    pub return_type: Option<String>,
    pub description: Option<String>,
    pub is_global: bool,
}

impl TypeSystem {
    pub fn new() -> Self {
        let mut type_system = Self {
            builtin_types: HashMap::new(),
            known_objects: HashMap::new(),
            method_cache: HashMap::new(),
            global_functions: HashMap::new(),
        };
        
        type_system.initialize_builtin_types();
        type_system.initialize_known_objects();
        type_system.initialize_method_cache();
        type_system.initialize_global_functions();
        
        type_system
    }
    
    fn initialize_builtin_types(&mut self) {
        self.builtin_types.insert("Строка".to_string(), "String".to_string());
        self.builtin_types.insert("Число".to_string(), "Number".to_string());
        self.builtin_types.insert("Булево".to_string(), "Boolean".to_string());
        self.builtin_types.insert("Дата".to_string(), "Date".to_string());
        self.builtin_types.insert("Неопределено".to_string(), "Undefined".to_string());
        self.builtin_types.insert("Произвольный".to_string(), "Any".to_string());
        self.builtin_types.insert("УникальныйИдентификатор".to_string(), "UUID".to_string());
    }
    
    fn initialize_known_objects(&mut self) {
        self.known_objects.insert("ТаблицаЗначений".to_string(), "ValueTable".to_string());
        self.known_objects.insert("Массив".to_string(), "Array".to_string());
        self.known_objects.insert("Структура".to_string(), "Structure".to_string());
        self.known_objects.insert("Соответствие".to_string(), "Map".to_string());
        self.known_objects.insert("Запрос".to_string(), "Query".to_string());
        self.known_objects.insert("Выборка".to_string(), "Selection".to_string());
        self.known_objects.insert("РезультатЗапроса".to_string(), "QueryResult".to_string());
        self.known_objects.insert("МенеджерВременныхТаблиц".to_string(), "TempTableManager".to_string());
        self.known_objects.insert("ПостроительЗапроса".to_string(), "QueryBuilder".to_string());
        self.known_objects.insert("СтрокаТаблицыЗначений".to_string(), "ValueTableRow".to_string());
        self.known_objects.insert("КолонкаТаблицыЗначений".to_string(), "ValueTableColumn".to_string());
        self.known_objects.insert("СписокЗначений".to_string(), "ValueList".to_string());
        self.known_objects.insert("ДеревоЗначений".to_string(), "ValueTree".to_string());
        self.known_objects.insert("ТекстовыйДокумент".to_string(), "TextDocument".to_string());
        self.known_objects.insert("ЧтениеXML".to_string(), "XMLReader".to_string());
        self.known_objects.insert("ЗаписьXML".to_string(), "XMLWriter".to_string());
        self.known_objects.insert("HTTPСоединение".to_string(), "HTTPConnection".to_string());
        self.known_objects.insert("HTTPЗапрос".to_string(), "HTTPRequest".to_string());
        self.known_objects.insert("HTTPОтвет".to_string(), "HTTPResponse".to_string());
    }
    
    fn initialize_method_cache(&mut self) {
        // ТаблицаЗначений methods
        let table_methods = vec![
            MethodInfo {
                name: "Добавить".to_string(),
                parameters: vec![],
                return_type: Some("СтрокаТаблицыЗначений".to_string()),
                description: Some("Добавляет новую строку в таблицу значений".to_string()),
                is_procedure: false,
            },
            MethodInfo {
                name: "Удалить".to_string(),
                parameters: vec![ParameterInfo {
                    name: "Строка".to_string(),
                    param_type: Some("СтрокаТаблицыЗначений".to_string()),
                    is_optional: false,
                    default_value: None,
                }],
                return_type: None,
                description: Some("Удаляет строку из таблицы значений".to_string()),
                is_procedure: true,
            },
            MethodInfo {
                name: "Очистить".to_string(),
                parameters: vec![],
                return_type: None,
                description: Some("Очищает все строки таблицы значений".to_string()),
                is_procedure: true,
            },
            MethodInfo {
                name: "Количество".to_string(),
                parameters: vec![],
                return_type: Some("Число".to_string()),
                description: Some("Возвращает количество строк в таблице".to_string()),
                is_procedure: false,
            },
            MethodInfo {
                name: "Найти".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Значение".to_string(),
                        param_type: Some("Произвольный".to_string()),
                        is_optional: false,
                        default_value: None,
                    },
                    ParameterInfo {
                        name: "Колонка".to_string(),
                        param_type: Some("Строка".to_string()),
                        is_optional: true,
                        default_value: None,
                    }
                ],
                return_type: Some("СтрокаТаблицыЗначений".to_string()),
                description: Some("Ищет строку в таблице по значению".to_string()),
                is_procedure: false,
            },
        ];
        self.method_cache.insert("ТаблицаЗначений".to_string(), table_methods);
        
        // Запрос methods
        let query_methods = vec![
            MethodInfo {
                name: "Выполнить".to_string(),
                parameters: vec![],
                return_type: Some("РезультатЗапроса".to_string()),
                description: Some("Выполняет запрос и возвращает результат".to_string()),
                is_procedure: false,
            },
            MethodInfo {
                name: "УстановитьПараметр".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "Имя".to_string(),
                        param_type: Some("Строка".to_string()),
                        is_optional: false,
                        default_value: None,
                    },
                    ParameterInfo {
                        name: "Значение".to_string(),
                        param_type: Some("Произвольный".to_string()),
                        is_optional: false,
                        default_value: None,
                    }
                ],
                return_type: None,
                description: Some("Устанавливает значение параметра запроса".to_string()),
                is_procedure: true,
            }
        ];
        self.method_cache.insert("Запрос".to_string(), query_methods);
        
        // Массив methods
        let array_methods = vec![
            MethodInfo {
                name: "Добавить".to_string(),
                parameters: vec![ParameterInfo {
                    name: "Значение".to_string(),
                    param_type: Some("Произвольный".to_string()),
                    is_optional: false,
                    default_value: None,
                }],
                return_type: None,
                description: Some("Добавляет элемент в массив".to_string()),
                is_procedure: true,
            },
            MethodInfo {
                name: "Количество".to_string(),
                parameters: vec![],
                return_type: Some("Число".to_string()),
                description: Some("Возвращает количество элементов массива".to_string()),
                is_procedure: false,
            }
        ];
        self.method_cache.insert("Массив".to_string(), array_methods);
    }
    
    fn initialize_global_functions(&mut self) {
        self.global_functions.insert("Сообщить".to_string(), FunctionInfo {
            name: "Сообщить".to_string(),
            parameters: vec![ParameterInfo {
                name: "Сообщение".to_string(),
                param_type: Some("Строка".to_string()),
                is_optional: false,
                default_value: None,
            }],
            return_type: None,
            description: Some("Выводит сообщение пользователю".to_string()),
            is_global: true,
        });
        
        self.global_functions.insert("Строка".to_string(), FunctionInfo {
            name: "Строка".to_string(),
            parameters: vec![ParameterInfo {
                name: "Значение".to_string(),
                param_type: Some("Произвольный".to_string()),
                is_optional: false,
                default_value: None,
            }],
            return_type: Some("Строка".to_string()),
            description: Some("Преобразует значение в строку".to_string()),
            is_global: true,
        });
        
        self.global_functions.insert("Число".to_string(), FunctionInfo {
            name: "Число".to_string(),
            parameters: vec![ParameterInfo {
                name: "Значение".to_string(),
                param_type: Some("Произвольный".to_string()),
                is_optional: false,
                default_value: None,
            }],
            return_type: Some("Число".to_string()),
            description: Some("Преобразует значение в число".to_string()),
            is_global: true,
        });
        
        self.global_functions.insert("ТипЗнч".to_string(), FunctionInfo {
            name: "ТипЗнч".to_string(),
            parameters: vec![ParameterInfo {
                name: "Значение".to_string(),
                param_type: Some("Произвольный".to_string()),
                is_optional: false,
                default_value: None,
            }],
            return_type: Some("Тип".to_string()),
            description: Some("Возвращает тип значения".to_string()),
            is_global: true,
        });
    }
    
    pub fn is_builtin_type(&self, type_name: &str) -> bool {
        self.builtin_types.contains_key(type_name)
    }
    
    pub fn is_known_object(&self, object_name: &str) -> bool {
        self.known_objects.contains_key(object_name)
    }
    
    pub fn is_global_function(&self, function_name: &str) -> bool {
        self.global_functions.contains_key(function_name)
    }
    
    pub fn get_global_function(&self, function_name: &str) -> Option<&FunctionInfo> {
        self.global_functions.get(function_name)
    }
    
    pub fn infer_type_from_literal(&self, literal: &str) -> Option<String> {
        if literal.starts_with('"') || literal.starts_with('\'') {
            Some("Строка".to_string())
        } else if literal.parse::<f64>().is_ok() {
            Some("Число".to_string())
        } else if literal == "Истина" || literal == "Ложь" {
            Some("Булево".to_string())
        } else if literal == "Неопределено" {
            Some("Неопределено".to_string())
        } else {
            None
        }
    }
    
    /// Проверяет существование метода у типа с полной информацией
    pub fn get_method_info(&self, object_type: &str, method_name: &str) -> Option<&MethodInfo> {
        if let Some(methods) = self.method_cache.get(object_type) {
            methods.iter().find(|m| m.name == method_name)
        } else {
            None
        }
    }
    
    /// Проверяет существование метода у типа (упрощенная версия)
    pub fn method_exists(&self, object_type: &str, method_name: &str) -> bool {
        self.get_method_info(object_type, method_name).is_some()
    }
    
    /// Получает сигнатуру метода
    pub fn get_method_signature(&self, object_type: &str, method_name: &str) -> Option<String> {
        if let Some(method_info) = self.get_method_info(object_type, method_name) {
            let params: Vec<String> = method_info.parameters.iter()
                .map(|p| {
                    let param_str = if let Some(ref param_type) = p.param_type {
                        format!("{}: {}", p.name, param_type)
                    } else {
                        p.name.clone()
                    };
                    if p.is_optional {
                        format!("[{}]", param_str)
                    } else {
                        param_str
                    }
                })
                .collect();
            
            let signature = format!("{}({})", method_name, params.join(", "));
            
            if let Some(ref return_type) = method_info.return_type {
                Some(format!("{} -> {}", signature, return_type))
            } else {
                Some(signature)
            }
        } else {
            None
        }
    }
    
    /// Получает список доступных методов для типа
    pub fn get_available_methods(&self, object_type: &str) -> Vec<String> {
        if let Some(methods) = self.method_cache.get(object_type) {
            methods.iter().map(|m| m.name.clone()).collect()
        } else {
            Vec::new()
        }
    }
    
    /// Получает полную информацию о методах типа
    pub fn get_methods_info(&self, object_type: &str) -> Option<&Vec<MethodInfo>> {
        self.method_cache.get(object_type)
    }
    
    /// Получает информацию о типе
    pub fn get_type_info(&self, type_name: &str) -> Option<TypeInfo> {
        // Создаем базовую информацию о типе
        if self.is_builtin_type(type_name) || self.is_known_object(type_name) {
            Some(TypeInfo {
                name: type_name.to_string(),
                description: Some(format!("Системный тип: {}", type_name)),
                parent: None,
                methods: self.get_method_map(type_name),
            })
        } else {
            None
        }
    }
    
    /// Проверяет совместимость типов
    pub fn types_compatible(&self, source_type: &str, target_type: &str) -> bool {
        // Простая проверка совместимости
        if source_type == target_type {
            return true;
        }
        
        // Произвольный тип совместим с любым
        if source_type == "Произвольный" || target_type == "Произвольный" {
            return true;
        }
        
        // Неопределено совместимо с любым типом
        if source_type == "Неопределено" || target_type == "Неопределено" {
            return true;
        }
        
        // Число может быть совместимо со строкой в определенных случаях
        if (source_type == "Число" && target_type == "Строка") ||
           (source_type == "Строка" && target_type == "Число") {
            return true;
        }
        
        false
    }
    
    /// Валидирует параметры вызова метода
    pub fn validate_method_call(&self, object_type: &str, method_name: &str, arg_count: usize) -> Result<(), String> {
        if let Some(method_info) = self.get_method_info(object_type, method_name) {
            let required_params = method_info.parameters.iter()
                .filter(|p| !p.is_optional)
                .count();
            let total_params = method_info.parameters.len();
            
            if arg_count < required_params {
                return Err(format!(
                    "Недостаточно параметров для метода {}. Требуется: {}, передано: {}",
                    method_name, required_params, arg_count
                ));
            }
            
            if arg_count > total_params {
                return Err(format!(
                    "Слишком много параметров для метода {}. Максимум: {}, передано: {}",
                    method_name, total_params, arg_count
                ));
            }
            
            Ok(())
        } else {
            Err(format!("Метод {} не найден у типа {}", method_name, object_type))
        }
    }
    
    /// Получает карту методов для типа (вспомогательная функция)
    fn get_method_map(&self, type_name: &str) -> HashMap<String, String> {
        let methods = self.get_available_methods(type_name);
        let mut method_map = HashMap::new();
        
        for method in methods {
            let signature = self.get_method_signature(type_name, &method)
                .unwrap_or_else(|| format!("{}()", method));
            method_map.insert(method, signature);
        }
        
        method_map
    }
    
    /// Получает список всех доступных объектных типов
    pub fn get_available_objects(&self) -> Vec<String> {
        self.known_objects.keys().cloned().collect()
    }
    
    /// Получает список всех глобальных функций
    pub fn get_global_functions(&self) -> Vec<String> {
        self.global_functions.keys().cloned().collect()
    }
}

impl Default for TypeSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Enhanced semantic analyzer configuration
#[derive(Debug, Clone)]
pub struct SemanticAnalysisConfig {
    pub check_unused_variables: bool,
    pub check_undefined_variables: bool,
    pub check_type_compatibility: bool,
    pub check_method_calls: bool,
    pub check_parameter_count: bool,
    pub warn_on_implicit_conversions: bool,
    pub suggest_similar_names: bool,
    pub analyze_global_functions: bool,
    pub verbose: bool,
}

impl Default for SemanticAnalysisConfig {
    fn default() -> Self {
        Self {
            check_unused_variables: true,
            check_undefined_variables: true,
            check_type_compatibility: true,
            check_method_calls: true,
            check_parameter_count: true,
            warn_on_implicit_conversions: true,
            suggest_similar_names: true,
            analyze_global_functions: true,
            verbose: false,
        }
    }
}

/// Информация о вызове метода
#[derive(Debug, Clone)]
struct MethodCallInfo {
    object_type: String,
    method_name: String,
    args: Vec<String>,
}

/// Semantic analyzer for BSL
pub struct SemanticAnalyzer {
    pub type_system: TypeSystem,
    pub config: SemanticAnalysisConfig,
    pub current_scope: Scope,
    pub current_file_path: std::path::PathBuf,
    pub errors: Vec<AnalysisError>,
    pub warnings: Vec<AnalysisError>,
}

impl SemanticAnalyzer {
    pub fn new(config: SemanticAnalysisConfig) -> Self {
        Self {
            type_system: TypeSystem::new(),
            config,
            current_scope: Scope::new("global".to_string(), ScopeType::Global),
            current_file_path: std::path::PathBuf::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
    
    /// Set current file path for error reporting
    pub fn set_file_path(&mut self, file_path: std::path::PathBuf) {
        self.current_file_path = file_path;
    }
    
    /// Perform semantic analysis on AST
    pub fn analyze(&mut self, ast: &AstNode) -> Result<()> {
        if self.config.verbose {
            println!("📊 Starting semantic analysis");
        }
        
        self.errors.clear();
        self.warnings.clear();
        self.current_scope = Scope::new("global".to_string(), ScopeType::Global);
        
        // Analyze the AST
        self.analyze_node(ast)?;
        
        // Check for unused variables
        if self.config.check_unused_variables {
            self.check_unused_variables();
        }
        
        if self.config.verbose {
            println!("📊 Semantic analysis completed. Errors: {}, Warnings: {}", 
                    self.errors.len(), self.warnings.len());
        }
        
        Ok(())
    }
    
    /// Analyze a single AST node
    fn analyze_node(&mut self, node: &AstNode) -> Result<()> {
        if self.config.verbose {
            println!("🔍 Analyzing node: {:?}", node.node_type);
        }
        
        match node.node_type {
            AstNodeType::Module => self.analyze_module(node),
            AstNodeType::Procedure => self.analyze_procedure(node),
            AstNodeType::Function => self.analyze_function(node),
            AstNodeType::Variable => self.analyze_variable(node),
            AstNodeType::Assignment => self.analyze_assignment(node),
            AstNodeType::Identifier => self.analyze_identifier(node),
            AstNodeType::Block => self.analyze_block(node),
            AstNodeType::CallExpression => self.analyze_call_expression(node),
            AstNodeType::Expression => self.analyze_expression(node),
            AstNodeType::IfStatement => self.analyze_if_statement(node),
            AstNodeType::Parameter => self.analyze_parameter(node),
            _ => {
                // Recursively analyze children for other node types
                for child in &node.children {
                    self.analyze_node(child)?;
                }
                Ok(())
            }
        }
    }
    
    fn analyze_module(&mut self, node: &AstNode) -> Result<()> {
        // Analyze all children (procedures, functions, variable declarations)
        for child in &node.children {
            self.analyze_node(child)?;
        }
        Ok(())
    }
    
    fn analyze_procedure(&mut self, node: &AstNode) -> Result<()> {
        let proc_name = node.name().unwrap_or("unnamed_procedure").to_string();
        
        // Create new scope for procedure
        let proc_scope = Scope::new(proc_name.clone(), ScopeType::Procedure);
        let old_scope = std::mem::replace(&mut self.current_scope, proc_scope);
        
        // TODO: Add procedure parameters to scope
        
        // Analyze procedure body
        for child in &node.children {
            self.analyze_node(child)?;
        }
        
        // Restore previous scope
        self.current_scope = old_scope;
        Ok(())
    }
    
    fn analyze_function(&mut self, node: &AstNode) -> Result<()> {
        let func_name = node.name().unwrap_or("unnamed_function").to_string();
        
        // Create new scope for function
        let func_scope = Scope::new(func_name.clone(), ScopeType::Function);
        let old_scope = std::mem::replace(&mut self.current_scope, func_scope);
        
        // TODO: Add function parameters to scope
        // TODO: Check return statements
        
        // Analyze function body
        for child in &node.children {
            self.analyze_node(child)?;
        }
        
        // Restore previous scope
        self.current_scope = old_scope;
        Ok(())
    }
    
    fn analyze_variable(&mut self, node: &AstNode) -> Result<()> {
        if let Some(var_name) = node.name() {
            let var_info = VariableInfo::new(var_name.to_string(), node.position());
            
            // Check if variable already exists in current scope
            if self.current_scope.variables.contains_key(var_name) {
                self.add_error(
                    format!("Variable '{}' is already declared in this scope", var_name),
                    node.position(),
                    ErrorLevel::Error,
                );
            } else {
                self.current_scope.add_variable(var_info);
            }
        }
        Ok(())
    }
    
    fn analyze_assignment(&mut self, node: &AstNode) -> Result<()> {
        // Find the identifier being assigned to
        for child in &node.children {
            if child.node_type == AstNodeType::Identifier {
                if let Some(var_name) = child.name() {
                    // Check if variable is declared
                    if self.config.check_undefined_variables && self.current_scope.get_variable(var_name).is_none() {
                        self.add_warning(
                            format!("Variable '{}' is used but not declared", var_name),
                            child.position(),
                            ErrorLevel::Warning,
                        );
                    }
                    
                    // Mark as used
                    self.current_scope.mark_variable_as_used(var_name);
                }
            }
        }
        
        // Analyze all children
        for child in &node.children {
            self.analyze_node(child)?;
        }
        
        Ok(())
    }
    
    fn analyze_identifier(&mut self, node: &AstNode) -> Result<()> {
        if let Some(identifier) = node.name() {
            if self.config.verbose {
                println!("🔍 Analyzing identifier: {}", identifier);
            }
            
            // Проверяем, не является ли идентификатор частью конструктора
            if self.is_identifier_in_constructor(node) {
                return Ok(());
            }
            
            // Проверяем, не является ли это глобальной функцией
            if self.type_system.is_global_function(identifier) {
                if self.config.verbose {
                    println!("✅ Identifier '{}' is a global function", identifier);
                }
                return Ok(());
            }
            
            // Проверяем объявленные переменные
            if self.config.check_undefined_variables {
                if let Some(_var_info) = self.current_scope.get_variable(identifier) {
                    // Переменная найдена, отмечаем как использованную
                    self.current_scope.mark_variable_as_used(identifier);
                    
                    if self.config.verbose {
                        println!("✅ Variable '{}' found and marked as used", identifier);
                    }
                } else {
                    // Переменная не найдена
                    if self.config.suggest_similar_names {
                        let available_vars: Vec<String> = self.current_scope.variables.keys().cloned().collect();
                        let similar_vars = self.find_similar_variables(identifier, &available_vars);
                        
                        let warning_msg = format!("Идентификатор '{}' используется, но не объявлен", identifier);
                        
                        if !similar_vars.is_empty() {
                            let suggestion = format!("Возможно, вы имели в виду: {}", similar_vars.join(", "));
                            self.add_warning_with_suggestion(
                                warning_msg,
                                node.position(),
                                ErrorLevel::Warning,
                                suggestion,
                            );
                        } else {
                            self.add_warning(
                                warning_msg,
                                node.position(),
                                ErrorLevel::Warning,
                            );
                        }
                    } else {
                        self.add_warning(
                            format!("Идентификатор '{}' используется, но не объявлен", identifier),
                            node.position(),
                            ErrorLevel::Warning,
                        );
                    }
                }
            }
        }
        Ok(())
    }
    
    /// Находит похожие переменные
    fn find_similar_variables(&self, target_var: &str, available_vars: &[String]) -> Vec<String> {
        let mut similar = Vec::new();
        let target_lower = target_var.to_lowercase();
        
        for var_name in available_vars {
            let var_lower = var_name.to_lowercase();
            
            // Проверяем различные критерии схожести
            if self.levenshtein_distance(&target_lower, &var_lower) <= 2 ||
               target_lower.contains(&var_lower) ||
               var_lower.contains(&target_lower) {
                similar.push(var_name.clone());
            }
        }
        
        // Ограничиваем количество предложений
        similar.truncate(3);
        similar
    }
    
    /// Check if identifier is part of constructor expression 
    fn is_identifier_in_constructor(&self, node: &AstNode) -> bool {
        // This is a simplified check - in real implementation we'd need
        // to traverse up the AST to check if this identifier follows a "Новый" keyword
        // For now, we check if the identifier is a known object type
        if let Some(identifier) = node.name() {
            self.type_system.is_known_object(identifier)
        } else {
            false
        }
    }
    
    fn analyze_block(&mut self, node: &AstNode) -> Result<()> {
        // Create new block scope
        let block_scope = Scope::new("block".to_string(), ScopeType::Block);
        let old_scope = std::mem::replace(&mut self.current_scope, block_scope);
        
        // Analyze all statements in block
        for child in &node.children {
            self.analyze_node(child)?;
        }
        
        // Restore previous scope
        self.current_scope = old_scope;
        Ok(())
    }
    
    fn analyze_call_expression(&mut self, node: &AstNode) -> Result<()> {
        if self.config.verbose {
            println!("🔍 Analyzing call expression: {:?}", node);
        }
        
        // Анализируем вызов функции или метода
        if let Some(function_name) = self.extract_function_name(node) {
            // Проверяем глобальные функции
            if self.config.analyze_global_functions && self.type_system.is_global_function(&function_name) {
                self.analyze_global_function_call(&function_name, node)?;
            } else if let Some(object_info) = self.extract_method_call_info(node) {
                // Проверяем вызов метода объекта
                self.analyze_method_call(&object_info.object_type, &object_info.method_name, &object_info.args, node)?;
            }
        }
        
        // Анализируем все дочерние узлы
        for child in &node.children {
            self.analyze_node(child)?;
        }
        
        Ok(())
    }
    
    /// Извлекает имя функции из узла вызова
    fn extract_function_name(&self, node: &AstNode) -> Option<String> {
        // Ищем первый идентификатор среди детей
        for child in &node.children {
            if child.node_type == AstNodeType::Identifier {
                return child.name().map(|s| s.to_string());
            }
        }
        None
    }
    
    /// Извлекает информацию о вызове метода
    fn extract_method_call_info(&self, node: &AstNode) -> Option<MethodCallInfo> {
        // Упрощенный алгоритм - ищем паттерн: Объект.Метод(аргументы)
        // В реальной реализации нужен более сложный парсинг AST
        
        if node.children.len() >= 2 {
            // Предполагаем, что первый ребенок - объект, второй - метод
            if let (Some(object_name), Some(method_name)) = (
                node.children[0].name(),
                node.children[1].name()
            ) {
                // Определяем тип объекта (упрощенно - по имени переменной)
                if let Some(var_info) = self.current_scope.get_variable(object_name) {
                    if let Some(ref var_type) = var_info.var_type {
                        return Some(MethodCallInfo {
                            object_type: var_type.clone(),
                            method_name: method_name.to_string(),
                            args: Vec::new(), // TODO: извлечь аргументы
                        });
                    }
                }
            }
        }
        
        None
    }
    
    /// Анализирует вызов глобальной функции
    fn analyze_global_function_call(&mut self, function_name: &str, node: &AstNode) -> Result<()> {
        if let Some(function_info) = self.type_system.get_global_function(function_name) {
            if self.config.verbose {
                println!("✅ Found global function: {}", function_name);
            }
            
            // Подсчитываем количество аргументов (упрощенно)
            let arg_count = self.count_arguments(node);
            
            // Проверяем количество параметров
            if self.config.check_parameter_count {
                let required_params = function_info.parameters.iter()
                    .filter(|p| !p.is_optional)
                    .count();
                let total_params = function_info.parameters.len();
                
                if arg_count < required_params {
                    self.add_error(
                        format!(
                            "Недостаточно параметров для функции '{}'. Требуется: {}, передано: {}",
                            function_name, required_params, arg_count
                        ),
                        node.position(),
                        ErrorLevel::Error,
                    );
                } else if arg_count > total_params {
                    self.add_error(
                        format!(
                            "Слишком много параметров для функции '{}'. Максимум: {}, передано: {}",
                            function_name, total_params, arg_count
                        ),
                        node.position(),
                        ErrorLevel::Error,
                    );
                }
            }
        } else {
            // Функция не найдена среди глобальных - возможно, это пользовательская функция
            if self.config.verbose {
                println!("⚠️ Unknown global function: {}", function_name);
            }
        }
        
        Ok(())
    }
    
    /// Анализирует вызов метода объекта
    fn analyze_method_call(&mut self, object_type: &str, method_name: &str, _args: &[String], node: &AstNode) -> Result<()> {
        if self.config.check_method_calls {
            if let Some(_method_info) = self.type_system.get_method_info(object_type, method_name) {
                if self.config.verbose {
                    println!("✅ Found method: {}.{}", object_type, method_name);
                }
                
                // Проверяем количество параметров
                if self.config.check_parameter_count {
                    let arg_count = self.count_arguments(node);
                    
                    if let Err(error_msg) = self.type_system.validate_method_call(object_type, method_name, arg_count) {
                        self.add_error(
                            error_msg,
                            node.position(),
                            ErrorLevel::Error,
                        );
                    }
                }
            } else {
                // Метод не найден
                let available_methods = self.type_system.get_available_methods(object_type);
                
                if !available_methods.is_empty() {
                    let similar_methods = self.find_similar_methods(method_name, &available_methods);
                    
                    let error_msg = format!("Метод '{}' не найден у типа '{}'", method_name, object_type);
                    let suggestion = if !similar_methods.is_empty() {
                        format!("Возможно, вы имели в виду: {}", similar_methods.join(", "))
                    } else {
                        format!("Доступные методы: {}", available_methods.join(", "))
                    };
                    
                    self.add_error_with_suggestion(
                        error_msg,
                        node.position(),
                        ErrorLevel::Error,
                        suggestion,
                    );
                } else {
                    self.add_error(
                        format!("Неизвестный тип объекта '{}'", object_type),
                        node.position(),
                        ErrorLevel::Warning,
                    );
                }
            }
        }
        
        Ok(())
    }
    
    /// Подсчитывает количество аргументов в вызове (упрощенно)
    fn count_arguments(&self, node: &AstNode) -> usize {
        // Упрощенная логика - считаем узлы Expression среди детей
        node.children.iter()
            .filter(|child| child.node_type == AstNodeType::Expression)
            .count()
    }
    
    /// Находит похожие методы
    fn find_similar_methods(&self, target_method: &str, available_methods: &[String]) -> Vec<String> {
        let mut similar = Vec::new();
        let target_lower = target_method.to_lowercase();
        
        for method_name in available_methods {
            let method_lower = method_name.to_lowercase();
            
            // Проверяем различные критерии схожести
            if self.levenshtein_distance(&target_lower, &method_lower) <= 2 ||
               target_lower.contains(&method_lower) ||
               method_lower.contains(&target_lower) {
                similar.push(method_name.clone());
            }
        }
        
        // Ограничиваем количество предложений
        similar.truncate(3);
        similar
    }
    
    fn analyze_expression(&mut self, node: &AstNode) -> Result<()> {
        if self.config.verbose {
            println!("🔍 Analyzing expression node with {} children", node.children.len());
        }
        
        // Check for constructor expressions like "Новый ТипОбъекта()"
        self.check_constructor_expression(node)?;
        
        // Recursively analyze child expressions
        for child in &node.children {
            self.analyze_node(child)?;
        }
        
        Ok(())
    }
    
    fn analyze_if_statement(&mut self, node: &AstNode) -> Result<()> {
        // Analyze condition and blocks
        for child in &node.children {
            match child.node_type {
                AstNodeType::Expression => {
                    // Analyze condition
                    self.analyze_node(child)?;
                }
                AstNodeType::Block => {
                    // Analyze then/else blocks
                    self.analyze_node(child)?;
                }
                _ => {
                    self.analyze_node(child)?;
                }
            }
        }
        Ok(())
    }
    
    fn analyze_parameter(&mut self, node: &AstNode) -> Result<()> {
        if let Some(param_name) = node.name() {
            let var_info = VariableInfo::new(param_name.to_string(), node.position())
                .as_parameter();
            
            self.current_scope.add_variable(var_info);
        }
        Ok(())
    }
    
    /// Check for constructor expressions like "Новый ТипОбъекта()"
    fn check_constructor_expression(&mut self, node: &AstNode) -> Result<()> {
        let children = &node.children;
        
        if self.config.verbose {
            println!("🔍 Checking expression node, children: {}", children.len());
            for (i, child) in children.iter().enumerate() {
                println!("  {}: type={:?}, value={:?}", i, child.node_type, child.value);
            }
        }
        
        // Check for pattern: "Новый" + "ТипОбъекта" + "(" + ")"
        if children.len() >= 2 {
            let first_child = &children[0];
            let second_child = &children[1];
            
            // Check if first token is "Новый" (keyword) and second is identifier
            if matches!(first_child.node_type, AstNodeType::Keyword) &&
               first_child.value.as_ref().is_some_and(|v| v == "Новый") &&
               matches!(second_child.node_type, AstNodeType::Identifier) {
                
                if let Some(type_name) = &second_child.value {
                    if self.config.verbose {
                        println!("🎯 Found constructor: Новый {}", type_name);
                    }
                    
                    // Check if this type exists in the type system
                    if self.type_system.get_type_info(type_name).is_none() {
                        let available_types = self.get_available_object_types();
                        
                        // Find similar types
                        let similar_types = self.find_similar_types(type_name, &available_types);
                        
                        let error_msg = format!("Unknown object type '{}' in constructor", type_name);
                        let suggestion = if !similar_types.is_empty() {
                            format!("Did you mean: {}", similar_types.join(", "))
                        } else {
                            format!("Available types: {}", available_types.join(", "))
                        };
                        
                        self.add_error_with_suggestion(
                            error_msg,
                            second_child.position(),
                            ErrorLevel::Error,
                            suggestion,
                        );
                    } else if self.config.verbose {
                        println!("✅ Type {} found in type system", type_name);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Get list of available object types
    pub fn get_available_object_types(&self) -> Vec<String> {
        self.type_system.known_objects.keys().cloned().collect()
    }
    
    /// Find types similar to target type using Levenshtein distance
    pub fn find_similar_types(&self, target_type: &str, available_types: &[String]) -> Vec<String> {
        let mut similar = Vec::new();
        let target_lower = target_type.to_lowercase();
        
        for type_name in available_types {
            let type_lower = type_name.to_lowercase();
            
            // Check various similarity criteria
            if self.levenshtein_distance(&target_lower, &type_lower) <= 2 ||
               target_lower.contains(&type_lower) ||
               type_lower.contains(&target_lower) {
                similar.push(type_name.clone());
            }
        }
        
        // Limit suggestions to 3
        similar.truncate(3);
        similar
    }
    
    /// Calculate Levenshtein distance between two strings
    pub fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        let len1 = s1.chars().count();
        let len2 = s2.chars().count();
        
        if len1 == 0 {
            return len2;
        }
        if len2 == 0 {
            return len1;
        }
        
        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];
        
        // Initialize first row and column
        for (i, row) in matrix.iter_mut().enumerate().take(len1 + 1) {
            row[0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }
        
        let s1_chars: Vec<char> = s1.chars().collect();
        let s2_chars: Vec<char> = s2.chars().collect();
        
        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if s1_chars[i - 1] == s2_chars[j - 1] { 0 } else { 1 };
                
                matrix[i][j] = std::cmp::min(
                    std::cmp::min(
                        matrix[i - 1][j] + 1,      // deletion
                        matrix[i][j - 1] + 1       // insertion
                    ),
                    matrix[i - 1][j - 1] + cost    // substitution
                );
            }
        }
        
        matrix[len1][len2]
    }
    
    fn check_unused_variables(&mut self) {
        let unused: Vec<(String, Position)> = self.current_scope.get_unused_variables()
            .iter()
            .map(|var| (var.name.clone(), var.position))
            .collect();
            
        for (name, position) in unused {
            self.add_warning(
                format!("Variable '{}' is declared but never used", name),
                position,
                ErrorLevel::Warning,
            );
        }
    }
    
    fn add_error(&mut self, message: String, position: Position, level: ErrorLevel) {
        let error = AnalysisError::new(message, self.current_file_path.clone(), position, level);
        match level {
            ErrorLevel::Error => self.errors.push(error),
            ErrorLevel::Warning => self.warnings.push(error),
            _ => self.warnings.push(error),
        }
    }
    
    fn add_warning(&mut self, message: String, position: Position, level: ErrorLevel) {
        let warning = AnalysisError::new(message, self.current_file_path.clone(), position, level);
        self.warnings.push(warning);
    }
    
    fn add_warning_with_suggestion(&mut self, message: String, position: Position, level: ErrorLevel, suggestion: String) {
        let warning = AnalysisError::new(message, self.current_file_path.clone(), position, level)
            .with_suggestion(suggestion);
        self.warnings.push(warning);
    }
    
    fn add_error_with_suggestion(&mut self, message: String, position: Position, level: ErrorLevel, suggestion: String) {
        let error = AnalysisError::new(message, self.current_file_path.clone(), position, level)
            .with_suggestion(suggestion);
        
        match level {
            ErrorLevel::Error => self.errors.push(error),
            ErrorLevel::Warning => self.warnings.push(error),
            _ => self.warnings.push(error),
        }
    }
    
    /// Get all analysis results
    pub fn get_results(&self) -> (Vec<AnalysisError>, Vec<AnalysisError>) {
        (self.errors.clone(), self.warnings.clone())
    }
    
    /// Clear all analysis results
    pub fn clear_results(&mut self) {
        self.errors.clear();
        self.warnings.clear();
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new(SemanticAnalysisConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::parser::ast::{AstNode, Span}; // Unused imports

    #[test]
    fn test_scope_creation() {
        let scope = Scope::new("test".to_string(), ScopeType::Global);
        assert_eq!(scope.name, "test");
        assert_eq!(scope.scope_type, ScopeType::Global);
    }

    #[test]
    fn test_variable_info() {
        let pos = Position::new(1, 1, 0);
        let var_info = VariableInfo::new("test_var".to_string(), pos)
            .with_type("String".to_string())
            .as_parameter();
        
        assert_eq!(var_info.name, "test_var");
        assert_eq!(var_info.var_type, Some("String".to_string()));
        assert!(var_info.is_parameter);
        assert!(!var_info.used);
    }

    #[test]
    fn test_type_system() {
        let type_system = TypeSystem::new();
        assert!(type_system.is_builtin_type("Строка"));
        assert!(type_system.is_known_object("ТаблицаЗначений"));
        assert!(!type_system.is_builtin_type("НеизвестныйТип"));
    }

    #[test]
    fn test_semantic_analyzer_creation() {
        let analyzer = SemanticAnalyzer::new(SemanticAnalysisConfig::default());
        assert_eq!(analyzer.current_scope.name, "global");
        assert_eq!(analyzer.current_scope.scope_type, ScopeType::Global);
    }
}
