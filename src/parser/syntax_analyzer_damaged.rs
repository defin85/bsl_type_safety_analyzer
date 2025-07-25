/*!
# BSL Syntax Analyzer

Синтаксический анализатор для построения AST из токенов BSL.
Портирован с Python версии с полной поддержкой конструкций языка.
*/

use std::collections::HashMap;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

use crate::parser::lexer::{Token, TokenType};
use crate::parser::ast::{AstNode, AstNodeType, Position, Span};
use crate::core::errors::{AnalysisError, ErrorLevel};

/// Синтаксический анализатор BSL
pub struct SyntaxAnalyzer {
                self.current_index += 1; // Пропускаем '='
                
                let mut assignment = AstNode::new(AstNodeType::Assignment, Span::new(start_pos, self.current_position()));
                
                // Добавляем левую часть (имя переменной)
                let left_side = AstNode::identifier(Span::new(start_pos, self.current_position()), identifier);
                assignment.add_child(left_side);
                
                // Парсим правую часть (выражение)
                let right_side = self.parse_simple_expression()?;
                assignment.add_child(right_side);
                
                // Пропускаем точку с запятой, если есть
                if self.match_token(&TokenType::Semicolon) {
                    self.current_index += 1;
                }
                
                Ok(Some(assignment))
            } else if self.match_token(&TokenType::LeftParen) {
                // Это вызов функции/процедуры
                self.current_index += 1; // Пропускаем '('
                
                let mut call = AstNode::new(AstNodeType::CallExpression, Span::new(start_pos, self.current_position()));
                call.value = Some(identifier);
                
                // Парсим аргументы
                while !self.match_token(&TokenType::RightParen) && self.current_index < self.tokens.len() {
                    let arg = self.parse_simple_expression()?;
                    call.add_child(arg);
                    
                    // Пропускаем запятую, если есть
                    if self.match_token(&TokenType::Comma) {
                        self.current_index += 1;
                    }
                }
                
                if self.match_token(&TokenType::RightParen) {
                    self.current_index += 1; // Пропускаем ')'
                }
                
                // Пропускаем точку с запятой, если есть
                if self.match_token(&TokenType::Semicolon) {
                    self.current_index += 1;
                }
                
                Ok(Some(call))
            } else if self.match_token(&TokenType::Dot) {
                // Это доступ к члену объекта (может быть вызов метода)
                self.current_index += 1; // Пропускаем '.'
                
                if let Some(member_name) = self.consume_identifier() {
                    if self.match_token(&TokenType::LeftParen) {
                        // Это вызов метода объекта
                        self.current_index += 1; // Пропускаем '('
                        
                        let mut call = AstNode::new(AstNodeType::CallExpression, Span::new(start_pos, self.current_position()));
                        call.value = Some(format!("{}.{}", identifier, member_name));
                        
                        // Парсим аргументы
                        while !self.match_token(&TokenType::RightParen) && self.current_index < self.tokens.len() {
                            let arg = self.parse_simple_expression()?;
                            call.add_child(arg);
                            
                            // Пропускаем запятую, если есть
                            if self.match_token(&TokenType::Comma) {
                                self.current_index += 1;
                            }
                        }
                        
                        if self.match_token(&TokenType::RightParen) {
                            self.current_index += 1; // Пропускаем ')'
                        }
                        
                        // Пропускаем точку с запятой, если есть
                        if self.match_token(&TokenType::Semicolon) {
                            self.current_index += 1;
                        }
                        
                        Ok(Some(call))
                    } else {
                        // Это просто доступ к свойству
                        let member_access = AstNode::new(AstNodeType::MemberExpression, Span::new(start_pos, self.current_position()));
                        Ok(Some(member_access))
                    }
                } else {
                    // Ошибка - ожидался идентификатор после точки
                    let node = AstNode::identifier(Span::new(start_pos, self.current_position()), identifier);
                    Ok(Some(node))
                }
            } else {
                // Просто идентификатор
                let node = AstNode::identifier(Span::new(start_pos, self.current_position()), identifier);
                Ok(Some(node))
            }
        } else {
            Ok(None)
        }
    }an};
use crate::core::errors::{AnalysisError, ErrorLevel};

/// Синтаксический анализатор BSL
pub struct SyntaxAnalyzer {
    /// Текущая позиция в списке токенов
    current_index: usize,
    /// Список токенов для анализа
    tokens: Vec<Token>,
    /// Собранные ошибки парсинга
    errors: Vec<AnalysisError>,
    /// Собранные предупреждения
    warnings: Vec<AnalysisError>,
    /// Включен ли verbose режим
    verbose: bool,
}

impl SyntaxAnalyzer {
    /// Создает новый синтаксический анализатор
    pub fn new() -> Self {
        Self {
            current_index: 0,
            tokens: Vec::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
            verbose: false,
        }
    }
    
    /// Включает verbose режим
    pub fn set_verbose(&mut self, verbose: bool) {
        self.verbose = verbose;
    }
    
    /// Анализирует токены и строит AST
    pub fn analyze(&mut self, tokens: Vec<Token>) -> Result<AstNode> {
        self.clear_results();
        self.tokens = tokens;
        self.current_index = 0;
        
        self.parse_module()
    }
    
    /// Парсит код BSL в AST (удобный метод)
    pub fn parse(&mut self, code: &str) -> Result<AstNode> {
        // Используем лексер для получения токенов
        use crate::parser::lexer::BslLexer;
        let lexer = BslLexer::new();
        let tokens = lexer.tokenize(code).map_err(|e| anyhow!("Ошибка токенизации: {}", e))?;
        
        // Анализируем токены
        self.analyze(tokens)
    }
    
    /// Парсит модуль (основная точка входа для анализа)
    fn parse_module(&mut self) -> Result<AstNode> {        
        if self.verbose {
            println!("Начинаем синтаксический анализ, токенов: {}", self.tokens.len());
        }
        
        if self.tokens.is_empty() {
            self.add_warning("Токены отсутствуют, создаем пустое AST".to_string(), Position::zero());
            return Ok(AstNode::module(Span::zero()));
        }
        
        self.build_ast()
    }
    
    /// Очищает результаты предыдущего анализа
    fn clear_results(&mut self) {
        self.errors.clear();
        self.warnings.clear();
    }
    
    /// Строит абстрактное синтаксическое дерево
    fn build_ast(&mut self) -> Result<AstNode> {
        // Создаем корневой узел программы
        let mut root = AstNode::module(Span::zero());
        
        // Парсим операторы до конца токенов
        while self.current_index < self.tokens.len() {
            match self.parse_statement() {
                Ok(Some(node)) => {
                    root.add_child(node);
                }
                Ok(None) => {
                    // Пропускаем неизвестные токены
                    self.current_index += 1;
                }
                Err(e) => {
                    self.add_error(format!("Ошибка парсинга: {}", e), self.current_position());
                    self.current_index += 1; // Пропускаем проблемный токен
                }
            }
        }
        
        if self.verbose {
            println!("AST построен успешно, узлов: {}", self.count_nodes(&root));
        }
        
        Ok(root)
    }
    
    /// Парсит оператор
    fn parse_statement(&mut self) -> Result<Option<AstNode>> {
        if self.current_index >= self.tokens.len() {
            return Ok(None);
        }
        
        let token = &self.tokens[self.current_index];
        
        match &token.token_type {
            TokenType::Процедура => self.parse_procedure(),
            TokenType::Функция => self.parse_function(),
            TokenType::Перем => self.parse_variable_declaration(),
            TokenType::If => self.parse_if_statement(),
            TokenType::For => self.parse_for_loop(),
            TokenType::While => self.parse_while_loop(),
            TokenType::Попытка => self.parse_try_statement(),
            TokenType::Identifier => self.parse_assignment_or_call(),
            _ => {
                // Пропускаем неизвестные токены
                Ok(None)
            }
        }
    }
    
    /// Парсит процедуру
    fn parse_procedure(&mut self) -> Result<Option<AstNode>> {
        let start_pos = self.current_position();
        self.current_index += 1; // Пропускаем 'Процедура'
        
        // Получаем имя процедуры
        let name = if let Some(name_token) = self.consume_identifier() {
            name_token
        } else {
            self.add_error("Ожидается имя процедуры".to_string(), self.current_position());
            return Ok(None);
        };
        
        let mut procedure = AstNode::procedure(Span::new(start_pos, self.current_position()), name);
        
        // Парсим параметры
        if self.match_token(&TokenType::LeftParen) {
            self.current_index += 1; // Пропускаем '('
            
            let mut params = AstNode::new(AstNodeType::ParameterList, Span::new(start_pos, self.current_position()));
            
            while !self.match_token(&TokenType::RightParen) && self.current_index < self.tokens.len() {
                if let Some(param_name) = self.consume_identifier() {
                    let param = AstNode::parameter(Span::new(start_pos, self.current_position()), param_name);
                    params.add_child(param);
                }
                
                // Пропускаем запятую, если есть
                if self.match_token(&TokenType::Comma) {
                    self.current_index += 1;
                }
            }
            
            if self.match_token(&TokenType::RightParen) {
                self.current_index += 1; // Пропускаем ')'
                procedure.add_child(params);
            }
        }
        
        // Парсим тело процедуры
        let body = self.parse_block(&TokenType::КонецПроцедуры)?;
        procedure.add_child(body);
        
        // Пропускаем 'КонецПроцедуры'
        if self.match_token(&TokenType::КонецПроцедуры) {
            self.current_index += 1;
        }
        
        Ok(Some(procedure))
    }
    
    /// Парсит функцию
    fn parse_function(&mut self) -> Result<Option<AstNode>> {
        let start_pos = self.current_position();
        self.current_index += 1; // Пропускаем 'Функция'
        
        // Получаем имя функции
        let name = if let Some(name_token) = self.consume_identifier() {
            name_token
        } else {
            self.add_error("Ожидается имя функции".to_string(), self.current_position());
            return Ok(None);
        };
        
        let mut function = AstNode::function(Span::new(start_pos, self.current_position()), name);
        
        // Парсим параметры (аналогично процедуре)
        if self.match_token(&TokenType::LeftParen) {
            self.current_index += 1;
            
            let mut params = AstNode::new(AstNodeType::ParameterList, Span::new(start_pos, self.current_position()));
            
            while !self.match_token(&TokenType::RightParen) && self.current_index < self.tokens.len() {
                if let Some(param_name) = self.consume_identifier() {
                    let param = AstNode::parameter(Span::new(start_pos, self.current_position()), param_name);
                    params.add_child(param);
                }
                
                if self.match_token(&TokenType::Comma) {
                    self.current_index += 1;
                }
            }
            
            if self.match_token(&TokenType::RightParen) {
                self.current_index += 1;
                function.add_child(params);
            }
        }
        
        // Парсим тело функции
        let body = self.parse_block(&TokenType::КонецФункции)?;
        function.add_child(body);
        
        // Пропускаем 'КонецФункции'
        if self.match_token(&TokenType::КонецФункции) {
            self.current_index += 1;
        }
        
        Ok(Some(function))
    }
    
    /// Парсит объявление переменной
    fn parse_variable_declaration(&mut self) -> Result<Option<AstNode>> {
        let start_pos = self.current_position();
        self.current_index += 1; // Пропускаем 'Перем'
        
        if let Some(var_name) = self.consume_identifier() {
            let var_decl = AstNode::with_value(
                AstNodeType::VariableDeclaration,
                Span::new(start_pos, self.current_position()),
                var_name
            );
            
            // Пропускаем точку с запятой, если есть
            if self.match_token(&TokenType::Semicolon) {
                self.current_index += 1;
            }
            
            Ok(Some(var_decl))
        } else {
            self.add_error("Ожидается имя переменной".to_string(), self.current_position());
            Ok(None)
        }
    }
    
    /// Парсит условную конструкцию
    fn parse_if_statement(&mut self) -> Result<Option<AstNode>> {
        let start_pos = self.current_position();
        self.current_index += 1; // Пропускаем 'Если'
        
        let mut if_stmt = AstNode::new(AstNodeType::IfStatement, Span::new(start_pos, self.current_position()));
        
        // Парсим условие
        let condition = self.parse_expression_until(&TokenType::Then)?;
        if_stmt.add_child(condition);
        
        // Пропускаем 'Тогда'
        if self.match_token(&TokenType::Then) {
            self.current_index += 1;
        }
        
        // Парсим блок 'Тогда'
        let then_block = self.parse_block_until(&[TokenType::Else, TokenType::EndIf])?;
        if_stmt.add_child(then_block);
        
        // Парсим блок 'Иначе', если есть
        if self.match_token(&TokenType::Else) {
            self.current_index += 1;
            let else_block = self.parse_block(&TokenType::EndIf)?;
            if_stmt.add_child(else_block);
        }
        
        // Пропускаем 'КонецЕсли'
        if self.match_token(&TokenType::EndIf) {
            self.current_index += 1;
        }
        
        Ok(Some(if_stmt))
    }
    
    /// Парсит цикл Для
    fn parse_for_loop(&mut self) -> Result<Option<AstNode>> {
        let start_pos = self.current_position();
        self.current_index += 1; // Пропускаем 'Для'
        
        let mut for_loop = AstNode::new(AstNodeType::ForLoop, Span::new(start_pos, self.current_position()));
        
        // Парсим тело цикла
        let body = self.parse_block(&TokenType::EndDo)?;
        for_loop.add_child(body);
        
        // Пропускаем 'КонецЦикла'
        if self.match_token(&TokenType::EndDo) {
            self.current_index += 1;
        }
        
        Ok(Some(for_loop))
    }
    
    /// Парсит цикл Пока
    fn parse_while_loop(&mut self) -> Result<Option<AstNode>> {
        let start_pos = self.current_position();
        self.current_index += 1; // Пропускаем 'Пока'
        
        let mut while_loop = AstNode::new(AstNodeType::WhileLoop, Span::new(start_pos, self.current_position()));
        
        // Парсим тело цикла
        let body = self.parse_block(&TokenType::EndDo)?;
        while_loop.add_child(body);
        
        // Пропускаем 'КонецЦикла'
        if self.match_token(&TokenType::EndDo) {
            self.current_index += 1;
        }
        
        Ok(Some(while_loop))
    }
    
    /// Парсит блок попытки
    fn parse_try_statement(&mut self) -> Result<Option<AstNode>> {
        let start_pos = self.current_position();
        self.current_index += 1; // Пропускаем 'Попытка'
        
        let mut try_stmt = AstNode::new(AstNodeType::TryStatement, Span::new(start_pos, self.current_position()));
        
        // Парсим тело попытки
        let body = self.parse_block_until(&[TokenType::Исключение, TokenType::КонецПопытки])?;
        try_stmt.add_child(body);
        
        // Парсим блок исключения, если есть
        if self.match_token(&TokenType::Исключение) {
            self.current_index += 1;
            let exception_block = self.parse_block(&TokenType::КонецПопытки)?;
            try_stmt.add_child(exception_block);
        }
        
        // Пропускаем 'КонецПопытки'
        if self.match_token(&TokenType::КонецПопытки) {
            self.current_index += 1;
        }
        
        Ok(Some(try_stmt))
    }
    
    /// Парсит присваивание или вызов метода
    fn parse_assignment_or_call(&mut self) -> Result<Option<AstNode>> {
        let start_pos = self.current_position();
        
        if let Some(identifier) = self.consume_identifier() {
            // Смотрим, что идет после идентификатора
            if self.match_token(&TokenType::Equal) {
                // Это присваивание
                self.current_index += 1; // Пропускаем '='
                
                let mut assignment = AstNode::new(AstNodeType::Assignment, Span::new(start_pos, self.current_position()));
                
                // Добавляем левую часть (имя переменной)
                let left_side = AstNode::identifier(Span::new(start_pos, self.current_position()), identifier);
                assignment.add_child(left_side);
                
                // Парсим правую часть (выражение)
                let right_side = self.parse_simple_expression()?;
                assignment.add_child(right_side);
                
                // Пропускаем точку с запятой, если есть
                if self.match_token(&TokenType::Semicolon) {
                    self.current_index += 1;
                }
                
                Ok(Some(assignment))
            } else if self.match_token(&TokenType::LeftParen) {
                // Это вызов функции/процедуры
                self.current_index += 1; // Пропускаем '('
                
                let mut call = AstNode::new(AstNodeType::CallExpression, Span::new(start_pos, self.current_position()));
                call.value = Some(identifier);
                
                // Парсим аргументы
                while !self.match_token(&TokenType::RightParen) && self.current_index < self.tokens.len() {
                    let arg = self.parse_simple_expression()?;
                    call.add_child(arg);
                    
                    // Пропускаем запятую, если есть
                    if self.match_token(&TokenType::Comma) {
                        self.current_index += 1;
                    }
                }
                
                if self.match_token(&TokenType::RightParen) {
                    self.current_index += 1; // Пропускаем ')'
                }
                
                // Пропускаем точку с запятой, если есть
                if self.match_token(&TokenType::Semicolon) {
                    self.current_index += 1;
                }
                
                Ok(Some(call))
            } else if self.match_token(&TokenType::Dot) {
                // Это доступ к члену объекта (может быть вызов метода)
                self.current_index += 1; // Пропускаем '.'
                
                if let Some(member_name) = self.consume_identifier() {
                    if self.match_token(&TokenType::LeftParen) {
                        // Это вызов метода объекта
                        self.current_index += 1; // Пропускаем '('
                        
                        let mut call = AstNode::new(AstNodeType::CallExpression, Span::new(start_pos, self.current_position()));
                        call.value = Some(format!("{}.{}", identifier, member_name));
                        
                        // Парсим аргументы
                        while !self.match_token(&TokenType::RightParen) && self.current_index < self.tokens.len() {
                            let arg = self.parse_simple_expression()?;
                            call.add_child(arg);
                            
                            // Пропускаем запятую, если есть
                            if self.match_token(&TokenType::Comma) {
                                self.current_index += 1;
                            }
                        }
                        
                        if self.match_token(&TokenType::RightParen) {
                            self.current_index += 1; // Пропускаем ')'
                        }
                        
                        // Пропускаем точку с запятой, если есть
                        if self.match_token(&TokenType::Semicolon) {
                            self.current_index += 1;
                        }
                        
                        Ok(Some(call))
                    } else {
                        // Это просто доступ к свойству
                        let member_access = AstNode::new(AstNodeType::MemberExpression, Span::new(start_pos, self.current_position()));
                        Ok(Some(member_access))
                    }
                } else {
                    // Ошибка - ожидался идентификатор после точки
                    let node = AstNode::identifier(Span::new(start_pos, self.current_position()), identifier);
                    Ok(Some(node))
                }
            } else {
                // Просто идентификатор
                let node = AstNode::identifier(Span::new(start_pos, self.current_position()), identifier);
                Ok(Some(node))
            }
        } else {
            Ok(None)
        }
    }
    
    /// Парсит блок до указанного токена
    fn parse_block(&mut self, end_token: &TokenType) -> Result<AstNode> {
        let start_pos = self.current_position();
        let mut block = AstNode::new(AstNodeType::Block, Span::new(start_pos, self.current_position()));
        
        while !self.match_token(end_token) && self.current_index < self.tokens.len() {
            if let Ok(Some(stmt)) = self.parse_statement() {
                block.add_child(stmt);
            } else {
                self.current_index += 1; // Пропускаем проблемный токен
            }
        }
        
        Ok(block)
    }
    
    /// Парсит блок до одного из указанных токенов
    fn parse_block_until(&mut self, end_tokens: &[TokenType]) -> Result<AstNode> {
        let start_pos = self.current_position();
        let mut block = AstNode::new(AstNodeType::Block, Span::new(start_pos, self.current_position()));
        
        while !self.match_any_token(end_tokens) && self.current_index < self.tokens.len() {
            if let Ok(Some(stmt)) = self.parse_statement() {
                block.add_child(stmt);
            } else {
                self.current_index += 1;
            }
        }
        
        Ok(block)
    }
    
    /// Парсит выражение до указанного токена
    fn parse_expression_until(&mut self, end_token: &TokenType) -> Result<AstNode> {
        let start_pos = self.current_position();
        let mut expr = AstNode::new(AstNodeType::Expression, Span::new(start_pos, self.current_position()));
        
        while !self.match_token(end_token) && self.current_index < self.tokens.len() {
            let token = &self.tokens[self.current_index];
            
            let child = match &token.token_type {
                TokenType::Identifier => {
                    AstNode::identifier(Span::new(self.current_position(), self.current_position()), token.value.clone())
                }
                TokenType::NumberLiteral => {
                    AstNode::number_literal(Span::new(self.current_position(), self.current_position()), token.value.clone())
                }
                TokenType::StringLiteral => {
                    AstNode::string_literal(Span::new(self.current_position(), self.current_position()), token.value.clone())
                }
                _ => {
                    AstNode::new(AstNodeType::Unknown, Span::new(self.current_position(), self.current_position()))
                }
            };
            
            expr.add_child(child);
            self.current_index += 1;
        }
        
        Ok(expr)
    }
    
    /// Проверяет, соответствует ли текущий токен ожидаемому
    fn match_token(&self, expected: &TokenType) -> bool {
        if self.current_index >= self.tokens.len() {
            return false;
        }
        
        std::mem::discriminant(&self.tokens[self.current_index].token_type) == std::mem::discriminant(expected)
    }
    
    /// Парсит простое выражение (идентификатор, литерал, или вызов функции)
    fn parse_simple_expression(&mut self) -> Result<AstNode> {
        let start_pos = self.current_position();
        
        if self.current_index >= self.tokens.len() {
            return Ok(AstNode::identifier(Span::new(start_pos, start_pos), "".to_string()));
        }
        
        let token = &self.tokens[self.current_index];
        
        match &token.token_type {
            TokenType::Identifier => {
                if let Some(identifier) = self.consume_identifier() {
                    // Проверяем, является ли это вызовом функции
                    if self.match_token(&TokenType::LeftParen) {
                        self.current_index += 1; // Пропускаем '('
                        
                        let mut call = AstNode::new(AstNodeType::CallExpression, Span::new(start_pos, self.current_position()));
                        call.value = Some(identifier);
                        
                        // Парсим аргументы
                        while !self.match_token(&TokenType::RightParen) && self.current_index < self.tokens.len() {
                            let arg = self.parse_simple_expression()?;
                            call.add_child(arg);
                            
                            if self.match_token(&TokenType::Comma) {
                                self.current_index += 1;
                            }
                        }
                        
                        if self.match_token(&TokenType::RightParen) {
                            self.current_index += 1;
                        }
                        
                        Ok(call)
                    } else {
                        Ok(AstNode::identifier(Span::new(start_pos, self.current_position()), identifier))
                    }
                } else {
                    Ok(AstNode::identifier(Span::new(start_pos, start_pos), "".to_string()))
                }
            }
            TokenType::StringLiteral => {
                let value = token.value.clone().unwrap_or_default();
                self.current_index += 1;
                Ok(AstNode::string_literal(Span::new(start_pos, self.current_position()), value))
            }
            TokenType::NumberLiteral => {
                let value = token.value.clone().unwrap_or_default();
                self.current_index += 1;
                Ok(AstNode::number_literal(Span::new(start_pos, self.current_position()), value))
            }
            TokenType::True | TokenType::False => {
                let value = token.value.clone().unwrap_or_default();
                self.current_index += 1;
                Ok(AstNode::boolean_literal(Span::new(start_pos, self.current_position()), value))
            }
            _ => {
                // Неизвестный токен - создаем пустой идентификатор
                Ok(AstNode::identifier(Span::new(start_pos, start_pos), "".to_string()))
            }
        }
    }
    
    /// Проверяет, соответствует ли текущий токен одному из ожидаемых
    fn match_any_token(&self, expected: &[TokenType]) -> bool {
        expected.iter().any(|token_type| self.match_token(token_type))
    }
    
    /// Потребляет идентификатор и возвращает его значение
    fn consume_identifier(&mut self) -> Option<String> {
        if self.current_index >= self.tokens.len() {
            return None;
        }
        
        if let TokenType::Identifier = &self.tokens[self.current_index].token_type {
            let result = self.tokens[self.current_index].value.clone();
            self.current_index += 1;
            Some(result)
        } else {
            None
        }
    }
    
    /// Получает текущую позицию
    fn current_position(&self) -> Position {
        if self.current_index < self.tokens.len() {
            self.tokens[self.current_index].position
        } else {
            Position::zero()
        }
    }
    
    /// Добавляет ошибку
    fn add_error(&mut self, message: String, position: Position) {
        let error = AnalysisError::new(message, position, ErrorLevel::Error);
        self.errors.push(error);
    }
    
    /// Добавляет предупреждение
    fn add_warning(&mut self, message: String, position: Position) {
        let warning = AnalysisError::new(message, position, ErrorLevel::Warning);
        self.warnings.push(warning);
    }
    
    /// Подсчитывает количество узлов в AST
    fn count_nodes(&self, node: &AstNode) -> usize {
        1 + node.children.iter().map(|child| self.count_nodes(child)).sum::<usize>()
    }
    
    /// Получает ошибки парсинга
    pub fn get_errors(&self) -> &[AnalysisError] {
        &self.errors
    }
    
    /// Получает предупреждения парсинга
    pub fn get_warnings(&self) -> &[AnalysisError] {
        &self.warnings
    }
}

impl Default for SyntaxAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::lexer::BslLexer;
    
    #[test]
    fn test_parse_empty_program() {
        let mut analyzer = SyntaxAnalyzer::new();
        let result = analyzer.analyze(vec![]).unwrap();
        
        assert_eq!(result.node_type, AstNodeType::Module);
        assert_eq!(result.children.len(), 0);
    }
    
    #[test]
    fn test_parse_simple_procedure() {
        let mut lexer = BslLexer::new();
        let tokens = lexer.tokenize("Процедура Тест() КонецПроцедуры").unwrap();
        
        let mut analyzer = SyntaxAnalyzer::new();
        let result = analyzer.analyze(tokens).unwrap();
        
        assert_eq!(result.node_type, AstNodeType::Module);
        assert_eq!(result.children.len(), 1);
        assert_eq!(result.children[0].node_type, AstNodeType::Procedure);
    }
    
    #[test]
    fn test_parse_variable_declaration() {
        let mut lexer = BslLexer::new();
        let tokens = lexer.tokenize("Перем Переменная;").unwrap();
        
        let mut analyzer = SyntaxAnalyzer::new();
        let result = analyzer.analyze(tokens).unwrap();
        
        assert_eq!(result.children.len(), 1);
        assert_eq!(result.children[0].node_type, AstNodeType::VariableDeclaration);
    }
}
