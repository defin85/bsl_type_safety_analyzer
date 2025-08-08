//! BSL Parser на базе tree-sitter
//!
//! Этот модуль предоставляет парсер для BSL (1C:Enterprise) кода,
//! оптимизированный для быстрой проверки синтаксиса и извлечения
//! информации о вызовах методов.

pub mod analyzer;
pub mod ast;
pub mod ast_bridge;
pub mod data_flow;
pub mod diagnostics;
pub mod keywords;
pub mod parser;
pub mod semantic;
pub mod tree_sitter_adapter;

pub use analyzer::{AnalysisConfig, AnalysisLevel, BslAnalyzer};
pub use ast::{BslAst, FunctionCall, MethodCall, PropertyAccess};
pub use ast_bridge::AstBridge;
pub use data_flow::{DataFlowAnalyzer, VariableState};
pub use diagnostics::{Diagnostic, DiagnosticSeverity, Location};
pub use parser::{BslParser, ParseResult};
pub use semantic::{Scope, ScopeType, SemanticAnalysisConfig, SemanticAnalyzer, VariableInfo};
pub use tree_sitter_adapter::TreeSitterAdapter;

/// Версия парсера
pub const PARSER_VERSION: &str = "0.1.0";

/// Поддерживаемые расширения файлов
pub const SUPPORTED_EXTENSIONS: &[&str] = &["bsl", "os"];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_version() {
        assert!(!PARSER_VERSION.is_empty());
    }
}
