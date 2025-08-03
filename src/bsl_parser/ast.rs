//! AST структуры для BSL

use serde::{Deserialize, Serialize};
use super::Location;

/// Корневой узел AST
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BslAst {
    pub module: Module,
}

/// Модуль BSL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub directives: Vec<CompilerDirective>,
    pub declarations: Vec<Declaration>,
    pub location: Location,
}

/// Директивы компиляции
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompilerDirective {
    AtClient,
    AtServer,
    AtServerNoContext,
    AtClientAtServerNoContext,
    AtClientAtServer,
}

/// Объявления верхнего уровня
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Declaration {
    Procedure(ProcedureDecl),
    Function(FunctionDecl),
    Variable(VariableDecl),
}

/// Объявление процедуры
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureDecl {
    pub name: String,
    pub export: bool,
    pub params: Vec<Parameter>,
    pub directives: Vec<CompilerDirective>,
    pub body: Vec<Statement>,
    pub location: Location,
}

/// Объявление функции
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDecl {
    pub name: String,
    pub export: bool,
    pub params: Vec<Parameter>,
    pub directives: Vec<CompilerDirective>,
    pub body: Vec<Statement>,
    pub location: Location,
}

/// Параметр процедуры/функции
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub by_val: bool,
    pub default_value: Option<Expression>,
    pub location: Location,
}

/// Объявление переменной
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableDecl {
    pub names: Vec<String>,
    pub export: bool,
    pub location: Location,
}

/// Операторы
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Statement {
    Expression(Expression),
    Assignment(Assignment),
    If(IfStatement),
    While(WhileStatement),
    For(ForStatement),
    ForEach(ForEachStatement),
    Return(ReturnStatement),
    Break,
    Continue,
    Try(TryStatement),
}

/// Присваивание
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assignment {
    pub target: Expression,
    pub value: Expression,
    pub location: Location,
}

/// Условный оператор
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfStatement {
    pub condition: Expression,
    pub then_branch: Vec<Statement>,
    pub else_ifs: Vec<ElseIf>,
    pub else_branch: Option<Vec<Statement>>,
    pub location: Location,
}

/// ИначеЕсли
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElseIf {
    pub condition: Expression,
    pub body: Vec<Statement>,
    pub location: Location,
}

/// Цикл Пока
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhileStatement {
    pub condition: Expression,
    pub body: Vec<Statement>,
    pub location: Location,
}

/// Цикл Для
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForStatement {
    pub variable: String,
    pub from: Expression,
    pub to: Expression,
    pub body: Vec<Statement>,
    pub location: Location,
}

/// Цикл Для Каждого
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForEachStatement {
    pub variable: String,
    pub collection: Expression,
    pub body: Vec<Statement>,
    pub location: Location,
}

/// Возврат
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnStatement {
    pub value: Option<Expression>,
    pub location: Location,
}

/// Обработка исключений
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TryStatement {
    pub body: Vec<Statement>,
    pub except: Vec<Statement>,
    pub location: Location,
}

/// Выражения
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expression {
    Literal(Literal),
    Identifier(String),
    MethodCall(MethodCall),
    FunctionCall(FunctionCall),
    PropertyAccess(PropertyAccess),
    New(NewExpression),
    Binary(BinaryOp),
    Unary(UnaryOp),
    Ternary(TernaryOp),
    Index(IndexAccess),
}

/// Литералы
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Literal {
    Number(f64),
    String(String),
    Boolean(bool),
    Date(String),
    Undefined,
    Null,
}

/// Вызов метода
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodCall {
    pub object: Box<Expression>,
    pub method: String,
    pub args: Vec<Expression>,
    pub location: Location,
}

/// Вызов глобальной функции
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub args: Vec<Expression>,
    pub location: Location,
}

/// Обращение к свойству
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyAccess {
    pub object: Box<Expression>,
    pub property: String,
    pub location: Location,
}

/// Создание объекта
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewExpression {
    pub type_name: String,
    pub args: Vec<Expression>,
    pub location: Location,
}

/// Бинарная операция
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryOp {
    pub left: Box<Expression>,
    pub op: BinaryOperator,
    pub right: Box<Expression>,
    pub location: Location,
}

/// Унарная операция
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnaryOp {
    pub op: UnaryOperator,
    pub operand: Box<Expression>,
    pub location: Location,
}

/// Тернарная операция
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TernaryOp {
    pub condition: Box<Expression>,
    pub then_expr: Box<Expression>,
    pub else_expr: Box<Expression>,
    pub location: Location,
}

/// Индексный доступ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexAccess {
    pub object: Box<Expression>,
    pub index: Box<Expression>,
    pub location: Location,
}

/// Бинарные операторы
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    Less,
    Greater,
    LessOrEqual,
    GreaterOrEqual,
    And,
    Or,
}

/// Унарные операторы
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOperator {
    Not,
    Minus,
}

impl BslAst {
    /// Извлекает все вызовы методов из AST
    pub fn extract_method_calls(&self) -> Vec<&MethodCall> {
        let mut calls = Vec::new();
        self.visit_method_calls(&mut calls);
        calls
    }

    fn visit_method_calls<'a>(&'a self, calls: &mut Vec<&'a MethodCall>) {
        for decl in &self.module.declarations {
            match decl {
                Declaration::Procedure(p) => {
                    for stmt in &p.body {
                        Self::visit_statement_method_calls(stmt, calls);
                    }
                }
                Declaration::Function(f) => {
                    for stmt in &f.body {
                        Self::visit_statement_method_calls(stmt, calls);
                    }
                }
                _ => {}
            }
        }
    }

    fn visit_statement_method_calls<'a>(stmt: &'a Statement, calls: &mut Vec<&'a MethodCall>) {
        match stmt {
            Statement::Expression(expr) => Self::visit_expression_method_calls(expr, calls),
            Statement::Assignment(a) => {
                Self::visit_expression_method_calls(&a.target, calls);
                Self::visit_expression_method_calls(&a.value, calls);
            }
            Statement::If(i) => {
                Self::visit_expression_method_calls(&i.condition, calls);
                for s in &i.then_branch {
                    Self::visit_statement_method_calls(s, calls);
                }
                for elif in &i.else_ifs {
                    Self::visit_expression_method_calls(&elif.condition, calls);
                    for s in &elif.body {
                        Self::visit_statement_method_calls(s, calls);
                    }
                }
                if let Some(else_branch) = &i.else_branch {
                    for s in else_branch {
                        Self::visit_statement_method_calls(s, calls);
                    }
                }
            }
            Statement::Return(r) => {
                if let Some(expr) = &r.value {
                    Self::visit_expression_method_calls(expr, calls);
                }
            }
            _ => {}
        }
    }

    fn visit_expression_method_calls<'a>(expr: &'a Expression, calls: &mut Vec<&'a MethodCall>) {
        match expr {
            Expression::MethodCall(call) => {
                calls.push(call);
                Self::visit_expression_method_calls(&call.object, calls);
                for arg in &call.args {
                    Self::visit_expression_method_calls(arg, calls);
                }
            }
            Expression::PropertyAccess(access) => {
                Self::visit_expression_method_calls(&access.object, calls);
            }
            Expression::New(new_expr) => {
                for arg in &new_expr.args {
                    Self::visit_expression_method_calls(arg, calls);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_method_call_extraction() {
        let ast = BslAst {
            module: Module {
                directives: vec![],
                declarations: vec![
                    Declaration::Function(FunctionDecl {
                        name: "Test".to_string(),
                        export: false,
                        params: vec![],
                        directives: vec![],
                        body: vec![
                            Statement::Expression(Expression::MethodCall(MethodCall {
                                object: Box::new(Expression::Identifier("Array".to_string())),
                                method: "Add".to_string(),
                                args: vec![Expression::Literal(Literal::Number(1.0))],
                                location: Location::new("test.bsl".to_string(), 1, 1, 0, 10),
                            })),
                        ],
                        location: Location::new("test.bsl".to_string(), 1, 1, 0, 100),
                    }),
                ],
                location: Location::new("test.bsl".to_string(), 1, 1, 0, 100),
            },
        };

        let calls = ast.extract_method_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].method, "Add");
    }
}