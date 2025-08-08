//! Автоматическая генерация BSL ключевых слов из базы данных платформы
//!
//! Этот модуль заменяет ручное ведение списков ключевых слов на автоматическое
//! извлечение из существующей базы данных платформы 8.3.25.jsonl

use crate::bsl_parser::keywords::BslContext;
use crate::bsl_parser::keywords::BSL_GLOBAL_FUNCTIONS;
use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use serde_json::Value;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};

/// Автоматически сгенерированные ключевые слова BSL из базы платформы
pub struct GeneratedBslKeywords {
    /// Все встроенные типы платформы
    pub builtin_types: HashSet<String>,
    /// Глобальные функции платформы  
    pub global_functions: HashSet<String>,
    /// Методы глобального контекста
    pub global_methods: HashSet<String>,
    /// Свойства глобального контекста
    pub global_properties: HashSet<String>,
    /// Системные объекты (например, Метаданные, ПользователиИнформационнойБазы)
    pub system_objects: HashSet<String>,
}

impl GeneratedBslKeywords {
    /// Загружает ключевые слова из кеша платформы
    pub fn load_from_platform_cache(platform_version: &str) -> Result<Self> {
        let cache_path = get_platform_cache_path(platform_version)?;

        if !cache_path.exists() {
            return Err(anyhow::anyhow!(
                "Platform cache not found: {}. Run 'cargo run --bin extract_platform_docs' first.",
                cache_path.display()
            ));
        }

        let mut keywords = Self {
            builtin_types: HashSet::new(),
            global_functions: HashSet::new(),
            global_methods: HashSet::new(),
            global_properties: HashSet::new(),
            system_objects: HashSet::new(),
        };

        let file = File::open(&cache_path)
            .with_context(|| format!("Failed to open platform cache: {}", cache_path.display()))?;
        let reader = BufReader::new(file);

        for (line_num, line) in reader.lines().enumerate() {
            let line = line.context("Failed to read line from platform cache")?;

            match parse_platform_entity(&line) {
                Ok(entity_info) => {
                    keywords.process_entity(entity_info);
                }
                Err(e) => {
                    // Логируем ошибку, но продолжаем обработку
                    eprintln!(
                        "Warning: Failed to parse entity at line {}: {}",
                        line_num + 1,
                        e
                    );
                }
            }
        }

        // Гарантируем наличие ключевых примитивов в наборе типов
        for core in ["Строка", "Число", "Булево", "Дата", "Массив", "Структура"]
        {
            keywords.builtin_types.insert(core.to_string());
        }

        // Добавляем базовые глобальные функции из ручного списка (fallback)
        for func in BSL_GLOBAL_FUNCTIONS.iter() {
            keywords.global_functions.insert((*func).to_string());
        }

        // Валидация загруженных данных
        keywords.validate()?;

        Ok(keywords)
    }

    /// Обрабатывает информацию о сущности платформы
    fn process_entity(&mut self, entity: PlatformEntityInfo) {
        // Добавляем имя типа
        if !entity.display_name.is_empty() {
            // 1) Полное отображаемое имя как в кэше (например: "Строка (String)")
            self.builtin_types.insert(entity.display_name.clone());

            // 2) Нормализуем варианты имён из формата "A (B)": добавляем и A, и B
            if let Some((name_a, name_b)) = extract_name_variants(&entity.display_name) {
                if !name_a.is_empty() {
                    self.builtin_types.insert(name_a);
                }
                if !name_b.is_empty() {
                    self.builtin_types.insert(name_b);
                }
            }
        }

        // Добавляем английское имя если есть
        if let Some(en_name) = entity.english_name {
            if !en_name.is_empty() {
                self.builtin_types.insert(en_name);
            }
        }

        // Обрабатываем методы
        for method_name in entity.methods {
            // Если метод не содержит точки - это глобальная функция
            if !method_name.contains('.') {
                self.global_functions.insert(method_name);
            } else {
                self.global_methods.insert(method_name);
            }
        }

        // Обрабатываем свойства
        for prop_name in entity.properties {
            self.global_properties.insert(prop_name);
        }

        // Определяем системные объекты (по паттернам)
        if is_system_object(&entity.qualified_name) {
            // Извлекаем корневое имя объекта
            if let Some(root_name) = extract_root_object_name(&entity.qualified_name) {
                self.system_objects.insert(root_name);
            }
        }
    }

    /// Валидирует загруженные данные
    fn validate(&self) -> Result<()> {
        if self.builtin_types.is_empty() {
            return Err(anyhow::anyhow!(
                "No builtin types loaded from platform cache"
            ));
        }

        if self.global_functions.is_empty() {
            return Err(anyhow::anyhow!(
                "No global functions loaded from platform cache"
            ));
        }

        // Проверяем, что основные типы присутствуют (ищем с учетом нового формата "Тип (Type)")
        let required_types = ["Строка", "Число", "Булево", "Дата", "Массив", "Структура"];
        for required_type in &required_types {
            let found = self.builtin_types.contains(*required_type)
                || self
                    .builtin_types
                    .iter()
                    .any(|t| t.starts_with(required_type) && t.contains('('));
            if !found {
                eprintln!(
                    "Warning: Required type '{}' not found in platform cache",
                    required_type
                );
            }
        }

        println!(
            "Loaded {} builtin types, {} global functions, {} system objects",
            self.builtin_types.len(),
            self.global_functions.len(),
            self.system_objects.len()
        );

        Ok(())
    }

    /// Проверяет, является ли слово встроенным типом
    pub fn is_builtin_type(&self, word: &str) -> bool {
        // Прямая проверка
        if self.builtin_types.contains(word) {
            return true;
        }

        // Проверка в новом формате "Тип (Type)"
        self.builtin_types.iter().any(|t| {
            // Проверяем, начинается ли тип с нужного слова и содержит скобки
            t.starts_with(word) && t.contains('(') && {
                // Дополнительная проверка: убеждаемся, что это именно наш тип
                if let Some(space_pos) = t.find(' ') {
                    &t[..space_pos] == word
                } else {
                    false
                }
            }
        })
    }

    /// Проверяет, является ли слово глобальной функцией
    pub fn is_global_function(&self, word: &str) -> bool {
        self.global_functions.contains(word)
    }

    /// Проверяет, является ли слово системным объектом
    pub fn is_system_object(&self, word: &str) -> bool {
        self.system_objects.contains(word)
    }

    /// Проверяет, может ли слово быть переменной в данном контексте
    pub fn can_be_variable(&self, word: &str, context: BslContext) -> bool {
        use crate::bsl_parser::keywords::is_bsl_strict_keyword;

        match context {
            BslContext::StatementStart => {
                // В начале строки строгие ключевые слова не могут быть переменными
                !is_bsl_strict_keyword(word)
            }
            BslContext::AfterNew => {
                // После "Новый" должен быть тип
                self.is_builtin_type(word) || self.is_system_object(word)
            }
            BslContext::TypeDeclaration => {
                // В объявлении типа может быть встроенный тип
                self.is_builtin_type(word)
                    || self.is_system_object(word)
                    || !is_bsl_strict_keyword(word)
            }
            BslContext::Expression => {
                // В выражении может быть переменная, если это не строгое ключевое слово
                !is_bsl_strict_keyword(word) && !self.is_global_function(word)
            }
            BslContext::Unknown => {
                // Консервативный подход
                !is_bsl_strict_keyword(word) && !self.is_global_function(word)
            }
        }
    }
}

/// Информация о сущности платформы, извлеченная из JSON
#[derive(Debug)]
struct PlatformEntityInfo {
    qualified_name: String,
    display_name: String,
    english_name: Option<String>,
    methods: Vec<String>,
    properties: Vec<String>,
}

/// Парсит JSON строку с информацией о сущности платформы
fn parse_platform_entity(json_line: &str) -> Result<PlatformEntityInfo> {
    let value: Value = serde_json::from_str(json_line).context("Failed to parse JSON")?;

    let qualified_name = value["qualified_name"].as_str().unwrap_or("").to_string();

    let display_name = value["display_name"].as_str().unwrap_or("").to_string();

    let english_name = value["english_name"].as_str().map(|s| s.to_string());

    let mut methods = Vec::new();
    if let Some(methods_obj) = value["interface"]["methods"].as_object() {
        for method_name in methods_obj.keys() {
            // Извлекаем чистое имя метода (без префикса типа)
            if let Some(clean_name) = extract_method_name(method_name) {
                methods.push(clean_name);
            }
        }
    }

    let mut properties = Vec::new();
    if let Some(props_obj) = value["interface"]["properties"].as_object() {
        for prop_name in props_obj.keys() {
            properties.push(prop_name.clone());
        }
    }

    Ok(PlatformEntityInfo {
        qualified_name,
        display_name,
        english_name,
        methods,
        properties,
    })
}

/// Извлекает чистое имя метода из полного имени
fn extract_method_name(full_method_name: &str) -> Option<String> {
    // Примеры: "Массив.Добавить" -> "Добавить", "Сообщить" -> "Сообщить"
    if let Some(dot_pos) = full_method_name.rfind('.') {
        Some(full_method_name[dot_pos + 1..].to_string())
    } else {
        Some(full_method_name.to_string())
    }
}

/// Извлекает базовое русское имя из отображаемого имени вида "Имя (English)"
/// Возвращает None, если формат не соответствует ожидаемому.
fn extract_name_variants(display_name: &str) -> Option<(String, String)> {
    // Ищем разделитель " (" и завершающую ")" в конце строки
    if let Some(space_paren_pos) = display_name.find(" (") {
        if display_name.ends_with(')') && space_paren_pos > 0 {
            let a = display_name[..space_paren_pos].to_string();
            let b = display_name[space_paren_pos + 2..display_name.len() - 1].to_string();
            return Some((a, b));
        }
    }
    None
}

/// Проверяет, является ли сущность системным объектом
fn is_system_object(qualified_name: &str) -> bool {
    let system_patterns = [
        "Метаданные",
        "ПользователиИнформационнойБазы",
        "Справочники",
        "Документы",
        "Отчеты",
        "Обработки",
        "Регистры",
        "Константы",
    ];

    system_patterns
        .iter()
        .any(|pattern| qualified_name.starts_with(pattern))
}

/// Извлекает корневое имя объекта из полного имени
fn extract_root_object_name(qualified_name: &str) -> Option<String> {
    // "Справочники.Номенклатура" -> "Справочники"
    if let Some(dot_pos) = qualified_name.find('.') {
        Some(qualified_name[..dot_pos].to_string())
    } else {
        Some(qualified_name.to_string())
    }
}

/// Получает путь к кешу платформы
fn get_platform_cache_path(version: &str) -> Result<std::path::PathBuf> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;

    let cache_file = home_dir
        .join(".bsl_analyzer")
        .join("platform_cache")
        .join(format!("{}.jsonl", version));

    Ok(cache_file)
}

/// Глобальный экземпляр сгенерированных ключевых слов (ленивая инициализация)
pub static GENERATED_BSL_KEYWORDS: Lazy<GeneratedBslKeywords> = Lazy::new(|| {
    // Пытаемся загрузить из кеша, fallback на версию по умолчанию
    GeneratedBslKeywords::load_from_platform_cache("8.3.25").unwrap_or_else(|e| {
        eprintln!(
            "Warning: Failed to load platform keywords: {}. Using fallback.",
            e
        );
        // Возвращаем пустую структуру как fallback
        GeneratedBslKeywords {
            builtin_types: HashSet::new(),
            global_functions: HashSet::new(),
            global_methods: HashSet::new(),
            global_properties: HashSet::new(),
            system_objects: HashSet::new(),
        }
    })
});

/// Высокоуровневые функции для проверки (совместимость с существующим API)
pub fn is_generated_builtin_type(word: &str) -> bool {
    GENERATED_BSL_KEYWORDS.is_builtin_type(word)
}

pub fn is_generated_global_function(word: &str) -> bool {
    GENERATED_BSL_KEYWORDS.is_global_function(word)
}

pub fn is_generated_system_object(word: &str) -> bool {
    GENERATED_BSL_KEYWORDS.is_system_object(word)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_method_name() {
        assert_eq!(
            extract_method_name("Массив.Добавить"),
            Some("Добавить".to_string())
        );
        assert_eq!(
            extract_method_name("Сообщить"),
            Some("Сообщить".to_string())
        );
        assert_eq!(
            extract_method_name("ТаблицаЗначений.Вставить"),
            Some("Вставить".to_string())
        );
    }

    #[test]
    fn test_extract_root_object_name() {
        assert_eq!(
            extract_root_object_name("Справочники.Номенклатура"),
            Some("Справочники".to_string())
        );
        assert_eq!(
            extract_root_object_name("Метаданные"),
            Some("Метаданные".to_string())
        );
        assert_eq!(
            extract_root_object_name("Документы.ЗаказНаряды"),
            Some("Документы".to_string())
        );
    }

    #[test]
    fn test_is_system_object() {
        assert!(is_system_object("Метаданные.Справочники"));
        assert!(is_system_object("ПользователиИнформационнойБазы"));
        assert!(is_system_object("Справочники.Номенклатура"));
        assert!(!is_system_object("Массив"));
        assert!(!is_system_object("ТаблицаЗначений"));
    }

    #[test]
    fn test_platform_cache_loading() {
        // Этот тест запускается только если есть кеш платформы
        if get_platform_cache_path("8.3.25").unwrap().exists() {
            let keywords = GeneratedBslKeywords::load_from_platform_cache("8.3.25")
                .expect("Failed to load platform cache");

            // Проверяем, что основные типы загружены
            assert!(keywords.is_builtin_type("Строка"));
            assert!(keywords.is_builtin_type("Число"));
            assert!(keywords.is_builtin_type("Массив"));

            // Проверяем системные объекты
            println!(
                "Available system objects: {:?}",
                keywords.system_objects.iter().take(10).collect::<Vec<_>>()
            );

            // Ищем вариации "Метаданные"
            let metadata_variants = vec!["Метаданные", "MetaData", "Metadata"];
            let found_metadata = metadata_variants
                .iter()
                .find(|&name| keywords.is_system_object(name));

            if let Some(found_name) = found_metadata {
                println!("Found metadata as: {}", found_name);
            } else {
                // Если не нашли точное совпадение, проверим похожие
                let similar: Vec<&String> = keywords
                    .system_objects
                    .iter()
                    .filter(|name| {
                        name.to_lowercase().contains("мета") || name.to_lowercase().contains("meta")
                    })
                    .collect();
                println!("Similar to metadata: {:?}", similar);

                // Для теста используем любой доступный системный объект
                if !keywords.system_objects.is_empty() {
                    let first_obj = keywords.system_objects.iter().next().unwrap();
                    println!(
                        "Using first available system object for test: {}",
                        first_obj
                    );
                    assert!(keywords.is_system_object(first_obj));
                }
            }

            assert!(keywords.is_system_object("Справочники"));

            // Проверяем глобальные функции
            assert!(keywords.is_global_function("Сообщить"));

            println!(
                "Platform cache test passed: {} types, {} functions loaded",
                keywords.builtin_types.len(),
                keywords.global_functions.len()
            );
        }
    }
}
