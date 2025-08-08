//! Мост между новым tree-sitter AST и старой системой анализа

use crate::bsl_parser::ast::{
    BslAst, CompilerDirective, Declaration, Expression, Literal, Statement,
};
use crate::parser::ast::{AstNode, AstNodeType, Position, Span};

/// Преобразует новый BSL AST в старый формат для совместимости
pub struct AstBridge;

impl AstBridge {
    /// Конвертирует BslAst в старый AstNode
    pub fn convert_bsl_ast_to_ast_node(bsl_ast: &BslAst) -> AstNode {
        let mut module_node = AstNode::new(
            AstNodeType::Module,
            Self::location_to_span(&bsl_ast.module.location),
        );

        // Добавляем директивы компиляции как комментарии
        for directive in &bsl_ast.module.directives {
            let directive_node = Self::convert_compiler_directive(directive);
            module_node.add_child(directive_node);
        }

        // Добавляем объявления
        for declaration in &bsl_ast.module.declarations {
            let decl_node = Self::convert_declaration(declaration);
            module_node.add_child(decl_node);
        }

        module_node
    }

    /// Конвертирует директиву компиляции
    fn convert_compiler_directive(directive: &CompilerDirective) -> AstNode {
        let text = match directive {
            CompilerDirective::AtClient => "&НаКлиенте",
            CompilerDirective::AtServer => "&НаСервере",
            CompilerDirective::AtServerNoContext => "&НаСервереБезКонтекста",
            CompilerDirective::AtClientAtServerNoContext => "&НаКлиентеНаСервереБезКонтекста",
            CompilerDirective::AtClientAtServer => "&НаКлиентеНаСервере",
        };

        AstNode::with_value(AstNodeType::Comment, Span::zero(), text.to_string())
    }

    /// Конвертирует объявление
    fn convert_declaration(declaration: &Declaration) -> AstNode {
        match declaration {
            Declaration::Procedure(proc) => {
                let mut proc_node = AstNode::with_value(
                    AstNodeType::Procedure,
                    Self::location_to_span(&proc.location),
                    proc.name.clone(),
                );

                if proc.export {
                    proc_node.add_attribute("export".to_string(), "true".to_string());
                }

                // Добавляем параметры
                if !proc.params.is_empty() {
                    let mut param_list = AstNode::new(AstNodeType::ParameterList, Span::zero());

                    for param in &proc.params {
                        let mut param_node = AstNode::with_value(
                            AstNodeType::Parameter,
                            Self::location_to_span(&param.location),
                            param.name.clone(),
                        );

                        if param.by_val {
                            param_node.add_attribute("by_val".to_string(), "true".to_string());
                        }

                        param_list.add_child(param_node);
                    }

                    proc_node.add_child(param_list);
                }

                // Добавляем тело процедуры
                let mut body_block = AstNode::new(AstNodeType::Block, Span::zero());
                for statement in &proc.body {
                    let stmt_node = Self::convert_statement(statement);
                    body_block.add_child(stmt_node);
                }
                proc_node.add_child(body_block);

                proc_node
            }
            Declaration::Function(func) => {
                let mut func_node = AstNode::with_value(
                    AstNodeType::Function,
                    Self::location_to_span(&func.location),
                    func.name.clone(),
                );

                if func.export {
                    func_node.add_attribute("export".to_string(), "true".to_string());
                }

                // Добавляем параметры
                if !func.params.is_empty() {
                    let mut param_list = AstNode::new(AstNodeType::ParameterList, Span::zero());

                    for param in &func.params {
                        let mut param_node = AstNode::with_value(
                            AstNodeType::Parameter,
                            Self::location_to_span(&param.location),
                            param.name.clone(),
                        );

                        if param.by_val {
                            param_node.add_attribute("by_val".to_string(), "true".to_string());
                        }

                        param_list.add_child(param_node);
                    }

                    func_node.add_child(param_list);
                }

                // Добавляем тело функции
                let mut body_block = AstNode::new(AstNodeType::Block, Span::zero());
                for statement in &func.body {
                    let stmt_node = Self::convert_statement(statement);
                    body_block.add_child(stmt_node);
                }
                func_node.add_child(body_block);

                func_node
            }
            Declaration::Variable(var) => {
                let mut var_node = AstNode::new(
                    AstNodeType::VariableDeclaration,
                    Self::location_to_span(&var.location),
                );

                if var.export {
                    var_node.add_attribute("export".to_string(), "true".to_string());
                }

                // Добавляем имена переменных
                for name in &var.names {
                    let name_node =
                        AstNode::with_value(AstNodeType::Variable, Span::zero(), name.clone());
                    var_node.add_child(name_node);
                }

                var_node
            }
        }
    }

    /// Конвертирует оператор
    fn convert_statement(statement: &Statement) -> AstNode {
        match statement {
            Statement::Expression(expr) => Self::convert_expression(expr),
            Statement::Assignment(assign) => {
                let mut assign_node = AstNode::new(
                    AstNodeType::Assignment,
                    Self::location_to_span(&assign.location),
                );

                assign_node.add_child(Self::convert_expression(&assign.target));
                assign_node.add_child(Self::convert_expression(&assign.value));

                assign_node
            }
            Statement::If(if_stmt) => {
                let mut if_node = AstNode::new(
                    AstNodeType::IfStatement,
                    Self::location_to_span(&if_stmt.location),
                );

                if_node.add_child(Self::convert_expression(&if_stmt.condition));

                // Добавляем блок then
                let mut then_block = AstNode::new(AstNodeType::Block, Span::zero());
                for stmt in &if_stmt.then_branch {
                    then_block.add_child(Self::convert_statement(stmt));
                }
                if_node.add_child(then_block);

                // Добавляем блок else если есть
                if let Some(else_branch) = &if_stmt.else_branch {
                    let mut else_block = AstNode::new(AstNodeType::Block, Span::zero());
                    for stmt in else_branch {
                        else_block.add_child(Self::convert_statement(stmt));
                    }
                    if_node.add_child(else_block);
                }

                if_node
            }
            Statement::Return(ret) => {
                let mut return_node = AstNode::new(
                    AstNodeType::ReturnStatement,
                    Self::location_to_span(&ret.location),
                );

                if let Some(value) = &ret.value {
                    return_node.add_child(Self::convert_expression(value));
                }

                return_node
            }
            Statement::Break => AstNode::new(AstNodeType::BreakStatement, Span::zero()),
            Statement::Continue => AstNode::new(AstNodeType::ContinueStatement, Span::zero()),
            _ => {
                // Для других типов операторов создаем общий узел
                AstNode::new(AstNodeType::Expression, Span::zero())
            }
        }
    }

    /// Конвертирует выражение
    fn convert_expression(expression: &Expression) -> AstNode {
        match expression {
            Expression::Literal(literal) => Self::convert_literal(literal),
            Expression::Identifier(name) => {
                AstNode::with_value(AstNodeType::Identifier, Span::zero(), name.clone())
            }
            Expression::MethodCall(call) => {
                let mut call_node = AstNode::new(
                    AstNodeType::CallExpression,
                    Self::location_to_span(&call.location),
                );

                call_node.add_child(Self::convert_expression(&call.object));

                let method_node =
                    AstNode::with_value(AstNodeType::Identifier, Span::zero(), call.method.clone());
                call_node.add_child(method_node);

                // Добавляем аргументы
                for arg in &call.args {
                    call_node.add_child(Self::convert_expression(arg));
                }

                call_node
            }
            Expression::PropertyAccess(access) => {
                let mut access_node = AstNode::new(
                    AstNodeType::MemberExpression,
                    Self::location_to_span(&access.location),
                );

                access_node.add_child(Self::convert_expression(&access.object));

                let prop_node = AstNode::with_value(
                    AstNodeType::Identifier,
                    Span::zero(),
                    access.property.clone(),
                );
                access_node.add_child(prop_node);

                access_node
            }
            Expression::New(new_expr) => {
                let mut new_node = AstNode::new(
                    AstNodeType::NewExpression,
                    Self::location_to_span(&new_expr.location),
                );

                let type_node = AstNode::with_value(
                    AstNodeType::Identifier,
                    Span::zero(),
                    new_expr.type_name.clone(),
                );
                new_node.add_child(type_node);

                // Добавляем аргументы
                for arg in &new_expr.args {
                    new_node.add_child(Self::convert_expression(arg));
                }

                new_node
            }
            _ => {
                // Для других типов выражений создаем общий узел
                AstNode::new(AstNodeType::Expression, Span::zero())
            }
        }
    }

    /// Конвертирует литерал
    fn convert_literal(literal: &Literal) -> AstNode {
        match literal {
            Literal::Number(n) => {
                AstNode::with_value(AstNodeType::NumberLiteral, Span::zero(), n.to_string())
            }
            Literal::String(s) => {
                AstNode::with_value(AstNodeType::StringLiteral, Span::zero(), s.clone())
            }
            Literal::Boolean(b) => {
                AstNode::with_value(AstNodeType::BooleanLiteral, Span::zero(), b.to_string())
            }
            Literal::Date(d) => {
                AstNode::with_value(AstNodeType::DateLiteral, Span::zero(), d.clone())
            }
            Literal::Undefined => AstNode::new(AstNodeType::UndefinedLiteral, Span::zero()),
            Literal::Null => AstNode::new(AstNodeType::NullLiteral, Span::zero()),
        }
    }

    /// Конвертирует Location в Span
    fn location_to_span(location: &crate::bsl_parser::Location) -> Span {
        let start = Position::new(location.line, location.column, location.offset);
        let end = Position::new(
            location.line,
            location.column + location.length,
            location.offset + location.length,
        );
        Span::new(start, end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bsl_parser::ast::*;

    #[test]
    fn test_convert_simple_module() {
        let bsl_ast = BslAst {
            module: Module {
                directives: vec![CompilerDirective::AtServer],
                declarations: vec![Declaration::Procedure(ProcedureDecl {
                    name: "TestProc".to_string(),
                    export: true,
                    params: vec![],
                    directives: vec![],
                    body: vec![],
                    location: crate::bsl_parser::Location::new("test.bsl".to_string(), 1, 1, 0, 10),
                })],
                location: crate::bsl_parser::Location::new("test.bsl".to_string(), 1, 1, 0, 100),
            },
        };

        let ast_node = AstBridge::convert_bsl_ast_to_ast_node(&bsl_ast);

        assert_eq!(ast_node.node_type, AstNodeType::Module);
        assert_eq!(ast_node.children.len(), 2); // директива + процедура

        let proc_node = &ast_node.children[1];
        assert_eq!(proc_node.node_type, AstNodeType::Procedure);
        assert_eq!(proc_node.text(), "TestProc");
        assert!(proc_node.is_export());
    }
}
