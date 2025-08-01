//! BSL Parser на базе tree-sitter

use anyhow::Result;
use tree_sitter::Parser;
use crate::bsl_parser::{ast::*, diagnostics::*, Location};
use super::tree_sitter_adapter::TreeSitterAdapter;

/// Результат парсинга
#[derive(Debug)]
pub struct ParseResult {
    pub ast: Option<BslAst>,
    pub diagnostics: Vec<Diagnostic>,
}

/// BSL Parser
pub struct BslParser {
    adapter: TreeSitterAdapter,
}

impl BslParser {
    /// Создает новый экземпляр парсера
    pub fn new() -> Result<Self> {
        Ok(Self { 
            adapter: TreeSitterAdapter::new(),
        })
    }

    /// Парсит BSL код
    pub fn parse(&self, source: &str, file_path: &str) -> ParseResult {
        let mut diagnostics = Vec::new();
        
        // Парсинг через tree-sitter
        let ast = self.parse_with_tree_sitter(source, file_path, &mut diagnostics);
        
        ParseResult { ast, diagnostics }
    }

    /// Парсит код с использованием tree-sitter
    fn parse_with_tree_sitter(
        &self,
        source: &str,
        file_path: &str,
        _diagnostics: &mut Vec<Diagnostic>,
    ) -> Option<BslAst> {
        // TODO: Включить когда будет решена проблема с API
        // let mut parser = Parser::new();
        // let language = unsafe { tree_sitter::Language::from_raw(tree_sitter_bsl::tree_sitter_bsl()) };
        // parser.set_language(language).ok()?;
        // let tree = parser.parse(source, None)?;
        // self.adapter.convert_tree_to_ast(tree, source, file_path, diagnostics)
        
        // Временная заглушка
        Some(BslAst {
            module: Module {
                directives: vec![],
                declarations: vec![],
                location: Location::new(file_path.to_string(), 1, 1, 0, source.len()),
            },
        })
    }





    /// Валидирует AST с использованием UnifiedBslIndex
    pub fn validate(
        &self,
        ast: &BslAst,
        index: &crate::unified_index::UnifiedBslIndex,
    ) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        
        // Проверяем все вызовы методов
        for method_call in ast.extract_method_calls() {
            if let Expression::Identifier(type_name) = &*method_call.object {
                if let Some(entity) = index.find_entity(type_name) {
                    // Проверяем существование метода
                    if !entity.interface.methods.contains_key(&method_call.method) {
                        diagnostics.push(
                            Diagnostic::new(
                                DiagnosticSeverity::Error,
                                method_call.location.clone(),
                                codes::UNKNOWN_METHOD,
                                format!(
                                    "Метод '{}' не найден для типа '{}'",
                                    method_call.method,
                                    type_name
                                ),
                            )
                            .with_found(&method_call.method)
                            .with_expected(
                                entity.interface.methods.keys()
                                    .take(3)
                                    .cloned()
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            )
                        );
                    }
                } else {
                    diagnostics.push(
                        Diagnostic::new(
                            DiagnosticSeverity::Warning,
                            method_call.location.clone(),
                            codes::UNKNOWN_CONSTRUCT,
                            format!("Неизвестный тип '{}'", type_name),
                        )
                    );
                }
            }
        }
        
        diagnostics
    }
}

impl Default for BslParser {
    fn default() -> Self {
        Self::new().expect("Failed to create BSL parser")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let parser = BslParser::new();
        assert!(parser.is_ok());
    }

    #[test]
    fn test_parse_empty() {
        let mut parser = BslParser::new().unwrap();
        let result = parser.parse("", "test.bsl");
        
        assert!(result.ast.is_some());
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_simple() {
        let mut parser = BslParser::new().unwrap();
        let code = r#"
            Процедура Тест()
                Массив = Новый Массив();
                Массив.Добавить(1);
            КонецПроцедуры
        "#;
        
        let result = parser.parse(code, "test.bsl");
        assert!(result.ast.is_some());
    }
}