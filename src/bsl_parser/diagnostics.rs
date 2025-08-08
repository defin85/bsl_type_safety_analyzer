//! Структуры для диагностических сообщений

use serde::{Deserialize, Serialize};

/// Уровень серьезности диагностики
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
    Info,
    Hint,
}

/// Местоположение в исходном коде
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub offset: usize,
    pub length: usize,
}

impl Location {
    pub fn new(file: String, line: usize, column: usize, offset: usize, length: usize) -> Self {
        Self {
            file,
            line,
            column,
            offset,
            length,
        }
    }
}

/// Детали диагностики
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticDetails {
    /// Найденное значение
    pub found: Option<String>,
    /// Ожидаемое значение
    pub expected: Option<String>,
    /// Дополнительная информация
    pub info: Option<String>,
    /// Предлагаемое исправление
    pub suggestion: Option<String>,
}

/// Диагностическое сообщение
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub location: Location,
    pub code: String,
    pub message: String,
    pub details: DiagnosticDetails,
}

impl Diagnostic {
    /// Создает новую диагностику
    pub fn new(
        severity: DiagnosticSeverity,
        location: Location,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            location,
            code: code.into(),
            message: message.into(),
            details: DiagnosticDetails {
                found: None,
                expected: None,
                info: None,
                suggestion: None,
            },
        }
    }

    /// Добавляет информацию о найденном значении
    pub fn with_found(mut self, found: impl Into<String>) -> Self {
        self.details.found = Some(found.into());
        self
    }

    /// Добавляет информацию об ожидаемом значении
    pub fn with_expected(mut self, expected: impl Into<String>) -> Self {
        self.details.expected = Some(expected.into());
        self
    }

    /// Добавляет дополнительную информацию
    pub fn with_info(mut self, info: impl Into<String>) -> Self {
        self.details.info = Some(info.into());
        self
    }

    /// Добавляет предложение по исправлению
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.details.suggestion = Some(suggestion.into());
        self
    }
}

/// Коды диагностик
pub mod codes {
    /// Синтаксическая ошибка
    pub const SYNTAX_ERROR: &str = "BSL001";
    /// Неизвестная конструкция языка
    pub const UNKNOWN_CONSTRUCT: &str = "BSL002";
    /// Вызов несуществующего метода
    pub const UNKNOWN_METHOD: &str = "BSL003";
    /// Неверное количество параметров
    pub const WRONG_PARAM_COUNT: &str = "BSL004";
    /// Обращение к несуществующему свойству
    pub const UNKNOWN_PROPERTY: &str = "BSL005";
    /// Несовместимые типы
    pub const TYPE_MISMATCH: &str = "BSL006";
    /// Необъявленная переменная
    pub const UNDECLARED_VARIABLE: &str = "BSL007";
    /// Неиспользуемая переменная
    pub const UNUSED_VARIABLE: &str = "BSL008";
    /// Неинициализированная переменная
    pub const UNINITIALIZED_VARIABLE: &str = "BSL009";
    /// Дублированный параметр
    pub const DUPLICATE_PARAMETER: &str = "BSL010";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_builder() {
        let location = Location::new("test.bsl".to_string(), 10, 5, 100, 10);
        let diag = Diagnostic::new(
            DiagnosticSeverity::Error,
            location,
            codes::UNKNOWN_METHOD,
            "Метод не найден",
        )
        .with_found("ПолучитьДанные")
        .with_expected("Получить, ПолучитьОбъект")
        .with_suggestion("Возможно, вы имели в виду 'Получить'?");

        assert_eq!(diag.code, codes::UNKNOWN_METHOD);
        assert_eq!(diag.details.found, Some("ПолучитьДанные".to_string()));
        assert!(diag.details.suggestion.is_some());
    }
}
