/*!
# BSL Analyzer Module

Main analysis engine with semantic analysis, type checking, and rule validation.
*/

pub mod engine;
pub mod rules;
pub mod semantic;
pub mod data_flow_analyzer;
pub mod validation_manager;
pub mod lexical_analyzer;
pub mod dependency_analyzer;

#[cfg(test)]
mod engine_test;

#[cfg(test)]
mod semantic_analyzer_integration_test;

#[cfg(test)]
mod data_flow_analyzer_integration_test;

#[cfg(test)]
mod lexical_analyzer_integration_test;

pub use semantic::{SemanticAnalyzer, SemanticAnalysisConfig};
pub use data_flow_analyzer::{DataFlowAnalyzer, VariableState};
pub use validation_manager::{ValidationManager, ValidationResult};
pub use lexical_analyzer::{LexicalAnalyzer, LexicalToken, LexicalTokenType, LexicalAnalysisConfig};
pub use dependency_analyzer::{DependencyAnalyzer, DependencyGraph, DependencyNode, DependencyEdge, DependencyCycle};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::collections::HashMap;

use crate::diagnostics::Diagnostic;
use crate::parser::ast::AstNode;
use crate::core::errors::{AnalysisError, ErrorCollector};

/// Контекст анализа для передачи данных между анализаторами
#[derive(Debug, Clone)]
pub struct AnalysisContext {
    /// Путь к анализируемому файлу
    pub file_path: String,
    /// Исходный код
    pub code: String,
    /// AST дерево
    pub ast: Option<AstNode>,
    /// Переменные
    pub variables: HashMap<String, serde_json::Value>,
    /// Диагностики (ошибки, предупреждения, информация)
    pub diagnostics: Vec<Diagnostic>,
}

impl AnalysisContext {
    /// Создает новый контекст анализа
    pub fn new(file_path: String, code: String) -> Self {
        Self {
            file_path,
            code,
            ast: None,
            variables: HashMap::new(),
            diagnostics: Vec::new(),
        }
    }

    /// Добавляет диагностику в контекст
    pub fn add_diagnostic(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Проверяет наличие ошибок в контексте
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter()
            .any(|d| matches!(d.level, crate::diagnostics::DiagnosticLevel::Error))
    }

    /// Проверяет наличие предупреждений в контексте
    pub fn has_warnings(&self) -> bool {
        self.diagnostics.iter()
            .any(|d| matches!(d.level, crate::diagnostics::DiagnosticLevel::Warning))
    }

    /// Возвращает общее количество диагностик
    pub fn get_total_issues(&self) -> usize {
        self.diagnostics.len()
    }
}

/// Analysis result with errors and warnings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub file_path: std::path::PathBuf,
    pub errors: Vec<AnalysisError>,
    pub warnings: Vec<AnalysisError>,
    pub metrics: AnalysisMetrics,
}

impl AnalysisResult {
    pub fn new(file_path: &Path) -> Self {
        Self {
            file_path: file_path.to_path_buf(),
            errors: Vec::new(),
            warnings: Vec::new(),
            metrics: AnalysisMetrics::default(),
        }
    }
    
    pub fn with_semantic_results(file_path: &Path, errors: Vec<AnalysisError>, warnings: Vec<AnalysisError>) -> Self {
        Self {
            file_path: file_path.to_path_buf(),
            errors,
            warnings,
            metrics: AnalysisMetrics::default(),
        }
    }
    
    pub fn has_issues(&self) -> bool {
        !self.errors.is_empty() || !self.warnings.is_empty()
    }
}

/// Basic analysis metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnalysisMetrics {
    pub lines_analyzed: usize,
    pub procedures_count: usize,
    pub functions_count: usize,
    pub variables_count: usize,
}

/// Main analyzer that coordinates all analysis phases
pub struct BslAnalyzer {
    pub semantic_analyzer: SemanticAnalyzer,
    pub error_collector: ErrorCollector,
}

impl BslAnalyzer {
    pub fn new() -> Self {
        Self {
            semantic_analyzer: SemanticAnalyzer::default(),
            error_collector: ErrorCollector::new(),
        }
    }
    
    /// Perform complete analysis on AST
    pub fn analyze(&mut self, ast: &AstNode) -> Result<()> {
        // Clear previous results
        self.error_collector.clear();
        
        // Run semantic analysis
        if let Err(e) = self.semantic_analyzer.analyze(ast) {
            eprintln!("Semantic analysis failed: {}", e);
        }
        
        // Collect semantic analysis results
        let (errors, warnings) = self.semantic_analyzer.get_results();
        for error in errors {
            self.error_collector.add_error(error);
        }
        for warning in warnings {
            self.error_collector.add_error(warning);
        }
        
        Ok(())
    }
    
    pub fn get_results(&self) -> &ErrorCollector {
        &self.error_collector
    }
}

impl Default for BslAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
