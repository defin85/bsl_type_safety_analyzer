/*!
# Method Verifier

Верификатор методов для BSL - проверяет корректность вызовов методов объектов 1С,
предоставляет предложения по исправлению ошибок и валидирует совместимость типов.
*/

use std::collections::HashMap;
// use crate::bsl_parser::semantic::TypeSystem; // TODO: перенести TypeSystem
use crate::diagnostics::Diagnostic;

/// Результат проверки вызова метода
#[derive(Debug, Clone)]
pub struct MethodCallResult {
    /// Успешна ли проверка
    pub is_valid: bool,
    /// Сообщение об ошибке (если есть)
    pub error_message: Option<String>,
    /// Предложения исправлений
    pub suggestions: Vec<String>,
    /// Найден ли метод
    pub method_found: bool,
    /// Правильность аргументов
    pub arguments_valid: bool,
}

impl MethodCallResult {
    pub fn success() -> Self {
        Self {
            is_valid: true,
            error_message: None,
            suggestions: Vec::new(),
            method_found: true,
            arguments_valid: true,
        }
    }
    
    pub fn method_not_found(object_type: &str, method_name: &str, suggestions: Vec<String>) -> Self {
        Self {
            is_valid: false,
            error_message: Some(format!("Метод '{}' не найден у типа '{}'", method_name, object_type)),
            suggestions,
            method_found: false,
            arguments_valid: true,
        }
    }
    
    pub fn invalid_arguments(message: String, suggestions: Vec<String>) -> Self {
        Self {
            is_valid: false,
            error_message: Some(message),
            suggestions,
            method_found: true,
            arguments_valid: false,
        }
    }
}

/// Информация об аргументе метода
#[derive(Debug, Clone)]
pub struct ArgumentInfo {
    /// Тип аргумента
    pub arg_type: String,
    /// Значение аргумента (если доступно)
    pub value: Option<String>,
    /// Позиция в списке аргументов
    pub position: usize,
}

/// Верификатор методов BSL
pub struct MethodVerifier {
    /// Система типов для проверки
    // type_system: TypeSystem, // TODO: перенести TypeSystem в bsl_parser
    /// Кэш результатов проверки для производительности
    verification_cache: HashMap<String, MethodCallResult>,
}

impl MethodVerifier {
    /// Создает новый верификатор методов
    pub fn new(/* type_system: TypeSystem */) -> Self {
        Self {
            // type_system,
            verification_cache: HashMap::new(),
        }
    }
    
    /// Проверяет вызов метода и возвращает результат проверки
    pub fn verify_call(
        &mut self,
        object_type: &str,
        method_name: &str,
        arguments: &[ArgumentInfo],
        _line: usize
    ) -> MethodCallResult {
        // Создаем ключ для кэширования
        let cache_key = format!("{}::{}", object_type, method_name);
        
        // Проверяем кэш для базовой информации о методе
        if let Some(cached_result) = self.verification_cache.get(&cache_key) {
            if cached_result.method_found {
                // Проверяем только аргументы для кэшированного результата
                return self.verify_arguments_only(object_type, method_name, arguments);
            } else {
                return cached_result.clone();
            }
        }
        
        // Проверяем существование типа объекта
        if !self.verify_object_type(object_type) {
            let result = MethodCallResult::method_not_found(
                object_type,
                method_name,
                vec![format!("Тип '{}' не найден в системе типов", object_type)]
            );
            self.verification_cache.insert(cache_key, result.clone());
            return result;
        }
        
        // Проверяем существование метода
        if !self.verify_method_exists(object_type, method_name) {
            let suggestions = self.get_suggestions_for_method(object_type, method_name);
            let result = MethodCallResult::method_not_found(object_type, method_name, suggestions);
            self.verification_cache.insert(cache_key, result.clone());
            return result;
        }
        
        // Проверяем аргументы
        let arguments_result = self.verify_method_arguments(object_type, method_name, arguments);
        if !arguments_result.is_valid {
            return arguments_result;
        }
        
        // Кэшируем успешный результат
        let result = MethodCallResult::success();
        self.verification_cache.insert(cache_key, result.clone());
        result
    }
    
    /// Проверяет существование метода у типа
    pub fn verify_method_exists(&self, _object_type: &str, _method_name: &str) -> bool {
        false // TODO: реализовать после переноса TypeSystem
    }
    
    /// Получает сигнатуру метода
    pub fn get_method_signature(&self, _object_type: &str, _method_name: &str) -> Option<String> {
        None // TODO: реализовать после переноса TypeSystem
    }
    
    /// Возвращает список доступных методов для типа
    pub fn get_available_methods(&self, _object_type: &str) -> Vec<String> {
        vec![] // TODO: реализовать после переноса TypeSystem
    }
    
    /// Проверяет существование типа объекта
    pub fn verify_object_type(&self, _object_type: &str) -> bool {
        false // TODO: реализовать после переноса TypeSystem
    }
    
    /// Получает предложения для исправления ошибочного вызова метода
    pub fn get_suggestions_for_method(&self, object_type: &str, method_name: &str) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        // Проверяем существование типа
        if !self.verify_object_type(object_type) {
            suggestions.push(format!("Тип '{}' не найден в документации", object_type));
            return suggestions;
        }
        
        // Получаем доступные методы
        let available_methods = self.get_available_methods(object_type);
        
        if available_methods.is_empty() {
            suggestions.push(format!("У типа '{}' нет доступных методов", object_type));
            return suggestions;
        }
        
        // Ищем похожие методы
        let similar_methods = self.find_similar_methods(method_name, &available_methods);
        
        if !similar_methods.is_empty() {
            suggestions.push(format!("Возможно, вы имели в виду: {}", similar_methods.join(", ")));
        } else if available_methods.len() <= 10 {
            // Показываем все методы, если их немного
            suggestions.push(format!("Доступные методы: {}", available_methods.join(", ")));
        } else {
            // Показываем только первые 10 методов
            let methods_count = available_methods.len();
            let first_methods: Vec<String> = available_methods.into_iter().take(10).collect();
            suggestions.push(format!("Некоторые доступные методы: {} (и еще {})", 
                first_methods.join(", "), methods_count - 10));
        }
        
        suggestions
    }
    
    /// Анализирует тип выражения
    pub fn analyze_expression_type(&self, expression: &str) -> String {
        // Простая эвристика для определения типа по строковому представлению
        let trimmed = expression.trim();
        
        if trimmed.starts_with('"') && trimmed.ends_with('"') {
            return "Строка".to_string();
        }
        
        if trimmed.parse::<i64>().is_ok() || trimmed.parse::<f64>().is_ok() {
            return "Число".to_string();
        }
        
        match trimmed {
            "Истина" | "Ложь" => "Булево".to_string(),
            "Неопределено" => "Неопределено".to_string(),
            _ => {
                // Проверяем, является ли это вызовом конструктора
                if trimmed.starts_with("Новый ") {
                    let type_name = trimmed.strip_prefix("Новый ").unwrap_or("");
                    if let Some(paren_pos) = type_name.find('(') {
                        let constructor_type = &type_name[..paren_pos];
                        if self.verify_object_type(constructor_type) {
                            return constructor_type.to_string();
                        }
                    }
                }
                
                "Неопределено".to_string()
            }
        }
    }
    
    /// Проверяет совместимость типов
    pub fn verify_type_compatibility(&self, _source_type: &str, _target_type: &str) -> bool {
        false // TODO: реализовать после переноса TypeSystem
    }
    
    /// Получает информацию об иерархии типов
    pub fn get_type_hierarchy_info(&self, _type_name: &str) -> Option<HashMap<String, String>> {
        // TODO: реализовать после переноса TypeSystem
        None
    }
    
    /// Создает диагностическое сообщение из результата проверки
    pub fn create_diagnostic(&self, result: &MethodCallResult, line: usize, column: usize) -> Option<Diagnostic> {
        if result.is_valid {
            return None;
        }
        
        let message = result.error_message.as_ref()?;
        let mut full_message = message.clone();
        
        if !result.suggestions.is_empty() {
            full_message.push_str("\n\nПредложения:\n");
            for suggestion in &result.suggestions {
                full_message.push_str(&format!("• {}\n", suggestion));
            }
        }
        
        Some(Diagnostic::error(
            full_message,
            line,
            column
        ))
    }
    
    /// Очищает кэш проверок
    pub fn clear_cache(&mut self) {
        self.verification_cache.clear();
    }
    
    // PRIVATE METHODS
    
    /// Проверяет только аргументы метода (для кэшированных результатов)
    fn verify_arguments_only(&self, object_type: &str, method_name: &str, arguments: &[ArgumentInfo]) -> MethodCallResult {
        self.verify_method_arguments(object_type, method_name, arguments)
    }
    
    /// Проверяет аргументы метода
    fn verify_method_arguments(&self, object_type: &str, method_name: &str, arguments: &[ArgumentInfo]) -> MethodCallResult {
        // Получаем сигнатуру метода
        if let Some(signature) = self.get_method_signature(object_type, method_name) {
            // Простая проверка - в реальной реализации здесь будет полный парсинг сигнатуры
            let expected_args = self.parse_method_signature(&signature);
            
            if arguments.len() != expected_args.len() {
                return MethodCallResult::invalid_arguments(
                    format!(
                        "Неверное количество аргументов для метода '{}'. Ожидается: {}, передано: {}",
                        method_name, expected_args.len(), arguments.len()
                    ),
                    vec![format!("Сигнатура метода: {}", signature)]
                );
            }
            
            // Проверяем типы аргументов
            for (i, (expected_type, actual_arg)) in expected_args.iter().zip(arguments.iter()).enumerate() {
                if !self.verify_type_compatibility(&actual_arg.arg_type, expected_type) {
                    return MethodCallResult::invalid_arguments(
                        format!(
                            "Несовместимый тип аргумента {} для метода '{}'. Ожидается: {}, передано: {}",
                            i + 1, method_name, expected_type, actual_arg.arg_type
                        ),
                        vec![format!("Сигнатура метода: {}", signature)]
                    );
                }
            }
        }
        
        MethodCallResult::success()
    }
    
    /// Находит методы, похожие на целевой (простейшая реализация)
    fn find_similar_methods(&self, target_method: &str, available_methods: &[String]) -> Vec<String> {
        let mut similar = Vec::new();
        let target_lower = target_method.to_lowercase();
        
        for method in available_methods {
            let method_lower = method.to_lowercase();
            
            // Простая эвристика: проверяем подстроки
            if target_lower.contains(&method_lower) || method_lower.contains(&target_lower) {
                similar.push(method.clone());
            } else if self.levenshtein_distance(&target_lower, &method_lower) <= 2 {
                // Добавляем методы с небольшим расстоянием редактирования
                similar.push(method.clone());
            }
        }
        
        // Ограничиваем количество предложений
        similar.truncate(3);
        similar
    }
    
    /// Простая реализация расстояния Левенштейна
    fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        let len1 = s1.chars().count();
        let len2 = s2.chars().count();
        
        if len1 == 0 { return len2; }
        if len2 == 0 { return len1; }
        
        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];
        
        // Инициализация первой строки и столбца
        for (i, row) in matrix.iter_mut().enumerate().take(len1 + 1) { row[0] = i; }
        for j in 0..=len2 { matrix[0][j] = j; }
        
        let chars1: Vec<char> = s1.chars().collect();
        let chars2: Vec<char> = s2.chars().collect();
        
        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if chars1[i-1] == chars2[j-1] { 0 } else { 1 };
                matrix[i][j] = *[
                    matrix[i-1][j] + 1,      // удаление
                    matrix[i][j-1] + 1,      // вставка
                    matrix[i-1][j-1] + cost  // замена
                ].iter().min().unwrap();
            }
        }
        
        matrix[len1][len2]
    }
    
    /// Парсит сигнатуру метода для извлечения типов аргументов
    fn parse_method_signature(&self, signature: &str) -> Vec<String> {
        // Простейшая реализация - в реальности будет более сложный парсер
        // Формат: "method_name(arg1: Type1, arg2: Type2) -> ReturnType"
        
        if let Some(start) = signature.find('(') {
            if let Some(end) = signature.find(')') {
                let args_str = &signature[start + 1..end];
                if args_str.trim().is_empty() {
                    return Vec::new();
                }
                
                return args_str
                    .split(',')
                    .map(|arg| {
                        if let Some(colon_pos) = arg.find(':') {
                            arg[colon_pos + 1..].trim().to_string()
                        } else {
                            "Произвольный".to_string()
                        }
                    })
                    .collect();
            }
        }
        
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::bsl_parser::semantic::TypeSystem; // TODO: перенести TypeSystem
    
    fn create_test_verifier() -> MethodVerifier {
        MethodVerifier::new()
    }
    
    #[test]
    fn test_method_verifier_creation() {
        let verifier = create_test_verifier();
        assert!(verifier.verification_cache.is_empty());
    }
    
    #[test]
    fn test_expression_type_analysis() {
        let verifier = create_test_verifier();
        
        assert_eq!(verifier.analyze_expression_type("\"строка\""), "Строка");
        assert_eq!(verifier.analyze_expression_type("42"), "Число");
        assert_eq!(verifier.analyze_expression_type("3.14"), "Число");
        assert_eq!(verifier.analyze_expression_type("Истина"), "Булево");
        assert_eq!(verifier.analyze_expression_type("Ложь"), "Булево");
        assert_eq!(verifier.analyze_expression_type("Неопределено"), "Неопределено");
    }
    
    #[test]
    fn test_levenshtein_distance() {
        let verifier = create_test_verifier();
        
        assert_eq!(verifier.levenshtein_distance("test", "test"), 0);
        assert_eq!(verifier.levenshtein_distance("test", "tost"), 1);
        assert_eq!(verifier.levenshtein_distance("test", ""), 4);
        assert_eq!(verifier.levenshtein_distance("", "test"), 4);
    }
    
    #[test]
    fn test_method_call_result_creation() {
        let success = MethodCallResult::success();
        assert!(success.is_valid);
        assert!(success.method_found);
        assert!(success.arguments_valid);
        
        let not_found = MethodCallResult::method_not_found(
            "ТаблицаЗначений", 
            "НесуществующийМетод", 
            vec!["Предложение".to_string()]
        );
        assert!(!not_found.is_valid);
        assert!(!not_found.method_found);
        assert!(not_found.arguments_valid);
        assert_eq!(not_found.suggestions.len(), 1);
    }
    
    #[test]
    fn test_argument_info_creation() {
        let arg = ArgumentInfo {
            arg_type: "Строка".to_string(),
            value: Some("\"тест\"".to_string()),
            position: 0,
        };
        
        assert_eq!(arg.arg_type, "Строка");
        assert_eq!(arg.value, Some("\"тест\"".to_string()));
        assert_eq!(arg.position, 0);
    }
    
    #[test]
    fn test_cache_functionality() {
        let mut verifier = create_test_verifier();
        
        // Кэш должен быть пустым
        assert!(verifier.verification_cache.is_empty());
        
        // После очистки кэш должен остаться пустым
        verifier.clear_cache();
        assert!(verifier.verification_cache.is_empty());
    }
}
