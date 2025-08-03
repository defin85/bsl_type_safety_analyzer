use std::path::Path;
use anyhow::{Result, Context};
use log::info;

use super::index::UnifiedBslIndex;
use super::entity::{BslEntity, BslProperty, BslApplicationMode};
use super::platform_cache::PlatformDocsCache;
use super::project_cache::ProjectIndexCache;
use super::xml_parser::ConfigurationXmlParser;

pub struct UnifiedIndexBuilder {
    platform_cache: PlatformDocsCache,
    project_cache: ProjectIndexCache,
    application_mode: BslApplicationMode,
}

impl UnifiedIndexBuilder {
    pub fn new() -> Result<Self> {
        Ok(Self {
            platform_cache: PlatformDocsCache::new()?,
            project_cache: ProjectIndexCache::new()?,
            application_mode: BslApplicationMode::ManagedApplication, // по умолчанию управляемый режим
        })
    }
    
    pub fn with_application_mode(mut self, mode: BslApplicationMode) -> Self {
        self.application_mode = mode;
        self
    }
    
    pub fn build_index(
        &self, 
        config_path: impl AsRef<Path>,
        platform_version: &str
    ) -> Result<UnifiedBslIndex> {
        let config_path = config_path.as_ref();
        
        info!("Building unified BSL index for: {}", config_path.display());
        info!("DEBUG: About to call project_cache.get_or_create");
        
        // Используем project cache для автоматического кеширования
        let index = self.project_cache.get_or_create(
            config_path,
            platform_version,
            &|| self.build_index_from_scratch(config_path, platform_version)
        )?;
        
        // Примитивные типы теперь добавляются в platform_cache автоматически
        
        Ok(index)
    }
    
    fn build_index_from_scratch(
        &self,
        config_path: &Path,
        platform_version: &str
    ) -> Result<UnifiedBslIndex> {
        info!("Building index from scratch for: {}", config_path.display());
        info!("Application mode: {:?}", self.application_mode);
        
        let mut index = UnifiedBslIndex::with_application_mode(self.application_mode);
        
        // 1. Загружаем платформенные типы
        let start = std::time::Instant::now();
        let platform_entities = self.platform_cache.get_or_create(platform_version)
            .context("Failed to load platform types")?;
        info!("Platform types: {} (loaded in {:?})", platform_entities.len(), start.elapsed());
        
        // Добавляем платформенные типы в индекс
        for entity in platform_entities {
            index.add_entity(entity)?;
        }
        
        // 2. Парсим конфигурацию
        let start = std::time::Instant::now();
        let xml_parser = ConfigurationXmlParser::new(config_path);
        let config_entities = xml_parser.parse_configuration()
            .context("Failed to parse configuration")?;
        info!("Configuration objects: {} (parsed in {:?})", config_entities.len(), start.elapsed());
        
        // Добавляем объекты конфигурации в индекс
        for entity in config_entities {
            index.add_entity(entity)?;
        }
        
        // 3. Загружаем данные из существующих парсеров (если есть)
        self.load_legacy_data(&mut index, config_path)?;
        
        // 4. Строим граф наследования
        let start = std::time::Instant::now();
        index.build_inheritance_relationships()?;
        
        // 5. Инициализируем глобальные алиасы 1С
        index.initialize_global_aliases()?;
        
        info!("✅ Index built successfully: {} entities (total time: {:?})", 
            index.get_entity_count(), start.elapsed());
        
        Ok(index)
    }
    
    fn load_legacy_data(&self, index: &mut UnifiedBslIndex, _config_path: &Path) -> Result<()> {
        // Проверяем наличие данных от legacy парсеров
        let output_dir = std::path::PathBuf::from("output/hybrid_docs");
        
        if output_dir.exists() {
            // Загружаем metadata_types
            let metadata_types_dir = output_dir.join("configuration/metadata_types");
            if metadata_types_dir.exists() {
                let mut count = 0;
                for entry in std::fs::read_dir(&metadata_types_dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("json") {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if let Ok(legacy_data) = serde_json::from_str::<serde_json::Value>(&content) {
                                if let Ok(entity) = self.convert_legacy_metadata(&legacy_data) {
                                    index.add_entity(entity)?;
                                    count += 1;
                                }
                            }
                        }
                    }
                }
                if count > 0 {
                    info!("Legacy metadata: {} entities", count);
                }
            }
            
            // Загружаем формы
            let forms_dir = output_dir.join("configuration/forms");
            if forms_dir.exists() {
                let mut count = 0;
                for entry in walkdir::WalkDir::new(&forms_dir)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_type().is_file())
                    .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("json"))
                {
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        if let Ok(legacy_data) = serde_json::from_str::<serde_json::Value>(&content) {
                            if let Ok(entity) = self.convert_legacy_form(&legacy_data) {
                                index.add_entity(entity)?;
                                count += 1;
                            }
                        }
                    }
                }
                if count > 0 {
                    info!("Legacy forms: {} entities", count);
                }
            }
        }
        
        Ok(())
    }
    
    fn convert_legacy_metadata(&self, legacy_data: &serde_json::Value) -> Result<BslEntity> {
        use super::entity::*;
        
        let name = legacy_data.get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing name in legacy data"))?;
            
        let type_str = legacy_data.get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
            
        let kind = match type_str {
            "catalog" => BslEntityKind::Catalog,
            "document" => BslEntityKind::Document,
            "information_register" => BslEntityKind::InformationRegister,
            "accumulation_register" => BslEntityKind::AccumulationRegister,
            _ => BslEntityKind::Other(type_str.to_string()),
        };
        
        let mut entity = BslEntity::new(
            name.to_string(),
            name.to_string(),
            BslEntityType::Configuration,
            kind
        );
        
        // Конвертируем атрибуты в свойства
        if let Some(attributes) = legacy_data.get("attributes").and_then(|v| v.as_array()) {
            for attr in attributes {
                if let Ok(property) = self.convert_legacy_attribute(attr) {
                    entity.interface.properties.insert(property.name.clone(), property);
                }
            }
        }
        
        // Конвертируем табличные части
        if let Some(tabular_sections) = legacy_data.get("tabular_sections").and_then(|v| v.as_array()) {
            for ts in tabular_sections {
                if let Some(ts_name) = ts.get("name").and_then(|v| v.as_str()) {
                    // Создаем упрощенную табличную часть без атрибутов для legacy данных
                    let tabular_section = super::entity::BslTabularSection {
                        name: ts_name.to_string(),
                        display_name: ts_name.to_string(),
                        attributes: Vec::new(),
                    };
                    entity.relationships.tabular_sections.push(tabular_section);
                }
            }
        }
        
        entity.source = BslEntitySource::TextReport { path: "legacy_import".to_string() };
        
        Ok(entity)
    }
    
    fn convert_legacy_attribute(&self, attr_data: &serde_json::Value) -> Result<BslProperty> {
        use super::entity::*;
        
        let name = attr_data.get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing attribute name"))?;
            
        let type_name = attr_data.get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();
            
        Ok(BslProperty {
            name: name.to_string(),
            english_name: None,
            type_name,
            is_readonly: false,
            is_indexed: attr_data.get("indexed").and_then(|v| v.as_bool()).unwrap_or(false),
            documentation: attr_data.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()),
            availability: vec![BslContext::Server, BslContext::Client],
        })
    }
    
    fn convert_legacy_form(&self, form_data: &serde_json::Value) -> Result<BslEntity> {
        use super::entity::*;
        
        let name = form_data.get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing form name"))?;
            
        let parent = form_data.get("parent")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");
            
        let form_type = form_data.get("form_type")
            .and_then(|v| v.as_str())
            .unwrap_or("ManagedForm");
            
        let kind = match form_type {
            "ManagedForm" => BslEntityKind::ManagedForm,
            "OrdinaryForm" => BslEntityKind::OrdinaryForm,
            _ => BslEntityKind::Form,
        };
        
        let mut entity = BslEntity::new(
            format!("{}.{}", parent, name),
            format!("{}.{}", parent, name),
            BslEntityType::Form,
            kind
        );
        
        entity.relationships.owner = Some(parent.to_string());
        entity.source = BslEntitySource::FormXml { path: "legacy_import".to_string() };
        
        // Конвертируем элементы формы в свойства
        if let Some(elements) = form_data.get("elements").and_then(|v| v.as_array()) {
            for element in elements {
                if let Some(elem_name) = element.get("name").and_then(|v| v.as_str()) {
                    let elem_type = element.get("type").and_then(|v| v.as_str()).unwrap_or("Unknown");
                    
                    let property = BslProperty {
                        name: elem_name.to_string(),
                        english_name: None,
                        type_name: elem_type.to_string(),
                        is_readonly: true,
                        is_indexed: false,
                        documentation: None,
                        availability: vec![BslContext::Client],
                    };
                    
                    entity.interface.properties.insert(elem_name.to_string(), property);
                }
            }
        }
        
        Ok(entity)
    }
    
}