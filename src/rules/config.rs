/*!
# Rules Configuration System

Configuration structures and loading for BSL analysis rules.
Supports TOML and YAML configuration files with comprehensive
rule customization options.
*/

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Rule severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum RuleSeverity {
    /// Error - blocks compilation/CI
    Error,
    /// Warning - should be fixed but doesn't block
    #[default]
    Warning,
    /// Info - informational message
    Info,
    /// Hint - subtle suggestion
    Hint,
}

impl std::fmt::Display for RuleSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuleSeverity::Error => write!(f, "error"),
            RuleSeverity::Warning => write!(f, "warning"),
            RuleSeverity::Info => write!(f, "info"),
            RuleSeverity::Hint => write!(f, "hint"),
        }
    }
}

impl From<RuleSeverity> for crate::core::errors::ErrorLevel {
    fn from(severity: RuleSeverity) -> Self {
        match severity {
            RuleSeverity::Error => crate::core::errors::ErrorLevel::Error,
            RuleSeverity::Warning => crate::core::errors::ErrorLevel::Warning,
            RuleSeverity::Info => crate::core::errors::ErrorLevel::Info,
            RuleSeverity::Hint => crate::core::errors::ErrorLevel::Hint,
        }
    }
}

/// Configuration for a single rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleConfig {
    /// Whether the rule is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Rule severity level
    #[serde(default)]
    pub severity: RuleSeverity,

    /// Human-readable description
    pub description: Option<String>,

    /// Custom message template
    pub message: Option<String>,

    /// Rule-specific configuration
    #[serde(default)]
    pub config: HashMap<String, toml::Value>,

    /// Rule categories/tags
    #[serde(default)]
    pub tags: Vec<String>,

    /// Minimum confidence level (0.0 to 1.0)
    #[serde(default = "default_confidence")]
    pub min_confidence: f64,
}

fn default_true() -> bool {
    true
}

fn default_confidence() -> f64 {
    0.8
}

impl Default for RuleConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            severity: RuleSeverity::Warning,
            description: None,
            message: None,
            config: HashMap::new(),
            tags: Vec::new(),
            min_confidence: 0.8,
        }
    }
}

/// Rule profile configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleProfile {
    /// Profile name
    pub name: String,

    /// Profile description
    pub description: Option<String>,

    /// Base profile to extend
    pub extends: Option<String>,

    /// Rules to include in this profile
    #[serde(default)]
    pub includes: Vec<String>,

    /// Rules to exclude from this profile
    #[serde(default)]
    pub excludes: Vec<String>,

    /// Default severity for unspecified rules
    #[serde(default)]
    pub default_severity: RuleSeverity,
}

impl Default for RuleProfile {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            description: Some("Default rule profile".to_string()),
            extends: None,
            includes: Vec::new(),
            excludes: Vec::new(),
            default_severity: RuleSeverity::Warning,
        }
    }
}

/// Global rules configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulesConfig {
    /// Configuration version
    #[serde(default = "default_version")]
    pub version: String,

    /// Active profile
    #[serde(default = "default_profile")]
    pub active_profile: String,

    /// Available profiles
    #[serde(default)]
    pub profiles: HashMap<String, RuleProfile>,

    /// Rule configurations
    #[serde(default)]
    pub rules: HashMap<String, RuleConfig>,

    /// Global settings
    #[serde(default)]
    pub settings: GlobalSettings,
}

fn default_version() -> String {
    "1.0".to_string()
}

fn default_profile() -> String {
    "default".to_string()
}

/// Global analysis settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSettings {
    /// Maximum number of errors before stopping analysis
    #[serde(default = "default_max_errors")]
    pub max_errors: Option<u32>,

    /// Whether to show rule IDs in messages
    #[serde(default = "default_true")]
    pub show_rule_ids: bool,

    /// Whether to use colors in output
    #[serde(default = "default_true")]
    pub use_colors: bool,

    /// Parallel processing threads
    pub threads: Option<usize>,

    /// Cache rules evaluation results
    #[serde(default = "default_true")]
    pub cache_results: bool,
}

fn default_max_errors() -> Option<u32> {
    Some(100)
}

impl Default for GlobalSettings {
    fn default() -> Self {
        Self {
            max_errors: Some(100),
            show_rule_ids: true,
            use_colors: true,
            threads: None,
            cache_results: true,
        }
    }
}

impl Default for RulesConfig {
    fn default() -> Self {
        let mut rules = HashMap::new();
        let mut profiles = HashMap::new();

        // Default builtin rules
        let builtin_rules = [
            ("BSL001", "Unused variable", RuleSeverity::Warning),
            ("BSL002", "Undefined variable", RuleSeverity::Error),
            ("BSL003", "Type mismatch", RuleSeverity::Warning),
            ("BSL004", "Unknown method", RuleSeverity::Warning),
            ("BSL005", "Circular dependency", RuleSeverity::Error),
            ("BSL006", "Dead code", RuleSeverity::Info),
            ("BSL007", "Complex function", RuleSeverity::Hint),
            ("BSL008", "Missing documentation", RuleSeverity::Hint),
        ];

        for (rule_id, description, severity) in builtin_rules {
            rules.insert(
                rule_id.to_string(),
                RuleConfig {
                    enabled: true,
                    severity,
                    description: Some(description.to_string()),
                    message: None,
                    config: HashMap::new(),
                    tags: vec!["builtin".to_string()],
                    min_confidence: 0.8,
                },
            );
        }

        // Default profile
        profiles.insert("default".to_string(), RuleProfile::default());

        // Strict profile
        profiles.insert(
            "strict".to_string(),
            RuleProfile {
                name: "strict".to_string(),
                description: Some("Strict rules for production code".to_string()),
                extends: Some("default".to_string()),
                includes: Vec::new(),
                excludes: Vec::new(),
                default_severity: RuleSeverity::Error,
            },
        );

        // Lenient profile
        profiles.insert(
            "lenient".to_string(),
            RuleProfile {
                name: "lenient".to_string(),
                description: Some("Lenient rules for prototyping".to_string()),
                extends: Some("default".to_string()),
                includes: Vec::new(),
                excludes: vec!["BSL001".to_string(), "BSL006".to_string()],
                default_severity: RuleSeverity::Info,
            },
        );

        Self {
            version: "1.0".to_string(),
            active_profile: "default".to_string(),
            profiles,
            rules,
            settings: GlobalSettings::default(),
        }
    }
}

impl RulesConfig {
    /// Create strict profile configuration
    pub fn strict_profile() -> Self {
        let mut config = Self {
            active_profile: "strict".to_string(),
            ..Self::default()
        };

        // Enable all rules in strict mode
        for (_rule_id, rule_config) in config.rules.iter_mut() {
            rule_config.enabled = true;
            // Make rules stricter
            if rule_config.severity == RuleSeverity::Info {
                rule_config.severity = RuleSeverity::Warning;
            }
            if rule_config.severity == RuleSeverity::Hint {
                rule_config.severity = RuleSeverity::Warning;
            }
        }

        // Add strict profile
        let strict_profile = RuleProfile {
            name: "strict".to_string(),
            description: Some("Strict rule profile with all rules enabled".to_string()),
            extends: None,
            includes: config.rules.keys().cloned().collect(),
            excludes: Vec::new(),
            default_severity: RuleSeverity::Warning,
        };
        config.profiles.insert("strict".to_string(), strict_profile);

        config
    }

    /// Alias for load_from_file for backwards compatibility
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::load_from_file(path)
    }

    /// Export configuration to TOML file
    pub fn export_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.save_to_file(path)
    }

    /// Load configuration from TOML file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(&path).with_context(|| {
            format!(
                "Failed to read rules config from {}",
                path.as_ref().display()
            )
        })?;

        let config: Self = toml::from_str(&content).with_context(|| {
            format!(
                "Failed to parse TOML config from {}",
                path.as_ref().display()
            )
        })?;

        config.validate()?;
        Ok(config)
    }

    /// Save configuration to TOML file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content =
            toml::to_string_pretty(self).context("Failed to serialize rules config to TOML")?;

        std::fs::write(&path, content).with_context(|| {
            format!(
                "Failed to write rules config to {}",
                path.as_ref().display()
            )
        })?;

        Ok(())
    }

    /// Load configuration from YAML file
    pub fn load_from_yaml<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(&path).with_context(|| {
            format!(
                "Failed to read rules config from {}",
                path.as_ref().display()
            )
        })?;

        let config: Self = serde_yaml::from_str(&content).with_context(|| {
            format!(
                "Failed to parse YAML config from {}",
                path.as_ref().display()
            )
        })?;

        config.validate()?;
        Ok(config)
    }

    /// Get active profile
    pub fn get_active_profile(&self) -> Result<&RuleProfile> {
        self.profiles
            .get(&self.active_profile)
            .with_context(|| format!("Active profile '{}' not found", self.active_profile))
    }

    /// Get rule configuration
    pub fn get_rule(&self, rule_id: &str) -> Option<&RuleConfig> {
        self.rules.get(rule_id)
    }

    /// Check if rule is enabled in active profile
    pub fn is_rule_enabled(&self, rule_id: &str) -> Result<bool> {
        let profile = self.get_active_profile()?;

        // Check if rule is explicitly excluded
        if profile.excludes.contains(&rule_id.to_string()) {
            return Ok(false);
        }

        // Check if rule is explicitly included
        if !profile.includes.is_empty() && !profile.includes.contains(&rule_id.to_string()) {
            return Ok(false);
        }

        // Check rule's enabled flag
        if let Some(rule_config) = self.get_rule(rule_id) {
            Ok(rule_config.enabled)
        } else {
            Ok(false)
        }
    }

    /// Get effective severity for rule in active profile
    pub fn get_rule_severity(&self, rule_id: &str) -> Result<RuleSeverity> {
        if let Some(rule_config) = self.get_rule(rule_id) {
            Ok(rule_config.severity)
        } else {
            let profile = self.get_active_profile()?;
            Ok(profile.default_severity)
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<Vec<String>> {
        let mut warnings = Vec::new();

        // Check if active profile exists
        if !self.profiles.contains_key(&self.active_profile) {
            warnings.push(format!(
                "Active profile '{}' not found",
                self.active_profile
            ));
        }

        // Validate profile inheritance
        for (profile_name, profile) in &self.profiles {
            if let Some(ref extends) = profile.extends {
                if !self.profiles.contains_key(extends) {
                    warnings.push(format!(
                        "Profile '{}' extends non-existent profile '{}'",
                        profile_name, extends
                    ));
                }
            }
        }

        // Validate rule references in profiles
        for (profile_name, profile) in &self.profiles {
            for rule_id in &profile.includes {
                if !self.rules.contains_key(rule_id) {
                    warnings.push(format!(
                        "Profile '{}' includes unknown rule '{}'",
                        profile_name, rule_id
                    ));
                }
            }

            for rule_id in &profile.excludes {
                if !self.rules.contains_key(rule_id) {
                    warnings.push(format!(
                        "Profile '{}' excludes unknown rule '{}'",
                        profile_name, rule_id
                    ));
                }
            }
        }

        // Validate rule configurations
        for (rule_id, rule_config) in &self.rules {
            if rule_config.min_confidence < 0.0 || rule_config.min_confidence > 1.0 {
                warnings.push(format!(
                    "Rule '{}' has invalid confidence level: {}",
                    rule_id, rule_config.min_confidence
                ));
            }
        }

        Ok(warnings)
    }

    /// Create example configuration file
    pub fn create_example_config<P: AsRef<Path>>(path: P) -> Result<()> {
        let config = Self::default();
        config.save_to_file(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = RulesConfig::default();
        assert!(!config.rules.is_empty());
        assert!(!config.profiles.is_empty());
        assert_eq!(config.active_profile, "default");
    }

    #[test]
    fn test_rule_severity_conversion() {
        let error: crate::core::errors::ErrorLevel = RuleSeverity::Error.into();
        assert_eq!(error, crate::core::errors::ErrorLevel::Error);

        let warning: crate::core::errors::ErrorLevel = RuleSeverity::Warning.into();
        assert_eq!(warning, crate::core::errors::ErrorLevel::Warning);
    }

    #[test]
    fn test_config_serialization() {
        let config = RulesConfig::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(toml_str.contains("[rules"));
        assert!(toml_str.contains("[profiles"));
    }

    #[test]
    fn test_config_file_roundtrip() {
        let config = RulesConfig::default();

        let temp_file = NamedTempFile::new().unwrap();
        config.save_to_file(temp_file.path()).unwrap();

        let loaded_config = RulesConfig::load_from_file(temp_file.path()).unwrap();
        assert_eq!(config.active_profile, loaded_config.active_profile);
        assert_eq!(config.rules.len(), loaded_config.rules.len());
    }

    #[test]
    fn test_rule_enabled_check() {
        let config = RulesConfig::default();

        // Test enabled rule
        assert!(config.is_rule_enabled("BSL001").unwrap());

        // Test non-existent rule
        assert!(!config.is_rule_enabled("NONEXISTENT").unwrap());
    }

    #[test]
    fn test_config_validation() {
        let config = RulesConfig::default();
        let warnings = config.validate().unwrap();
        assert!(warnings.is_empty());
    }
}
