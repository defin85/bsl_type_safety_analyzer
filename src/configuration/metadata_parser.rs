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
        // Используем множественные формы, как в реальных отчетах
        allowed_root_types.insert("Справочники".to_string(), ObjectType::Directory);
        allowed_root_types.insert("Документы".to_string(), ObjectType::Document);
        allowed_root_types.insert("Регистры".to_string(), ObjectType::Register);
        allowed_root_types.insert("РегистрыСведений".to_string(), ObjectType::InformationRegister);
        allowed_root_types.insert("РегистрыНакопления".to_string(), ObjectType::AccumulationRegister);
        allowed_root_types.insert("РегистрыБухгалтерии".to_string(), ObjectType::AccountingRegister);
        allowed_root_types.insert("Отчеты".to_string(), ObjectType::Report);
        allowed_root_types.insert("Обработки".to_string(), ObjectType::DataProcessor);
        allowed_root_types.insert("Перечисления".to_string(), ObjectType::Enumeration);
        allowed_root_types.insert("ОбщиеМодули".to_string(), ObjectType::CommonModule);
        allowed_root_types.insert("Подсистемы".to_string(), ObjectType::Subsystem);
        allowed_root_types.insert("Роли".to_string(), ObjectType::Role);
        allowed_root_types.insert("ОбщиеРеквизиты".to_string(), ObjectType::CommonAttribute);
        allowed_root_types.insert("ПланыОбмена".to_string(), ObjectType::ExchangePlan);
        allowed_root_types.insert("Константы".to_string(), ObjectType::Constant);
        
        // Дополнительные типы из Python версии
        allowed_root_types.insert("Конфигурации".to_string(), ObjectType::Document); // TODO: Добавить отдельный тип
        allowed_root_types.insert("Языки".to_string(), ObjectType::CommonModule); // TODO: Добавить отдельный тип
        allowed_root_types.insert("ПланыСчетов".to_string(), ObjectType::ChartOfAccounts);
        allowed_root_types.insert("РегистрыБухгалтерии".to_string(), ObjectType::AccountingRegister);
        allowed_root_types.insert("РегистрыРасчета".to_string(), ObjectType::Register); // TODO: Добавить отдельный тип
        allowed_root_types.insert("ПланыВидовРасчета".to_string(), ObjectType::ChartOfCalculationTypes);
        allowed_root_types.insert("ПланыВидовХарактеристик".to_string(), ObjectType::ChartOfCharacteristicTypes);
        allowed_root_types.insert("ЖурналыДокументов".to_string(), ObjectType::DocumentJournal);
        allowed_root_types.insert("БизнесПроцессы".to_string(), ObjectType::BusinessProcess);
        allowed_root_types.insert("Задачи".to_string(), ObjectType::Task);
        allowed_root_types.insert("ВнешниеИсточникиДанных".to_string(), ObjectType::ExternalDataSource);
        allowed_root_types.insert("HTTPСервисы".to_string(), ObjectType::HTTPService);
        allowed_root_types.insert("WebСервисы".to_string(), ObjectType::WebService);
        allowed_root_types.insert("ОпределяемыеТипы".to_string(), ObjectType::DefinedType);
        allowed_root_types.insert("РегламентныеЗадания".to_string(), ObjectType::ScheduledJob);
        allowed_root_types.insert("ХранилищаНастроек".to_string(), ObjectType::SettingsStorage);
        allowed_root_types.insert("ФункциональныеОпции".to_string(), ObjectType::FunctionalOption);
        allowed_root_types.insert("ПараметрыСеанса".to_string(), ObjectType::Constant); // TODO: Добавить отдельный тип
        allowed_root_types.insert("ОбщиеКоманды".to_string(), ObjectType::CommonModule); // TODO: Добавить отдельный тип
        allowed_root_types.insert("ОбщиеКартинки".to_string(), ObjectType::CommonModule); // TODO: Добавить отдельный тип
        allowed_root_types.insert("КритерииОтбора".to_string(), ObjectType::FilterCriterion);
        allowed_root_types.insert("ПодпискиНаСобытия".to_string(), ObjectType::CommonModule); // TODO: Добавить отдельный тип
        allowed_root_types.insert("Последовательности".to_string(), ObjectType::Sequence);
        
        // Компилируем регулярные выражения для парсинга
        let mut section_patterns = HashMap::new();
        section_patterns.insert("object_header".to_string(), 
            Regex::new(r"^\s*-\s*([\w\u0400-\u04FF]+)\.([\w\u0400-\u04FF]+)$").context("Failed to compile object header regex")?);
        section_patterns.insert("attributes_section".to_string(),
            Regex::new(r"^\s*Реквизиты:").context("Failed to compile attributes section regex")?);
        section_patterns.insert("tabular_sections".to_string(),
            Regex::new(r"^\s*Табличные части:").context("Failed to compile tabular sections regex")?);
        section_patterns.insert("forms_section".to_string(),
            Regex::new(r"^\s*Формы:").context("Failed to compile forms section regex")?);
        
        let attribute_pattern = Regex::new(r"^\s*([\w\u0400-\u04FF]+)\s*\(([^)]+)\)")
            .context("Failed to compile attribute regex")?;
        let tabular_section_pattern = Regex::new(r"^\s*([\w\u0400-\u04FF]+)\s*:")
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
        let mut current_object: Option<(String, ObjectType, usize)> = None;
        let mut current_structure: Option<ObjectStructure> = None;
        let mut last_element_path: Option<String> = None; // Путь к последнему элементу для обработки типа
        
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            // Пропускаем пустые строки
            if trimmed.is_empty() {
                continue;
            }
            
            // Обработка строк с "-"
            if trimmed.starts_with("-") {
                let element_line = trimmed[1..].trim();
                let parts: Vec<&str> = element_line.split('.').collect();
                
                // Проверка корневого объекта
                if parts.len() == 2 && self.allowed_root_types.contains_key(parts[0]) {
                    // Сохраняем предыдущий объект
                    if let (Some((obj_name, obj_type, start_line)), Some(structure)) = (current_object.take(), current_structure.take()) {
                        let contract = self.create_contract(&obj_name, obj_type, structure, source_path, encoding);
                        contracts.push(contract);
                    }
                    
                    // Начинаем новый объект
                    let object_type = self.allowed_root_types[parts[0]].clone();
                    let full_name = element_line.to_string();
                    tracing::debug!("Found object: {}", full_name);
                    
                    current_object = Some((full_name, object_type, i));
                    current_structure = Some(ObjectStructure {
                        attributes: Vec::new(),
                        tabular_sections: Vec::new(),
                        forms: Vec::new(),
                        templates: Vec::new(),
                        commands: Vec::new(),
                        comments: None,
                    });
                    last_element_path = None;
                    
                } else if let Some((ref obj_name, _, _)) = current_object {
                    // Обработка дочерних элементов
                    if element_line.starts_with(obj_name) {
                        let relative_path = element_line.strip_prefix(obj_name).unwrap().trim_start_matches('.');
                        let rel_parts: Vec<&str> = relative_path.split('.').collect();
                        
                        if let Some(ref mut structure) = current_structure {
                            match rel_parts.as_slice() {
                            ["Реквизиты", attr_name] => {
                                let attr = AttributeInfo {
                                    name: attr_name.to_string(),
                                    data_type: "Неопределено".to_string(),
                                    length: None,
                                    precision: None,
                                    attribute_use: AttributeUse::ForFolderAndItem,
                                    indexing: AttributeIndexing::DontIndex,
                                    fill_checking: FillChecking::DontCheck,
                                };
                                structure.attributes.push(attr);
                                last_element_path = Some(element_line.to_string());
                                tracing::debug!("Added attribute: {}", attr_name);
                            }
                            ["ТабличныеЧасти", ts_name] => {
                                let ts = TabularSection {
                                    name: ts_name.to_string(),
                                    attributes: Vec::new(),
                                    indexing: None,
                                };
                                structure.tabular_sections.push(ts);
                                last_element_path = None;
                                tracing::debug!("Added tabular section: {}", ts_name);
                            }
                            ["ТабличныеЧасти", ts_name, "Реквизиты", col_name] => {
                                // Находим табличную часть и добавляем реквизит
                                if let Some(ts) = structure.tabular_sections.iter_mut().find(|t| t.name == *ts_name) {
                                    let attr = AttributeInfo {
                                        name: col_name.to_string(),
                                        data_type: "Неопределено".to_string(),
                                        length: None,
                                        precision: None,
                                        attribute_use: AttributeUse::ForFolderAndItem,
                                        indexing: AttributeIndexing::DontIndex,
                                        fill_checking: FillChecking::DontCheck,
                                    };
                                    ts.attributes.push(attr);
                                    last_element_path = Some(element_line.to_string());
                                    tracing::debug!("Added tabular attribute {} to {}", col_name, ts_name);
                                }
                            }
                            ["Формы", form_name] => {
                                structure.forms.push(form_name.to_string());
                                tracing::debug!("Added form: {}", form_name);
                            }
                            _ => {}
                            }
                        }
                    }
                }
            }
            // Обработка поля "Тип:"
            else if trimmed.starts_with("Тип:") && last_element_path.is_some() {
                // Собираем многострочный тип
                let mut type_parts = Vec::new();
                let base_indent = line.len() - line.trim_start().len();
                
                let mut j = i + 1;
                while j < lines.len() {
                    let next_line = lines[j];
                    let next_indent = next_line.len() - next_line.trim_start().len();
                    
                    if next_line.trim().is_empty() {
                        j += 1;
                        continue;
                    }
                    
                    if next_indent > base_indent {
                        let part = next_line.trim().trim_matches('"').trim_end_matches(',');
                        type_parts.push(part.to_string());
                        j += 1;
                    } else {
                        break;
                    }
                }
                
                if !type_parts.is_empty() {
                    // Обновляем тип у последнего добавленного элемента
                    if let (Some((ref obj_name, _, _)), Some(ref mut structure), Some(ref elem_path)) = 
                        (&current_object, &mut current_structure, &last_element_path) 
                    {
                        let relative_path = elem_path.strip_prefix(obj_name).unwrap().trim_start_matches('.');
                        let rel_parts: Vec<&str> = relative_path.split('.').collect();
                        
                        match rel_parts.as_slice() {
                            ["Реквизиты", attr_name] => {
                                if let Some(attr) = structure.attributes.iter_mut().find(|a| a.name == *attr_name) {
                                    attr.data_type = type_parts.join(", ");
                                    tracing::debug!("Set type for attribute {}: {}", attr.name, attr.data_type);
                                }
                            }
                            ["ТабличныеЧасти", ts_name, "Реквизиты", col_name] => {
                                if let Some(ts) = structure.tabular_sections.iter_mut().find(|t| t.name == *ts_name) {
                                    if let Some(attr) = ts.attributes.iter_mut().find(|a| a.name == *col_name) {
                                        attr.data_type = type_parts.join(", ");
                                        tracing::debug!("Set type for tabular attribute {}.{}: {}", ts_name, attr.name, attr.data_type);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            // Обработка комментария
            else if trimmed.starts_with("Комментарий:") {
                let comment = trimmed.strip_prefix("Комментарий:").unwrap().trim().trim_matches('"');
                if !comment.is_empty() {
                    if let Some(ref mut structure) = current_structure {
                        structure.comments = Some(comment.to_string());
                    }
                }
            }
        }
        
        // Сохраняем последний объект
        if let (Some((obj_name, obj_type, _)), Some(structure)) = (current_object, current_structure) {
            let contract = self.create_contract(&obj_name, obj_type, structure, source_path, encoding);
            contracts.push(contract);
        }
        
        tracing::info!("Parsed {} metadata objects", contracts.len());
        Ok(contracts)
    }
    
    /// Создает контракт из структуры объекта
    fn create_contract(
        &self,
        full_object_name: &str,
        object_type: ObjectType,
        structure: ObjectStructure,
        source_path: &Path,
        encoding: &str
    ) -> MetadataContract {
        // Извлекаем короткое имя объекта (после последней точки)
        let object_name = full_object_name.split('.').last().unwrap_or(full_object_name);
        
        MetadataContract {
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
        assert!(parser.allowed_root_types.contains_key("Справочники"));
        assert!(parser.allowed_root_types.contains_key("Документы"));
    }
    
    #[test]
    fn test_object_type_mapping() {
        let parser = MetadataReportParser::new().unwrap();
        assert_eq!(parser.allowed_root_types["Справочники"], ObjectType::Directory);
        assert_eq!(parser.allowed_root_types["Документы"], ObjectType::Document);
        assert_eq!(parser.allowed_root_types["Отчеты"], ObjectType::Report);
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