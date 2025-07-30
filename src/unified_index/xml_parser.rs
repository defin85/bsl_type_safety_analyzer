use std::path::{Path, PathBuf};
use std::fs;
use anyhow::{Result, Context};
use quick_xml::events::Event;
use quick_xml::Reader;

use super::entity::{BslEntity, BslEntityType, BslEntityKind, BslEntitySource, BslProperty, BslContext};

pub struct ConfigurationXmlParser {
    config_path: PathBuf,
}

impl ConfigurationXmlParser {
    pub fn new(config_path: impl AsRef<Path>) -> Self {
        Self {
            config_path: config_path.as_ref().to_path_buf(),
        }
    }
    
    pub fn parse_configuration(&self) -> Result<Vec<BslEntity>> {
        let mut entities = Vec::new();
        
        // Парсим Configuration.xml
        let config_xml_path = self.config_path.join("Configuration.xml");
        if config_xml_path.exists() {
            let config_entity = self.parse_configuration_xml(&config_xml_path)?;
            entities.push(config_entity);
        }
        
        // Парсим все объекты метаданных
        entities.extend(self.parse_metadata_objects("Catalogs", BslEntityKind::Catalog)?);
        entities.extend(self.parse_metadata_objects("Documents", BslEntityKind::Document)?);
        entities.extend(self.parse_metadata_objects("ChartsOfCharacteristicTypes", BslEntityKind::ChartOfCharacteristicTypes)?);
        entities.extend(self.parse_metadata_objects("ChartsOfAccounts", BslEntityKind::ChartOfAccounts)?);
        entities.extend(self.parse_metadata_objects("ChartsOfCalculationTypes", BslEntityKind::ChartOfCalculationTypes)?);
        entities.extend(self.parse_metadata_objects("InformationRegisters", BslEntityKind::InformationRegister)?);
        entities.extend(self.parse_metadata_objects("AccumulationRegisters", BslEntityKind::AccumulationRegister)?);
        entities.extend(self.parse_metadata_objects("AccountingRegisters", BslEntityKind::AccountingRegister)?);
        entities.extend(self.parse_metadata_objects("CalculationRegisters", BslEntityKind::CalculationRegister)?);
        entities.extend(self.parse_metadata_objects("BusinessProcesses", BslEntityKind::BusinessProcess)?);
        entities.extend(self.parse_metadata_objects("Tasks", BslEntityKind::Task)?);
        entities.extend(self.parse_metadata_objects("ExchangePlans", BslEntityKind::ExchangePlan)?);
        
        // Парсим общие модули
        entities.extend(self.parse_common_modules()?);
        
        Ok(entities)
    }
    
    pub fn parse_metadata_object(&self, xml_path: &Path) -> Result<BslEntity> {
        let content = fs::read_to_string(xml_path)
            .context("Failed to read metadata XML file")?;
            
        let mut reader = Reader::from_str(&content);
        reader.trim_text(true);
        
        let mut entity = BslEntity::new(
            String::new(),
            String::new(),
            BslEntityType::Configuration,
            BslEntityKind::Other(String::new())
        );
        
        let mut buf = Vec::new();
        let mut in_properties = false;
        let mut in_attributes = false;
        let mut _in_tabular_sections = false;
        let mut current_element = String::new();
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    
                    match tag_name.as_str() {
                        "MetaDataObject" => {
                            // Определяем тип объекта по атрибутам
                            for attr in e.attributes() {
                                if let Ok(attr) = attr {
                                    let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                                    let value = String::from_utf8_lossy(&attr.value).to_string();
                                    
                                    if key == "name" {
                                        entity.display_name = value.clone();
                                        entity.id.0 = value;
                                    }
                                }
                            }
                        }
                        "Properties" => in_properties = true,
                        "ChildObjects" => {
                            if let Ok(Event::Start(ref child)) = reader.read_event_into(&mut buf) {
                                let child_name = String::from_utf8_lossy(child.name().as_ref()).to_string();
                                match child_name.as_str() {
                                    "Attribute" => in_attributes = true,
                                    "TabularSection" => _in_tabular_sections = true,
                                    _ => {}
                                }
                            }
                        }
                        _ => current_element = tag_name,
                    }
                }
                Ok(Event::Text(e)) => {
                    let text = e.unescape()?.to_string();
                    
                    if in_properties {
                        match current_element.as_str() {
                            "Name" => entity.display_name = text.clone(),
                            "Synonym" => {
                                if let Some(ru_synonym) = self.extract_ru_synonym(&text) {
                                    entity.documentation = Some(ru_synonym);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    match tag_name.as_str() {
                        "Properties" => in_properties = false,
                        "Attribute" => {
                            if in_attributes {
                                // Добавляем атрибут как свойство
                                if let Ok(attr) = self.parse_attribute(&mut reader, &mut buf) {
                                    entity.interface.properties.insert(attr.name.clone(), attr);
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(anyhow::anyhow!("XML parsing error: {}", e)),
                _ => {}
            }
            buf.clear();
        }
        
        entity.source = BslEntitySource::ConfigurationXml { path: xml_path.to_string_lossy().to_string() };
        Ok(entity)
    }
    
    fn parse_configuration_xml(&self, path: &Path) -> Result<BslEntity> {
        let mut entity = BslEntity::new(
            "Configuration".to_string(),
            "Configuration".to_string(),
            BslEntityType::Configuration,
            BslEntityKind::Other("Configuration".to_string())
        );
        
        entity.source = BslEntitySource::ConfigurationXml { path: path.to_string_lossy().to_string() };
        
        // TODO: Парсинг свойств конфигурации
        
        Ok(entity)
    }
    
    fn parse_metadata_objects(&self, folder_name: &str, kind: BslEntityKind) -> Result<Vec<BslEntity>> {
        let mut entities = Vec::new();
        let objects_path = self.config_path.join(folder_name);
        
        if !objects_path.exists() {
            return Ok(entities);
        }
        
        for entry in fs::read_dir(&objects_path)? {
            let entry = entry?;
            let path = entry.path();
            
            // Поддержка двух структур:
            // 1. Старая: Catalogs/Контрагенты/Контрагенты.xml
            // 2. Новая: Catalogs/Контрагенты.xml
            
            if path.is_dir() {
                // Старая структура - XML внутри папки
                let xml_path = path.join(format!("{}.xml", path.file_name().unwrap().to_string_lossy()));
                if xml_path.exists() {
                    if let Ok(mut entity) = self.parse_metadata_object(&xml_path) {
                        entity.entity_kind = kind.clone();
                        
                        // Формируем квалифицированное имя
                        let object_name = path.file_name().unwrap().to_string_lossy().to_string();
                        entity.qualified_name = format!("{}.{}", self.get_kind_prefix(&kind), object_name);
                        entity.id.0 = entity.qualified_name.clone();
                        
                        // Устанавливаем родительские типы
                        entity.constraints.parent_types = self.get_parent_types(&kind);
                        
                        // Парсим формы объекта
                        let forms_path = path.join("Forms");
                        if forms_path.exists() {
                            for form_entry in fs::read_dir(&forms_path)? {
                                let form_entry = form_entry?;
                                let form_name = form_entry.file_name().to_string_lossy().to_string();
                                entity.relationships.forms.push(form_name);
                            }
                        }
                        
                        entities.push(entity);
                    }
                }
            } else if path.extension().map_or(false, |ext| ext == "xml") {
                // Новая структура - XML файлы прямо в папке
                if let Ok(mut entity) = self.parse_metadata_object(&path) {
                    entity.entity_kind = kind.clone();
                    
                    // Формируем квалифицированное имя из имени файла
                    let object_name = path.file_stem().unwrap().to_string_lossy().to_string();
                    entity.qualified_name = format!("{}.{}", self.get_kind_prefix(&kind), object_name);
                    entity.id.0 = entity.qualified_name.clone();
                    
                    // Устанавливаем родительские типы
                    entity.constraints.parent_types = self.get_parent_types(&kind);
                    
                    // Проверяем наличие папки с таким же именем для форм
                    let object_dir = self.config_path.join(folder_name).join(&object_name);
                    if object_dir.exists() {
                        let forms_path = object_dir.join("Forms");
                        if forms_path.exists() {
                            for form_entry in fs::read_dir(&forms_path)? {
                                let form_entry = form_entry?;
                                let form_name = form_entry.file_name().to_string_lossy().to_string();
                                entity.relationships.forms.push(form_name);
                            }
                        }
                    }
                    
                    entities.push(entity);
                }
            }
        }
        
        Ok(entities)
    }
    
    fn parse_common_modules(&self) -> Result<Vec<BslEntity>> {
        let mut entities = Vec::new();
        let modules_path = self.config_path.join("CommonModules");
        
        if !modules_path.exists() {
            return Ok(entities);
        }
        
        for entry in fs::read_dir(&modules_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                let xml_path = path.join(format!("{}.xml", path.file_name().unwrap().to_string_lossy()));
                if xml_path.exists() {
                    if let Ok(mut entity) = self.parse_metadata_object(&xml_path) {
                        entity.entity_kind = BslEntityKind::CommonModule;
                        entity.entity_type = BslEntityType::Module;
                        
                        let module_name = path.file_name().unwrap().to_string_lossy().to_string();
                        entity.qualified_name = format!("ОбщиеМодули.{}", module_name);
                        entity.id.0 = entity.qualified_name.clone();
                        
                        entities.push(entity);
                    }
                }
            }
        }
        
        Ok(entities)
    }
    
    fn parse_attribute(&self, reader: &mut Reader<&[u8]>, buf: &mut Vec<u8>) -> Result<BslProperty> {
        let mut property = BslProperty {
            name: String::new(),
            english_name: None,
            type_name: String::new(),
            is_readonly: false,
            is_indexed: false,
            documentation: None,
            availability: vec![BslContext::Server, BslContext::Client],
        };
        
        let mut in_properties = false;
        let mut in_type = false;
        let mut current_element = String::new();
        
        loop {
            match reader.read_event_into(buf) {
                Ok(Event::Start(ref e)) => {
                    let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    match tag_name.as_str() {
                        "Properties" => in_properties = true,
                        "Type" => in_type = true,
                        _ => current_element = tag_name,
                    }
                }
                Ok(Event::Text(e)) => {
                    let text = e.unescape()?.to_string();
                    
                    if in_properties && current_element == "Name" {
                        property.name = text;
                    } else if in_type {
                        property.type_name = self.parse_type_definition(&text);
                    }
                }
                Ok(Event::End(ref e)) => {
                    let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if tag_name == "Attribute" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                _ => {}
            }
            buf.clear();
        }
        
        Ok(property)
    }
    
    fn get_kind_prefix(&self, kind: &BslEntityKind) -> &'static str {
        match kind {
            BslEntityKind::Catalog => "Справочники",
            BslEntityKind::Document => "Документы",
            BslEntityKind::ChartOfCharacteristicTypes => "ПланыВидовХарактеристик",
            BslEntityKind::ChartOfAccounts => "ПланыСчетов",
            BslEntityKind::ChartOfCalculationTypes => "ПланыВидовРасчета",
            BslEntityKind::InformationRegister => "РегистрыСведений",
            BslEntityKind::AccumulationRegister => "РегистрыНакопления",
            BslEntityKind::AccountingRegister => "РегистрыБухгалтерии",
            BslEntityKind::CalculationRegister => "РегистрыРасчета",
            BslEntityKind::BusinessProcess => "БизнесПроцессы",
            BslEntityKind::Task => "Задачи",
            BslEntityKind::ExchangePlan => "ПланыОбмена",
            _ => "Прочие",
        }
    }
    
    fn get_parent_types(&self, kind: &BslEntityKind) -> Vec<String> {
        match kind {
            BslEntityKind::Catalog => vec!["СправочникОбъект".to_string()],
            BslEntityKind::Document => vec!["ДокументОбъект".to_string()],
            BslEntityKind::ChartOfCharacteristicTypes => vec!["ПланВидовХарактеристикОбъект".to_string()],
            BslEntityKind::ChartOfAccounts => vec!["ПланСчетовОбъект".to_string()],
            BslEntityKind::ChartOfCalculationTypes => vec!["ПланВидовРасчетаОбъект".to_string()],
            BslEntityKind::InformationRegister => vec!["РегистрСведенийНаборЗаписей".to_string()],
            BslEntityKind::AccumulationRegister => vec!["РегистрНакопленияНаборЗаписей".to_string()],
            BslEntityKind::AccountingRegister => vec!["РегистрБухгалтерииНаборЗаписей".to_string()],
            BslEntityKind::CalculationRegister => vec!["РегистрРасчетаНаборЗаписей".to_string()],
            _ => vec![],
        }
    }
    
    fn extract_ru_synonym(&self, synonym: &str) -> Option<String> {
        // Извлекаем русский синоним из мультиязычной строки
        if synonym.contains("ru='") {
            let start = synonym.find("ru='")? + 4;
            let end = synonym[start..].find('\'')?;
            Some(synonym[start..start+end].to_string())
        } else {
            Some(synonym.to_string())
        }
    }
    
    fn parse_type_definition(&self, type_str: &str) -> String {
        // Упрощенный парсинг типов
        // TODO: Полная реализация парсинга составных типов
        type_str.to_string()
    }
    
    pub fn parse_object_forms(&self, object_path: &Path) -> Result<Vec<BslEntity>> {
        let mut forms = Vec::new();
        let forms_path = object_path.join("Forms");
        
        if !forms_path.exists() {
            return Ok(forms);
        }
        
        for entry in fs::read_dir(&forms_path)? {
            let entry = entry?;
            let form_path = entry.path();
            
            if form_path.is_dir() {
                let form_xml = form_path.join("Form.xml");
                if form_xml.exists() {
                    let form_name = form_path.file_name().unwrap().to_string_lossy().to_string();
                    let parent_name = object_path.file_name().unwrap().to_string_lossy().to_string();
                    
                    let mut form_entity = BslEntity::new(
                        format!("{}.{}", parent_name, form_name),
                        format!("{}.{}", parent_name, form_name),
                        BslEntityType::Form,
                        BslEntityKind::ManagedForm
                    );
                    
                    form_entity.source = BslEntitySource::FormXml { path: form_xml.to_string_lossy().to_string() };
                    form_entity.relationships.owner = Some(parent_name);
                    
                    forms.push(form_entity);
                }
            }
        }
        
        Ok(forms)
    }
}