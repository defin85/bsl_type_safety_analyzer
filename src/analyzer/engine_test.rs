/*!
# Tests for AnalysisEngine

Integration tests for coordinated analysis pipeline
*/

#[cfg(test)]
mod tests {
    use super::super::engine::AnalysisEngine;
    use serde_json;
    
    #[test]
    fn test_analysis_engine_integration() {
        let mut engine = AnalysisEngine::new();
        let config = serde_json::Value::Object(serde_json::Map::new());
        
        let bsl_code = r#"
            Процедура ТестоваяПроцедура(Параметр1)
                Переменная = "значение";
                ВозвратьПеременная;
            КонецПроцедуры
        "#;
        
        let result = engine.analyze_code(bsl_code, "test.bsl", &config);
        
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_engine_with_invalid_code() {
        let mut engine = AnalysisEngine::new();
        let config = serde_json::Value::Object(serde_json::Map::new());
        
        let invalid_code = r#"
            НеПравильнаяСинтаксис((()
            Переменная без значения = ;
        "#;
        
        let result = engine.analyze_code(invalid_code, "invalid.bsl", &config);
        
        // Должен вернуть результат с ошибками валидации
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_engine_statistics() {
        let mut engine = AnalysisEngine::new();
        let config = serde_json::Value::Object(serde_json::Map::new());
        
        let bsl_code = r#"
            Процедура Процедура1()
                Переменная1 = 42;
            КонецПроцедуры
            
            Функция Функция1()
                Возврат "результат";
            КонецФункции
        "#;
        
        let result = engine.analyze_code(bsl_code, "stats.bsl", &config);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_engine_empty_code() {
        let mut engine = AnalysisEngine::new();
        let config = serde_json::Value::Object(serde_json::Map::new());
        
        let result = engine.analyze_code("", "empty.bsl", &config);
        
        assert!(result.is_ok());
    }
}
