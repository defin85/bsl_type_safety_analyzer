/*!
# BSL Lexical Analyzer - Complete Implementation

Full-featured lexical analyzer for 1C:Enterprise BSL language.
Ported from Python implementation with all tokens, keywords, and error handling.
*/

use logos::Logos;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::collections::HashSet;

use super::ast::Position;

/// Complete BSL token types with all language constructs
#[derive(Logos, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenType {
    // Keywords - Control Flow
    #[token("Если", ignore(case))]
    If,
    #[token("Тогда", ignore(case))]
    Then,
    #[token("ИначеЕсли", ignore(case))]
    ElseIf,
    #[token("Иначе", ignore(case))]
    Else,
    #[token("КонецЕсли", ignore(case))]
    EndIf,
    
    // Keywords - Loops
    #[token("Для", ignore(case))]
    For,
    #[token("Каждого", ignore(case))]
    Each,
    #[token("Из", ignore(case))]
    From,
    #[token("По", ignore(case))]
    To,
    #[token("Цикл", ignore(case))]
    Do,
    #[token("КонецЦикла", ignore(case))]
    EndDo,
    #[token("Пока", ignore(case))]
    While,
    #[token("Прервать", ignore(case))]
    Break,
    #[token("Продолжить", ignore(case))]
    Continue,
    
    // Keywords - Procedures and Functions
    #[token("Процедура", ignore(case))]
    Procedure,
    #[token("КонецПроцедуры", ignore(case))]
    EndProcedure,
    #[token("Функция", ignore(case))]
    Function,
    #[token("КонецФункции", ignore(case))]
    EndFunction,
    #[token("Возврат", ignore(case))]
    Return,
    #[token("Экспорт", ignore(case))]
    Export,
    
    // Keywords - Variables and Types
    #[token("Перем", ignore(case))]
    Var,
    #[token("Знач", ignore(case))]
    Val,
    
    // Keywords - Exception Handling
    #[token("Попытка", ignore(case))]
    Try,
    #[token("Исключение", ignore(case))]
    Except,
    #[token("КонецПопытки", ignore(case))]
    EndTry,
    #[token("ВызватьИсключение", ignore(case))]
    Raise,
    
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
    
    // Arithmetic operators
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
    
    // Comparison operators
    #[token("=")]
    Equal,
    #[token("<>")]
    NotEqual,
    #[token("<")]
    Less,
    #[token(">")]
    Greater,
    #[token("<=")]
    LessEqual,
    #[token(">=")]
    GreaterEqual,
    
    // Assignment
    #[token(":=")]
    Assign,
    
    // Delimiters
    #[token("(")]
    LeftParen,
    #[token(")")]
    RightParen,
    #[token("[")]
    LeftBracket,
    #[token("]")]
    RightBracket,
    #[token("{")]
    LeftBrace,
    #[token("}")]
    RightBrace,
    
    // Punctuation
    #[token(".")]
    Dot,
    #[token(",")]
    Comma,
    #[token(";")]
    Semicolon,
    #[token(":")]
    Colon,
    #[token("?")]
    Question,
    
    // Preprocessor directives
    #[token("&НаКлиенте", ignore(case))]
    AtClient,
    #[token("&НаСервере", ignore(case))]
    AtServer,
    #[token("&НаСервереБезКонтекста", ignore(case))]
    AtServerNoContext,
    #[token("&НаКлиентеНаСервере", ignore(case))]
    AtClientAtServer,
    #[token("&Область", ignore(case))]
    Region,
    #[token("&КонецОбласти", ignore(case))]
    EndRegion,
    #[token("&УсловнаяКомпиляция", ignore(case))]
    IfDef,
    #[token("&КонецЕсли", ignore(case))]
    EndIfDef,
    
    // Literals
    #[regex(r#""([^"\\]|\\.)*""#)]
    StringLiteral,
    #[regex(r"'([^'\\]|\\.)*'")]
    StringLiteralSingle,
    #[regex(r"'[0-9]{8}([0-9]{6})?'")]
    DateLiteral,
    #[regex(r"\d+(\.\d+)?")]
    NumberLiteral,
    
    // Identifiers (lower priority to avoid conflicts with keywords)
    #[regex(r"[А-Яа-яA-Za-z_][А-Яа-яA-Za-z0-9_]*", priority = 1)]
    Identifier,
    
    // Comments
    #[regex(r"//[^\r\n]*")]
    LineComment,
    
    // Whitespace and newlines
    #[regex(r"[ \t\f]+", logos::skip)]
    Whitespace,
    #[regex(r"[\r\n]+")]
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
            TokenType::Return => write!(f, "Возврат"),
            TokenType::Export => write!(f, "Экспорт"),
            TokenType::Var => write!(f, "Перем"),
            TokenType::Val => write!(f, "Знач"),
            TokenType::If => write!(f, "Если"),
            TokenType::Then => write!(f, "Тогда"),
            TokenType::ElseIf => write!(f, "ИначеЕсли"),
            TokenType::Else => write!(f, "Иначе"),
            TokenType::EndIf => write!(f, "КонецЕсли"),
            TokenType::For => write!(f, "Для"),
            TokenType::Each => write!(f, "Каждого"),
            TokenType::From => write!(f, "Из"),
            TokenType::To => write!(f, "По"),
            TokenType::Do => write!(f, "Цикл"),
            TokenType::EndDo => write!(f, "КонецЦикла"),
            TokenType::While => write!(f, "Пока"),
            TokenType::Break => write!(f, "Прервать"),
            TokenType::Continue => write!(f, "Продолжить"),
            TokenType::Try => write!(f, "Попытка"),
            TokenType::Except => write!(f, "Исключение"),
            TokenType::EndTry => write!(f, "КонецПопытки"),
            TokenType::Raise => write!(f, "ВызватьИсключение"),
            TokenType::True => write!(f, "Истина"),
            TokenType::False => write!(f, "Ложь"),
            TokenType::Undefined => write!(f, "Неопределено"),
            TokenType::Null => write!(f, "Null"),
            TokenType::And => write!(f, "И"),
            TokenType::Or => write!(f, "Или"),
            TokenType::Not => write!(f, "Не"),
            TokenType::New => write!(f, "Новый"),
            TokenType::Plus => write!(f, "+"),
            TokenType::Minus => write!(f, "-"),
            TokenType::Multiply => write!(f, "*"),
            TokenType::Divide => write!(f, "/"),
            TokenType::Modulo => write!(f, "%"),
            TokenType::Equal => write!(f, "="),
            TokenType::NotEqual => write!(f, "<>"),
            TokenType::Less => write!(f, "<"),
            TokenType::Greater => write!(f, ">"),
            TokenType::LessEqual => write!(f, "<="),
            TokenType::GreaterEqual => write!(f, ">="),
            TokenType::Assign => write!(f, ":="),
            TokenType::LeftParen => write!(f, "("),
            TokenType::RightParen => write!(f, ")"),
            TokenType::LeftBracket => write!(f, "["),
            TokenType::RightBracket => write!(f, "]"),
            TokenType::LeftBrace => write!(f, "{{"),
            TokenType::RightBrace => write!(f, "}}"),
            TokenType::Dot => write!(f, "."),
            TokenType::Comma => write!(f, ","),
            TokenType::Semicolon => write!(f, ";"),
            TokenType::Colon => write!(f, ":"),
            TokenType::Question => write!(f, "?"),
            TokenType::AtClient => write!(f, "&НаКлиенте"),
            TokenType::AtServer => write!(f, "&НаСервере"),
            TokenType::AtServerNoContext => write!(f, "&НаСервереБезКонтекста"),
            TokenType::AtClientAtServer => write!(f, "&НаКлиентеНаСервере"),
            TokenType::Region => write!(f, "&Область"),
            TokenType::EndRegion => write!(f, "&КонецОбласти"),
            TokenType::IfDef => write!(f, "&УсловнаяКомпиляция"),
            TokenType::EndIfDef => write!(f, "&КонецЕсли"),
            TokenType::StringLiteral => write!(f, "STRING"),
            TokenType::StringLiteralSingle => write!(f, "STRING"),
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
    pub value: String,
    pub position: Position,
    pub length: usize,
}

impl Token {
    pub fn new(token_type: TokenType, value: String, position: Position) -> Self {
        let length = value.len();
        Self {
            token_type,
            value,
            position,
            length,
        }
    }
}

/// BSL Lexer with comprehensive error handling and position tracking
pub struct BslLexer {
    keywords: HashSet<String>,
    operators: HashSet<String>,
}

impl BslLexer {
    pub fn new() -> Self {
        let mut keywords = HashSet::new();
        
        // All BSL keywords from Python implementation
        let keyword_list = vec![
            // Control flow
            "Если", "Тогда", "ИначеЕсли", "Иначе", "КонецЕсли",
            "Для", "Каждого", "Из", "По", "Цикл", "КонецЦикла",
            "Пока", "Прервать", "Продолжить",
            
            // Procedures and functions
            "Процедура", "КонецПроцедуры", "Функция", "КонецФункции",
            "Возврат", "Экспорт",
            
            // Variables
            "Перем", "Знач",
            
            // Exception handling
            "Попытка", "Исключение", "КонецПопытки", "ВызватьИсключение",
            
            // Values
            "Истина", "Ложь", "Неопределено", "Null",
            
            // Operators
            "И", "Или", "Не", "Новый",
            
            // Preprocessor
            "&НаКлиенте", "&НаСервере", "&НаСервереБезКонтекста", 
            "&НаКлиентеНаСервере", "&Область", "&КонецОбласти",
            "&УсловнаяКомпиляция", "&КонецЕсли",
        ];
        
        for keyword in keyword_list {
            keywords.insert(keyword.to_string());
        }
        
        let mut operators = HashSet::new();
        let operator_list = vec![
            "+", "-", "*", "/", "=", "<", ">", "<=", ">=", "<>", ":=",
            "(", ")", "[", "]", "{", "}", ".", ",", ";", ":", "?", "%"
        ];
        
        for op in operator_list {
            operators.insert(op.to_string());
        }
        
        Self { keywords, operators }
    }
    
    /// Tokenize BSL source code with full error handling
    pub fn tokenize(&self, input: &str) -> Result<Vec<Token>, String> {
        if input.trim().is_empty() {
            return Ok(Vec::new());
        }
        
        let mut tokens = Vec::new();
        let mut lexer = TokenType::lexer(input);
        let mut line = 1;
        let mut column = 1;
        let mut offset = 0;
        
        while let Some(result) = lexer.next() {
            let token_text = lexer.slice();
            let token_len = token_text.len();
            
            match result {
                Ok(token_type) => {
                    // Skip whitespace but track position
                    if matches!(token_type, TokenType::Whitespace) {
                        column += token_len;
                        offset += token_len;
                        continue;
                    }
                    
                    let position = Position::new(line, column, offset);
                    let token = Token::new(token_type, token_text.to_string(), position);
                    
                    // Handle newlines for position tracking
                    if matches!(token_type, TokenType::Newline) {
                        line += token_text.matches('\n').count();
                        column = 1;
                    } else {
                        column += token_len;
                    }
                    offset += token_len;
                    
                    tokens.push(token);
                },
                Err(_) => {
                    return Err(format!(
                        "Lexical error at line {}, column {}: unexpected character '{}'",
                        line, column, token_text
                    ));
                }
            }
        }
        
        // Add EOF token
        let position = Position::new(line, column, offset);
        tokens.push(Token::new(TokenType::Eof, String::new(), position));
        
        Ok(tokens)
    }
    
    /// Check if a string is a BSL keyword
    pub fn is_keyword(&self, text: &str) -> bool {
        self.keywords.contains(&text.to_string()) ||
        self.keywords.contains(&text.to_lowercase()) ||
        self.keywords.contains(&text.to_uppercase())
    }
    
    /// Check if a string is a BSL operator
    pub fn is_operator(&self, text: &str) -> bool {
        self.operators.contains(text)
    }
    
    /// Get token statistics for analysis
    pub fn get_token_stats(&self, tokens: &[Token]) -> TokenStats {
        let mut stats = TokenStats::default();
        
        for token in tokens {
            match token.token_type {
                TokenType::LineComment => stats.comments += 1,
                TokenType::StringLiteral | TokenType::StringLiteralSingle => stats.strings += 1,
                TokenType::NumberLiteral => stats.numbers += 1,
                TokenType::Identifier => stats.identifiers += 1,
                _ if self.is_keyword(&token.value) => stats.keywords += 1,
                _ if self.is_operator(&token.value) => stats.operators += 1,
                _ => stats.other += 1,
            }
        }
        
        stats.total = tokens.len();
        stats
    }
}

impl Default for BslLexer {
    fn default() -> Self {
        Self::new()
    }
}

/// Token statistics for analysis and reporting
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TokenStats {
    pub total: usize,
    pub keywords: usize,
    pub identifiers: usize,
    pub operators: usize,
    pub strings: usize,
    pub numbers: usize,
    pub comments: usize,
    pub other: usize,
}

impl fmt::Display for TokenStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Token Statistics:")?;
        writeln!(f, "  Total tokens: {}", self.total)?;
        writeln!(f, "  Keywords: {}", self.keywords)?;
        writeln!(f, "  Identifiers: {}", self.identifiers)?;
        writeln!(f, "  Operators: {}", self.operators)?;
        writeln!(f, "  Strings: {}", self.strings)?;
        writeln!(f, "  Numbers: {}", self.numbers)?;
        writeln!(f, "  Comments: {}", self.comments)?;
        writeln!(f, "  Other: {}", self.other)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokenization() {
        let lexer = BslLexer::new();
        let input = "Процедура Тест() Сообщить(\"Привет\"); КонецПроцедуры";
        
        let tokens = lexer.tokenize(input).unwrap();
        assert!(tokens.len() > 5);
        assert_eq!(tokens[0].token_type, TokenType::Procedure);
        assert_eq!(tokens[1].token_type, TokenType::Identifier);
    }
    
    #[test]
    fn test_keywords() {
        let lexer = BslLexer::new();
        assert!(lexer.is_keyword("Если"));
        assert!(lexer.is_keyword("Процедура"));
        assert!(!lexer.is_keyword("МояПеременная"));
    }
    
    #[test]
    fn test_empty_input() {
        let lexer = BslLexer::new();
        let tokens = lexer.tokenize("").unwrap();
        assert!(tokens.is_empty());
    }
}
