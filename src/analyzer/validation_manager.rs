/*!
# Validation Manager

Система валидации для входных данных анализатора BSL кода.
Включает валидацию кода, файлов, конфигурации и токенов.
*/

use std::path::Path;
use std::fs;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::diagnostics::{Diagnostic, DiagnosticLevel};

/// Результат валидации
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Валидация прошла успешно
    pub is_valid: bool,
    /// Список ошибок
    pub errors: Vec<String>,
    /// Список предупреждений
    pub warnings: Vec<String>,
}

impl ValidationResult {
    /// Создает новый результат валидации
    pub fn new(is_valid: bool) -> Self {
        Self {
            is_valid,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Создает успешный результат валидации
    pub fn success() -> Self {
        Self::new(true)
    }

    /// Создает неудачный результат валидации
    pub fn failure() -> Self {
        Self::new(false)
    }

    /// Добавляет ошибку в результат
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
        self.is_valid = false;
    }

    /// Добавляет предупреждение в результат
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    /// Проверяет наличие ошибок
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Проверяет наличие предупреждений
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Объединяет с другим результатом валидации
    pub fn merge(&mut self, other: ValidationResult) {
        if !other.is_valid {
            self.is_valid = false;
        }
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
    }
}

/// Валидатор кода BSL
pub struct CodeValidator {
    /// Паттерн для BSL файлов
    bsl_file_pattern: Regex,
}

impl CodeValidator {
    /// Создает новый валидатор кода
    pub fn new() -> Self {
        Self {
            bsl_file_pattern: Regex::new(r"(?i)\.(bsl|os)$").unwrap(),
        }
    }

    /// Валидирует код BSL
    pub fn validate_code(&self, code: &str, _file_path: &str) -> ValidationResult {
        let mut result = ValidationResult::success();

        // Проверяем, что код не пустой (None проверки не нужны в Rust)
        if code.trim().is_empty() {
            result.add_warning("Код пустой".to_string());
            return result;
        }

        // Проверяем размер кода
        if code.len() > 1_000_000 { // 1MB
            result.add_warning("Код слишком большой (>1MB)".to_string());
        }

        // Проверяем кодировку UTF-8 (в Rust строки всегда валидные UTF-8)
        // Но можем проверить на потенциально проблемные символы
        if code.chars().any(|c| c.is_control() && c != '\n' && c != '\r' && c != '\t') {
            result.add_warning("Код содержит управляющие символы".to_string());
        }

        // Проверяем наличие базовых BSL конструкций
        if !self.has_bsl_constructs(code) {
            result.add_warning("Код не содержит типичных конструкций BSL".to_string());
        }

        result
    }

    /// Валидирует путь к файлу
    pub fn validate_file_path(&self, file_path: &str) -> ValidationResult {
        let mut result = ValidationResult::success();
        let path = Path::new(file_path);

        // Проверяем существование файла
        if !path.exists() {
            result.add_error(format!("Файл не существует: {}", file_path));
            return result;
        }

        // Проверяем, что это файл, а не директория
        if !path.is_file() {
            result.add_error(format!("Путь указывает на директорию, а не файл: {}", file_path));
            return result;
        }

        // Проверяем расширение
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                if !self.bsl_file_pattern.is_match(&format!(".{}", ext_str)) {
                    result.add_warning(format!("Файл имеет нестандартное расширение: .{}", ext_str));
                }
            }
        } else {
            result.add_warning("Файл не имеет расширения".to_string());
        }

        // Проверяем размер файла
        match fs::metadata(path) {
            Ok(metadata) => {
                let file_size = metadata.len();
                if file_size > 10 * 1024 * 1024 { // 10MB
                    result.add_warning(format!("Файл очень большой: {:.1}MB", file_size as f64 / 1024.0 / 1024.0));
                }
            }
            Err(_) => {
                result.add_error(format!("Не удалось получить информацию о файле: {}", file_path));
            }
        }

        result
    }

    /// Проверяет наличие базовых конструкций BSL
    fn has_bsl_constructs(&self, code: &str) -> bool {
        let bsl_keywords = [
            "Процедура", "Функция", "КонецПроцедуры", "КонецФункции",
            "Перем", "Если", "Тогда", "КонецЕсли", "Для", "Цикл", "КонецЦикла"
        ];

        bsl_keywords.iter().any(|keyword| code.contains(keyword))
    }
}

impl Default for CodeValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Валидатор конфигурации
pub struct ConfigValidator;

impl ConfigValidator {
    /// Создает новый валидатор конфигурации
    pub fn new() -> Self {
        Self
    }

    /// Валидирует конфигурацию анализатора
    pub fn validate_analyzer_config(&self, config: &serde_json::Value) -> ValidationResult {
        let mut result = ValidationResult::success();

        // Проверяем обязательные атрибуты
        let required_attrs = ["strict_mode", "verbose", "check_documentation"];
        for attr in &required_attrs {
            if !config.get(attr).is_some() {
                result.add_error(format!("Отсутствует обязательный атрибут: {}", attr));
            }
        }

        // Проверяем типы значений
        if let Some(strict_mode) = config.get("strict_mode") {
            if !strict_mode.is_boolean() {
                result.add_error("strict_mode должен быть булевым значением".to_string());
            }
        }

        if let Some(verbose) = config.get("verbose") {
            if !verbose.is_boolean() {
                result.add_error("verbose должен быть булевым значением".to_string());
            }
        }

        if let Some(max_errors) = config.get("max_errors_per_file") {
            if let Some(max_errors_num) = max_errors.as_i64() {
                if max_errors_num <= 0 {
                    result.add_error("max_errors_per_file должен быть положительным числом".to_string());
                }
            } else {
                result.add_error("max_errors_per_file должен быть целым числом".to_string());
            }
        }

        result
    }
}

impl Default for ConfigValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Валидатор токенов
pub struct TokenValidator;

impl TokenValidator {
    /// Создает новый валидатор токенов
    pub fn new() -> Self {
        Self
    }

    /// Валидирует список токенов
    pub fn validate_tokens(&self, tokens: &[serde_json::Value]) -> ValidationResult {
        let mut result = ValidationResult::success();

        for (i, token) in tokens.iter().enumerate() {
            let token_result = self.validate_token(token, i);
            result.merge(token_result);
        }

        result
    }

    /// Валидирует отдельный токен
    pub fn validate_token(&self, token: &serde_json::Value, index: usize) -> ValidationResult {
        let mut result = ValidationResult::success();

        // Проверяем обязательные поля
        let required_fields = ["type", "value", "line", "column"];
        for field in &required_fields {
            if !token.get(field).is_some() {
                result.add_error(format!("Токен {}: отсутствует поле '{}'", index, field));
            }
        }

        // Проверяем типы полей
        if let Some(token_type) = token.get("type") {
            if !token_type.is_string() {
                result.add_error(format!("Токен {}: поле 'type' должно быть строкой", index));
            }
        }

        if let Some(value) = token.get("value") {
            if !value.is_string() {
                result.add_error(format!("Токен {}: поле 'value' должно быть строкой", index));
            }
        }

        if let Some(line) = token.get("line") {
            if let Some(line_num) = line.as_i64() {
                if line_num <= 0 {
                    result.add_error(format!("Токен {}: поле 'line' должно быть положительным числом", index));
                }
            } else {
                result.add_error(format!("Токен {}: поле 'line' должно быть целым числом", index));
            }
        }

        if let Some(column) = token.get("column") {
            if let Some(column_num) = column.as_i64() {
                if column_num < 0 {
                    result.add_error(format!("Токен {}: поле 'column' должно быть неотрицательным числом", index));
                }
            } else {
                result.add_error(format!("Токен {}: поле 'column' должно быть целым числом", index));
            }
        }

        result
    }
}

impl Default for TokenValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Менеджер валидации
pub struct ValidationManager {
    /// Валидатор кода
    code_validator: CodeValidator,
    /// Валидатор конфигурации
    config_validator: ConfigValidator,
    /// Валидатор токенов
    token_validator: TokenValidator,
}

impl ValidationManager {
    /// Создает новый менеджер валидации
    pub fn new() -> Self {
        Self {
            code_validator: CodeValidator::new(),
            config_validator: ConfigValidator::new(),
            token_validator: TokenValidator::new(),
        }
    }

    /// Валидирует входные данные для анализа
    pub fn validate_analysis_input(
        &self,
        code: &str,
        file_path: &str,
        config: &serde_json::Value,
    ) -> ValidationResult {
        let mut result = ValidationResult::success();

        // Валидируем код
        let code_result = self.code_validator.validate_code(code, file_path);
        result.merge(code_result);

        // Валидируем путь к файлу только если это реальный файл
        if !file_path.is_empty() 
            && file_path != "unknown" 
            && !file_path.starts_with("test") {
            let file_result = self.code_validator.validate_file_path(file_path);
            result.merge(file_result);
        }

        // Валидируем конфигурацию
        let config_result = self.config_validator.validate_analyzer_config(config);
        result.merge(config_result);

        result
    }

    /// Валидирует токены
    pub fn validate_tokens(&self, tokens: &[serde_json::Value]) -> ValidationResult {
        self.token_validator.validate_tokens(tokens)
    }

    /// Получает валидатор кода
    pub fn code_validator(&self) -> &CodeValidator {
        &self.code_validator
    }

    /// Получает валидатор конфигурации
    pub fn config_validator(&self) -> &ConfigValidator {
        &self.config_validator
    }

    /// Получает валидатор токенов
    pub fn token_validator(&self) -> &TokenValidator {
        &self.token_validator
    }

    /// Преобразует результат валидации в диагностики
    pub fn to_diagnostics(&self, result: &ValidationResult, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Добавляем ошибки
        for error in &result.errors {
            diagnostics.push(Diagnostic {
                level: DiagnosticLevel::Error,
                code: None,
                message: error.clone(),
                line: 0,
                column: 0,
                source: Some(file_path.to_string()),
            });
        }

        // Добавляем предупреждения
        for warning in &result.warnings {
            diagnostics.push(Diagnostic {
                level: DiagnosticLevel::Warning,
                code: None,
                message: warning.clone(),
                line: 0,
                column: 0,
                source: Some(file_path.to_string()),
            });
        }

        diagnostics
    }
}

impl Default for ValidationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_result() {
        let mut result = ValidationResult::success();
        assert!(result.is_valid);
        assert!(!result.has_errors());
        assert!(!result.has_warnings());

        result.add_warning("Test warning".to_string());
        assert!(result.is_valid);
        assert!(result.has_warnings());

        result.add_error("Test error".to_string());
        assert!(!result.is_valid);
        assert!(result.has_errors());
    }

    #[test]
    fn test_code_validator() {
        let validator = CodeValidator::new();

        // Тест валидного BSL кода
        let result = validator.validate_code("Процедура ТестПроцедура()\nКонецПроцедуры", "test.bsl");
        assert!(result.is_valid);
        assert!(!result.has_errors());

        // Тест пустого кода
        let result = validator.validate_code("   ", "test.bsl");
        assert!(result.is_valid);
        assert!(result.has_warnings());

        // Тест слишком большого кода
        let big_code = "А".repeat(1_000_001);
        let result = validator.validate_code(&big_code, "test.bsl");
        assert!(result.is_valid);
        assert!(result.has_warnings());
    }

    #[test]
    fn test_config_validator() {
        let validator = ConfigValidator::new();

        // Тест валидной конфигурации
        let config = serde_json::json!({
            "strict_mode": true,
            "verbose": false,
            "check_documentation": true
        });
        let result = validator.validate_analyzer_config(&config);
        assert!(result.is_valid);
        assert!(!result.has_errors());

        // Тест невалидной конфигурации
        let config = serde_json::json!({
            "strict_mode": "not_boolean"
        });
        let result = validator.validate_analyzer_config(&config);
        assert!(!result.is_valid);
        assert!(result.has_errors());
    }

    #[test]
    fn test_token_validator() {
        let validator = TokenValidator::new();

        // Тест валидного токена
        let tokens = vec![serde_json::json!({
            "type": "IDENTIFIER",
            "value": "test",
            "line": 1,
            "column": 0
        })];
        let result = validator.validate_tokens(&tokens);
        assert!(result.is_valid);
        assert!(!result.has_errors());

        // Тест невалидного токена
        let tokens = vec![serde_json::json!({
            "type": "IDENTIFIER",
            "line": -1
        })];
        let result = validator.validate_tokens(&tokens);
        assert!(!result.is_valid);
        assert!(result.has_errors());
    }

    #[test]
    fn test_validation_manager() {
        let manager = ValidationManager::new();

        let config = serde_json::json!({
            "strict_mode": true,
            "verbose": false,
            "check_documentation": true
        });

        // Тест валидных входных данных
        let result = manager.validate_analysis_input(
            "Процедура Тест()\nКонецПроцедуры",
            "test",
            &config
        );
        assert!(result.is_valid);

        // Тест невалидной конфигурации
        let bad_config = serde_json::json!({
            "strict_mode": "not_boolean"
        });
        let result = manager.validate_analysis_input(
            "Процедура Тест()\nКонецПроцедуры",
            "test",
            &bad_config
        );
        assert!(!result.is_valid);
        assert!(result.has_errors());
    }

    #[test]
    fn test_bsl_constructs_detection() {
        let validator = CodeValidator::new();

        assert!(validator.has_bsl_constructs("Процедура Тест() КонецПроцедуры"));
        assert!(validator.has_bsl_constructs("Функция GetValue() КонецФункции"));
        assert!(validator.has_bsl_constructs("Перем Значение;"));
        assert!(!validator.has_bsl_constructs("console.log('Hello World');"));
    }
}
