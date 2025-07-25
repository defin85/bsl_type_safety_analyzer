/*!
# Semantic Analyzer Integration Tests

Comprehensive integration tests for the enhanced BSL Semantic Analyzer.
Tests realistic BSL code scenarios and semantic analysis features.
*/

#[cfg(test)]
mod tests {
    use crate::analyzer::semantic::{
        SemanticAnalyzer, SemanticAnalysisConfig, TypeSystem, ScopeType
    };
    use crate::parser::ast::{AstNode, AstNodeType, Span, Position};

    /// Creates a test semantic analyzer with enhanced configuration
    fn create_test_analyzer() -> SemanticAnalyzer {
        SemanticAnalyzer::new(SemanticAnalysisConfig {
            check_unused_variables: true,
            check_undefined_variables: true,
            check_type_compatibility: true,
            check_method_calls: true,
            check_parameter_count: true,
            warn_on_implicit_conversions: true,
            suggest_similar_names: true,
            analyze_global_functions: true,
            verbose: false,
        })
    }

    /// Creates a verbose test analyzer for debugging
    fn create_verbose_analyzer() -> SemanticAnalyzer {
        SemanticAnalyzer::new(SemanticAnalysisConfig {
            verbose: true,
            ..Default::default()
        })
    }
    
    fn create_span(line: usize, column: usize) -> Span {
        let start = Position::new(line, column, 0);
        let end = Position::new(line, column + 10, 10);
        Span::new(start, end)
    }
    
    fn create_test_node(node_type: AstNodeType, value: Option<String>) -> AstNode {
        if let Some(val) = value {
            AstNode::with_value(node_type, create_span(1, 1), val)
        } else {
            AstNode::new(node_type, create_span(1, 1))
        }
    }

    #[test]
    fn test_enhanced_semantic_analyzer_creation() {
        let analyzer = create_test_analyzer();
        assert_eq!(analyzer.current_scope.name, "global");
        assert_eq!(analyzer.current_scope.scope_type, ScopeType::Global);
        assert!(analyzer.errors.is_empty());
        assert!(analyzer.warnings.is_empty());
        assert!(analyzer.config.check_method_calls);
        assert!(analyzer.config.suggest_similar_names);
        assert!(analyzer.config.analyze_global_functions);
    }

    #[test]
    fn test_enhanced_type_system() {
        let type_system = TypeSystem::new();
        
        // Test enhanced builtin types
        assert!(type_system.is_builtin_type("Строка"));
        assert!(type_system.is_builtin_type("Число"));
        assert!(type_system.is_builtin_type("УникальныйИдентификатор"));
        assert!(type_system.is_builtin_type("Произвольный"));
        
        // Test enhanced known objects
        assert!(type_system.is_known_object("ТаблицаЗначений"));
        assert!(type_system.is_known_object("HTTPСоединение"));
        assert!(type_system.is_known_object("ЧтениеXML"));
        assert!(type_system.is_known_object("ПостроительЗапроса"));
        assert!(type_system.is_known_object("МенеджерВременныхТаблиц"));
        
        // Test global functions
        assert!(type_system.is_global_function("Сообщить"));
        assert!(type_system.is_global_function("Строка"));
        assert!(type_system.is_global_function("ТипЗнч"));
        assert!(!type_system.is_global_function("НеизвестнаяФункция"));
    }

    #[test]
    fn test_method_info_system() {
        let type_system = TypeSystem::new();
        
        // Test ТаблицаЗначений methods
        let add_method = type_system.get_method_info("ТаблицаЗначений", "Добавить");
        assert!(add_method.is_some());
        
        let method_info = add_method.unwrap();
        assert_eq!(method_info.name, "Добавить");
        assert_eq!(method_info.return_type, Some("СтрокаТаблицыЗначений".to_string()));
        assert!(!method_info.is_procedure);
        assert!(method_info.parameters.is_empty());
        
        // Test method with parameters
        let find_method = type_system.get_method_info("ТаблицаЗначений", "Найти");
        assert!(find_method.is_some());
        
        let find_info = find_method.unwrap();
        assert_eq!(find_info.parameters.len(), 2);
        assert_eq!(find_info.parameters[0].name, "Значение");
        assert!(!find_info.parameters[0].is_optional);
        assert_eq!(find_info.parameters[1].name, "Колонка");
        assert!(find_info.parameters[1].is_optional);
        
        // Test unknown method
        assert!(type_system.get_method_info("ТаблицаЗначений", "НеизвестныйМетод").is_none());
    }

    #[test]
    fn test_method_signature_generation() {
        let type_system = TypeSystem::new();
        
        let signature = type_system.get_method_signature("ТаблицаЗначений", "Добавить");
        assert_eq!(signature, Some("Добавить() -> СтрокаТаблицыЗначений".to_string()));
        
        let find_signature = type_system.get_method_signature("ТаблицаЗначений", "Найти");
        assert!(find_signature.is_some());
        let sig = find_signature.unwrap();
        assert!(sig.contains("Значение: Произвольный"));
        assert!(sig.contains("[Колонка: Строка]")); // Optional parameter
        assert!(sig.contains("-> СтрокаТаблицыЗначений"));
        
        // Test unknown method
        let unknown_sig = type_system.get_method_signature("НеизвестныйТип", "Метод");
        assert!(unknown_sig.is_none());
    }

    #[test]
    fn test_method_call_validation() {
        let type_system = TypeSystem::new();
        
        // Valid parameter count for method without parameters
        let result = type_system.validate_method_call("ТаблицаЗначений", "Добавить", 0);
        assert!(result.is_ok());
        
        // Valid parameter count for method with required parameter
        let result = type_system.validate_method_call("ТаблицаЗначений", "Найти", 1);
        assert!(result.is_ok());
        
        // Valid parameter count with optional parameter
        let result = type_system.validate_method_call("ТаблицаЗначений", "Найти", 2);
        assert!(result.is_ok());
        
        // Invalid parameter count - too few
        let result = type_system.validate_method_call("ТаблицаЗначений", "Найти", 0);
        assert!(result.is_err());
        
        // Invalid parameter count - too many
        let result = type_system.validate_method_call("ТаблицаЗначений", "Добавить", 1);
        assert!(result.is_err());
        
        // Unknown method
        let result = type_system.validate_method_call("ТаблицаЗначений", "НеизвестныйМетод", 0);
        assert!(result.is_err());
        
        // Unknown object
        let result = type_system.validate_method_call("НеизвестныйОбъект", "Метод", 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_basic_analysis() {
        let mut analyzer = create_test_analyzer();
        
        // Create a simple function node
        let function_node = create_test_node(AstNodeType::Function, Some("ТестФункция".to_string()));
        
        // Use the public analyze method
        let result = analyzer.analyze(&function_node);
        
        // Should analyze without critical errors
        assert!(result.is_ok());
    }

    #[test]
    fn test_identifier_analysis() {
        let mut analyzer = create_test_analyzer();
        
        // Test identifier analysis
        let identifier_node = create_test_node(AstNodeType::Identifier, Some("СообщитьПользователю".to_string()));
        
        let result = analyzer.analyze(&identifier_node);
        
        // Should analyze identifiers
        assert!(result.is_ok());
    }

    #[test]
    fn test_expression_analysis() {
        let mut analyzer = create_test_analyzer();
        
        // Create a simple expression
        let expression_node = create_test_node(AstNodeType::Expression, None);
        
        let result = analyzer.analyze(&expression_node);
        
        // Should handle expressions
        assert!(result.is_ok());
    }

    #[test]
    fn test_global_function_list() {
        let type_system = TypeSystem::new();
        
        let global_functions = type_system.get_global_functions();
        
        // Should contain key BSL functions
        assert!(global_functions.contains(&"Сообщить".to_string()));
        assert!(global_functions.contains(&"Строка".to_string()));
        assert!(global_functions.contains(&"Число".to_string()));
        assert!(global_functions.contains(&"ТипЗнч".to_string()));
        assert!(!global_functions.is_empty());
    }

    #[test]
    fn test_enhanced_configuration() {
        let config = SemanticAnalysisConfig {
            check_unused_variables: true,
            check_undefined_variables: true,
            check_type_compatibility: true,
            check_method_calls: true,
            check_parameter_count: true,
            warn_on_implicit_conversions: true,
            suggest_similar_names: true,
            analyze_global_functions: true,
            verbose: false,
        };

        let analyzer = SemanticAnalyzer::new(config);
        
        // All enhanced features should be enabled
        assert!(analyzer.config.check_method_calls);
        assert!(analyzer.config.check_parameter_count);
        assert!(analyzer.config.suggest_similar_names);
        assert!(analyzer.config.analyze_global_functions);
    }

    #[test]
    fn test_verbose_mode() {
        let verbose_analyzer = create_verbose_analyzer();
        
        // Verbose mode should be enabled
        assert!(verbose_analyzer.config.verbose);
    }

    #[test]
    fn test_type_system_object_types() {
        let type_system = TypeSystem::new();
        
        // Test core BSL object types that we know are implemented
        let core_objects = vec![
            "ТаблицаЗначений",
            "HTTPСоединение", 
            "ЧтениеXML",
            "ЗаписьXML",
            "ПостроительЗапроса",
            "МенеджерВременныхТаблиц"
        ];

        for object_type in core_objects {
            assert!(type_system.is_known_object(object_type), 
                "Основной объект {} должен быть известен", object_type);
        }
    }

    #[test]  
    fn test_builtin_types_comprehensive() {
        let type_system = TypeSystem::new();
        
        // Test core builtin types that should definitely be supported
        let core_types = vec![
            "Строка", "Число", "Дата", "Булево", "УникальныйИдентификатор",
            "Неопределено", "Произвольный"
        ];

        for builtin_type in core_types {
            assert!(type_system.is_builtin_type(builtin_type), 
                "Основной встроенный тип {} должен быть известен", builtin_type);
        }
    }
}
