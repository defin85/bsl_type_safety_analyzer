/*!
# BSL Lexical Analyzer - Complete Implementation

Full-featured lexical analyzer for 1C:Enterprise BSL language.
Ported from Python implementation with all tokens, keywords, and error handling.
*/

use logos::Logos;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;

use super::ast::Position;

/// Reads a BSL file with proper encoding detection and BOM handling
/// Returns the content as UTF-8 string with BOM removed
pub fn read_bsl_file<P: AsRef<std::path::Path>>(path: P) -> Result<String, std::io::Error> {
    let bytes = std::fs::read(path)?;

    // Try to detect encoding and convert to UTF-8
    let content = if bytes.len() >= 2 {
        match (bytes[0], bytes[1]) {
            // UTF-16LE BOM: FF FE
            (0xFF, 0xFE) => {
                let (decoded, _, had_errors) = encoding_rs::UTF_16LE.decode(&bytes);
                if had_errors {
                    tracing::warn!("Errors detected while decoding UTF-16LE file");
                }
                decoded.into_owned()
            }
            // UTF-16BE BOM: FE FF
            (0xFE, 0xFF) => {
                let (decoded, _, had_errors) = encoding_rs::UTF_16BE.decode(&bytes);
                if had_errors {
                    tracing::warn!("Errors detected while decoding UTF-16BE file");
                }
                decoded.into_owned()
            }
            // UTF-8 BOM: EF BB BF or regular UTF-8
            _ => {
                // Try UTF-8 first
                match String::from_utf8(bytes.clone()) {
                    Ok(s) => s,
                    Err(_) => {
                        // Fall back to Windows-1251 (common in Russian 1C installations)
                        tracing::debug!("UTF-8 decoding failed, trying Windows-1251");
                        let (decoded, _, had_errors) = encoding_rs::WINDOWS_1251.decode(&bytes);
                        if had_errors {
                            tracing::warn!("Errors detected while decoding Windows-1251 file");
                        }
                        decoded.into_owned()
                    }
                }
            }
        }
    } else {
        String::from_utf8(bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?
    };

    // Remove BOM if present after encoding conversion
    Ok(strip_bom(&content).to_string())
}

/// Removes BOM (Byte Order Mark) from the beginning of text if present
/// Handles UTF-8, UTF-16LE, and UTF-16BE BOMs commonly found in 1C files
fn strip_bom(input: &str) -> &str {
    // UTF-8 BOM as Unicode character U+FEFF
    if input.starts_with('\u{FEFF}') {
        // U+FEFF is encoded as 3 bytes in UTF-8, so we need to find the char boundary
        let mut char_indices = input.char_indices();
        if let Some((_, _)) = char_indices.next() {
            // Skip the first character (BOM) and return the rest
            if let Some((next_char_start, _)) = char_indices.next() {
                return &input[next_char_start..];
            } else {
                // Input was only BOM
                return "";
            }
        }
    }

    // Check for UTF-8 BOM bytes (EF BB BF)
    let bytes = input.as_bytes();
    if bytes.len() >= 3 && bytes[0] == 0xEF && bytes[1] == 0xBB && bytes[2] == 0xBF {
        return &input[3..];
    }

    // For UTF-16 BOMs, they should be handled at the encoding level
    // before the text reaches this function, but we can detect them
    if bytes.len() >= 2 {
        // UTF-16LE BOM: FF FE
        if bytes[0] == 0xFF && bytes[1] == 0xFE {
            tracing::warn!("UTF-16LE BOM detected - file should be converted to UTF-8 first");
        }
        // UTF-16BE BOM: FE FF
        else if bytes[0] == 0xFE && bytes[1] == 0xFF {
            tracing::warn!("UTF-16BE BOM detected - file should be converted to UTF-8 first");
        }
    }

    input
}

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
    Процедура,
    #[token("КонецПроцедуры", ignore(case))]
    КонецПроцедуры,
    #[token("Функция", ignore(case))]
    Функция,
    #[token("КонецФункции", ignore(case))]
    КонецФункции,
    #[token("Возврат", ignore(case))]
    Return,
    #[token("Экспорт", ignore(case))]
    Export,

    // Keywords - Variables and Types
    #[token("Перем", ignore(case))]
    Перем,
    #[token("Знач", ignore(case))]
    Val,

    // Keywords - Exception Handling
    #[token("Попытка", ignore(case))]
    Попытка,
    #[token("Исключение", ignore(case))]
    Исключение,
    #[token("КонецПопытки", ignore(case))]
    КонецПопытки,
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
            TokenType::Процедура => write!(f, "Процедура"),
            TokenType::КонецПроцедуры => write!(f, "КонецПроцедуры"),
            TokenType::Функция => write!(f, "Функция"),
            TokenType::КонецФункции => write!(f, "КонецФункции"),
            TokenType::Return => write!(f, "Возврат"),
            TokenType::Export => write!(f, "Экспорт"),
            TokenType::Перем => write!(f, "Перем"),
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
            TokenType::Попытка => write!(f, "Попытка"),
            TokenType::Исключение => write!(f, "Исключение"),
            TokenType::КонецПопытки => write!(f, "КонецПопытки"),
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
            "Если",
            "Тогда",
            "ИначеЕсли",
            "Иначе",
            "КонецЕсли",
            "Для",
            "Каждого",
            "Из",
            "По",
            "Цикл",
            "КонецЦикла",
            "Пока",
            "Прервать",
            "Продолжить",
            // Procedures and functions
            "Процедура",
            "КонецПроцедуры",
            "Функция",
            "КонецФункции",
            "Возврат",
            "Экспорт",
            // Variables
            "Перем",
            "Знач",
            // Exception handling
            "Попытка",
            "Исключение",
            "КонецПопытки",
            "ВызватьИсключение",
            // Values
            "Истина",
            "Ложь",
            "Неопределено",
            "Null",
            // Operators
            "И",
            "Или",
            "Не",
            "Новый",
            // Preprocessor
            "&НаКлиенте",
            "&НаСервере",
            "&НаСервереБезКонтекста",
            "&НаКлиентеНаСервере",
            "&Область",
            "&КонецОбласти",
            "&УсловнаяКомпиляция",
            "&КонецЕсли",
        ];

        for keyword in keyword_list {
            keywords.insert(keyword.to_string());
        }

        let mut operators = HashSet::new();
        let operator_list = vec![
            "+", "-", "*", "/", "=", "<", ">", "<=", ">=", "<>", ":=", "(", ")", "[", "]", "{",
            "}", ".", ",", ";", ":", "?", "%",
        ];

        for op in operator_list {
            operators.insert(op.to_string());
        }

        Self {
            keywords,
            operators,
        }
    }

    /// Tokenize BSL source code with full error handling
    pub fn tokenize(&self, input: &str) -> Result<Vec<Token>, String> {
        // Remove BOM if present
        let cleaned_input = strip_bom(input);

        if cleaned_input.trim().is_empty() {
            return Ok(Vec::new());
        }

        let mut tokens = Vec::new();
        let mut lexer = TokenType::lexer(cleaned_input);
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
                }
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
        self.keywords.contains(text)
            || self.keywords.contains(&text.to_lowercase())
            || self.keywords.contains(&text.to_uppercase())
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
        assert_eq!(tokens[0].token_type, TokenType::Процедура);
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

    #[test]
    fn test_strip_bom() {
        // Test UTF-8 BOM (Unicode character U+FEFF)
        let input_with_bom = "\u{FEFF}Процедура Тест()";
        let cleaned = strip_bom(input_with_bom);
        assert_eq!(cleaned, "Процедура Тест()");

        // Test input without BOM
        let input_without_bom = "Процедура Тест()";
        let cleaned = strip_bom(input_without_bom);
        assert_eq!(cleaned, "Процедура Тест()");

        // Test empty string
        let empty = strip_bom("");
        assert_eq!(empty, "");
    }

    #[test]
    fn test_tokenize_with_bom() {
        let lexer = BslLexer::new();

        // Test with UTF-8 BOM
        let input_with_bom = "\u{FEFF}Процедура Тест() КонецПроцедуры";
        let tokens = lexer.tokenize(input_with_bom).unwrap();

        assert!(!tokens.is_empty());
        assert_eq!(tokens[0].token_type, TokenType::Процедура);
        assert_eq!(tokens[1].token_type, TokenType::Identifier);
        assert_eq!(tokens[1].value, "Тест");

        // Ensure BOM doesn't create extra tokens
        let input_without_bom = "Процедура Тест() КонецПроцедуры";
        let tokens_without_bom = lexer.tokenize(input_without_bom).unwrap();

        assert_eq!(tokens.len(), tokens_without_bom.len());
    }
}
