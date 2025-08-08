//! BSL Semantic Analyzer для tree-sitter парсера

use crate::bsl_parser::{ast::*, diagnostics::*, keywords};
use crate::core::errors::{AnalysisError, ErrorLevel};
use crate::parser::ast::Position;
use crate::unified_index::UnifiedBslIndex;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Информация о переменной в области видимости
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableInfo {
    pub name: String,
    pub var_type: Option<String>,
    pub location: Location,
    pub used: bool,
    pub is_parameter: bool,
    pub is_export: bool,
    /// Отслеживает, была ли переменная инициализирована
    pub is_initialized: bool,
    /// Последнее известное значение/тип из присваивания
    pub inferred_type: Option<String>,
}

impl VariableInfo {
    pub fn new(name: String, location: Location) -> Self {
        Self {
            name,
            var_type: None,
            location,
            used: false,
            is_parameter: false,
            is_export: false,
            is_initialized: false,
            inferred_type: None,
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

    /// Устанавливает выведенный тип из присваивания
    pub fn set_inferred_type(&mut self, inferred_type: String) {
        self.inferred_type = Some(inferred_type);
        self.is_initialized = true;
    }

    /// Возвращает наиболее точный известный тип переменной
    pub fn get_effective_type(&self) -> Option<&String> {
        // Приоритет: выведенный тип > объявленный тип
        self.inferred_type.as_ref().or(self.var_type.as_ref())
    }
}

/// Область видимости для отслеживания переменных
#[derive(Debug, Clone)]
pub struct Scope {
    pub name: String,
    pub variables: HashMap<String, VariableInfo>,
    pub parent: Option<Box<Scope>>,
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
            scope_type,
        }
    }

    pub fn with_parent(name: String, scope_type: ScopeType, parent: Box<Scope>) -> Self {
        Self {
            name,
            variables: HashMap::new(),
            parent: Some(parent),
            scope_type,
        }
    }

    /// Добавить переменную в текущую область видимости
    pub fn add_variable(&mut self, var_info: VariableInfo) {
        self.variables.insert(var_info.name.clone(), var_info);
    }

    /// Найти переменную в текущей или родительских областях
    pub fn get_variable(&self, name: &str) -> Option<&VariableInfo> {
        if let Some(var_info) = self.variables.get(name) {
            Some(var_info)
        } else if let Some(parent) = &self.parent {
            parent.get_variable(name)
        } else {
            None
        }
    }

    /// Получить мутабельную ссылку на переменную
    pub fn get_variable_mut(&mut self, name: &str) -> Option<&mut VariableInfo> {
        if self.variables.contains_key(name) {
            self.variables.get_mut(name)
        } else if let Some(parent) = &mut self.parent {
            parent.get_variable_mut(name)
        } else {
            None
        }
    }

    /// Отметить переменную как использованную
    pub fn mark_variable_as_used(&mut self, name: &str) {
        if let Some(var_info) = self.get_variable_mut(name) {
            var_info.mark_used();
        }
    }

    /// Получить неиспользованные переменные
    pub fn get_unused_variables(&self) -> Vec<&VariableInfo> {
        self.variables
            .values()
            .filter(|var| !var.used && !var.is_parameter)
            .collect()
    }
}

/// Конфигурация семантического анализа
#[derive(Debug, Clone)]
pub struct SemanticAnalysisConfig {
    pub check_unused_variables: bool,
    pub check_undeclared_variables: bool,
    pub check_uninitialized_variables: bool,
    pub check_duplicate_parameters: bool,
    pub check_method_calls: bool,
    pub strict_typing: bool,
}

impl Default for SemanticAnalysisConfig {
    fn default() -> Self {
        Self {
            check_unused_variables: true,
            check_undeclared_variables: true,
            check_uninitialized_variables: true,
            check_duplicate_parameters: true,
            check_method_calls: true,
            strict_typing: false,
        }
    }
}

/// Информация о типе менеджера 1С
#[derive(Debug, Clone)]
struct ManagerInfo {
    /// Базовый тип менеджера (например, "СправочникМенеджер")
    base_type: String,
    /// Вид объекта для сообщений (именительный падеж, например, "Справочник")
    kind: String,
}

/// Семантический анализатор для BSL
pub struct SemanticAnalyzer {
    config: SemanticAnalysisConfig,
    current_scope: Scope,
    scope_stack: Vec<Scope>,
    diagnostics: Vec<Diagnostic>,
    index: Option<UnifiedBslIndex>,
    /// Локальные функции и процедуры, определенные в текущем модуле
    local_functions: HashMap<String, Location>,
}

impl SemanticAnalyzer {
    pub fn new(config: SemanticAnalysisConfig) -> Self {
        Self {
            config,
            current_scope: Scope::new("global".to_string(), ScopeType::Global),
            scope_stack: Vec::new(),
            diagnostics: Vec::new(),
            index: None,
            local_functions: HashMap::new(),
        }
    }

    /// Создает новый анализатор с UnifiedBslIndex
    pub fn with_index(config: SemanticAnalysisConfig, index: UnifiedBslIndex) -> Self {
        Self {
            config,
            current_scope: Scope::new("global".to_string(), ScopeType::Global),
            scope_stack: Vec::new(),
            diagnostics: Vec::new(),
            index: Some(index),
            local_functions: HashMap::new(),
        }
    }

    /// Анализ AST
    pub fn analyze(&mut self, ast: &BslAst) -> Result<()> {
        self.diagnostics.clear();
        self.local_functions.clear();

        // Сначала собираем все локальные функции и процедуры
        self.collect_local_functions(&ast.module);

        // Затем анализируем модуль
        self.analyze_module(&ast.module)?;

        // Проверяем неиспользованные переменные
        if self.config.check_unused_variables {
            self.check_unused_variables();
        }

        Ok(())
    }

    /// Анализ модуля
    fn analyze_module(&mut self, module: &Module) -> Result<()> {
        // Анализируем объявления
        for declaration in &module.declarations {
            self.analyze_declaration(declaration)?;
        }

        Ok(())
    }

    /// Анализ объявления
    fn analyze_declaration(&mut self, declaration: &Declaration) -> Result<()> {
        match declaration {
            Declaration::Procedure(proc) => {
                self.enter_scope(proc.name.clone(), ScopeType::Procedure);

                // Проверяем дублированные параметры
                self.check_duplicate_parameters(&proc.params);

                // Добавляем параметры
                for param in &proc.params {
                    let var_info = VariableInfo::new(param.name.clone(), param.location.clone())
                        .as_parameter();
                    self.current_scope.add_variable(var_info);
                }

                // Анализируем тело процедуры
                for stmt in &proc.body {
                    self.analyze_statement(stmt)?;
                }

                self.exit_scope();
            }
            Declaration::Function(func) => {
                self.enter_scope(func.name.clone(), ScopeType::Function);

                // Проверяем дублированные параметры
                self.check_duplicate_parameters(&func.params);

                // Добавляем параметры
                for param in &func.params {
                    let var_info = VariableInfo::new(param.name.clone(), param.location.clone())
                        .as_parameter();
                    self.current_scope.add_variable(var_info);
                }

                // Анализируем тело функции
                for stmt in &func.body {
                    self.analyze_statement(stmt)?;
                }

                self.exit_scope();
            }
            Declaration::Variable(var_decl) => {
                // Добавляем переменные в текущую область видимости
                for name in &var_decl.names {
                    let var_info = VariableInfo::new(name.clone(), var_decl.location.clone());
                    self.current_scope.add_variable(var_info);
                }
            }
        }

        Ok(())
    }

    /// Анализ выражения
    fn analyze_statement(&mut self, statement: &Statement) -> Result<()> {
        match statement {
            Statement::Expression(expr) => {
                self.analyze_expression(expr)?;
            }
            Statement::Assignment(assignment) => {
                // Сначала анализируем правую часть для определения типа
                self.analyze_expression(&assignment.value)?;

                if let Expression::Identifier(name) = &assignment.target {
                    // Не проверяем ключевые слова BSL
                    if !keywords::is_bsl_reserved_word(name) {
                        // Проверяем, объявлена ли переменная
                        let var_exists = self.current_scope.get_variable(name).is_some();

                        if !var_exists && self.config.check_undeclared_variables {
                            self.diagnostics.push(Diagnostic::new(
                                DiagnosticSeverity::Error,
                                assignment.location.clone(),
                                codes::UNDECLARED_VARIABLE,
                                format!("Переменная '{}' не объявлена", name),
                            ));
                        }

                        // Выводим тип из правой части присваивания
                        if let Some(inferred_type) = self.infer_expression_type(&assignment.value) {
                            println!(
                                "🔍 Вывод типа: {} = {} (тип: {})",
                                name,
                                match &assignment.value {
                                    Expression::New(new_expr) =>
                                        format!("Новый {}()", new_expr.type_name),
                                    Expression::FunctionCall(func) => format!("{}()", func.name),
                                    Expression::Literal(_) => "литерал".to_string(),
                                    _ => "выражение".to_string(),
                                },
                                inferred_type
                            );

                            if var_exists {
                                // Обновляем существующую переменную
                                self.update_variable_type(name, inferred_type);
                            } else {
                                // Создаем новую переменную (BSL поддерживает неявное объявление)
                                let mut var_info =
                                    VariableInfo::new(name.clone(), assignment.location.clone());
                                var_info.set_inferred_type(inferred_type);
                                self.current_scope.add_variable(var_info);
                            }
                        }
                    }
                }
            }
            Statement::If(if_stmt) => {
                self.analyze_expression(&if_stmt.condition)?;
                for stmt in &if_stmt.then_branch {
                    self.analyze_statement(stmt)?;
                }
                if let Some(else_branch) = &if_stmt.else_branch {
                    for stmt in else_branch {
                        self.analyze_statement(stmt)?;
                    }
                }
            }
            Statement::For(for_stmt) => {
                // Переменная цикла
                let var_info =
                    VariableInfo::new(for_stmt.variable.clone(), for_stmt.location.clone());
                self.current_scope.add_variable(var_info);

                self.analyze_expression(&for_stmt.from)?;
                self.analyze_expression(&for_stmt.to)?;

                for stmt in &for_stmt.body {
                    self.analyze_statement(stmt)?;
                }
            }
            Statement::While(while_stmt) => {
                self.analyze_expression(&while_stmt.condition)?;
                for stmt in &while_stmt.body {
                    self.analyze_statement(stmt)?;
                }
            }
            Statement::Return(return_stmt) => {
                if let Some(expr) = &return_stmt.value {
                    self.analyze_expression(expr)?;
                }
            }
            Statement::ForEach(_) => {
                // TODO: реализовать анализ ForEach
            }
            Statement::Break | Statement::Continue => {
                // Ничего не делаем
            }
            Statement::Try(_) => {
                // TODO: реализовать анализ Try
            }
        }

        Ok(())
    }

    /// Анализ выражения
    fn analyze_expression(&mut self, expression: &Expression) -> Result<()> {
        match expression {
            Expression::Identifier(name) => {
                // Проверяем, что это не ключевое слово или встроенный тип BSL
                if !keywords::is_bsl_reserved_word(name) {
                    // Отмечаем переменную как использованную
                    self.current_scope.mark_variable_as_used(name);

                    // Проверяем инициализацию переменной
                    self.check_variable_initialization(expression);

                    // Проверяем, что переменная объявлена
                    if self.config.check_undeclared_variables
                        && self.current_scope.get_variable(name).is_none()
                    {
                        // Дополнительная проверка - не является ли это частью сложного выражения
                        if !self.is_part_of_complex_expression(name) {
                            self.diagnostics.push(Diagnostic::new(
                                DiagnosticSeverity::Warning,
                                Location::new("".to_string(), 0, 0, 0, 0), // TODO: правильная локация
                                codes::UNDECLARED_VARIABLE,
                                format!("Переменная '{}' не объявлена", name),
                            ));
                        }
                    }
                }
            }
            Expression::MethodCall(method_call) => {
                // Специальная обработка присваивания (если парсер использует метод "=" для присваивания)
                if method_call.method == "=" && method_call.args.len() == 1 {
                    // Анализируем правую часть присваивания
                    self.analyze_expression(&method_call.args[0])?;

                    // Если левая часть - это идентификатор, обрабатываем присваивание
                    if let Expression::Identifier(var_name) = &*method_call.object {
                        self.handle_assignment_inference(
                            var_name,
                            &method_call.args[0],
                            method_call.location.clone(),
                        )?;
                    }
                } else {
                    // Обычный вызов метода
                    self.analyze_expression(&method_call.object)?;
                    for arg in &method_call.args {
                        self.analyze_expression(arg)?;
                    }

                    // Проверяем вызов метода с помощью UnifiedBslIndex
                    if self.config.check_method_calls {
                        self.validate_method_call(method_call)?;
                    }
                }
            }
            Expression::FunctionCall(function_call) => {
                for arg in &function_call.args {
                    self.analyze_expression(arg)?;
                }

                // Проверяем вызов глобальной функции
                if self.config.check_method_calls {
                    self.validate_function_call(function_call)?;
                }
            }
            Expression::PropertyAccess(prop_access) => {
                self.analyze_expression(&prop_access.object)?;
            }
            Expression::Binary(binary_op) => {
                self.analyze_expression(&binary_op.left)?;
                self.analyze_expression(&binary_op.right)?;
            }
            Expression::Unary(unary_op) => {
                self.analyze_expression(&unary_op.operand)?;
            }
            Expression::Index(index_access) => {
                self.analyze_expression(&index_access.object)?;
                self.analyze_expression(&index_access.index)?;
            }
            Expression::Literal(_) => {
                // Литералы не требуют анализа
            }
            Expression::New(_) | Expression::Ternary(_) => {
                // TODO: реализовать анализ
            }
        }

        Ok(())
    }

    /// Войти в новую область видимости
    fn enter_scope(&mut self, name: String, scope_type: ScopeType) {
        let old_scope = std::mem::replace(&mut self.current_scope, Scope::new(name, scope_type));
        self.scope_stack.push(old_scope);
    }

    /// Выйти из текущей области видимости
    fn exit_scope(&mut self) {
        if let Some(parent_scope) = self.scope_stack.pop() {
            self.current_scope = parent_scope;
        }
    }

    /// Проверить неиспользованные переменные
    fn check_unused_variables(&mut self) {
        let unused = self.current_scope.get_unused_variables();
        for var in unused {
            self.diagnostics.push(Diagnostic::new(
                DiagnosticSeverity::Warning,
                var.location.clone(),
                codes::UNUSED_VARIABLE,
                format!("Переменная '{}' объявлена, но не используется", var.name),
            ));
        }
    }

    /// Найти переменную в текущей и родительских областях видимости
    fn find_variable(&self, name: &str) -> Option<&VariableInfo> {
        // Сначала ищем в текущей области
        if let Some(var) = self.current_scope.variables.get(name) {
            return Some(var);
        }

        // Затем ищем в родительских областях
        for scope in &self.scope_stack {
            if let Some(var) = scope.variables.get(name) {
                return Some(var);
            }
        }

        None
    }

    /// Получить результаты анализа
    pub fn get_diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Получить результаты в старом формате (для совместимости)
    pub fn get_results(&self) -> (Vec<AnalysisError>, Vec<AnalysisError>) {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        for diagnostic in &self.diagnostics {
            let position = Position {
                line: diagnostic.location.line,
                column: diagnostic.location.column,
                offset: diagnostic.location.offset,
            };

            let level = match diagnostic.severity {
                DiagnosticSeverity::Error => ErrorLevel::Error,
                DiagnosticSeverity::Warning => ErrorLevel::Warning,
                DiagnosticSeverity::Info | DiagnosticSeverity::Information => ErrorLevel::Info,
                DiagnosticSeverity::Hint => ErrorLevel::Hint,
            };

            let error = AnalysisError::new(
                diagnostic.message.clone(),
                diagnostic.location.file.clone().into(),
                position,
                level,
            )
            .with_code(diagnostic.code.clone());

            match diagnostic.severity {
                DiagnosticSeverity::Error => errors.push(error),
                _ => warnings.push(error),
            }
        }

        (errors, warnings)
    }

    /// Проверяет дублированные параметры
    fn check_duplicate_parameters(&mut self, params: &[Parameter]) {
        let mut seen_params = std::collections::HashSet::new();

        for param in params {
            if seen_params.contains(&param.name) {
                self.diagnostics.push(Diagnostic::new(
                    DiagnosticSeverity::Error,
                    param.location.clone(),
                    codes::DUPLICATE_PARAMETER,
                    format!("Дублированный параметр '{}'", param.name),
                ));
            } else {
                seen_params.insert(param.name.clone());
            }
        }
    }

    /// Проверяет инициализацию переменных в выражениях
    fn check_variable_initialization(&mut self, expr: &Expression) {
        match expr {
            Expression::Identifier(name) => {
                // Проверяем, что переменная была инициализирована
                if let Some(var_info) = self.find_variable(name) {
                    if !var_info.used && !var_info.is_parameter {
                        // Переменная объявлена, но может быть не инициализирована
                        // Помечаем как используемую для избежания дубликатов
                        if let Some(var) = self.current_scope.variables.get_mut(name) {
                            var.mark_used();
                        } else {
                            // Ищем в родительских областях
                            for scope in &mut self.scope_stack {
                                if let Some(var) = scope.variables.get_mut(name) {
                                    var.mark_used();
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            Expression::MethodCall(call) => {
                self.check_variable_initialization(&call.object);
                for arg in &call.args {
                    self.check_variable_initialization(arg);
                }
            }
            Expression::PropertyAccess(access) => {
                self.check_variable_initialization(&access.object);
            }
            Expression::New(new_expr) => {
                for arg in &new_expr.args {
                    self.check_variable_initialization(arg);
                }
            }
            _ => {}
        }
    }

    /// Проверяет неинициализированные переменные в теле функций/процедур
    #[allow(dead_code)]
    fn check_uninitialized_variables(&mut self) {
        for (name, var_info) in &self.current_scope.variables {
            if !var_info.is_parameter && !var_info.used {
                // Переменная объявлена, но ни разу не использована (может быть неинициализирована)
                if self.config.check_uninitialized_variables {
                    self.diagnostics.push(Diagnostic::new(
                        DiagnosticSeverity::Warning,
                        var_info.location.clone(),
                        codes::UNINITIALIZED_VARIABLE,
                        format!("Переменная '{}' может быть неинициализирована", name),
                    ));
                }
            }
        }
    }

    /// Проверяет, является ли имя частью сложного выражения (условных операторов, циклов и т.д.)
    fn is_part_of_complex_expression(&self, name: &str) -> bool {
        // Проверяем составные операторы BSL которые могут быть ошибочно разобраны как переменные
        let complex_patterns = [
            "НЕ ДанныеПользователя",
            "НЕ ДанныеПользователей",
            "ИначеЕсли РезультатОбработки",
            "Для Каждого Язык Из Метаданные",
        ];

        complex_patterns
            .iter()
            .any(|pattern| pattern.contains(name))
    }

    /// Валидирует вызов метода с помощью UnifiedBslIndex
    fn validate_method_call(&mut self, method_call: &MethodCall) -> Result<()> {
        if let Some(index) = &self.index {
            // Определяем тип объекта
            let object_type = self.infer_expression_type(&method_call.object);

            // Отладочный вывод для понимания структуры вызова метода
            println!(
                "🔍 DEBUG MethodCall: object={:?}, method={}, object_type={:?}",
                method_call.object, method_call.method, object_type
            );

            // ОБХОДНОЙ ПУТЬ: Обрабатываем составные методы типа "Пользователи.СоздатьЭлемент"
            if method_call.method.contains('.') && object_type.is_some() {
                let object_type_name = object_type.as_ref().unwrap();
                println!(
                    "🔧 WORKAROUND: Обрабатываем составной метод: {} для {}",
                    method_call.method, object_type_name
                );

                // Разбираем составной метод на части (например, "Пользователи.СоздатьЭлемент")
                let parts: Vec<&str> = method_call.method.split('.').collect();
                if parts.len() == 2 {
                    let property_name = parts[0];
                    let method_name = parts[1];

                    // НОВАЯ ЛОГИКА: Прямой поиск конкретных объектов конфигурации
                    // Вместо поиска шаблонных типов, ищем точные объекты: Справочники.Пользователи, Документы.Заказ и т.д.
                    if let Some(manager_info) = self.parse_manager_type(object_type_name) {
                        // Формируем конкретный тип: manager_info.base_type заменяем на полное имя объекта
                        let concrete_type = format!("{}.{}", object_type_name, property_name);
                        println!(
                            "🔧 ИСПРАВЛЕНО: Поиск конкретного объекта {}: {}",
                            manager_info.kind, concrete_type
                        );

                        // Проверяем существование конкретного объекта в конфигурации
                        if let Some(entity) = index.find_entity(&concrete_type) {
                            let all_methods = index.get_all_methods(&entity.qualified_name);
                            println!(
                                "🔍 DEBUG: Найден объект {}, методов: {}",
                                concrete_type,
                                all_methods.len()
                            );

                            // Проверяем наличие метода по короткому имени и по полному имени
                            let method_found = all_methods.contains_key(method_name)
                                || all_methods.keys().any(|full_name| {
                                    full_name.ends_with(&format!(".{}", method_name))
                                });

                            if method_found {
                                println!(
                                    "✅ ИСПРАВЛЕНО: Метод {} найден в конкретном объекте {}",
                                    method_name, concrete_type
                                );
                                return Ok(()); // Метод найден - всё в порядке
                            } else {
                                println!(
                                    "❌ ИСПРАВЛЕНО: Метод {} НЕ найден в конкретном объекте {}",
                                    method_name, concrete_type
                                );
                                // Выдаём корректную ошибку
                                self.diagnostics.push(
                                    Diagnostic::new(
                                        DiagnosticSeverity::Error,
                                        method_call.location.clone(),
                                        codes::UNKNOWN_METHOD,
                                        format!(
                                            "Метод '{}' не найден для объекта '{}'",
                                            method_name, concrete_type
                                        ),
                                    )
                                    .with_found(method_name)
                                    .with_expected(format!(
                                        "доступные методы объекта {}",
                                        concrete_type
                                    )),
                                );
                                return Ok(()); // Обработали ошибку
                            }
                        } else {
                            println!(
                                "❌ ИСПРАВЛЕНО: Конкретный объект {} не найден в конфигурации",
                                concrete_type
                            );
                            // Объект не найден в конфигурации
                            self.diagnostics.push(Diagnostic::new(
                                DiagnosticSeverity::Warning,
                                method_call.location.clone(),
                                codes::UNKNOWN_CONSTRUCT,
                                format!(
                                    "{} '{}' не найден в конфигурации",
                                    manager_info.kind, property_name
                                ),
                            ));
                            return Ok(()); // Обработали ошибку
                        }
                    }
                }
            }

            if let Some(type_name) = object_type {
                // Ищем тип в индексе
                if let Some(entity) = index.find_entity(&type_name) {
                    println!(
                        "🔍 DEBUG найден entity: qualified_name='{}', display_name='{}'",
                        entity.qualified_name, entity.display_name
                    );

                    // Получаем все методы (включая унаследованные)
                    let all_methods = index.get_all_methods(&entity.qualified_name);
                    println!(
                        "🔍 DEBUG методы для {}: {:?}",
                        entity.qualified_name,
                        all_methods.keys().collect::<Vec<_>>()
                    );

                    // Проверяем наличие метода по короткому имени и по полному имени
                    let method_found = all_methods.contains_key(&method_call.method)
                        || all_methods.keys().any(|full_name| {
                            full_name.ends_with(&format!(".{}", &method_call.method))
                        });

                    if !method_found {
                        // Метод не найден
                        self.diagnostics.push(
                            Diagnostic::new(
                                DiagnosticSeverity::Error,
                                method_call.location.clone(),
                                codes::UNKNOWN_METHOD,
                                format!(
                                    "Метод '{}' не найден для типа '{}'",
                                    method_call.method, type_name
                                ),
                            )
                            .with_found(&method_call.method)
                            .with_expected(
                                all_methods
                                    .keys()
                                    .take(5)
                                    .cloned()
                                    .collect::<Vec<_>>()
                                    .join(", "),
                            ),
                        );
                    } else {
                        // Метод найден - проверяем количество параметров
                        if let Some(method) = all_methods.get(&method_call.method) {
                            let expected_params = method.parameters.len();
                            let actual_params = method_call.args.len();

                            if expected_params != actual_params {
                                self.diagnostics.push(
                                    Diagnostic::new(
                                        DiagnosticSeverity::Error,
                                        method_call.location.clone(),
                                        codes::WRONG_PARAM_COUNT,
                                        format!(
                                            "Метод '{}' ожидает {} параметров, получено {}",
                                            method_call.method, expected_params, actual_params
                                        ),
                                    )
                                    .with_found(actual_params.to_string())
                                    .with_expected(expected_params.to_string()),
                                );
                            }
                        }
                    }
                } else {
                    // Тип не найден в индексе
                    self.diagnostics.push(Diagnostic::new(
                        DiagnosticSeverity::Warning,
                        method_call.location.clone(),
                        codes::UNKNOWN_CONSTRUCT,
                        format!("Неизвестный тип '{}'", type_name),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Валидирует вызов функции (глобальной или локальной)
    fn validate_function_call(&mut self, function_call: &FunctionCall) -> Result<()> {
        // 1. Сначала проверяем, является ли это глобальной функцией BSL
        if keywords::is_bsl_global_function(&function_call.name) {
            return Ok(()); // Глобальная функция найдена
        }

        // 2. Затем проверяем локальные функции в текущем модуле
        if self.is_local_function_defined(&function_call.name) {
            return Ok(()); // Локальная функция найдена
        }

        // 3. Только если функция не найдена нигде - выдаем ошибку
        self.diagnostics.push(
            Diagnostic::new(
                DiagnosticSeverity::Error,
                function_call.location.clone(),
                codes::UNKNOWN_METHOD,
                format!(
                    "Функция '{}' не найдена ни в глобальном контексте, ни в текущем модуле",
                    function_call.name
                ),
            )
            .with_found(&function_call.name)
            .with_expected("список доступных функций или определение в модуле"),
        );

        Ok(())
    }

    /// Собирает все локальные функции и процедуры модуля
    fn collect_local_functions(&mut self, module: &Module) {
        for declaration in &module.declarations {
            match declaration {
                Declaration::Function(func) => {
                    self.local_functions
                        .insert(func.name.clone(), func.location.clone());
                }
                Declaration::Procedure(proc) => {
                    self.local_functions
                        .insert(proc.name.clone(), proc.location.clone());
                }
                Declaration::Variable(_) => {
                    // Обрабатываем объявления переменных
                }
            }
        }
    }

    /// Проверяет, определена ли локальная функция в текущем модуле
    fn is_local_function_defined(&self, function_name: &str) -> bool {
        self.local_functions.contains_key(function_name)
    }

    /// Обновляет тип переменной в текущей или родительских областях
    fn update_variable_type(&mut self, name: &str, inferred_type: String) {
        // Сначала проверяем текущую область
        if let Some(var) = self.current_scope.variables.get_mut(name) {
            var.set_inferred_type(inferred_type);
            return;
        }

        // Если не найдена в текущей области, ищем в стеке областей
        for scope in &mut self.scope_stack {
            if let Some(var) = scope.variables.get_mut(name) {
                var.set_inferred_type(inferred_type);
                return;
            }
        }
    }

    /// Обрабатывает логику вывода типов из присваиваний
    fn handle_assignment_inference(
        &mut self,
        var_name: &str,
        value_expr: &Expression,
        location: Location,
    ) -> Result<()> {
        // Не проверяем ключевые слова BSL
        if !keywords::is_bsl_reserved_word(var_name) {
            // Проверяем, объявлена ли переменная
            let var_exists = self.current_scope.get_variable(var_name).is_some();

            if !var_exists && self.config.check_undeclared_variables {
                self.diagnostics.push(Diagnostic::new(
                    DiagnosticSeverity::Error,
                    location.clone(),
                    codes::UNDECLARED_VARIABLE,
                    format!("Переменная '{}' не объявлена", var_name),
                ));
            }

            // Выводим тип из правой части присваивания
            if let Some(inferred_type) = self.infer_expression_type(value_expr) {
                println!(
                    "🔍 Вывод типа: {} = {} (тип: {})",
                    var_name,
                    match value_expr {
                        Expression::New(new_expr) => format!("Новый {}()", new_expr.type_name),
                        Expression::FunctionCall(func) => format!("{}()", func.name),
                        Expression::Literal(_) => "литерал".to_string(),
                        _ => "выражение".to_string(),
                    },
                    inferred_type
                );

                if var_exists {
                    // Обновляем существующую переменную
                    self.update_variable_type(var_name, inferred_type);
                } else {
                    // Создаем новую переменную (BSL поддерживает неявное объявление)
                    let mut var_info = VariableInfo::new(var_name.to_string(), location);
                    var_info.set_inferred_type(inferred_type);
                    self.current_scope.add_variable(var_info);
                }
            }
        }

        Ok(())
    }

    /// Парсит тип менеджера и возвращает информацию о нём
    fn parse_manager_type(&self, type_name: &str) -> Option<ManagerInfo> {
        // Определяем паттерны менеджеров 1С
        let manager_patterns = [
            // (паттерн для поиска, базовый тип, вид именительный)
            ("Справочники", "СправочникМенеджер", "Справочник"), // Глобальный алиас
            ("CatalogsManager", "СправочникМенеджер", "Справочник"),
            ("СправочникиМенеджер", "СправочникМенеджер", "Справочник"),
            ("DocumentsManager", "ДокументМенеджер", "Документ"),
            ("МенеджерДокументов", "ДокументМенеджер", "Документ"),
            (
                "InformationRegistersManager",
                "РегистрСведенийМенеджер",
                "Регистр сведений",
            ),
            (
                "МенеджерРегистровСведений",
                "РегистрСведенийМенеджер",
                "Регистр сведений",
            ),
            (
                "AccumulationRegistersManager",
                "РегистрНакопленияМенеджер",
                "Регистр накопления",
            ),
            (
                "МенеджерРегистровНакопления",
                "РегистрНакопленияМенеджер",
                "Регистр накопления",
            ),
            ("DataProcessorsManager", "ОбработкаМенеджер", "Обработка"),
            ("МенеджерОбработок", "ОбработкаМенеджер", "Обработка"),
            ("ReportsManager", "ОтчетМенеджер", "Отчет"),
            ("МенеджерОтчетов", "ОтчетМенеджер", "Отчет"),
            ("ConstantsManager", "КонстантаМенеджер", "Константа"),
            ("МенеджерКонстант", "КонстантаМенеджер", "Константа"),
        ];

        for (pattern, base_type, kind) in &manager_patterns {
            if type_name.contains(pattern) {
                return Some(ManagerInfo {
                    base_type: base_type.to_string(),
                    kind: kind.to_string(),
                });
            }
        }

        None
    }

    /// Выводит тип выражения (улучшенная реализация с поддержкой вывода типов)
    fn infer_expression_type(&self, expr: &Expression) -> Option<String> {
        match expr {
            Expression::Identifier(name) => {
                // Проверяем переменные в области видимости
                if let Some(var_info) = self.find_variable(name) {
                    // Возвращаем наиболее точный известный тип
                    return var_info.get_effective_type().cloned();
                }

                // ВАЖНО: Сначала проверяем глобальные алиасы, потом ключевые слова BSL
                // Проверяем глобальные алиасы через UnifiedBslIndex
                if let Some(index) = &self.index {
                    println!("🔍 DEBUG Ищем identifier: {}", name);
                    if let Some(entity) = index.find_entity(name) {
                        println!("🔍 DEBUG Identifier: {} -> {}", name, entity.qualified_name);
                        return Some(entity.qualified_name.clone());
                    } else {
                        println!("🔍 DEBUG Identifier {} НЕ найден в индексе", name);
                    }
                }

                // Если переменная не найдена в алиасах, проверяем ключевые слова BSL
                if keywords::is_bsl_reserved_word(name) {
                    println!("🔍 DEBUG {} is BSL reserved word", name);
                    return Some(name.clone());
                }

                None
            }
            Expression::New(new_expr) => {
                // Тип объекта создается из конструктора
                println!("🔍 DEBUG New expression: {}", new_expr.type_name);
                Some(new_expr.type_name.clone())
            }
            Expression::MethodCall(method_call) => {
                // Для вызовов методов нужно определить возвращаемый тип
                if let Some(index) = &self.index {
                    if let Some(object_type) = self.infer_expression_type(&method_call.object) {
                        if let Some(entity) = index.find_entity(&object_type) {
                            let all_methods = index.get_all_methods(&entity.qualified_name);
                            if let Some(method) = all_methods.get(&method_call.method) {
                                return method.return_type.clone();
                            }
                        }
                    }
                }
                None
            }
            Expression::FunctionCall(function_call) => {
                // Для глобальных функций тоже можно определить возвращаемый тип
                if let Some(index) = &self.index {
                    // Поиск в глобальных функциях
                    for entity in index.get_all_entities() {
                        if let Some(methods) = entity.interface.methods.get(&function_call.name) {
                            return methods.return_type.clone();
                        }
                    }
                }
                None
            }
            Expression::Literal(lit) => match lit {
                Literal::String(_) => Some("Строка".to_string()),
                Literal::Number(_) => Some("Число".to_string()),
                Literal::Boolean(_) => Some("Булево".to_string()),
                Literal::Date(_) => Some("Дата".to_string()),
                Literal::Undefined => Some("Неопределено".to_string()),
                Literal::Null => Some("Null".to_string()),
            },
            Expression::PropertyAccess(prop_access) => {
                // Обрабатываем составные выражения типа Справочники.Пользователи
                println!(
                    "🔍 DEBUG PropertyAccess: object={:?}, property={}",
                    prop_access.object, prop_access.property
                );

                if let Some(index) = &self.index {
                    if let Some(object_type) = self.infer_expression_type(&prop_access.object) {
                        println!("🔍 DEBUG PropertyAccess object_type: {}", object_type);

                        // Универсальная обработка менеджеров 1С
                        if let Some(manager_info) = self.parse_manager_type(&object_type) {
                            // Менеджер.Свойство -> БазовыйТип.Свойство
                            let target_type =
                                format!("{}.{}", manager_info.base_type, prop_access.property);
                            println!(
                                "🔍 DEBUG PropertyAccess manager: {} -> {}",
                                object_type, target_type
                            );
                            return Some(target_type);
                        }

                        // Стандартная обработка свойств
                        if let Some(entity) = index.find_entity(&object_type) {
                            if let Some(property) =
                                entity.interface.properties.get(&prop_access.property)
                            {
                                return Some(property.type_name.clone());
                            }
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new(SemanticAnalysisConfig::default())
    }
}
