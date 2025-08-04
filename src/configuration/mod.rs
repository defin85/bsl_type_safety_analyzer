/*!
# Configuration Management

Modern configuration management using UnifiedBslIndex architecture.
Legacy parsers have been removed in favor of direct XML parsing and unified index.
*/

pub mod metadata;
pub mod modules;
pub mod objects;
pub mod dependencies;
pub mod module_generator;

pub use metadata::{ConfigurationMetadata, MetadataObject};
pub use modules::{BslModule, ModuleType};
pub use objects::ConfigurationObject;
pub use dependencies::{DependencyGraph, ModuleDependency};
pub use module_generator::{ModuleGenerator, ModuleContract};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Main configuration structure (Modern UnifiedBslIndex architecture)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    pub path: PathBuf,
    pub metadata: ConfigurationMetadata,
    pub modules: Vec<BslModule>,
    pub objects: Vec<ConfigurationObject>,
    pub dependencies: DependencyGraph,
}

impl Configuration {
    /// Loads configuration from directory (Modern UnifiedBslIndex architecture)
    pub fn load_from_directory<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        
        tracing::info!("Loading configuration from: {}", path.display());
        
        // Load metadata from Configuration.xml
        let metadata = ConfigurationMetadata::load_from_path(&path)
            .context("Failed to load configuration metadata")?;
        
        // Discover and load BSL modules
        let modules = Self::discover_bsl_modules(&path)
            .context("Failed to discover BSL modules")?;
        
        // Load configuration objects from metadata
        let objects = Self::load_configuration_objects(&path, &metadata)
            .context("Failed to load configuration objects")?;
        
        // Build dependency graph
        let dependencies = DependencyGraph::build_for_configuration(&modules, &objects)
            .context("Failed to build dependency graph")?;
        
        tracing::info!(
            "Configuration loaded: {} modules, {} objects",
            modules.len(),
            objects.len()
        );
        
        Ok(Configuration {
            path,
            metadata,
            modules,
            objects,
            dependencies,
        })
    }
    
    /// Discovers all BSL modules in the configuration
    fn discover_bsl_modules(config_path: &Path) -> Result<Vec<BslModule>> {
        let mut modules = Vec::new();
        
        for entry in WalkDir::new(config_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("bsl") {
                let module = BslModule::load_from_file(path)
                    .with_context(|| format!("Failed to load module: {}", path.display()))?;
                
                modules.push(module);
            }
        }
        
        // Sort modules by name for consistency
        modules.sort_by(|a, b| a.name.cmp(&b.name));
        
        Ok(modules)
    }
    
    /// Loads configuration objects from metadata
    fn load_configuration_objects(
        _config_path: &Path,
        metadata: &ConfigurationMetadata,
    ) -> Result<Vec<ConfigurationObject>> {
        let mut objects = Vec::new();
        
        // Load objects from metadata
        for metadata_object in &metadata.objects {
            let object = ConfigurationObject::from_metadata(metadata_object)?;
            objects.push(object);
        }
        
        Ok(objects)
    }
    
    
    /// Gets module by name
    pub fn get_module(&self, name: &str) -> Option<&BslModule> {
        self.modules.iter().find(|m| m.name == name)
    }
    
    /// Gets all modules
    pub fn get_modules(&self) -> &Vec<BslModule> {
        &self.modules
    }
    
    /// Gets all export procedures/functions
    pub fn get_all_exports(&self) -> HashMap<String, Vec<&BslModule>> {
        let mut exports = HashMap::new();
        
        for module in &self.modules {
            for export in &module.exports {
                exports.entry(export.name.clone())
                    .or_insert_with(Vec::new)
                    .push(module);
            }
        }
        
        exports
    }
    
    /// Finds modules that depend on the given module
    pub fn find_dependents(&self, module_name: &str) -> Vec<&BslModule> {
        self.modules
            .iter()
            .filter(|m| {
                m.imports.iter().any(|imp| imp.module_name == module_name)
            })
            .collect()
    }
    
    /// Validates configuration consistency
    pub fn validate(&self) -> Result<ValidationResult> {
        let mut issues = Vec::new();
        
        // Check for circular dependencies
        let circular_deps = self.dependencies.find_circular_dependencies();
        for circular in circular_deps {
            issues.push(format!("Circular dependency: {}", circular.format()));
        }
        
        // Check for missing dependencies
        for module in &self.modules {
            for import in &module.imports {
                if !self.modules.iter().any(|m| m.name == import.module_name) {
                    issues.push(format!(
                        "Module '{}' imports missing module '{}'",
                        module.name, import.module_name
                    ));
                }
            }
        }
        
        // Check for duplicate module names
        let mut module_names = std::collections::HashSet::new();
        for module in &self.modules {
            if !module_names.insert(&module.name) {
                issues.push(format!("Duplicate module name: '{}'", module.name));
            }
        }
        
        Ok(ValidationResult::new(issues))
    }
    
    /// Gets statistics about the configuration
    pub fn statistics(&self) -> ConfigurationStatistics {
        let total_modules = self.modules.len();
        let total_exports = self.modules.iter()
            .map(|m| m.exports.len())
            .sum();
        let total_imports = self.modules.iter()
            .map(|m| m.imports.len())
            .sum();
        
        let mut module_types = HashMap::new();
        for module in &self.modules {
            *module_types.entry(module.module_type.clone()).or_insert(0) += 1;
        }
        
        ConfigurationStatistics {
            total_modules,
            total_exports,
            total_imports,
            total_objects: self.objects.len(),
            module_types,
        }
    }
}

/// Configuration validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    issues: Vec<String>,
}

impl ValidationResult {
    pub fn new(issues: Vec<String>) -> Self {
        Self { issues }
    }
    
    pub fn is_valid(&self) -> bool {
        self.issues.is_empty()
    }
    
    pub fn issues(&self) -> &[String] {
        &self.issues
    }
}

/// Configuration statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationStatistics {
    pub total_modules: usize,
    pub total_exports: usize,
    pub total_imports: usize,
    pub total_objects: usize,
    pub module_types: HashMap<ModuleType, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    
    #[test]
    fn test_load_simple_configuration() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path();
        
        // Create Configuration.xml
        let metadata_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <MetaDataObject xmlns="http://v8.1c.ru/8.3/MDClasses" xmlns:app="http://v8.1c.ru/8.2/managed-application/core" xmlns:cfg="http://v8.1c.ru/8.1/data/enterprise/current-config" xmlns:cmi="http://v8.1c.ru/8.2/managed-application/cmi" xmlns:ent="http://v8.1c.ru/8.1/data/enterprise" xmlns:lf="http://v8.1c.ru/8.2/managed-application/logform" xmlns:style="http://v8.1c.ru/8.1/data/ui/style" xmlns:sys="http://v8.1c.ru/8.1/data/ui/fonts/system" xmlns:v8="http://v8.1c.ru/8.1/data/core" xmlns:v8ui="http://v8.1c.ru/8.1/data/ui" xmlns:web="http://v8.1c.ru/8.1/data/ui/colors/web" xmlns:win="http://v8.1c.ru/8.1/data/ui/colors/windows" xmlns:xen="http://v8.1c.ru/8.3/xcf/enums" xmlns:xpr="http://v8.1c.ru/8.3/xcf/predef" xmlns:xr="http://v8.1c.ru/8.3/xcf/readable" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" version="2.19">
          <Configuration uuid="12345678-1234-1234-1234-123456789012">
            <Name>TestConfiguration</Name>
            <Synonym>Test Configuration</Synonym>
            <Comment/>
            <NamePrefix/>
            <ConfigurationExtensionCompatibilityMode>8.3.20</ConfigurationExtensionCompatibilityMode>
            <DefaultRunMode>ManagedApplication</DefaultRunMode>
            <UsePurposes>
              <v8:Value xsi:type="app:ApplicationUsePurpose">PlatformApplication</v8:Value>
            </UsePurposes>
            <ScriptVariant>Russian</ScriptVariant>
            <DefaultRoles/>
            <Vendor/>
            <Version>1.0.0</Version>
            <UpdateCatalogAddress/>
            <IncludeHelpInContents>true</IncludeHelpInContents>
            <UseManagedFormInOrdinaryApplication>false</UseManagedFormInOrdinaryApplication>
            <UseOrdinaryFormInManagedApplication>false</UseOrdinaryFormInManagedApplication>
            <AdditionalFullTextSearchDictionaries/>
            <CommonModules>
              <CommonModule uuid="11111111-1111-1111-1111-111111111111">
                <Properties>
                  <Name>ОбщийМодуль</Name>
                  <Synonym>Общий модуль</Synonym>
                  <Comment/>
                </Properties>
              </CommonModule>
            </CommonModules>
          </Configuration>
        </MetaDataObject>"#;
        
        fs::write(config_path.join("Configuration.xml"), metadata_xml).unwrap();
        
        // Create BSL module
        let module_content = r#"
            Процедура ТестоваяПроцедура() Экспорт
                Сообщить("Тест");
            КонецПроцедуры
        "#;
        
        fs::create_dir_all(config_path.join("CommonModules").join("ОбщийМодуль")).unwrap();
        fs::write(
            config_path.join("CommonModules").join("ОбщийМодуль").join("Module.bsl"),
            module_content
        ).unwrap();
        
        let config = Configuration::load_from_directory(config_path).unwrap();
        
        assert_eq!(config.metadata.name, "TestConfiguration");
        assert_eq!(config.metadata.version, "1.0.0");
        assert!(!config.modules.is_empty());
    }
    
    #[test]
    fn test_configuration_validation() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path();
        
        // Create minimal valid configuration
        fs::write(config_path.join("Configuration.xml"), r#"<?xml version="1.0"?>
        <Configuration>
            <Name>TestConfig</Name>
            <Version>1.0</Version>
        </Configuration>"#).unwrap();
        
        let config = Configuration::load_from_directory(config_path).unwrap();
        let validation = config.validate().unwrap();
        
        assert!(validation.is_valid());
    }
}
