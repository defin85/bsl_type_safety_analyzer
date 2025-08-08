// Граф зависимостей
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::modules::BslModule;
use super::objects::ConfigurationObject;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub modules: HashMap<String, ModuleDependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleDependency {
    pub module_name: String,
    pub dependencies: Vec<String>,
    pub dependents: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircularDependency {
    pub cycle: Vec<String>,
}

impl CircularDependency {
    pub fn format(&self) -> String {
        self.cycle.join(" -> ")
    }
}

impl DependencyGraph {
    pub fn build_for_configuration(
        _modules: &[BslModule],
        _objects: &[ConfigurationObject],
    ) -> Result<Self> {
        // TODO: Build dependency graph
        Ok(Self {
            modules: HashMap::new(),
        })
    }

    pub fn find_circular_dependencies(&self) -> Vec<CircularDependency> {
        // TODO: Detect circular dependencies
        Vec::new()
    }
}
