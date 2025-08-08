//! Конвертер для методов

use crate::unified_index::entity::{BslMethod, BslParameter, BslContext};

/// Конвертер для методов
pub struct MethodConverter;

impl MethodConverter {
    /// Конвертирует данные метода из JSON в BslMethod
    pub fn from_json(method_data: &serde_json::Value) -> anyhow::Result<BslMethod> {
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
                if let Ok(param) = Self::convert_parameter(param_data) {
                    method.parameters.push(param);
                }
            }
        }
        
        // Устанавливаем доступность
        if let Some(contexts) = method_data.get("availability").and_then(|v| v.as_array()) {
            method.availability = contexts.iter()
                .filter_map(|v| v.as_str())
                .filter_map(|s| Self::parse_context(s))
                .collect();
        }
        
        Ok(method)
    }
    
    fn convert_parameter(param_data: &serde_json::Value) -> anyhow::Result<BslParameter> {
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