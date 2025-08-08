/*!
# Maintainability Analysis for BSL Code

Calculates maintainability index and related metrics for BSL code.
*/

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Maintainability metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintainabilityMetrics {
    /// Maintainability Index (0-100, higher is better)
    pub maintainability_index: f64,
    /// Halstead volume
    pub halstead_volume: f64,
    /// Cyclomatic complexity
    pub cyclomatic_complexity: f64,
    /// Lines of code
    pub lines_of_code: u32,
}

/// Analyzer for maintainability metrics
pub struct MaintainabilityAnalyzer {
    // Configuration if needed
}

impl MaintainabilityAnalyzer {
    pub fn new() -> Self {
        Self {}
    }

    /// Analyze maintainability of BSL content
    pub fn analyze_content(&mut self, content: &str) -> Result<MaintainabilityMetrics> {
        let lines_of_code = content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .count() as u32;

        // Simple complexity estimation
        let decisions = content.matches("Если").count()
            + content.matches("Цикл").count()
            + content.matches("Для").count()
            + content.matches("Пока").count();
        let cyclomatic_complexity = (decisions + 1) as f64;

        // Estimate Halstead volume (simplified)
        let operators = content.matches("=").count()
            + content.matches("+").count()
            + content.matches("-").count()
            + content.matches("И").count()
            + content.matches("ИЛИ").count();
        let operands = content.split_whitespace().count();
        let halstead_volume =
            ((operators + operands) as f64).log2() * (operators + operands) as f64;

        // Calculate Maintainability Index
        // MI = 171 - 5.2 * ln(Halstead Volume) - 0.23 * (Cyclomatic Complexity) - 16.2 * ln(Lines of Code)
        let mi = 171.0
            - 5.2 * halstead_volume.ln()
            - 0.23 * cyclomatic_complexity
            - 16.2 * (lines_of_code as f64).ln();
        let maintainability_index = mi.clamp(0.0, 100.0);

        Ok(MaintainabilityMetrics {
            maintainability_index,
            halstead_volume,
            cyclomatic_complexity,
            lines_of_code,
        })
    }
}

impl Default for MaintainabilityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
