/*!
# Code Quality Metrics System

Comprehensive code quality measurement and technical debt analysis for BSL code.
Provides metrics for complexity, maintainability, and technical debt identification.
*/

pub mod complexity;
pub mod duplication;
pub mod maintainability;
pub mod recommendations;
pub mod technical_debt;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub use complexity::{ComplexityAnalyzer, ComplexityMetrics, FunctionComplexityMetrics};
pub use duplication::DuplicationAnalyzer;
pub use maintainability::{MaintainabilityAnalyzer, MaintainabilityMetrics};
pub use recommendations::RecommendationsEngine;
pub use technical_debt::{
    DebtItem, DebtSeverity, DebtType, TechnicalDebtAnalysis, TechnicalDebtAnalyzer,
};

/// Complete quality metrics for a BSL file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// Overall quality score (0-100)
    pub quality_score: f64,

    /// Maintainability index
    pub maintainability_index: f64,

    /// Complexity metrics
    pub complexity_metrics: ComplexityMetrics,

    /// Technical debt analysis
    pub technical_debt: TechnicalDebtAnalysis,

    /// Code duplication percentage
    pub duplication_percentage: f64,

    /// Intelligent recommendations
    pub recommendations: Vec<String>,
}

/// Manager for all quality metrics analysis
pub struct QualityMetricsManager {
    complexity_analyzer: ComplexityAnalyzer,
    maintainability_analyzer: MaintainabilityAnalyzer,
    debt_analyzer: TechnicalDebtAnalyzer,
    duplication_analyzer: DuplicationAnalyzer,
    recommendations_engine: RecommendationsEngine,
}

impl QualityMetricsManager {
    /// Create new quality metrics manager
    pub fn new() -> Self {
        Self {
            complexity_analyzer: ComplexityAnalyzer::new(),
            maintainability_analyzer: MaintainabilityAnalyzer::new(),
            debt_analyzer: TechnicalDebtAnalyzer::new(),
            duplication_analyzer: DuplicationAnalyzer::new(),
            recommendations_engine: RecommendationsEngine::new(),
        }
    }

    /// Analyze a BSL file for quality metrics
    pub fn analyze_file<P: AsRef<Path>>(
        &mut self,
        file_path: P,
        content: &str,
    ) -> Result<QualityMetrics> {
        self.analyze_content(file_path.as_ref().to_string_lossy().as_ref(), content)
    }

    /// Analyze BSL content for quality metrics
    pub fn analyze_content(&mut self, _filename: &str, content: &str) -> Result<QualityMetrics> {
        // Analyze complexity
        let complexity_metrics = self.complexity_analyzer.analyze_content(content)?;

        // Analyze maintainability
        let maintainability_metrics = self.maintainability_analyzer.analyze_content(content)?;

        // Analyze technical debt
        let technical_debt = self.debt_analyzer.analyze_content(content)?;

        // Analyze duplication
        let duplication_percentage = self.duplication_analyzer.analyze_content(content)?;

        // Calculate overall quality score
        let quality_score = self.calculate_quality_score(
            &complexity_metrics,
            &maintainability_metrics,
            &technical_debt,
            duplication_percentage,
        );

        // Generate recommendations
        let recommendations = self.recommendations_engine.generate_recommendations(
            &complexity_metrics,
            &maintainability_metrics,
            &technical_debt,
            duplication_percentage,
        );

        Ok(QualityMetrics {
            quality_score,
            maintainability_index: maintainability_metrics.maintainability_index,
            complexity_metrics,
            technical_debt,
            duplication_percentage,
            recommendations,
        })
    }

    /// Calculate overall quality score (0-100)
    fn calculate_quality_score(
        &self,
        complexity: &ComplexityMetrics,
        maintainability: &MaintainabilityMetrics,
        debt: &TechnicalDebtAnalysis,
        duplication: f64,
    ) -> f64 {
        // Weighted scoring system
        let complexity_score = if complexity.average_cyclomatic_complexity <= 2.0 {
            100.0
        } else if complexity.average_cyclomatic_complexity <= 5.0 {
            80.0
        } else if complexity.average_cyclomatic_complexity <= 10.0 {
            60.0
        } else {
            30.0
        };

        let maintainability_score = maintainability.maintainability_index;

        let debt_score = match debt.debt_items.len() {
            0 => 100.0,
            1..=3 => 85.0,
            4..=7 => 70.0,
            8..=15 => 50.0,
            _ => 25.0,
        };

        let duplication_score = if duplication < 5.0 {
            100.0
        } else if duplication < 10.0 {
            80.0
        } else if duplication < 20.0 {
            60.0
        } else {
            30.0
        };

        // Weighted average
        (complexity_score * 0.3
            + maintainability_score * 0.3
            + debt_score * 0.25
            + duplication_score * 0.15)
            .clamp(0.0, 100.0)
    }
}

impl Default for QualityMetricsManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_manager_creation() {
        let _manager = QualityMetricsManager::new();
        // Just test that it creates without errors
    }

    #[test]
    fn test_quality_score_calculation() {
        let manager = QualityMetricsManager::new();

        let complexity = ComplexityMetrics {
            average_cyclomatic_complexity: 2.0,
            max_cyclomatic_complexity: 5,
            function_metrics: std::collections::HashMap::new(),
            average_cognitive_complexity: 2.0,
            max_cognitive_complexity: 5,
        };

        let maintainability = MaintainabilityMetrics {
            maintainability_index: 85.0,
            halstead_volume: 100.0,
            cyclomatic_complexity: 2.0,
            lines_of_code: 50,
        };

        let debt = TechnicalDebtAnalysis {
            total_debt_minutes: 60,
            debt_items: Vec::new(),
            debt_by_type: std::collections::HashMap::new(),
            debt_by_severity: std::collections::HashMap::new(),
        };

        let score = manager.calculate_quality_score(&complexity, &maintainability, &debt, 5.0);
        assert!(score >= 80.0); // Should be high quality
    }

    #[test]
    fn test_analyze_simple_content() {
        let mut manager = QualityMetricsManager::new();

        let content = r#"
        Функция ПростаяФункция()
            Возврат "Привет мир";
        КонецФункции
        "#;

        let result = manager.analyze_content("test.bsl", content);
        assert!(result.is_ok());

        let metrics = result.unwrap();
        assert!(metrics.quality_score >= 0.0 && metrics.quality_score <= 100.0);
    }
}
