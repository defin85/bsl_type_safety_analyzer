/*!
# Rules Engine

Core rules application engine that processes analysis results
through configured rules and applies filtering, severity changes,
and custom transformations.
*/

use std::collections::HashMap;
use std::time::Instant;
use anyhow::{Context, Result};

use crate::core::{AnalysisResults, AnalysisError};
use super::{RulesConfig, RuleId, RuleStats};

/// Result of applying a single rule
#[derive(Debug, Clone)]
pub struct RuleResult {
    /// Rule that was applied
    pub rule_id: RuleId,
    /// Whether the rule matched
    pub matched: bool,
    /// Original error (if any)
    pub original_error: Option<AnalysisError>,
    /// Transformed error (if rule modified it)
    pub transformed_error: Option<AnalysisError>,
    /// Execution time in microseconds
    pub execution_time_us: u64,
    /// Rule confidence (0.0 to 1.0)
    pub confidence: f64,
}

/// Application of rules to analysis results
#[derive(Debug, Clone)]
pub struct RuleApplication {
    /// Original analysis results
    pub original_results: AnalysisResults,
    /// Filtered/transformed results
    pub filtered_results: AnalysisResults,
    /// Individual rule results
    pub rule_results: Vec<RuleResult>,
    /// Total execution time
    pub total_execution_time_us: u64,
    /// Statistics
    pub stats: HashMap<RuleId, RuleStats>,
}

/// Rules application engine
pub struct RulesEngine {
    /// Configuration
    config: RulesConfig,
    /// Rule statistics cache
    stats_cache: HashMap<RuleId, RuleStats>,
    /// Performance metrics
    performance_enabled: bool,
}

impl RulesEngine {
    /// Create new rules engine with configuration
    pub fn new(config: RulesConfig) -> Self {
        Self {
            config,
            stats_cache: HashMap::new(),
            performance_enabled: true,
        }
    }
    
    /// Enable/disable performance tracking
    pub fn set_performance_tracking(&mut self, enabled: bool) {
        self.performance_enabled = enabled;
    }
    
    /// Apply all configured rules to analysis results
    pub fn apply_rules(&mut self, results: &AnalysisResults) -> Result<AnalysisResults> {
        let start_time = Instant::now();
        
        let _filtered_results = results.clone();
        let mut rule_results = Vec::new();
        
        // Get active profile
        let _profile = self.config.get_active_profile()
            .context("Failed to get active profile")?;
        
        // Process errors
        let mut filtered_errors = Vec::new();
        for error in results.get_errors() {
            let (should_keep, transformed_error, rule_result) = 
                self.apply_rules_to_error(error, true)?;
            
            if should_keep {
                filtered_errors.push(transformed_error);
            }
            
            if let Some(result) = rule_result {
                rule_results.push(result);
            }
        }
        
        // Process warnings
        let mut filtered_warnings = Vec::new();
        for warning in results.get_warnings() {
            let (should_keep, transformed_warning, rule_result) = 
                self.apply_rules_to_error(warning, false)?;
            
            if should_keep {
                filtered_warnings.push(transformed_warning);
            }
            
            if let Some(result) = rule_result {
                rule_results.push(result);
            }
        }
        
        // Rebuild results with filtered errors/warnings
        let mut new_results = AnalysisResults::new();
        for error in filtered_errors {
            new_results.add_error(error);
        }
        for warning in filtered_warnings {
            new_results.add_warning(warning);
        }
        
        // Copy metadata
        new_results.set_files_analyzed(results.metadata().files_analyzed);
        if let Some(duration) = results.analysis_duration() {
            new_results.add_metric("original_analysis_time".to_string(), format!("{:?}", duration));
        }
        
        let total_time = start_time.elapsed().as_micros() as u64;
        new_results.add_metric("rules_processing_time_us".to_string(), total_time.to_string());
        
        // Update statistics
        self.update_stats(&rule_results);
        
        Ok(new_results)
    }
    
    /// Apply rules to a single error/warning
    fn apply_rules_to_error(
        &self, 
        error: &AnalysisError, 
        _is_error: bool
    ) -> Result<(bool, AnalysisError, Option<RuleResult>)> {
        let start_time = Instant::now();
        let mut transformed_error = error.clone();
        let mut should_keep = true;
        let mut matched_rule = None;
        
        // Get rule ID from error code
        let rule_id = error.error_code.as_deref().unwrap_or("UNKNOWN");
        
        // Check if rule is enabled
        if !self.config.is_rule_enabled(rule_id)? {
            should_keep = false;
            
            matched_rule = Some(RuleResult {
                rule_id: rule_id.to_string(),
                matched: true,
                original_error: Some(error.clone()),
                transformed_error: None,
                execution_time_us: start_time.elapsed().as_micros() as u64,
                confidence: 1.0,
            });
            
            return Ok((should_keep, transformed_error, matched_rule));
        }
        
        // Apply rule-specific transformations
        if let Some(rule_config) = self.config.get_rule(rule_id) {
            // Update severity if configured
            let new_level = rule_config.severity.into();
            if transformed_error.level != new_level {
                transformed_error.level = new_level;
            }
            
            // Apply custom message if configured
            if let Some(ref custom_message) = rule_config.message {
                transformed_error.message = custom_message.clone();
            }
            
            // Check confidence threshold
            if rule_config.min_confidence > 0.8 {
                // For now, assume all rules have confidence 1.0
                // In the future, rules could return confidence scores
                let confidence = 1.0;
                if confidence < rule_config.min_confidence {
                    should_keep = false;
                }
            }
            
            matched_rule = Some(RuleResult {
                rule_id: rule_id.to_string(),
                matched: true,
                original_error: Some(error.clone()),
                transformed_error: if transformed_error != *error { 
                    Some(transformed_error.clone()) 
                } else { 
                    None 
                },
                execution_time_us: start_time.elapsed().as_micros() as u64,
                confidence: 1.0,
            });
        }
        
        // Apply global settings
        if let Some(_max_errors) = self.config.settings.max_errors {
            // This would be implemented at a higher level to count total errors
        }
        
        Ok((should_keep, transformed_error, matched_rule))
    }
    
    /// Update rule statistics
    fn update_stats(&mut self, rule_results: &[RuleResult]) {
        for result in rule_results {
            let stats = self.stats_cache.entry(result.rule_id.clone()).or_default();
            
            let violations = if result.matched { 1 } else { 0 };
            stats.update(violations, result.execution_time_us);
        }
    }
    
    /// Get rule statistics
    pub fn get_stats(&self) -> &HashMap<RuleId, RuleStats> {
        &self.stats_cache
    }
    
    /// Clear statistics
    pub fn clear_stats(&mut self) {
        self.stats_cache.clear();
    }
    
    /// Get configuration
    pub fn config(&self) -> &RulesConfig {
        &self.config
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: RulesConfig) {
        self.config = config;
    }
    
    /// Validate rules against analysis results
    pub fn validate_rules(&self, results: &AnalysisResults) -> Result<Vec<String>> {
        let mut issues = Vec::new();
        
        // Check if any rules reference non-existent error codes
        for (rule_id, _rule_config) in &self.config.rules {
            let mut found = false;
            
            for error in results.get_errors() {
                if let Some(ref error_code) = error.error_code {
                    if error_code == rule_id {
                        found = true;
                        break;
                    }
                }
            }
            
            for warning in results.get_warnings() {
                if let Some(ref error_code) = warning.error_code {
                    if error_code == rule_id {
                        found = true;
                        break;
                    }
                }
            }
            
            if !found && rule_id.starts_with("BSL") {
                issues.push(format!("Rule '{}' not found in analysis results", rule_id));
            }
        }
        
        Ok(issues)
    }
    
    /// Get rule application summary
    pub fn get_summary(&self) -> RulesSummary {
        let total_rules = self.config.rules.len();
        let enabled_rules = self.config.rules.iter()
            .filter(|(rule_id, _)| self.config.is_rule_enabled(rule_id).unwrap_or(false))
            .count();
        
        let total_applications: u64 = self.stats_cache.values()
            .map(|stats| stats.applications)
            .sum();
        
        let total_violations: u64 = self.stats_cache.values()
            .map(|stats| stats.violations)
            .sum();
        
        let avg_execution_time: f64 = if !self.stats_cache.is_empty() {
            self.stats_cache.values()
                .map(|stats| stats.avg_execution_time_us)
                .sum::<f64>() / self.stats_cache.len() as f64
        } else {
            0.0
        };
        
        RulesSummary {
            total_rules,
            enabled_rules,
            total_applications,
            total_violations,
            avg_execution_time_us: avg_execution_time,
            active_profile: self.config.active_profile.clone(),
        }
    }
}

/// Summary of rules application
#[derive(Debug, Clone)]
pub struct RulesSummary {
    /// Total number of configured rules
    pub total_rules: usize,
    /// Number of enabled rules
    pub enabled_rules: usize,
    /// Total rule applications
    pub total_applications: u64,
    /// Total violations found
    pub total_violations: u64,
    /// Average execution time per rule
    pub avg_execution_time_us: f64,
    /// Active profile name
    pub active_profile: String,
}

impl std::fmt::Display for RulesSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Rules Summary:")?;
        writeln!(f, "  Total rules: {}", self.total_rules)?;
        writeln!(f, "  Enabled rules: {}", self.enabled_rules)?;
        writeln!(f, "  Active profile: {}", self.active_profile)?;
        writeln!(f, "  Total applications: {}", self.total_applications)?;
        writeln!(f, "  Total violations: {}", self.total_violations)?;
        writeln!(f, "  Avg execution time: {:.2}μs", self.avg_execution_time_us)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::errors::{AnalysisError, ErrorLevel};
    use crate::parser::ast::Position;
    use std::path::PathBuf;
    
    fn create_test_error(code: &str, message: &str) -> AnalysisError {
        AnalysisError {
            message: message.to_string(),
            file_path: PathBuf::from("test.bsl"),
            position: Position { line: 1, column: 1, offset: 0 },
            level: ErrorLevel::Error,
            error_code: Some(code.to_string()),
            suggestion: None,
            related_positions: Vec::new(),
        }
    }
    
    #[test]
    fn test_rules_engine_creation() {
        let config = RulesConfig::default();
        let engine = RulesEngine::new(config);
        assert!(!engine.config().rules.is_empty());
    }
    
    #[test]
    fn test_rule_application() {
        let config = RulesConfig::default();
        let mut engine = RulesEngine::new(config);
        
        let mut results = AnalysisResults::new();
        results.add_error(create_test_error("BSL001", "Unused variable"));
        
        let filtered = engine.apply_rules(&results).unwrap();
        
        // BSL001 should be enabled by default, so error should remain
        assert_eq!(filtered.get_errors().len(), 1);
    }
    
    #[test]
    fn test_rule_filtering() {
        let mut config = RulesConfig::default();
        
        // Disable BSL001 rule
        if let Some(rule) = config.rules.get_mut("BSL001") {
            rule.enabled = false;
        }
        
        let mut engine = RulesEngine::new(config);
        
        let mut results = AnalysisResults::new();
        results.add_error(create_test_error("BSL001", "Unused variable"));
        
        let filtered = engine.apply_rules(&results).unwrap();
        
        // BSL001 is disabled, so error should be filtered out
        assert_eq!(filtered.get_errors().len(), 0);
    }
    
    #[test]
    fn test_severity_transformation() {
        let mut config = RulesConfig::default();
        
        // Change BSL001 severity to Info
        if let Some(rule) = config.rules.get_mut("BSL001") {
            rule.severity = crate::rules::RuleSeverity::Info;
        }
        
        let mut engine = RulesEngine::new(config);
        
        let mut results = AnalysisResults::new();
        results.add_error(create_test_error("BSL001", "Unused variable"));
        
        let filtered = engine.apply_rules(&results).unwrap();
        
        // Error should be transformed but still present
        assert_eq!(filtered.get_errors().len(), 1);
        assert_eq!(filtered.get_errors()[0].level, ErrorLevel::Info);
    }
    
    #[test]
    fn test_stats_tracking() {
        let config = RulesConfig::default();
        let mut engine = RulesEngine::new(config);
        
        let mut results = AnalysisResults::new();
        results.add_error(create_test_error("BSL001", "Unused variable"));
        
        engine.apply_rules(&results).unwrap();
        
        let stats = engine.get_stats();
        assert!(stats.contains_key("BSL001"));
        
        let bsl001_stats = &stats["BSL001"];
        assert_eq!(bsl001_stats.applications, 1);
        assert_eq!(bsl001_stats.violations, 1);
    }
    
    #[test]
    fn test_rules_summary() {
        let config = RulesConfig::default();
        let engine = RulesEngine::new(config);
        
        let summary = engine.get_summary();
        assert!(summary.total_rules > 0);
        assert_eq!(summary.active_profile, "default");
    }
}