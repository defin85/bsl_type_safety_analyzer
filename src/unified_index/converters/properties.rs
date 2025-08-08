//! Конвертер для свойств

use crate::unified_index::entity::{BslProperty, BslContext};

/// Конвертер для свойств
pub struct PropertyConverter;

impl PropertyConverter {
    /// Конвертирует данные свойства из JSON в BslProperty
    pub fn from_json(prop_data: &serde_json::Value) -> anyhow::Result<BslProperty> {
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
                .filter_map(|s| Self::parse_context(s))
                .collect();
        }
        
        Ok(property)
    }
    
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