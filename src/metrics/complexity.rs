/*!
# Complexity Analysis for BSL Code

Calculates cyclomatic and cognitive complexity metrics for BSL functions and procedures.
*/

use std::collections::HashMap;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Complexity metrics for a single function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionComplexityMetrics {
    pub name: String,
    pub cyclomatic_complexity: u32,
    pub cognitive_complexity: u32,
    pub lines_of_code: u32,
    pub parameters_count: u32,
}

/// Overall complexity metrics for a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityMetrics {
    pub average_cyclomatic_complexity: f64,
    pub max_cyclomatic_complexity: u32,
    pub function_metrics: HashMap<String, FunctionComplexityMetrics>,
    pub average_cognitive_complexity: f64,
    pub max_cognitive_complexity: u32,
}

/// Analyzer for code complexity
pub struct ComplexityAnalyzer {
    // Configuration or state if needed
}

impl ComplexityAnalyzer {
    pub fn new() -> Self {
        Self {}
    }
    
    /// Analyze complexity of BSL content
    pub fn analyze_content(&mut self, content: &str) -> Result<ComplexityMetrics> {
        // Simple implementation for demo
        // In a real implementation, this would parse the BSL code and calculate complexity
        
        let mut function_metrics = HashMap::new();
        
        // Mock analysis - count simple patterns
        let function_count = content.matches("Функция").count() + content.matches("Процедура").count();
        let if_count = content.matches("Если").count();
        let loop_count = content.matches("Цикл").count() + content.matches("Для").count() + content.matches("Пока").count();
        
        if function_count > 0 {
            // Create a sample function metric
            let func_name = "TestFunction".to_string();
            let complexity = (if_count + loop_count + 1) as u32; // Base complexity + decisions
            
            function_metrics.insert(func_name.clone(), FunctionComplexityMetrics {
                name: func_name,
                cyclomatic_complexity: complexity,
                cognitive_complexity: complexity + (loop_count as u32), // Cognitive is typically higher
                lines_of_code: content.lines().count() as u32,
                parameters_count: 0, // Would need proper parsing
            });
        }
        
        let avg_cyclomatic = if function_metrics.is_empty() {
            1.0
        } else {
            function_metrics.values()
                .map(|f| f.cyclomatic_complexity as f64)
                .sum::<f64>() / function_metrics.len() as f64
        };
        
        let max_cyclomatic = function_metrics.values()
            .map(|f| f.cyclomatic_complexity)
            .max()
            .unwrap_or(1);
            
        let avg_cognitive = if function_metrics.is_empty() {
            1.0
        } else {
            function_metrics.values()
                .map(|f| f.cognitive_complexity as f64)
                .sum::<f64>() / function_metrics.len() as f64
        };
        
        let max_cognitive = function_metrics.values()
            .map(|f| f.cognitive_complexity)
            .max()
            .unwrap_or(1);
        
        Ok(ComplexityMetrics {
            average_cyclomatic_complexity: avg_cyclomatic,
            max_cyclomatic_complexity: max_cyclomatic,
            function_metrics,
            average_cognitive_complexity: avg_cognitive,
            max_cognitive_complexity: max_cognitive,
        })
    }
}

impl Default for ComplexityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}