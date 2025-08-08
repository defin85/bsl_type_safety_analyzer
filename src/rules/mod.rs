/*!
# Rules Engine for BSL Analyzer

Configurable rules system for BSL static analysis.
Allows users to customize rule severity, enable/disable rules,
and create custom analysis profiles.

## Features
- TOML/YAML configuration files
- Rule severity customization (error/warning/info/hint)
- Rule enable/disable toggle
- Rule profiles for different projects
- Custom rule support
- Performance tracking per rule

## Usage

```rust,ignore
use bsl_analyzer::rules::{RulesEngine, RulesConfig};

// Load rules from config file
let config = RulesConfig::load_from_file("bsl-rules.toml")?;
let engine = RulesEngine::new(config);

// Apply rules to analysis results
let filtered_results = engine.apply_rules(&analysis_results)?;
```

## Configuration Example

```toml
[rules.profile]
name = "strict"
description = "Strict rules for production code"

[rules.BSL001]
enabled = true
severity = "warning"
description = "Unused variable"

[rules.BSL002]
enabled = true
severity = "error"
description = "Undefined variable"

[rules.custom]
enabled = true
pattern = "regex"
message = "Custom rule violation"
```
*/

pub mod builtin;
pub mod config;
pub mod custom;
pub mod engine;

pub use builtin::BuiltinRules;
pub use config::{RuleConfig, RuleProfile, RuleSeverity, RulesConfig};
pub use custom::CustomRule;
pub use engine::{RuleApplication, RuleResult, RulesEngine};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Rule identifier (e.g., "BSL001", "custom_rule_1")
pub type RuleId = String;

/// Rule application statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleStats {
    /// Number of times rule was applied
    pub applications: u64,
    /// Number of violations found
    pub violations: u64,
    /// Average execution time in microseconds
    pub avg_execution_time_us: f64,
    /// Total execution time in microseconds
    pub total_execution_time_us: u64,
}

impl Default for RuleStats {
    fn default() -> Self {
        Self {
            applications: 0,
            violations: 0,
            avg_execution_time_us: 0.0,
            total_execution_time_us: 0,
        }
    }
}

impl RuleStats {
    /// Update statistics with new execution
    pub fn update(&mut self, violations: u64, execution_time_us: u64) {
        self.applications += 1;
        self.violations += violations;
        self.total_execution_time_us += execution_time_us;
        self.avg_execution_time_us = self.total_execution_time_us as f64 / self.applications as f64;
    }

    /// Get violation rate (violations per application)
    pub fn violation_rate(&self) -> f64 {
        if self.applications == 0 {
            0.0
        } else {
            self.violations as f64 / self.applications as f64
        }
    }
}

/// Rules manager - main entry point for rules system
pub struct RulesManager {
    /// Configuration
    config: RulesConfig,
    /// Rules engine
    engine: RulesEngine,
    /// Rule statistics
    stats: HashMap<RuleId, RuleStats>,
}

impl RulesManager {
    /// Create new rules manager with default configuration
    pub fn new() -> Self {
        let config = RulesConfig::default();
        let engine = RulesEngine::new(config.clone());

        Self {
            config,
            engine,
            stats: HashMap::new(),
        }
    }

    /// Create rules manager with custom configuration
    pub fn new_with_config(config: RulesConfig) -> Self {
        let engine = RulesEngine::new(config.clone());

        Self {
            config,
            engine,
            stats: HashMap::new(),
        }
    }

    /// Create rules manager from configuration file
    pub fn from_file<P: AsRef<std::path::Path>>(config_path: P) -> Result<Self> {
        let config = RulesConfig::load_from_file(config_path)?;
        let engine = RulesEngine::new(config.clone());

        Ok(Self {
            config,
            engine,
            stats: HashMap::new(),
        })
    }

    /// Apply rules to analysis results
    pub fn apply_rules(
        &mut self,
        results: &crate::core::AnalysisResults,
    ) -> Result<crate::core::AnalysisResults> {
        let start_time = std::time::Instant::now();

        let filtered_results = self.engine.apply_rules(results)?;

        let execution_time = start_time.elapsed().as_micros() as u64;

        // Update global statistics
        let rule_id = "all_rules".to_string();
        let violations = filtered_results.total_issues() as u64;

        self.stats
            .entry(rule_id)
            .or_default()
            .update(violations, execution_time);

        Ok(filtered_results)
    }

    /// Get rule statistics
    pub fn get_stats(&self) -> &HashMap<RuleId, RuleStats> {
        &self.stats
    }

    /// Get configuration
    pub fn config(&self) -> &RulesConfig {
        &self.config
    }

    /// Reload configuration from file
    pub fn reload_config<P: AsRef<std::path::Path>>(&mut self, config_path: P) -> Result<()> {
        self.config = RulesConfig::load_from_file(config_path)?;
        self.engine = RulesEngine::new(self.config.clone());
        Ok(())
    }

    /// Reload configuration from memory
    pub fn reload_config_from_memory(&mut self, config: RulesConfig) -> Result<()> {
        self.config = config;
        self.engine = RulesEngine::new(self.config.clone());
        Ok(())
    }

    /// Validate configuration
    pub fn validate_config(&self) -> Result<Vec<String>> {
        self.config.validate()
    }

    /// Export statistics to JSON
    pub fn export_stats(&self) -> Result<String> {
        serde_json::to_string_pretty(&self.stats).context("Failed to serialize rule statistics")
    }

    /// Get rules engine summary
    pub fn get_engine_summary(&self) -> engine::RulesSummary {
        self.engine.get_summary()
    }
}

impl Default for RulesManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::AnalysisResults;

    #[test]
    fn test_rules_manager_creation() {
        let manager = RulesManager::new();
        assert!(!manager.config().rules.is_empty());
    }

    #[test]
    fn test_rules_application() {
        let mut manager = RulesManager::new();
        let results = AnalysisResults::new();

        let filtered = manager.apply_rules(&results).unwrap();
        assert_eq!(filtered.total_issues(), 0);
    }

    #[test]
    fn test_stats_tracking() {
        let mut manager = RulesManager::new();
        let results = AnalysisResults::new();

        manager.apply_rules(&results).unwrap();

        let stats = manager.get_stats();
        assert!(stats.contains_key("all_rules"));

        let all_rules_stats = &stats["all_rules"];
        assert_eq!(all_rules_stats.applications, 1);
    }
}
