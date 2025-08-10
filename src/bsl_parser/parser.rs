//! BSL Parser на базе tree-sitter

use super::tree_sitter_adapter::TreeSitterAdapter;
use crate::bsl_parser::{ast::*, diagnostics::*, Location, ArenaConverter};
use crate::ast_core::BuiltAst;
use anyhow::Result;

/// Результат парсинга
#[derive(Debug)]
pub struct ParseResult {
    pub ast: Option<BslAst>,
    pub diagnostics: Vec<Diagnostic>,
    pub arena: Option<BuiltAst>,
    pub parse_time_ns: u128,
    pub arena_time_ns: u128,
}

/// BSL Parser
pub struct BslParser {
    #[allow(dead_code)]
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
        let t_parse_start = std::time::Instant::now();
        // Парсинг через tree-sitter / заглушка
        let ast = self.parse_with_tree_sitter(source, file_path, &mut diagnostics);
        let parse_time_ns = t_parse_start.elapsed().as_nanos();
        let t_arena_start = std::time::Instant::now();
        let arena = ast.as_ref().map(|a| ArenaConverter::build_module(&a.module));
        let arena_time_ns = t_arena_start.elapsed().as_nanos();
        ParseResult { ast, diagnostics, arena, parse_time_ns, arena_time_ns }
    }

    /// Парсит код с использованием tree-sitter
    fn parse_with_tree_sitter(
        &self,
        source: &str,
        file_path: &str,
        diagnostics: &mut [Diagnostic],
    ) -> Option<BslAst> {
        // TODO: Включить когда будет решена проблема с tree-sitter-language версиями
        // В данный момент tree-sitter 0.22 не совместим с LanguageFn из tree-sitter-bsl

        // ВРЕМЕННАЯ УЛУЧШЕННАЯ ЗАГЛУШКА - парсит простую структуру BSL
        self.parse_simple_bsl_structure(source, file_path, diagnostics)
    }

    /// Простой парсер BSL структуры (временная замена tree-sitter)
    fn parse_simple_bsl_structure(
        &self,
        source: &str,
        file_path: &str,
        _diagnostics: &mut [Diagnostic],
    ) -> Option<BslAst> {
        let mut declarations = Vec::new();
        let lines: Vec<&str> = source.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let trimmed = lines[i].trim();

            // Парсим объявления процедур
            if trimmed.starts_with("Процедура ") || trimmed.starts_with("Procedure ") {
                if let Some((proc, lines_consumed)) =
                    self.parse_simple_procedure_with_body(&lines, i, file_path)
                {
                    declarations.push(Declaration::Procedure(proc));
                    i += lines_consumed;
                    continue;
                }
            }

            // Парсим объявления функций
            if trimmed.starts_with("Функция ") || trimmed.starts_with("Function ") {
                if let Some((func, lines_consumed)) =
                    self.parse_simple_function_with_body(&lines, i, file_path)
                {
                    declarations.push(Declaration::Function(func));
                    i += lines_consumed;
                    continue;
                }
            }

            // Парсим объявления функций
            if trimmed.starts_with("Функция ") || trimmed.starts_with("Function ") {
                if let Some(func) = self.parse_simple_function(trimmed, i + 1, file_path) {
                    declarations.push(Declaration::Function(func));
                }
            }

            // Парсим объявления процедур
            if trimmed.starts_with("Процедура ") || trimmed.starts_with("Procedure ") {
                if let Some(proc) = self.parse_simple_procedure(trimmed, i + 1, file_path) {
                    declarations.push(Declaration::Procedure(proc));
                }
            }

            // Парсим объявления переменных
            if trimmed.starts_with("Перем ") || trimmed.starts_with("Var ") {
                if let Some(var) = self.parse_simple_variable(trimmed, i + 1, file_path) {
                    declarations.push(Declaration::Variable(var));
                }
            }

            i += 1;
        }

        Some(BslAst {
            module: Module {
                directives: vec![],
                declarations,
                location: Location::new(file_path.to_string(), 1, 1, 0, source.len()),
            },
        })
    }

    /// Парсит простое объявление процедуры
    fn parse_simple_procedure(
        &self,
        line: &str,
        line_num: usize,
        file_path: &str,
    ) -> Option<ProcedureDecl> {
        // Простой regex-парсинг: "Процедура Имя(параметры) Экспорт"
        let export = line.contains("Экспорт") || line.contains("Export");

        // Извлекаем имя процедуры с правильной обработкой UTF-8
        let name_start = if line.starts_with("Процедура ") {
            "Процедура ".len()
        } else {
            "Procedure ".len()
        };
        let name_end = line.find('(').unwrap_or_else(|| {
            // Если нет скобок, ищем пробел или "Экспорт"
            line.find(" Экспорт")
                .or_else(|| line.find(" Export"))
                .unwrap_or(line.len())
        });
        let name = line[name_start..name_end].trim().to_string();

        Some(ProcedureDecl {
            name,
            export,
            params: vec![], // Упрощенно - не парсим параметры
            directives: vec![],
            body: vec![],
            location: Location::new(file_path.to_string(), line_num, 1, 0, line.len()),
        })
    }

    /// Парсит простое объявление функции
    fn parse_simple_function(
        &self,
        line: &str,
        line_num: usize,
        file_path: &str,
    ) -> Option<FunctionDecl> {
        let export = line.contains("Экспорт") || line.contains("Export");

        // Правильное извлечение имени функции с UTF-8
        let name_start = if line.starts_with("Функция ") {
            "Функция ".len()
        } else {
            "Function ".len()
        };
        let name_end = line.find('(').unwrap_or_else(|| {
            // Если нет скобок, ищем пробел или "Экспорт"
            line.find(" Экспорт")
                .or_else(|| line.find(" Export"))
                .unwrap_or(line.len())
        });
        let name = line[name_start..name_end].trim().to_string();

        Some(FunctionDecl {
            name,
            export,
            params: vec![],
            directives: vec![],
            body: vec![],
            location: Location::new(file_path.to_string(), line_num, 1, 0, line.len()),
        })
    }

    /// Парсит простое объявление переменной
    fn parse_simple_variable(
        &self,
        line: &str,
        line_num: usize,
        file_path: &str,
    ) -> Option<VariableDecl> {
        let export = line.contains("Экспорт") || line.contains("Export");

        let name_start = if line.starts_with("Перем ") {
            6
        } else {
            4
        }; // "Var " = 4 символа
        let names_part = &line[name_start..];
        let names = names_part
            .split(',')
            .map(|s| s.split_whitespace().next().unwrap_or("").to_string())
            .filter(|s| !s.is_empty() && s != "Экспорт" && s != "Export")
            .collect();

        Some(VariableDecl {
            names,
            export,
            location: Location::new(file_path.to_string(), line_num, 1, 0, line.len()),
        })
    }

    /// Парсит процедуру с телом
    fn parse_simple_procedure_with_body(
        &self,
        lines: &[&str],
        start_idx: usize,
        file_path: &str,
    ) -> Option<(ProcedureDecl, usize)> {
        let header_line = lines[start_idx].trim();
        let export = header_line.contains("Экспорт") || header_line.contains("Export");

        // Извлекаем имя процедуры и параметры
        let name_start = if header_line.starts_with("Процедура ") {
            "Процедура ".len()
        } else {
            "Procedure ".len()
        };
        let paren_pos = header_line.find('(')?;
        let name = header_line[name_start..paren_pos].trim().to_string();

        // Парсим параметры
        let params = self.parse_simple_parameters(header_line, start_idx + 1, file_path);

        // Ищем тело процедуры до КонецПроцедуры
        let mut body = Vec::new();
        let mut lines_consumed = 1;
        let mut current_idx = start_idx + 1;

        while current_idx < lines.len() {
            let line = lines[current_idx].trim();
            if line.starts_with("КонецПроцедуры") || line.starts_with("EndProcedure")
            {
                lines_consumed = current_idx - start_idx + 1;
                break;
            }

            // Парсим операторы в теле
            if let Some(stmt) = self.parse_simple_statement(line, current_idx + 1, file_path) {
                body.push(stmt);
            }

            current_idx += 1;
        }

        Some((
            ProcedureDecl {
                name,
                export,
                params,
                directives: vec![],
                body,
                location: Location::new(
                    file_path.to_string(),
                    start_idx + 1,
                    1,
                    0,
                    header_line.len(),
                ),
            },
            lines_consumed,
        ))
    }

    /// Парсит функцию с телом
    fn parse_simple_function_with_body(
        &self,
        lines: &[&str],
        start_idx: usize,
        file_path: &str,
    ) -> Option<(FunctionDecl, usize)> {
        let header_line = lines[start_idx].trim();
        let export = header_line.contains("Экспорт") || header_line.contains("Export");

        let name_start = if header_line.starts_with("Функция ") {
            "Функция ".len()
        } else {
            "Function ".len()
        };
        let paren_pos = header_line.find('(')?;
        let name = header_line[name_start..paren_pos].trim().to_string();

        let params = self.parse_simple_parameters(header_line, start_idx + 1, file_path);

        let mut body = Vec::new();
        let mut lines_consumed = 1;
        let mut current_idx = start_idx + 1;

        while current_idx < lines.len() {
            let line = lines[current_idx].trim();
            if line.starts_with("КонецФункции") || line.starts_with("EndFunction") {
                lines_consumed = current_idx - start_idx + 1;
                break;
            }

            if let Some(stmt) = self.parse_simple_statement(line, current_idx + 1, file_path) {
                body.push(stmt);
            }

            current_idx += 1;
        }

        Some((
            FunctionDecl {
                name,
                export,
                params,
                directives: vec![],
                body,
                location: Location::new(
                    file_path.to_string(),
                    start_idx + 1,
                    1,
                    0,
                    header_line.len(),
                ),
            },
            lines_consumed,
        ))
    }

    /// Парсит параметры функции/процедуры
    fn parse_simple_parameters(
        &self,
        line: &str,
        line_num: usize,
        file_path: &str,
    ) -> Vec<Parameter> {
        let mut params = Vec::new();

        if let Some(start) = line.find('(') {
            if let Some(end) = line.find(')') {
                let params_str = &line[start + 1..end];
                if !params_str.trim().is_empty() {
                    for param_str in params_str.split(',') {
                        let param_parts: Vec<&str> = param_str.split_whitespace().collect();
                        if let Some(param_name) = param_parts.first() {
                            if !param_name.is_empty() {
                                params.push(Parameter {
                                    name: param_name.to_string(),
                                    by_val: param_str.contains("Знач") || param_str.contains("Val"),
                                    default_value: None, // Упрощенно
                                    location: Location::new(
                                        file_path.to_string(),
                                        line_num,
                                        1,
                                        0,
                                        param_str.len(),
                                    ),
                                });
                            }
                        }
                    }
                }
            }
        }

        params
    }

    /// Парсит простой оператор
    fn parse_simple_statement(
        &self,
        line: &str,
        line_num: usize,
        file_path: &str,
    ) -> Option<Statement> {
        let trimmed = line.trim();

        // Пропускаем пустые строки и комментарии
        if trimmed.is_empty() || trimmed.starts_with("//") {
            return None;
        }

        // Парсим присваивание
        if trimmed.contains(" = ") {
            return self
                .parse_simple_assignment(trimmed, line_num, file_path)
                .map(Statement::Assignment);
        }

        // Парсим условный оператор
        if trimmed.starts_with("Если ") || trimmed.starts_with("If ") {
            return self
                .parse_simple_if(trimmed, line_num, file_path)
                .map(Statement::If);
        }

        // Парсим возврат
        if trimmed.starts_with("Возврат ") || trimmed.starts_with("Return ") {
            return self
                .parse_simple_return(trimmed, line_num, file_path)
                .map(Statement::Return);
        }

        // Парсим объявление переменных
        if trimmed.starts_with("Перем ") || trimmed.starts_with("Var ") {
            return None; // Уже обработано выше
        }

        // Парсим вызов процедуры/функции как выражение
        if let Some(expr) = self.parse_simple_expression(trimmed, line_num, file_path) {
            return Some(Statement::Expression(expr));
        }

        None
    }

    /// Парсит присваивание
    fn parse_simple_assignment(
        &self,
        line: &str,
        line_num: usize,
        file_path: &str,
    ) -> Option<Assignment> {
        let parts: Vec<&str> = line.splitn(2, " = ").collect();
        if parts.len() == 2 {
            let target = self.parse_simple_expression(parts[0].trim(), line_num, file_path)?;
            let value = self.parse_simple_expression(parts[1].trim(), line_num, file_path)?;

            Some(Assignment {
                target,
                value,
                location: Location::new(file_path.to_string(), line_num, 1, 0, line.len()),
            })
        } else {
            None
        }
    }

    /// Парсит условный оператор (упрощенно)
    fn parse_simple_if(&self, line: &str, line_num: usize, file_path: &str) -> Option<IfStatement> {
        // Упрощенный парсинг условия с правильной обработкой UTF-8
        let keyword_len = if line.starts_with("Если ") {
            "Если ".len()
        } else if line.starts_with("If ") {
            "If ".len()
        } else {
            return None;
        };

        let condition_end = line.find(" Тогда").or_else(|| line.find(" Then"))?;
        let condition_str = &line[keyword_len..condition_end];

        self.parse_simple_expression(condition_str, line_num, file_path)
            .map(|condition| IfStatement {
                condition,
                then_branch: vec![], // Упрощенно - не парсим тело
                else_ifs: vec![],
                else_branch: None,
                location: Location::new(file_path.to_string(), line_num, 1, 0, line.len()),
            })
    }

    /// Парсит возврат
    fn parse_simple_return(
        &self,
        line: &str,
        line_num: usize,
        file_path: &str,
    ) -> Option<ReturnStatement> {
        let keyword_len = if line.starts_with("Возврат ") {
            "Возврат ".len()
        } else if line.starts_with("Return ") {
            "Return ".len()
        } else {
            return None;
        };
        let value_str = line[keyword_len..].trim();

        let value = if value_str.is_empty() {
            None
        } else {
            self.parse_simple_expression(value_str, line_num, file_path)
        };

        Some(ReturnStatement {
            value,
            location: Location::new(file_path.to_string(), line_num, 1, 0, line.len()),
        })
    }

    /// Парсит простое выражение
    fn parse_simple_expression(
        &self,
        expr: &str,
        line_num: usize,
        file_path: &str,
    ) -> Option<Expression> {
        let trimmed = expr.trim();

        // Строковый литерал
        if (trimmed.starts_with('"') && trimmed.ends_with('"'))
            || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
        {
            return Some(Expression::Literal(Literal::String(trimmed.to_string())));
        }

        // Числовой литерал
        if trimmed.parse::<f64>().is_ok() {
            return Some(Expression::Literal(Literal::Number(
                trimmed.parse().unwrap_or(0.0),
            )));
        }

        // Вызов глобальной функции (содержит скобки, но без точки)
        if trimmed.contains('(') && !trimmed.contains('.') {
            return self.parse_simple_function_call(trimmed, line_num, file_path);
        }

        // Вызов метода (содержит точку)
        if trimmed.contains('.') && trimmed.contains('(') {
            return self.parse_simple_method_call(trimmed, line_num, file_path);
        }

        // Создание объекта
        if trimmed.starts_with("Новый ") || trimmed.starts_with("New ") {
            return self.parse_simple_new_expression(trimmed, line_num, file_path);
        }

        // Обращение к свойству
        if trimmed.contains('.') && !trimmed.contains('(') {
            return self.parse_simple_property_access(trimmed, line_num, file_path);
        }

        // Простой идентификатор
        if trimmed.chars().all(|c| {
            c.is_alphanumeric()
                || c == '_'
                || "АБВГДЕЁЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯабвгдеёжзийклмнопрстуфхцчшщъыьэюя".contains(c)
        }) {
            return Some(Expression::Identifier(trimmed.to_string()));
        }

        None
    }

    /// Парсит вызов метода
    fn parse_simple_method_call(
        &self,
        expr: &str,
        line_num: usize,
        file_path: &str,
    ) -> Option<Expression> {
        let dot_pos = expr.find('.')?;
        let paren_pos = expr.find('(')?;

        if dot_pos < paren_pos {
            let object_str = &expr[..dot_pos];
            let method_start = dot_pos + 1;
            let method_name = &expr[method_start..paren_pos];

            let object = Box::new(Expression::Identifier(object_str.to_string()));

            // Простой парсинг аргументов (упрощенно)
            let args = vec![]; // TODO: парсить аргументы

            Some(Expression::MethodCall(MethodCall {
                object,
                method: method_name.to_string(),
                args,
                location: Location::new(file_path.to_string(), line_num, 1, 0, expr.len()),
            }))
        } else {
            None
        }
    }

    /// Парсит создание объекта
    fn parse_simple_new_expression(
        &self,
        expr: &str,
        line_num: usize,
        file_path: &str,
    ) -> Option<Expression> {
        let type_start = if expr.starts_with("Новый ") {
            6
        } else {
            4
        };
        let paren_pos = expr.find('(')?;
        let type_name = expr[type_start..paren_pos].trim();

        Some(Expression::New(NewExpression {
            type_name: type_name.to_string(),
            args: vec![], // Упрощенно
            location: Location::new(file_path.to_string(), line_num, 1, 0, expr.len()),
        }))
    }

    /// Парсит вызов глобальной функции
    fn parse_simple_function_call(
        &self,
        expr: &str,
        line_num: usize,
        file_path: &str,
    ) -> Option<Expression> {
        let paren_pos = expr.find('(')?;
        let function_name = expr[..paren_pos].trim();

        // Проверяем, что это допустимая функция
        if !function_name.chars().all(|c| {
            c.is_alphanumeric()
                || c == '_'
                || "АБВГДЕЁЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯабвгдеёжзийклмнопрстуфхцчшщъыьэюя".contains(c)
        }) {
            return None;
        }

        // Парсим аргументы (упрощенно)
        let close_paren = expr.rfind(')')?;
        let args_str = &expr[paren_pos + 1..close_paren];
        let mut args = Vec::new();

        if !args_str.trim().is_empty() {
            for arg_str in args_str.split(',') {
                if let Some(arg_expr) =
                    self.parse_simple_expression(arg_str.trim(), line_num, file_path)
                {
                    args.push(arg_expr);
                }
            }
        }

        Some(Expression::FunctionCall(FunctionCall {
            name: function_name.to_string(),
            args,
            location: Location::new(file_path.to_string(), line_num, 1, 0, expr.len()),
        }))
    }

    /// Парсит обращение к свойству
    fn parse_simple_property_access(
        &self,
        expr: &str,
        line_num: usize,
        file_path: &str,
    ) -> Option<Expression> {
        let dot_pos = expr.find('.')?;
        let object_str = &expr[..dot_pos];
        let property_name = &expr[dot_pos + 1..];

        let object = Box::new(Expression::Identifier(object_str.to_string()));

        Some(Expression::PropertyAccess(PropertyAccess {
            object,
            property: property_name.to_string(),
            location: Location::new(file_path.to_string(), line_num, 1, 0, expr.len()),
        }))
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
                                    method_call.method, type_name
                                ),
                            )
                            .with_found(&method_call.method)
                            .with_expected(
                                entity
                                    .interface
                                    .methods
                                    .keys()
                                    .take(3)
                                    .cloned()
                                    .collect::<Vec<_>>()
                                    .join(", "),
                            ),
                        );
                    }
                } else {
                    diagnostics.push(Diagnostic::new(
                        DiagnosticSeverity::Warning,
                        method_call.location.clone(),
                        codes::UNKNOWN_CONSTRUCT,
                        format!("Неизвестный тип '{}'", type_name),
                    ));
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
        let parser = BslParser::new().unwrap();
        let result = parser.parse("", "test.bsl");

        assert!(result.ast.is_some());
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_simple() {
        let parser = BslParser::new().unwrap();
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
