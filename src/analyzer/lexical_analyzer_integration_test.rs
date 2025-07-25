/*!
# Lexical Analyzer Integration Tests

Comprehensive integration tests for the BSL lexical analyzer with real-world
BSL code examples and edge cases.
*/

#[cfg(test)]
mod tests {
    use crate::analyzer::lexical_analyzer::{LexicalAnalyzer, LexicalTokenType, LexicalAnalysisConfig};

    fn create_test_analyzer() -> LexicalAnalyzer {
        LexicalAnalyzer::new()
    }

    fn create_verbose_analyzer() -> LexicalAnalyzer {
        LexicalAnalyzer::with_config(LexicalAnalysisConfig {
            include_whitespace: false,
            include_comments: true,
            verbose: true,
        })
    }

    #[test]
    fn test_lexical_analyzer_creation() {
        let _analyzer = create_test_analyzer();
        // Analyzer should be created successfully
        assert!(true);
    }

    #[test]
    fn test_empty_code_tokenization() {
        let analyzer = create_test_analyzer();
        let tokens = analyzer.tokenize("").unwrap();
        assert!(tokens.is_empty());
        
        let tokens = analyzer.tokenize("   ").unwrap();
        assert!(tokens.is_empty()); // Whitespace filtered out by default
    }

    #[test]
    fn test_basic_procedure_tokenization() {
        let analyzer = create_test_analyzer();
        let code = r#"
Процедура ТестоваяПроцедура()
    Сообщить("Тест");
КонецПроцедуры
        "#;
        
        let tokens = analyzer.tokenize(code).unwrap();
        assert!(!tokens.is_empty());
        
        // Find key tokens
        let procedure_tokens: Vec<_> = tokens.iter()
            .filter(|t| t.token_type == LexicalTokenType::Keyword && t.value == "Процедура")
            .collect();
        assert_eq!(procedure_tokens.len(), 1);
        
        let identifier_tokens: Vec<_> = tokens.iter()
            .filter(|t| t.token_type == LexicalTokenType::Identifier)
            .collect();
        assert!(identifier_tokens.len() >= 2); // ТестоваяПроцедура, Сообщить
        
        let string_tokens: Vec<_> = tokens.iter()
            .filter(|t| t.token_type == LexicalTokenType::String)
            .collect();
        assert_eq!(string_tokens.len(), 1);
        assert_eq!(string_tokens[0].value, r#""Тест""#);
    }

    #[test]
    fn test_function_with_parameters_tokenization() {
        let analyzer = create_test_analyzer();
        let code = r#"
Функция ВычислитьСумму(Число1, Число2)
    Возврат Число1 + Число2;
КонецФункции
        "#;
        
        let tokens = analyzer.tokenize(code).unwrap();
        
        // Check function keyword
        let function_tokens: Vec<_> = tokens.iter()
            .filter(|t| t.token_type == LexicalTokenType::Keyword && t.value == "Функция")
            .collect();
        assert_eq!(function_tokens.len(), 1);
        
        // Check return keyword
        let return_tokens: Vec<_> = tokens.iter()
            .filter(|t| t.token_type == LexicalTokenType::Keyword && t.value == "Возврат")
            .collect();
        assert_eq!(return_tokens.len(), 1);
        
        // Check operators
        let plus_tokens: Vec<_> = tokens.iter()
            .filter(|t| t.token_type == LexicalTokenType::Operator && t.value == "+")
            .collect();
        assert_eq!(plus_tokens.len(), 1);
    }

    #[test]
    fn test_if_statement_tokenization() {
        let analyzer = create_test_analyzer();
        let code = r#"
Если Условие Тогда
    Результат = Истина;
Иначе
    Результат = Ложь;
КонецЕсли;
        "#;
        
        let tokens = analyzer.tokenize(code).unwrap();
        
        // Check conditional keywords
        let conditional_keywords = vec!["Если", "Тогда", "Иначе", "КонецЕсли"];
        for keyword in conditional_keywords {
            let keyword_tokens: Vec<_> = tokens.iter()
                .filter(|t| t.token_type == LexicalTokenType::Keyword && t.value == keyword)
                .collect();
            assert_eq!(keyword_tokens.len(), 1, "Missing keyword: {}", keyword);
        }
        
        // Check boolean constants
        let bool_keywords = vec!["Истина", "Ложь"];
        for keyword in bool_keywords {
            let bool_tokens: Vec<_> = tokens.iter()
                .filter(|t| t.token_type == LexicalTokenType::Keyword && t.value == keyword)
                .collect();
            assert_eq!(bool_tokens.len(), 1, "Missing boolean: {}", keyword);
        }
    }

    #[test]
    fn test_for_loop_tokenization() {
        let analyzer = create_test_analyzer();
        let code = r#"
Для Каждого Элемент Из Коллекция Цикл
    Обработать(Элемент);
КонецЦикла;
        "#;
        
        let tokens = analyzer.tokenize(code).unwrap();
        
        // Check loop keywords
        let loop_keywords = vec!["Для", "Каждого", "Из", "Цикл", "КонецЦикла"];
        for keyword in loop_keywords {
            let keyword_tokens: Vec<_> = tokens.iter()
                .filter(|t| t.token_type == LexicalTokenType::Keyword && t.value == keyword)
                .collect();
            assert_eq!(keyword_tokens.len(), 1, "Missing loop keyword: {}", keyword);
        }
    }

    #[test]
    fn test_try_catch_tokenization() {
        let analyzer = create_test_analyzer();
        let code = r#"
Попытка
    РискованнаяОперация();
Исключение
    ОбработатьОшибку();
КонецПопытки;
        "#;
        
        let tokens = analyzer.tokenize(code).unwrap();
        
        // Check exception handling keywords
        let exception_keywords = vec!["Попытка", "Исключение", "КонецПопытки"];
        for keyword in exception_keywords {
            let keyword_tokens: Vec<_> = tokens.iter()
                .filter(|t| t.token_type == LexicalTokenType::Keyword && t.value == keyword)
                .collect();
            assert_eq!(keyword_tokens.len(), 1, "Missing exception keyword: {}", keyword);
        }
    }

    #[test]
    fn test_number_literals_tokenization() {
        let analyzer = create_test_analyzer();
        let code = "Перем Число1 = 123; Перем Число2 = 45.67; Перем Число3 = 0;";
        
        let tokens = analyzer.tokenize(code).unwrap();
        
        let number_tokens: Vec<_> = tokens.iter()
            .filter(|t| t.token_type == LexicalTokenType::Number)
            .collect();
        assert_eq!(number_tokens.len(), 3);
        
        let expected_numbers = ["123", "45.67", "0"];
        for (i, expected) in expected_numbers.iter().enumerate() {
            assert_eq!(number_tokens[i].value, *expected);
        }
    }

    #[test]
    fn test_string_literals_tokenization() {
        let analyzer = create_test_analyzer();
        let code = r#"Перем Строка1 = "Двойные кавычки"; Перем Строка2 = 'Одинарные кавычки';"#;
        
        let tokens = analyzer.tokenize(code).unwrap();
        
        let string_tokens: Vec<_> = tokens.iter()
            .filter(|t| t.token_type == LexicalTokenType::String)
            .collect();
        assert_eq!(string_tokens.len(), 2);
        
        assert_eq!(string_tokens[0].value, r#""Двойные кавычки""#);
        assert_eq!(string_tokens[1].value, "'Одинарные кавычки'");
    }

    #[test]
    fn test_operators_tokenization() {
        let analyzer = create_test_analyzer();
        let code = "А = Б + В - Г * Д / Е; Если А <> Б И В <= Г Или Д >= Е Тогда КонецЕсли;";
        
        let tokens = analyzer.tokenize(code).unwrap();
        
        let expected_operators = vec!["=", "+", "-", "*", "/", ";", "<>", "<=", ">="];
        for op in expected_operators {
            let op_tokens: Vec<_> = tokens.iter()
                .filter(|t| t.token_type == LexicalTokenType::Operator && t.value == op)
                .collect();
            assert!(!op_tokens.is_empty(), "Missing operator: {}", op);
        }
    }

    #[test]
    fn test_constructor_tokenization() {
        let analyzer = create_test_analyzer();
        let code = r#"
Перем Таблица = Новый ТаблицаЗначений();
Перем КонструкторЗапроса = Новый Запрос("ВЫБРАТЬ * ИЗ Справочник");
        "#;
        
        let tokens = analyzer.tokenize(code).unwrap();
        
        // Check "Новый" keyword
        let new_tokens: Vec<_> = tokens.iter()
            .filter(|t| t.token_type == LexicalTokenType::Keyword && t.value == "Новый")
            .collect();
        assert_eq!(new_tokens.len(), 2);
        
        // Check constructor types
        let constructor_identifiers = vec!["ТаблицаЗначений", "Запрос"];
        for identifier in constructor_identifiers {
            let id_tokens: Vec<_> = tokens.iter()
                .filter(|t| t.token_type == LexicalTokenType::Identifier && t.value == identifier)
                .collect();
            assert_eq!(id_tokens.len(), 1, "Missing constructor identifier: {}", identifier);
        }
    }

    #[test]
    fn test_comments_tokenization() {
        let analyzer = LexicalAnalyzer::with_config(LexicalAnalysisConfig {
            include_comments: true,
            ..Default::default()
        });
        
        let code = r#"
// Это однострочный комментарий
Процедура Тест() // Еще один комментарий
    // Комментарий внутри процедуры
КонецПроцедуры
        "#;
        
        let tokens = analyzer.tokenize(code).unwrap();
        
        let comment_tokens: Vec<_> = tokens.iter()
            .filter(|t| t.token_type == LexicalTokenType::Comment)
            .collect();
        assert_eq!(comment_tokens.len(), 3);
        
        // All comments should start with "//"
        for token in comment_tokens {
            assert!(token.value.starts_with("//"));
        }
    }

    #[test]
    fn test_position_tracking() {
        let analyzer = create_test_analyzer();
        let code = r#"Процедура
  Тест
    Параметр"#;
        
        let tokens = analyzer.tokenize(code).unwrap();
        
        // Check positions are tracked correctly
        assert_eq!(tokens[0].line, 1);
        assert_eq!(tokens[0].column, 1);
        assert_eq!(tokens[0].value, "Процедура");
        
        assert_eq!(tokens[1].line, 2);
        assert_eq!(tokens[1].column, 3);
        assert_eq!(tokens[1].value, "Тест");
        
        assert_eq!(tokens[2].line, 3);
        assert_eq!(tokens[2].column, 5);
        assert_eq!(tokens[2].value, "Параметр");
    }

    #[test]
    fn test_complex_procedure_tokenization() {
        let analyzer = create_test_analyzer();
        let code = r#"
Процедура ОбработатьДанные(Данные)
    Для Каждого Элемент Из Данные Цикл
        Попытка
            РезультатОбработки = ВыполнитьОбработку(Элемент);
            СохранитьРезультат(РезультатОбработки);
        Исключение
            ЗаписьОшибки = НоваяЗаписьОшибки();
            СохранитьОшибку(ЗаписьОшибки);
        КонецПопытки;
    КонецЦикла;
КонецПроцедуры
        "#;
        
        let tokens = analyzer.tokenize(code).unwrap();
        assert!(!tokens.is_empty());
        
        // Count different token types
        let keyword_count = tokens.iter().filter(|t| t.token_type == LexicalTokenType::Keyword).count();
        let identifier_count = tokens.iter().filter(|t| t.token_type == LexicalTokenType::Identifier).count();
        let operator_count = tokens.iter().filter(|t| t.token_type == LexicalTokenType::Operator).count();
        
        // Should have multiple keywords, identifiers, and operators
        assert!(keyword_count >= 8); // Процедура, Для, Каждого, Из, Цикл, etc.
        assert!(identifier_count >= 6); // ОбработатьДанные, Данные, Элемент, etc.
        assert!(operator_count >= 10); // Parentheses, semicolons, assignment, etc.
    }

    #[test]
    fn test_unknown_tokens_handling() {
        let analyzer = create_test_analyzer();
        let code = "Процедура № Тест @ КонецПроцедуры";
        
        let tokens = analyzer.tokenize(code).unwrap();
        
        // Should have some unknown tokens for № and @
        let unknown_tokens: Vec<_> = tokens.iter()
            .filter(|t| t.token_type == LexicalTokenType::Unknown)
            .collect();
        assert_eq!(unknown_tokens.len(), 2);
        assert_eq!(unknown_tokens[0].value, "№");
        assert_eq!(unknown_tokens[1].value, "@");
    }

    #[test]
    fn test_token_validation() {
        let analyzer = create_test_analyzer();
        let tokens = analyzer.tokenize("Процедура Тест() КонецПроцедуры").unwrap();
        
        // All tokens should be valid
        assert!(analyzer.validate_tokens(&tokens).is_ok());
        
        // Check that all tokens have proper positions
        for token in &tokens {
            assert!(token.line > 0);
            assert!(token.column > 0);
            assert!(!token.value.is_empty());
        }
    }

    #[test]
    fn test_verbose_analysis() {
        let analyzer = create_verbose_analyzer();
        let code = "Процедура Тест() КонецПроцедуры";
        
        // Should not panic with verbose output
        let tokens = analyzer.tokenize(code).unwrap();
        assert!(!tokens.is_empty());
    }
}
