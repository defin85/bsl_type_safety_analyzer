/*!
# Intelligent Recommendations Engine

Generates actionable recommendations based on code quality metrics.
*/

use super::{ComplexityMetrics, DebtSeverity, MaintainabilityMetrics, TechnicalDebtAnalysis};

/// Engine for generating intelligent recommendations
pub struct RecommendationsEngine {
    // Configuration
}

impl RecommendationsEngine {
    pub fn new() -> Self {
        Self {}
    }

    /// Generate recommendations based on metrics
    pub fn generate_recommendations(
        &self,
        complexity: &ComplexityMetrics,
        maintainability: &MaintainabilityMetrics,
        debt: &TechnicalDebtAnalysis,
        duplication: f64,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Complexity recommendations
        if complexity.average_cyclomatic_complexity > 10.0 {
            recommendations.push(
                "Consider breaking down complex functions into smaller, more focused functions"
                    .to_string(),
            );
        }

        if complexity.max_cyclomatic_complexity > 15 {
            recommendations.push(
                "Identify the most complex function and refactor it using Extract Method pattern"
                    .to_string(),
            );
        }

        // Maintainability recommendations
        if maintainability.maintainability_index < 60.0 {
            recommendations.push("Focus on improving code maintainability by reducing complexity and adding documentation".to_string());
        }

        if maintainability.lines_of_code > 500 {
            recommendations
                .push("Consider splitting this file into multiple smaller modules".to_string());
        }

        // Technical debt recommendations
        let critical_debt = debt
            .debt_by_severity
            .get(&DebtSeverity::Critical)
            .unwrap_or(&0);
        let high_debt = debt.debt_by_severity.get(&DebtSeverity::High).unwrap_or(&0);

        if *critical_debt > 0 {
            recommendations.push("Address critical technical debt items immediately".to_string());
        }

        if *high_debt > 60 {
            // More than 1 hour of high-severity debt
            recommendations.push(
                "Prioritize fixing high-severity technical debt in the next sprint".to_string(),
            );
        }

        if debt.total_debt_minutes > 240 {
            // More than 4 hours total
            recommendations
                .push("Consider dedicating time each sprint to reduce technical debt".to_string());
        }

        // Duplication recommendations
        if duplication > 15.0 {
            recommendations.push(
                "Extract common code into reusable functions to reduce duplication".to_string(),
            );
        }

        if duplication > 25.0 {
            recommendations
                .push("High code duplication detected - consider major refactoring".to_string());
        }

        // General recommendations
        if recommendations.is_empty() {
            recommendations.push(
                "Code quality looks good! Consider adding more comprehensive tests".to_string(),
            );
            recommendations.push(
                "Keep monitoring metrics to maintain good code quality over time".to_string(),
            );
        }

        // Prioritize recommendations
        if recommendations.len() > 5 {
            recommendations.truncate(5);
            recommendations.push(
                "Additional recommendations available - focus on top priorities first".to_string(),
            );
        }

        recommendations
    }
}

impl Default for RecommendationsEngine {
    fn default() -> Self {
        Self::new()
    }
}
