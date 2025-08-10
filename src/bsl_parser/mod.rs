//! BSL Parser на базе tree-sitter
//!
//! Этот модуль предоставляет парсер для BSL (1C:Enterprise) кода,
//! оптимизированный для быстрой проверки синтаксиса и извлечения
//! информации о вызовах методов.

pub mod analyzer;
pub mod ast; // TODO: удалить после миграции semantic на Arena
pub mod cst_to_arena; // временный конвертер BslAst -> Arena
pub mod semantic_arena; // экспериментальная семантика поверх Arena
pub mod simple_types; // простая система типов для arena семантики
pub mod data_flow;
pub mod diagnostics;
pub mod keywords;
pub mod parser;
pub mod semantic;
pub mod tree_sitter_adapter;

pub use analyzer::{AnalysisConfig, AnalysisLevel, BslAnalyzer};
pub use ast::{BslAst, FunctionCall, MethodCall, PropertyAccess}; // временно
pub use cst_to_arena::ArenaConverter;
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
