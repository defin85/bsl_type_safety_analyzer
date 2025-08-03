//! Data Flow Analyzer для tree-sitter парсера
//! 
//! Анализатор потоков данных для отслеживания переменных, их инициализации 
//! и использования в BSL коде.

use std::collections::HashMap;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::bsl_parser::{ast::*, diagnostics::*};

/// Состояние переменной в потоке данных
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableState {
    /// Переменная объявлена
    pub declared: bool,
    /// Переменная инициализирована
    pub initialized: bool,
    /// Переменная использована
    pub used: bool,
    /// Строка объявления переменной
    pub declaration_line: usize,
    /// Строка первого использования
    pub first_use_line: Option<usize>,
    /// Строка последнего использования
    pub last_use_line: Option<usize>,
    /// Является ли параметром функции/процедуры
    pub is_parameter: bool,
}

impl VariableState {
    /// Создает новое состояние переменной
    pub fn new(declaration_line: usize, is_parameter: bool) -> Self {
        Self {
            declared: true,
            initialized: is_parameter, // Параметры считаются инициализированными
            used: false,
            declaration_line,
            first_use_line: None,
            last_use_line: None,
            is_parameter,
        }
    }

    /// Отмечает переменную как использованную
    pub fn mark_used(&mut self, line: usize) {
        self.used = true;
        if self.first_use_line.is_none() {
            self.first_use_line = Some(line);
        }
        self.last_use_line = Some(line);
    }

    /// Отмечает переменную как инициализированную
    pub fn mark_initialized(&mut self) {
        self.initialized = true;
    }

    /// Проверяет, была ли переменная использована до инициализации
    pub fn is_used_before_initialized(&self) -> bool {
        self.used && !self.initialized
    }

    /// Проверяет, является ли переменная неиспользованной
    pub fn is_unused(&self) -> bool {
        !self.used && !self.is_parameter
    }
}

/// Анализатор потоков данных
pub struct DataFlowAnalyzer {
    /// Состояния переменных
    variables: HashMap<String, VariableState>,
    /// Диагностики
    diagnostics: Vec<Diagnostic>,
}

impl DataFlowAnalyzer {
    /// Создает новый анализатор потоков данных
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            diagnostics: Vec::new(),
        }
    }

    /// Анализирует AST
    pub fn analyze(&mut self, ast: &BslAst) -> Result<()> {
        self.variables.clear();
        self.diagnostics.clear();
        
        self.analyze_module(&ast.module)?;
        self.check_variable_usage();
        
        Ok(())
    }

    /// Анализирует модуль
    fn analyze_module(&mut self, module: &Module) -> Result<()> {
        for declaration in &module.declarations {
            self.analyze_declaration(declaration)?;
        }
        Ok(())
    }

    /// Анализирует объявление
    fn analyze_declaration(&mut self, declaration: &Declaration) -> Result<()> {
        match declaration {
            Declaration::Procedure(proc) => {
                // Добавляем параметры
                for param in &proc.params {
                    let state = VariableState::new(param.location.line, true);
                    self.variables.insert(param.name.clone(), state);
                }
                
                // Анализируем тело процедуры
                for stmt in &proc.body {
                    self.analyze_statement(stmt)?;
                }
            }
            Declaration::Function(func) => {
                // Добавляем параметры
                for param in &func.params {
                    let state = VariableState::new(param.location.line, true);
                    self.variables.insert(param.name.clone(), state);
                }
                
                // Анализируем тело функции
                for stmt in &func.body {
                    self.analyze_statement(stmt)?;
                }
            }
            Declaration::Variable(var_decl) => {
                // Добавляем переменные
                for name in &var_decl.names {
                    let state = VariableState::new(var_decl.location.line, false);
                    self.variables.insert(name.clone(), state);
                }
            }
        }
        
        Ok(())
    }

    /// Анализирует выражение
    fn analyze_statement(&mut self, statement: &Statement) -> Result<()> {
        match statement {
            Statement::Expression(expr) => {
                self.analyze_expression(expr)?;
            }
            Statement::Assignment(assignment) => {
                // Отмечаем переменную как инициализированную
                if let Expression::Identifier(name) = &assignment.target {
                    if let Some(state) = self.variables.get_mut(name) {
                        state.mark_initialized();
                    } else {
                        // Переменная не объявлена, добавляем как неявную
                        let mut state = VariableState::new(assignment.location.line, false);
                        state.mark_initialized();
                        self.variables.insert(name.clone(), state);
                    }
                }
                
                // Анализируем правую часть
                self.analyze_expression(&assignment.value)?;
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
                let state = VariableState::new(for_stmt.location.line, false);
                self.variables.insert(for_stmt.variable.clone(), state);
                
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

    /// Анализирует выражение
    fn analyze_expression(&mut self, expression: &Expression) -> Result<()> {
        match expression {
            Expression::Identifier(name) => {
                // Отмечаем переменную как использованную
                if let Some(state) = self.variables.get_mut(name) {
                    state.mark_used(0); // TODO: получить правильную строку
                    
                    // Проверяем использование до инициализации
                    if state.is_used_before_initialized() {
                        self.diagnostics.push(
                            Diagnostic::new(
                                DiagnosticSeverity::Warning,
                                Location::new("".to_string(), 0, 0, 0, 0), // TODO: правильная локация
                                codes::UNINITIALIZED_VARIABLE,
                                format!("Переменная '{}' используется до инициализации", name),
                            )
                        );
                    }
                }
            }
            Expression::MethodCall(method_call) => {
                self.analyze_expression(&method_call.object)?;
                for arg in &method_call.args {
                    self.analyze_expression(arg)?;
                }
            }
            Expression::FunctionCall(function_call) => {
                for arg in &function_call.args {
                    self.analyze_expression(arg)?;
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

    /// Проверяет использование переменных
    fn check_variable_usage(&mut self) {
        for (name, state) in &self.variables {
            if state.is_unused() {
                self.diagnostics.push(
                    Diagnostic::new(
                        DiagnosticSeverity::Warning,
                        Location::new("".to_string(), state.declaration_line, 0, 0, 0),
                        codes::UNUSED_VARIABLE,
                        format!("Переменная '{}' объявлена, но не используется", name),
                    )
                );
            }
        }
    }

    /// Получает диагностики
    pub fn get_diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Получает состояния переменных
    pub fn get_variable_states(&self) -> &HashMap<String, VariableState> {
        &self.variables
    }

    /// Получает состояние конкретной переменной
    pub fn get_variable_state(&self, name: &str) -> Option<&VariableState> {
        self.variables.get(name)
    }
}

impl Default for DataFlowAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_state_creation() {
        let state = VariableState::new(1, false);
        assert!(state.declared);
        assert!(!state.initialized);
        assert!(!state.used);
        assert_eq!(state.declaration_line, 1);
        assert!(!state.is_parameter);
    }

    #[test]
    fn test_parameter_state() {
        let state = VariableState::new(1, true);
        assert!(state.declared);
        assert!(state.initialized); // Параметры инициализированы
        assert!(!state.used);
        assert!(state.is_parameter);
    }

    #[test]
    fn test_mark_used() {
        let mut state = VariableState::new(1, false);
        state.mark_used(5);
        
        assert!(state.used);
        assert_eq!(state.first_use_line, Some(5));
        assert_eq!(state.last_use_line, Some(5));
    }

    #[test]
    fn test_analyzer_creation() {
        let analyzer = DataFlowAnalyzer::new();
        assert!(analyzer.variables.is_empty());
        assert!(analyzer.diagnostics.is_empty());
    }
}