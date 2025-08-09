/*!
# Custom Rules System

Support for user-defined custom rules with pattern matching,
regex support, and flexible configuration options.
*/

use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::RuleSeverity;
use crate::core::{AnalysisError, AnalysisResults};

/// Type of custom rule pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PatternType {
    /// Simple substring matching
    Contains,
    /// Regular expression matching
    Regex,
    /// Exact string matching
    Exact,
    /// AST-based pattern matching (future)
    Ast,
}

/// Custom rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomRule {
    /// Rule identifier
    pub id: String,

    /// Rule name
    pub name: String,

    /// Rule description
    pub description: String,

    /// Pattern type
    pub pattern_type: PatternType,

    /// Pattern to match against
    pub pattern: String,

    /// Compiled regex (if pattern_type is Regex)
    #[serde(skip)]
    pub compiled_regex: Option<Regex>,

    /// Rule severity
    pub severity: RuleSeverity,

    /// Custom message template
    pub message_template: String,

    /// Where to apply the rule
    pub applies_to: RuleTarget,

    /// Rule configuration
    #[serde(default)]
    pub config: HashMap<String, toml::Value>,

    /// Whether rule is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Minimum confidence level
    #[serde(default = "default_confidence")]
    pub confidence: f64,
}

fn default_true() -> bool {
    true
}

fn default_confidence() -> f64 {
    0.8
}

/// What the rule applies to
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleTarget {
    /// Apply to error messages
    ErrorMessages,
    /// Apply to warning messages
    WarningMessages,
    /// Apply to all messages
    AllMessages,
    /// Apply to file paths
    FilePaths,
    /// Apply to source code content
    SourceCode,
    /// Apply to variable names
    VariableNames,
    /// Apply to function names
    FunctionNames,
}

impl CustomRule {
    /// Create new custom rule
    pub fn new(
        id: String,
        name: String,
        pattern_type: PatternType,
        pattern: String,
        severity: RuleSeverity,
        message_template: String,
        applies_to: RuleTarget,
    ) -> Result<Self> {
        let compiled_regex = if matches!(pattern_type, PatternType::Regex) {
            Some(Regex::new(&pattern).context("Invalid regex pattern")?)
        } else {
            None
        };

        Ok(Self {
            id,
            name,
            description: String::new(),
            pattern_type,
            pattern,
            compiled_regex,
            severity,
            message_template,
            applies_to,
            config: HashMap::new(),
            enabled: true,
            confidence: 0.8,
        })
    }

    /// Apply this custom rule to analysis results
    pub fn apply(&self, results: &AnalysisResults) -> Result<Vec<AnalysisError>> {
        if !self.enabled {
            return Ok(Vec::new());
        }

        let mut new_errors = Vec::new();

        match self.applies_to {
            RuleTarget::ErrorMessages => {
                for error in results.get_errors() {
                    if self.matches_text(&error.message)? {
                        if let Some(custom_error) = self.create_custom_error(error)? {
                            new_errors.push(custom_error);
                        }
                    }
                }
            }

            RuleTarget::WarningMessages => {
                for warning in results.get_warnings() {
                    if self.matches_text(&warning.message)? {
                        if let Some(custom_error) = self.create_custom_error(warning)? {
                            new_errors.push(custom_error);
                        }
                    }
                }
            }

            RuleTarget::AllMessages => {
                for error in results
                    .get_errors()
                    .iter()
                    .chain(results.get_warnings().iter())
                {
                    if self.matches_text(&error.message)? {
                        if let Some(custom_error) = self.create_custom_error(error)? {
                            new_errors.push(custom_error);
                        }
                    }
                }
            }

            RuleTarget::FilePaths => {
                for error in results
                    .get_errors()
                    .iter()
                    .chain(results.get_warnings().iter())
                {
                    let file_path_str = error.file_path.to_string_lossy();
                    if self.matches_text(&file_path_str)? {
                        if let Some(custom_error) = self.create_custom_error(error)? {
                            new_errors.push(custom_error);
                        }
                    }
                }
            }

            RuleTarget::SourceCode | RuleTarget::VariableNames | RuleTarget::FunctionNames => {
                // These would require AST analysis or source code access
                // For now, return empty - to be implemented later
            }
        }

        Ok(new_errors)
    }

    /// Check if text matches the rule pattern
    fn matches_text(&self, text: &str) -> Result<bool> {
        match self.pattern_type {
            PatternType::Contains => Ok(text.contains(&self.pattern)),

            PatternType::Regex => {
                if let Some(ref regex) = self.compiled_regex {
                    Ok(regex.is_match(text))
                } else {
                    let regex =
                        Regex::new(&self.pattern).context("Failed to compile regex pattern")?;
                    Ok(regex.is_match(text))
                }
            }

            PatternType::Exact => Ok(text == self.pattern),

            PatternType::Ast => {
                // AST matching not implemented yet
                Ok(false)
            }
        }
    }

    /// Create custom error from matched error
    fn create_custom_error(&self, original_error: &AnalysisError) -> Result<Option<AnalysisError>> {
        let custom_message = self.format_message(&original_error.message);

        let custom_error = AnalysisError {
            message: custom_message,
            file_path: original_error.file_path.clone(),
            position: original_error.position,
            level: self.severity.into(),
            error_code: Some(self.id.clone()),
            suggestion: self.get_suggestion(),
            related_positions: original_error.related_positions.clone(),
        };

        Ok(Some(custom_error))
    }

    /// Format message using template
    fn format_message(&self, original_message: &str) -> String {
        self.message_template
            .replace("{original_message}", original_message)
            .replace("{rule_name}", &self.name)
            .replace("{rule_id}", &self.id)
    }

    /// Get suggestion for this rule
    fn get_suggestion(&self) -> Option<String> {
        self.config
            .get("suggestion")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    /// Validate rule configuration
    pub fn validate(&self) -> Result<Vec<String>> {
        let mut issues = Vec::new();

        // Validate ID
        if self.id.is_empty() {
            issues.push("Rule ID cannot be empty".to_string());
        }

        if !self.id.starts_with("CUSTOM") && !self.id.starts_with("USER") {
            issues.push("Custom rule ID should start with 'CUSTOM' or 'USER'".to_string());
        }

        // Validate pattern
        if self.pattern.is_empty() {
            issues.push("Rule pattern cannot be empty".to_string());
        }

        // Validate regex if needed
        if matches!(self.pattern_type, PatternType::Regex) {
            if let Err(e) = Regex::new(&self.pattern) {
                issues.push(format!("Invalid regex pattern: {}", e));
            }
        }

        // Validate confidence
        if self.confidence < 0.0 || self.confidence > 1.0 {
            issues.push(format!(
                "Confidence must be between 0.0 and 1.0, got {}",
                self.confidence
            ));
        }

        // Validate message template
        if self.message_template.is_empty() {
            issues.push("Message template cannot be empty".to_string());
        }

        Ok(issues)
    }
}

/// Manager for custom rules
pub struct CustomRulesManager {
    /// Custom rules by ID
    rules: HashMap<String, CustomRule>,
}

impl CustomRulesManager {
    /// Create new custom rules manager
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
        }
    }

    /// Load custom rules from TOML file
    pub fn load_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(&path).with_context(|| {
            format!(
                "Failed to read custom rules from {}",
                path.as_ref().display()
            )
        })?;

        let rules_data: HashMap<String, CustomRule> =
            toml::from_str(&content).context("Failed to parse custom rules TOML")?;

        let mut manager = Self::new();
        for (id, mut rule) in rules_data {
            rule.id = id.clone();

            // Compile regex if needed
            if matches!(rule.pattern_type, PatternType::Regex) {
                rule.compiled_regex = Some(
                    Regex::new(&rule.pattern)
                        .with_context(|| format!("Invalid regex in rule {}", id))?,
                );
            }

            manager.add_rule(rule)?;
        }

        Ok(manager)
    }

    /// Add custom rule
    pub fn add_rule(&mut self, rule: CustomRule) -> Result<()> {
        let issues = rule.validate()?;
        if !issues.is_empty() {
            return Err(anyhow::anyhow!(
                "Rule validation failed: {}",
                issues.join(", ")
            ));
        }

        self.rules.insert(rule.id.clone(), rule);
        Ok(())
    }

    /// Remove custom rule
    pub fn remove_rule(&mut self, rule_id: &str) -> Option<CustomRule> {
        self.rules.remove(rule_id)
    }

    /// Get custom rule
    pub fn get_rule(&self, rule_id: &str) -> Option<&CustomRule> {
        self.rules.get(rule_id)
    }

    /// Get all custom rules
    pub fn get_all_rules(&self) -> &HashMap<String, CustomRule> {
        &self.rules
    }

    /// Apply all custom rules to analysis results
    pub fn apply_all_rules(&self, results: &AnalysisResults) -> Result<Vec<AnalysisError>> {
        let mut all_custom_errors = Vec::new();

        for rule in self.rules.values() {
            if rule.enabled {
                let custom_errors = rule.apply(results)?;
                all_custom_errors.extend(custom_errors);
            }
        }

        Ok(all_custom_errors)
    }

    /// Export custom rules to TOML file
    pub fn export_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(&self.rules)
            .context("Failed to serialize custom rules to TOML")?;

        std::fs::write(&path, content).with_context(|| {
            format!(
                "Failed to write custom rules to {}",
                path.as_ref().display()
            )
        })?;

        Ok(())
    }

    /// Validate all custom rules
    pub fn validate_all(&self) -> Result<HashMap<String, Vec<String>>> {
        let mut all_issues = HashMap::new();

        for (rule_id, rule) in &self.rules {
            let issues = rule.validate()?;
            if !issues.is_empty() {
                all_issues.insert(rule_id.clone(), issues);
            }
        }

        Ok(all_issues)
    }

    /// Create example custom rules file
    pub fn create_example_file<P: AsRef<std::path::Path>>(path: P) -> Result<()> {
        let mut manager = Self::new();

        // Example 1: Russian language check
        let rule1 = CustomRule::new(
            "CUSTOM001".to_string(),
            "Russian Comments Only".to_string(),
            PatternType::Regex,
            r"//\s*[a-zA-Z]".to_string(),
            RuleSeverity::Info,
            "Comment should be in Russian: {original_message}".to_string(),
            RuleTarget::SourceCode,
        )?;

        // Example 2: Forbidden function names
        let rule2 = CustomRule::new(
            "CUSTOM002".to_string(),
            "No Hungarian Notation".to_string(),
            PatternType::Regex,
            r"\b(str|int|bool|obj)[A-Z]".to_string(),
            RuleSeverity::Hint,
            "Avoid Hungarian notation in variable names".to_string(),
            RuleTarget::VariableNames,
        )?;

        // Example 3: File naming convention
        let rule3 = CustomRule::new(
            "CUSTOM003".to_string(),
            "Module Naming Convention".to_string(),
            PatternType::Regex,
            r".*[А-Я].*\.bsl$".to_string(),
            RuleSeverity::Warning,
            "Module file names should be in English".to_string(),
            RuleTarget::FilePaths,
        )?;

        manager.add_rule(rule1)?;
        manager.add_rule(rule2)?;
        manager.add_rule(rule3)?;

        manager.export_to_file(path)?;
        Ok(())
    }
}

impl Default for CustomRulesManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::errors::ErrorLevel;
    use crate::core::position::Position;
    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    fn create_test_error(message: &str) -> AnalysisError {
        AnalysisError {
            message: message.to_string(),
            file_path: PathBuf::from("test.bsl"),
            position: Position {
                line: 1,
                column: 1,
                offset: 0,
            },
            level: ErrorLevel::Error,
            error_code: Some("TEST001".to_string()),
            suggestion: None,
            related_positions: Vec::new(),
        }
    }

    #[test]
    fn test_custom_rule_creation() {
        let rule = CustomRule::new(
            "CUSTOM001".to_string(),
            "Test Rule".to_string(),
            PatternType::Contains,
            "test".to_string(),
            RuleSeverity::Warning,
            "Test message: {original_message}".to_string(),
            RuleTarget::ErrorMessages,
        )
        .unwrap();

        assert_eq!(rule.id, "CUSTOM001");
        assert_eq!(rule.name, "Test Rule");
        assert!(rule.enabled);
    }

    #[test]
    fn test_pattern_matching() {
        let rule = CustomRule::new(
            "CUSTOM001".to_string(),
            "Contains Test".to_string(),
            PatternType::Contains,
            "unused".to_string(),
            RuleSeverity::Info,
            "Found: {original_message}".to_string(),
            RuleTarget::ErrorMessages,
        )
        .unwrap();

        assert!(rule.matches_text("unused variable").unwrap());
        assert!(!rule.matches_text("undefined variable").unwrap());
    }

    #[test]
    fn test_regex_pattern() {
        let rule = CustomRule::new(
            "CUSTOM002".to_string(),
            "Regex Test".to_string(),
            PatternType::Regex,
            r"\bunused\b".to_string(),
            RuleSeverity::Warning,
            "Regex match: {original_message}".to_string(),
            RuleTarget::ErrorMessages,
        )
        .unwrap();

        assert!(rule.matches_text("unused variable").unwrap());
        assert!(!rule.matches_text("unusedVariable").unwrap());
    }

    #[test]
    fn test_rule_application() {
        let rule = CustomRule::new(
            "CUSTOM001".to_string(),
            "Test Rule".to_string(),
            PatternType::Contains,
            "unused".to_string(),
            RuleSeverity::Hint,
            "Custom: {original_message}".to_string(),
            RuleTarget::ErrorMessages,
        )
        .unwrap();

        let mut results = crate::core::AnalysisResults::new();
        results.add_error(create_test_error("unused variable"));
        results.add_error(create_test_error("undefined variable"));

        let custom_errors = rule.apply(&results).unwrap();
        assert_eq!(custom_errors.len(), 1);
        assert!(custom_errors[0].message.contains("Custom:"));
        assert_eq!(custom_errors[0].level, ErrorLevel::Hint);
    }

    #[test]
    fn test_custom_rules_manager() {
        let mut manager = CustomRulesManager::new();

        let rule = CustomRule::new(
            "CUSTOM001".to_string(),
            "Test Rule".to_string(),
            PatternType::Contains,
            "test".to_string(),
            RuleSeverity::Info,
            "Test: {original_message}".to_string(),
            RuleTarget::ErrorMessages,
        )
        .unwrap();

        manager.add_rule(rule).unwrap();

        assert!(manager.get_rule("CUSTOM001").is_some());
        assert_eq!(manager.get_all_rules().len(), 1);
    }

    #[test]
    fn test_rule_validation() {
        // Test that creating a rule with invalid regex fails
        let result = CustomRule::new(
            "CUSTOM001".to_string(),
            "Test Rule".to_string(),
            PatternType::Regex,
            "[invalid regex".to_string(),
            RuleSeverity::Error,
            "Test".to_string(),
            RuleTarget::ErrorMessages,
        );

        // Should fail because the regex is invalid
        assert!(result.is_err());
    }

    #[test]
    fn test_file_roundtrip() {
        let mut manager = CustomRulesManager::new();

        let rule = CustomRule::new(
            "CUSTOM001".to_string(),
            "Test Rule".to_string(),
            PatternType::Contains,
            "test".to_string(),
            RuleSeverity::Warning,
            "Test: {original_message}".to_string(),
            RuleTarget::ErrorMessages,
        )
        .unwrap();

        manager.add_rule(rule).unwrap();

        let temp_file = NamedTempFile::new().unwrap();
        manager.export_to_file(temp_file.path()).unwrap();

        let loaded_manager = CustomRulesManager::load_from_file(temp_file.path()).unwrap();
        assert_eq!(loaded_manager.get_all_rules().len(), 1);
        assert!(loaded_manager.get_rule("CUSTOM001").is_some());
    }
}
