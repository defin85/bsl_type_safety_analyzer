/// <module>
///   <name>tools</name>
///   <purpose>Реализация MCP инструментов для работы с BSL типами</purpose>
/// </module>
use crate::mcp_server::analyzer::BslTypeAnalyzer;
use crate::mcp_server::types::BslLanguagePreference;
use crate::unified_index::index::BslLanguagePreference as IndexLanguagePreference;
use crate::unified_index::{BslContext, BslEntity, BslMethod};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// <type>
///   <name>FindTypeResult</name>
///   <purpose>Результат поиска типа</purpose>
/// </type>
#[derive(Debug, Serialize, Deserialize)]
pub struct FindTypeResult {
    pub found: bool,
    pub entity: Option<EntityInfo>,
    pub suggestions: Vec<String>,
}

/// <type>
///   <name>EntityInfo</name>
///   <purpose>Информация о найденной сущности</purpose>
/// </type>
#[derive(Debug, Serialize, Deserialize)]
pub struct EntityInfo {
    pub id: String,
    pub display_name: String,
    pub entity_type: String,
    pub entity_kind: String,
    pub methods_count: usize,
    pub properties_count: usize,
    pub parent_types: Vec<String>,
    pub implements: Vec<String>,
}

/// <type>
///   <name>MethodInfo</name>
///   <purpose>Информация о методе</purpose>
/// </type>
#[derive(Debug, Serialize, Deserialize)]
pub struct MethodInfo {
    pub name: String,
    pub english_name: Option<String>,
    pub is_function: bool,
    pub parameters: Vec<ParameterInfo>,
    pub return_type: Option<String>,
    pub availability: Vec<String>,
    pub inherited_from: Option<String>,
    pub is_deprecated: bool,
    pub documentation: Option<String>,
}

/// <type>
///   <name>ParameterInfo</name>
///   <purpose>Информация о параметре метода</purpose>
/// </type>
#[derive(Debug, Serialize, Deserialize)]
pub struct ParameterInfo {
    pub name: String,
    pub type_name: Option<String>,
    pub is_optional: bool,
    pub default_value: Option<String>,
    pub description: Option<String>,
}

/// <type>
///   <name>TypeCompatibilityResult</name>
///   <purpose>Результат проверки совместимости типов</purpose>
/// </type>
#[derive(Debug, Serialize, Deserialize)]
pub struct TypeCompatibilityResult {
    pub compatible: bool,
    pub reason: String,
    pub path: Vec<String>,
}

/// <type>
///   <name>ValidationResult</name>
///   <purpose>Результат валидации вызова метода</purpose>
/// </type>
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub method: Option<MethodInfo>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl From<&BslEntity> for EntityInfo {
    fn from(entity: &BslEntity) -> Self {
        EntityInfo {
            id: entity.id.0.clone(),
            display_name: entity.display_name.clone(),
            entity_type: format!("{:?}", entity.entity_type),
            entity_kind: format!("{:?}", entity.entity_kind),
            methods_count: entity.interface.methods.len(),
            properties_count: entity.interface.properties.len(),
            parent_types: entity.constraints.parent_types.clone(),
            implements: entity.constraints.implements.clone(),
        }
    }
}

impl From<&BslMethod> for MethodInfo {
    fn from(method: &BslMethod) -> Self {
        MethodInfo {
            name: method.name.clone(),
            english_name: method.english_name.clone(),
            is_function: method.is_function,
            parameters: method
                .parameters
                .iter()
                .map(|p| ParameterInfo {
                    name: p.name.clone(),
                    type_name: p.type_name.clone(),
                    is_optional: p.is_optional,
                    default_value: p.default_value.clone(),
                    description: p.description.clone(),
                })
                .collect(),
            return_type: method.return_type.clone(),
            availability: method
                .availability
                .iter()
                .map(|c| format!("{:?}", c))
                .collect(),
            inherited_from: None,
            is_deprecated: method.is_deprecated,
            documentation: method.documentation.clone(),
        }
    }
}

// Implementation functions for MCP tools

pub async fn find_type_impl(
    analyzer: &BslTypeAnalyzer,
    type_name: String,
    language_preference: Option<String>,
) -> String {
    let _ = analyzer.ensure_index().await;

    let guard = analyzer.get_index().read().await;
    let index = match &*guard {
        Some(idx) => idx,
        None => return serde_json::to_string(&FindTypeResult {
            found: false,
            entity: None,
            suggestions: vec!["Индекс не загружен. Используйте load_configuration.".to_string()],
        })
        .unwrap_or_default(),
    };

    let preference = BslLanguagePreference::from(language_preference);
    let index_preference = match preference {
        BslLanguagePreference::Russian => IndexLanguagePreference::Russian,
        BslLanguagePreference::English => IndexLanguagePreference::English,
        BslLanguagePreference::Auto => IndexLanguagePreference::Auto,
    };

    let result =
        if let Some(entity) = index.find_entity_with_preference(&type_name, index_preference) {
            FindTypeResult {
                found: true,
                entity: Some(EntityInfo::from(entity)),
                suggestions: vec![],
            }
        } else {
            // Получаем похожие имена
            let suggestions = index.suggest_similar_names(&type_name);
            FindTypeResult {
                found: false,
                entity: None,
                suggestions: suggestions.into_iter().take(5).collect(),
            }
        };

    serde_json::to_string(&result)
        .unwrap_or_else(|e| format!("{{\"error\": \"Serialization failed: {}\"}}", e))
}

pub async fn get_type_methods_impl(
    analyzer: &BslTypeAnalyzer,
    type_name: String,
    include_inherited: Option<bool>,
    filter_context: Option<String>,
) -> String {
    let _ = analyzer.ensure_index().await;

    let guard = analyzer.get_index().read().await;
    let index = match &*guard {
        Some(idx) => idx,
        None => return "{\"error\": \"Index not loaded\"}".to_string(),
    };

    let entity = match index.find_entity(&type_name) {
        Some(e) => e,
        None => return format!("{{\"error\": \"Type '{}' not found\"}}", type_name),
    };

    let include_inherited = include_inherited.unwrap_or(true);

    let methods: HashMap<String, BslMethod> = if include_inherited {
        index.get_all_methods(&type_name)
    } else {
        entity.interface.methods.clone()
    };

    // Фильтруем по контексту если указан
    let filtered_methods: Vec<MethodInfo> = methods
        .values()
        .filter(|method| {
            if let Some(ref ctx) = filter_context {
                match ctx.as_str() {
                    "Client" => method.availability.contains(&BslContext::Client),
                    "Server" => method.availability.contains(&BslContext::Server),
                    _ => true,
                }
            } else {
                true
            }
        })
        .map(MethodInfo::from)
        .collect();

    let result = serde_json::json!({
        "type_name": type_name,
        "total_methods": filtered_methods.len(),
        "own_methods": entity.interface.methods.len(),
        "inherited_methods": if include_inherited {
            filtered_methods.len() - entity.interface.methods.len()
        } else { 0 },
        "methods": filtered_methods
    });

    serde_json::to_string(&result)
        .unwrap_or_else(|e| format!("{{\"error\": \"Serialization failed: {}\"}}", e))
}

pub async fn check_type_compatibility_impl(
    analyzer: &BslTypeAnalyzer,
    from_type: String,
    to_type: String,
) -> String {
    let _ = analyzer.ensure_index().await;

    let guard = analyzer.get_index().read().await;
    let index = match &*guard {
        Some(idx) => idx,
        None => return "{\"error\": \"Index not loaded\"}".to_string(),
    };

    let compatible = index.is_assignable(&from_type, &to_type);

    let result = if compatible {
        // Пытаемся определить путь совместимости
        let from_entity = index.find_entity(&from_type);
        let reason = if from_type == to_type {
            "exact_match".to_string()
        } else if let Some(entity) = from_entity {
            if entity.constraints.parent_types.contains(&to_type) {
                "inheritance".to_string()
            } else if entity.constraints.implements.contains(&to_type) {
                "implements_interface".to_string()
            } else {
                "compatible".to_string()
            }
        } else {
            "unknown".to_string()
        };

        TypeCompatibilityResult {
            compatible: true,
            reason,
            path: vec![from_type.clone(), "→".to_string(), to_type.clone()],
        }
    } else {
        TypeCompatibilityResult {
            compatible: false,
            reason: "incompatible_types".to_string(),
            path: vec![],
        }
    };

    serde_json::to_string(&result)
        .unwrap_or_else(|e| format!("{{\"error\": \"Serialization failed: {}\"}}", e))
}

pub async fn validate_method_call_impl(
    analyzer: &BslTypeAnalyzer,
    object_type: String,
    method_name: String,
    context: Option<String>,
) -> String {
    let _ = analyzer.ensure_index().await;

    let guard = analyzer.get_index().read().await;
    let index = match &*guard {
        Some(idx) => idx,
        None => return "{\"error\": \"Index not loaded\"}".to_string(),
    };

    let _entity = match index.find_entity(&object_type) {
        Some(e) => e,
        None => {
            let result = ValidationResult {
                valid: false,
                method: None,
                errors: vec![format!("Type '{}' not found", object_type)],
                warnings: vec![],
            };
            return serde_json::to_string(&result).unwrap_or_default();
        }
    };

    let all_methods = index.get_all_methods(&object_type);
    let method = all_methods.get(&method_name);

    let context_enum = match context.as_deref() {
        Some("Client") => BslContext::Client,
        Some("Server") => BslContext::Server,
        _ => BslContext::Server,
    };

    let result = match method {
        Some(m) => {
            let mut errors = Vec::new();
            let mut warnings = Vec::new();

            // Проверяем доступность в контексте
            if !m.availability.is_empty() && !m.availability.contains(&context_enum) {
                errors.push(format!(
                    "Method '{}' is not available in {} context",
                    method_name,
                    context.as_deref().unwrap_or("Server")
                ));
            }

            // Проверяем deprecation
            if m.is_deprecated {
                warnings.push(format!(
                    "Method '{}' is deprecated{}",
                    method_name,
                    m.deprecation_info
                        .as_ref()
                        .map(|info| format!(": {}", info))
                        .unwrap_or_default()
                ));
            }

            ValidationResult {
                valid: errors.is_empty(),
                method: Some(MethodInfo::from(m)),
                errors,
                warnings,
            }
        }
        None => {
            // Пытаемся найти похожие методы
            let all_method_names: Vec<&str> = all_methods.keys().map(|s| s.as_str()).collect();
            let similar = find_similar_strings(&method_name, &all_method_names, 3);

            let mut errors = vec![format!(
                "Method '{}' not found for type '{}'",
                method_name, object_type
            )];
            if !similar.is_empty() {
                errors.push(format!("Did you mean: {}?", similar.join(", ")));
            }

            ValidationResult {
                valid: false,
                method: None,
                errors,
                warnings: vec![],
            }
        }
    };

    serde_json::to_string(&result)
        .unwrap_or_else(|e| format!("{{\"error\": \"Serialization failed: {}\"}}", e))
}

/// <function>
///   <name>find_similar_strings</name>
///   <purpose>Найти похожие строки используя расстояние Левенштейна</purpose>
/// </function>
fn find_similar_strings(target: &str, candidates: &[&str], max_results: usize) -> Vec<String> {
    let target_lower = target.to_lowercase();
    let mut similarities: Vec<(String, usize)> = candidates
        .iter()
        .map(|&s| {
            let distance = levenshtein_distance(&target_lower, &s.to_lowercase());
            (s.to_string(), distance)
        })
        .filter(|(_, dist)| *dist <= 3) // Максимальное расстояние 3
        .collect();

    similarities.sort_by_key(|(_, dist)| *dist);
    similarities
        .into_iter()
        .take(max_results)
        .map(|(s, _)| s)
        .collect()
}

/// <function>
///   <name>levenshtein_distance</name>
///   <purpose>Вычисление расстояния Левенштейна между строками</purpose>
/// </function>
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.chars().count();
    let len2 = s2.chars().count();
    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    for (i, row) in matrix.iter_mut().enumerate().take(len1 + 1) {
        row[0] = i;
    }
    for (j, cell) in matrix[0].iter_mut().enumerate().take(len2 + 1) {
        *cell = j;
    }

    for (i, c1) in s1.chars().enumerate() {
        for (j, c2) in s2.chars().enumerate() {
            let cost = if c1 == c2 { 0 } else { 1 };
            matrix[i + 1][j + 1] = std::cmp::min(
                matrix[i][j + 1] + 1,
                std::cmp::min(matrix[i + 1][j] + 1, matrix[i][j] + cost),
            );
        }
    }

    matrix[len1][len2]
}
