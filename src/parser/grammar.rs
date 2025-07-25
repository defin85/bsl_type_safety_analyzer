// Заглушка для грамматики BSL
use crate::parser::{Token, ast::AstNode};
use anyhow::Result;

pub fn parse_module(_tokens: &[Token]) -> Result<AstNode> {
    // TODO: Implement BSL grammar parsing
    Ok(AstNode::module(crate::parser::ast::Span::zero()))
}

pub fn parse_declarations(_tokens: &[Token]) -> Result<Vec<AstNode>> {
    // TODO: Implement declaration parsing
    Ok(Vec::new())
}
