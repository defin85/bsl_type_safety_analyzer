// Объекты конфигурации
use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::metadata::MetadataObject;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationObject {
    pub name: String,
    pub object_type: String,
    pub properties: Vec<ObjectProperty>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectProperty {
    pub name: String,
    pub property_type: String,
    pub description: Option<String>,
}

impl ConfigurationObject {
    pub fn from_metadata(metadata: &MetadataObject) -> Result<Self> {
        Ok(Self {
            name: metadata.name.clone(),
            object_type: metadata.object_type.clone(),
            properties: Vec::new(),
        })
    }
}
