/*!
# Abstract Syntax Tree (AST) for BSL

Defines the AST node types and structures for representing parsed BSL code.
*/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Position in source code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

impl Position {
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Self {
            line,
            column,
            offset,
        }
    }

    pub fn zero() -> Self {
        Self::new(0, 0, 0)
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// Span in source code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Span {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    pub fn zero() -> Self {
        Self::new(Position::zero(), Position::zero())
    }
}

/// AST Node types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AstNodeType {
    // Module level
    Module,

    // Declarations
    Procedure,
    Function,
    Variable,
    VariableDeclaration,

    // Parameters
    Parameter,
    ParameterList,

    // Statements
    Assignment,
    IfStatement,
    ForEachStatement,
    ForLoop,
    WhileStatement,
    WhileLoop,
    TryStatement,
    ReturnStatement,
    BreakStatement,
    ContinueStatement,

    // Expressions
    Expression,
    BinaryExpression,
    UnaryExpression,
    CallExpression,
    MemberExpression,
    ArrayAccess,
    NewExpression,

    // Literals
    StringLiteral,
    NumberLiteral,
    BooleanLiteral,
    DateLiteral,
    UndefinedLiteral,
    NullLiteral,

    // Identifiers
    Identifier,
    Keyword,

    // Comments
    Comment,

    // Blocks
    Block,

    // Unknown
    Unknown,
}

impl fmt::Display for AstNodeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AstNodeType::Module => write!(f, "Module"),
            AstNodeType::Procedure => write!(f, "Procedure"),
            AstNodeType::Function => write!(f, "Function"),
            AstNodeType::Variable => write!(f, "Variable"),
            AstNodeType::VariableDeclaration => write!(f, "VariableDeclaration"),
            AstNodeType::Parameter => write!(f, "Parameter"),
            AstNodeType::ParameterList => write!(f, "ParameterList"),
            AstNodeType::Assignment => write!(f, "Assignment"),
            AstNodeType::IfStatement => write!(f, "IfStatement"),
            AstNodeType::ForEachStatement => write!(f, "ForEachStatement"),
            AstNodeType::ForLoop => write!(f, "ForLoop"),
            AstNodeType::WhileStatement => write!(f, "WhileStatement"),
            AstNodeType::WhileLoop => write!(f, "WhileLoop"),
            AstNodeType::TryStatement => write!(f, "TryStatement"),
            AstNodeType::ReturnStatement => write!(f, "ReturnStatement"),
            AstNodeType::BreakStatement => write!(f, "BreakStatement"),
            AstNodeType::ContinueStatement => write!(f, "ContinueStatement"),
            AstNodeType::Expression => write!(f, "Expression"),
            AstNodeType::BinaryExpression => write!(f, "BinaryExpression"),
            AstNodeType::UnaryExpression => write!(f, "UnaryExpression"),
            AstNodeType::CallExpression => write!(f, "CallExpression"),
            AstNodeType::MemberExpression => write!(f, "MemberExpression"),
            AstNodeType::ArrayAccess => write!(f, "ArrayAccess"),
            AstNodeType::NewExpression => write!(f, "NewExpression"),
            AstNodeType::StringLiteral => write!(f, "StringLiteral"),
            AstNodeType::NumberLiteral => write!(f, "NumberLiteral"),
            AstNodeType::BooleanLiteral => write!(f, "BooleanLiteral"),
            AstNodeType::DateLiteral => write!(f, "DateLiteral"),
            AstNodeType::UndefinedLiteral => write!(f, "UndefinedLiteral"),
            AstNodeType::NullLiteral => write!(f, "NullLiteral"),
            AstNodeType::Identifier => write!(f, "Identifier"),
            AstNodeType::Keyword => write!(f, "Keyword"),
            AstNodeType::Comment => write!(f, "Comment"),
            AstNodeType::Block => write!(f, "Block"),
            AstNodeType::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Main AST Node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstNode {
    pub node_type: AstNodeType,
    pub span: Span,
    pub value: Option<String>,
    pub attributes: HashMap<String, String>,
    pub children: Vec<AstNode>,
}

impl AstNode {
    /// Creates a new AST node
    pub fn new(node_type: AstNodeType, span: Span) -> Self {
        Self {
            node_type,
            span,
            value: None,
            attributes: HashMap::new(),
            children: Vec::new(),
        }
    }

    /// Creates node with value
    pub fn with_value(node_type: AstNodeType, span: Span, value: String) -> Self {
        Self {
            node_type,
            span,
            value: Some(value),
            attributes: HashMap::new(),
            children: Vec::new(),
        }
    }

    /// Adds a child node
    pub fn add_child(&mut self, child: AstNode) {
        self.children.push(child);
    }

    /// Adds an attribute
    pub fn add_attribute(&mut self, key: String, value: String) {
        self.attributes.insert(key, value);
    }

    /// Sets the value
    pub fn set_value(&mut self, value: String) {
        self.value = Some(value);
    }

    /// Gets attribute value
    pub fn get_attribute(&self, key: &str) -> Option<&String> {
        self.attributes.get(key)
    }

    /// Checks if node has attribute
    pub fn has_attribute(&self, key: &str) -> bool {
        self.attributes.contains_key(key)
    }

    /// Finds first child of given type
    pub fn find_child(&self, node_type: AstNodeType) -> Option<&AstNode> {
        self.children
            .iter()
            .find(|child| child.node_type == node_type)
    }

    /// Finds all children of given type
    pub fn find_children(&self, node_type: AstNodeType) -> Vec<&AstNode> {
        self.children
            .iter()
            .filter(|child| child.node_type == node_type)
            .collect()
    }

    /// Recursively finds all nodes of given type
    pub fn find_all(&self, node_type: AstNodeType) -> Vec<&AstNode> {
        let mut result = Vec::new();
        self.find_all_recursive(&node_type, &mut result);
        result
    }

    fn find_all_recursive<'a>(&'a self, node_type: &AstNodeType, result: &mut Vec<&'a AstNode>) {
        if &self.node_type == node_type {
            result.push(self);
        }

        for child in &self.children {
            child.find_all_recursive(node_type, result);
        }
    }

    /// Gets the text value of the node
    pub fn text(&self) -> &str {
        self.value.as_deref().unwrap_or("")
    }

    /// Creates a module node
    pub fn module(span: Span) -> Self {
        Self::new(AstNodeType::Module, span)
    }

    /// Creates a procedure node
    pub fn procedure(span: Span, name: String) -> Self {
        Self::with_value(AstNodeType::Procedure, span, name)
    }

    /// Creates a function node
    pub fn function(span: Span, name: String) -> Self {
        Self::with_value(AstNodeType::Function, span, name)
    }

    /// Creates a parameter node
    pub fn parameter(span: Span, name: String) -> Self {
        Self::with_value(AstNodeType::Parameter, span, name)
    }

    /// Creates an identifier node
    pub fn identifier(span: Span, name: String) -> Self {
        Self::with_value(AstNodeType::Identifier, span, name)
    }

    /// Creates a number literal node
    pub fn number_literal(span: Span, value: String) -> Self {
        Self::with_value(AstNodeType::NumberLiteral, span, value)
    }

    /// Creates a string literal node
    pub fn string_literal(span: Span, value: String) -> Self {
        Self::with_value(AstNodeType::StringLiteral, span, value)
    }

    /// Checks if this is an export declaration
    pub fn is_export(&self) -> bool {
        self.has_attribute("export") || self.get_attribute("export").is_some_and(|v| v == "true")
    }

    /// Gets procedure/function name
    pub fn name(&self) -> Option<&str> {
        match self.node_type {
            AstNodeType::Procedure
            | AstNodeType::Function
            | AstNodeType::Variable
            | AstNodeType::Identifier => self.value.as_deref(),
            _ => None,
        }
    }

    /// Gets the position of the node
    pub fn position(&self) -> Position {
        self.span.start
    }

    /// Gets parameters for procedure/function
    pub fn parameters(&self) -> Vec<&AstNode> {
        if let Some(param_list) = self.find_child(AstNodeType::ParameterList) {
            param_list.find_children(AstNodeType::Parameter)
        } else {
            Vec::new()
        }
    }
}

impl PartialEq for AstNode {
    fn eq(&self, other: &Self) -> bool {
        self.node_type == other.node_type
            && self.value == other.value
            && self.attributes == other.attributes
            && self.children == other.children
    }
}

/// AST visitor trait for traversing the tree
pub trait AstVisitor {
    fn visit_node(&mut self, node: &AstNode);

    fn walk(&mut self, node: &AstNode) {
        self.visit_node(node);
        for child in &node.children {
            self.walk(child);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_node_creation() {
        let span = Span::zero();
        let mut node = AstNode::procedure(span, "TestProcedure".to_string());

        assert_eq!(node.node_type, AstNodeType::Procedure);
        assert_eq!(node.text(), "TestProcedure");

        let param = AstNode::parameter(span, "Param1".to_string());
        node.add_child(param);

        assert_eq!(node.children.len(), 1);
        assert_eq!(node.children[0].text(), "Param1");
    }

    #[test]
    fn test_find_methods() {
        let span = Span::zero();
        let mut module = AstNode::module(span);

        let proc1 = AstNode::procedure(span, "Proc1".to_string());
        let func1 = AstNode::function(span, "Func1".to_string());

        module.add_child(proc1);
        module.add_child(func1);

        let procedures = module.find_children(AstNodeType::Procedure);
        assert_eq!(procedures.len(), 1);
        assert_eq!(procedures[0].text(), "Proc1");

        let functions = module.find_children(AstNodeType::Function);
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].text(), "Func1");
    }
}
