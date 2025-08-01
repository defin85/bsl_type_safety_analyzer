/*!
# BSL Parser

High-performance parser for BSL (1C:Enterprise) language using nom parser combinators.

## Features

- **Fast lexical analysis** with logos lexer
- **Robust parsing** with nom combinators  
- **Complete AST** representation
- **Error recovery** for partial parsing
- **Position tracking** for diagnostics

## Usage

```rust
use bsl_analyzer::parser::{BslParser, AstNode};

let parser = BslParser::new();
let ast = parser.parse_text(r#"
    Процедура ТестоваяПроцедура() Экспорт
        Сообщить("Тест");
    КонецПроцедуры
"#)?;
```
*/

pub mod lexer;
pub mod grammar;
pub mod ast;
pub mod syntax_analyzer;
pub mod incremental;

#[cfg(test)]
mod syntax_analyzer_integration_test;

pub use ast::{AstNode, AstNodeType, Position, Span};
pub use lexer::{BslLexer, Token, TokenType, read_bsl_file};
pub use syntax_analyzer::SyntaxAnalyzer;
pub use incremental::{IncrementalParser, TextEdit};

use anyhow::{Context, Result};
use std::path::Path;

/// Main BSL parser - теперь использует tree-sitter внутри
pub struct BslParser {
    tree_sitter_parser: crate::bsl_parser::BslParser,
}

impl BslParser {
    /// Creates a new parser instance
    pub fn new() -> Self {
        Self {
            tree_sitter_parser: crate::bsl_parser::BslParser::new().expect("Failed to create tree-sitter parser"),
        }
    }
    
    /// Parses BSL code from string
    pub fn parse_text(&self, input: &str) -> Result<AstNode> {
        // Используем tree-sitter парсер
        let parse_result = self.tree_sitter_parser.parse(input, "<string>");
        
        if let Some(bsl_ast) = parse_result.ast {
            // Конвертируем BSL AST в старый формат
            let ast_node = crate::bsl_parser::AstBridge::convert_bsl_ast_to_ast_node(&bsl_ast);
            Ok(ast_node)
        } else {
            Err(anyhow::anyhow!("Failed to parse BSL code"))
        }
    }
    
    /// Parses BSL file with proper encoding detection and BOM handling
    pub fn parse_file<P: AsRef<Path>>(&self, file_path: P) -> Result<AstNode> {
        let content = read_bsl_file(file_path.as_ref())
            .with_context(|| format!("Failed to read file: {}", file_path.as_ref().display()))?;
            
        self.parse_text(&content)
    }
    
    /// Parses only procedure/function declarations (fast)
    pub fn parse_declarations(&self, input: &str) -> Result<Vec<AstNode>> {
        // Парсим весь модуль и извлекаем только объявления
        let ast = self.parse_text(input)?;
        let declarations = ast.find_children(AstNodeType::Procedure)
            .into_iter()
            .chain(ast.find_children(AstNodeType::Function))
            .cloned()
            .collect();
        Ok(declarations)
    }
}

impl Default for BslParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse result with optional error recovery
#[derive(Debug, Clone)]
pub struct ParseResult {
    pub ast: Option<AstNode>,
    pub errors: Vec<ParseError>,
    pub warnings: Vec<ParseWarning>,
}

impl ParseResult {
    pub fn new(ast: AstNode) -> Self {
        Self {
            ast: Some(ast),
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
    
    pub fn with_errors(mut self, errors: Vec<ParseError>) -> Self {
        self.errors = errors;
        self
    }
    
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
    
    pub fn is_success(&self) -> bool {
        self.ast.is_some() && self.errors.is_empty()
    }
}

/// Parse error with position information
#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub position: Position,
    pub expected: Option<String>,
    pub found: Option<String>,
}

/// Parse warning
#[derive(Debug, Clone)]  
pub struct ParseWarning {
    pub message: String,
    pub position: Position,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_simple_procedure() {
        let parser = BslParser::new();
        let code = r#"
            Процедура ТестоваяПроцедура() Экспорт
                Сообщить("Тест");
            КонецПроцедуры
        "#;
        
        let ast = parser.parse_text(code).unwrap();
        assert_eq!(ast.node_type, AstNodeType::Module);
        assert!(!ast.children.is_empty());
    }
    
    #[test] 
    fn test_parse_function_with_parameters() {
        let parser = BslParser::new();
        let code = r#"
            Функция ВычислитьСумму(Число1, Число2)
                Возврат Число1 + Число2;
            КонецФункции
        "#;
        
        let ast = parser.parse_text(code).unwrap();
        assert_eq!(ast.node_type, AstNodeType::Module);
        
        let function = &ast.children[0];
        assert_eq!(function.node_type, AstNodeType::Function);
    }
    
    #[test]
    fn test_parse_declarations_only() {
        let parser = BslParser::new();
        let code = r#"
            Процедура Процедура1() Экспорт
                // Implementation
            КонецПроцедуры
            
            Функция Функция1() Экспорт
                // Implementation  
            КонецФункции
        "#;
        
        let declarations = parser.parse_declarations(code).unwrap();
        assert_eq!(declarations.len(), 2);
        assert_eq!(declarations[0].node_type, AstNodeType::Procedure);
        assert_eq!(declarations[1].node_type, AstNodeType::Function);
    }
}
