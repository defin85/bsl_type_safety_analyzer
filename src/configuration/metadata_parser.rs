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
use crate::docs_integration::hybrid_storage::{
    HybridDocumentationStorage, TypeDefinition, TypeCategory,
    MethodDefinition, PropertyDefinition, ParameterDefinition
};

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
    Configuration,      // Конфигурация
    Language,          // Язык
    CommonForm,        // ОбщаяФорма
    CommonCommand,     // ОбщаяКоманда
    CommonPicture,     // ОбщаяКартинка
    CommonTemplate,    // ОбщийМакет
    XDTOPackage,       // XDTOПакет
    Style,             // Стиль
    StyleItem,         // ЭлементСтиля
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
            ObjectType::Configuration => "Конфигурация",
            ObjectType::Language => "Язык",
            ObjectType::CommonForm => "ОбщаяФорма",
            ObjectType::CommonCommand => "ОбщаяКоманда",
            ObjectType::CommonPicture => "ОбщаяКартинка",
            ObjectType::CommonTemplate => "ОбщийМакет",
            ObjectType::XDTOPackage => "XDTOПакет",
            ObjectType::Style => "Стиль",
            ObjectType::StyleItem => "ЭлементСтиля",
        };
        write!(f, "{}", name)
    }
}

/// Структура объекта метаданных (замена Python ObjectStructure)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectStructure {
    pub attributes: Vec<AttributeInfo>,
    pub tabular_sections: Vec<TabularSection>,
    pub forms: Vec<String>,
    pub templates: Vec<String>,
    pub commands: Vec<String>,
    pub comments: Option<String>,
    // Специальные поля для регистров
    pub dimensions: Option<Vec<AttributeInfo>>, // Измерения (только для регистров)
    pub resources: Option<Vec<AttributeInfo>>,  // Ресурсы (только для регистров)
}

/// Информация о реквизите (замена Python AttributeInfo)
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

/// Табличная часть объекта
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabularSection {
    pub name: String,
    pub attributes: Vec<AttributeInfo>,
    pub indexing: Option<String>,
}

/// Использование реквизита
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
    IndexWithOrdering, // ИндексироватьСДопУпорядочиванием
    IndexWithAdditionalOrder, // ИндексироватьСДополнительнымПорядком (legacy)
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

/// Формат отчета конфигурации
#[derive(Debug, Clone, PartialEq)]
enum ReportFormat {
    RealReport,        // Реальный отчет из 1С с табуляцией и "-"
    PythonStyle,       // Формат из Python проекта с "-" и отступами
    SimplifiedExample, // Упрощенный формат примера
}


pub struct MetadataReportParser {
    allowed_root_types: HashMap<String, ObjectType>,
}

impl MetadataReportParser {
    /// Создает новый парсер отчетов метаданных
    pub fn new() -> Result<Self> {
        let mut allowed_root_types = HashMap::new();
        
        // Инициализируем разрешенные типы объектов
        // Единственное число
        allowed_root_types.insert("Справочник".to_string(), ObjectType::Directory);
        allowed_root_types.insert("Документ".to_string(), ObjectType::Document);
        allowed_root_types.insert("РегистрСведений".to_string(), ObjectType::InformationRegister);
        allowed_root_types.insert("РегистрНакопления".to_string(), ObjectType::AccumulationRegister);
        allowed_root_types.insert("РегистрБухгалтерии".to_string(), ObjectType::AccountingRegister);
        allowed_root_types.insert("Перечисление".to_string(), ObjectType::Enumeration);
        allowed_root_types.insert("ПланВидовХарактеристик".to_string(), ObjectType::ChartOfCharacteristicTypes);
        allowed_root_types.insert("ПланСчетов".to_string(), ObjectType::ChartOfAccounts);
        allowed_root_types.insert("ПланВидовРасчета".to_string(), ObjectType::ChartOfCalculationTypes);
        allowed_root_types.insert("ПланОбмена".to_string(), ObjectType::ExchangePlan);
        allowed_root_types.insert("Отчет".to_string(), ObjectType::Report);
        allowed_root_types.insert("Обработка".to_string(), ObjectType::DataProcessor);
        allowed_root_types.insert("ЖурналДокументов".to_string(), ObjectType::DocumentJournal);
        allowed_root_types.insert("Последовательность".to_string(), ObjectType::Sequence);
        allowed_root_types.insert("Задача".to_string(), ObjectType::Task);
        allowed_root_types.insert("Константа".to_string(), ObjectType::Constant);
        allowed_root_types.insert("ОбщийМодуль".to_string(), ObjectType::CommonModule);
        allowed_root_types.insert("ОбщийРеквизит".to_string(), ObjectType::CommonAttribute);
        allowed_root_types.insert("БизнесПроцесс".to_string(), ObjectType::BusinessProcess);
        allowed_root_types.insert("Роль".to_string(), ObjectType::Role);
        allowed_root_types.insert("Конфигурация".to_string(), ObjectType::Configuration);
        allowed_root_types.insert("Язык".to_string(), ObjectType::Language);
        allowed_root_types.insert("Подсистема".to_string(), ObjectType::Subsystem);
        allowed_root_types.insert("ОбщаяФорма".to_string(), ObjectType::CommonForm);
        allowed_root_types.insert("ОбщаяКоманда".to_string(), ObjectType::CommonCommand);
        allowed_root_types.insert("ОбщаяКартинка".to_string(), ObjectType::CommonPicture);
        allowed_root_types.insert("ОбщийМакет".to_string(), ObjectType::CommonTemplate);
        allowed_root_types.insert("XDTOПакет".to_string(), ObjectType::XDTOPackage);
        allowed_root_types.insert("WebСервис".to_string(), ObjectType::WebService);
        allowed_root_types.insert("HTTPСервис".to_string(), ObjectType::HTTPService);
        allowed_root_types.insert("КритерийОтбора".to_string(), ObjectType::FilterCriterion);
        allowed_root_types.insert("ХранилищеНастроек".to_string(), ObjectType::SettingsStorage);
        allowed_root_types.insert("ФункциональнаяОпция".to_string(), ObjectType::FunctionalOption);
        allowed_root_types.insert("ОпределяемыйТип".to_string(), ObjectType::DefinedType);
        allowed_root_types.insert("РегламентноеЗадание".to_string(), ObjectType::ScheduledJob);
        allowed_root_types.insert("ВнешнийИсточникДанных".to_string(), ObjectType::ExternalDataSource);
        allowed_root_types.insert("Стиль".to_string(), ObjectType::Style);
        allowed_root_types.insert("ЭлементСтиля".to_string(), ObjectType::StyleItem);
        
        // Множественное число (для совместимости с Python)
        allowed_root_types.insert("Справочники".to_string(), ObjectType::Directory);
        allowed_root_types.insert("Документы".to_string(), ObjectType::Document);
        allowed_root_types.insert("Константы".to_string(), ObjectType::Constant);
        allowed_root_types.insert("ОбщиеФормы".to_string(), ObjectType::CommonForm);
        allowed_root_types.insert("Отчеты".to_string(), ObjectType::Report);
        allowed_root_types.insert("Обработки".to_string(), ObjectType::DataProcessor);
        allowed_root_types.insert("РегистрыСведений".to_string(), ObjectType::InformationRegister);
        allowed_root_types.insert("РегистрыНакопления".to_string(), ObjectType::AccumulationRegister);
        allowed_root_types.insert("ПланыВидовХарактеристик".to_string(), ObjectType::ChartOfCharacteristicTypes);
        allowed_root_types.insert("ПланыОбмена".to_string(), ObjectType::ExchangePlan);
        allowed_root_types.insert("БизнесПроцессы".to_string(), ObjectType::BusinessProcess);
        allowed_root_types.insert("Задачи".to_string(), ObjectType::Task);
        allowed_root_types.insert("Языки".to_string(), ObjectType::Language);
        allowed_root_types.insert("Подсистемы".to_string(), ObjectType::Subsystem);
        allowed_root_types.insert("Роли".to_string(), ObjectType::Role);
        allowed_root_types.insert("ПланыСчетов".to_string(), ObjectType::ChartOfAccounts);
        allowed_root_types.insert("РегистрыБухгалтерии".to_string(), ObjectType::AccountingRegister);
        allowed_root_types.insert("ПланыВидовРасчета".to_string(), ObjectType::ChartOfCalculationTypes);
        allowed_root_types.insert("Перечисления".to_string(), ObjectType::Enumeration);
        allowed_root_types.insert("ОбщиеМодули".to_string(), ObjectType::CommonModule);
        allowed_root_types.insert("HTTPСервисы".to_string(), ObjectType::HTTPService);
        allowed_root_types.insert("WebСервисы".to_string(), ObjectType::WebService);
        allowed_root_types.insert("XDTOПакеты".to_string(), ObjectType::XDTOPackage);
        allowed_root_types.insert("Стили".to_string(), ObjectType::Style);
        allowed_root_types.insert("ЭлементыСтиля".to_string(), ObjectType::StyleItem);
        allowed_root_types.insert("ХранилищаНастроек".to_string(), ObjectType::SettingsStorage);
        allowed_root_types.insert("РегламентныеЗадания".to_string(), ObjectType::ScheduledJob);
        allowed_root_types.insert("ЖурналыДокументов".to_string(), ObjectType::DocumentJournal);
        allowed_root_types.insert("ОпределяемыеТипы".to_string(), ObjectType::DefinedType);
        allowed_root_types.insert("ОбщиеКартинки".to_string(), ObjectType::CommonPicture);
        allowed_root_types.insert("ОбщиеКоманды".to_string(), ObjectType::CommonCommand);
        allowed_root_types.insert("ОбщиеРеквизиты".to_string(), ObjectType::CommonAttribute);
        allowed_root_types.insert("ФункциональныеОпции".to_string(), ObjectType::FunctionalOption);
        allowed_root_types.insert("КритерииОтбора".to_string(), ObjectType::FilterCriterion);
        
        Ok(Self {
            allowed_root_types,
        })
    }
    
    /// Парсит отчет конфигурации
    pub fn parse_report<P: AsRef<Path>>(&self, report_path: P) -> Result<Vec<MetadataContract>> {
        let path = report_path.as_ref();
        tracing::info!("Parsing configuration report: {}", path.display());
        
        // Проверяем существование файла
        if !path.exists() {
            anyhow::bail!("Report file not found: {}", path.display());
        }
        
        // Читаем и декодируем файл
        let (content, encoding) = self.read_file_with_encoding(path)?;
        
        // Извлекаем объекты метаданных
        let contracts = self.extract_metadata_objects(&content, path, &encoding)?;
        
        tracing::info!("Parsed {} metadata objects from report", contracts.len());
        Ok(contracts)
    }
    
    /// Читает файл с автоопределением кодировки
    fn read_file_with_encoding(&self, path: &Path) -> Result<(String, String)> {
        let file_bytes = std::fs::read(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;
        
        // Пробуем разные кодировки в порядке приоритета
        let encodings = [
            (UTF_16LE, "UTF-16LE"),
            (UTF_8, "UTF-8"),
            (WINDOWS_1251, "Windows-1251"),
        ];
        
        for (encoding, name) in encodings.iter() {
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
        // Определяем формат отчета
        let format = self.detect_report_format(content);
        tracing::info!("Detected report format: {:?}", format);
        
        match format {
            ReportFormat::RealReport => self.parse_real_format(content, source_path, encoding),
            ReportFormat::PythonStyle => self.parse_python_format(content, source_path, encoding),
            ReportFormat::SimplifiedExample => self.parse_simplified_format(content, source_path, encoding),
        }
    }
    
    /// Определяет формат отчета
    fn detect_report_format(&self, content: &str) -> ReportFormat {
        let lines: Vec<&str> = content.lines().take(20).collect();
        
        // Проверяем на реальный формат (с табуляцией и префиксом "-")
        for line in &lines {
            if line.trim().starts_with("-") && line.contains(".") {
                return ReportFormat::RealReport;
            }
        }
        
        // Проверяем на Python формат (с "-" и отступами)
        for line in &lines {
            if line.starts_with("- ") && line.contains(".") {
                return ReportFormat::PythonStyle;
            }
        }
        
        // По умолчанию - упрощенный формат
        ReportFormat::SimplifiedExample
    }
    
    /// Парсит реальный формат отчета 1С
    fn parse_real_format(&self, content: &str, source_path: &Path, encoding: &str) -> Result<Vec<MetadataContract>> {
        let mut contracts = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut current_object: Option<(String, String, ObjectType)> = None;
        let mut current_structure: Option<ObjectStructure> = None;
        let mut current_section: Option<String> = None;
        let mut object_info: HashMap<String, String> = HashMap::new();
        let mut current_tabular_section: Option<String> = None;
        let mut last_attribute_name: Option<String> = None;
        let mut collecting_composite_type = false;
        let mut composite_type_parts: Vec<String> = Vec::new();
        
        
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i];
            let trimmed = line.trim();
            
            if trimmed.is_empty() {
                i += 1;
                continue;
            }
            
            // Если собираем составной тип
            if collecting_composite_type {
                
                // Проверяем, содержит ли строка часть составного типа
                if trimmed.contains('"') {
                    // Извлекаем содержимое между кавычек
                    let start_quote = trimmed.find('"');
                    let end_quote = trimmed.rfind('"');
                    
                    if let (Some(start), Some(end)) = (start_quote, end_quote) {
                        if start != end {
                            // Полная строка в кавычках на одной строке
                            let content = &trimmed[start + 1..end];
                            let type_part = content.trim_end_matches(',').trim();
                            let has_comma = content.ends_with(',');
                            
                            if !type_part.is_empty() {
                                composite_type_parts.push(type_part.to_string());
                                
                                if !has_comma {
                                    // Завершаем сбор - это последняя часть
                                    let composite_type = composite_type_parts.join(", ");
                                    if let Some(ref attr_name) = last_attribute_name {
                                        self.create_attribute_with_type(attr_name, &composite_type, &current_section, &current_tabular_section, &mut current_structure, None, None);
                                    }
                                    collecting_composite_type = false;
                                    composite_type_parts.clear();
                                    // НЕ очищаем last_attribute_name - он нужен для последующих свойств
                                }
                            }
                        } else {
                            // Одна кавычка - начало или конец многострочного типа
                            if trimmed.starts_with('"') {
                                // Начало многострочного типа: "СправочникСсылка.Контрагенты,
                                let content = trimmed.trim_start_matches('"');
                                let type_part = content.trim_end_matches(',').trim();
                                if !type_part.is_empty() {
                                    composite_type_parts.push(type_part.to_string());
                                }
                            } else if trimmed.ends_with('"') {
                                // Конец многострочного типа:  Строка(10, Переменная)"
                                let content = trimmed.trim_end_matches('"');
                                let type_part = content.trim_end_matches(',').trim();
                                let _has_comma = content.ends_with(',');
                                
                                if !type_part.is_empty() {
                                    composite_type_parts.push(type_part.to_string());
                                }
                                
                                // Завершаем сбор многострочного типа
                                let composite_type = composite_type_parts.join(", ");
                                if let Some(ref attr_name) = last_attribute_name {
                                    self.create_attribute_with_type(attr_name, &composite_type, &current_section, &current_tabular_section, &mut current_structure, None, None);
                                }
                                collecting_composite_type = false;
                                composite_type_parts.clear();
                                // НЕ очищаем last_attribute_name - он нужен для последующих свойств
                            }
                        }
                    }
                } else if !trimmed.is_empty() && (line.starts_with('\t') || line.starts_with(' ')) {
                    // Средняя строка многострочного типа с отступом
                    let type_part = trimmed.trim_end_matches(',').trim();
                    if !type_part.is_empty() && !type_part.contains(':') {
                        composite_type_parts.push(type_part.to_string());
                    }
                } else {
                    // Не строка типа - завершаем сбор с тем, что есть
                    
                    if !composite_type_parts.is_empty() {
                        let composite_type = composite_type_parts.join(", ");
                        if let Some(ref attr_name) = last_attribute_name {
                            self.create_attribute_with_type(attr_name, &composite_type, &current_section, &current_tabular_section, &mut current_structure, None, None);
                        }
                    } else if let Some(ref attr_name) = last_attribute_name {
                        self.create_attribute_with_type(attr_name, "Строка", &current_section, &current_tabular_section, &mut current_structure, None, None);
                    }
                    collecting_composite_type = false;
                    composite_type_parts.clear();
                    // НЕ очищаем last_attribute_name - он нужен для последующих свойств
                }
                i += 1;
                continue;
            }
            
            // Проверяем на начало объекта конфигурации
            if trimmed.starts_with("-") {
                let object_line = trimmed.trim_start_matches("-").trim();
                
                // Проверяем, что это объект конфигурации (ТОЛЬКО 2 части: Тип.Имя)
                let parts: Vec<&str> = object_line.split('.').collect();
                if parts.len() == 2 {
                    let type_str = self.clean_type_string(parts[0]);
                    if self.allowed_root_types.contains_key(&type_str) {
                        // Сохраняем предыдущий объект
                        if let Some((full_name, _name, obj_type)) = current_object.take() {
                            if let Some(structure) = current_structure.take() {
                                let mut contract = self.create_contract(&full_name, obj_type, structure, source_path, encoding);
                                // Добавляем комментарий из object_info
                                if let Some(comment) = object_info.get("комментарий") {
                                    contract.structure.comments = Some(comment.clone());
                                }
                                contracts.push(contract);
                            }
                        }
                        
                        // Начинаем новый объект
                        let object_type = self.allowed_root_types[&type_str].clone();
                        let name = self.clean_type_string(parts[1]);
                        current_object = Some((object_line.to_string(), name, object_type));
                        current_structure = Some(ObjectStructure {
                            attributes: Vec::new(),
                            tabular_sections: Vec::new(),
                            forms: Vec::new(),
                            templates: Vec::new(),
                            commands: Vec::new(),
                            comments: None,
                            dimensions: None,
                            resources: None,
                        });
                        object_info.clear();
                        current_section = None;
                        current_tabular_section = None;
                        last_attribute_name = None;
                        tracing::debug!("Found object: {}", object_line);
                        
                        // Собираем информацию об объекте
                        i += 1;
                        while i < lines.len() {
                            let info_line = lines[i].trim();
                            if info_line.starts_with("-") || info_line.starts_with("📌") {
                                break;
                            }
                            if let Some(colon_pos) = info_line.find(':') {
                                let key = self.clean_type_string(&info_line[..colon_pos]).to_lowercase();
                                let value = info_line[colon_pos+1..].trim().trim_matches('"').to_string();
                                object_info.insert(key, value);
                            }
                            i += 1;
                        }
                        continue;
                    }
                }
                
                // Handle nested elements (attributes, tabular sections) - moved outside of the main if
                if parts.len() >= 3 && current_object.is_some() {
                    // Это вложенный элемент с префиксом "-" (реквизит через путь)
                    // Проверяем Справочники.Организации.Реквизиты.КраткоеНаименование
                    if parts.len() >= 4 {
                        let element_type = self.clean_type_string(parts[2]).to_lowercase();
                        if element_type == "реквизиты" || element_type == "измерения" || element_type == "ресурсы" {
                            // Для регистров: различаем секции (Измерения, Ресурсы, Реквизиты)
                            let section_type = match element_type.as_str() {
                                "измерения" => "dimensions",
                                "ресурсы" => "resources", 
                                "реквизиты" => "attributes",
                                _ => "attributes"
                            };
                            current_section = Some(section_type.to_string());
                            let attr_name = self.clean_type_string(parts[3]);
                            // Создаем реквизит/измерение/ресурс, тип будет определен позже
                            last_attribute_name = Some(attr_name.clone());
                            tracing::debug!("Found {} element: {} (section: {})", element_type, attr_name, section_type);
                        } else if element_type == "табличныечасти" {
                            if parts.len() == 4 {
                                // Табличная часть: Документы.ЗаказНаряды.ТабличныеЧасти.Работы
                                current_section = Some("tabular".to_string());
                                let ts_name = self.clean_type_string(parts[3]);
                                if let Some(ref mut structure) = current_structure {
                                    let ts = TabularSection {
                                        name: ts_name.clone(),
                                        attributes: Vec::new(),
                                        indexing: None,
                                    };
                                    structure.tabular_sections.push(ts);
                                    current_tabular_section = Some(ts_name.clone());
                                }
                            } else if parts.len() >= 6 && parts[4] == "Реквизиты" {
                                // Реквизит табличной части: Документы.ЗаказНаряды.ТабличныеЧасти.Работы.Реквизиты.ВидРаботы
                                let ts_name = self.clean_type_string(parts[3]);
                                let attr_name = self.clean_type_string(parts[5]);
                                current_tabular_section = Some(ts_name.clone());
                                last_attribute_name = Some(attr_name.clone());
                            }
                        }
                    }
                }
            }
            // Не обрабатываем эмодзи - их нет в реальном формате
            // Обработка свойств объекта и реквизитов
            else if current_object.is_some() && trimmed.contains(":") && !trimmed.starts_with("-") {
                if let Some((key, value)) = trimmed.split_once(':') {
                    let key = key.trim();
                    let value = value.trim().trim_matches('"');
                    
                    
                    // Если у нас есть имя последнего атрибута и это строка с "Имя:"
                    if key == "Имя" && last_attribute_name.is_some() {
                        // Проверяем, что значение совпадает с именем атрибута
                        if let Some(ref attr_name) = last_attribute_name {
                            if value == attr_name {
                                // Начинаем собирать свойства для этого атрибута
                                tracing::debug!("Collecting properties for attribute: {}", attr_name);
                            }
                        }
                    }
                    // Если это строка с типом и у нас есть имя атрибута
                    else if key == "Тип" && last_attribute_name.is_some() {
                        if let Some(ref attr_name) = last_attribute_name {
                            if value.is_empty() {
                                // Пустое значение типа - начинаем собирать составной тип
                                collecting_composite_type = true;
                                composite_type_parts.clear();
                                tracing::debug!("Starting composite type collection for attribute: {}", attr_name);
                            } else {
                                // Простой тип - сразу создаем атрибут
                                self.create_attribute_with_type(attr_name, value, &current_section, &current_tabular_section, &mut current_structure, None, None);
                                // НЕ очищаем last_attribute_name здесь - он нужен для последующих свойств
                            }
                        }
                    }
                    // Обработка свойств атрибута (Индексирование, ПроверкаЗаполнения и т.д.)
                    else if key == "Индексирование" && last_attribute_name.is_some() {
                        self.update_last_attribute_indexing(&last_attribute_name, value, &current_section, &current_tabular_section, &mut current_structure);
                    }
                    else if key == "ПроверкаЗаполнения" && last_attribute_name.is_some() {
                        self.update_last_attribute_fill_checking(&last_attribute_name, value, &current_section, &current_tabular_section, &mut current_structure);
                    }
                    // Сохраняем общие свойства объекта
                    else if key != "Тип" && current_section.is_none() {
                        object_info.insert(key.to_string(), value.to_string());
                    }
                }
            }
            // Обработка табличных частей
            else if current_object.is_some() && (trimmed.ends_with("Табличные части:") || trimmed == "Табличная часть") {
                current_section = Some("tabular".to_string());
            }
            // Обработка названия табличной части  
            else if current_section == Some("tabular".to_string()) && trimmed.contains("Имя:") {
                if let Some(colon_pos) = trimmed.find(':') {
                    let ts_name = trimmed[colon_pos+1..].trim().trim_matches('"').to_string();
                    if !ts_name.is_empty() {
                        if let Some(ref mut structure) = current_structure {
                            let ts = TabularSection {
                                name: ts_name.clone(),
                                attributes: Vec::new(),
                                indexing: None,
                            };
                            structure.tabular_sections.push(ts);
                            current_tabular_section = Some(ts_name.clone());
                            tracing::debug!("Added tabular section: {}", ts_name);
                        }
                    }
                }
            }
            
            i += 1;
        }
        
        // Сохраняем последний объект
        if let Some((full_name, _name, obj_type)) = current_object {
            if let Some(structure) = current_structure {
                let mut contract = self.create_contract(&full_name, obj_type, structure, source_path, encoding);
                if let Some(comment) = object_info.get("комментарий") {
                    contract.structure.comments = Some(comment.clone());
                }
                contracts.push(contract);
            }
        }
        
        tracing::info!("Parsed {} metadata objects from real format", contracts.len());
        
        tracing::debug!("Successfully parsed {} metadata objects", contracts.len());
        
        Ok(contracts)
    }
    
    /// Очищает строку типа от лишних символов
    fn clean_type_string(&self, s: &str) -> String {
        s.chars()
            .filter(|c| c.is_alphanumeric() || *c == '_' || *c == ' ')
            .collect::<String>()
            .trim()
            .to_string()
    }
    
    /// Парсит Python формат (совместимость)
    fn parse_python_format(&self, _content: &str, _source_path: &Path, _encoding: &str) -> Result<Vec<MetadataContract>> {
        // TODO: Реализовать парсинг Python формата
        tracing::warn!("Python format parsing not yet implemented");
        Ok(vec![])
    }
    
    /// Парсит упрощенный формат (старая реализация)
    fn parse_simplified_format(&self, content: &str, source_path: &Path, encoding: &str) -> Result<Vec<MetadataContract>> {
        let mut contracts = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut current_object: Option<(String, ObjectType, usize)> = None;
        let mut current_structure: Option<ObjectStructure> = None;
        
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            if trimmed.is_empty() {
                continue;
            }
            
            // Проверяем объекты конфигурации
            if !trimmed.starts_with(" ") && trimmed.contains(".") {
                let parts: Vec<&str> = trimmed.split('.').collect();
                
                if parts.len() == 2 && self.allowed_root_types.contains_key(parts[0]) {
                    // Сохраняем предыдущий объект
                    if let (Some((obj_name, obj_type, _)), Some(structure)) = (current_object.take(), current_structure.take()) {
                        let contract = self.create_contract(&obj_name, obj_type, structure, source_path, encoding);
                        contracts.push(contract);
                    }
                    
                    // Начинаем новый объект
                    let object_type = self.allowed_root_types[parts[0]].clone();
                    let full_name = trimmed.to_string();
                    tracing::debug!("Found object: {}", full_name);
                    
                    current_object = Some((full_name, object_type, i));
                    current_structure = Some(ObjectStructure {
                        attributes: Vec::new(),
                        tabular_sections: Vec::new(),
                        forms: Vec::new(),
                        templates: Vec::new(),
                        commands: Vec::new(),
                        comments: None,
                        dimensions: None,
                        resources: None,
                    });
                }
            }
            // Обработка элементов внутри объекта
            else if let Some((_, _, _)) = current_object {
                // Проверяем отступ строки
                let indent_level = line.len() - line.trim_start().len();
                
                if indent_level == 2 {
                    // Реквизит основного объекта или табличная часть
                    if let Some(ref mut structure) = current_structure {
                        if trimmed.ends_with(":") && !trimmed.contains("(") {
                            // Это табличная часть
                            let ts_name = trimmed.trim_end_matches(':');
                            let ts = TabularSection {
                                name: ts_name.to_string(),
                                attributes: Vec::new(),
                                indexing: None,
                            };
                            structure.tabular_sections.push(ts);
                            tracing::debug!("Added tabular section: {}", ts_name);
                        } else if let Some(paren_pos) = trimmed.find('(') {
                            // Это реквизит с типом
                            let attr_name = trimmed[..paren_pos].trim();
                            let type_part = trimmed[paren_pos+1..].trim_end_matches(')');
                            
                            let attr = AttributeInfo {
                                name: attr_name.to_string(),
                                data_type: type_part.to_string(),
                                length: None,
                                precision: None,
                                attribute_use: AttributeUse::ForFolderAndItem,
                                indexing: AttributeIndexing::DontIndex,
                                fill_checking: FillChecking::DontCheck,
                            };
                            structure.attributes.push(attr);
                            tracing::debug!("Added attribute: {} ({})", attr_name, type_part);
                        }
                    }
                } else if indent_level == 4 {
                    // Реквизит табличной части
                    if let Some(ref mut structure) = current_structure {
                        if let Some(ts) = structure.tabular_sections.last_mut() {
                            if let Some(paren_pos) = trimmed.find('(') {
                                let attr_name = trimmed[..paren_pos].trim();
                                let type_part = trimmed[paren_pos+1..].trim_end_matches(')');
                                
                                let attr = AttributeInfo {
                                    name: attr_name.to_string(),
                                    data_type: type_part.to_string(),
                                    length: None,
                                    precision: None,
                                    attribute_use: AttributeUse::ForFolderAndItem,
                                    indexing: AttributeIndexing::DontIndex,
                                    fill_checking: FillChecking::DontCheck,
                                };
                                ts.attributes.push(attr);
                                tracing::debug!("Added tabular attribute {} to {}", attr_name, ts.name);
                            }
                        }
                    }
                }
            }
        }
        
        // Сохраняем последний объект
        if let (Some((obj_name, obj_type, _)), Some(structure)) = (current_object, current_structure) {
            let contract = self.create_contract(&obj_name, obj_type, structure, source_path, encoding);
            contracts.push(contract);
        }
        
        tracing::info!("Parsed {} metadata objects from simplified format", contracts.len());
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
        let parts: Vec<&str> = full_object_name.split('.').collect();
        let name = if parts.len() >= 2 { parts[1] } else { full_object_name };
        
        MetadataContract {
            metadata_type: "Metadata".to_string(),
            name: name.to_string(),
            object_type,
            structure,
            search_keywords: self.generate_search_keywords(name),
            generation_metadata: GenerationMetadata {
                timestamp: Utc::now().to_rfc3339(),
                generator_version: "1.0.0".to_string(),
                source_file: source_path.display().to_string(),
                encoding_used: encoding.to_string(),
            },
        }
    }
    
    /// Генерирует ключевые слова для поиска
    fn generate_search_keywords(&self, object_name: &str) -> Vec<String> {
        let mut keywords = vec![object_name.to_string()];
        
        // Добавляем части CamelCase
        let parts = self.split_camel_case(object_name);
        keywords.extend(parts);
        
        keywords
    }
    
    /// Разбивает CamelCase строку на части
    fn split_camel_case(&self, s: &str) -> Vec<String> {
        let re = Regex::new(r"[А-ЯA-Z][а-яa-z]*").unwrap();
        re.find_iter(s)
            .map(|m| m.as_str().to_string())
            .collect()
    }
    
    /// Находит файл отчета конфигурации в директории
    pub fn find_configuration_report<P: AsRef<Path>>(config_dir: P) -> Result<Option<PathBuf>> {
        let config_dir = config_dir.as_ref();
        
        // Список возможных имен файлов отчета
        let possible_names = vec![
            "config_report.txt",
            "ОтчетПоКонфигурации.txt",
            "ConfigurationReport.txt",
            "СтруктураХранения.txt",
            "StructureStorage.txt",
        ];
        
        // Проверяем стандартные имена
        for name in &possible_names {
            let report_path = config_dir.join(name);
            if report_path.exists() {
                tracing::info!("Found configuration report: {}", report_path.display());
                return Ok(Some(report_path));
            }
        }
        
        // Ищем любой .txt файл, содержащий характерные строки
        if let Ok(entries) = std::fs::read_dir(config_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("txt") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if content.contains("Справочник.") || content.contains("Документ.") {
                            tracing::info!("Found potential configuration report: {}", path.display());
                            return Ok(Some(path));
                        }
                    }
                }
            }
        }
        
        tracing::warn!("No configuration report found in: {}", config_dir.display());
        Ok(None)
    }
    
    /// Парсит отчет и записывает в гибридное хранилище
    pub fn parse_to_hybrid_storage<P: AsRef<Path>>(
        &self, 
        report_path: P,
        storage: &mut HybridDocumentationStorage
    ) -> Result<()> {
        let contracts = self.parse_report(report_path)?;
        
        for contract in contracts {
            let type_def = self.convert_to_type_definition(contract);
            storage.add_configuration_type(type_def)?;
        }
        
        Ok(())
    }
    
    /// Преобразует MetadataContract в TypeDefinition
    fn convert_to_type_definition(&self, contract: MetadataContract) -> TypeDefinition {
        let mut methods = HashMap::new();
        let mut properties = HashMap::new();
        
        // Добавляем реквизиты как свойства
        for attr in &contract.structure.attributes {
            properties.insert(attr.name.clone(), PropertyDefinition {
                name: attr.name.clone(),
                english_name: None,
                description: format!("Реквизит типа {}", attr.data_type),
                property_type: attr.data_type.clone(),
                readonly: false,
                availability: vec!["Сервер".to_string(), "Клиент".to_string()],
                deprecated: false,
            });
        }
        
        // Добавляем табличные части как свойства-коллекции
        for ts in &contract.structure.tabular_sections {
            properties.insert(ts.name.clone(), PropertyDefinition {
                name: ts.name.clone(),
                english_name: None,
                description: format!("Табличная часть"),
                property_type: "ТабличнаяЧасть".to_string(),
                readonly: false,
                availability: vec!["Сервер".to_string(), "Клиент".to_string()],
                deprecated: false,
            });
        }
        
        // Добавляем стандартные методы для объектов конфигурации
        match contract.object_type {
            ObjectType::Directory => {
                methods.insert("НайтиПоКоду".to_string(), MethodDefinition {
                    name: "НайтиПоКоду".to_string(),
                    english_name: Some("FindByCode".to_string()),
                    description: "Найти элемент справочника по коду".to_string(),
                    parameters: vec![ParameterDefinition {
                        name: "Код".to_string(),
                        parameter_type: "Строка".to_string(),
                        required: true,
                        description: "Код элемента".to_string(),
                        default_value: None,
                    }],
                    return_type: Some(format!("СправочникСсылка.{}", contract.name)),
                    is_function: true,
                    availability: vec!["Сервер".to_string()],
                    examples: vec![],
                    deprecated: false,
                });
            },
            ObjectType::Document => {
                methods.insert("Провести".to_string(), MethodDefinition {
                    name: "Провести".to_string(),
                    english_name: Some("Post".to_string()),
                    description: "Провести документ".to_string(),
                    parameters: vec![],
                    return_type: Some("Булево".to_string()),
                    is_function: true,
                    availability: vec!["Сервер".to_string()],
                    examples: vec![],
                    deprecated: false,
                });
            },
            _ => {}
        }
        
        TypeDefinition {
            id: format!("{}.{}", contract.object_type, contract.name),
            name: contract.name.clone(),
            english_name: None,
            category: TypeCategory::Configuration,
            description: format!("{} конфигурации", self.get_object_type_description(&contract.object_type)),
            methods,
            properties,
            constructors: vec![],
            parent_types: vec![],
            interfaces: vec![],
            availability: vec!["Сервер".to_string(), "Клиент".to_string()],
        }
    }
    
    /// Получить описание типа объекта
    fn get_object_type_description(&self, object_type: &ObjectType) -> &'static str {
        match object_type {
            ObjectType::Directory => "Справочник",
            ObjectType::Document => "Документ", 
            ObjectType::InformationRegister => "Регистр сведений",
            ObjectType::AccumulationRegister => "Регистр накопления",
            ObjectType::AccountingRegister => "Регистр бухгалтерии",
            ObjectType::Register => "Регистр",
            ObjectType::Enumeration => "Перечисление",
            ObjectType::ChartOfCharacteristicTypes => "План видов характеристик",
            ObjectType::ChartOfAccounts => "План счетов",
            ObjectType::ChartOfCalculationTypes => "План видов расчета",
            ObjectType::ExchangePlan => "План обмена",
            ObjectType::Report => "Отчет",
            ObjectType::DataProcessor => "Обработка",
            ObjectType::DocumentJournal => "Журнал документов",
            ObjectType::Sequence => "Последовательность",
            ObjectType::Task => "Задача",
            ObjectType::Constant => "Константа",
            ObjectType::CommonModule => "Общий модуль",
            ObjectType::CommonAttribute => "Общий реквизит",
            ObjectType::BusinessProcess => "Бизнес-процесс",
            ObjectType::WebService => "Web-сервис",
            ObjectType::HTTPService => "HTTP-сервис",
            ObjectType::ScheduledJob => "Регламентное задание",
            ObjectType::FunctionalOption => "Функциональная опция",
            ObjectType::DefinedType => "Определяемый тип",
            ObjectType::SettingsStorage => "Хранилище настроек",
            ObjectType::FilterCriterion => "Критерий отбора",
            ObjectType::Subsystem => "Подсистема",
            ObjectType::Role => "Роль",
            ObjectType::ExternalDataSource => "Внешний источник данных",
            ObjectType::Configuration => "Конфигурация",
            ObjectType::Language => "Язык",
            ObjectType::CommonForm => "Общая форма",
            ObjectType::CommonCommand => "Общая команда",
            ObjectType::CommonPicture => "Общая картинка",
            ObjectType::CommonTemplate => "Общий макет",
            ObjectType::XDTOPackage => "XDTO-пакет",
            ObjectType::Style => "Стиль",
            ObjectType::StyleItem => "Элемент стиля",
        }
    }
    
    /// Создает атрибут с указанным типом и добавляет в соответствующую секцию
    fn create_attribute_with_type(
        &self,
        attr_name: &str,
        data_type: &str,
        current_section: &Option<String>,
        current_tabular_section: &Option<String>,
        current_structure: &mut Option<ObjectStructure>,
        indexing: Option<AttributeIndexing>,
        fill_checking: Option<FillChecking>
    ) {
        self.create_attribute_with_properties(
            attr_name, 
            data_type, 
            current_section, 
            current_tabular_section, 
            current_structure,
            indexing.unwrap_or(AttributeIndexing::DontIndex), // По умолчанию без индексирования
            fill_checking.unwrap_or(FillChecking::DontCheck)  // По умолчанию без проверки заполнения
        );
    }

    /// Создает атрибут с полными свойствами и добавляет в соответствующую секцию  
    fn create_attribute_with_properties(
        &self,
        attr_name: &str,
        data_type: &str,
        current_section: &Option<String>,
        current_tabular_section: &Option<String>,
        current_structure: &mut Option<ObjectStructure>,
        indexing: AttributeIndexing,
        fill_checking: FillChecking
    ) {
        let final_type = if data_type.is_empty() { "Строка" } else { data_type };
        
        // Извлекаем длину и точность из типа данных
        let (length, precision) = self.extract_type_constraints(final_type);
        
        if let Some(ref section) = current_section {
            if section == "attributes" {
                if let Some(ref mut structure) = current_structure {
                    let attr = AttributeInfo {
                        name: attr_name.to_string(),
                        data_type: final_type.to_string(),
                        length,
                        precision,
                        attribute_use: AttributeUse::ForFolderAndItem,
                        indexing,
                        fill_checking,
                    };
                    structure.attributes.push(attr);
                    tracing::debug!("Added attribute {} with type {} (length: {:?}, precision: {:?})", 
                        attr_name, final_type, length, precision);
                }
            } else if section == "dimensions" {
                // Добавляем в секцию измерений для регистров
                if let Some(ref mut structure) = current_structure {
                    if structure.dimensions.is_none() {
                        structure.dimensions = Some(Vec::new());
                    }
                    let attr = AttributeInfo {
                        name: attr_name.to_string(),
                        data_type: final_type.to_string(),
                        length,
                        precision,
                        attribute_use: AttributeUse::ForFolderAndItem,
                        indexing,
                        fill_checking,
                    };
                    structure.dimensions.as_mut().unwrap().push(attr);
                    tracing::debug!("Added dimension {} with type {} (length: {:?}, precision: {:?})", 
                        attr_name, final_type, length, precision);
                }
            } else if section == "resources" {
                // Добавляем в секцию ресурсов для регистров
                if let Some(ref mut structure) = current_structure {
                    if structure.resources.is_none() {
                        structure.resources = Some(Vec::new());
                    }
                    let attr = AttributeInfo {
                        name: attr_name.to_string(),
                        data_type: final_type.to_string(),
                        length,
                        precision,
                        attribute_use: AttributeUse::ForFolderAndItem,
                        indexing,
                        fill_checking,
                    };
                    structure.resources.as_mut().unwrap().push(attr);
                    tracing::debug!("Added resource {} with type {} (length: {:?}, precision: {:?})", 
                        attr_name, final_type, length, precision);
                }
            } else if section == "tabular" {
                if let Some(ref ts_name) = current_tabular_section {
                    if let Some(ref mut structure) = current_structure {
                        if let Some(ts) = structure.tabular_sections.iter_mut()
                            .find(|t| t.name == *ts_name) {
                            let attr = AttributeInfo {
                                name: attr_name.to_string(),
                                data_type: final_type.to_string(),
                                length,
                                precision,
                                attribute_use: AttributeUse::ForFolderAndItem,
                                indexing: AttributeIndexing::DontIndex,
                                fill_checking: FillChecking::DontCheck,
                            };
                            ts.attributes.push(attr);
                            tracing::debug!("Added tabular attribute {} to {} with type {} (length: {:?}, precision: {:?})", 
                                attr_name, ts_name, final_type, length, precision);
                        }
                    }
                }
            }
        }
    }
    
    /// Парсит значение индексирования атрибута
    fn parse_indexing(&self, value: &str) -> AttributeIndexing {
        match value {
            "Индексировать" => AttributeIndexing::Index,
            "ИндексироватьСДопУпорядочиванием" => AttributeIndexing::IndexWithOrdering,
            "ИндексироватьСДополнительнымПорядком" => AttributeIndexing::IndexWithAdditionalOrder,
            "НеИндексировать" => AttributeIndexing::DontIndex,
            _ => {
                tracing::warn!("Unknown indexing value: {}, defaulting to DontIndex", value);
                AttributeIndexing::DontIndex
            }
        }
    }

    /// Парсит значение проверки заполнения атрибута
    fn parse_fill_checking(&self, value: &str) -> FillChecking {
        match value {
            "ВыдаватьОшибку" => FillChecking::ShowError,
            "НеПроверять" => FillChecking::DontCheck,
            _ => {
                tracing::warn!("Unknown fill checking value: {}, defaulting to DontCheck", value);
                FillChecking::DontCheck
            }
        }
    }

    fn update_last_attribute_indexing(
        &self,
        last_attr_name: &Option<String>,
        value: &str,
        current_section: &Option<String>,
        current_tabular_section: &Option<String>,
        current_structure: &mut Option<ObjectStructure>
    ) {
        if let (Some(attr_name), Some(ref mut structure)) = (last_attr_name, current_structure) {
            let indexing = self.parse_indexing(value);
            
            // Найти и обновить атрибут в соответствующей секции
            if let Some(ref ts_name) = current_tabular_section {
                // Табличная часть
                for ts in &mut structure.tabular_sections {
                    if ts.name == *ts_name {
                        for attr in &mut ts.attributes {
                            if attr.name == *attr_name {
                                attr.indexing = indexing.clone();
                                return;
                            }
                        }
                    }
                }
            } else if let Some(ref section) = current_section {
                // Регистр: измерения, ресурсы, реквизиты
                match section.as_str() {
                    "dimensions" => {
                        if let Some(ref mut dims) = structure.dimensions {
                            for attr in dims {
                                if attr.name == *attr_name {
                                    attr.indexing = indexing.clone();
                                    return;
                                }
                            }
                        }
                    },
                    "resources" => {
                        if let Some(ref mut res) = structure.resources {
                            for attr in res {
                                if attr.name == *attr_name {
                                    attr.indexing = indexing.clone();
                                    return;
                                }
                            }
                        }
                    },
                    _ => {
                        // Обычные атрибуты
                        for attr in &mut structure.attributes {
                            if attr.name == *attr_name {
                                attr.indexing = indexing.clone();
                                return;
                            }
                        }
                    }
                }
            } else {
                // Обычные атрибуты объекта
                for attr in &mut structure.attributes {
                    if attr.name == *attr_name {
                        attr.indexing = indexing.clone();
                        return;
                    }
                }
            }
        }
    }

    fn update_last_attribute_fill_checking(
        &self,
        last_attr_name: &Option<String>,
        value: &str,
        current_section: &Option<String>,
        current_tabular_section: &Option<String>,
        current_structure: &mut Option<ObjectStructure>
    ) {
        if let (Some(attr_name), Some(ref mut structure)) = (last_attr_name, current_structure) {
            let fill_checking = self.parse_fill_checking(value);
            
            // Найти и обновить атрибут в соответствующей секции
            if let Some(ref ts_name) = current_tabular_section {
                // Табличная часть
                for ts in &mut structure.tabular_sections {
                    if ts.name == *ts_name {
                        for attr in &mut ts.attributes {
                            if attr.name == *attr_name {
                                attr.fill_checking = fill_checking;
                                return;
                            }
                        }
                    }
                }
            } else if let Some(ref section) = current_section {
                // Регистр: измерения, ресурсы, реквизиты
                match section.as_str() {
                    "dimensions" => {
                        if let Some(ref mut dims) = structure.dimensions {
                            for attr in dims {
                                if attr.name == *attr_name {
                                    attr.fill_checking = fill_checking;
                                    return;
                                }
                            }
                        }
                    },
                    "resources" => {
                        if let Some(ref mut res) = structure.resources {
                            for attr in res {
                                if attr.name == *attr_name {
                                    attr.fill_checking = fill_checking;
                                    return;
                                }
                            }
                        }
                    },
                    _ => {
                        // Обычные атрибуты
                        for attr in &mut structure.attributes {
                            if attr.name == *attr_name {
                                attr.fill_checking = fill_checking;
                                return;
                            }
                        }
                    }
                }
            } else {
                // Обычные атрибуты объекта
                for attr in &mut structure.attributes {
                    if attr.name == *attr_name {
                        attr.fill_checking = fill_checking;
                        return;
                    }
                }
            }
        }
    }

    /// Извлекает ограничения длины и точности из типа данных
    fn extract_type_constraints(&self, data_type: &str) -> (Option<u32>, Option<u32>) {
        use regex::Regex;
        
        // Regex for extracting constraints from types like "Строка(10, Переменная)" or "Число(15, 2)"
        let string_regex = Regex::new(r"Строка\((\d+)(?:,\s*(\w+))?\)").unwrap();
        let number_regex = Regex::new(r"Число\((\d+)(?:,\s*(\d+))?\)").unwrap();
        
        // Check for string type constraints
        if let Some(captures) = string_regex.captures(data_type) {
            if let Some(length_str) = captures.get(1) {
                if let Ok(length) = length_str.as_str().parse::<u32>() {
                    return (Some(length), None);
                }
            }
        }
        
        // Check for number type constraints  
        if let Some(captures) = number_regex.captures(data_type) {
            let length = captures.get(1)
                .and_then(|m| m.as_str().parse::<u32>().ok());
            let precision = captures.get(2)
                .and_then(|m| m.as_str().parse::<u32>().ok());
            return (length, precision);
        }
        
        // Check composite types - extract constraints from each part
        if data_type.contains(',') {
            let parts: Vec<&str> = data_type.split(',').collect();
            for part in parts {
                let part = part.trim();
                let (length, precision) = self.extract_type_constraints(part);
                if length.is_some() || precision.is_some() {
                    return (length, precision);
                }
            }
        }
        
        (None, None)
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
    fn test_parser_creation() {
        let parser = MetadataReportParser::new().unwrap();
        assert!(!parser.allowed_root_types.is_empty());
        assert!(parser.allowed_root_types.contains_key("Справочник"));
        assert!(parser.allowed_root_types.contains_key("Документ"));
    }
}