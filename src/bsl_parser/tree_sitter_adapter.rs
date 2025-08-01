//! Адаптер для конвертации tree-sitter дерева в наш AST

use tree_sitter::{Node, Tree};
use crate::bsl_parser::{ast::*, diagnostics::*, Location};

/// Конвертер tree-sitter дерева в BSL AST
#[derive(Clone)]
pub struct TreeSitterAdapter;

impl TreeSitterAdapter {
    pub fn new() -> Self {
        Self
    }

    /// Конвертирует tree-sitter дерево в наш AST
    pub fn convert_tree_to_ast(
        &self,
        tree: Tree,
        source: &str,
        file_path: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Option<BslAst> {
        let root_node = tree.root_node();
        
        // Проверяем на синтаксические ошибки
        if root_node.has_error() {
            self.collect_syntax_errors(&root_node, source, file_path, diagnostics);
        }
        
        // Конвертируем в AST
        self.parse_module(root_node, source, file_path, diagnostics)
    }

    /// Собирает синтаксические ошибки из дерева
    fn collect_syntax_errors(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let mut cursor = node.walk();
        
        loop {
            let node = cursor.node();
            
            if node.is_error() {
                let location = self.node_to_location(&node, file_path);
                let text = node.utf8_text(source.as_bytes()).unwrap_or("<invalid>");
                
                diagnostics.push(
                    Diagnostic::new(
                        DiagnosticSeverity::Error,
                        location,
                        codes::SYNTAX_ERROR,
                        format!("Синтаксическая ошибка: неожиданный токен '{}'", text),
                    )
                );
            }
            
            if cursor.goto_first_child() {
                continue;
            }
            
            loop {
                if cursor.goto_next_sibling() {
                    break;
                }
                
                if !cursor.goto_parent() {
                    return;
                }
            }
        }
    }

    /// Парсит модуль
    fn parse_module(
        &self,
        node: Node,
        source: &str,
        file_path: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Option<BslAst> {
        let mut module = Module {
            directives: vec![],
            declarations: vec![],
            location: self.node_to_location(&node, file_path),
        };

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "preprocessor_directive" => {
                    if let Some(directive) = self.parse_compiler_directive(child, source) {
                        module.directives.push(directive);
                    }
                }
                "procedure_declaration" => {
                    if let Some(proc) = self.parse_procedure_declaration(child, source, file_path) {
                        module.declarations.push(Declaration::Procedure(proc));
                    }
                }
                "function_declaration" => {
                    if let Some(func) = self.parse_function_declaration(child, source, file_path) {
                        module.declarations.push(Declaration::Function(func));
                    }
                }
                "variable_declaration" => {
                    if let Some(var) = self.parse_variable_declaration(child, source, file_path) {
                        module.declarations.push(Declaration::Variable(var));
                    }
                }
                _ => {}
            }
        }

        Some(BslAst { module })
    }

    /// Парсит директиву компиляции
    fn parse_compiler_directive(&self, node: Node, source: &str) -> Option<CompilerDirective> {
        let text = node.utf8_text(source.as_bytes()).ok()?;
        match text {
            "&НаКлиенте" | "&AtClient" => Some(CompilerDirective::AtClient),
            "&НаСервере" | "&AtServer" => Some(CompilerDirective::AtServer),
            "&НаСервереБезКонтекста" | "&AtServerNoContext" => Some(CompilerDirective::AtServerNoContext),
            "&НаКлиентеНаСервереБезКонтекста" | "&AtClientAtServerNoContext" => Some(CompilerDirective::AtClientAtServerNoContext),
            "&НаКлиентеНаСервере" | "&AtClientAtServer" => Some(CompilerDirective::AtClientAtServer),
            _ => None,
        }
    }

    /// Парсит объявление процедуры
    fn parse_procedure_declaration(&self, node: Node, source: &str, file_path: &str) -> Option<ProcedureDecl> {
        let mut name = String::new();
        let mut export = false;
        let mut params = vec![];
        let mut directives = vec![];
        let mut body = vec![];

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "identifier" => {
                    if name.is_empty() {
                        name = child.utf8_text(source.as_bytes()).unwrap_or_default().to_string();
                    }
                }
                "export_keyword" => export = true,
                "parameter_list" => {
                    params = self.parse_parameter_list(child, source, file_path);
                }
                "statement_list" => {
                    body = self.parse_statement_list(child, source, file_path);
                }
                _ => {}
            }
        }

        Some(ProcedureDecl {
            name,
            export,
            params,
            directives,
            body,
            location: self.node_to_location(&node, file_path),
        })
    }

    /// Парсит объявление функции
    fn parse_function_declaration(&self, node: Node, source: &str, file_path: &str) -> Option<FunctionDecl> {
        let mut name = String::new();
        let mut export = false;
        let mut params = vec![];
        let mut directives = vec![];
        let mut body = vec![];

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "identifier" => {
                    if name.is_empty() {
                        name = child.utf8_text(source.as_bytes()).unwrap_or_default().to_string();
                    }
                }
                "export_keyword" => export = true,
                "parameter_list" => {
                    params = self.parse_parameter_list(child, source, file_path);
                }
                "statement_list" => {
                    body = self.parse_statement_list(child, source, file_path);
                }
                _ => {}
            }
        }

        Some(FunctionDecl {
            name,
            export,
            params,
            directives,
            body,
            location: self.node_to_location(&node, file_path),
        })
    }

    /// Парсит объявление переменной
    fn parse_variable_declaration(&self, node: Node, source: &str, file_path: &str) -> Option<VariableDecl> {
        let mut names = vec![];
        let mut export = false;

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "identifier" => {
                    names.push(child.utf8_text(source.as_bytes()).unwrap_or_default().to_string());
                }
                "export_keyword" => export = true,
                _ => {}
            }
        }

        Some(VariableDecl {
            names,
            export,
            location: self.node_to_location(&node, file_path),
        })
    }

    /// Парсит список параметров
    fn parse_parameter_list(&self, node: Node, source: &str, file_path: &str) -> Vec<Parameter> {
        let mut params = vec![];
        
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "parameter" {
                if let Some(param) = self.parse_parameter(child, source, file_path) {
                    params.push(param);
                }
            }
        }
        
        params
    }

    /// Парсит параметр
    fn parse_parameter(&self, node: Node, source: &str, file_path: &str) -> Option<Parameter> {
        let mut name = String::new();
        let mut by_val = false;
        let mut default_value = None;

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "val_keyword" => by_val = true,
                "identifier" => {
                    name = child.utf8_text(source.as_bytes()).unwrap_or_default().to_string();
                }
                "default_value" => {
                    default_value = self.parse_expression(child, source, file_path);
                }
                _ => {}
            }
        }

        Some(Parameter {
            name,
            by_val,
            default_value,
            location: self.node_to_location(&node, file_path),
        })
    }

    /// Парсит список операторов
    fn parse_statement_list(&self, node: Node, source: &str, file_path: &str) -> Vec<Statement> {
        let mut statements = vec![];
        
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some(stmt) = self.parse_statement(child, source, file_path) {
                statements.push(stmt);
            }
        }
        
        statements
    }

    /// Парсит оператор
    fn parse_statement(&self, node: Node, source: &str, file_path: &str) -> Option<Statement> {
        match node.kind() {
            "assignment_statement" => {
                self.parse_assignment_statement(node, source, file_path).map(Statement::Assignment)
            }
            "if_statement" => {
                self.parse_if_statement(node, source, file_path).map(Statement::If)
            }
            "return_statement" => {
                self.parse_return_statement(node, source, file_path).map(Statement::Return)
            }
            "expression_statement" => {
                self.parse_expression(node, source, file_path).map(Statement::Expression)
            }
            _ => None,
        }
    }

    /// Парсит присваивание
    fn parse_assignment_statement(&self, node: Node, source: &str, file_path: &str) -> Option<Assignment> {
        let mut target = None;
        let mut value = None;

        let mut cursor = node.walk();
        for (i, child) in node.children(&mut cursor).enumerate() {
            if i == 0 {
                target = self.parse_expression(child, source, file_path);
            } else if child.kind() != "=" {
                value = self.parse_expression(child, source, file_path);
                break;
            }
        }

        Some(Assignment {
            target: target?,
            value: value?,
            location: self.node_to_location(&node, file_path),
        })
    }

    /// Парсит условный оператор
    fn parse_if_statement(&self, node: Node, source: &str, file_path: &str) -> Option<IfStatement> {
        let mut condition = None;
        let mut then_branch = vec![];
        let mut else_ifs = vec![];
        let mut else_branch = None;

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "condition" => {
                    condition = self.parse_expression(child, source, file_path);
                }
                "then_clause" => {
                    then_branch = self.parse_statement_list(child, source, file_path);
                }
                "else_clause" => {
                    else_branch = Some(self.parse_statement_list(child, source, file_path));
                }
                _ => {}
            }
        }

        Some(IfStatement {
            condition: condition?,
            then_branch,
            else_ifs,
            else_branch,
            location: self.node_to_location(&node, file_path),
        })
    }

    /// Парсит оператор возврата
    fn parse_return_statement(&self, node: Node, source: &str, file_path: &str) -> Option<ReturnStatement> {
        let mut value = None;

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() != "return_keyword" && child.kind() != ";" {
                value = self.parse_expression(child, source, file_path);
                break;
            }
        }

        Some(ReturnStatement {
            value,
            location: self.node_to_location(&node, file_path),
        })
    }

    /// Парсит выражение
    fn parse_expression(&self, node: Node, source: &str, file_path: &str) -> Option<Expression> {
        match node.kind() {
            "identifier" => {
                let name = node.utf8_text(source.as_bytes()).unwrap_or_default().to_string();
                Some(Expression::Identifier(name))
            }
            "number" => {
                let value = node.utf8_text(source.as_bytes()).unwrap_or_default().parse().unwrap_or(0.0);
                Some(Expression::Literal(Literal::Number(value)))
            }
            "string" => {
                let value = node.utf8_text(source.as_bytes()).unwrap_or_default().to_string();
                Some(Expression::Literal(Literal::String(value)))
            }
            "method_call" => {
                self.parse_method_call(node, source, file_path)
            }
            "property_access" => {
                self.parse_property_access(node, source, file_path)
            }
            "new_expression" => {
                self.parse_new_expression(node, source, file_path)
            }
            _ => None,
        }
    }

    /// Парсит вызов метода
    fn parse_method_call(&self, node: Node, source: &str, file_path: &str) -> Option<Expression> {
        let mut object = None;
        let mut method = String::new();
        let mut args = vec![];

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "expression" => {
                    if object.is_none() {
                        object = Some(Box::new(self.parse_expression(child, source, file_path)?));
                    }
                }
                "identifier" => {
                    method = child.utf8_text(source.as_bytes()).unwrap_or_default().to_string();
                }
                "argument_list" => {
                    args = self.parse_argument_list(child, source, file_path);
                }
                _ => {}
            }
        }

        Some(Expression::MethodCall(MethodCall {
            object: object?,
            method,
            args,
            location: self.node_to_location(&node, file_path),
        }))
    }

    /// Парсит обращение к свойству
    fn parse_property_access(&self, node: Node, source: &str, file_path: &str) -> Option<Expression> {
        let mut object = None;
        let mut property = String::new();

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "expression" => {
                    object = Some(Box::new(self.parse_expression(child, source, file_path)?));
                }
                "identifier" => {
                    property = child.utf8_text(source.as_bytes()).unwrap_or_default().to_string();
                }
                _ => {}
            }
        }

        Some(Expression::PropertyAccess(PropertyAccess {
            object: object?,
            property,
            location: self.node_to_location(&node, file_path),
        }))
    }

    /// Парсит создание объекта
    fn parse_new_expression(&self, node: Node, source: &str, file_path: &str) -> Option<Expression> {
        let mut type_name = String::new();
        let mut args = vec![];

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "identifier" => {
                    type_name = child.utf8_text(source.as_bytes()).unwrap_or_default().to_string();
                }
                "argument_list" => {
                    args = self.parse_argument_list(child, source, file_path);
                }
                _ => {}
            }
        }

        Some(Expression::New(NewExpression {
            type_name,
            args,
            location: self.node_to_location(&node, file_path),
        }))
    }

    /// Парсит список аргументов
    fn parse_argument_list(&self, node: Node, source: &str, file_path: &str) -> Vec<Expression> {
        let mut args = vec![];
        
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some(expr) = self.parse_expression(child, source, file_path) {
                args.push(expr);
            }
        }
        
        args
    }

    /// Конвертирует Node в Location
    fn node_to_location(&self, node: &Node, file_path: &str) -> Location {
        let start = node.start_position();
        let end = node.end_position();
        
        Location::new(
            file_path.to_string(),
            start.row + 1,
            start.column + 1,
            node.start_byte(),
            node.end_byte() - node.start_byte(),
        )
    }
}