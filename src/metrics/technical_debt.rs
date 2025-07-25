/*!
# Technical Debt Analysis for BSL Code

Identifies and categorizes technical debt in BSL code.
*/

use std::collections::HashMap;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Type of technical debt
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DebtType {
    Design,
    CodeQuality,  
    Performance,
    Security,
    Documentation,
}

/// Severity of technical debt
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DebtSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// A single technical debt item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtItem {
    pub debt_type: DebtType,
    pub severity: DebtSeverity,
    pub description: String,
    pub estimated_minutes: u32,
    pub line_number: Option<u32>,
}

/// Analysis of technical debt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalDebtAnalysis {
    pub total_debt_minutes: u32,
    pub debt_items: Vec<DebtItem>,
    pub debt_by_type: HashMap<DebtType, u32>,
    pub debt_by_severity: HashMap<DebtSeverity, u32>,
}

/// Analyzer for technical debt
pub struct TechnicalDebtAnalyzer {
    // Configuration
}

impl TechnicalDebtAnalyzer {
    pub fn new() -> Self {
        Self {}
    }
    
    /// Analyze technical debt in BSL content
    pub fn analyze_content(&mut self, content: &str) -> Result<TechnicalDebtAnalysis> {
        let mut debt_items = Vec::new();
        let mut debt_by_type = HashMap::new();
        let mut debt_by_severity = HashMap::new();
        
        // Simple pattern-based debt detection
        self.detect_code_smells(content, &mut debt_items);
        self.detect_performance_issues(content, &mut debt_items);
        self.detect_documentation_issues(content, &mut debt_items);
        
        // Calculate totals
        let total_debt_minutes = debt_items.iter().map(|item| item.estimated_minutes).sum();
        
        for item in &debt_items {
            *debt_by_type.entry(item.debt_type).or_insert(0) += item.estimated_minutes;
            *debt_by_severity.entry(item.severity).or_insert(0) += item.estimated_minutes;
        }
        
        Ok(TechnicalDebtAnalysis {
            total_debt_minutes,
            debt_items,
            debt_by_type,
            debt_by_severity,
        })
    }
    
    fn detect_code_smells(&self, content: &str, debt_items: &mut Vec<DebtItem>) {
        // Long functions
        for (line_num, line) in content.lines().enumerate() {
            if line.contains("Функция") || line.contains("Процедура") {
                let function_lines = content.lines().skip(line_num).take_while(|l| !l.contains("КонецФункции") && !l.contains("КонецПроцедуры")).count();
                if function_lines > 50 {
                    debt_items.push(DebtItem {
                        debt_type: DebtType::CodeQuality,
                        severity: DebtSeverity::Medium,
                        description: "Function is too long (>50 lines)".to_string(),
                        estimated_minutes: 30,
                        line_number: Some(line_num as u32 + 1),
                    });
                }
            }
        }
        
        // Magic numbers
        if content.matches(char::is_numeric).count() > 5 {
            debt_items.push(DebtItem {
                debt_type: DebtType::CodeQuality,
                severity: DebtSeverity::Low,
                description: "Multiple magic numbers found".to_string(),
                estimated_minutes: 15,
                line_number: None,
            });
        }
    }
    
    fn detect_performance_issues(&self, content: &str, debt_items: &mut Vec<DebtItem>) {
        // Nested loops
        let loop_count = content.matches("Цикл").count() + content.matches("Для").count();
        if loop_count > 2 {
            debt_items.push(DebtItem {
                debt_type: DebtType::Performance,
                severity: DebtSeverity::Medium,
                description: "Multiple nested loops detected".to_string(),
                estimated_minutes: 45,
                line_number: None,
            });
        }
        
        // String concatenation in loops
        if content.contains("Цикл") && content.contains("+ \"") {
            debt_items.push(DebtItem {
                debt_type: DebtType::Performance,
                severity: DebtSeverity::High,
                description: "String concatenation in loop".to_string(),
                estimated_minutes: 60,
                line_number: None,
            });
        }
    }
    
    fn detect_documentation_issues(&self, content: &str, debt_items: &mut Vec<DebtItem>) {
        let function_count = content.matches("Функция").count() + content.matches("Процедура").count();
        let comment_count = content.matches("//").count();
        
        if function_count > 0 && comment_count == 0 {
            debt_items.push(DebtItem {
                debt_type: DebtType::Documentation,
                severity: DebtSeverity::Medium,
                description: "No comments found in code with functions".to_string(),
                estimated_minutes: 20,
                line_number: None,
            });
        }
    }
}

impl Default for TechnicalDebtAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}