/*!
# Data Flow Analyzer Integration Tests

Integration tests for the BSL data flow analyzer with comprehensive variable tracking,
initialization checking, and usage analysis.
*/

#[cfg(test)]
mod tests {
    use crate::analyzer::{DataFlowAnalyzer, AnalysisContext};
    use crate::parser::ast::{AstNode, AstNodeType, Position, Span};
    use std::collections::HashMap;

    fn create_span(line: usize, column: usize) -> Span {
        let start = Position::new(line, column, 0);
        let end = Position::new(line, column + 1, 0);
        Span::new(start, end)
    }
    
    fn create_test_node(node_type: AstNodeType, value: Option<String>, line: usize) -> AstNode {
        AstNode {
            node_type,
            span: create_span(line, 0),
            value,
            attributes: HashMap::new(),
            children: Vec::new(),
        }
    }
    
    fn create_module_with_variables() -> AstNode {
        let mut module = AstNode::new(AstNodeType::Module, create_span(1, 1));
        
        // Объявление переменной: Перем TestVar;
        let var_decl = create_test_node(
            AstNodeType::VariableDeclaration,
            Some("TestVar".to_string()),
            2
        );
        module.add_child(var_decl);
        
        // Присваивание: TestVar = "Значение";
        let mut assignment = create_test_node(AstNodeType::Assignment, None, 3);
        let identifier = create_test_node(
            AstNodeType::Identifier,
            Some("TestVar".to_string()),
            3
        );
        assignment.add_child(identifier);
        module.add_child(assignment);
        
        // Использование переменной
        let usage = create_test_node(
            AstNodeType::Identifier,
            Some("TestVar".to_string()),
            4
        );
        module.add_child(usage);
        
        module
    }
    
    fn create_procedure_with_parameters() -> AstNode {
        let mut procedure = create_test_node(
            AstNodeType::Procedure,
            Some("TestProcedure".to_string()),
            1
        );
        
        // Параметр процедуры
        let param = create_test_node(
            AstNodeType::Parameter,
            Some("Param1".to_string()),
            1
        );
        procedure.add_child(param);
        
        // Локальная переменная
        let local_var = create_test_node(
            AstNodeType::VariableDeclaration,
            Some("LocalVar".to_string()),
            2
        );
        procedure.add_child(local_var);
        
        // Использование параметра
        let param_usage = create_test_node(
            AstNodeType::Identifier,
            Some("Param1".to_string()),
            3
        );
        procedure.add_child(param_usage);
        
        procedure
    }
    
    fn create_undeclared_variable_usage() -> AstNode {
        let mut module = AstNode::new(AstNodeType::Module, create_span(1, 1));
        
        // Использование необъявленной переменной
        let undeclared_usage = create_test_node(
            AstNodeType::Identifier,
            Some("UndeclaredVar".to_string()),
            2
        );
        module.add_child(undeclared_usage);
        
        module
    }
    
    #[test]
    fn test_data_flow_analyzer_creation() {
        let analyzer = DataFlowAnalyzer::new();
        assert!(analyzer.get_variable_states().is_empty());
        assert!(analyzer.get_diagnostics().is_empty());
    }
    
    #[test]
    fn test_variable_declaration_and_usage() {
        let mut analyzer = DataFlowAnalyzer::new();
        let mut context = AnalysisContext::new(
            "test.bsl".to_string(),
            "Перем TestVar; TestVar = \"Значение\"; Сообщить(TestVar);".to_string()
        );
        
        let ast = create_module_with_variables();
        context.ast = Some(ast);
        
        let result = analyzer.analyze(&mut context);
        assert!(result.is_ok());
        
        let states = analyzer.get_variable_states();
        assert!(states.contains_key("TestVar"));
        
        let state = &states["TestVar"];
        assert!(state.declared, "Variable should be declared");
        assert!(state.initialized, "Variable should be initialized");
        assert!(state.used, "Variable should be used");
        assert!(!state.is_parameter, "Variable should not be a parameter");
    }
    
    #[test]
    fn test_parameter_handling() {
        let mut analyzer = DataFlowAnalyzer::new();
        let mut context = AnalysisContext::new(
            "test.bsl".to_string(),
            "Процедура TestProcedure(Param1) Перем LocalVar; КонецПроцедуры".to_string()
        );
        
        let ast = create_procedure_with_parameters();
        context.ast = Some(ast);
        
        let result = analyzer.analyze(&mut context);
        assert!(result.is_ok());
        
        let states = analyzer.get_variable_states();
        
        // Проверяем параметр
        assert!(states.contains_key("Param1"));
        let param_state = &states["Param1"];
        assert!(param_state.declared, "Parameter should be declared");
        assert!(param_state.initialized, "Parameter should be initialized");
        assert!(param_state.used, "Parameter should be used");
        assert!(param_state.is_parameter, "Should be marked as parameter");
        
        // Проверяем локальную переменную
        assert!(states.contains_key("LocalVar"));
        let local_state = &states["LocalVar"];
        assert!(local_state.declared, "Local variable should be declared");
        assert!(!local_state.initialized, "Local variable should not be initialized");
        assert!(!local_state.used, "Local variable should not be used");
        assert!(!local_state.is_parameter, "Should not be marked as parameter");
    }
    
    #[test]
    fn test_undeclared_variable_error() {
        let mut analyzer = DataFlowAnalyzer::new();
        let mut context = AnalysisContext::new(
            "test.bsl".to_string(),
            "UndeclaredVar = 10;".to_string()
        );
        
        let ast = create_undeclared_variable_usage();
        context.ast = Some(ast);
        
        let result = analyzer.analyze(&mut context);
        assert!(result.is_ok());
        
        // Должна быть ошибка об необъявленной переменной
        let diagnostics = analyzer.get_diagnostics();
        assert!(!diagnostics.is_empty(), "Should have diagnostics for undeclared variable");
        
        let error = &diagnostics[0];
        assert!(error.message.contains("используется, но не объявлена"));
        assert!(error.message.contains("UndeclaredVar"));
    }
    
    #[test]
    fn test_uninitialized_variable_warning() {
        let mut analyzer = DataFlowAnalyzer::new();
        let mut context = AnalysisContext::new(
            "test.bsl".to_string(),
            "Перем UninitVar;".to_string()
        );
        
        // Создаем AST с необинициализированной переменной
        let mut module = AstNode::new(AstNodeType::Module, create_span(1, 1));
        let var_decl = create_test_node(
            AstNodeType::VariableDeclaration,
            Some("UninitVar".to_string()),
            1
        );
        module.add_child(var_decl);
        
        context.ast = Some(module);
        
        let result = analyzer.analyze(&mut context);
        assert!(result.is_ok());
        
        // Проверяем список неинициализированных переменных
        let uninitialized = analyzer.get_uninitialized_variables();
        assert!(!uninitialized.is_empty(), "Should have uninitialized variables");
        assert!(uninitialized.contains(&"UninitVar".to_string()));
        
        // Проверяем диагностики
        let diagnostics = analyzer.get_diagnostics();
        let warning_found = diagnostics.iter().any(|d| 
            d.message.contains("может быть не инициализирована") &&
            d.message.contains("UninitVar")
        );
        assert!(warning_found, "Should have warning about uninitialized variable");
    }
    
    #[test]
    fn test_unused_variable_warning() {
        let mut analyzer = DataFlowAnalyzer::new();
        let mut context = AnalysisContext::new(
            "test.bsl".to_string(),
            "Перем UnusedVar; UnusedVar = 10;".to_string()
        );
        
        // Создаем AST с неиспользуемой переменной
        let mut module = AstNode::new(AstNodeType::Module, create_span(1, 1));
        
        // Объявление переменной
        let var_decl = create_test_node(
            AstNodeType::VariableDeclaration,
            Some("UnusedVar".to_string()),
            1
        );
        module.add_child(var_decl);
        
        // Присваивание (инициализация)
        let mut assignment = create_test_node(AstNodeType::Assignment, None, 2);
        let identifier = create_test_node(
            AstNodeType::Identifier,
            Some("UnusedVar".to_string()),
            2
        );
        assignment.add_child(identifier);
        module.add_child(assignment);
        
        // НЕ используем переменную после этого
        
        context.ast = Some(module);
        
        let result = analyzer.analyze(&mut context);
        assert!(result.is_ok());
        
        // Проверяем список неиспользуемых переменных
        let unused = analyzer.get_unused_variables();
        assert!(!unused.is_empty(), "Should have unused variables");
        assert!(unused.contains(&"UnusedVar".to_string()));
        
        // Проверяем диагностики
        let diagnostics = analyzer.get_diagnostics();
        let warning_found = diagnostics.iter().any(|d| 
            d.message.contains("не используется") &&
            d.message.contains("UnusedVar")
        );
        assert!(warning_found, "Should have warning about unused variable");
    }
    
    #[test]
    fn test_variable_state_tracking() {
        let mut analyzer = DataFlowAnalyzer::new();
        let mut context = AnalysisContext::new(
            "test.bsl".to_string(),
            "Перем TestVar; TestVar = 10; Сообщить(TestVar);".to_string()
        );
        
        let ast = create_module_with_variables();
        context.ast = Some(ast);
        
        let result = analyzer.analyze(&mut context);
        assert!(result.is_ok());
        
        let states = analyzer.get_variable_states();
        let state = &states["TestVar"];
        
        // Отладочная информация
        println!("TestVar state: declared={}, initialized={}, used={}", 
            state.declared, state.initialized, state.used);
        println!("Declaration line: {}", state.declaration_line);
        println!("First use line: {:?}", state.first_use_line);
        println!("Last use line: {:?}", state.last_use_line);
        
        // Проверяем правильность отслеживания состояния
        assert_eq!(state.declaration_line, 2);
        assert!(state.first_use_line.is_some());
        assert!(state.last_use_line.is_some());
        assert_eq!(state.first_use_line.unwrap(), 4); // Первое использование НЕ в присваивании
        assert_eq!(state.last_use_line.unwrap(), 4); // Последнее использование в выводе
    }
    
    #[test]
    fn test_complex_data_flow() {
        let mut analyzer = DataFlowAnalyzer::new();
        let mut context = AnalysisContext::new(
            "test.bsl".to_string(),
            "Complex data flow test".to_string()
        );
        
        // Создаем сложную структуру с несколькими переменными
        let mut module = AstNode::new(AstNodeType::Module, create_span(1, 1));
        
        // Переменная 1: объявлена, инициализирована, использована
        let var1_decl = create_test_node(
            AstNodeType::VariableDeclaration,
            Some("UsedVar".to_string()),
            2
        );
        module.add_child(var1_decl);
        
        let mut var1_assign = create_test_node(AstNodeType::Assignment, None, 3);
        let var1_id = create_test_node(
            AstNodeType::Identifier,
            Some("UsedVar".to_string()),
            3
        );
        var1_assign.add_child(var1_id);
        module.add_child(var1_assign);
        
        let var1_usage = create_test_node(
            AstNodeType::Identifier,
            Some("UsedVar".to_string()),
            4
        );
        module.add_child(var1_usage);
        
        // Переменная 2: объявлена, не инициализирована, не использована
        let var2_decl = create_test_node(
            AstNodeType::VariableDeclaration,
            Some("UnusedUninitVar".to_string()),
            5
        );
        module.add_child(var2_decl);
        
        // Переменная 3: объявлена, инициализирована, не использована
        let var3_decl = create_test_node(
            AstNodeType::VariableDeclaration,
            Some("UnusedInitVar".to_string()),
            6
        );
        module.add_child(var3_decl);
        
        let mut var3_assign = create_test_node(AstNodeType::Assignment, None, 7);
        let var3_id = create_test_node(
            AstNodeType::Identifier,
            Some("UnusedInitVar".to_string()),
            7
        );
        var3_assign.add_child(var3_id);
        module.add_child(var3_assign);
        
        context.ast = Some(module);
        
        let result = analyzer.analyze(&mut context);
        assert!(result.is_ok());
        
        let states = analyzer.get_variable_states();
        
        // Проверяем первую переменную
        let used_var = &states["UsedVar"];
        assert!(used_var.declared && used_var.initialized && used_var.used);
        
        // Проверяем вторую переменную
        let unused_uninit_var = &states["UnusedUninitVar"];
        assert!(unused_uninit_var.declared && !unused_uninit_var.initialized && !unused_uninit_var.used);
        
        // Проверяем третью переменную
        let unused_init_var = &states["UnusedInitVar"];
        assert!(unused_init_var.declared && unused_init_var.initialized && !unused_init_var.used);
        
        // Проверяем предупреждения
        let uninitialized = analyzer.get_uninitialized_variables();
        let unused = analyzer.get_unused_variables();
        
        assert!(uninitialized.contains(&"UnusedUninitVar".to_string()));
        assert!(unused.contains(&"UnusedUninitVar".to_string()));
        assert!(unused.contains(&"UnusedInitVar".to_string()));
        assert!(!unused.contains(&"UsedVar".to_string()));
    }
    
    #[test]
    fn test_assignment_without_declaration() {
        let mut analyzer = DataFlowAnalyzer::new();
        let mut context = AnalysisContext::new(
            "test.bsl".to_string(),
            "UndeclaredVar = 10;".to_string()
        );
        
        // Создаем AST с присваиванием необъявленной переменной
        let mut module = AstNode::new(AstNodeType::Module, create_span(1, 1));
        
        let mut assignment = create_test_node(AstNodeType::Assignment, None, 1);
        let identifier = create_test_node(
            AstNodeType::Identifier,
            Some("UndeclaredVar".to_string()),
            1
        );
        assignment.add_child(identifier);
        module.add_child(assignment);
        
        context.ast = Some(module);
        
        let result = analyzer.analyze(&mut context);
        assert!(result.is_ok());
        
        // Должна быть ошибка о присваивании необъявленной переменной
        let diagnostics = analyzer.get_diagnostics();
        let error_found = diagnostics.iter().any(|d| 
            d.message.contains("используется в присваивании, но не объявлена") &&
            d.message.contains("UndeclaredVar")
        );
        assert!(error_found, "Should have error about assignment to undeclared variable");
    }
    
    #[test]
    fn test_parameter_not_in_warnings() {
        let mut analyzer = DataFlowAnalyzer::new();
        let mut context = AnalysisContext::new(
            "test.bsl".to_string(),
            "Процедура Test(UnusedParam) КонецПроцедуры".to_string()
        );
        
        // Создаем AST с неиспользуемым параметром
        let mut procedure = create_test_node(
            AstNodeType::Procedure,
            Some("Test".to_string()),
            1
        );
        
        let param = create_test_node(
            AstNodeType::Parameter,
            Some("UnusedParam".to_string()),
            1
        );
        procedure.add_child(param);
        
        let mut module = AstNode::new(AstNodeType::Module, create_span(1, 1));
        module.add_child(procedure);
        
        context.ast = Some(module);
        
        let result = analyzer.analyze(&mut context);
        assert!(result.is_ok());
        
        // Параметры не должны попадать в списки неиспользуемых переменных
        let unused = analyzer.get_unused_variables();
        let uninitialized = analyzer.get_uninitialized_variables();
        
        assert!(!unused.contains(&"UnusedParam".to_string()));
        assert!(!uninitialized.contains(&"UnusedParam".to_string()));
        
        // Проверяем, что нет предупреждений о неиспользуемых параметрах
        let diagnostics = analyzer.get_diagnostics();
        let param_warning = diagnostics.iter().any(|d| 
            d.message.contains("UnusedParam")
        );
        assert!(!param_warning, "Should not have warnings about unused parameters");
    }
}
