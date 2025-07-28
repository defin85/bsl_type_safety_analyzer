/*!
# Form XML Parser

Парсер XML файлов форм 1С.
Портирован с Python проекта onec-contract-generator на Rust.

Основные возможности:
- Парсинг XML файлов форм через quick-xml
- Извлечение элементов управления и их свойств
- Генерация контрактов форм с типобезопасными структурами
- Поиск всех форм в конфигурации

## Использование

```rust
let parser = FormXmlParser::new();
let forms = parser.generate_all_contracts("./config")?;
let form_contract = parser.parse_form_xml("Form.xml")?;
```

## Важно

Парсер работает с XML файлами форм 1С, которые обычно находятся
в структуре: ConfigDir/ObjectType/ObjectName/Forms/FormName/Form.xml
*/

use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use quick_xml::{Reader, events::Event};
use walkdir::WalkDir;
use anyhow::{Context, Result};
use chrono::Utc;
use crate::docs_integration::hybrid_storage::HybridDocumentationStorage;

/// Контракт формы 1С (замена Python FormContract)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormContract {
    pub metadata_type: String, // Всегда "Form"
    pub name: String,
    pub synonym: Option<String>,
    pub comment: Option<String>,
    pub form_type: FormType,
    pub object_name: Option<String>, // К какому объекту относится форма
    pub structure: FormStructure,
    pub generation_metadata: FormGenerationMetadata,
}

/// Типы форм 1С
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FormType {
    ObjectForm,        // ФормаОбъекта
    ListForm,          // ФормаСписка
    ChoiceForm,        // ФормаВыбора
    ItemForm,          // ФормаЭлемента
    SettingsForm,      // ФормаНастроек
    CommonForm,        // ОбщаяФорма
    ReportForm,        // ФормаОтчета
    DataProcessorForm, // ФормаОбработки
    Unknown(String),   // Неизвестный тип
}

/// Структура формы
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormStructure {
    pub elements_count: usize,
    pub attributes_count: usize,
    pub elements: Vec<FormElement>,
    pub attributes: Vec<FormAttribute>,
    pub commands: Vec<FormCommand>,
}

/// Элемент управления формы
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormElement {
    pub name: String,
    pub element_type: FormElementType,
    pub title: Option<String>,
    pub data_path: Option<String>,
    pub properties: HashMap<String, serde_json::Value>,
    pub events: Vec<String>,
    pub parent: Option<String>,
    pub children: Vec<String>,
}

/// Типы элементов управления
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FormElementType {
    InputField,        // Поле
    Label,            // Надпись
    Button,           // Кнопка
    Picture,          // Картинка
    Table,            // Таблица
    Group,            // Группа
    Page,             // Страница
    CheckBox,         // Флажок
    RadioButton,      // Переключатель
    ProgressBar,      // ИндикаторПроцесса
    Calendar,         // Календарь
    HtmlDocument,     // HTMLДокумент
    SpreadsheetDocument, // ТабличныйДокумент
    TextDocument,     // ТекстовыйДокумент
    Chart,            // Диаграмма
    Decoration,       // Декорация
    CommandBar,       // КоманднаяПанель
    ContextMenu,      // КонтекстноеМеню
    FormattedDocument, // ФорматированныйДокумент
    Unknown(String),  // Неизвестный тип
}

/// Реквизит формы
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormAttribute {
    pub name: String,
    pub data_type: String,
    pub title: Option<String>,
    pub save_data: bool,
    pub properties: HashMap<String, serde_json::Value>,
}

/// Команда формы
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormCommand {
    pub name: String,
    pub title: Option<String>,
    pub action: Option<String>,
    pub representation: CommandRepresentation,
    pub properties: HashMap<String, serde_json::Value>,
}

/// Представление команды
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandRepresentation {
    Auto,        // Авто
    Text,        // Текст
    Picture,     // Картинка
    TextPicture, // ТекстКартинка
}

/// Метаданные генерации формы
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormGenerationMetadata {
    pub timestamp: String,
    pub generator_version: String,
    pub source_file: String,
    pub form_path: String,
}

/// Парсер XML форм (замена Python FormGenerator)
pub struct FormXmlParser {
    #[allow(dead_code)]
    xmlns_patterns: Vec<String>,
}

impl FormXmlParser {
    /// Создает новый парсер XML форм
    pub fn new() -> Self {
        Self {
            xmlns_patterns: vec![
                "http://v8.1c.ru/8.3/xcf/logform".to_string(),
                "http://v8.1c.ru/8.2/managed-application/logform".to_string(),
                "http://v8.1c.ru/8.1/data/ui".to_string(),
            ],
        }
    }
    
    /// Находит все XML файлы форм в конфигурации (замена Python find_form_files)
    pub fn find_form_files<P: AsRef<Path>>(&self, config_dir: P) -> Result<Vec<PathBuf>> {
        let config_dir = config_dir.as_ref();
        tracing::info!("Searching for form XML files in: {}", config_dir.display());
        
        let mut form_files = Vec::new();
        
        for entry in WalkDir::new(config_dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            
            // Ищем файлы Form.xml в структуре конфигурации
            if path.file_name() == Some(OsStr::new("Form.xml")) {
                // Проверяем, что это действительно форма 1С по структуре пути
                if self.is_valid_form_path(path) {
                    form_files.push(path.to_path_buf());
                }
            }
        }
        
        // Сортируем для консистентности
        form_files.sort();
        
        tracing::info!("Found {} form XML files", form_files.len());
        Ok(form_files)
    }
    
    /// Парсит один XML файл формы (обновлено для реальных форм 1С)
    pub fn parse_form_xml<P: AsRef<Path>>(&self, xml_path: P) -> Result<FormContract> {
        let xml_path = xml_path.as_ref();
        tracing::debug!("Parsing real 1C form XML: {}", xml_path.display());
        
        let content = std::fs::read_to_string(xml_path)
            .with_context(|| format!("Failed to read form XML: {}", xml_path.display()))?;
        
        let mut reader = Reader::from_str(&content);
        reader.trim_text(true);
        
        let mut form_contract = FormContract {
            metadata_type: "Form".to_string(),
            name: self.extract_form_name_from_path(xml_path),
            synonym: None,
            comment: None,
            form_type: FormType::Unknown("Unknown".to_string()),
            object_name: self.extract_object_name_from_path(xml_path),
            structure: FormStructure {
                elements_count: 0,
                attributes_count: 0,
                elements: Vec::new(),
                attributes: Vec::new(),
                commands: Vec::new(),
            },
            generation_metadata: FormGenerationMetadata {
                timestamp: Utc::now().to_rfc3339(),
                generator_version: env!("CARGO_PKG_VERSION").to_string(),
                source_file: xml_path.display().to_string(),
                form_path: xml_path.parent().unwrap_or(xml_path).display().to_string(),
            },
        };
        
        let mut buf = Vec::new();
        let mut current_element: Option<FormElement> = None;
        let mut in_child_items = false;
        let mut current_tag_stack: Vec<String> = Vec::new();
        let mut current_data_path: Option<String> = None;
        
        // Парсим XML с реальной структурой форм 1С
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let tag_name = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                    current_tag_stack.push(tag_name.clone());
                    
                    match tag_name.as_str() {
                        // Корневой элемент формы
                        "Form" => {
                            // Извлекаем атрибуты корневого элемента
                            self.extract_real_form_attributes(&mut form_contract, e)?;
                        }
                        
                        // Элементы формы в реальной структуре
                        "ChildItems" => {
                            in_child_items = true;
                        }
                        
                        // Типы элементов управления в реальных формах 1С
                        "InputField" | "Table" | "RadioButtonField" | "CheckBoxField" | 
                        "ButtonField" | "LabelField" | "PictureField" | "SpreadsheetDocumentField" |
                        "TextDocumentField" | "FormattedDocumentField" | "Pages" | "Page" |
                        "Group" | "Decoration" | "CommandBar" | "Button" => {
                            if in_child_items || current_tag_stack.contains(&"ChildItems".to_string()) {
                                current_element = Some(self.start_real_form_element(&tag_name, e)?);
                            }
                        }
                        
                        // События
                        "Events" => {
                            // Контекст для обработки событий
                        }
                        
                        "Event" => {
                            // Обрабатываем конкретное событие
                            if let Some(ref mut element) = current_element {
                                self.add_event_to_element(element, e)?;
                            }
                        }
                        
                        _ => {}
                    }
                }
                
                Ok(Event::Text(ref e)) => {
                    let text = e.unescape()?.trim().to_string();
                    if !text.is_empty() {
                        // Обрабатываем текстовое содержимое
                        if let Some(current_tag) = current_tag_stack.last() {
                            match current_tag.as_str() {
                                "DataPath" => {
                                    current_data_path = Some(text.clone());
                                    if let Some(ref mut element) = current_element {
                                        element.data_path = Some(text);
                                    }
                                }
                                "Title" => {
                                    if let Some(ref mut element) = current_element {
                                        element.title = Some(text);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                
                Ok(Event::End(ref e)) => {
                    let tag_name = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                    current_tag_stack.pop();
                    
                    match tag_name.as_str() {
                        "ChildItems" => {
                            in_child_items = false;
                        }
                        
                        // Завершение элементов управления
                        "InputField" | "Table" | "RadioButtonField" | "CheckBoxField" | 
                        "ButtonField" | "LabelField" | "PictureField" | "SpreadsheetDocumentField" |
                        "TextDocumentField" | "FormattedDocumentField" | "Pages" | "Page" |
                        "Group" | "Decoration" | "CommandBar" | "Button" => {
                            if let Some(element) = current_element.take() {
                                form_contract.structure.elements.push(element);
                            }
                        }
                        
                        _ => {}
                    }
                }
                
                Ok(Event::Eof) => break,
                Err(e) => {
                    tracing::warn!("XML parsing error in {}: {}", xml_path.display(), e);
                    break;
                }
                _ => {}
            }
            
            buf.clear();
        }
        
        // Обновляем счетчики
        form_contract.structure.elements_count = form_contract.structure.elements.len();
        form_contract.structure.attributes_count = form_contract.structure.attributes.len();
        
        // Определяем тип формы по имени и структуре
        form_contract.form_type = self.determine_form_type(&form_contract);
        
        tracing::debug!("Parsed {} elements from real form", form_contract.structure.elements_count);
        Ok(form_contract)
    }
    
    /// Генерирует контракты для всех форм (замена Python generate)
    pub fn generate_all_contracts<P: AsRef<Path>>(&self, config_dir: P) -> Result<Vec<FormContract>> {
        let form_files = self.find_form_files(config_dir)?;
        let mut contracts = Vec::new();
        
        for form_file in form_files {
            match self.parse_form_xml(&form_file) {
                Ok(contract) => contracts.push(contract),
                Err(e) => {
                    tracing::warn!("Failed to parse form {}: {}", form_file.display(), e);
                }
            }
        }
        
        tracing::info!("Generated {} form contracts", contracts.len());
        Ok(contracts)
    }
    
    /// Проверяет, является ли путь валидным путем к форме 1С
    fn is_valid_form_path(&self, path: &Path) -> bool {
        // Проверяем структуру пути: .../Forms/FormName/Ext/Form.xml
        let components: Vec<_> = path.components().collect();
        
        if components.len() < 4 {
            return false;
        }
        
        // Проверяем, что в пути есть папка "Forms" и "Ext"
        let has_forms = components.iter().any(|c| c.as_os_str() == "Forms");
        let has_ext = components.iter().any(|c| c.as_os_str() == "Ext");
        
        has_forms && has_ext
    }
    
    /// Извлекает имя формы из пути к файлу
    fn extract_form_name_from_path(&self, path: &Path) -> String {
        // Структура: .../Forms/FormName/Ext/Form.xml
        // Имя формы - папка перед Ext
        if let Some(ext_parent) = path.parent() {
            if let Some(form_parent) = ext_parent.parent() {
                if let Some(form_name) = form_parent.file_name() {
                    return form_name.to_string_lossy().to_string();
                }
            }
        }
        "Unknown".to_string()
    }
    
    /// Извлекает имя объекта из пути к форме
    fn extract_object_name_from_path(&self, path: &Path) -> Option<String> {
        let components: Vec<_> = path.components().collect();
        
        // Ищем структуру: ObjectType/ObjectName/Forms/FormName/Ext/Form.xml
        // Ищем компонент "Forms" и берем предыдущий компонент как имя объекта
        for (i, component) in components.iter().enumerate() {
            if component.as_os_str() == "Forms" && i > 0 {
                if let Some(object_component) = components.get(i - 1) {
                    if let Some(name) = object_component.as_os_str().to_str() {
                        return Some(name.to_string());
                    }
                }
            }
        }
        
        None
    }
    
    /// Извлекает атрибуты основного элемента Form
    fn extract_form_attributes(&self, form_contract: &mut FormContract, element: &quick_xml::events::BytesStart) -> Result<()> {
        for attr in element.attributes() {
            let attr = attr?;
            let key = String::from_utf8_lossy(attr.key.as_ref());
            let value = String::from_utf8_lossy(&attr.value);
            
            match key.as_ref() {
                "Synonym" => form_contract.synonym = Some(value.to_string()),
                "Comment" => form_contract.comment = Some(value.to_string()),
                _ => {}
            }
        }
        Ok(())
    }
    
    /// Проверяет, является ли тег элементом формы
    fn is_form_element_tag(&self, tag_name: &str) -> bool {
        matches!(tag_name, 
            "InputField" | "Label" | "Button" | "Picture" | "Table" | 
            "Group" | "Page" | "CheckBox" | "RadioButton" | "ProgressBar" |
            "Calendar" | "HtmlDocument" | "SpreadsheetDocument" | "TextDocument" |
            "Chart" | "Decoration" | "CommandBar" | "ContextMenu" | "FormattedDocument"
        )
    }
    
    /// Начинает парсинг элемента формы
    fn start_form_element(&self, tag_name: &str, element: &quick_xml::events::BytesStart) -> Result<FormElement> {
        let mut form_element = FormElement {
            name: String::new(),
            element_type: self.parse_element_type(tag_name),
            title: None,
            data_path: None,
            properties: HashMap::new(),
            events: Vec::new(),
            parent: None,
            children: Vec::new(),
        };
        
        // Извлекаем атрибуты элемента
        for attr in element.attributes() {
            let attr = attr?;
            let key = String::from_utf8_lossy(attr.key.as_ref());
            let value = String::from_utf8_lossy(&attr.value);
            
            match key.as_ref() {
                "name" => form_element.name = value.to_string(),
                "Title" => form_element.title = Some(value.to_string()),
                "DataPath" => form_element.data_path = Some(value.to_string()),
                _ => {
                    form_element.properties.insert(key.to_string(), serde_json::Value::String(value.to_string()));
                }
            }
        }
        
        Ok(form_element)
    }
    
    /// Начинает парсинг реквизита формы
    fn start_form_attribute(&self, element: &quick_xml::events::BytesStart) -> Result<FormAttribute> {
        let mut form_attribute = FormAttribute {
            name: String::new(),
            data_type: "Undefined".to_string(),
            title: None,
            save_data: true,
            properties: HashMap::new(),
        };
        
        // Извлекаем атрибуты реквизита
        for attr in element.attributes() {
            let attr = attr?;
            let key = String::from_utf8_lossy(attr.key.as_ref());
            let value = String::from_utf8_lossy(&attr.value);
            
            match key.as_ref() {
                "name" => form_attribute.name = value.to_string(),
                "Title" => form_attribute.title = Some(value.to_string()),
                "Type" => form_attribute.data_type = value.to_string(),
                "SaveData" => form_attribute.save_data = value == "true",
                _ => {
                    form_attribute.properties.insert(key.to_string(), serde_json::Value::String(value.to_string()));
                }
            }
        }
        
        Ok(form_attribute)
    }
    
    /// Начинает парсинг команды формы
    fn start_form_command(&self, element: &quick_xml::events::BytesStart) -> Result<FormCommand> {
        let mut form_command = FormCommand {
            name: String::new(),
            title: None,
            action: None,
            representation: CommandRepresentation::Auto,
            properties: HashMap::new(),
        };
        
        // Извлекаем атрибуты команды
        for attr in element.attributes() {
            let attr = attr?;
            let key = String::from_utf8_lossy(attr.key.as_ref());
            let value = String::from_utf8_lossy(&attr.value);
            
            match key.as_ref() {
                "name" => form_command.name = value.to_string(),
                "Title" => form_command.title = Some(value.to_string()),
                "Action" => form_command.action = Some(value.to_string()),
                "Representation" => form_command.representation = self.parse_command_representation(&value),
                _ => {
                    form_command.properties.insert(key.to_string(), serde_json::Value::String(value.to_string()));
                }
            }
        }
        
        Ok(form_command)
    }
    
    /// Обрабатывает свойство элемента формы
    fn process_element_property(
        &self,
        current_element: &mut Option<FormElement>,
        property_name: &str,
        reader: &mut Reader<&[u8]>
    ) -> Result<()> {
        // Читаем значение свойства
        let mut buf = Vec::new();
        if let Ok(Event::Text(e)) = reader.read_event_into(&mut buf) {
            let value = e.unescape().unwrap_or_default();
            
            if let Some(element) = current_element {
                element.properties.insert(
                    property_name.to_string(),
                    serde_json::Value::String(value.to_string())
                );
            }
        }
        
        Ok(())
    }
    
    /// Извлекает атрибуты реальной формы 1С
    fn extract_real_form_attributes(&self, form_contract: &mut FormContract, element: &quick_xml::events::BytesStart) -> Result<()> {
        // В реальных формах атрибуты редко находятся в корневом элементе
        // Большинство информации извлекается из подэлементов
        tracing::debug!("Extracting real form attributes");
        Ok(())
    }
    
    /// Начинает парсинг элемента реальной формы 1С
    fn start_real_form_element(&self, tag_name: &str, element: &quick_xml::events::BytesStart) -> Result<FormElement> {
        let mut form_element = FormElement {
            name: String::new(),
            element_type: self.parse_real_element_type(tag_name),
            title: None,
            data_path: None,
            properties: HashMap::new(),
            events: Vec::new(),
            parent: None,
            children: Vec::new(),
        };
        
        // Извлекаем атрибуты элемента из реальной структуры
        for attr in element.attributes() {
            let attr = attr?;
            let key = String::from_utf8_lossy(attr.key.as_ref());
            let value = String::from_utf8_lossy(&attr.value);
            
            match key.as_ref() {
                "name" => form_element.name = value.to_string(),
                "id" => {
                    form_element.properties.insert("id".to_string(), serde_json::Value::String(value.to_string()));
                }
                _ => {
                    form_element.properties.insert(key.to_string(), serde_json::Value::String(value.to_string()));
                }
            }
        }
        
        tracing::debug!("Created real form element: {} ({})", form_element.name, tag_name);
        Ok(form_element)
    }
    
    /// Добавляет событие к элементу формы
    fn add_event_to_element(&self, element: &mut FormElement, event_element: &quick_xml::events::BytesStart) -> Result<()> {
        let mut event_name = String::new();
        
        // Извлекаем атрибуты события
        for attr in event_element.attributes() {
            let attr = attr?;
            let key = String::from_utf8_lossy(attr.key.as_ref());
            let value = String::from_utf8_lossy(&attr.value);
            
            match key.as_ref() {
                "name" => event_name = value.to_string(),
                _ => {}
            }
        }
        
        if !event_name.is_empty() {
            element.events.push(event_name);
        }
        
        Ok(())
    }
    
    /// Парсит тип элемента управления для реальных форм
    fn parse_real_element_type(&self, tag_name: &str) -> FormElementType {
        match tag_name {
            "InputField" => FormElementType::InputField,
            "Table" => FormElementType::Table,
            "RadioButtonField" => FormElementType::RadioButton,
            "CheckBoxField" => FormElementType::CheckBox,
            "ButtonField" | "Button" => FormElementType::Button,
            "LabelField" => FormElementType::Label,
            "PictureField" => FormElementType::Picture,
            "SpreadsheetDocumentField" => FormElementType::SpreadsheetDocument,
            "TextDocumentField" => FormElementType::TextDocument,
            "FormattedDocumentField" => FormElementType::FormattedDocument,
            "Pages" => FormElementType::Group, // Страницы как группы
            "Page" => FormElementType::Group,
            "Group" => FormElementType::Group,
            "Decoration" => FormElementType::Decoration,
            "CommandBar" => FormElementType::CommandBar,
            _ => FormElementType::Unknown(tag_name.to_string()),
        }
    }

    /// Парсит тип элемента управления
    fn parse_element_type(&self, tag_name: &str) -> FormElementType {
        match tag_name {
            "InputField" => FormElementType::InputField,
            "Label" => FormElementType::Label,
            "Button" => FormElementType::Button,
            "Picture" => FormElementType::Picture,
            "Table" => FormElementType::Table,
            "Group" => FormElementType::Group,
            "Page" => FormElementType::Page,
            "CheckBox" => FormElementType::CheckBox,
            "RadioButton" => FormElementType::RadioButton,
            "ProgressBar" => FormElementType::ProgressBar,
            "Calendar" => FormElementType::Calendar,
            "HtmlDocument" => FormElementType::HtmlDocument,
            "SpreadsheetDocument" => FormElementType::SpreadsheetDocument,
            "TextDocument" => FormElementType::TextDocument,
            "Chart" => FormElementType::Chart,
            "Decoration" => FormElementType::Decoration,
            "CommandBar" => FormElementType::CommandBar,
            "ContextMenu" => FormElementType::ContextMenu,
            "FormattedDocument" => FormElementType::FormattedDocument,
            _ => FormElementType::Unknown(tag_name.to_string()),
        }
    }
    
    /// Парсит представление команды
    fn parse_command_representation(&self, value: &str) -> CommandRepresentation {
        match value {
            "Auto" => CommandRepresentation::Auto,
            "Text" => CommandRepresentation::Text,
            "Picture" => CommandRepresentation::Picture,
            "TextPicture" => CommandRepresentation::TextPicture,
            _ => CommandRepresentation::Auto,
        }
    }
    
    /// Определяет тип формы по ее имени и структуре
    fn determine_form_type(&self, form_contract: &FormContract) -> FormType {
        let name_lower = form_contract.name.to_lowercase();
        
        if name_lower.contains("список") || name_lower.contains("list") {
            FormType::ListForm
        } else if name_lower.contains("выбор") || name_lower.contains("choice") {
            FormType::ChoiceForm
        } else if name_lower.contains("элемент") || name_lower.contains("item") {
            FormType::ItemForm
        } else if name_lower.contains("настройки") || name_lower.contains("settings") {
            FormType::SettingsForm
        } else if name_lower.contains("отчет") || name_lower.contains("report") {
            FormType::ReportForm
        } else if name_lower.contains("обработка") || name_lower.contains("dataprocessor") {
            FormType::DataProcessorForm
        } else if form_contract.object_name.is_some() {
            FormType::ObjectForm
        } else {
            FormType::CommonForm
        }
    }
    
    /// Парсит все формы и записывает в гибридное хранилище
    pub fn parse_to_hybrid_storage<P: AsRef<Path>>(
        &self, 
        config_dir: P,
        storage: &mut HybridDocumentationStorage
    ) -> Result<()> {
        let form_files = self.find_form_files(config_dir)?;
        
        for form_file in form_files {
            let form_contract = self.parse_form_xml(&form_file)?;
            
            // Сохраняем форму в оптимизированное хранилище
            storage.add_form_optimized(&form_contract)?;
        }
        
        Ok(())
    }
}

impl Default for FormXmlParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    
    #[test]
    fn test_form_parser_creation() {
        let parser = FormXmlParser::new();
        assert!(!parser.xmlns_patterns.is_empty());
    }
    
    #[test]
    fn test_is_valid_form_path() {
        let parser = FormXmlParser::new();
        
        assert!(parser.is_valid_form_path(Path::new("Catalogs/Items/Forms/ItemForm/Form.xml")));
        assert!(parser.is_valid_form_path(Path::new("CommonForms/MainForm/Forms/Form/Form.xml")));
        assert!(!parser.is_valid_form_path(Path::new("invalid/path/Form.xml")));
        assert!(!parser.is_valid_form_path(Path::new("Form.xml")));
    }
    
    #[test]
    fn test_extract_form_name_from_path() {
        let parser = FormXmlParser::new();
        
        let name = parser.extract_form_name_from_path(Path::new("Catalogs/Items/Forms/ItemForm/Form.xml"));
        assert_eq!(name, "ItemForm");
        
        let name = parser.extract_form_name_from_path(Path::new("CommonForms/MainForm/Forms/Form/Form.xml"));
        assert_eq!(name, "Form");
    }
    
    #[test]
    fn test_extract_object_name_from_path() {
        let parser = FormXmlParser::new();
        
        let name = parser.extract_object_name_from_path(Path::new("Catalogs/Items/Forms/ItemForm/Form.xml"));
        assert_eq!(name, Some("Items".to_string()));
        
        let name = parser.extract_object_name_from_path(Path::new("CommonForms/MainForm/Form.xml"));
        assert_eq!(name, None);
    }
    
    #[test]
    fn test_parse_element_type() {
        let parser = FormXmlParser::new();
        
        assert!(matches!(parser.parse_element_type("InputField"), FormElementType::InputField));
        assert!(matches!(parser.parse_element_type("Button"), FormElementType::Button));
        assert!(matches!(parser.parse_element_type("Table"), FormElementType::Table));
        
        if let FormElementType::Unknown(name) = parser.parse_element_type("CustomElement") {
            assert_eq!(name, "CustomElement");
        } else {
            panic!("Expected Unknown type");
        }
    }
    
    #[test]
    fn test_determine_form_type() {
        let parser = FormXmlParser::new();
        
        let mut form = FormContract {
            metadata_type: "Form".to_string(),
            name: "СписокТоваров".to_string(),
            synonym: None,
            comment: None,
            form_type: FormType::Unknown("".to_string()),
            object_name: Some("Товары".to_string()),
            structure: FormStructure {
                elements_count: 0,
                attributes_count: 0,
                elements: Vec::new(),
                attributes: Vec::new(),
                commands: Vec::new(),
            },
            generation_metadata: FormGenerationMetadata {
                timestamp: "".to_string(),
                generator_version: "".to_string(),
                source_file: "".to_string(),
                form_path: "".to_string(),
            },
        };
        
        assert!(matches!(parser.determine_form_type(&form), FormType::ListForm));
        
        form.name = "ФормаЭлемента".to_string();
        assert!(matches!(parser.determine_form_type(&form), FormType::ItemForm));
    }
    
    #[test]
    fn test_find_form_files() {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path();
        
        // Создаем структуру с формами
        let form_dir = config_dir.join("Catalogs").join("Items").join("Forms").join("ItemForm");
        fs::create_dir_all(&form_dir).unwrap();
        fs::write(form_dir.join("Form.xml"), "<Form></Form>").unwrap();
        
        let parser = FormXmlParser::new();
        let forms = parser.find_form_files(config_dir).unwrap();
        
        assert_eq!(forms.len(), 1);
        assert!(forms[0].ends_with("Form.xml"));
    }
}