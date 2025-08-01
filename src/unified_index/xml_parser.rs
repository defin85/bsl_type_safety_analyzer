use std::path::{Path, PathBuf};
use std::fs;
use anyhow::{Result, Context};
use quick_xml::events::Event;
use quick_xml::Reader;

use super::entity::{BslEntity, BslEntityType, BslEntityKind, BslEntitySource, BslProperty, BslContext, BslEntityId, BslInterface, BslConstraints, BslRelationships, BslLifecycle, BslTabularSection};

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
        
        // Парсим все объекты метаданных с их формами
        entities.extend(self.parse_metadata_objects_with_forms("Catalogs", BslEntityKind::Catalog)?);
        entities.extend(self.parse_metadata_objects_with_forms("Documents", BslEntityKind::Document)?);
        entities.extend(self.parse_metadata_objects_with_forms("ChartsOfCharacteristicTypes", BslEntityKind::ChartOfCharacteristicTypes)?);
        entities.extend(self.parse_metadata_objects_with_forms("ChartsOfAccounts", BslEntityKind::ChartOfAccounts)?);
        entities.extend(self.parse_metadata_objects_with_forms("ChartsOfCalculationTypes", BslEntityKind::ChartOfCalculationTypes)?);
        entities.extend(self.parse_metadata_objects_with_forms("InformationRegisters", BslEntityKind::InformationRegister)?);
        entities.extend(self.parse_metadata_objects_with_forms("AccumulationRegisters", BslEntityKind::AccumulationRegister)?);
        entities.extend(self.parse_metadata_objects_with_forms("AccountingRegisters", BslEntityKind::AccountingRegister)?);
        entities.extend(self.parse_metadata_objects_with_forms("CalculationRegisters", BslEntityKind::CalculationRegister)?);
        entities.extend(self.parse_metadata_objects_with_forms("BusinessProcesses", BslEntityKind::BusinessProcess)?);
        entities.extend(self.parse_metadata_objects_with_forms("Tasks", BslEntityKind::Task)?);
        entities.extend(self.parse_metadata_objects_with_forms("ExchangePlans", BslEntityKind::ExchangePlan)?);
        
        // Парсим константы (они не имеют форм)
        entities.extend(self.parse_metadata_objects("Constants", BslEntityKind::Constant)?);
        
        // Парсим перечисления (они не имеют форм)
        entities.extend(self.parse_metadata_objects("Enums", BslEntityKind::Enum)?);
        
        // Парсим отчеты с их формами
        entities.extend(self.parse_metadata_objects_with_forms("Reports", BslEntityKind::Report)?);
        
        // Парсим обработки с их формами
        entities.extend(self.parse_metadata_objects_with_forms("DataProcessors", BslEntityKind::DataProcessor)?);
        
        // Парсим журналы документов с их формами
        entities.extend(self.parse_metadata_objects_with_forms("DocumentJournals", BslEntityKind::DocumentJournal)?);
        
        // Парсим общие модули
        entities.extend(self.parse_common_modules()?);
        
        // Парсим общие формы
        entities.extend(self.parse_common_forms()?);
        
        Ok(entities)
    }
    
    fn parse_metadata_objects_with_forms(&self, folder_name: &str, kind: BslEntityKind) -> Result<Vec<BslEntity>> {
        let mut all_entities = Vec::new();
        
        // Сначала парсим сами объекты
        let objects = self.parse_metadata_objects(folder_name, kind)?;
        
        // Затем для каждого объекта парсим его формы
        for object in objects {
            let object_name = object.display_name.clone();
            let qualified_name = object.qualified_name.clone();
            all_entities.push(object);
            
            // Определяем путь к объекту
            let object_path = self.config_path.join(folder_name).join(&object_name);
            if object_path.exists() {
                // Парсим формы объекта с полным квалифицированным именем
                let forms = self.parse_object_forms(&object_path, &qualified_name)?;
                all_entities.extend(forms);
            }
        }
        
        Ok(all_entities)
    }
    
    fn parse_common_forms(&self) -> Result<Vec<BslEntity>> {
        let mut entities = Vec::new();
        let forms_path = self.config_path.join("CommonForms");
        
        if !forms_path.exists() {
            return Ok(entities);
        }
        
        for entry in fs::read_dir(&forms_path)? {
            let entry = entry?;
            let form_path = entry.path();
            
            if form_path.is_dir() {
                let form_xml = form_path.join("Form.xml");
                let ext_form_xml = form_path.join("Ext").join("Form.xml");
                
                let actual_form_xml = if form_xml.exists() {
                    form_xml
                } else if ext_form_xml.exists() {
                    ext_form_xml
                } else {
                    continue;
                };
                
                let form_name = form_path.file_name().unwrap().to_string_lossy().to_string();
                let form_entity = self.parse_form_xml(&actual_form_xml, "ОбщиеФормы", &form_name)?;
                entities.push(form_entity);
            }
        }
        
        Ok(entities)
    }
    
    pub fn parse_metadata_object(&self, xml_path: &Path, kind: Option<&BslEntityKind>) -> Result<BslEntity> {
        let content = fs::read_to_string(xml_path)
            .context("Failed to read metadata XML file")?;
            
        let mut reader = Reader::from_str(&content);
        reader.trim_text(true);
        
        let mut entity = BslEntity::new(
            String::new(),
            String::new(),
            BslEntityType::Configuration,
            kind.cloned().unwrap_or(BslEntityKind::Other(String::new()))
        );
        
        let mut buf = Vec::new();
        let mut in_properties = false;
        let mut in_attributes = false;
        let mut in_child_objects = false;
        let mut in_enum_value = false;
        let mut in_enum_properties = false;
        let mut current_element = String::new();
        let mut current_enum_name = String::new();
        let mut current_enum_synonym = String::new();
        
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
                        "Properties" if !in_enum_value => in_properties = true,
                        "Type" if in_properties => {
                            // Для констант парсим тип как свойство
                            if entity.entity_kind == BslEntityKind::Constant {
                                if let Ok(type_name) = self.parse_constant_type(&mut reader, &mut buf) {
                                let prop = BslProperty {
                                    name: "Значение".to_string(),
                                    type_name,
                                    is_readonly: false,
                                    documentation: None,
                                    availability: vec![BslContext::Client, BslContext::Server],
                                    english_name: Some("Value".to_string()),
                                    is_indexed: false,
                                };
                                entity.interface.properties.insert("Значение".to_string(), prop);
                            }
                            }
                        }
                        "ChildObjects" => {
                            in_child_objects = true;
                        }
                        "EnumValue" if in_child_objects => {
                            in_enum_value = true;
                            current_enum_name.clear();
                            current_enum_synonym.clear();
                        }
                        "Properties" if in_enum_value => {
                            in_enum_properties = true;
                        }
                        "Attribute" if !in_attributes => {
                            // Это атрибут объекта, не табличной части
                            in_attributes = true;
                        }
                        "TabularSection" => {
                            // Парсим табличную часть
                            if let Ok(ts) = self.parse_tabular_section(&mut reader, &mut buf) {
                                entity.relationships.tabular_sections.push(ts);
                            }
                        }
                        _ => current_element = tag_name,
                    }
                }
                Ok(Event::Text(e)) => {
                    let text = e.unescape()?.to_string();
                    
                    if in_properties && !in_enum_value {
                        match current_element.as_str() {
                            "Name" => entity.display_name = text.clone(),
                            "Synonym" => {
                                if let Some(ru_synonym) = self.extract_ru_synonym(&text) {
                                    entity.documentation = Some(ru_synonym);
                                }
                            }
                            _ => {}
                        }
                    } else if in_enum_properties && in_enum_value {
                        match current_element.as_str() {
                            "Name" => current_enum_name = text,
                            "v8:content" => current_enum_synonym = text,
                            _ => {}
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    match tag_name.as_str() {
                        "Properties" if in_enum_value => in_enum_properties = false,
                        "Properties" => in_properties = false,
                        "EnumValue" => {
                            // Сохраняем значение перечисления
                            if !current_enum_name.is_empty() {
                                let enum_prop = BslProperty {
                                    name: current_enum_name.clone(),
                                    english_name: None,
                                    type_name: "EnumValue".to_string(),
                                    is_readonly: true,
                                    is_indexed: false,
                                    documentation: if current_enum_synonym.is_empty() { None } else { Some(current_enum_synonym.clone()) },
                                    availability: vec![BslContext::Client, BslContext::Server],
                                };
                                entity.interface.properties.insert(current_enum_name.clone(), enum_prop);
                            }
                            in_enum_value = false;
                        }
                        "ChildObjects" => in_child_objects = false,
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
                    if let Ok(mut entity) = self.parse_metadata_object(&xml_path, Some(&kind)) {
                        
                        // Формируем квалифицированное имя
                        let object_name = path.file_name().unwrap().to_string_lossy().to_string();
                        entity.qualified_name = format!("{}.{}", self.get_kind_prefix(&kind), object_name);
                        entity.id.0 = entity.qualified_name.clone();
                        
                        // Устанавливаем родительские типы
                        entity.constraints.parent_types = self.get_parent_types(&kind);
                        
                        // Собираем только имена форм для relationships
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
                if let Ok(mut entity) = self.parse_metadata_object(&path, Some(&kind)) {
                    
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
                    if let Ok(mut entity) = self.parse_metadata_object(&xml_path, None) {
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
            BslEntityKind::Constant => "Константы",
            BslEntityKind::Enum => "Перечисления",
            BslEntityKind::Report => "Отчеты",
            BslEntityKind::DataProcessor => "Обработки",
            BslEntityKind::DocumentJournal => "ЖурналыДокументов",
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
            BslEntityKind::Constant => vec!["КонстантаМенеджерЗначения".to_string()],
            BslEntityKind::Enum => vec!["ПеречислениеСсылка".to_string()],
            BslEntityKind::Report => vec!["ОтчетОбъект".to_string()],
            BslEntityKind::DataProcessor => vec!["ОбработкаОбъект".to_string()],
            BslEntityKind::DocumentJournal => vec!["ЖурналДокументовОбъект".to_string()],
            _ => vec![],
        }
    }
    
    fn extract_ru_synonym(&self, synonym_xml: &str) -> Option<String> {
        // Для XML синонимов (не text) нужен специальный парсинг
        // В данном случае синоним приходит как текст между тегами v8:content
        // Возвращаем как есть, так как это уже извлеченное содержимое
        if synonym_xml.trim().is_empty() {
            None
        } else {
            Some(synonym_xml.trim().to_string())
        }
    }
    
    fn parse_constant_type(&self, reader: &mut Reader<&[u8]>, buf: &mut Vec<u8>) -> Result<String> {
        let mut type_name = String::new();
        let mut in_v8_type = false;
        
        loop {
            match reader.read_event_into(buf) {
                Ok(Event::Start(ref e)) => {
                    let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    match tag_name.as_str() {
                        "v8:Type" => in_v8_type = true,
                        _ => {}
                    }
                }
                Ok(Event::Text(e)) => {
                    let text = e.unescape()?.to_string();
                    if in_v8_type {
                        type_name = match text.as_str() {
                            "xs:string" => "Строка".to_string(),
                            "xs:boolean" => "Булево".to_string(),
                            "xs:decimal" => "Число".to_string(),
                            "xs:dateTime" => "ДатаВремя".to_string(),
                            _ => text,
                        };
                    }
                }
                Ok(Event::End(ref e)) => {
                    let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    match tag_name.as_str() {
                        "Type" => break,
                        "v8:Type" => in_v8_type = false,
                        _ => {}
                    }
                }
                Ok(Event::Eof) => break,
                _ => {}
            }
            buf.clear();
        }
        
        Ok(type_name)
    }
    
    fn parse_tabular_section(&self, reader: &mut Reader<&[u8]>, buf: &mut Vec<u8>) -> Result<BslTabularSection> {
        let mut tabular_section = BslTabularSection {
            name: String::new(),
            display_name: String::new(),
            attributes: Vec::new(),
        };
        
        let mut in_properties = false;
        let mut in_child_objects = false;
        let mut current_element = String::new();
        
        loop {
            match reader.read_event_into(buf) {
                Ok(Event::Start(ref e)) => {
                    let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    match tag_name.as_str() {
                        "Properties" => in_properties = true,
                        "ChildObjects" => in_child_objects = true,
                        "Attribute" if in_child_objects => {
                            if let Ok(attr) = self.parse_attribute(reader, buf) {
                                tabular_section.attributes.push(attr);
                            }
                        }
                        _ => current_element = tag_name,
                    }
                }
                Ok(Event::Text(e)) => {
                    if in_properties {
                        let text = e.unescape()?.to_string();
                        match current_element.as_str() {
                            "Name" => tabular_section.name = text,
                            "Synonym" => {
                                if let Some(ru_synonym) = self.extract_ru_synonym(&text) {
                                    tabular_section.display_name = ru_synonym;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    match tag_name.as_str() {
                        "TabularSection" => break,
                        "Properties" => in_properties = false,
                        "ChildObjects" => in_child_objects = false,
                        _ => {}
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(anyhow::anyhow!("XML parsing error in tabular section: {}", e)),
                _ => {}
            }
            buf.clear();
        }
        
        // Если display_name пустой, используем name
        if tabular_section.display_name.is_empty() {
            tabular_section.display_name = tabular_section.name.clone();
        }
        
        Ok(tabular_section)
    }
    
    fn parse_type_definition(&self, type_str: &str) -> String {
        // Упрощенный парсинг типов
        // TODO: Полная реализация парсинга составных типов
        type_str.to_string()
    }
    
    
    pub fn parse_object_forms(&self, object_path: &Path, parent_qualified_name: &str) -> Result<Vec<BslEntity>> {
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
                let ext_form_xml = form_path.join("Ext").join("Form.xml");
                
                // Поддержка двух структур: Forms/FormName/Form.xml и Forms/FormName/Ext/Form.xml
                let actual_form_xml = if form_xml.exists() {
                    form_xml
                } else if ext_form_xml.exists() {
                    ext_form_xml
                } else {
                    continue;
                };
                
                let form_name = form_path.file_name().unwrap().to_string_lossy().to_string();
                
                // Парсим детальную информацию о форме с полным квалифицированным именем
                let form_entity = self.parse_form_xml(&actual_form_xml, parent_qualified_name, &form_name)?;
                forms.push(form_entity);
            }
        }
        
        Ok(forms)
    }
    
    fn parse_form_xml(&self, form_xml_path: &Path, parent_name: &str, form_name: &str) -> Result<BslEntity> {
        let content = fs::read_to_string(form_xml_path)
            .context("Failed to read form XML file")?;
            
        let mut reader = Reader::from_str(&content);
        reader.trim_text(true);
        
        let qualified_name = format!("{}.Form.{}", parent_name, form_name);
        let mut form_entity = BslEntity {
            id: BslEntityId(qualified_name.clone()),
            qualified_name: qualified_name.clone(),
            display_name: form_name.to_string(),
            english_name: None,
            entity_type: BslEntityType::Form,
            entity_kind: BslEntityKind::ManagedForm,
            source: BslEntitySource::FormXml { path: String::new() },
            interface: BslInterface::default(),
            constraints: BslConstraints::default(),
            relationships: BslRelationships::default(),
            documentation: None,
            availability: vec![],
            lifecycle: BslLifecycle {
                introduced_version: None,
                deprecated_version: None,
                removed_version: None,
                replacement: None,
            },
            extended_data: serde_json::Map::new(),
        };
        
        form_entity.source = BslEntitySource::FormXml { 
            path: form_xml_path.to_string_lossy().to_string() 
        };
        form_entity.relationships.owner = Some(parent_name.to_string());
        
        let mut buf = Vec::new();
        let mut _in_form = false;
        let mut _in_items = false;
        let mut _current_item_name = String::new();
        let mut in_commands = false;
        let mut form_commands = Vec::new();
        let mut form_items = Vec::new();
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    
                    match tag_name.as_str() {
                        "Form" => {
                            _in_form = true;
                            // Извлекаем атрибуты формы
                            for attr in e.attributes() {
                                if let Ok(attr) = attr {
                                    let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                                    let value = String::from_utf8_lossy(&attr.value).to_string();
                                    
                                    // Сохраняем важные атрибуты как свойства
                                    if key == "xmlns" || key.starts_with("xmlns:") {
                                        continue; // Пропускаем namespace декларации
                                    }
                                    
                                    let prop = BslProperty {
                                        name: key.clone(),
                                        english_name: None,
                                        type_name: "String".to_string(),
                                        is_readonly: true,
                                        is_indexed: false,
                                        documentation: Some(value.clone()),
                                        availability: vec![BslContext::Client, BslContext::Server],
                                    };
                                    form_entity.interface.properties.insert(key, prop);
                                }
                            }
                        }
                        "Items" => _in_items = true,
                        "Table" | "CommandBar" | "Button" | "InputField" | "Field" | "UsualGroup" | "Group" => {
                            // Парсим элементы формы
                            for attr in e.attributes() {
                                if let Ok(attr) = attr {
                                    let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                                    let value = String::from_utf8_lossy(&attr.value).to_string();
                                    
                                    if key == "name" {
                                        _current_item_name = value.clone();
                                        form_items.push(format!("{} ({})", value, tag_name));
                                    }
                                }
                            }
                        }
                        "Command" if in_commands => {
                            // Парсим команды формы внутри Commands
                            for attr in e.attributes() {
                                if let Ok(attr) = attr {
                                    let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                                    let value = String::from_utf8_lossy(&attr.value).to_string();
                                    
                                    if key == "name" {
                                        form_commands.push(value);
                                    }
                                }
                            }
                        }
                        "Commands" => {
                            // Входим в секцию команд
                            in_commands = true;
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) => {
                    let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    match tag_name.as_str() {
                        "Form" => _in_form = false,
                        "Items" => _in_items = false,
                        "Commands" => in_commands = false,
                        _ => {}
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(anyhow::anyhow!("Form XML parsing error: {}", e)),
                _ => {}
            }
            buf.clear();
        }
        
        // Добавляем команды как методы формы
        for command in form_commands {
            let method = super::entity::BslMethod {
                name: command.clone(),
                english_name: None,
                parameters: vec![],
                return_type: None,
                documentation: Some(format!("Команда формы {}", command)),
                availability: vec![BslContext::Client],
                is_function: false,
                is_export: false,
                is_deprecated: false,
                deprecation_info: None,
            };
            form_entity.interface.methods.insert(command, method);
        }
        
        // Сохраняем информацию об элементах управления в extended_data
        if !form_items.is_empty() {
            form_entity.extended_data.insert(
                "form_items".to_string(), 
                serde_json::Value::Array(
                    form_items.into_iter()
                        .map(serde_json::Value::String)
                        .collect()
                )
            );
        }
        
        Ok(form_entity)
    }
}

#[cfg(test)]
#[path = "xml_parser_test.rs"]
mod xml_parser_test;