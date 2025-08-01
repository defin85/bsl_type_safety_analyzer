/// <module>
///   <name>types</name>
///   <purpose>Типы данных для MCP сервера</purpose>
/// </module>

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// <type>
///   <name>McpResult</name>
///   <purpose>Результат выполнения MCP операций</purpose>
/// </type>
pub type McpResult<T> = Result<T, McpError>;

/// <type>
///   <name>McpError</name>
///   <purpose>Ошибки MCP сервера</purpose>
/// </type>
#[derive(Error, Debug)]
pub enum McpError {
    #[error("Type not found: {0}")]
    TypeNotFound(String),
    
    #[error("Method not found: {method} for type {type_name}")]
    MethodNotFound { type_name: String, method: String },
    
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
    
    #[error("Index not loaded")]
    IndexNotLoaded,
    
    #[error("Internal error: {0}")]
    Internal(String),
}

/// <type>
///   <name>BslLanguagePreference</name>
///   <purpose>Языковые предпочтения для поиска</purpose>
/// </type>
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BslLanguagePreference {
    Russian,
    English,
    Auto,
}

impl Default for BslLanguagePreference {
    fn default() -> Self {
        Self::Auto
    }
}

impl From<Option<String>> for BslLanguagePreference {
    fn from(value: Option<String>) -> Self {
        match value.as_deref() {
            Some("russian") => Self::Russian,
            Some("english") => Self::English,
            _ => Self::Auto,
        }
    }
}