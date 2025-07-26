/*!
# Metadata Report Parser

Парсер отчетов конфигурации 1С.
Портирован с Python проекта onec-contract-generator на Rust.

Основные возможности:
- Парсинг текстовых отчетов конфигурации 1С (не XML!)
- Поддержка множественных кодировок (UTF-16, UTF-8, CP1251)
- Извлечение метаданных объектов конфигурации
- Генерация типобезопасных контрактов метаданных

## Использование

```rust
let parser = MetadataReportParser::new();
let contracts = parser.parse_report("config_report.txt")?;
```

## Важно

Этот парсер работает с ТЕКСТОВЫМИ ОТЧЕТАМИ конфигурации,
а не с XML файлами Configuration.xml. Отчет можно получить
через конфигуратор 1С: "Конфигурация" -> "Отчеты" -> "Структура хранения".
*/

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use encoding_rs::{UTF_16LE, UTF_8, WINDOWS_1251};
use anyhow::{Context, Result};
use regex::Regex;
use chrono::Utc;

/// Контракт метаданных объекта 1С (замена Python MetadataContract)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataContract {
    pub metadata_type: String, // Всегда "Metadata"
    pub name: String,
    pub object_type: ObjectType,
    pub structure: ObjectStructure,
    pub search_keywords: Vec<String>,
    pub generation_metadata: GenerationMetadata,
}

/// Типы объектов 1С (замена Python ALLOWED_ROOT_TYPES)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ObjectType {
    Directory,       // Справочник
    Document,        // Документ
    Register,        // Регистр
    InformationRegister, // РегистрСведений
    AccumulationRegister, // РегистрНакопления
    AccountingRegister,   // РегистрБухгалтерии
    Report,          // Отчет
    DataProcessor,   // Обработка
    Enumeration,     // Перечисление
    CommonModule,    // ОбщийМодуль
    Subsystem,       // Подсистема
    Role,           // Роль
    CommonAttribute, // ОбщийРеквизит
    ExchangePlan,   // ПланОбмена
    FilterCriterion, // КритерийОтбора
    SettingsStorage, // ХранилищеНастроек
    FunctionalOption, // ФункциональнаяОпция
    DefinedType,    // ОпределяемыйТип
    WebService,     // WebСервис
    HTTPService,    // HTTPСервис
    ScheduledJob,   // РегламентноеЗадание
    Constant,       // Константа
    Sequence,       // Последовательность
    DocumentJournal, // ЖурналДокументов
    ChartOfCharacteristicTypes, // ПланВидовХарактеристик
    ChartOfAccounts,    // ПланСчетов
    ChartOfCalculationTypes, // ПланВидовРасчета
    BusinessProcess,     // БизнесПроцесс
    Task,               // Задача
    ExternalDataSource, // ВнешнийИсточникДанных
}

impl std::fmt::Display for ObjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            ObjectType::Directory => "Справочник",
            ObjectType::Document => "Документ", 
            ObjectType::Register => "Регистр",
            ObjectType::InformationRegister => "РегистрСведений",
            ObjectType::AccumulationRegister => "РегистрНакопления",
            ObjectType::AccountingRegister => "РегистрБухгалтерии",
            ObjectType::Report => "Отчет",
            ObjectType::DataProcessor => "Обработка",
            ObjectType::Enumeration => "Перечисление",
            ObjectType::CommonModule => "ОбщийМодуль",
            ObjectType::Subsystem => "Подсистема",
            ObjectType::Role => "Роль",
            ObjectType::CommonAttribute => "ОбщийРеквизит",
            ObjectType::ExchangePlan => "ПланОбмена",
            ObjectType::FilterCriterion => "КритерийОтбора",
            ObjectType::SettingsStorage => "ХранилищеНастроек",
            ObjectType::FunctionalOption => "ФункциональнаяОпция",
            ObjectType::DefinedType => "ОпределяемыйТип",
            ObjectType::WebService => "WebСервис",
            ObjectType::HTTPService => "HTTPСервис",
            ObjectType::ScheduledJob => "РегламентноеЗадание",
            ObjectType::Constant => "Константа",
            ObjectType::Sequence => "Последовательность",
            ObjectType::DocumentJournal => "ЖурналДокументов",
            ObjectType::ChartOfCharacteristicTypes => "ПланВидовХарактеристик",
            ObjectType::ChartOfAccounts => "ПланСчетов",
            ObjectType::ChartOfCalculationTypes => "ПланВидовРасчета",
            ObjectType::BusinessProcess => "БизнесПроцесс",
            ObjectType::Task => "Задача",
            ObjectType::ExternalDataSource => "ВнешнийИсточникДанных",
        };
        write!(f, "{}", name)
    }
}

/// Структура объекта конфигурации
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectStructure {
    pub attributes: Vec<AttributeInfo>,
    pub tabular_sections: Vec<TabularSection>,
    pub forms: Vec<String>,
    pub templates: Vec<String>,
    pub commands: Vec<String>,
    pub comments: Option<String>,
}

/// Информация о реквизите
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeInfo {
    pub name: String,
    pub data_type: String,
    pub length: Option<u32>,
    pub precision: Option<u32>,
    pub attribute_use: AttributeUse, // Переименовано из "use" (ключевое слово)
    pub indexing: AttributeIndexing,
    pub fill_checking: FillChecking,
}

/// Табличная часть
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabularSection {
    pub name: String,
    pub attributes: Vec<AttributeInfo>,
    pub indexing: Option<String>,
}

/// Назначение реквизита
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttributeUse {
    ForFolderAndItem, // ДляПапкиИЭлемента
    ForFolder,        // ДляПапки
    ForItem,          // ДляЭлемента
}

/// Индексирование реквизита
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttributeIndexing {
    Index,            // Индексировать
    DontIndex,        // НеИндексировать
    IndexWithAdditionalOrder, // ИндексироватьСДополнительнымПорядком
}

/// Проверка заполнения
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FillChecking {
    DontCheck,        // НеПроверять
    ShowError,        // ВыдаватьОшибку
    ShowWarning,      // ВыдаватьПредупреждение
}

/// Метаданные генерации
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationMetadata {
    pub timestamp: String,
    pub generator_version: String,
    pub source_file: String,
    pub encoding_used: String,
}

/// Парсер отчетов конфигурации (замена Python MetadataGenerator)
pub struct MetadataReportParser {
    allowed_root_types: HashMap<String, ObjectType>,
    section_patterns: HashMap<String, Regex>,
    attribute_pattern: Regex,
    tabular_section_pattern: Regex,
}

impl MetadataReportParser {
    /// Создает новый парсер отчетов метаданных
    pub fn new() -> Result<Self> {
        let mut allowed_root_types = HashMap::new();
        
        // Инициализируем разрешенные типы объектов (из Python ALLOWED_ROOT_TYPES)
        allowed_root_types.insert("Справочник".to_string(), ObjectType::Directory);
        allowed_root_types.insert("Документ".to_string(), ObjectType::Document);
        allowed_root_types.insert("Регистр".to_string(), ObjectType::Register);
        allowed_root_types.insert("РегистрСведений".to_string(), ObjectType::InformationRegister);
        allowed_root_types.insert("РегистрНакопления".to_string(), ObjectType::AccumulationRegister);
        allowed_root_types.insert("РегистрБухгалтерии".to_string(), ObjectType::AccountingRegister);
        allowed_root_types.insert("Отчет".to_string(), ObjectType::Report);
        allowed_root_types.insert("Обработка".to_string(), ObjectType::DataProcessor);
        allowed_root_types.insert("Перечисление".to_string(), ObjectType::Enumeration);
        allowed_root_types.insert("ОбщийМодуль".to_string(), ObjectType::CommonModule);
        allowed_root_types.insert("Подсистема".to_string(), ObjectType::Subsystem);
        allowed_root_types.insert("Роль".to_string(), ObjectType::Role);
        allowed_root_types.insert("ОбщийРеквизит".to_string(), ObjectType::CommonAttribute);
        allowed_root_types.insert("ПланОбмена".to_string(), ObjectType::ExchangePlan);
        allowed_root_types.insert("Константа".to_string(), ObjectType::Constant);
        
        // Компилируем регулярные выражения для парсинга
        let mut section_patterns = HashMap::new();
        section_patterns.insert("object_header".to_string(), 
            Regex::new(r"^(\w+)\.(\w+)$").context("Failed to compile object header regex")?);
        section_patterns.insert("attributes_section".to_string(),
            Regex::new(r"^\s*Реквизиты:").context("Failed to compile attributes section regex")?);
        section_patterns.insert("tabular_sections".to_string(),
            Regex::new(r"^\s*Табличные части:").context("Failed to compile tabular sections regex")?);
        section_patterns.insert("forms_section".to_string(),
            Regex::new(r"^\s*Формы:").context("Failed to compile forms section regex")?);
        
        let attribute_pattern = Regex::new(r"^\s*(\w+)\s*\(([^)]+)\)")
            .context("Failed to compile attribute regex")?;
        let tabular_section_pattern = Regex::new(r"^\s*(\w+)\s*:")
            .context("Failed to compile tabular section regex")?;
        
        Ok(Self {
            allowed_root_types,
            section_patterns,
            attribute_pattern,
            tabular_section_pattern,
        })
    }
    
    /// Парсит отчет конфигурации (замена Python parse_report)
    pub fn parse_report<P: AsRef<Path>>(&self, report_path: P) -> Result<Vec<MetadataContract>> {
        let path = report_path.as_ref();
        tracing::info!("Parsing configuration report: {}", path.display());
        
        // Проверяем существование файла
        if !path.exists() {
            anyhow::bail!("Report file not found: {}", path.display());
        }
        
        // Читаем файл с поддержкой множественных кодировок
        let (content, encoding_used) = self.read_with_encoding_fallback(path)?;
        
        // Парсим содержимое отчета
        let contracts = self.extract_metadata_objects(&content, path, &encoding_used)?;
        
        tracing::info!("Parsed {} metadata objects from report", contracts.len());
        Ok(contracts)
    }
    
    /// Читает файл с fallback по кодировкам (замена Python encoding handling)
    fn read_with_encoding_fallback(&self, path: &Path) -> Result<(String, String)> {
        let file_bytes = std::fs::read(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;
        
        // Пробуем кодировки в порядке приоритета: UTF-16LE -> UTF-8 -> Windows-1251
        let encodings = [
            ("UTF-16LE", UTF_16LE),
            ("UTF-8", UTF_8),
            ("Windows-1251", WINDOWS_1251),
        ];
        
        for (name, encoding) in &encodings {
            let (decoded, _, had_errors) = encoding.decode(&file_bytes);
            
            if !had_errors {
                tracing::debug!("Successfully decoded file with {} encoding", name);
                return Ok((decoded.into_owned(), name.to_string()));
            }
        }
        
        // Если все кодировки не сработали, используем UTF-8 с заменой ошибочных символов
        let (decoded, _, _) = UTF_8.decode(&file_bytes);
        tracing::warn!("Used UTF-8 with error replacement for file: {}", path.display());
        Ok((decoded.into_owned(), "UTF-8 (with errors)".to_string()))
    }
    
    /// Извлекает объекты метаданных из содержимого отчета
    fn extract_metadata_objects(&self, content: &str, source_path: &Path, encoding: &str) -> Result<Vec<MetadataContract>> {
        let mut contracts = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        
        while i < lines.len() {
            let line = lines[i].trim();
            
            // Ищем заголовки объектов (например, "Справочник.Номенклатура")
            if let Some(captures) = self.section_patterns["object_header"].captures(line) {
                let object_type_str = &captures[1];
                let object_name = &captures[2];
                
                if let Some(object_type) = self.allowed_root_types.get(object_type_str) {
                    tracing::debug!("Found object: {} {}", object_type_str, object_name);
                    
                    // Парсим объект начиная с текущей позиции
                    let (contract, lines_consumed) = self.parse_single_object(
                        &lines[i..],
                        object_name,
                        object_type.clone(),
                        source_path,
                        encoding
                    )?;
                    
                    contracts.push(contract);
                    i += lines_consumed;
                } else {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }
        
        Ok(contracts)
    }
    
    /// Парсит один объект метаданных
    fn parse_single_object(
        &self,
        lines: &[&str],
        object_name: &str,
        object_type: ObjectType,
        source_path: &Path,
        encoding: &str
    ) -> Result<(MetadataContract, usize)> {
        let mut structure = ObjectStructure {
            attributes: Vec::new(),
            tabular_sections: Vec::new(),
            forms: Vec::new(),
            templates: Vec::new(),
            commands: Vec::new(),
            comments: None,
        };
        
        let mut i = 1; // Пропускаем заголовок объекта
        let mut current_section = None;
        let mut current_tabular_section: Option<TabularSection> = None;
        
        // Парсим секции объекта
        while i < lines.len() {
            let line = lines[i].trim();
            
            // Проверяем, не начался ли новый объект
            if self.section_patterns["object_header"].is_match(line) {
                break;
            }
            
            // Определяем текущую секцию
            if self.section_patterns["attributes_section"].is_match(line) {
                current_section = Some("attributes");
                i += 1;
                continue;
            } else if self.section_patterns["tabular_sections"].is_match(line) {
                // Сохраняем предыдущую табличную часть
                if let Some(ts) = current_tabular_section.take() {
                    structure.tabular_sections.push(ts);
                }
                current_section = Some("tabular_sections");
                i += 1;
                continue;
            } else if self.section_patterns["forms_section"].is_match(line) {
                current_section = Some("forms");
                i += 1;
                continue;
            }
            
            // Обрабатываем содержимое секций
            match current_section {
                Some("attributes") => {
                    if let Some(attribute) = self.parse_attribute_line(line)? {
                        structure.attributes.push(attribute);
                    }
                }
                Some("tabular_sections") => {
                    // Проверяем, начинается ли новая табличная часть
                    if let Some(captures) = self.tabular_section_pattern.captures(line) {
                        // Сохраняем предыдущую табличную часть
                        if let Some(ts) = current_tabular_section.take() {
                            structure.tabular_sections.push(ts);
                        }
                        
                        // Начинаем новую табличную часть
                        current_tabular_section = Some(TabularSection {
                            name: captures[1].to_string(),
                            attributes: Vec::new(),
                            indexing: None,
                        });
                    } else if let Some(ref mut ts) = current_tabular_section {
                        // Добавляем реквизит к текущей табличной части
                        if let Some(attribute) = self.parse_attribute_line(line)? {
                            ts.attributes.push(attribute);
                        }
                    }
                }
                Some("forms") => {
                    if !line.is_empty() && !line.starts_with('-') {
                        structure.forms.push(line.to_string());
                    }
                }
                _ => {
                    // Ищем комментарии или другую информацию
                    if line.starts_with("//") || line.starts_with("/*") {
                        if structure.comments.is_none() {
                            structure.comments = Some(line.to_string());
                        } else {
                            structure.comments = Some(format!("{}\n{}", structure.comments.as_ref().unwrap(), line));
                        }
                    }
                }
            }
            
            i += 1;
        }
        
        // Сохраняем последнюю табличную часть
        if let Some(ts) = current_tabular_section {
            structure.tabular_sections.push(ts);
        }
        
        // Создаем контракт
        let contract = MetadataContract {
            metadata_type: "Metadata".to_string(),
            name: object_name.to_string(),
            object_type,
            structure,
            search_keywords: self.generate_search_keywords(object_name),
            generation_metadata: GenerationMetadata {
                timestamp: Utc::now().to_rfc3339(),
                generator_version: env!("CARGO_PKG_VERSION").to_string(),
                source_file: source_path.display().to_string(),
                encoding_used: encoding.to_string(),
            },
        };
        
        Ok((contract, i))
    }
    
    /// Парсит строку с реквизитом
    fn parse_attribute_line(&self, line: &str) -> Result<Option<AttributeInfo>> {
        if line.is_empty() || line.starts_with('-') || line.starts_with("//") {
            return Ok(None);
        }
        
        if let Some(captures) = self.attribute_pattern.captures(line) {
            let name = captures[1].to_string();
            let type_info = &captures[2];
            
            let attribute = AttributeInfo {
                name,
                data_type: self.parse_data_type(type_info),
                length: self.extract_length(type_info),
                precision: self.extract_precision(type_info),
                attribute_use: AttributeUse::ForFolderAndItem, // По умолчанию
                indexing: AttributeIndexing::DontIndex, // По умолчанию  
                fill_checking: FillChecking::DontCheck, // По умолчанию
            };
            
            Ok(Some(attribute))
        } else {
            Ok(None)
        }
    }
    
    /// Парсит тип данных из строки типа
    fn parse_data_type(&self, type_info: &str) -> String {
        // Извлекаем основной тип данных
        if let Some(paren_pos) = type_info.find('(') {
            type_info[..paren_pos].to_string()
        } else if let Some(space_pos) = type_info.find(' ') {
            type_info[..space_pos].to_string()
        } else {
            type_info.to_string()
        }
    }
    
    /// Извлекает длину из информации о типе
    fn extract_length(&self, type_info: &str) -> Option<u32> {
        // Ищем число в скобках для строковых типов
        if type_info.contains("Строка") {
            let re = Regex::new(r"\((\d+)\)").ok()?;
            if let Some(captures) = re.captures(type_info) {
                return captures[1].parse().ok();
            }
        }
        None
    }
    
    /// Извлекает точность из информации о типе
    fn extract_precision(&self, type_info: &str) -> Option<u32> {
        // Ищем точность для числовых типов (например, "Число(10,2)")
        if type_info.contains("Число") {
            let re = Regex::new(r"\(\d+,(\d+)\)").ok()?;
            if let Some(captures) = re.captures(type_info) {
                return captures[1].parse().ok();
            }
        }
        None
    }
    
    /// Генерирует ключевые слова для поиска
    fn generate_search_keywords(&self, object_name: &str) -> Vec<String> {
        let mut keywords = vec![object_name.to_string()];
        
        // Добавляем части имени, разделенные по camelCase
        let parts = self.split_camel_case(object_name);
        keywords.extend(parts);
        
        keywords
    }
    
    /// Разбивает строку по camelCase
    fn split_camel_case(&self, s: &str) -> Vec<String> {
        let re = Regex::new(r"([A-ZА-Я][a-zа-я]*)|([0-9]+)").unwrap();
        re.find_iter(s)
            .map(|m| m.as_str().to_string())
            .collect()
    }
    
    /// Ищет файл отчета конфигурации в директории
    pub fn find_configuration_report<P: AsRef<Path>>(config_dir: P) -> Result<Option<PathBuf>> {
        let config_dir = config_dir.as_ref();
        tracing::debug!("Looking for configuration report in: {}", config_dir.display());
        
        // Возможные имена файлов отчетов
        let report_names = [
            "ConfigurationReport.txt",
            "config_report.txt", 
            "отчет_конфигурации.txt",
            "structure_report.txt",
            "metadata_report.txt",
        ];
        
        for name in &report_names {
            let report_path = config_dir.join(name);
            if report_path.exists() {
                tracing::info!("Found configuration report: {}", report_path.display());
                return Ok(Some(report_path));
            }
        }
        
        // Ищем любые .txt файлы, содержащие структуру конфигурации
        for entry in std::fs::read_dir(config_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("txt") {
                // Проверяем, содержит ли файл метаданные конфигурации
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if content.contains("Справочник.") || content.contains("Документ.") {
                        tracing::info!("Found potential configuration report: {}", path.display());
                        return Ok(Some(path));
                    }
                }
            }
        }
        
        tracing::warn!("No configuration report found in: {}", config_dir.display());
        Ok(None)
    }
}

impl Default for MetadataReportParser {
    fn default() -> Self {
        Self::new().expect("Failed to create default MetadataReportParser")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    
    #[test]
    fn test_metadata_parser_creation() {
        let parser = MetadataReportParser::new().unwrap();
        assert!(!parser.allowed_root_types.is_empty());
        assert!(parser.allowed_root_types.contains_key("Справочник"));
        assert!(parser.allowed_root_types.contains_key("Документ"));
    }
    
    #[test]
    fn test_object_type_mapping() {
        let parser = MetadataReportParser::new().unwrap();
        assert_eq!(parser.allowed_root_types["Справочник"], ObjectType::Directory);
        assert_eq!(parser.allowed_root_types["Документ"], ObjectType::Document);
        assert_eq!(parser.allowed_root_types["Отчет"], ObjectType::Report);
    }
    
    #[test]
    fn test_split_camel_case() {
        let parser = MetadataReportParser::new().unwrap();
        let parts = parser.split_camel_case("НоменклатураТоваров");
        assert!(parts.contains(&"Номенклатура".to_string()));
        assert!(parts.contains(&"Товаров".to_string()));
    }
    
    #[test]
    fn test_parse_data_type() {
        let parser = MetadataReportParser::new().unwrap();
        assert_eq!(parser.parse_data_type("Строка(100)"), "Строка");
        assert_eq!(parser.parse_data_type("Число(10,2)"), "Число");
        assert_eq!(parser.parse_data_type("Булево"), "Булево");
    }
    
    #[test]
    fn test_extract_length() {
        let parser = MetadataReportParser::new().unwrap();
        assert_eq!(parser.extract_length("Строка(100)"), Some(100));
        assert_eq!(parser.extract_length("Строка(255)"), Some(255));
        assert_eq!(parser.extract_length("Булево"), None);
    }
    
    #[test]
    fn test_extract_precision() {
        let parser = MetadataReportParser::new().unwrap();
        assert_eq!(parser.extract_precision("Число(10,2)"), Some(2));
        assert_eq!(parser.extract_precision("Число(15,4)"), Some(4));
        assert_eq!(parser.extract_precision("Строка(100)"), None);
    }
    
    #[test]
    fn test_find_configuration_report() {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path();
        
        // Создаем тестовый файл отчета
        let report_content = "Справочник.Номенклатура\nРеквизиты:\n  Код (Строка(9))";
        fs::write(config_dir.join("config_report.txt"), report_content).unwrap();
        
        let found = MetadataReportParser::find_configuration_report(config_dir).unwrap();
        assert!(found.is_some());
        assert!(found.unwrap().file_name().unwrap() == "config_report.txt");
    }
}