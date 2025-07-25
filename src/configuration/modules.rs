// BSL модули
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ModuleType {
    CommonModule,
    ObjectModule,
    FormModule,
    ManagerModule,
    ApplicationModule,
    SessionModule,
}

impl std::fmt::Display for ModuleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModuleType::CommonModule => write!(f, "CommonModule"),
            ModuleType::ObjectModule => write!(f, "ObjectModule"),
            ModuleType::FormModule => write!(f, "FormModule"),
            ModuleType::ManagerModule => write!(f, "ManagerModule"),
            ModuleType::ApplicationModule => write!(f, "ApplicationModule"),
            ModuleType::SessionModule => write!(f, "SessionModule"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BslModule {
    pub name: String,
    pub path: std::path::PathBuf,
    pub module_type: ModuleType,
    pub exports: Vec<ExportDeclaration>,
    pub imports: Vec<ImportDeclaration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportDeclaration {
    pub name: String,
    pub declaration_type: String,
    pub parameters: Vec<String>,
    pub line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportDeclaration {
    pub module_name: String,
    pub procedure_name: String,
    pub line: usize,
}

impl BslModule {
    pub fn load_from_file(path: &Path) -> Result<Self> {
        // TODO: Parse BSL file and extract exports/imports
        Ok(Self {
            name: path.file_stem().unwrap().to_string_lossy().to_string(),
            path: path.to_path_buf(),
            module_type: ModuleType::CommonModule,
            exports: Vec::new(),
            imports: Vec::new(),
        })
    }
}
