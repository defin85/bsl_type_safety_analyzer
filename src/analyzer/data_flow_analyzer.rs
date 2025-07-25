/*!
# Data Flow Analyzer

Анализатор потоков данных для отслеживания переменных, их инициализации и использования в BSL коде.
*/

use std::collections::HashMap;
use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::parser::ast::{AstNode, AstNodeType};
use crate::diagnostics::Diagnostic;
use super::AnalysisContext;

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
}

/// Анализатор потоков данных для отслеживания переменных
pub struct DataFlowAnalyzer {
    /// Состояния всех переменных
    variable_states: HashMap<String, VariableState>,
    /// Собранные диагностики
    diagnostics: Vec<Diagnostic>,
    /// Текущий контекст анализа
    current_file: Option<String>,
}

impl DataFlowAnalyzer {
    /// Создает новый анализатор потоков данных
    pub fn new() -> Self {
        Self {
            variable_states: HashMap::new(),
            diagnostics: Vec::new(),
            current_file: None,
        }
    }

    /// Выполняет анализ потоков данных
    pub fn analyze(&mut self, context: &mut AnalysisContext) -> Result<()> {
        self.clear_results();
        self.current_file = Some(context.file_path.clone());

        if let Some(ref ast) = context.ast {
            // Анализируем потоки данных
            self.analyze_data_flow(ast)?;
            
            // Проверяем потенциальные проблемы
            self.check_potential_issues();
            
            // Добавляем диагностики в контекст
            for diagnostic in &self.diagnostics {
                context.diagnostics.push(diagnostic.clone());
            }
        } else {
            self.add_warning("AST отсутствует, пропускаем анализ потоков данных", 0);
        }

        Ok(())
    }

    /// Очищает результаты предыдущего анализа
    fn clear_results(&mut self) {
        self.variable_states.clear();
        self.diagnostics.clear();
    }

    /// Анализирует потоки данных в AST
    fn analyze_data_flow(&mut self, ast: &AstNode) -> Result<()> {
        // Находим все объявления переменных
        self.find_variable_declarations(ast)?;
        
        // Анализируем использование переменных
        self.analyze_variable_usage(ast)?;
        
        // Проверяем инициализацию переменных
        self.check_variable_initialization();

        Ok(())
    }

    /// Находит все объявления переменных в AST
    fn find_variable_declarations(&mut self, ast: &AstNode) -> Result<()> {
        match ast.node_type {
            AstNodeType::VariableDeclaration => {
                self.process_variable_declaration(ast)?;
            }
            AstNodeType::Parameter => {
                self.process_parameter_declaration(ast)?;
            }
            _ => {
                // Рекурсивно обрабатываем дочерние узлы
                for child in &ast.children {
                    self.find_variable_declarations(child)?;
                }
            }
        }
        Ok(())
    }

    /// Обрабатывает объявление переменной
    fn process_variable_declaration(&mut self, node: &AstNode) -> Result<()> {
        if let Some(name) = &node.value {
            let line = node.span.start.line;
            let state = VariableState::new(line, false);
            self.variable_states.insert(name.clone(), state);
        }
        Ok(())
    }

    /// Обрабатывает объявление параметра
    fn process_parameter_declaration(&mut self, node: &AstNode) -> Result<()> {
        if let Some(name) = &node.value {
            let line = node.span.start.line;
            let state = VariableState::new(line, true);
            self.variable_states.insert(name.clone(), state);
        }
        Ok(())
    }

    /// Анализирует использование переменных в AST
    fn analyze_variable_usage(&mut self, ast: &AstNode) -> Result<()> {
        match ast.node_type {
            AstNodeType::Identifier => {
                self.process_variable_usage(ast)?;
            }
            AstNodeType::Assignment => {
                self.process_assignment(ast)?;
            }
            _ => {
                // Рекурсивно обрабатываем дочерние узлы
                for child in &ast.children {
                    self.analyze_variable_usage(child)?;
                }
            }
        }
        Ok(())
    }

    /// Обрабатывает использование переменной
    fn process_variable_usage(&mut self, node: &AstNode) -> Result<()> {
        if let Some(name) = &node.value {
            let line = node.span.start.line;
            
            if let Some(var_state) = self.variable_states.get_mut(name) {
                var_state.mark_used(line);
            } else {
                // Переменная не объявлена
                self.add_error(
                    &format!("Переменная '{}' используется, но не объявлена", name),
                    line,
                );
            }
        }
        Ok(())
    }

    /// Обрабатывает присваивание
    fn process_assignment(&mut self, node: &AstNode) -> Result<()> {
        // Анализируем левую часть (переменная для присваивания)
        if let Some(left_child) = node.children.first() {
            if left_child.node_type == AstNodeType::Identifier {
                if let Some(var_name) = &left_child.value {
                    let line = left_child.span.start.line;
                    
                    if let Some(var_state) = self.variable_states.get_mut(var_name) {
                        var_state.mark_initialized();
                        // НЕ помечаем как используемую - присваивание не является использованием
                    } else {
                        // Переменная не объявлена
                        self.add_error(
                            &format!("Переменная '{}' используется в присваивании, но не объявлена", var_name),
                            line,
                        );
                    }
                }
            }
        }

        // Анализируем правую часть (выражение присваивания)
        // Пропускаем первый дочерний элемент (левая часть), анализируем остальные
        for (i, child) in node.children.iter().enumerate() {
            if i == 0 {
                continue; // Пропускаем левую часть
            }
            self.analyze_variable_usage(child)?;
        }

        Ok(())
    }

    /// Проверяет инициализацию переменных
    fn check_variable_initialization(&mut self) {
        let uninitialized_vars: Vec<(String, usize)> = self.variable_states
            .iter()
            .filter(|(_, state)| state.declared && !state.initialized && !state.is_parameter)
            .map(|(name, state)| (name.clone(), state.declaration_line))
            .collect();

        for (var_name, line) in uninitialized_vars {
            self.add_warning(
                &format!("Переменная '{}' объявлена, но может быть не инициализирована", var_name),
                line,
            );
        }
    }

    /// Проверяет потенциальные проблемы в потоках данных
    fn check_potential_issues(&mut self) {
        // Проверяем неиспользуемые переменные
        let unused_vars: Vec<(String, usize)> = self.variable_states
            .iter()
            .filter(|(_, state)| state.declared && !state.used && !state.is_parameter)
            .map(|(name, state)| (name.clone(), state.declaration_line))
            .collect();

        for (var_name, line) in unused_vars {
            self.add_warning(
                &format!("Переменная '{}' объявлена, но не используется", var_name),
                line,
            );
        }
    }

    /// Возвращает состояния всех переменных
    pub fn get_variable_states(&self) -> &HashMap<String, VariableState> {
        &self.variable_states
    }

    /// Возвращает список неинициализированных переменных
    pub fn get_uninitialized_variables(&self) -> Vec<String> {
        self.variable_states
            .iter()
            .filter(|(_, state)| state.declared && !state.initialized && !state.is_parameter)
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Возвращает список неиспользуемых переменных
    pub fn get_unused_variables(&self) -> Vec<String> {
        self.variable_states
            .iter()
            .filter(|(_, state)| state.declared && !state.used && !state.is_parameter)
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Добавляет ошибку в диагностики
    fn add_error(&mut self, message: &str, line: usize) {
        let diagnostic = Diagnostic::error(
            message.to_string(),
            line,
            0
        );
        self.diagnostics.push(diagnostic);
    }

    /// Добавляет предупреждение в диагностики
    fn add_warning(&mut self, message: &str, line: usize) {
        let diagnostic = Diagnostic::warning(
            message.to_string(),
            line,
            0
        );
        self.diagnostics.push(diagnostic);
    }

    /// Возвращает собранные диагностики
    pub fn get_diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
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
    use crate::parser::ast::{Position, Span};
    use std::collections::HashMap;

    /// Создает тестовый узел AST
    fn create_test_node(node_type: AstNodeType, value: Option<String>, line: usize) -> AstNode {
        AstNode {
            node_type,
            span: Span {
                start: Position { line, column: 0, offset: 0 },
                end: Position { line, column: 0, offset: 0 },
            },
            value,
            attributes: HashMap::new(),
            children: Vec::new(),
        }
    }

    #[test]
    fn test_variable_declaration() {
        let mut analyzer = DataFlowAnalyzer::new();
        let var_node = create_test_node(
            AstNodeType::VariableDeclaration,
            Some("TestVar".to_string()),
            1
        );

        analyzer.process_variable_declaration(&var_node).unwrap();

        let states = analyzer.get_variable_states();
        assert!(states.contains_key("TestVar"));
        let state = &states["TestVar"];
        assert!(state.declared);
        assert!(!state.initialized);
        assert!(!state.used);
        assert!(!state.is_parameter);
    }

    #[test]
    fn test_parameter_declaration() {
        let mut analyzer = DataFlowAnalyzer::new();
        let param_node = create_test_node(
            AstNodeType::Parameter,
            Some("Param1".to_string()),
            1
        );

        analyzer.process_parameter_declaration(&param_node).unwrap();

        let states = analyzer.get_variable_states();
        assert!(states.contains_key("Param1"));
        let state = &states["Param1"];
        assert!(state.declared);
        assert!(state.initialized); // Параметры считаются инициализированными
        assert!(!state.used);
        assert!(state.is_parameter);
    }

    #[test]
    fn test_uninitialized_variables() {
        let mut analyzer = DataFlowAnalyzer::new();
        
        // Объявляем переменную
        let var_state = VariableState::new(1, false);
        analyzer.variable_states.insert("UnusedVar".to_string(), var_state);

        let uninitialized = analyzer.get_uninitialized_variables();
        assert_eq!(uninitialized.len(), 1);
        assert_eq!(uninitialized[0], "UnusedVar");
    }

    #[test]
    fn test_unused_variables() {
        let mut analyzer = DataFlowAnalyzer::new();
        
        // Объявляем и инициализируем переменную, но не используем
        let mut var_state = VariableState::new(1, false);
        var_state.mark_initialized();
        analyzer.variable_states.insert("UnusedVar".to_string(), var_state);

        let unused = analyzer.get_unused_variables();
        assert_eq!(unused.len(), 1);
        assert_eq!(unused[0], "UnusedVar");
    }
}
