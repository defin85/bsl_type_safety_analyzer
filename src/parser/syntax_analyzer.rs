/*!
# BSL Syntax Analyzer

Simplified syntax analyzer for BSL (1C:Enterprise) language.
Focuses on essential parsing functionality for integration tests.
*/

use crate::parser::ast::{AstNode, AstNodeType, Position, Span};
use crate::parser::lexer::{BslLexer, Token, TokenType};
use anyhow::{anyhow, Result};

/// Simplified BSL syntax analyzer
pub struct SyntaxAnalyzer {
    /// Current token index
    current_index: usize,
    /// Tokens to analyze
    tokens: Vec<Token>,
    /// Verbose mode flag
    verbose: bool,
}

impl SyntaxAnalyzer {
    /// Creates a new syntax analyzer
    pub fn new() -> Self {
        Self {
            current_index: 0,
            tokens: Vec::new(),
            verbose: false,
        }
    }

    /// Sets verbose mode
    pub fn set_verbose(&mut self, verbose: bool) {
        self.verbose = verbose;
    }

    /// Parses BSL code into AST
    pub fn parse(&mut self, code: &str) -> Result<AstNode> {
        let lexer = BslLexer::new();
        let tokens = lexer
            .tokenize(code)
            .map_err(|e| anyhow!("Tokenization error: {}", e))?;

        self.tokens = tokens;
        self.current_index = 0;

        self.parse_module()
    }

    /// Parses a module (main entry point)
    fn parse_module(&mut self) -> Result<AstNode> {
        let mut root = AstNode::module(Span::zero());

        while self.current_index < self.tokens.len() {
            if let Some(node) = self.parse_statement()? {
                root.add_child(node);
            } else {
                self.current_index += 1; // Skip unknown tokens
            }
        }

        Ok(root)
    }

    /// Parses a statement
    fn parse_statement(&mut self) -> Result<Option<AstNode>> {
        if self.current_index >= self.tokens.len() {
            return Ok(None);
        }

        let token = &self.tokens[self.current_index];

        match &token.token_type {
            TokenType::Перем => self.parse_variable_declaration(),
            TokenType::Процедура => self.parse_procedure(),
            TokenType::Функция => self.parse_function(),
            TokenType::If => self.parse_if_statement(),
            TokenType::For => self.parse_for_loop(),
            TokenType::While => self.parse_while_loop(),
            TokenType::Попытка => self.parse_try_statement(),
            TokenType::Identifier => self.parse_assignment_or_call(),
            _ => {
                // Skip unknown tokens
                Ok(None)
            }
        }
    }

    /// Parses variable declaration
    fn parse_variable_declaration(&mut self) -> Result<Option<AstNode>> {
        let start_pos = self.current_position();
        self.current_index += 1; // Skip 'Перем'

        if let Some(var_name) = self.consume_identifier() {
            let var_decl = AstNode::with_value(
                AstNodeType::VariableDeclaration,
                Span::new(start_pos, self.current_position()),
                var_name,
            );

            // Skip semicolon if present
            if self.match_token(&TokenType::Semicolon) {
                self.current_index += 1;
            }

            Ok(Some(var_decl))
        } else {
            Ok(None)
        }
    }

    /// Parses procedure declaration
    fn parse_procedure(&mut self) -> Result<Option<AstNode>> {
        let start_pos = self.current_position();
        self.current_index += 1; // Skip 'Процедура'

        if let Some(proc_name) = self.consume_identifier() {
            let mut procedure =
                AstNode::procedure(Span::new(start_pos, self.current_position()), proc_name);

            // Parse parameters if present
            if self.match_token(&TokenType::LeftParen) {
                self.current_index += 1; // Skip '('

                let mut params = AstNode::new(
                    AstNodeType::ParameterList,
                    Span::new(start_pos, self.current_position()),
                );

                while !self.match_token(&TokenType::RightParen)
                    && self.current_index < self.tokens.len()
                {
                    if let Some(param_name) = self.consume_identifier() {
                        let param = AstNode::parameter(
                            Span::new(start_pos, self.current_position()),
                            param_name,
                        );
                        params.add_child(param);
                    }

                    if self.match_token(&TokenType::Comma) {
                        self.current_index += 1;
                    }
                }

                if self.match_token(&TokenType::RightParen) {
                    self.current_index += 1; // Skip ')'
                }

                procedure.add_child(params);
            }

            // Parse body (skip until КонецПроцедуры)
            let body = self.parse_block_until_keyword(&TokenType::КонецПроцедуры)?;
            procedure.add_child(body);

            if self.match_token(&TokenType::КонецПроцедуры) {
                self.current_index += 1;
            }

            Ok(Some(procedure))
        } else {
            Ok(None)
        }
    }

    /// Parses function declaration
    fn parse_function(&mut self) -> Result<Option<AstNode>> {
        let start_pos = self.current_position();
        self.current_index += 1; // Skip 'Функция'

        if let Some(func_name) = self.consume_identifier() {
            let mut function =
                AstNode::function(Span::new(start_pos, self.current_position()), func_name);

            // Parse parameters if present
            if self.match_token(&TokenType::LeftParen) {
                self.current_index += 1; // Skip '('

                let mut params = AstNode::new(
                    AstNodeType::ParameterList,
                    Span::new(start_pos, self.current_position()),
                );

                while !self.match_token(&TokenType::RightParen)
                    && self.current_index < self.tokens.len()
                {
                    if let Some(param_name) = self.consume_identifier() {
                        let param = AstNode::parameter(
                            Span::new(start_pos, self.current_position()),
                            param_name,
                        );
                        params.add_child(param);
                    }

                    if self.match_token(&TokenType::Comma) {
                        self.current_index += 1;
                    }
                }

                if self.match_token(&TokenType::RightParen) {
                    self.current_index += 1; // Skip ')'
                }

                function.add_child(params);
            }

            // Parse body (skip until КонецФункции)
            let body = self.parse_block_until_keyword(&TokenType::КонецФункции)?;
            function.add_child(body);

            if self.match_token(&TokenType::КонецФункции) {
                self.current_index += 1;
            }

            Ok(Some(function))
        } else {
            Ok(None)
        }
    }

    /// Parses if statement
    fn parse_if_statement(&mut self) -> Result<Option<AstNode>> {
        let start_pos = self.current_position();
        self.current_index += 1; // Skip 'Если'

        let mut if_stmt = AstNode::new(
            AstNodeType::IfStatement,
            Span::new(start_pos, self.current_position()),
        );

        // Skip to 'Тогда'
        while !self.match_token(&TokenType::Then) && self.current_index < self.tokens.len() {
            self.current_index += 1;
        }

        if self.match_token(&TokenType::Then) {
            self.current_index += 1; // Skip 'Тогда'
        }

        // Parse then block
        let then_block = self.parse_block_until_keywords(&[TokenType::Else, TokenType::EndIf])?;
        if_stmt.add_child(then_block);

        // Parse else block if present
        if self.match_token(&TokenType::Else) {
            self.current_index += 1;
            let else_block = self.parse_block_until_keyword(&TokenType::EndIf)?;
            if_stmt.add_child(else_block);
        }

        if self.match_token(&TokenType::EndIf) {
            self.current_index += 1;
        }

        Ok(Some(if_stmt))
    }

    /// Parses for loop
    fn parse_for_loop(&mut self) -> Result<Option<AstNode>> {
        let start_pos = self.current_position();
        self.current_index += 1; // Skip 'Для'

        let mut for_loop = AstNode::new(
            AstNodeType::ForLoop,
            Span::new(start_pos, self.current_position()),
        );

        // Skip to 'Цикл'
        while !self.match_token(&TokenType::Do) && self.current_index < self.tokens.len() {
            self.current_index += 1;
        }

        if self.match_token(&TokenType::Do) {
            self.current_index += 1; // Skip 'Цикл'
        }

        // Parse statements inside the loop until 'КонецЦикла'
        let block = self.parse_block_until_keyword(&TokenType::EndDo)?;
        for_loop.add_child(block);

        if self.match_token(&TokenType::EndDo) {
            self.current_index += 1;
        }

        Ok(Some(for_loop))
    }

    /// Parses while loop
    fn parse_while_loop(&mut self) -> Result<Option<AstNode>> {
        let start_pos = self.current_position();
        self.current_index += 1; // Skip 'Пока'

        let mut while_loop = AstNode::new(
            AstNodeType::WhileLoop,
            Span::new(start_pos, self.current_position()),
        );

        // Skip to 'Цикл'
        while !self.match_token(&TokenType::Do) && self.current_index < self.tokens.len() {
            self.current_index += 1;
        }

        if self.match_token(&TokenType::Do) {
            self.current_index += 1; // Skip 'Цикл'
        }

        // Parse statements inside the loop until 'КонецЦикла'
        let block = self.parse_block_until_keyword(&TokenType::EndDo)?;
        while_loop.add_child(block);

        if self.match_token(&TokenType::EndDo) {
            self.current_index += 1;
        }

        Ok(Some(while_loop))
    }

    /// Parses try statement
    fn parse_try_statement(&mut self) -> Result<Option<AstNode>> {
        let start_pos = self.current_position();
        self.current_index += 1; // Skip 'Попытка'

        let mut try_stmt = AstNode::new(
            AstNodeType::TryStatement,
            Span::new(start_pos, self.current_position()),
        );

        // Parse try block until 'Исключение' or 'КонецПопытки'
        let try_block =
            self.parse_block_until_keywords(&[TokenType::Исключение, TokenType::КонецПопытки])?;
        try_stmt.add_child(try_block);

        if self.match_token(&TokenType::Исключение) {
            self.current_index += 1; // Skip 'Исключение'

            // Parse exception block until 'КонецПопытки'
            let exception_block = self.parse_block_until_keyword(&TokenType::КонецПопытки)?;
            try_stmt.add_child(exception_block);
        }

        if self.match_token(&TokenType::КонецПопытки) {
            self.current_index += 1; // Skip 'КонецПопытки'
        }

        Ok(Some(try_stmt))
    }

    /// Parses assignment or call
    fn parse_assignment_or_call(&mut self) -> Result<Option<AstNode>> {
        let start_pos = self.current_position();

        if let Some(identifier) = self.consume_identifier() {
            if self.match_token(&TokenType::Equal) {
                // Assignment
                self.current_index += 1; // Skip '='

                let mut assignment = AstNode::new(
                    AstNodeType::Assignment,
                    Span::new(start_pos, self.current_position()),
                );
                let left_side =
                    AstNode::identifier(Span::new(start_pos, self.current_position()), identifier);
                assignment.add_child(left_side);

                // Skip to semicolon or end of line
                while !self.match_token(&TokenType::Semicolon)
                    && self.current_index < self.tokens.len()
                    && !self.is_statement_terminator()
                {
                    self.current_index += 1;
                }

                if self.match_token(&TokenType::Semicolon) {
                    self.current_index += 1;
                }

                Ok(Some(assignment))
            } else if self.match_token(&TokenType::LeftParen) {
                // Function call
                self.current_index += 1; // Skip '('

                let mut call = AstNode::new(
                    AstNodeType::CallExpression,
                    Span::new(start_pos, self.current_position()),
                );
                call.value = Some(identifier);

                // Skip arguments until ')'
                while !self.match_token(&TokenType::RightParen)
                    && self.current_index < self.tokens.len()
                {
                    self.current_index += 1;
                }

                if self.match_token(&TokenType::RightParen) {
                    self.current_index += 1;
                }

                if self.match_token(&TokenType::Semicolon) {
                    self.current_index += 1;
                }

                Ok(Some(call))
            } else if self.match_token(&TokenType::Dot) {
                // Method call
                self.current_index += 1; // Skip '.'

                if let Some(method_name) = self.consume_identifier() {
                    if self.match_token(&TokenType::LeftParen) {
                        self.current_index += 1; // Skip '('

                        let mut call = AstNode::new(
                            AstNodeType::CallExpression,
                            Span::new(start_pos, self.current_position()),
                        );
                        call.value = Some(format!("{}.{}", identifier, method_name));

                        // Skip arguments until ')'
                        while !self.match_token(&TokenType::RightParen)
                            && self.current_index < self.tokens.len()
                        {
                            self.current_index += 1;
                        }

                        if self.match_token(&TokenType::RightParen) {
                            self.current_index += 1;
                        }

                        if self.match_token(&TokenType::Semicolon) {
                            self.current_index += 1;
                        }

                        Ok(Some(call))
                    } else {
                        Ok(Some(AstNode::identifier(
                            Span::new(start_pos, self.current_position()),
                            identifier,
                        )))
                    }
                } else {
                    Ok(Some(AstNode::identifier(
                        Span::new(start_pos, self.current_position()),
                        identifier,
                    )))
                }
            } else {
                // Just identifier
                Ok(Some(AstNode::identifier(
                    Span::new(start_pos, self.current_position()),
                    identifier,
                )))
            }
        } else {
            Ok(None)
        }
    }

    /// Parses a block until specified keyword
    fn parse_block_until_keyword(&mut self, end_token: &TokenType) -> Result<AstNode> {
        let start_pos = self.current_position();
        let mut block = AstNode::new(
            AstNodeType::Block,
            Span::new(start_pos, self.current_position()),
        );

        while !self.match_token(end_token) && self.current_index < self.tokens.len() {
            if let Some(stmt) = self.parse_statement()? {
                block.add_child(stmt);
            } else {
                self.current_index += 1;
            }
        }

        Ok(block)
    }

    /// Parses a block until one of specified keywords
    fn parse_block_until_keywords(&mut self, end_tokens: &[TokenType]) -> Result<AstNode> {
        let start_pos = self.current_position();
        let mut block = AstNode::new(
            AstNodeType::Block,
            Span::new(start_pos, self.current_position()),
        );

        while !self.match_any_token(end_tokens) && self.current_index < self.tokens.len() {
            if let Some(stmt) = self.parse_statement()? {
                block.add_child(stmt);
            } else {
                self.current_index += 1;
            }
        }

        Ok(block)
    }

    /// Helper methods
    fn current_position(&self) -> Position {
        if self.current_index < self.tokens.len() {
            self.tokens[self.current_index].position
        } else {
            Position::zero()
        }
    }

    fn consume_identifier(&mut self) -> Option<String> {
        if self.current_index < self.tokens.len()
            && self.tokens[self.current_index].token_type == TokenType::Identifier
        {
            let identifier = self.tokens[self.current_index].value.clone();
            self.current_index += 1;
            Some(identifier)
        } else {
            None
        }
    }

    fn match_token(&self, expected: &TokenType) -> bool {
        if self.current_index >= self.tokens.len() {
            return false;
        }

        std::mem::discriminant(&self.tokens[self.current_index].token_type)
            == std::mem::discriminant(expected)
    }

    fn match_any_token(&self, tokens: &[TokenType]) -> bool {
        tokens.iter().any(|token| self.match_token(token))
    }

    fn is_statement_terminator(&self) -> bool {
        if self.current_index >= self.tokens.len() {
            return true;
        }

        matches!(
            self.tokens[self.current_index].token_type,
            TokenType::Перем
                | TokenType::Процедура
                | TokenType::Функция
                | TokenType::If
                | TokenType::For
                | TokenType::While
                | TokenType::КонецПроцедуры
                | TokenType::КонецФункции
                | TokenType::EndIf
                | TokenType::EndDo
        )
    }
}

impl Default for SyntaxAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
