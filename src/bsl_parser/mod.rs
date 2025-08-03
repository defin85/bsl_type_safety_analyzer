//! BSL Parser на базе tree-sitter
//! 
//! Этот модуль предоставляет парсер для BSL (1C:Enterprise) кода,
//! оптимизированный для быстрой проверки синтаксиса и извлечения
//! информации о вызовах методов.

pub mod parser;
pub mod diagnostics;
pub mod ast;
pub mod tree_sitter_adapter;
pub mod ast_bridge;
pub mod semantic;
pub mod data_flow;
pub mod analyzer;
pub mod keywords;

pub use parser::{BslParser, ParseResult};
pub use diagnostics::{Diagnostic, DiagnosticSeverity, Location};
pub use ast::{BslAst, MethodCall, FunctionCall, PropertyAccess};
pub use tree_sitter_adapter::TreeSitterAdapter;
pub use ast_bridge::AstBridge;
pub use semantic::{SemanticAnalyzer, SemanticAnalysisConfig, VariableInfo, Scope, ScopeType};
pub use data_flow::{DataFlowAnalyzer, VariableState};
pub use analyzer::BslAnalyzer;

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