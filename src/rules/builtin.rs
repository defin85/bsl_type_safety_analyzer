/*!
# Builtin Rules for BSL Analyzer

Standard built-in rules for BSL static analysis.
These rules cover common BSL code quality issues and best practices.
*/

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::core::{AnalysisResults, AnalysisError};
use super::{RuleConfig, RuleSeverity};

/// Built-in BSL analysis rules
pub struct BuiltinRules;

impl BuiltinRules {
    /// Get all builtin rule configurations
    pub fn get_all_rules() -> HashMap<String, RuleConfig> {
        let mut rules = HashMap::new();
        
        // BSL001: Unused Variable
        rules.insert("BSL001".to_string(), RuleConfig {
            enabled: true,
            severity: RuleSeverity::Warning,
            description: Some("Variable is declared but never used".to_string()),
            message: Some("Unused variable '{name}'. Consider removing it or prefixing with '_'".to_string()),
            config: Self::create_config(&[
                ("check_parameters", "true"),
                ("ignore_underscore", "true"),
            ]),
            tags: vec!["unused".to_string(), "cleanup".to_string()],
            min_confidence: 0.9,
        });
        
        // BSL002: Undefined Variable
        rules.insert("BSL002".to_string(), RuleConfig {
            enabled: true,
            severity: RuleSeverity::Error,
            description: Some("Variable is used but never declared".to_string()),
            message: Some("Undefined variable '{name}'. Declare it with 'Перем' statement".to_string()),
            config: Self::create_config(&[
                ("check_globals", "true"),
                ("check_builtins", "true"),
            ]),
            tags: vec!["undefined".to_string(), "error".to_string()],
            min_confidence: 0.95,
        });
        
        // BSL003: Type Mismatch
        rules.insert("BSL003".to_string(), RuleConfig {
            enabled: true,
            severity: RuleSeverity::Warning,
            description: Some("Assignment of incompatible types".to_string()),
            message: Some("Type mismatch: cannot assign {source_type} to {target_type}".to_string()),
            config: Self::create_config(&[
                ("strict_mode", "false"),
                ("check_implicit_conversions", "true"),
            ]),
            tags: vec!["types".to_string(), "safety".to_string()],
            min_confidence: 0.8,
        });
        
        // BSL004: Unknown Method
        rules.insert("BSL004".to_string(), RuleConfig {
            enabled: true,
            severity: RuleSeverity::Warning,
            description: Some("Method call to unknown or undocumented method".to_string()),
            message: Some("Unknown method '{method}'. Check spelling or documentation".to_string()),
            config: Self::create_config(&[
                ("check_documentation", "true"),
                ("check_metadata", "true"),
                ("suggest_alternatives", "true"),
            ]),
            tags: vec!["methods".to_string(), "documentation".to_string()],
            min_confidence: 0.85,
        });
        
        // BSL005: Circular Dependency
        rules.insert("BSL005".to_string(), RuleConfig {
            enabled: true,
            severity: RuleSeverity::Error,
            description: Some("Circular dependency detected between modules".to_string()),
            message: Some("Circular dependency: {module_a} -> {module_b} -> {module_a}".to_string()),
            config: Self::create_config(&[
                ("max_depth", "10"),
                ("ignore_common_modules", "false"),
            ]),
            tags: vec!["dependencies".to_string(), "architecture".to_string()],
            min_confidence: 1.0,
        });
        
        // BSL006: Dead Code
        rules.insert("BSL006".to_string(), RuleConfig {
            enabled: true,
            severity: RuleSeverity::Info,
            description: Some("Unreachable or dead code detected".to_string()),
            message: Some("Dead code detected: this code will never be executed".to_string()),
            config: Self::create_config(&[
                ("check_after_return", "true"),
                ("check_impossible_conditions", "true"),
            ]),
            tags: vec!["dead-code".to_string(), "cleanup".to_string()],
            min_confidence: 0.9,
        });
        
        // BSL007: Complex Function
        rules.insert("BSL007".to_string(), RuleConfig {
            enabled: true,
            severity: RuleSeverity::Hint,
            description: Some("Function or procedure is too complex".to_string()),
            message: Some("Function '{name}' is complex (complexity: {complexity}). Consider refactoring".to_string()),
            config: Self::create_config(&[
                ("max_complexity", "10"),
                ("max_lines", "50"),
                ("max_parameters", "5"),
            ]),
            tags: vec!["complexity".to_string(), "refactoring".to_string()],
            min_confidence: 0.8,
        });
        
        // BSL008: Missing Documentation
        rules.insert("BSL008".to_string(), RuleConfig {
            enabled: false, // Disabled by default as it can be noisy
            severity: RuleSeverity::Hint,
            description: Some("Public function or procedure lacks documentation".to_string()),
            message: Some("Public {type} '{name}' should have documentation comment".to_string()),
            config: Self::create_config(&[
                ("check_public_only", "true"),
                ("check_exports", "true"),
                ("min_comment_lines", "1"),
            ]),
            tags: vec!["documentation".to_string(), "public-api".to_string()],
            min_confidence: 0.7,
        });
        
        // BSL009: Performance Warning
        rules.insert("BSL009".to_string(), RuleConfig {
            enabled: true,
            severity: RuleSeverity::Info,
            description: Some("Potential performance issue detected".to_string()),
            message: Some("Performance warning: {issue}. Consider {suggestion}".to_string()),
            config: Self::create_config(&[
                ("check_loops", "true"),
                ("check_string_concatenation", "true"),
                ("check_query_in_loop", "true"),
            ]),
            tags: vec!["performance".to_string(), "optimization".to_string()],
            min_confidence: 0.75,
        });
        
        // BSL010: Security Warning
        rules.insert("BSL010".to_string(), RuleConfig {
            enabled: true,
            severity: RuleSeverity::Warning,
            description: Some("Potential security issue detected".to_string()),
            message: Some("Security warning: {issue}. {recommendation}".to_string()),
            config: Self::create_config(&[
                ("check_sql_injection", "true"),
                ("check_privilege_escalation", "true"),
                ("check_unsafe_eval", "true"),
            ]),
            tags: vec!["security".to_string(), "safety".to_string()],
            min_confidence: 0.9,
        });
        
        rules
    }
    
    /// Get rule descriptions for documentation
    pub fn get_rule_descriptions() -> HashMap<String, RuleDescription> {
        let mut descriptions = HashMap::new();
        
        descriptions.insert("BSL001".to_string(), RuleDescription {
            title: "Unused Variable".to_string(),
            description: "Detects variables that are declared but never used in the code.".to_string(),
            rationale: "Unused variables clutter the code and may indicate incomplete implementation or refactoring artifacts.".to_string(),
            examples: vec![
                RuleExample {
                    title: "Bad".to_string(),
                    code: r#"
Функция ПримерФункции()
    Перем НеиспользуемаяПеременная;
    Перем ИспользуемаяПеременная;
    
    ИспользуемаяПеременная = "значение";
    Возврат ИспользуемаяПеременная;
КонецФункции
"#.to_string(),
                },
                RuleExample {
                    title: "Good".to_string(),
                    code: r#"
Функция ПримерФункции()
    Перем ИспользуемаяПеременная;
    
    ИспользуемаяПеременная = "значение";
    Возврат ИспользуемаяПеременная;
КонецФункции
"#.to_string(),
                },
            ],
            configuration: vec![
                ConfigOption {
                    name: "check_parameters".to_string(),
                    description: "Check unused function parameters".to_string(),
                    default_value: "true".to_string(),
                },
                ConfigOption {
                    name: "ignore_underscore".to_string(),
                    description: "Ignore variables starting with underscore".to_string(),
                    default_value: "true".to_string(),
                },
            ],
        });
        
        descriptions.insert("BSL002".to_string(), RuleDescription {
            title: "Undefined Variable".to_string(),
            description: "Detects usage of variables that were never declared.".to_string(),
            rationale: "Using undefined variables will cause runtime errors and indicates missing variable declarations.".to_string(),
            examples: vec![
                RuleExample {
                    title: "Bad".to_string(),
                    code: r#"
Функция ПримерФункции()
    НеобъявленнаяПеременная = "значение";
    Возврат НеобъявленнаяПеременная;
КонецФункции
"#.to_string(),
                },
                RuleExample {
                    title: "Good".to_string(),
                    code: r#"
Функция ПримерФункции()
    Перем ОбъявленнаяПеременная;
    ОбъявленнаяПеременная = "значение";
    Возврат ОбъявленнаяПеременная;
КонецФункции
"#.to_string(),
                },
            ],
            configuration: vec![
                ConfigOption {
                    name: "check_globals".to_string(),
                    description: "Check access to global variables".to_string(),
                    default_value: "true".to_string(),
                },
            ],
        });
        
        // Add more descriptions as needed...
        
        descriptions
    }
    
    /// Create configuration HashMap from key-value pairs
    fn create_config(pairs: &[(&str, &str)]) -> HashMap<String, toml::Value> {
        let mut config = HashMap::new();
        for (key, value) in pairs {
            // Try to parse as boolean first, then string
            let toml_value = if value == &"true" {
                toml::Value::Boolean(true)
            } else if value == &"false" {
                toml::Value::Boolean(false)
            } else if let Ok(num) = value.parse::<i64>() {
                toml::Value::Integer(num)
            } else if let Ok(num) = value.parse::<f64>() {
                toml::Value::Float(num)
            } else {
                toml::Value::String(value.to_string())
            };
            
            config.insert(key.to_string(), toml_value);
        }
        config
    }
    
    /// Apply builtin rule logic to analysis results
    pub fn apply_builtin_rules(results: &mut AnalysisResults, rule_configs: &HashMap<String, RuleConfig>) {
        // This would contain the actual rule logic
        // For now, we'll just update error codes and messages based on configuration
        
        let mut new_errors = Vec::new();
        for error in results.get_errors() {
            let mut updated_error = error.clone();
            
            if let Some(ref error_code) = error.error_code {
                if let Some(rule_config) = rule_configs.get(error_code) {
                    // Apply custom message if configured
                    if let Some(ref custom_message) = rule_config.message {
                        updated_error.message = Self::format_message(custom_message, &updated_error);
                    }
                    
                    // Update error level based on severity
                    updated_error.level = rule_config.severity.into();
                }
            }
            
            new_errors.push(updated_error);
        }
        
        // Similarly for warnings...
        let mut new_warnings = Vec::new();
        for warning in results.get_warnings() {
            let mut updated_warning = warning.clone();
            
            if let Some(ref error_code) = warning.error_code {
                if let Some(rule_config) = rule_configs.get(error_code) {
                    if let Some(ref custom_message) = rule_config.message {
                        updated_warning.message = Self::format_message(custom_message, &updated_warning);
                    }
                    
                    updated_warning.level = rule_config.severity.into();
                }
            }
            
            new_warnings.push(updated_warning);
        }
    }
    
    /// Format rule message with context
    fn format_message(template: &str, error: &AnalysisError) -> String {
        // Simple template substitution
        // In a full implementation, this would be more sophisticated
        template.replace("{message}", &error.message)
    }
    
    /// Get rule recommendations
    pub fn get_recommendations(rule_id: &str) -> Vec<String> {
        match rule_id {
            "BSL001" => vec![
                "Remove the unused variable".to_string(),
                "Prefix variable name with '_' if it's intentionally unused".to_string(),
                "Use the variable in your code logic".to_string(),
            ],
            "BSL002" => vec![
                "Add 'Перем VariableName;' declaration".to_string(),
                "Check for typos in variable name".to_string(),
                "Verify variable scope and accessibility".to_string(),
            ],
            "BSL003" => vec![
                "Use explicit type conversion functions".to_string(),
                "Check data types compatibility".to_string(),
                "Review assignment logic".to_string(),
            ],
            "BSL004" => vec![
                "Check method name spelling".to_string(),
                "Verify method is available in current context".to_string(),
                "Consult 1C:Enterprise documentation".to_string(),
                "Check if method is from external component".to_string(),
            ],
            "BSL005" => vec![
                "Refactor modules to remove circular dependency".to_string(),
                "Extract common functionality to separate module".to_string(),
                "Review module architecture".to_string(),
            ],
            _ => vec!["Follow BSL best practices".to_string()],
        }
    }
}

/// Rule description for documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleDescription {
    pub title: String,
    pub description: String,
    pub rationale: String,
    pub examples: Vec<RuleExample>,
    pub configuration: Vec<ConfigOption>,
}

/// Code example for rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleExample {
    pub title: String,
    pub code: String,
}

/// Configuration option for rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigOption {
    pub name: String,
    pub description: String,
    pub default_value: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_builtin_rules_generation() {
        let rules = BuiltinRules::get_all_rules();
        
        assert!(rules.contains_key("BSL001"));
        assert!(rules.contains_key("BSL002"));
        assert!(rules.contains_key("BSL005"));
        
        // Check that BSL001 has correct configuration
        let bsl001 = &rules["BSL001"];
        assert_eq!(bsl001.severity, RuleSeverity::Warning);
        assert!(bsl001.enabled);
        assert!(bsl001.config.contains_key("check_parameters"));
    }
    
    #[test]
    fn test_rule_descriptions() {
        let descriptions = BuiltinRules::get_rule_descriptions();
        
        assert!(descriptions.contains_key("BSL001"));
        assert!(descriptions.contains_key("BSL002"));
        
        let bsl001_desc = &descriptions["BSL001"];
        assert_eq!(bsl001_desc.title, "Unused Variable");
        assert!(!bsl001_desc.examples.is_empty());
    }
    
    #[test]
    fn test_recommendations() {
        let recommendations = BuiltinRules::get_recommendations("BSL001");
        assert!(!recommendations.is_empty());
        assert!(recommendations[0].contains("Remove"));
        
        let unknown_recommendations = BuiltinRules::get_recommendations("UNKNOWN");
        assert_eq!(unknown_recommendations.len(), 1);
        assert!(unknown_recommendations[0].contains("best practices"));
    }
    
    #[test]
    fn test_config_creation() {
        let config = BuiltinRules::create_config(&[
            ("enabled", "true"),
            ("threshold", "10"),
            ("name", "test"),
        ]);
        
        assert_eq!(config["enabled"], toml::Value::Boolean(true));
        assert_eq!(config["threshold"], toml::Value::Integer(10));
        assert_eq!(config["name"], toml::Value::String("test".to_string()));
    }
}