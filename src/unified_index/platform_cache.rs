use std::path::{Path, PathBuf};
use std::fs;
use std::io::{BufReader, BufWriter, BufRead, Write};
use anyhow::{Result, Context};
use serde_json;

use super::entity::{BslEntity, BslEntityKind, BslEntityType, BslEntitySource, BslMethod, BslProperty, BslParameter, BslContext};

pub struct PlatformDocsCache {
    cache_dir: PathBuf,
}

impl PlatformDocsCache {
    pub fn new() -> Result<Self> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
        let cache_dir = home_dir.join(".bsl_analyzer").join("platform_cache");
        
        fs::create_dir_all(&cache_dir)
            .context("Failed to create platform cache directory")?;
            
        Ok(Self { cache_dir })
    }
    
    pub fn get_or_create(&self, version: &str) -> Result<Vec<BslEntity>> {
        let cache_file = self.get_cache_file_path(version);
        
        if cache_file.exists() {
            self.load_from_cache(&cache_file)
        } else {
            // Если кеша нет, пытаемся извлечь из существующего hybrid storage
            self.extract_from_hybrid_storage(version)
        }
    }
    
    pub fn save_to_cache(&self, version: &str, entities: &[BslEntity]) -> Result<()> {
        let cache_file = self.get_cache_file_path(version);
        let file = fs::File::create(&cache_file)
            .context("Failed to create cache file")?;
        let mut writer = BufWriter::new(file);
        
        for entity in entities {
            let json = serde_json::to_string(entity)?;
            writeln!(writer, "{}", json)?;
        }
        
        writer.flush()?;
        Ok(())
    }
    
    fn get_cache_file_path(&self, version: &str) -> PathBuf {
        // Нормализуем версию - убираем префикс "v" если есть
        let normalized_version = version.strip_prefix("v").unwrap_or(version);
        self.cache_dir.join(format!("{}.jsonl", normalized_version))
    }
    
    fn load_from_cache(&self, cache_file: &Path) -> Result<Vec<BslEntity>> {
        let file = fs::File::open(cache_file)
            .context("Failed to open cache file")?;
        let reader = BufReader::new(file);
        let mut entities = Vec::new();
        
        for line in reader.lines() {
            let line = line?;
            if !line.trim().is_empty() {
                let entity: BslEntity = serde_json::from_str(&line)
                    .context("Failed to deserialize entity")?;
                entities.push(entity);
            }
        }
        
        Ok(entities)
    }
    
    fn extract_from_hybrid_storage(&self, version: &str) -> Result<Vec<BslEntity>> {
        // Попытка загрузить из существующего hybrid storage
        let hybrid_path = PathBuf::from("output/hybrid_docs");
        if !hybrid_path.exists() {
            return Err(anyhow::anyhow!(
                "Platform types for version {} not found. Please extract platform docs first using:\n\
                cargo run --bin extract_platform_docs -- --archive \"path/to/1c_{}.zip\" --version \"{}\"",
                version, version, version
            ));
        }
        
        let mut entities = Vec::new();
        
        // Загружаем типы из всех категорий
        let categories = ["collections", "database", "forms", "io", "system", "web"];
        for category in &categories {
            let category_file = hybrid_path
                .join("core")
                .join("builtin_types")
                .join(format!("{}.json", category));
                
            if category_file.exists() {
                let content = fs::read_to_string(&category_file)?;
                let category_data: serde_json::Value = serde_json::from_str(&content)?;
                
                if let Some(types) = category_data.get("types").and_then(|v| v.as_object()) {
                    for (_type_name, type_data) in types {
                        // Конвертируем из формата hybrid storage в BslEntity
                        if let Ok(entity) = self.convert_hybrid_to_entity(type_data, version) {
                            entities.push(entity);
                        }
                    }
                }
            }
        }
        
        // Сохраняем в кеш для будущего использования
        if !entities.is_empty() {
            self.save_to_cache(version, &entities)?;
        }
        
        Ok(entities)
    }
    
    fn convert_hybrid_to_entity(&self, hybrid_data: &serde_json::Value, version: &str) -> Result<BslEntity> {
        let name = hybrid_data.get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing name field"))?;
            
        let english_name = hybrid_data.get("english_name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
            
        let mut entity = BslEntity::new(
            name.to_string(),
            name.to_string(),
            BslEntityType::Platform,
            self.determine_entity_kind(name)
        );
        
        entity.english_name = english_name;
        entity.source = BslEntitySource::HBK { version: version.to_string() };
        
        // Конвертируем методы
        if let Some(methods) = hybrid_data.get("methods").and_then(|v| v.as_array()) {
            for method_data in methods {
                if let Ok(method) = self.convert_method(method_data) {
                    entity.interface.methods.insert(method.name.clone(), method);
                }
            }
        }
        
        // Конвертируем свойства
        if let Some(properties) = hybrid_data.get("properties").and_then(|v| v.as_array()) {
            for prop_data in properties {
                if let Ok(property) = self.convert_property(prop_data) {
                    entity.interface.properties.insert(property.name.clone(), property);
                }
            }
        }
        
        // Устанавливаем доступность
        if let Some(contexts) = hybrid_data.get("availability").and_then(|v| v.as_array()) {
            entity.availability = contexts.iter()
                .filter_map(|v| v.as_str())
                .filter_map(|s| self.parse_context(s))
                .collect();
        }
        
        Ok(entity)
    }
    
    fn determine_entity_kind(&self, name: &str) -> BslEntityKind {
        match name {
            "Массив" | "Array" => BslEntityKind::Array,
            "Структура" | "Structure" => BslEntityKind::Structure,
            "Соответствие" | "Map" => BslEntityKind::Map,
            "СписокЗначений" | "ValueList" => BslEntityKind::ValueList,
            "ТаблицаЗначений" | "ValueTable" => BslEntityKind::ValueTable,
            "ДеревоЗначений" | "ValueTree" => BslEntityKind::ValueTree,
            "Число" | "Number" | "Строка" | "String" | "Булево" | "Boolean" | "Дата" | "Date" => BslEntityKind::Primitive,
            _ => BslEntityKind::System,
        }
    }
    
    fn convert_method(&self, method_data: &serde_json::Value) -> Result<BslMethod> {
        let name = method_data.get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing method name"))?;
            
        let mut method = BslMethod {
            name: name.to_string(),
            english_name: method_data.get("english_name").and_then(|v| v.as_str()).map(|s| s.to_string()),
            parameters: Vec::new(),
            return_type: method_data.get("return_type").and_then(|v| v.as_str()).map(|s| s.to_string()),
            documentation: method_data.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()),
            availability: Vec::new(),
            is_function: method_data.get("is_function").and_then(|v| v.as_bool()).unwrap_or(false),
            is_export: false,
            is_deprecated: method_data.get("is_deprecated").and_then(|v| v.as_bool()).unwrap_or(false),
            deprecation_info: method_data.get("deprecation_info").and_then(|v| v.as_str()).map(|s| s.to_string()),
        };
        
        // Конвертируем параметры
        if let Some(params) = method_data.get("parameters").and_then(|v| v.as_array()) {
            for param_data in params {
                if let Ok(param) = self.convert_parameter(param_data) {
                    method.parameters.push(param);
                }
            }
        }
        
        // Устанавливаем доступность
        if let Some(contexts) = method_data.get("availability").and_then(|v| v.as_array()) {
            method.availability = contexts.iter()
                .filter_map(|v| v.as_str())
                .filter_map(|s| self.parse_context(s))
                .collect();
        }
        
        Ok(method)
    }
    
    fn convert_property(&self, prop_data: &serde_json::Value) -> Result<BslProperty> {
        let name = prop_data.get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing property name"))?;
            
        let mut property = BslProperty {
            name: name.to_string(),
            english_name: prop_data.get("english_name").and_then(|v| v.as_str()).map(|s| s.to_string()),
            type_name: prop_data.get("type").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
            is_readonly: prop_data.get("is_readonly").and_then(|v| v.as_bool()).unwrap_or(false),
            is_indexed: false,
            documentation: prop_data.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()),
            availability: Vec::new(),
        };
        
        // Устанавливаем доступность
        if let Some(contexts) = prop_data.get("availability").and_then(|v| v.as_array()) {
            property.availability = contexts.iter()
                .filter_map(|v| v.as_str())
                .filter_map(|s| self.parse_context(s))
                .collect();
        }
        
        Ok(property)
    }
    
    fn convert_parameter(&self, param_data: &serde_json::Value) -> Result<BslParameter> {
        let name = param_data.get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing parameter name"))?;
            
        Ok(BslParameter {
            name: name.to_string(),
            type_name: param_data.get("type").and_then(|v| v.as_str()).map(|s| s.to_string()),
            is_optional: param_data.get("is_optional").and_then(|v| v.as_bool()).unwrap_or(false),
            default_value: param_data.get("default_value").and_then(|v| v.as_str()).map(|s| s.to_string()),
            description: param_data.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()),
        })
    }
    
    fn parse_context(&self, context_str: &str) -> Option<BslContext> {
        match context_str {
            "Client" | "Клиент" => Some(BslContext::Client),
            "Server" | "Сервер" => Some(BslContext::Server),
            "ExternalConnection" | "ВнешнееСоединение" => Some(BslContext::ExternalConnection),
            "MobileApp" | "МобильноеПриложение" => Some(BslContext::MobileApp),
            "MobileClient" | "МобильныйКлиент" => Some(BslContext::MobileClient),
            "MobileServer" | "МобильныйСервер" => Some(BslContext::MobileServer),
            "ThickClient" | "ТолстыйКлиент" => Some(BslContext::ThickClient),
            "ThinClient" | "ТонкийКлиент" => Some(BslContext::ThinClient),
            "WebClient" | "ВебКлиент" => Some(BslContext::WebClient),
            _ => None,
        }
    }
}