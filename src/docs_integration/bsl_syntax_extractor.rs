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

```rust,ignore
let mut extractor = BslSyntaxExtractor::new("1C_Help.hbk");
let database = extractor.extract_syntax_database(Some(1000))?;
let method_info = database.get_method_info("Сообщить");
```
*/

use anyhow::Result;
use regex::Regex;
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use super::{HbkArchiveParser, LinkInfo};

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
    pub english_name: Option<String>,
    pub syntax_variants: Vec<SyntaxVariant>,
    pub parameters: Vec<ParameterInfo>,
    pub parameters_by_variant: HashMap<String, Vec<ParameterInfo>>,
    pub return_type: Option<String>,
    pub return_type_description: Option<String>,
    pub description: Option<String>,
    pub availability: Vec<String>,
    pub version: Option<String>,
    pub examples: Vec<String>,
    pub object_context: Option<String>, // К какому объекту относится метод
    pub links: Vec<LinkInfo>,
}

/// Вариант синтаксиса метода
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxVariant {
    pub variant_name: String,
    pub syntax: String,
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
    pub type_description: Option<String>,
    pub description: Option<String>,
    pub is_optional: bool,
    pub default_value: Option<String>,
    pub link: Option<String>,
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

/// Информация об элементах коллекции
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionElementsInfo {
    pub description: Option<String>,
    pub usage: Option<String>,
    pub element_type: Option<String>,
}

/// Извлекатель синтаксиса BSL (замена Python BSLSyntaxExtractor)
pub struct BslSyntaxExtractor {
    context_parser: HbkArchiveParser, // для shcntx архива (объекты, методы)
    language_parser: Option<HbkArchiveParser>, // для shlang архива (примитивные типы, директивы)
    #[allow(dead_code)]
    syntax_patterns: HashMap<String, Regex>,
    type_mapping: HashMap<String, TypeInfo>,
}

/// Информация о типе
#[derive(Debug, Clone)]
struct TypeInfo {
    type_name: String,
    description: String,
}

impl BslSyntaxExtractor {
    /// Извлекает тип и описание из ссылки v8help
    fn extract_type_from_link(&self, link: &str) -> (String, String) {
        if link.is_empty() {
            return (String::new(), String::new());
        }

        // Базовые типы языка
        if link.contains("def_") {
            let type_key = link.split("def_").last().unwrap_or("");
            if let Some(type_info) = self.type_mapping.get(type_key) {
                return (type_info.type_name.clone(), type_info.description.clone());
            } else {
                return (type_key.to_string(), format!("Базовый тип: {}", type_key));
            }
        }

        // Объектные типы
        if link.contains("objects/") {
            let object_path = link
                .split("objects/")
                .last()
                .unwrap_or("")
                .replace(".html", "");
            let object_name = object_path.split('/').next_back().unwrap_or("");

            if let Some(type_info) = self.type_mapping.get(object_name) {
                return (type_info.type_name.clone(), type_info.description.clone());
            } else {
                return (object_name.to_string(), format!("Объект: {}", object_name));
            }
        }

        (String::new(), String::new())
    }

    /// Извлекает варианты синтаксиса
    fn extract_syntax_variants(&self, document: &Html, syntax_info: &mut SyntaxInfo) -> Result<()> {
        let chapter_selector = Selector::parse("p.V8SH_chapter").unwrap();
        let mut current_variant: Option<String> = None;

        for elem in document.select(&chapter_selector) {
            let text = elem.text().collect::<String>().trim().to_string();

            if text.contains("Вариант синтаксиса:") {
                // Начинаем новый вариант
                current_variant = Some(text.replace("Вариант синтаксиса:", "").trim().to_string());
            } else if text.contains("Синтаксис:") || text.contains("Синтаксис") {
                // Ищем синтаксис
                if let Some(syntax_text) = self.get_next_text_content(&elem) {
                    if let Some(variant_name) = &current_variant {
                        syntax_info.syntax_variants.push(SyntaxVariant {
                            variant_name: variant_name.clone(),
                            syntax: syntax_text.clone(),
                        });
                    } else {
                        // Обычный синтаксис (без вариантов)
                        syntax_info.syntax = syntax_text;
                    }
                }
            }
        }

        // Если есть варианты, используем первый как основной синтаксис
        if !syntax_info.syntax_variants.is_empty() && syntax_info.syntax.is_empty() {
            syntax_info.syntax = syntax_info.syntax_variants[0].syntax.clone();
        }

        Ok(())
    }

    /// Получает следующий текстовый контент после элемента
    fn get_next_text_content(&self, elem: &ElementRef) -> Option<String> {
        let mut current = elem.next_sibling();
        while let Some(node) = current {
            if let Some(elem_ref) = ElementRef::wrap(node) {
                let tag_name = elem_ref.value().name();
                if tag_name != "p"
                    || !elem_ref
                        .value()
                        .attr("class")
                        .unwrap_or("")
                        .contains("V8SH_chapter")
                {
                    let text = elem_ref.text().collect::<String>().trim().to_string();
                    if !text.is_empty() && text != "Параметры:" {
                        return Some(text);
                    }
                }
            } else if let Some(text_node) = node.value().as_text() {
                let text = text_node.trim();
                if !text.is_empty() {
                    return Some(text.to_string());
                }
            }
            current = node.next_sibling();
        }
        None
    }

    /// Извлекает описание
    fn extract_description(&self, document: &Html, syntax_info: &mut SyntaxInfo) -> Result<()> {
        let chapter_selector = Selector::parse("p.V8SH_chapter").unwrap();

        for elem in document.select(&chapter_selector) {
            let text = elem.text().collect::<String>().trim().to_string();
            if text.contains("Описание") {
                let _p_selector = Selector::parse("p").unwrap();
                if let Some(desc_elem) =
                    elem.next_siblings().filter_map(ElementRef::wrap).find(|e| {
                        e.value().name() == "p"
                            && !e
                                .value()
                                .attr("class")
                                .unwrap_or("")
                                .contains("V8SH_chapter")
                    })
                {
                    syntax_info.description =
                        desc_elem.text().collect::<String>().trim().to_string();
                }
                break;
            }
        }

        Ok(())
    }

    /// Извлекает доступность
    fn extract_availability(&self, document: &Html, syntax_info: &mut SyntaxInfo) -> Result<()> {
        let chapter_selector = Selector::parse("p.V8SH_chapter").unwrap();

        for elem in document.select(&chapter_selector) {
            let text = elem.text().collect::<String>().trim().to_string();
            if text.contains("Доступность") {
                if let Some(avail_elem) = elem
                    .next_siblings()
                    .filter_map(ElementRef::wrap)
                    .find(|e| e.value().name() == "p")
                {
                    let availability_text =
                        avail_elem.text().collect::<String>().trim().to_string();
                    // Разбиваем по запятым и очищаем
                    syntax_info.availability = availability_text
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .collect();
                }
                break;
            }
        }

        Ok(())
    }
    /// Создает новый извлекатель синтаксиса с поддержкой двух архивов
    pub fn new<P: AsRef<Path>>(context_archive_path: P) -> Self {
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

        // Инициализируем справочник типов
        let mut type_mapping = HashMap::new();
        type_mapping.insert(
            "def_String".to_string(),
            TypeInfo {
                type_name: "String".to_string(),
                description: "Строковый тип данных".to_string(),
            },
        );
        type_mapping.insert(
            "def_Number".to_string(),
            TypeInfo {
                type_name: "Number".to_string(),
                description: "Числовой тип данных".to_string(),
            },
        );
        type_mapping.insert(
            "def_Boolean".to_string(),
            TypeInfo {
                type_name: "Boolean".to_string(),
                description: "Логический тип данных".to_string(),
            },
        );
        type_mapping.insert(
            "def_BooleanTrue".to_string(),
            TypeInfo {
                type_name: "Boolean".to_string(),
                description: "Логический тип данных (Истина)".to_string(),
            },
        );
        type_mapping.insert(
            "def_Date".to_string(),
            TypeInfo {
                type_name: "Date".to_string(),
                description: "Тип данных Дата".to_string(),
            },
        );
        type_mapping.insert(
            "def_Time".to_string(),
            TypeInfo {
                type_name: "Time".to_string(),
                description: "Тип данных Время".to_string(),
            },
        );
        type_mapping.insert(
            "Array".to_string(),
            TypeInfo {
                type_name: "Array".to_string(),
                description: "Массив значений".to_string(),
            },
        );
        type_mapping.insert(
            "Structure".to_string(),
            TypeInfo {
                type_name: "Structure".to_string(),
                description: "Структура данных".to_string(),
            },
        );
        type_mapping.insert(
            "ValueTable".to_string(),
            TypeInfo {
                type_name: "ValueTable".to_string(),
                description: "Таблица значений".to_string(),
            },
        );
        type_mapping.insert(
            "FormDataCollectionItem".to_string(),
            TypeInfo {
                type_name: "FormDataCollectionItem".to_string(),
                description: "Элемент коллекции данных формы".to_string(),
            },
        );
        type_mapping.insert(
            "FormDataTreeItem".to_string(),
            TypeInfo {
                type_name: "FormDataTreeItem".to_string(),
                description: "Элемент дерева данных формы".to_string(),
            },
        );

        // Автоматически пытаемся найти языковой архив (shlang) рядом с контекстным (shcntx)
        let context_path = context_archive_path.as_ref();
        let language_parser = Self::auto_detect_language_archive(context_path);

        Self {
            context_parser: HbkArchiveParser::new(context_archive_path),
            language_parser,
            syntax_patterns: patterns,
            type_mapping,
        }
    }

    /// Автоматически определяет путь к языковому архиву на основе контекстного
    fn auto_detect_language_archive(context_path: &Path) -> Option<HbkArchiveParser> {
        // Получаем имя файла контекстного архива
        let context_file_name = context_path.file_name()?.to_str()?;

        // Если это shcntx архив, пытаемся найти соответствующий shlang архив
        if context_file_name.contains("shcntx") {
            let language_file_name = context_file_name.replace("shcntx", "shlang");
            let parent_dir = context_path.parent().unwrap_or(Path::new("."));
            let language_path = parent_dir.join(language_file_name);

            if language_path.exists() {
                tracing::info!(
                    "Auto-detected language archive: {}",
                    language_path.display()
                );
                return Some(HbkArchiveParser::new(language_path));
            } else {
                tracing::debug!("Language archive not found at: {}", language_path.display());
            }
        }

        None
    }

    /// Устанавливает архив языковых элементов (shlang)
    pub fn set_language_archive<P: AsRef<Path>>(&mut self, language_archive_path: P) {
        self.language_parser = Some(HbkArchiveParser::new(language_archive_path));
    }

    /// Извлекает полную базу знаний синтаксиса BSL (замена Python extraction logic)
    pub fn extract_syntax_database(
        &mut self,
        max_files: Option<usize>,
    ) -> Result<BslSyntaxDatabase> {
        tracing::info!("Extracting BSL syntax database from context and language archives");

        let mut database = BslSyntaxDatabase {
            objects: HashMap::new(),
            methods: HashMap::new(),
            properties: HashMap::new(),
            functions: HashMap::new(),
            operators: HashMap::new(),
            keywords: Vec::new(),
        };

        // 1. Обработка архива языковой документации (shlang) для примитивных типов и директив
        if let Some(ref mut language_parser) = self.language_parser {
            tracing::info!("Processing language archive for primitive types and directives");
            language_parser.open_archive()?;

            // Собираем содержимое файлов без вызова методов self
            let mut primitive_contents = Vec::new();
            let primitive_types = vec![
                "def_String",
                "def_Number",
                "def_Date",
                "def_Boolean",
                "def_Undefined",
            ];
            for primitive_type in &primitive_types {
                // В shlang архиве файлы БЕЗ расширения .html
                if let Some(html_content) = language_parser.extract_file_content(primitive_type) {
                    tracing::debug!(
                        "✅ Found primitive type {}, content length: {}",
                        primitive_type,
                        html_content.len()
                    );
                    primitive_contents.push((
                        primitive_type.to_string(),
                        primitive_type.to_string(),
                        html_content,
                    ));
                } else {
                    tracing::debug!(
                        "❌ Primitive type {} NOT FOUND in language archive",
                        primitive_type
                    );
                }
            }

            // Извлекаем директивы компиляции (тоже без .html)
            let pragma_content = language_parser.extract_file_content("Pragma");

            // Теперь обрабатываем собранное содержимое без borrowing conflict
            tracing::debug!(
                "Processing {} primitive types found in language archive",
                primitive_contents.len()
            );
            for (primitive_type, filename, html_content) in primitive_contents {
                tracing::debug!(
                    "Processing primitive type: {} (content length: {})",
                    primitive_type,
                    html_content.len()
                );
                match self.extract_syntax_info(&html_content, &filename) {
                    Ok(syntax_info) => {
                        // Преобразуем SyntaxInfo в BslObjectInfo для примитивного типа
                        let type_name = primitive_type.replace("def_", "");
                        tracing::debug!(
                            "✅ Successfully parsed primitive type: {} -> {}",
                            primitive_type,
                            type_name
                        );
                        let object_info = BslObjectInfo {
                            name: type_name.clone(),
                            object_type: "PrimitiveType".to_string(),
                            description: Some(syntax_info.description.clone()),
                            methods: Vec::new(), // Примитивные типы пока без методов
                            properties: Vec::new(),
                            constructors: Vec::new(),
                            availability: if syntax_info.availability.is_empty() {
                                None
                            } else {
                                Some(syntax_info.availability.join(", "))
                            },
                        };
                        database.objects.insert(type_name.clone(), object_info);
                        tracing::debug!("Added primitive type: {}", type_name);
                    }
                    Err(e) => {
                        tracing::debug!(
                            "Failed to extract primitive type {}: {}",
                            primitive_type,
                            e
                        );
                    }
                }
            }

            // Обрабатываем директивы
            if let Some(html_content) = pragma_content {
                match self.extract_pragma_directives(&html_content) {
                    Ok(directives) => {
                        for directive in directives {
                            database.keywords.push(directive);
                        }
                        tracing::debug!("Added compilation directives");
                    }
                    Err(e) => {
                        tracing::debug!("Failed to extract directives from Pragma.html: {}", e);
                    }
                }
            }
        }

        // 2. Обработка архива контекстной документации (shcntx) для объектов, методов, свойств
        tracing::info!("Processing context archive for objects, methods, and properties");
        self.context_parser.open_archive()?;

        // Получаем список HTML файлов из контекстного архива
        let html_files: Vec<String> = self
            .context_parser
            .list_contents()
            .into_iter()
            .filter(|f| f.ends_with(".html") || f.ends_with(".htm"))
            .collect();

        let files_to_process = if let Some(max) = max_files {
            html_files.into_iter().take(max).collect()
        } else {
            html_files
        };

        tracing::debug!(
            "Processing {} HTML files from context archive",
            files_to_process.len()
        );

        // Обрабатываем каждый HTML файл из контекстного архива
        for (i, filename) in files_to_process.iter().enumerate() {
            if i > 0 && i % 1000 == 0 {
                tracing::debug!("Processed {} files...", i);
            }

            // Извлекаем содержимое файла
            if let Some(html_content) = self.context_parser.extract_file_content(filename) {
                // Парсим HTML и извлекаем синтаксическую информацию
                match self.extract_syntax_info(&html_content, filename) {
                    Ok(syntax_info) => {
                        self.categorize_syntax(syntax_info, &mut database);
                    }
                    Err(e) => {
                        tracing::debug!("Failed to extract syntax from {}: {}", filename, e);
                    }
                }
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

    /// Извлекает информацию о синтаксисе из HTML контента (полный порт Python extract_syntax_info)
    pub fn extract_syntax_info(&self, html_content: &str, filename: &str) -> Result<SyntaxInfo> {
        if html_content.is_empty() {
            anyhow::bail!("Empty HTML content");
        }

        let document = Html::parse_document(html_content);
        let mut syntax_info = SyntaxInfo {
            filename: filename.to_string(),
            title: String::new(),
            syntax: String::new(),
            syntax_variants: Vec::new(),
            description: String::new(),
            parameters: Vec::new(),
            parameters_by_variant: HashMap::new(),
            return_value: String::new(),
            example: String::new(),
            category: String::new(),
            links: Vec::new(),
            availability: Vec::new(),
            version: String::new(),
            methods: Vec::new(),
            collection_elements: None,
            object_context: None,
        };

        // Извлекаем заголовок
        let title_selector = Selector::parse("h1.V8SH_pagetitle").unwrap();
        if let Some(title_elem) = document.select(&title_selector).next() {
            syntax_info.title = title_elem.text().collect::<String>().trim().to_string();
        }

        // Извлекаем контекст объекта для методов (p.V8SH_title)
        let object_title_selector = Selector::parse("p.V8SH_title").unwrap();
        if let Some(object_elem) = document.select(&object_title_selector).next() {
            let object_name = object_elem.text().collect::<String>().trim().to_string();
            // Извлекаем только русское название до скобки
            if let Some(pos) = object_name.find(" (") {
                syntax_info.object_context = Some(object_name[..pos].to_string());
            } else {
                syntax_info.object_context = Some(object_name);
            }
        }

        // Определяем категорию по пути файла
        if filename.contains("/methods/") {
            syntax_info.category = "method".to_string();
        } else if filename.contains("/properties/") {
            syntax_info.category = "property".to_string();
        } else if filename.contains("objects/")
            && !filename.contains("/methods/")
            && !filename.contains("/properties/")
        {
            syntax_info.category = "object".to_string();
        } else if filename.contains("tables/") {
            syntax_info.category = "table".to_string();
        }

        // Извлекаем синтаксис и другие элементы
        self.extract_syntax_variants(&document, &mut syntax_info)?;
        self.extract_description(&document, &mut syntax_info)?;
        self.extract_availability(&document, &mut syntax_info)?;
        self.extract_parameters(&document, &mut syntax_info)?;
        self.extract_return_value(&document, &mut syntax_info)?;
        self.extract_version(&document, &mut syntax_info)?;
        self.extract_example(&document, &mut syntax_info)?;
        self.extract_object_methods(&document, &mut syntax_info)?;
        self.extract_object_properties(&document, &mut syntax_info)?;
        self.extract_collection_elements(&document, &mut syntax_info)?;
        self.extract_links(&document, &mut syntax_info)?;

        Ok(syntax_info)
    }

    /// Извлекает тип из описания свойства (например, "Тип: МенеджерПользователейИнформационнойБазы")
    fn extract_type_from_description(&self, html_content: &str) -> Option<String> {
        tracing::debug!(
            "extract_type_from_description: parsing HTML content, length: {}",
            html_content.len()
        );
        let document = Html::parse_document(html_content);

        // Ищем секцию "Описание:"
        let chapter_selector = Selector::parse("p.V8SH_chapter").unwrap();
        tracing::debug!("extract_type_from_description: looking for 'Описание' section");
        for elem in document.select(&chapter_selector) {
            let text = elem.text().collect::<String>().trim().to_string();
            tracing::debug!("extract_type_from_description: found chapter: '{}'", text);
            if text.contains("Описание") {
                tracing::debug!("extract_type_from_description: found 'Описание' section");

                // ИСПРАВЛЕНИЕ: Ищем все элементы и текстовые узлы после "Описание:"
                for sibling in elem.next_siblings() {
                    match sibling.value() {
                        scraper::node::Node::Element(element) => {
                            let elem_ref = ElementRef::wrap(sibling).unwrap();
                            let tag_name = element.name();
                            tracing::debug!(
                                "extract_type_from_description: examining element: {}",
                                tag_name
                            );

                            // Получаем весь текст из этого элемента включая ссылки
                            let full_text = elem_ref.text().collect::<String>();
                            tracing::debug!(
                                "extract_type_from_description: element text: '{}'",
                                full_text.trim()
                            );

                            // Ищем паттерн "Тип:" в тексте
                            if full_text.contains("Тип:") {
                                tracing::debug!(
                                    "extract_type_from_description: found 'Тип:' in element text"
                                );

                                // Ищем ссылки внутри этого элемента
                                let link_selector = Selector::parse("a").unwrap();
                                for link in elem_ref.select(&link_selector) {
                                    let type_name =
                                        link.text().collect::<String>().trim().to_string();
                                    tracing::debug!(
                                        "extract_type_from_description: found link text: '{}'",
                                        type_name
                                    );

                                    if !type_name.is_empty() {
                                        tracing::debug!(
                                            "extract_type_from_description: returning type: '{}'",
                                            type_name
                                        );
                                        return Some(type_name);
                                    }
                                }
                            }

                            // Если это следующий V8SH_chapter - прекращаем поиск
                            if element.attr("class").unwrap_or("").contains("V8SH_chapter") {
                                break;
                            }
                        }
                        scraper::node::Node::Text(text_node) => {
                            let text_content = text_node.text.trim();
                            tracing::debug!(
                                "extract_type_from_description: examining text node: '{}'",
                                text_content
                            );

                            // В 1С документации "Тип:" может быть в текстовом узле, но ссылка всё равно в элементе
                            if text_content.contains("Тип:") {
                                tracing::debug!(
                                    "extract_type_from_description: found 'Тип:' in text node"
                                );
                                // Продолжаем искать в следующих узлах - ссылка должна быть в элементе
                            }
                        }
                        _ => {}
                    }
                }
                break;
            }
        }

        // ДОПОЛНИТЕЛЬНЫЙ ПОИСК: Ищем ссылки во всем тексте после "Описание:"
        // Используем более простой подход - парсим все содержимое после "Описание:"
        let raw_html = html_content;
        if let Some(description_pos) = raw_html.find("Описание:</p>") {
            let after_description = &raw_html[description_pos..];
            tracing::debug!(
                "extract_type_from_description: content after 'Описание:': '{}'",
                &after_description[..std::cmp::min(200, after_description.len())]
            );

            // Ищем первую ссылку после "Описание:"
            if let Some(link_start) = after_description.find("<a href=") {
                if let Some(link_content_start) = after_description[link_start..].find('>') {
                    if let Some(link_content_end) =
                        after_description[link_start + link_content_start + 1..].find("</a>")
                    {
                        let link_text = &after_description[link_start + link_content_start + 1
                            ..link_start + link_content_start + 1 + link_content_end];
                        let type_name = link_text.trim().to_string();
                        tracing::debug!(
                            "extract_type_from_description: extracted type from raw HTML: '{}'",
                            type_name
                        );
                        if !type_name.is_empty() {
                            return Some(type_name);
                        }
                    }
                }
            }
        }

        None
    }

    /// Классифицирует синтаксическую информацию и добавляет в базу данных
    fn categorize_syntax(&mut self, syntax_info: SyntaxInfo, database: &mut BslSyntaxDatabase) {
        let title = syntax_info.title.trim();
        if title.is_empty() {
            return;
        }

        // Определяем тип по заголовку, категории и синтаксису
        if title.contains("Функция") || title.to_lowercase().contains("function") {
            if let Ok(function_info) = self.convert_to_function_info(syntax_info) {
                database
                    .functions
                    .insert(function_info.name.clone(), function_info);
            }
        } else if title.contains("Метод")
            || title.to_lowercase().contains("method")
            || syntax_info.category == "method"
        {
            if let Ok(method_info) = self.convert_to_method_info(syntax_info) {
                database
                    .methods
                    .insert(method_info.name.clone(), method_info);
            }
        } else if title.contains("Свойство")
            || title.to_lowercase().contains("property")
            || syntax_info.category == "property"
        {
            if let Ok(property_info) = self.convert_to_property_info(syntax_info.clone()) {
                database
                    .properties
                    .insert(property_info.name.clone(), property_info);
            }

            // ИСПРАВЛЕНИЕ: Для Global context свойств и методов извлекаем типы и создаем объекты
            tracing::debug!("DEBUG: Checking filename: {}", syntax_info.filename);
            if syntax_info.filename.contains("Global context/properties/") {
                tracing::info!("🔍 Processing Global context property: {}", title);

                // Читаем HTML снова для извлечения типа (у нас есть доступ к parser)
                tracing::debug!(
                    "Trying to extract file content for: '{}'",
                    syntax_info.filename
                );
                if let Some(html_content) = self
                    .context_parser
                    .extract_file_content(&syntax_info.filename)
                {
                    tracing::debug!(
                        "Successfully read HTML content for {}, length: {}",
                        syntax_info.filename,
                        html_content.len()
                    );
                    if let Some(type_name) = self.extract_type_from_description(&html_content) {
                        tracing::info!(
                            "✅ Extracted type from Global context property {}: {}",
                            title,
                            type_name
                        );

                        // Создаем объект для типа менеджера
                        let manager_object = BslObjectInfo {
                            name: type_name.clone(),
                            object_type: "Manager".to_string(),
                            description: Some(format!("Менеджер для работы с {}", title)),
                            methods: Vec::new(), // Методы будут добавлены отдельно из других файлов
                            properties: Vec::new(),
                            constructors: Vec::new(),
                            availability: Some(
                                "Сервер, толстый клиент, внешнее соединение".to_string(),
                            ),
                        };
                        database.objects.insert(type_name.clone(), manager_object);
                        tracing::debug!("Created manager object: {}", type_name);

                        // Создаем также основной тип (например, ПользовательИнформационнойБазы)
                        let base_type_name = type_name.replace("Менеджер", "");
                        if !base_type_name.is_empty() && base_type_name != type_name {
                            let base_object = BslObjectInfo {
                                name: base_type_name.clone(),
                                object_type: "InfoBaseEntity".to_string(),
                                description: Some(format!(
                                    "Объект информационной базы: {}",
                                    base_type_name
                                )),
                                methods: Vec::new(),
                                properties: Vec::new(),
                                constructors: Vec::new(),
                                availability: Some(
                                    "Сервер, толстый клиент, внешнее соединение".to_string(),
                                ),
                            };
                            database.objects.insert(base_type_name.clone(), base_object);
                            tracing::debug!("Created base object: {}", base_type_name);
                        }

                        // Создаем глобальное свойство с наследованием методов от менеджерного типа
                        let global_property_name =
                            title.split('(').next().unwrap_or(title).trim().to_string();

                        // Получаем методы от менеджерного типа
                        let manager_methods =
                            if let Some(manager_obj) = database.objects.get(&type_name) {
                                manager_obj.methods.clone()
                            } else {
                                Vec::new()
                            };

                        let global_object = BslObjectInfo {
                            name: global_property_name.clone(),
                            object_type: "GlobalProperty".to_string(),
                            description: Some(format!("Глобальное свойство типа {}", type_name)),
                            methods: manager_methods, // ИСПРАВЛЕНИЕ: Наследуем методы от менеджера
                            properties: Vec::new(),
                            constructors: Vec::new(),
                            availability: Some(
                                "Сервер, толстый клиент, внешнее соединение".to_string(),
                            ),
                        };
                        let methods_count = global_object.methods.len();
                        database
                            .objects
                            .insert(global_property_name.clone(), global_object);
                        tracing::debug!(
                            "🔗 Global property {} inherits {} methods from {}",
                            global_property_name,
                            methods_count,
                            type_name
                        );
                    } else {
                        tracing::warn!(
                            "⚠️  Could not extract type from Global context property {}",
                            title
                        );
                    }
                } else {
                    tracing::warn!(
                        "⚠️  Could not read HTML content for file: '{}'",
                        syntax_info.filename
                    );
                }
            }

            // ДОБАВЛЕНО: Обработка Global context методов (глобальных функций)
            if syntax_info.filename.contains("Global context/methods/") {
                tracing::info!(
                    "🔍 Processing Global context method (global function): {}",
                    title
                );

                // Создаем глобальную функцию
                if let Ok(function_info) = self.convert_to_function_info(syntax_info.clone()) {
                    // Добавляем как функцию в базу данных
                    database
                        .functions
                        .insert(function_info.name.clone(), function_info.clone());
                    tracing::info!("✅ Added global function: {}", function_info.name);

                    // ВАЖНО: Также создаем объект "Global" для группировки всех глобальных функций
                    let global_object_name = "Global".to_string();
                    let global_method = BslMethodInfo {
                        name: function_info.name.clone(),
                        english_name: None, // BslFunctionInfo не имеет english_name
                        syntax_variants: function_info
                            .syntax_variants
                            .iter()
                            .map(|s| SyntaxVariant {
                                variant_name: "default".to_string(),
                                syntax: s.clone(),
                            })
                            .collect(),
                        parameters: function_info.parameters.clone(),
                        parameters_by_variant: HashMap::new(),
                        return_type: function_info.return_type.clone(),
                        return_type_description: None, // BslFunctionInfo не имеет return_type_description
                        description: function_info.description.clone(),
                        availability: function_info
                            .availability
                            .map(|av| vec![av])
                            .unwrap_or_default(),
                        version: None,        // BslFunctionInfo не имеет version
                        examples: Vec::new(), // BslFunctionInfo не имеет examples
                        object_context: Some(global_object_name.clone()),
                        links: Vec::new(), // BslFunctionInfo не имеет links
                    };

                    // Добавляем в коллекцию методов
                    database
                        .methods
                        .insert(function_info.name.clone(), global_method);

                    // Создаем или обновляем объект "Global" с методами
                    if let Some(global_obj) = database.objects.get_mut(&global_object_name) {
                        // Добавляем имя метода к существующему объекту Global
                        global_obj.methods.push(function_info.name.clone());
                    } else {
                        // Создаем новый объект Global
                        let global_object = BslObjectInfo {
                            name: global_object_name.clone(),
                            object_type: "GlobalContext".to_string(),
                            description: Some("Глобальный контекст - коллекция всех глобальных функций и свойств 1С".to_string()),
                            methods: vec![function_info.name.clone()],
                            properties: Vec::new(),
                            constructors: Vec::new(),
                            availability: Some("Сервер, толстый клиент, веб-клиент, мобильный клиент, внешнее соединение".to_string()),
                        };
                        database
                            .objects
                            .insert(global_object_name.clone(), global_object);
                        tracing::debug!("Created Global context object");
                    }
                } else {
                    tracing::warn!(
                        "⚠️  Could not convert Global context method to function: {}",
                        title
                    );
                }
            }
        } else if title.contains("Оператор") || title.to_lowercase().contains("operator") {
            if let Ok(operator_info) = self.convert_to_operator_info(syntax_info) {
                database
                    .operators
                    .insert(operator_info.operator.clone(), operator_info);
            }
        } else if syntax_info.category == "object" {
            if let Ok(object_info) = self.convert_to_object_info(syntax_info) {
                database
                    .objects
                    .insert(object_info.name.clone(), object_info);
            }
        } else {
            // По умолчанию добавляем в объекты
            if let Ok(object_info) = self.convert_to_object_info(syntax_info) {
                database
                    .objects
                    .insert(object_info.name.clone(), object_info);
            }
        }
    }

    /// Извлекает параметры с поддержкой вариантов
    fn extract_parameters(&self, document: &Html, syntax_info: &mut SyntaxInfo) -> Result<()> {
        let chapter_selector = Selector::parse("p.V8SH_chapter").unwrap();
        let _rubric_selector = Selector::parse("div.V8SH_rubric").unwrap();
        let mut current_variant: Option<String> = None;

        for elem in document.select(&chapter_selector) {
            let text = elem.text().collect::<String>().trim().to_string();

            // Определяем текущий вариант
            if text.contains("Вариант синтаксиса:") {
                current_variant = Some(text.replace("Вариант синтаксиса:", "").trim().to_string());
                if let Some(variant) = &current_variant {
                    syntax_info
                        .parameters_by_variant
                        .insert(variant.clone(), Vec::new());
                }
            }

            // Извлекаем параметры
            if text.contains("Параметры:") {
                // Ищем все div с классом V8SH_rubric до следующего заголовка
                let mut current = elem.next_sibling();
                while let Some(node) = current {
                    if let Some(elem_ref) = ElementRef::wrap(node) {
                        if elem_ref.value().name() == "p"
                            && elem_ref
                                .value()
                                .attr("class")
                                .unwrap_or("")
                                .contains("V8SH_chapter")
                        {
                            break;
                        }

                        if elem_ref.value().name() == "div"
                            && elem_ref
                                .value()
                                .attr("class")
                                .unwrap_or("")
                                .contains("V8SH_rubric")
                        {
                            if let Ok(param_info) = self.extract_parameter_info(&elem_ref) {
                                // Добавляем в общий список
                                syntax_info.parameters.push(param_info.clone());

                                // Добавляем к текущему варианту, если есть
                                if let Some(variant) = &current_variant {
                                    if let Some(variant_params) =
                                        syntax_info.parameters_by_variant.get_mut(variant)
                                    {
                                        variant_params.push(param_info);
                                    }
                                }
                            }
                        }
                    }
                    current = node.next_sibling();
                }
            }
        }

        Ok(())
    }

    /// Извлекает информацию об одном параметре
    fn extract_parameter_info(&self, param_block: &ElementRef) -> Result<ParameterInfo> {
        let mut param_info = ParameterInfo {
            name: String::new(),
            param_type: None,
            type_description: None,
            description: None,
            is_optional: false,
            default_value: None,
            link: None,
        };

        let param_text = param_block.text().collect::<String>();

        // Извлекаем имя параметра между < >
        if let Some(start) = param_text.find('<') {
            if let Some(end) = param_text.find('>') {
                if end > start {
                    param_info.name = param_text[start + 1..end].trim().to_string();
                }
            }
        }

        // Проверяем обязательность
        param_info.is_optional = param_text.contains("(необязательный)");

        // Ищем ссылку на тип
        let link_selector = Selector::parse("a").unwrap();
        if let Some(type_link) = param_block.select(&link_selector).next() {
            if let Some(href) = type_link.value().attr("href") {
                param_info.link = Some(href.to_string());

                // Извлекаем тип и описание из ссылки
                let (type_name, type_desc) = self.extract_type_from_link(href);
                if !type_name.is_empty() {
                    param_info.param_type = Some(type_name);
                    param_info.type_description = Some(type_desc);
                }
            }
        }

        // Ищем описание параметра в следующем элементе
        if let Some(next_sibling) = param_block.next_sibling() {
            if let Some(next_elem) = ElementRef::wrap(next_sibling) {
                let type_text = next_elem.text().collect::<String>();

                // Извлекаем тип после "Тип:"
                if type_text.contains("Тип:") {
                    if let Some(type_start) = type_text.find("Тип:") {
                        let type_end = type_text.find('.').unwrap_or(type_text.len());
                        if type_end > type_start + 4 {
                            let param_type = type_text[type_start + 4..type_end].trim().to_string();
                            if param_info.param_type.is_none() {
                                param_info.param_type = Some(param_type);
                            }
                        }
                    }
                }
            }
        }

        Ok(param_info)
    }

    /// Извлекает возвращаемое значение из HTML документации 1С
    fn extract_return_value(&self, document: &Html, syntax_info: &mut SyntaxInfo) -> Result<()> {
        let chapter_selector = Selector::parse("p.V8SH_chapter").unwrap();

        for elem in document.select(&chapter_selector) {
            let text = elem.text().collect::<String>().trim().to_string();
            if text.contains("Возвращаемое значение") {
                // Извлекаем секцию возвращаемого значения из HTML
                if let Some(return_section) = self.extract_return_value_section_html(elem) {
                    syntax_info.return_value = self.parse_return_type_from_html(&return_section);
                }
                break;
            }
        }

        Ok(())
    }

    /// Извлекает HTML секцию возвращаемого значения
    fn extract_return_value_section_html(&self, chapter_elem: ElementRef) -> Option<String> {
        let mut html_content = String::new();
        let mut current = chapter_elem.next_sibling();

        // Собираем HTML до следующего заголовка V8SH_chapter
        while let Some(node) = current {
            if let Some(elem_ref) = ElementRef::wrap(node) {
                // Прерываемся на следующем заголовке
                if elem_ref.value().name() == "p"
                    && elem_ref
                        .value()
                        .attr("class")
                        .unwrap_or("")
                        .contains("V8SH_chapter")
                {
                    break;
                }
                html_content.push_str(&elem_ref.html());
            } else {
                // Текстовые узлы тоже добавляем
                html_content.push_str(node.value().as_text()?.trim());
            }
            current = node.next_sibling();
        }

        if html_content.is_empty() {
            None
        } else {
            Some(html_content)
        }
    }

    /// Парсит тип возврата из HTML секции на основе реальной структуры документации 1С
    fn parse_return_type_from_html(&self, html_section: &str) -> String {
        // Паттерн 1: Тип: <a href="...">ИмяТипа</a>. <br>
        if let Some(type_match) = Regex::new(r#"Тип:\s*<a href="[^"]*">([^<]+)</a>\.\s*<br>"#)
            .ok()
            .and_then(|re| re.captures(html_section))
        {
            return type_match[1].trim().to_string();
        }

        // Паттерн 2: Type: <a href="...">TypeName</a>. <br>
        if let Some(type_match) = Regex::new(r#"Type:\s*<a href="[^"]*">([^<]+)</a>\.\s*<br>"#)
            .ok()
            .and_then(|re| re.captures(html_section))
        {
            return type_match[1].trim().to_string();
        }

        // Паттерн 3: Тип: <a href="...">ИмяТипа</a>
        if let Some(type_match) = Regex::new(r#"Тип:\s*<a href="[^"]*">([^<]+)</a>"#)
            .ok()
            .and_then(|re| re.captures(html_section))
        {
            return type_match[1].trim().to_string();
        }

        // Паттерн 4: Простой текст "Тип: ИмяТипа"
        if let Some(type_match) =
            Regex::new(r"Тип:\s*([А-ЯA-Z][а-яА-Яa-zA-Z0-9]*(?:\.[А-ЯA-Z][а-яА-Яa-zA-Z0-9]*)*)")
                .ok()
                .and_then(|re| re.captures(html_section))
        {
            return type_match[1].trim().to_string();
        }

        // Паттерн 5: Извлекаем любой тип из ссылки <a href="...">ТипВозврата</a>
        if let Some(type_match) = Regex::new(
            r#"<a href="[^"]*">([А-ЯA-Z][а-яА-Яa-zA-Z0-9]*(?:\.[А-ЯA-Z][а-яА-Яa-zA-Z0-9]*)*)</a>"#,
        )
        .ok()
        .and_then(|re| re.captures(html_section))
        {
            let potential_type = type_match[1].trim();

            // Исключаем служебные слова
            if !matches!(
                potential_type,
                "Описание" | "Description" | "Примечание" | "Note"
            ) {
                return potential_type.to_string();
            }
        }

        // Если ничего не нашли, возвращаем пустую строку (метод ничего не возвращает)
        String::new()
    }

    /// Извлекает версию
    fn extract_version(&self, document: &Html, syntax_info: &mut SyntaxInfo) -> Result<()> {
        let chapter_selector = Selector::parse("p.V8SH_chapter").unwrap();

        for elem in document.select(&chapter_selector) {
            let text = elem.text().collect::<String>().trim().to_string();
            if text.contains("Использование в версии") {
                let _version_selector = Selector::parse("p.V8SH_versionInfo").unwrap();
                if let Some(version_elem) =
                    elem.next_siblings().filter_map(ElementRef::wrap).find(|e| {
                        e.value()
                            .attr("class")
                            .unwrap_or("")
                            .contains("V8SH_versionInfo")
                    })
                {
                    let version_text = version_elem.text().collect::<String>().trim().to_string();
                    // Извлекаем номер версии
                    if let Some(version_pos) = version_text.find("версии") {
                        syntax_info.version = version_text[version_pos + 6..].trim().to_string();
                    }
                }
                break;
            }
        }

        Ok(())
    }

    /// Извлекает пример
    fn extract_example(&self, document: &Html, syntax_info: &mut SyntaxInfo) -> Result<()> {
        let chapter_selector = Selector::parse("p.V8SH_chapter").unwrap();

        for elem in document.select(&chapter_selector) {
            let text = elem.text().collect::<String>().trim().to_string();
            if text.contains("Пример") {
                let _table_selector = Selector::parse("table").unwrap();
                if let Some(table) = elem
                    .next_siblings()
                    .filter_map(ElementRef::wrap)
                    .find(|e| e.value().name() == "table")
                {
                    syntax_info.example = table.text().collect::<String>().trim().to_string();
                }
                break;
            }
        }

        Ok(())
    }

    /// Извлекает методы объекта
    fn extract_object_methods(&self, document: &Html, syntax_info: &mut SyntaxInfo) -> Result<()> {
        let chapter_selector = Selector::parse("p.V8SH_chapter").unwrap();

        for elem in document.select(&chapter_selector) {
            let text = elem.text().collect::<String>().trim().to_string();
            if text.contains("Методы") {
                // Ищем список методов
                let mut current = elem.next_sibling();
                while let Some(node) = current {
                    if let Some(elem_ref) = ElementRef::wrap(node) {
                        if elem_ref.value().name() == "ul" {
                            // Нашли список методов
                            let li_selector = Selector::parse("li").unwrap();
                            for li in elem_ref.select(&li_selector) {
                                let method_text = li.text().collect::<String>().trim().to_string();
                                if !method_text.is_empty() {
                                    let method_info = self.parse_method_from_text(&method_text);
                                    syntax_info.methods.push(method_info);
                                }
                            }
                            break;
                        } else if elem_ref.value().name() == "p"
                            && elem_ref
                                .value()
                                .attr("class")
                                .unwrap_or("")
                                .contains("V8SH_chapter")
                        {
                            break;
                        }
                    }
                    current = node.next_sibling();
                }

                // Если методы не найдены в списке, ищем ссылки
                if syntax_info.methods.is_empty() {
                    self.extract_method_links(document, syntax_info);
                }
                break;
            }
        }

        Ok(())
    }

    /// Извлекает методы из ссылок
    fn extract_method_links(&self, document: &Html, syntax_info: &mut SyntaxInfo) {
        let link_selector = Selector::parse("a").unwrap();
        let mut seen_methods = std::collections::HashSet::new();

        for link in document.select(&link_selector) {
            if let Some(href) = link.value().attr("href") {
                if href.contains("methods/") {
                    let text = link.text().collect::<String>().trim().to_string();
                    if !text.is_empty() {
                        let method_info = self.parse_method_from_text(&text);
                        let method_key =
                            format!("{}_{}", method_info.name, method_info.english_name);

                        if !seen_methods.contains(&method_key) {
                            syntax_info.methods.push(method_info);
                            seen_methods.insert(method_key);
                        }
                    }
                }
            }
        }
    }

    /// Извлекает свойства объекта
    fn extract_object_properties(
        &self,
        document: &Html,
        syntax_info: &mut SyntaxInfo,
    ) -> Result<()> {
        let chapter_selector = Selector::parse("p.V8SH_chapter").unwrap();

        for elem in document.select(&chapter_selector) {
            let text = elem.text().collect::<String>().trim().to_string();
            if text.contains("Свойства:") {
                // Ищем ссылки на свойства после заголовка
                let mut current = elem.next_sibling();
                while let Some(node) = current {
                    if let Some(elem_ref) = ElementRef::wrap(node) {
                        // Прерываемся, если встретили следующий заголовок
                        if elem_ref.value().name() == "p"
                            && elem_ref
                                .value()
                                .attr("class")
                                .unwrap_or("")
                                .contains("V8SH_chapter")
                        {
                            break;
                        }

                        // Ищем ссылки на свойства
                        if elem_ref.value().name() == "a" {
                            if let Some(href) = elem_ref.value().attr("href") {
                                if href.contains("properties/") {
                                    let property_text =
                                        elem_ref.text().collect::<String>().trim().to_string();
                                    if !property_text.is_empty() {
                                        // Парсим имя свойства и английское название
                                        let (rus_name, eng_name) =
                                            if let Some(pos) = property_text.find(" (") {
                                                let rus = property_text[..pos].to_string();
                                                let eng = property_text
                                                    [pos + 2..property_text.len() - 1]
                                                    .to_string();
                                                (rus, Some(eng))
                                            } else {
                                                (property_text, None)
                                            };

                                        // Добавляем в параметры как временное решение
                                        // TODO: добавить отдельную структуру для свойств в SyntaxInfo
                                        let param_info = ParameterInfo {
                                            name: rus_name,
                                            param_type: eng_name,
                                            type_description: Some("property".to_string()),
                                            description: None,
                                            is_optional: false,
                                            default_value: None,
                                            link: None,
                                        };
                                        syntax_info.parameters.push(param_info);
                                    }
                                }
                            }
                        }
                    }
                    current = node.next_sibling();
                }
                break;
            }
        }

        Ok(())
    }

    /// Парсит информацию о методе из текста
    fn parse_method_from_text(&self, text: &str) -> MethodInfo {
        let mut method_info = MethodInfo {
            name: text.to_string(),
            english_name: String::new(),
            full_name: text.to_string(),
        };

        // Пытаемся найти английское название в скобках
        if let Some(start) = text.find('(') {
            if let Some(end) = text.find(')') {
                if end > start {
                    method_info.name = text[..start].trim().to_string();
                    method_info.english_name = text[start + 1..end].trim().to_string();
                }
            }
        }

        method_info
    }

    /// Извлекает информацию об элементах коллекции
    fn extract_collection_elements(
        &self,
        document: &Html,
        syntax_info: &mut SyntaxInfo,
    ) -> Result<()> {
        let chapter_selector = Selector::parse("p.V8SH_chapter").unwrap();

        for elem in document.select(&chapter_selector) {
            let text = elem.text().collect::<String>().trim().to_string();
            if text.contains("Элементы коллекции") {
                let mut elements_info = CollectionElementsInfo {
                    description: None,
                    usage: None,
                    element_type: None,
                };

                // Собираем весь текст до следующего заголовка
                let mut full_text = String::new();
                let mut current = elem.next_sibling();

                while let Some(node) = current {
                    if let Some(elem_ref) = ElementRef::wrap(node) {
                        if elem_ref.value().name() == "p"
                            && elem_ref
                                .value()
                                .attr("class")
                                .unwrap_or("")
                                .contains("V8SH_chapter")
                        {
                            break;
                        }
                        full_text.push_str(&elem_ref.text().collect::<String>());
                        full_text.push(' ');
                    }
                    current = node.next_sibling();
                }

                // Разбиваем на предложения
                let sentences: Vec<String> = full_text
                    .split('.')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                if !sentences.is_empty() {
                    // Первое предложение - тип элементов
                    elements_info.element_type = Some(sentences[0].clone());

                    // Ищем информацию об использовании
                    let usage_sentences: Vec<String> = sentences
                        .iter()
                        .filter(|s| {
                            s.contains("Для каждого")
                                || s.contains("Из")
                                || s.contains("Цикл")
                                || s.contains("индекс")
                                || s.contains("оператор")
                        })
                        .cloned()
                        .collect();

                    if !usage_sentences.is_empty() {
                        elements_info.usage = Some(usage_sentences.join(". "));
                    }

                    // Формируем полное описание
                    elements_info.description = Some(sentences.join(". "));
                }

                syntax_info.collection_elements = Some(elements_info);
                break;
            }
        }

        Ok(())
    }

    /// Извлекает ссылки
    fn extract_links(&self, document: &Html, syntax_info: &mut SyntaxInfo) -> Result<()> {
        let link_selector = Selector::parse("a").unwrap();

        for link in document.select(&link_selector) {
            if let Some(href) = link.value().attr("href") {
                if href.starts_with("v8help://") {
                    syntax_info.links.push(LinkInfo {
                        text: link.text().collect::<String>().trim().to_string(),
                        href: href.to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    /// Преобразует SyntaxInfo в BslMethodInfo
    fn convert_to_method_info(&self, syntax_info: SyntaxInfo) -> Result<BslMethodInfo> {
        let method_name = self.extract_method_name(&syntax_info.title);

        let mut method_info = BslMethodInfo {
            name: method_name,
            english_name: None,
            syntax_variants: syntax_info.syntax_variants,
            parameters: syntax_info.parameters,
            parameters_by_variant: syntax_info.parameters_by_variant,
            return_type: if syntax_info.return_value.is_empty() {
                None
            } else {
                Some(syntax_info.return_value)
            },
            return_type_description: None,
            description: if syntax_info.description.is_empty() {
                None
            } else {
                Some(syntax_info.description)
            },
            availability: syntax_info.availability,
            version: if syntax_info.version.is_empty() {
                None
            } else {
                Some(syntax_info.version)
            },
            examples: if syntax_info.example.is_empty() {
                vec![]
            } else {
                vec![syntax_info.example]
            },
            object_context: None,
            links: syntax_info.links,
        };

        // Используем контекст объекта из HTML или пытаемся извлечь из имени
        method_info.object_context = syntax_info
            .object_context
            .or_else(|| self.extract_object_context(&method_info.name));

        Ok(method_info)
    }

    /// Преобразует SyntaxInfo в BslObjectInfo
    fn convert_to_object_info(&self, syntax_info: SyntaxInfo) -> Result<BslObjectInfo> {
        // Извлекаем свойства из parameters, где type_description == "property"
        let properties: Vec<String> = syntax_info
            .parameters
            .iter()
            .filter(|p| {
                p.type_description
                    .as_ref()
                    .map(|d| d == "property")
                    .unwrap_or(false)
            })
            .map(|p| p.name.clone())
            .collect();

        let object_info = BslObjectInfo {
            name: syntax_info.title.clone(),
            object_type: syntax_info.category,
            description: if syntax_info.description.is_empty() {
                None
            } else {
                Some(syntax_info.description)
            },
            methods: syntax_info.methods.iter().map(|m| m.name.clone()).collect(),
            properties,
            constructors: Vec::new(), // TODO: извлечь из описания
            availability: if syntax_info.availability.is_empty() {
                None
            } else {
                Some(syntax_info.availability.join(", "))
            },
        };

        Ok(object_info)
    }

    /// Преобразует SyntaxInfo в BslPropertyInfo
    fn convert_to_property_info(&self, syntax_info: SyntaxInfo) -> Result<BslPropertyInfo> {
        let property_info = BslPropertyInfo {
            name: syntax_info.title,
            property_type: "Variant".to_string(), // По умолчанию
            access_mode: AccessMode::ReadWrite,   // По умолчанию
            description: if syntax_info.description.is_empty() {
                None
            } else {
                Some(syntax_info.description)
            },
            availability: if syntax_info.availability.is_empty() {
                None
            } else {
                Some(syntax_info.availability.join(", "))
            },
            object_context: None,
        };

        Ok(property_info)
    }

    /// Преобразует SyntaxInfo в BslFunctionInfo
    fn convert_to_function_info(&self, syntax_info: SyntaxInfo) -> Result<BslFunctionInfo> {
        let function_name = self.extract_method_name(&syntax_info.title);

        let mut syntax_variants = Vec::new();
        for variant in &syntax_info.syntax_variants {
            syntax_variants.push(variant.syntax.clone());
        }
        if syntax_variants.is_empty() && !syntax_info.syntax.is_empty() {
            syntax_variants.push(syntax_info.syntax);
        }

        let function_info = BslFunctionInfo {
            name: function_name,
            syntax_variants,
            parameters: syntax_info.parameters,
            return_type: if syntax_info.return_value.is_empty() {
                None
            } else {
                Some(syntax_info.return_value)
            },
            description: if syntax_info.description.is_empty() {
                None
            } else {
                Some(syntax_info.description)
            },
            category: "Global".to_string(), // По умолчанию
            availability: if syntax_info.availability.is_empty() {
                None
            } else {
                Some(syntax_info.availability.join(", "))
            },
        };

        Ok(function_info)
    }

    /// Преобразует SyntaxInfo в BslOperatorInfo
    fn convert_to_operator_info(&self, syntax_info: SyntaxInfo) -> Result<BslOperatorInfo> {
        let operator_info = BslOperatorInfo {
            operator: syntax_info.title,
            syntax: syntax_info.syntax,
            description: if syntax_info.description.is_empty() {
                None
            } else {
                Some(syntax_info.description)
            },
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    fn parse_single_parameter(&self, param: &str) -> Result<ParameterInfo> {
        let mut parameter = ParameterInfo {
            name: param.to_string(),
            param_type: None,
            type_description: None,
            description: None,
            is_optional: false,
            default_value: None,
            link: None,
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
    #[allow(dead_code)]
    fn extract_additional_info_from_description(
        &self,
        description: &str,
        method_info: &mut BslMethodInfo,
    ) {
        // Извлекаем доступность
        if let Some(availability_regex) = self.syntax_patterns.get("availability") {
            if let Some(captures) = availability_regex.captures(description) {
                method_info.availability = vec![captures[1].trim().to_string()];
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
        method_name
            .find('.')
            .map(|dot_pos| method_name[..dot_pos].to_string())
    }

    /// Добавляет стандартные ключевые слова BSL
    fn add_standard_keywords(&self, database: &mut BslSyntaxDatabase) {
        let keywords = vec![
            // Управляющие конструкции
            "Если",
            "Тогда",
            "Иначе",
            "ИначеЕсли",
            "КонецЕсли",
            "Пока",
            "Цикл",
            "КонецЦикла",
            "Для",
            "По",
            "КонецДля",
            "Попытка",
            "Исключение",
            "КонецПопытки",
            "ВызватьИсключение",
            "Возврат",
            "Продолжить",
            "Прервать",
            // Объявления
            "Процедура",
            "КонецПроцедуры",
            "Функция",
            "КонецФункции",
            "Экспорт",
            "Перем",
            "Знач",
            // Логические операторы
            "И",
            "ИЛИ",
            "НЕ",
            "Истина",
            "Ложь",
            "Неопределено",
            "NULL",
            // Типы данных
            "Число",
            "Строка",
            "Дата",
            "Булево",
            "Тип",
            "ТипЗнч",
            // Прочие
            "Новый",
            "Как",
        ];

        database.keywords = keywords.into_iter().map(|s| s.to_string()).collect();
    }
}

/// Промежуточная структура для хранения распарсенной синтаксической информации
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyntaxInfo {
    pub filename: String,
    pub title: String,
    pub syntax: String,
    pub syntax_variants: Vec<SyntaxVariant>,
    pub description: String,
    pub parameters: Vec<ParameterInfo>,
    pub parameters_by_variant: HashMap<String, Vec<ParameterInfo>>,
    pub return_value: String,
    pub example: String,
    pub category: String,
    pub links: Vec<LinkInfo>,
    pub availability: Vec<String>,
    pub version: String,
    pub methods: Vec<MethodInfo>,
    pub collection_elements: Option<CollectionElementsInfo>,
    pub object_context: Option<String>,
}

/// Информация о методе объекта
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MethodInfo {
    pub name: String,
    pub english_name: String,
    pub full_name: String,
}

impl BslSyntaxDatabase {
    /// Поиск методов по запросу
    pub fn search_methods(&self, query: &str) -> Vec<&BslMethodInfo> {
        let query_lower = query.to_lowercase();
        self.methods
            .values()
            .filter(|method| {
                method.name.to_lowercase().contains(&query_lower)
                    || method
                        .description
                        .as_ref()
                        .is_some_and(|d| d.to_lowercase().contains(&query_lower))
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
                    detail: method.syntax_variants.first().map(|v| v.syntax.clone()),
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
            let params: Vec<String> = method
                .parameters
                .iter()
                .enumerate()
                .map(|(i, param)| format!("${{{i}:{}}}", param.name))
                .collect();
            format!("{}({})", method.name, params.join(", "))
        }
    }

    /// Генерирует текст для вставки функции с параметрами
    fn generate_function_insert_text(&self, function: &BslFunctionInfo) -> String {
        if function.parameters.is_empty() {
            format!("{}()", function.name)
        } else {
            let params: Vec<String> = function
                .parameters
                .iter()
                .enumerate()
                .map(|(i, param)| format!("${{{i}:{}}}", param.name))
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

impl BslSyntaxExtractor {
    /// Извлекает директивы компиляции из файла Pragma.html
    fn extract_pragma_directives(&self, html_content: &str) -> Result<Vec<String>> {
        let document = Html::parse_document(html_content);
        let mut directives = Vec::new();

        // Ищем все теги <STRONG> которые содержат директивы (&НаКлиенте, &НаСервере и т.д.)
        let strong_selector = Selector::parse("strong").unwrap();

        for element in document.select(&strong_selector) {
            let text = element.text().collect::<String>();

            // Проверяем что это директива компиляции (начинается с &)
            if text.starts_with('&') && text.len() > 1 {
                // Извлекаем русскую и английскую версии
                if text.contains('(') && text.contains(')') {
                    // Формат: &НаКлиенте (&AtClient)
                    let parts: Vec<&str> = text.split('(').collect();
                    if parts.len() >= 2 {
                        let russian_directive = parts[0].trim();
                        let english_part = parts[1].replace(')', "");
                        let english_directive = english_part.trim();

                        if !russian_directive.is_empty() {
                            directives.push(russian_directive.to_string());
                        }
                        if !english_directive.is_empty() {
                            directives.push(english_directive.to_string());
                        }
                    }
                } else if !text.is_empty() {
                    // Простая директива без скобок
                    directives.push(text);
                }
            }
        }

        // Удаляем дубликаты и сортируем
        directives.sort();
        directives.dedup();

        tracing::debug!("Extracted {} compilation directives", directives.len());
        Ok(directives)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_method_name() {
        let extractor = BslSyntaxExtractor::new("test.hbk");

        assert_eq!(extractor.extract_method_name("Сообщить()"), "Сообщить");
        assert_eq!(
            extractor.extract_method_name("НайтиПоРеквизиту(Значение)"),
            "НайтиПоРеквизиту"
        );
        assert_eq!(
            extractor.extract_method_name("Метод без скобок"),
            "Метод без скобок"
        );
    }

    #[test]
    fn test_basic_extraction() {
        let _extractor = BslSyntaxExtractor::new("test.hbk");
        // Базовый тест создания экстрактора
        assert!(true);
    }
}
