/*!
# Integration tests for MethodVerifier with AnalysisEngine

Tests verifying that MethodVerifier is properly integrated into the analysis pipeline
*/

#[cfg(test)]
mod tests {
    use crate::analyzer::engine::AnalysisEngine;
    use serde_json;
    
    #[test]
    fn test_method_verification_integration() {
        let mut engine = AnalysisEngine::new();
        let config = serde_json::Value::Object(serde_json::Map::new());
        
        // Код с правильным вызовом метода
        let valid_code = r#"
            Процедура Тест()
                Таблица = Новый ТаблицаЗначений();
                Строка = Таблица.Добавить();
            КонецПроцедуры
        "#;
        
        let result = engine.analyze_code(valid_code, "test_valid.bsl", &config);
        assert!(result.is_ok());
        
        // Пока проверяем базовую функциональность
        let diagnostics = engine.get_diagnostics();
        println!("Diagnostics count for valid code: {}", diagnostics.len());
    }
    
    #[test]
    fn test_method_verification_with_invalid_method() {
        let mut engine = AnalysisEngine::new();
        let config = serde_json::Value::Object(serde_json::Map::new());
        
        // Код с неправильным вызовом метода
        let invalid_code = r#"
            Процедура Тест()
                Таблица = Новый ТаблицаЗначений();
                Строка = Таблица.НесуществующийМетод();
            КонецПроцедуры
        "#;
        
        let result = engine.analyze_code(invalid_code, "test_invalid.bsl", &config);
        assert!(result.is_ok());
        
        let diagnostics = engine.get_diagnostics();
        println!("Diagnostics count for invalid code: {}", diagnostics.len());
        
        // В идеале здесь должны быть диагностики об ошибке
        // но пока что просто проверяем, что анализ не падает
    }
    
    #[test]
    fn test_method_verifier_access() {
        let engine = AnalysisEngine::new();
        
        // Проверяем доступ к верификатору методов
        let verifier = engine.get_method_verifier();
        
        // Тестируем базовую функциональность верификатора
        assert!(verifier.verify_method_exists("ТаблицаЗначений", "Добавить"));
        assert!(!verifier.verify_method_exists("ТаблицаЗначений", "НесуществующийМетод"));
        
        let methods = verifier.get_available_methods("ТаблицаЗначений");
        assert!(!methods.is_empty());
        assert!(methods.contains(&"Добавить".to_string()));
        assert!(methods.contains(&"Удалить".to_string()));
    }
    
    #[test]
    fn test_method_suggestions() {
        let engine = AnalysisEngine::new();
        let verifier = engine.get_method_verifier();
        
        // Тестируем получение предложений для исправления
        let suggestions = verifier.get_suggestions_for_method("ТаблицаЗначений", "Добавт");
        assert!(!suggestions.is_empty());
        
        // Предложения должны содержать правильный метод "Добавить"
        let suggestions_text = suggestions.join(" ");
        assert!(suggestions_text.contains("Добавить"));
    }
    
    #[test]
    fn test_type_compatibility() {
        let engine = AnalysisEngine::new();
        let verifier = engine.get_method_verifier();
        
        // Тестируем проверку совместимости типов
        assert!(verifier.verify_type_compatibility("Строка", "Строка"));
        assert!(verifier.verify_type_compatibility("Произвольный", "Строка"));
        assert!(verifier.verify_type_compatibility("Строка", "Произвольный"));
        assert!(verifier.verify_type_compatibility("Неопределено", "Строка"));
        
        // Числа и строки могут быть совместимыми в определенных случаях
        assert!(verifier.verify_type_compatibility("Число", "Строка"));
        
        // Несовместимые типы
        assert!(!verifier.verify_type_compatibility("Булево", "Число"));
    }
    
    #[test]
    fn test_expression_type_analysis() {
        let engine = AnalysisEngine::new();
        let verifier = engine.get_method_verifier();
        
        // Тестируем анализ типов выражений
        assert_eq!(verifier.analyze_expression_type("\"строка\""), "Строка");
        assert_eq!(verifier.analyze_expression_type("42"), "Число");
        assert_eq!(verifier.analyze_expression_type("3.14"), "Число");
        assert_eq!(verifier.analyze_expression_type("Истина"), "Булево");
        assert_eq!(verifier.analyze_expression_type("Ложь"), "Булево");
        assert_eq!(verifier.analyze_expression_type("Неопределено"), "Неопределено");
        
        // Конструкторы объектов
        assert_eq!(verifier.analyze_expression_type("Новый ТаблицаЗначений()"), "ТаблицаЗначений");
        assert_eq!(verifier.analyze_expression_type("Новый Запрос()"), "Запрос");
    }
}
