/*!
# BSL Syntax Extractor

Извлекатель синтаксиса BSL из документации 1С.
Портирован с Python проекта 1c-help-parser на Rust.

Основные возможности:
- Извлечение методов, объектов, свойств, функций из HTML документации
- Создание структурированной базы знаний BSL
- Поддержка поиска и автодополнения
- Классификация синтаксических элементов

## Использование

```rust
let mut extractor = BslSyntaxExtractor::new("1C_Help.hbk");
let database = extractor.extract_syntax_database(Some(1000))?;
let method_info = database.get_method_info("Сообщить");
```
*/

use std::collections::HashMap;
use std::path::Path;
use serde::{Deserialize, Serialize};
use regex::Regex;
use anyhow::{Context, Result};

use super::hbk_parser::{HbkArchiveParser, HtmlContent};

/// База знаний синтаксиса BSL (замена Python categorized syntax)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BslSyntaxDatabase {
    pub objects: HashMap<String, BslObjectInfo>,
    pub methods: HashMap<String, BslMethodInfo>,
    pub properties: HashMap<String, BslPropertyInfo>,
    pub functions: HashMap<String, BslFunctionInfo>,
    pub operators: HashMap<String, BslOperatorInfo>,
    pub keywords: Vec<String>,
}

/// Информация об объекте BSL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BslObjectInfo {
    pub name: String,
    pub object_type: String,
    pub description: Option<String>,
    pub methods: Vec<String>,
    pub properties: Vec<String>,
    pub constructors: Vec<String>,
    pub availability: Option<String>,
}

/// Информация о методе BSL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BslMethodInfo {
    pub name: String,
    pub syntax_variants: Vec<String>,
    pub parameters: Vec<ParameterInfo>,
    pub return_type: Option<String>,
    pub description: Option<String>,
    pub availability: Option<String>,
    pub version: Option<String>,
    pub examples: Vec<String>,
    pub object_context: Option<String>, // К какому объекту относится метод
}

/// Информация о свойстве BSL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BslPropertyInfo {
    pub name: String,
    pub property_type: String,
    pub access_mode: AccessMode,
    pub description: Option<String>,
    pub availability: Option<String>,
    pub object_context: Option<String>,
}

/// Информация о функции BSL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BslFunctionInfo {
    pub name: String,
    pub syntax_variants: Vec<String>,
    pub parameters: Vec<ParameterInfo>,
    pub return_type: Option<String>,
    pub description: Option<String>,
    pub category: String, // Global, String, Date, etc.
    pub availability: Option<String>,
}

/// Информация об операторе BSL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BslOperatorInfo {
    pub operator: String,
    pub syntax: String,
    pub description: Option<String>,
    pub precedence: u8,
}

/// Информация о параметре
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterInfo {
    pub name: String,
    pub param_type: Option<String>,
    pub description: Option<String>,
    pub is_optional: bool,
    pub default_value: Option<String>,
}

/// Режим доступа к свойству
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessMode {
    Read,
    Write,
    ReadWrite,
}

/// Элемент автодополнения для LSP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionItem {
    pub label: String,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub insert_text: Option<String>,
    pub kind: CompletionItemKind,
}

/// Тип элемента автодополнения
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompletionItemKind {
    Method,
    Function,
    Property,
    Object,
    Keyword,
}

/// Извлекатель синтаксиса BSL (замена Python BSLSyntaxExtractor)
pub struct BslSyntaxExtractor {
    hbk_parser: HbkArchiveParser,
    syntax_patterns: HashMap<String, Regex>,
}

impl BslSyntaxExtractor {
    /// Создает новый извлекатель синтаксиса
    pub fn new<P: AsRef<Path>>(hbk_archive_path: P) -> Self {
        let mut patterns = HashMap::new();
        
        // Компилируем регулярные выражения для извлечения синтаксиса
        if let Ok(syntax_regex) = Regex::new(r"Синтаксис:\s*(.+)") {
            patterns.insert("syntax".to_string(), syntax_regex);
        }
        if let Ok(params_regex) = Regex::new(r"Параметры:\s*(.+)") {
            patterns.insert("parameters".to_string(), params_regex);
        }
        if let Ok(return_regex) = Regex::new(r"Возвращаемое значение:\s*(.+)") {
            patterns.insert("return_value".to_string(), return_regex);
        }
        if let Ok(availability_regex) = Regex::new(r"Доступность:\s*(.+)") {
            patterns.insert("availability".to_string(), availability_regex);
        }
        if let Ok(version_regex) = Regex::new(r"Версия:\s*(.+)") {
            patterns.insert("version".to_string(), version_regex);
        }
        
        Self {
            hbk_parser: HbkArchiveParser::new(hbk_archive_path),
            syntax_patterns: patterns,
        }
    }
    
    /// Извлекает полную базу знаний синтаксиса BSL (замена Python extraction logic)
    pub fn extract_syntax_database(&mut self, max_files: Option<usize>) -> Result<BslSyntaxDatabase> {
        tracing::info!("Extracting BSL syntax database");
        
        let sample_content = self.hbk_parser.extract_sample_content(max_files.unwrap_or(usize::MAX))
            .context("Failed to extract sample content from HBK archive")?;
        
        let mut database = BslSyntaxDatabase {
            objects: HashMap::new(),
            methods: HashMap::new(),
            properties: HashMap::new(),
            functions: HashMap::new(),
            operators: HashMap::new(),
            keywords: Vec::new(),
        };
        
        // Обрабатываем каждый HTML файл
        for content in sample_content {
            if let Err(e) = self.categorize_and_extract_syntax(&content, &mut database) {
                tracing::warn!("Failed to extract syntax from content: {}", e);
            }
        }
        
        // Добавляем стандартные ключевые слова BSL
        self.add_standard_keywords(&mut database);
        
        tracing::info!(
            "Syntax database extracted: {} methods, {} objects, {} functions, {} properties",
            database.methods.len(),
            database.objects.len(),
            database.functions.len(),
            database.properties.len()
        );
        
        Ok(database)
    }
    
    /// Классифицирует и извлекает синтаксис из HTML контента
    fn categorize_and_extract_syntax(&self, content: &HtmlContent, database: &mut BslSyntaxDatabase) -> Result<()> {
        if let Some(title) = &content.title {
            let title_clean = title.trim();
            
            if title_clean.is_empty() {
                return Ok(());
            }
            
            // Определяем тип синтаксического элемента и извлекаем информацию
            if self.is_object_syntax(title_clean) {
                let object_info = self.extract_object_info(content)?;
                database.objects.insert(object_info.name.clone(), object_info);
            } else if self.is_method_syntax(title_clean) {
                let method_info = self.extract_method_info(content)?;
                database.methods.insert(method_info.name.clone(), method_info);
            } else if self.is_property_syntax(title_clean) {
                let property_info = self.extract_property_info(content)?;
                database.properties.insert(property_info.name.clone(), property_info);
            } else if self.is_function_syntax(title_clean) {
                let function_info = self.extract_function_info(content)?;
                database.functions.insert(function_info.name.clone(), function_info);
            } else if self.is_operator_syntax(title_clean) {
                let operator_info = self.extract_operator_info(content)?;
                database.operators.insert(operator_info.operator.clone(), operator_info);
            }
        }
        
        Ok(())
    }
    
    /// Извлекает информацию о методе из HTML контента
    fn extract_method_info(&self, content: &HtmlContent) -> Result<BslMethodInfo> {
        let method_name = self.extract_method_name(&content.title.as_deref().unwrap_or(""));
        
        let mut method_info = BslMethodInfo {
            name: method_name,
            syntax_variants: content.syntax.clone(),
            parameters: Vec::new(),
            return_type: None,
            description: content.description.clone(),
            availability: content.availability.clone(),
            version: content.version.clone(),
            examples: content.examples.clone(),
            object_context: None,
        };
        
        // Извлекаем параметры из синтаксиса
        for syntax in &content.syntax {
            let params = self.parse_parameters_from_syntax(syntax)?;
            method_info.parameters.extend(params);
        }
        
        // Извлекаем дополнительную информацию с помощью регулярных выражений
        if let Some(desc) = &content.description {
            self.extract_additional_info_from_description(desc, &mut method_info);
        }
        
        // Определяем контекст объекта
        method_info.object_context = self.extract_object_context(&method_info.name);
        
        Ok(method_info)
    }
    
    /// Извлекает информацию об объекте из HTML контента
    fn extract_object_info(&self, content: &HtmlContent) -> Result<BslObjectInfo> {
        let object_name = content.title.as_deref().unwrap_or("").to_string();
        
        let object_info = BslObjectInfo {
            name: object_name.clone(),
            object_type: content.object_type.as_deref().unwrap_or("object").to_string(),
            description: content.description.clone(),
            methods: Vec::new(), // TODO: извлечь из описания
            properties: Vec::new(), // TODO: извлечь из описания
            constructors: Vec::new(), // TODO: извлечь из описания
            availability: content.availability.clone(),
        };
        
        Ok(object_info)
    }
    
    /// Извлекает информацию о свойстве из HTML контента
    fn extract_property_info(&self, content: &HtmlContent) -> Result<BslPropertyInfo> {
        let property_name = content.title.as_deref().unwrap_or("").to_string();
        
        let property_info = BslPropertyInfo {
            name: property_name,
            property_type: "Variant".to_string(), // По умолчанию
            access_mode: AccessMode::ReadWrite, // По умолчанию
            description: content.description.clone(),
            availability: content.availability.clone(),
            object_context: None,
        };
        
        Ok(property_info)
    }
    
    /// Извлекает информацию о функции из HTML контента
    fn extract_function_info(&self, content: &HtmlContent) -> Result<BslFunctionInfo> {
        let function_name = self.extract_method_name(&content.title.as_deref().unwrap_or(""));
        
        let mut function_info = BslFunctionInfo {
            name: function_name,
            syntax_variants: content.syntax.clone(),
            parameters: Vec::new(),
            return_type: None,
            description: content.description.clone(),
            category: "Global".to_string(), // По умолчанию
            availability: content.availability.clone(),
        };
        
        // Извлекаем параметры из синтаксиса
        for syntax in &content.syntax {
            let params = self.parse_parameters_from_syntax(syntax)?;
            function_info.parameters.extend(params);
        }
        
        Ok(function_info)
    }
    
    /// Извлекает информацию об операторе из HTML контента
    fn extract_operator_info(&self, content: &HtmlContent) -> Result<BslOperatorInfo> {
        let operator_name = content.title.as_deref().unwrap_or("").to_string();
        
        let operator_info = BslOperatorInfo {
            operator: operator_name,
            syntax: content.syntax.first().cloned().unwrap_or_default(),
            description: content.description.clone(),
            precedence: 0, // TODO: определить приоритет
        };
        
        Ok(operator_info)
    }
    
    /// Извлекает имя метода/функции из заголовка
    fn extract_method_name(&self, title: &str) -> String {
        // Удаляем все после первой открывающей скобки
        if let Some(paren_pos) = title.find('(') {
            title[..paren_pos].trim().to_string()
        } else {
            title.trim().to_string()
        }
    }
    
    /// Парсит параметры из строки синтаксиса
    fn parse_parameters_from_syntax(&self, syntax: &str) -> Result<Vec<ParameterInfo>> {
        let mut parameters = Vec::new();
        
        // Ищем параметры в скобках
        if let Some(start) = syntax.find('(') {
            if let Some(end) = syntax.find(')') {
                let params_str = &syntax[start + 1..end];
                
                // Разбиваем параметры по запятым
                for param in params_str.split(',') {
                    let param = param.trim();
                    if !param.is_empty() {
                        let parameter_info = self.parse_single_parameter(param)?;
                        parameters.push(parameter_info);
                    }
                }
            }
        }
        
        Ok(parameters)
    }
    
    /// Парсит один параметр
    fn parse_single_parameter(&self, param: &str) -> Result<ParameterInfo> {
        let mut parameter = ParameterInfo {
            name: param.to_string(),
            param_type: None,
            description: None,
            is_optional: false,
            default_value: None,
        };
        
        // Проверяем наличие значения по умолчанию
        if let Some(equals_pos) = param.find('=') {
            parameter.name = param[..equals_pos].trim().to_string();
            parameter.default_value = Some(param[equals_pos + 1..].trim().to_string());
            parameter.is_optional = true;
        }
        
        // Проверяем опциональность по квадратным скобкам
        if param.starts_with('<') && param.ends_with('>') {
            parameter.is_optional = true;
            parameter.name = param[1..param.len() - 1].to_string();
        }
        
        Ok(parameter)
    }
    
    /// Извлекает дополнительную информацию из описания
    fn extract_additional_info_from_description(&self, description: &str, method_info: &mut BslMethodInfo) {
        // Извлекаем доступность
        if let Some(availability_regex) = self.syntax_patterns.get("availability") {
            if let Some(captures) = availability_regex.captures(description) {
                method_info.availability = Some(captures[1].trim().to_string());
            }
        }
        
        // Извлекаем версию
        if let Some(version_regex) = self.syntax_patterns.get("version") {
            if let Some(captures) = version_regex.captures(description) {
                method_info.version = Some(captures[1].trim().to_string());
            }
        }
        
        // Извлекаем возвращаемое значение
        if let Some(return_regex) = self.syntax_patterns.get("return_value") {
            if let Some(captures) = return_regex.captures(description) {
                method_info.return_type = Some(captures[1].trim().to_string());
            }
        }
    }
    
    /// Извлекает контекст объекта из имени метода
    fn extract_object_context(&self, method_name: &str) -> Option<String> {
        // Если имя содержит точку, то часть до точки - это объект
        if let Some(dot_pos) = method_name.find('.') {
            Some(method_name[..dot_pos].to_string())
        } else {
            None
        }
    }
    
    /// Проверяет, является ли заголовок синтаксисом объекта
    fn is_object_syntax(&self, title: &str) -> bool {
        let title_lower = title.to_lowercase();
        title_lower.contains("объект") || 
        title_lower.contains("коллекция") || 
        title_lower.contains("менеджер") ||
        title_lower.contains("object") ||
        title_lower.contains("collection") ||
        title_lower.contains("manager")
    }
    
    /// Проверяет, является ли заголовок синтаксисом метода
    fn is_method_syntax(&self, title: &str) -> bool {
        title.contains('(') && title.contains(')') && title.contains('.')
    }
    
    /// Проверяет, является ли заголовок синтаксисом свойства
    fn is_property_syntax(&self, title: &str) -> bool {
        !title.contains('(') && 
        !title.contains(')') && 
        title.contains('.') &&
        !self.is_object_syntax(title)
    }
    
    /// Проверяет, является ли заголовок синтаксисом функции
    fn is_function_syntax(&self, title: &str) -> bool {
        title.contains('(') && title.contains(')') && !title.contains('.')
    }
    
    /// Проверяет, является ли заголовок синтаксисом оператора
    fn is_operator_syntax(&self, title: &str) -> bool {
        let operators = ["+", "-", "*", "/", "=", "<>", "<", ">", "<=", ">=", "И", "ИЛИ", "НЕ"];
        operators.iter().any(|op| title.contains(op))
    }
    
    /// Добавляет стандартные ключевые слова BSL
    fn add_standard_keywords(&self, database: &mut BslSyntaxDatabase) {
        let keywords = vec![
            // Управляющие конструкции
            "Если", "Тогда", "Иначе", "ИначеЕсли", "КонецЕсли",
            "Пока", "Цикл", "КонецЦикла", "Для", "По", "КонецДля",
            "Попытка", "Исключение", "КонецПопытки", "ВызватьИсключение",
            "Возврат", "Продолжить", "Прервать",
            
            // Объявления
            "Процедура", "КонецПроцедуры", "Функция", "КонецФункции",
            "Экспорт", "Перем", "Знач",
            
            // Логические операторы
            "И", "ИЛИ", "НЕ", "Истина", "Ложь", "Неопределено", "NULL",
            
            // Типы данных
            "Число", "Строка", "Дата", "Булево", "Тип", "ТипЗнч",
            
            // Прочие
            "Новый", "Как"
        ];
        
        database.keywords = keywords.into_iter().map(|s| s.to_string()).collect();
    }
}

impl BslSyntaxDatabase {
    /// Поиск методов по запросу
    pub fn search_methods(&self, query: &str) -> Vec<&BslMethodInfo> {
        let query_lower = query.to_lowercase();
        self.methods
            .values()
            .filter(|method| {
                method.name.to_lowercase().contains(&query_lower) ||
                method.description.as_ref()
                    .map_or(false, |d| d.to_lowercase().contains(&query_lower))
            })
            .collect()
    }
    
    /// Получает элементы автодополнения для LSP
    pub fn get_completion_items(&self, prefix: &str) -> Vec<CompletionItem> {
        let mut items = Vec::new();
        let prefix_lower = prefix.to_lowercase();
        
        // Добавляем методы
        for method in self.methods.values() {
            if method.name.to_lowercase().starts_with(&prefix_lower) {
                items.push(CompletionItem {
                    label: method.name.clone(),
                    detail: method.syntax_variants.first().cloned(),
                    documentation: method.description.clone(),
                    insert_text: Some(self.generate_method_insert_text(method)),
                    kind: CompletionItemKind::Method,
                });
            }
        }
        
        // Добавляем функции
        for function in self.functions.values() {
            if function.name.to_lowercase().starts_with(&prefix_lower) {
                items.push(CompletionItem {
                    label: function.name.clone(),
                    detail: function.syntax_variants.first().cloned(),
                    documentation: function.description.clone(),
                    insert_text: Some(self.generate_function_insert_text(function)),
                    kind: CompletionItemKind::Function,
                });
            }
        }
        
        // Добавляем свойства
        for property in self.properties.values() {
            if property.name.to_lowercase().starts_with(&prefix_lower) {
                items.push(CompletionItem {
                    label: property.name.clone(),
                    detail: Some(property.property_type.clone()),
                    documentation: property.description.clone(),
                    insert_text: Some(property.name.clone()),
                    kind: CompletionItemKind::Property,
                });
            }
        }
        
        // Добавляем ключевые слова
        for keyword in &self.keywords {
            if keyword.to_lowercase().starts_with(&prefix_lower) {
                items.push(CompletionItem {
                    label: keyword.clone(),
                    detail: Some("Ключевое слово".to_string()),
                    documentation: None,
                    insert_text: Some(keyword.clone()),
                    kind: CompletionItemKind::Keyword,
                });
            }
        }
        
        items
    }
    
    /// Генерирует текст для вставки метода с параметрами
    fn generate_method_insert_text(&self, method: &BslMethodInfo) -> String {
        if method.parameters.is_empty() {
            format!("{}()", method.name)
        } else {
            let params: Vec<String> = method.parameters
                .iter()
                .enumerate()
                .map(|(i, param)| {
                    if param.is_optional {
                        format!("${{{i}:{}}}", param.name)
                    } else {
                        format!("${{{i}:{}}}", param.name)
                    }
                })
                .collect();
            format!("{}({})", method.name, params.join(", "))
        }
    }
    
    /// Генерирует текст для вставки функции с параметрами
    fn generate_function_insert_text(&self, function: &BslFunctionInfo) -> String {
        if function.parameters.is_empty() {
            format!("{}()", function.name)
        } else {
            let params: Vec<String> = function.parameters
                .iter()
                .enumerate()
                .map(|(i, param)| {
                    format!("${{{i}:{}}}", param.name)
                })
                .collect();
            format!("{}({})", function.name, params.join(", "))
        }
    }
    
    /// Получает информацию о методе по имени
    pub fn get_method_info(&self, method_name: &str) -> Option<&BslMethodInfo> {
        self.methods.get(method_name)
    }
    
    /// Получает информацию об объекте по имени
    pub fn get_object_info(&self, object_name: &str) -> Option<&BslObjectInfo> {
        self.objects.get(object_name)
    }
    
    /// Получает информацию о функции по имени
    pub fn get_function_info(&self, function_name: &str) -> Option<&BslFunctionInfo> {
        self.functions.get(function_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_method_name() {
        let extractor = BslSyntaxExtractor::new("test.hbk");
        
        assert_eq!(extractor.extract_method_name("Сообщить()"), "Сообщить");
        assert_eq!(extractor.extract_method_name("НайтиПоРеквизиту(Значение)"), "НайтиПоРеквизиту");
        assert_eq!(extractor.extract_method_name("Метод без скобок"), "Метод без скобок");
    }
    
    #[test]
    fn test_syntax_classification() {
        let extractor = BslSyntaxExtractor::new("test.hbk");
        
        assert!(extractor.is_method_syntax("Объект.Метод()"));
        assert!(extractor.is_function_syntax("ГлобальнаяФункция()"));
        assert!(extractor.is_property_syntax("СправочникСсылка.Наименование"));
        assert!(extractor.is_object_syntax("СправочникОбъект.Объект"));
    }
    
    #[test]
    fn test_parameter_parsing() {
        let extractor = BslSyntaxExtractor::new("test.hbk");
        
        let params = extractor.parse_parameters_from_syntax("Метод(Параметр1, Параметр2 = Значение)").unwrap();
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].name, "Параметр1");
        assert!(!params[0].is_optional);
        assert_eq!(params[1].name, "Параметр2");
        assert!(params[1].is_optional);
        assert_eq!(params[1].default_value, Some("Значение".to_string()));
    }
}