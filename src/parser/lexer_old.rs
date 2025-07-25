/*!
# BSL Lexer

Fast lexical analyzer for BSL using the logos crate.
*/

use logos::Logos;
use serde::{Deserialize, Serialize};
use std::fmt;

use super::ast::Position;

/// BSL Token types
#[derive(Logos, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenType {
    // Keywords - Declarations
    #[token("Процедура", ignore(case))]
    Procedure,
    #[token("КонецПроцедуры", ignore(case))]
    EndProcedure,
    #[token("Функция", ignore(case))]
    Function,
    #[token("КонецФункции", ignore(case))]
    EndFunction,
    #[token("Экспорт", ignore(case))]
    Export,
    #[token("Перем", ignore(case))]
    Var,
    
    // Keywords - Control Flow
    #[token("Если", ignore(case))]
    If,
    #[token("Тогда", ignore(case))]
    Then,
    #[token("Иначе", ignore(case))]
    Else,
    #[token("ИначеЕсли", ignore(case))]
    ElseIf,
    #[token("КонецЕсли", ignore(case))]
    EndIf,
    
    #[token("Для", ignore(case))]
    For,
    #[token("Каждого", ignore(case))]
    Each,
    #[token("Из", ignore(case))]
    In,
    #[token("По", ignore(case))]
    To,
    #[token("Цикл", ignore(case))]
    Do,
    #[token("КонецЦикла", ignore(case))]
    EndDo,
    
    #[token("Пока", ignore(case))]
    While,
    
    #[token("Попытка", ignore(case))]
    Try,
    #[token("Исключение", ignore(case))]
    Except,
    #[token("КонецПопытки", ignore(case))]
    EndTry,
    
    #[token("Возврат", ignore(case))]
    Return,
    #[token("Прервать", ignore(case))]
    Break,
    #[token("Продолжить", ignore(case))]
    Continue,
    
    // Keywords - Values
    #[token("Истина", ignore(case))]
    True,
    #[token("Ложь", ignore(case))]
    False,
    #[token("Неопределено", ignore(case))]
    Undefined,
    #[token("Null", ignore(case))]
    Null,
    
    // Keywords - Operators  
    #[token("И", ignore(case))]
    And,
    #[token("Или", ignore(case))]
    Or,
    #[token("Не", ignore(case))]
    Not,
    #[token("Новый", ignore(case))]
    New,
    
    // Operators
    #[token("=")]
    Assign,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Multiply,
    #[token("/")]
    Divide,
    #[token("%")]
    Modulo,
    
    #[token("==")]
    Equal,
    #[token("<>")]
    NotEqual,
    #[token("!=")]
    NotEqualAlt,
    #[token("<")]
    Less,
    #[token("<=")]
    LessEqual,
    #[token(">")]
    Greater,
    #[token(">=")]
    GreaterEqual,
    
    // Delimiters
    #[token("(")]
    LeftParen,
    #[token(")")]
    RightParen,
    #[token("[")]
    LeftBracket,
    #[token("]")]
    RightBracket,
    #[token(".")]
    Dot,
    #[token(",")]
    Comma,
    #[token(";")]
    Semicolon,
    #[token(":")]
    Colon,
    
    // Literals
    #[regex(r#""([^"\\]|\\.)*""#)]
    StringLiteral,
    
    #[regex(r#"\|([^|\\]|\\.)*\|"#)]
    DateLiteral,
    
    #[regex(r"\d+(\.\d+)?")]
    NumberLiteral,
    
    // Identifiers  
    #[regex(r"[А-Яа-яA-Za-z_][А-Яа-яA-Za-z0-9_]*", priority = 1)]
    Identifier,
    
    // Comments
    #[regex(r"//[^\r\n]*")]
    LineComment,
    
    // Whitespace and newlines
    #[regex(r"[ \t]+")]
    Whitespace,
    
    #[regex(r"\r\n|\r|\n")]
    Newline,
    
    // End of file
    Eof,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenType::Procedure => write!(f, "Процедура"),
            TokenType::EndProcedure => write!(f, "КонецПроцедуры"),
            TokenType::Function => write!(f, "Функция"),
            TokenType::EndFunction => write!(f, "КонецФункции"),
            TokenType::Export => write!(f, "Экспорт"),
            TokenType::Var => write!(f, "Перем"),
            TokenType::If => write!(f, "Если"),
            TokenType::Then => write!(f, "Тогда"),
            TokenType::Else => write!(f, "Иначе"),
            TokenType::ElseIf => write!(f, "ИначеЕсли"),
            TokenType::EndIf => write!(f, "КонецЕсли"),
            TokenType::For => write!(f, "Для"),
            TokenType::Each => write!(f, "Каждого"),
            TokenType::In => write!(f, "Из"),
            TokenType::To => write!(f, "По"),
            TokenType::Do => write!(f, "Цикл"),
            TokenType::EndDo => write!(f, "КонецЦикла"),
            TokenType::While => write!(f, "Пока"),
            TokenType::Try => write!(f, "Попытка"),
            TokenType::Except => write!(f, "Исключение"),
            TokenType::EndTry => write!(f, "КонецПопытки"),
            TokenType::Return => write!(f, "Возврат"),
            TokenType::Break => write!(f, "Прервать"),
            TokenType::Continue => write!(f, "Продолжить"),
            TokenType::True => write!(f, "Истина"),
            TokenType::False => write!(f, "Ложь"),
            TokenType::Undefined => write!(f, "Неопределено"),
            TokenType::Null => write!(f, "Null"),
            TokenType::And => write!(f, "И"),
            TokenType::Or => write!(f, "Или"),
            TokenType::Not => write!(f, "Не"),
            TokenType::New => write!(f, "Новый"),
            TokenType::Assign => write!(f, "="),
            TokenType::Plus => write!(f, "+"),
            TokenType::Minus => write!(f, "-"),
            TokenType::Multiply => write!(f, "*"),
            TokenType::Divide => write!(f, "/"),
            TokenType::Modulo => write!(f, "%"),
            TokenType::Equal => write!(f, "=="),
            TokenType::NotEqual => write!(f, "<>"),
            TokenType::NotEqualAlt => write!(f, "!="),
            TokenType::Less => write!(f, "<"),
            TokenType::LessEqual => write!(f, "<="),
            TokenType::Greater => write!(f, ">"),
            TokenType::GreaterEqual => write!(f, ">="),
            TokenType::LeftParen => write!(f, "("),
            TokenType::RightParen => write!(f, ")"),
            TokenType::LeftBracket => write!(f, "["),
            TokenType::RightBracket => write!(f, "]"),
            TokenType::Dot => write!(f, "."),
            TokenType::Comma => write!(f, ","),
            TokenType::Semicolon => write!(f, ";"),
            TokenType::Colon => write!(f, ":"),
            TokenType::StringLiteral => write!(f, "STRING"),
            TokenType::DateLiteral => write!(f, "DATE"),
            TokenType::NumberLiteral => write!(f, "NUMBER"),
            TokenType::Identifier => write!(f, "IDENTIFIER"),
            TokenType::LineComment => write!(f, "COMMENT"),
            TokenType::Whitespace => write!(f, "WHITESPACE"),
            TokenType::Newline => write!(f, "NEWLINE"),
            TokenType::Eof => write!(f, "EOF"),
        }
    }
}

/// Token with position information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub position: Position,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, position: Position) -> Self {
        Self {
            token_type,
            lexeme,
            position,
        }
    }
    
    pub fn is_keyword(&self) -> bool {
        matches!(
            self.token_type,
            TokenType::Procedure
                | TokenType::EndProcedure
                | TokenType::Function
                | TokenType::EndFunction
                | TokenType::Export
                | TokenType::Var
                | TokenType::If
                | TokenType::Then
                | TokenType::Else
                | TokenType::ElseIf
                | TokenType::EndIf
                | TokenType::For
                | TokenType::Each
                | TokenType::In
                | TokenType::To
                | TokenType::Do
                | TokenType::EndDo
                | TokenType::While
                | TokenType::Try
                | TokenType::Except
                | TokenType::EndTry
                | TokenType::Return
                | TokenType::Break
                | TokenType::Continue
                | TokenType::True
                | TokenType::False
                | TokenType::Undefined
                | TokenType::Null
                | TokenType::And
                | TokenType::Or
                | TokenType::Not
                | TokenType::New
        )
    }
    
    pub fn is_literal(&self) -> bool {
        matches!(
            self.token_type,
            TokenType::StringLiteral
                | TokenType::DateLiteral
                | TokenType::NumberLiteral
                | TokenType::True
                | TokenType::False
                | TokenType::Undefined
                | TokenType::Null
        )
    }
    
    pub fn is_operator(&self) -> bool {
        matches!(
            self.token_type,
            TokenType::Assign
                | TokenType::Plus
                | TokenType::Minus
                | TokenType::Multiply
                | TokenType::Divide
                | TokenType::Modulo
                | TokenType::Equal
                | TokenType::NotEqual
                | TokenType::NotEqualAlt
                | TokenType::Less
                | TokenType::LessEqual
                | TokenType::Greater
                | TokenType::GreaterEqual
                | TokenType::And
                | TokenType::Or
                | TokenType::Not
        )
    }
    
    pub fn is_whitespace(&self) -> bool {
        matches!(
            self.token_type,
            TokenType::Whitespace | TokenType::Newline | TokenType::LineComment
        )
    }
}

/// BSL Lexer
pub struct BslLexer {
    skip_whitespace: bool,
    skip_comments: bool,
}

impl BslLexer {
    pub fn new() -> Self {
        Self {
            skip_whitespace: true,
            skip_comments: true,
        }
    }
    
    pub fn with_whitespace(mut self, skip: bool) -> Self {
        self.skip_whitespace = skip;
        self
    }
    
    pub fn with_comments(mut self, skip: bool) -> Self {
        self.skip_comments = skip;
        self
    }
    
    /// Tokenizes input text
    pub fn tokenize(&self, input: &str) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        let mut lexer = TokenType::lexer(input);
        
        let mut line = 1;
        let mut column = 1;
        let mut offset = 0;
        
        while let Some(token_result) = lexer.next() {
            let token_type = token_result.map_err(|_| "Lexical error")?;
            let lexeme = lexer.slice().to_string();
            let position = Position::new(line, column, offset);
            
            // Update position tracking
            for ch in lexeme.chars() {
                if ch == '\n' {
                    line += 1;
                    column = 1;
                } else {
                    column += 1;
                }
                offset += ch.len_utf8();
            }
            
            // Skip whitespace and comments if configured
            if (self.skip_whitespace && matches!(token_type, TokenType::Whitespace | TokenType::Newline))
                || (self.skip_comments && token_type == TokenType::LineComment)
            {
                continue;
            }
            
            tokens.push(Token::new(token_type, lexeme, position));
        }
        
        // Add EOF token
        tokens.push(Token::new(
            TokenType::Eof,
            String::new(),
            Position::new(line, column, offset),
        ));
        
        Ok(tokens)
    }
    
    /// Tokenizes only identifiers and keywords (for fast analysis)
    pub fn tokenize_identifiers(&self, input: &str) -> Result<Vec<Token>, String> {
        let tokens = self.tokenize(input)?;
        
        Ok(tokens
            .into_iter()
            .filter(|token| {
                matches!(
                    token.token_type,
                    TokenType::Identifier
                        | TokenType::Procedure
                        | TokenType::Function
                        | TokenType::Export
                        | TokenType::Dot
                        | TokenType::LeftParen
                        | TokenType::RightParen
                )
            })
            .collect())
    }
}

impl Default for BslLexer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tokenize_procedure() {
        let lexer = BslLexer::new();
        let input = "Процедура ТестоваяПроцедура() Экспорт";
        let tokens = lexer.tokenize(input).unwrap();
        
        assert_eq!(tokens.len(), 6); // + EOF
        assert_eq!(tokens[0].token_type, TokenType::Procedure);
        assert_eq!(tokens[1].token_type, TokenType::Identifier);
        assert_eq!(tokens[1].lexeme, "ТестоваяПроцедура");
        assert_eq!(tokens[2].token_type, TokenType::LeftParen);
        assert_eq!(tokens[3].token_type, TokenType::RightParen);
        assert_eq!(tokens[4].token_type, TokenType::Export);
        assert_eq!(tokens[5].token_type, TokenType::Eof);
    }
    
    #[test]
    fn test_tokenize_expressions() {
        let lexer = BslLexer::new();
        let input = r#"Результат = Число1 + Число2 * 3.14;"#;
        let tokens = lexer.tokenize(input).unwrap();
        
        // Should contain identifiers, operators, and number
        assert!(tokens.iter().any(|t| t.token_type == TokenType::Identifier));
        assert!(tokens.iter().any(|t| t.token_type == TokenType::Assign));
        assert!(tokens.iter().any(|t| t.token_type == TokenType::Plus));
        assert!(tokens.iter().any(|t| t.token_type == TokenType::Multiply));
        assert!(tokens.iter().any(|t| t.token_type == TokenType::NumberLiteral));
    }
    
    #[test]
    fn test_tokenize_string_literal() {
        let lexer = BslLexer::new();
        let input = r#"Сообщить("Привет, мир!");"#;
        let tokens = lexer.tokenize(input).unwrap();
        
        let string_token = tokens.iter()
            .find(|t| t.token_type == TokenType::StringLiteral)
            .unwrap();
        assert_eq!(string_token.lexeme, r#""Привет, мир!""#);
    }
    
    #[test]
    fn test_position_tracking() {
        let lexer = BslLexer::new().with_whitespace(false);
        let input = "Процедура\nТест()";
        let tokens = lexer.tokenize(input).unwrap();
        
        assert_eq!(tokens[0].position.line, 1);
        assert_eq!(tokens[0].position.column, 1);
        
        // After newline
        assert_eq!(tokens[2].position.line, 2);
        assert_eq!(tokens[2].position.column, 1);
    }
}
