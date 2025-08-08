//! Конвертер для преобразования BslSyntaxDatabase в BslEntity

use crate::docs_integration::{BslSyntaxDatabase, BslMethodInfo, BslFunctionInfo};
use crate::unified_index::entity::{
    BslEntity, BslEntityType, BslEntityKind, BslEntitySource,
    BslMethod, BslProperty, BslParameter, BslContext
};
use std::collections::HashMap;
use anyhow::Result;

/// Конвертер для преобразования синтаксической БД в унифицированные сущности
pub struct SyntaxDbConverter {
    version: String,
}

impl SyntaxDbConverter {
    /// Создает новый конвертер для указанной версии платформы
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            version: version.into(),
        }
    }
    
    /// Конвертирует BslSyntaxDatabase в вектор BslEntity
    /// 
    /// ВАЖНО: Эта функция исправляет критический баг с перезаписью сущностей.
    /// Все объекты добавляются напрямую в вектор, а не через HashMap.
    pub fn convert(&self, syntax_db: &BslSyntaxDatabase) -> Result<Vec<BslEntity>> {
        let mut entities = Vec::new();
        let mut entity_map: HashMap<String, BslEntity> = HashMap::new();
        
        // Создаем сущности для всех объектов
        for (name, obj) in &syntax_db.objects {
            let mut entity = BslEntity::new(
                name.clone(),
                name.clone(),
                BslEntityType::Platform,
                Self::determine_entity_kind(name)
            );
            
            entity.source = BslEntitySource::HBK { version: self.version.clone() };
            entity.documentation = obj.description.clone();
            
            // Конвертируем availability
            if let Some(availability_str) = &obj.availability {
                entity.availability = availability_str
                    .split(',')
                    .filter_map(|ctx| Self::parse_context(ctx.trim()))
                    .collect();
            }
            
            // ИСПРАВЛЕНИЕ: Добавляем объекты напрямую в entities, чтобы избежать перезаписи
            entities.push(entity.clone());
            // Но сохраняем в entity_map для связывания методов
            entity_map.insert(name.clone(), entity);
        }
        
        // Добавляем методы к объектам
        for (method_name, method_info) in &syntax_db.methods {
            if let Some(object_name) = &method_info.object_context {
                let entity_key = if entity_map.contains_key(object_name) {
                    Some(object_name.clone())
                } else {
                    entity_map.keys()
                        .find(|key| key.starts_with(object_name))
                        .cloned()
                };
                
                if let Some(key) = entity_key {
                    if let Some(entity) = entity_map.get_mut(&key) {
                        let bsl_method = Self::convert_method_info(method_info);
                        entity.interface.methods.insert(method_name.clone(), bsl_method);
                    }
                }
            } else {
                // Глобальный метод
                let global_entity = entity_map.entry("Global".to_string()).or_insert_with(|| {
                    let mut entity = BslEntity::new(
                        "Global".to_string(),
                        "Global".to_string(),
                        BslEntityType::Platform,
                        BslEntityKind::Global
                    );
                    entity.source = BslEntitySource::HBK { version: self.version.clone() };
                    entity
                });
                
                let bsl_method = Self::convert_method_info(method_info);
                global_entity.interface.methods.insert(method_name.clone(), bsl_method);
            }
        }
        
        // Добавляем свойства к объектам
        for (prop_name, prop_info) in &syntax_db.properties {
            if let Some(object_name) = &prop_info.object_context {
                if let Some(entity) = entity_map.get_mut(object_name) {
                    let bsl_property = BslProperty {
                        name: prop_info.name.clone(),
                        english_name: None, // BslPropertyInfo не имеет english_name
                        type_name: prop_info.property_type.clone(),
                        is_readonly: matches!(prop_info.access_mode, crate::docs_integration::AccessMode::Read),
                        is_indexed: false,
                        documentation: prop_info.description.clone(),
                        availability: if let Some(availability_str) = &prop_info.availability {
                            availability_str
                                .split(',')
                                .filter_map(|ctx| Self::parse_context(ctx.trim()))
                                .collect()
                        } else {
                            vec![]
                        },
                    };
                    entity.interface.properties.insert(prop_name.clone(), bsl_property);
                }
            }
        }
        
        // Конвертируем функции в методы Global entity
        for (func_name, func_info) in &syntax_db.functions {
            let global_entity = entity_map.entry("Global".to_string()).or_insert_with(|| {
                let mut entity = BslEntity::new(
                    "Global".to_string(),
                    "Global".to_string(),
                    BslEntityType::Platform,
                    BslEntityKind::Global
                );
                entity.source = BslEntitySource::HBK { version: self.version.clone() };
                entity
            });
            
            let bsl_method = Self::convert_function_info(func_info);
            global_entity.interface.methods.insert(func_name.clone(), bsl_method);
        }
        
        // ИСПРАВЛЕНИЕ: Добавляем только уникальные сущности из entity_map
        // (Global entity и другие, которые не были добавлены выше)
        for entity in entity_map.into_values() {
            if !entities.iter().any(|e| e.id == entity.id) {
                entities.push(entity);
            }
        }
        
        log::info!("Converted {} entities from syntax database", entities.len());
        Ok(entities)
    }
    
    /// Конвертирует BslMethodInfo в BslMethod
    fn convert_method_info(method_info: &BslMethodInfo) -> BslMethod {
        BslMethod {
            name: method_info.name.clone(),
            english_name: method_info.english_name.clone(),
            parameters: method_info.parameters.iter()
                .map(|p| BslParameter {
                    name: p.name.clone(),
                    type_name: p.param_type.clone(),
                    is_optional: p.is_optional,
                    default_value: p.default_value.clone(),
                    description: p.description.clone(),
                })
                .collect(),
            return_type: method_info.return_type.clone(),
            documentation: method_info.description.clone(),
            availability: method_info.availability.iter()
                .filter_map(|ctx| Self::parse_context(ctx))
                .collect(),
            is_function: method_info.return_type.is_some(),
            is_export: false,
            is_deprecated: false,
            deprecation_info: None,
        }
    }
    
    /// Конвертирует BslFunctionInfo в BslMethod
    fn convert_function_info(func_info: &BslFunctionInfo) -> BslMethod {
        BslMethod {
            name: func_info.name.clone(),
            english_name: None,
            parameters: func_info.parameters.iter()
                .map(|p| BslParameter {
                    name: p.name.clone(),
                    type_name: p.param_type.clone(),
                    is_optional: p.is_optional,
                    default_value: p.default_value.clone(),
                    description: p.description.clone(),
                })
                .collect(),
            return_type: func_info.return_type.clone(),
            documentation: func_info.description.clone(),
            availability: if let Some(availability_str) = &func_info.availability {
                availability_str
                    .split(',')
                    .filter_map(|ctx| Self::parse_context(ctx.trim()))
                    .collect()
            } else {
                vec![]
            },
            is_function: true,
            is_export: false,
            is_deprecated: false,
            deprecation_info: None,
        }
    }
    
    /// Определяет тип сущности по имени
    fn determine_entity_kind(name: &str) -> BslEntityKind {
        match name {
            "Массив" | "Array" => BslEntityKind::Array,
            "Структура" | "Structure" => BslEntityKind::Structure,
            "Соответствие" | "Map" => BslEntityKind::Map,
            "СписокЗначений" | "ValueList" => BslEntityKind::ValueList,
            "ТаблицаЗначений" | "ValueTable" => BslEntityKind::ValueTable,
            "ДеревоЗначений" | "ValueTree" => BslEntityKind::ValueTree,
            "Число" | "Number" | "Строка" | "String" | 
            "Булево" | "Boolean" | "Дата" | "Date" => BslEntityKind::Primitive,
            _ => BslEntityKind::System,
        }
    }
    
    /// Парсит строку контекста в BslContext
    fn parse_context(context_str: &str) -> Option<BslContext> {
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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_converter_creation() {
        let converter = SyntaxDbConverter::new("8.3.25");
        assert_eq!(converter.version, "8.3.25");
    }
    
    #[test]
    fn test_entity_kind_determination() {
        assert_eq!(SyntaxDbConverter::determine_entity_kind("Массив"), BslEntityKind::Array);
        assert_eq!(SyntaxDbConverter::determine_entity_kind("Array"), BslEntityKind::Array);
        assert_eq!(SyntaxDbConverter::determine_entity_kind("Строка"), BslEntityKind::Primitive);
        assert_eq!(SyntaxDbConverter::determine_entity_kind("Unknown"), BslEntityKind::System);
    }
    
    #[test]
    fn test_context_parsing() {
        assert_eq!(SyntaxDbConverter::parse_context("Client"), Some(BslContext::Client));
        assert_eq!(SyntaxDbConverter::parse_context("Сервер"), Some(BslContext::Server));
        assert_eq!(SyntaxDbConverter::parse_context("Unknown"), None);
    }
}