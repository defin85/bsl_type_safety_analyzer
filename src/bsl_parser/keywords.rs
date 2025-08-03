//! BSL Keywords and Language Constructs
//! 
//! Этот модуль поддерживает две стратегии работы с ключевыми словами BSL:
//! 1. Ручные списки (для базовых случаев и fallback)
//! 2. Автоматическая генерация из базы данных платформы (рекомендуется)

use std::collections::HashSet;
use once_cell::sync::Lazy;

// Импортируем автоматический генератор ключевых слов
pub mod keyword_generator;
pub use keyword_generator::{GeneratedBslKeywords, GENERATED_BSL_KEYWORDS};

/// Строгие ключевые слова BSL (никогда не могут быть переменными)
pub static BSL_STRICT_KEYWORDS_RU: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    HashSet::from([
        // Объявления
        "Процедура", "КонецПроцедуры", "Функция", "КонецФункции", "Перем", "Экспорт",
        
        // Управление потоком  
        "Если", "Тогда", "ИначеЕсли", "Иначе", "КонецЕсли",
        "Пока", "Цикл", "КонецЦикла", "Для", "По", "До", "Каждого", "Из",
        "Попытка", "Исключение", "ВызватьИсключение", "КонецПопытки",
        "Прервать", "Продолжить", "Возврат",
        
        // Операторы
        "И", "ИЛИ", "НЕ", "Новый",
        
        // Области
        "Область", "КонецОбласти",
        
        // Директивы препроцессора
        "НаКлиенте", "НаСервере", "НаКлиентеНаСервере", "НаСервереБезКонтекста",
        
        // Модификаторы
        "Знач", "ПерефВыз", "ИмяПодсистемы",
        
        // Условная компиляция
        "ВебКлиент", "ТонкийКлиент", "ТолстыйКлиентОбычноеПриложение", 
        "ТолстыйКлиентУправляемоеПриложение", "Сервер", "ВнешнееСоединение",
        "МобильноеПриложениеКлиент", "МобильноеПриложениеСервер", "МобильныйКлиент",
    ])
});

/// Литералы BSL (могут появляться как значения)
pub static BSL_LITERALS_RU: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    HashSet::from([
        "Истина", "Ложь", "Неопределено", "NULL",
    ])
});

/// Русские ключевые слова BSL (объединение строгих и литералов)
pub static BSL_KEYWORDS_RU: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let mut keywords = BSL_STRICT_KEYWORDS_RU.clone();
    keywords.extend(BSL_LITERALS_RU.iter());
    keywords
});

/// Английские ключевые слова BSL
pub static BSL_KEYWORDS_EN: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    HashSet::from([
        // Declarations
        "Procedure", "EndProcedure", "Function", "EndFunction", "Var", "Export",
        
        // Control flow
        "If", "Then", "ElsIf", "Else", "EndIf",
        "While", "Do", "EndDo", "For", "To", "Each", "In",
        "Try", "Except", "Raise", "EndTry",
        "Break", "Continue", "Return",
        
        // Data types
        "True", "False", "Undefined", "NULL",
        
        // Operators
        "And", "Or", "Not", "New",
        
        // Regions
        "Region", "EndRegion",
        
        // Preprocessor directives
        "AtClient", "AtServer", "AtClientAtServer", "AtServerNoContext",
        
        // Modifiers
        "Val", "Var",
        
        // Conditional compilation
        "WebClient", "ThinClient", "ThickClientOrdinaryApplication",
        "ThickClientManagedApplication", "Server", "ExternalConnection",
        "MobileAppClient", "MobileAppServer", "MobileClient",
    ])
});

/// Основные встроенные типы BSL (могут использоваться как типы переменных)
pub static BSL_BUILTIN_TYPES: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    HashSet::from([
        // Основные типы
        "Строка", "String", "Число", "Number", "Булево", "Boolean", 
        "Дата", "Date",
        
        // Коллекции
        "Массив", "Array", "Структура", "Structure", "Соответствие", "Map",
        "СписокЗначений", "ValueList", "ТаблицаЗначений", "ValueTable",
        "ДеревоЗначений", "ValueTree",
        
        // Файловая система
        "Файл", "File", "ТекстовыйДокумент", "TextDocument",
        
        // Системные
        "СистемнаяИнформация", "SystemInfo", "ПользователиИнформационнойБазы", "InfoBaseUsers",
        "Метаданные", "Metadata",
    ])
});

/// Объекты платформы 1С (глобальные объекты конфигурации)
pub static BSL_PLATFORM_OBJECTS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    HashSet::from([
        // Справочники и документы (общие префиксы)
        "Справочники", "Catalogs", "Документы", "Documents", 
        "Регистры", "Registers", "ОтчетыИОбработки", "ReportsAndDataProcessors",
        "Отчеты", "Reports", "Обработки", "DataProcessors",
        "РегистрыСведений", "InformationRegisters", "РегистрыНакопления", "AccumulationRegisters",
        
        // Глобальные объекты
        "ПользователиИнформационнойБазы", "InfoBaseUsers",
        "Метаданные", "Metadata", "КонстантыМенеджер", "ConstantsManager",
    ])
});

/// Операторы BSL
pub static BSL_OPERATORS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    HashSet::from([
        "+", "-", "*", "/", "%", "=", "<>", "!=", "<", ">", "<=", ">=",
        "И", "ИЛИ", "НЕ", "And", "Or", "Not",
    ])
});

/// Контекст использования идентификатора в BSL коде
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BslContext {
    /// В начале строки (может быть ключевым словом)
    StatementStart,
    /// После "Новый" (тип для создания объекта)
    AfterNew,
    /// После "Как" в объявлении переменной (тип переменной)
    TypeDeclaration,
    /// В выражении (переменная или метод)
    Expression,
    /// Неизвестный контекст
    Unknown,
}

/// Проверяет, является ли строка строгим ключевым словом BSL
pub fn is_bsl_strict_keyword(word: &str) -> bool {
    BSL_STRICT_KEYWORDS_RU.contains(word) || BSL_KEYWORDS_EN.contains(word)
}

/// Проверяет, является ли строка ключевым словом BSL
pub fn is_bsl_keyword(word: &str) -> bool {
    BSL_KEYWORDS_RU.contains(word) || BSL_KEYWORDS_EN.contains(word)
}

/// Проверяет, является ли строка встроенным типом BSL
/// Комбинирует ручные списки и автоматически сгенерированные данные
pub fn is_bsl_builtin_type(word: &str) -> bool {
    // Сначала проверяем ручной список (быстро)
    if BSL_BUILTIN_TYPES.contains(word) {
        return true;
    }
    
    // Затем проверяем автоматически сгенерированные типы
    GENERATED_BSL_KEYWORDS.is_builtin_type(word)
}

/// Проверяет, является ли строка объектом платформы BSL
/// Комбинирует ручные списки и автоматически сгенерированные данные
pub fn is_bsl_platform_object(word: &str) -> bool {
    // Сначала проверяем ручной список (быстро)
    if BSL_PLATFORM_OBJECTS.contains(word) {
        return true;
    }
    
    // Затем проверяем автоматически сгенерированные системные объекты
    GENERATED_BSL_KEYWORDS.is_system_object(word)
}

/// Проверяет, является ли строка оператором BSL
pub fn is_bsl_operator(word: &str) -> bool {
    BSL_OPERATORS.contains(word)
}

/// Контекстно-зависимая проверка, может ли строка быть переменной
pub fn can_be_variable(word: &str, context: BslContext) -> bool {
    match context {
        BslContext::StatementStart => {
            // В начале строки строгие ключевые слова не могут быть переменными
            !is_bsl_strict_keyword(word)
        }
        BslContext::AfterNew => {
            // После "Новый" должен быть тип, не может быть переменной
            false
        }
        BslContext::TypeDeclaration => {
            // В объявлении типа может быть встроенный тип или объект платформы
            // но не строгое ключевое слово
            !is_bsl_strict_keyword(word)
        }
        BslContext::Expression => {
            // В выражении может быть переменная, если это не строгое ключевое слово
            !is_bsl_strict_keyword(word)
        }
        BslContext::Unknown => {
            // По умолчанию - консервативный подход
            !is_bsl_strict_keyword(word)
        }
    }
}

/// Проверяет, является ли строка служебным словом BSL в данном контексте
pub fn is_bsl_reserved_word_in_context(word: &str, context: BslContext) -> bool {
    match context {
        BslContext::StatementStart => {
            // В начале строки - все ключевые слова зарезервированы
            is_bsl_keyword(word) || is_bsl_operator(word)
        }
        BslContext::AfterNew => {
            // После "Новый" - только типы, не ключевые слова
            is_bsl_builtin_type(word) || is_bsl_platform_object(word)
        }
        BslContext::TypeDeclaration => {
            // В объявлении типа - типы и некоторые ключевые слова
            is_bsl_builtin_type(word) || is_bsl_platform_object(word) || is_bsl_strict_keyword(word)
        }
        BslContext::Expression => {
            // В выражении - строгие ключевые слова и операторы
            is_bsl_strict_keyword(word) || is_bsl_operator(word)
        }
        BslContext::Unknown => {
            // По умолчанию - все
            is_bsl_keyword(word) || is_bsl_builtin_type(word) || is_bsl_operator(word)
        }
    }
}

/// Совместимость со старым API
pub fn is_bsl_reserved_word(word: &str) -> bool {
    is_bsl_reserved_word_in_context(word, BslContext::Unknown)
}

/// Список глобальных функций BSL, которые должны распознаваться как вызовы функций
pub static BSL_GLOBAL_FUNCTIONS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    HashSet::from([
        // Системные функции
        "Сообщить", "Message", "ПредупреждениеОБезопасности", "SecurityWarning",
        "ВопросДаНет", "QuestionYesNo", "Вопрос", "Question",
        "ТекущаяДата", "CurrentDate", "ТекущаяДатаСеанса", "CurrentSessionDate",
        "Формат", "Format", "СтрДлина", "StrLen", "СтрСравнить", "StrCompare",
        "ВРег", "Upper", "НРег", "Lower", "СокрЛП", "TrimAll", "СокрП", "TrimR", "СокрЛ", "TrimL",
        "Лев", "Left", "Прав", "Right", "Сред", "Mid", "Найти", "Find", "СтрЗаменить", "StrReplace",
        "СтрСоединить", "StrConcat", "СтрРазделить", "StrSplit",
        
        // Функции преобразования типов
        "Строка", "String", "Число", "Number", "Булево", "Boolean", "Дата", "Date",
        "ТипЗнч", "TypeOf", "XMLСтрока", "XMLString", "XMLЗначение", "XMLValue",
        
        // Функции для работы с файлами
        "УдалитьФайлы", "DeleteFiles", "КопироватьФайл", "FileCopy", "ПереместитьФайл", "MoveFile",
        "КаталогВременныхФайлов", "TempFilesDir", "КаталогПрограммы", "BinDir",
        "ПолучитьИмяВременногоФайла", "GetTempFileName",
        
        // Математические функции
        "Цел", "Int", "Окр", "Round", "Мин", "Min", "Макс", "Max", "Абс", "Abs",
        "Cos", "Sin", "Tan", "Exp", "Log", "Log10", "Pow", "Sqrt",
        
        // JSON функции (НЕ СУЩЕСТВУЮЩИЕ - для тестов)
        // "ПрочитатьJSONИзСтроки", // НЕ СУЩЕСТВУЕТ
        // "ПрочитатьJSONВЗначение", // НЕ СУЩЕСТВУЕТ 
        
        // Правильные JSON функции (если есть)
        "ПрочитатьJSON", "ReadJSON", "ЗаписатьJSON", "WriteJSON",
        
        // Функции инициализации
        "ОписаниеОшибки", "ErrorDescription", "КодВозврата", "ExitCode",
        "ПолучитьОбщийМодуль", "GetCommonModule",
    ])
});

/// Проверяет, является ли строка глобальной функцией BSL
/// Комбинирует ручные списки и автоматически сгенерированные данные
pub fn is_bsl_global_function(word: &str) -> bool {
    // Сначала проверяем ручной список (быстро)
    if BSL_GLOBAL_FUNCTIONS.contains(word) {
        return true;
    }
    
    // Затем проверяем автоматически сгенерированные функции
    GENERATED_BSL_KEYWORDS.is_global_function(word)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strict_keywords() {
        assert!(is_bsl_strict_keyword("Если"));
        assert!(is_bsl_strict_keyword("Попытка"));
        assert!(is_bsl_strict_keyword("Процедура"));
        assert!(!is_bsl_strict_keyword("ТаблицаЗначений")); // Тип, не ключевое слово
        assert!(!is_bsl_strict_keyword("МояПеременная"));
    }

    #[test]
    fn test_builtin_types() {
        assert!(is_bsl_builtin_type("Строка"));
        assert!(is_bsl_builtin_type("Array"));
        assert!(is_bsl_builtin_type("ТаблицаЗначений"));
        assert!(!is_bsl_builtin_type("МойТип"));
        assert!(!is_bsl_builtin_type("Попытка")); // Ключевое слово, не тип
    }

    #[test]
    fn test_platform_objects() {
        assert!(is_bsl_platform_object("Справочники"));
        assert!(is_bsl_platform_object("Метаданные"));
        assert!(is_bsl_platform_object("ПользователиИнформационнойБазы"));
        assert!(!is_bsl_platform_object("ТаблицаЗначений")); // Встроенный тип
    }

    #[test]
    fn test_context_dependent_variables() {
        // В выражении ТаблицаЗначений может быть переменной
        assert!(can_be_variable("ТаблицаЗначений", BslContext::Expression));
        
        // В начале строки Попытка не может быть переменной
        assert!(!can_be_variable("Попытка", BslContext::StatementStart));
        
        // После "Новый" не может быть переменной
        assert!(!can_be_variable("ТаблицаЗначений", BslContext::AfterNew));
        
        // В объявлении типа может быть типом
        assert!(can_be_variable("ТаблицаЗначений", BslContext::TypeDeclaration));
    }

    #[test]
    fn test_global_functions() {
        assert!(is_bsl_global_function("Сообщить"));
        assert!(is_bsl_global_function("Message"));
        assert!(is_bsl_global_function("ТекущаяДата"));
        assert!(!is_bsl_global_function("ПрочитатьJSONИзСтроки")); // НЕ СУЩЕСТВУЕТ
    }

    #[test]
    fn test_context_reserved_words() {
        // В начале строки Попытка зарезервировано
        assert!(is_bsl_reserved_word_in_context("Попытка", BslContext::StatementStart));
        
        // В выражении ТаблицаЗначений не зарезервировано (может быть переменной)
        assert!(!is_bsl_reserved_word_in_context("ТаблицаЗначений", BslContext::Expression));
        
        // После "Новый" ТаблицаЗначений зарезервировано как тип
        assert!(is_bsl_reserved_word_in_context("ТаблицаЗначений", BslContext::AfterNew));
    }
}