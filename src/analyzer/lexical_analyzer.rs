/*!
# Lexical Analyzer for BSL

High-performance lexical analyzer for BSL (1C:Enterprise) language with complete
token recognition, position tracking, and error recovery.

## Features

- **Complete BSL token support** - keywords, identifiers, operators, literals
- **Position tracking** - line and column information for every token
- **Error recovery** - handles unknown tokens gracefully
- **Performance optimized** - regex-based tokenization with caching
- **Integration ready** - works with existing BslLexer and analyzer pipeline

## Usage

```rust
use crate::analyzer::lexical_analyzer::LexicalAnalyzer;
use crate::core::AnalysisError;

let mut analyzer = LexicalAnalyzer::new();
let tokens = analyzer.tokenize(bsl_code)?;
```
*/

use std::collections::HashSet;
use regex::Regex;
use anyhow::{Result, anyhow};

/// Token produced by lexical analysis
#[derive(Debug, Clone, PartialEq)]
pub struct LexicalToken {
    pub token_type: LexicalTokenType,
    pub value: String,
    pub line: usize,
    pub column: usize,
    pub length: usize,
}

impl LexicalToken {
    pub fn new(token_type: LexicalTokenType, value: String, line: usize, column: usize) -> Self {
        let length = value.len();
        Self {
            token_type,
            value,
            line,
            column,
            length,
        }
    }
}

/// Types of tokens recognized by the lexical analyzer
#[derive(Debug, Clone, PartialEq)]
pub enum LexicalTokenType {
    // Core constructs
    Keyword,
    Identifier,
    
    // Literals
    Number,
    String,
    
    // Operators and punctuation
    Operator,
    
    // Comments and whitespace
    Comment,
    Whitespace,
    Newline,
    
    // Unknown/error tokens
    Unknown,
}

/// Lexical analyzer configuration
#[derive(Debug, Clone)]
pub struct LexicalAnalysisConfig {
    pub include_whitespace: bool,
    pub include_comments: bool,
    pub verbose: bool,
}

impl Default for LexicalAnalysisConfig {
    fn default() -> Self {
        Self {
            include_whitespace: false,
            include_comments: true,
            verbose: false,
        }
    }
}

/// BSL Lexical Analyzer
pub struct LexicalAnalyzer {
    /// BSL keywords
    keywords: HashSet<&'static str>,
    /// BSL operators
    operators: HashSet<&'static str>,
    /// Compiled regex patterns for tokenization
    patterns: Vec<(LexicalTokenType, Regex)>,
    /// Configuration
    config: LexicalAnalysisConfig,
}

impl LexicalAnalyzer {
    /// Creates a new lexical analyzer
    pub fn new() -> Self {
        Self::with_config(LexicalAnalysisConfig::default())
    }
    
    /// Creates a new lexical analyzer with custom configuration
    pub fn with_config(config: LexicalAnalysisConfig) -> Self {
        let keywords = Self::create_keywords();
        let operators = Self::create_operators();
        let patterns = Self::create_patterns();
        
        Self {
            keywords,
            operators,  
            patterns,
            config,
        }
    }
    
    /// Creates BSL keywords set
    fn create_keywords() -> HashSet<&'static str> {
        let mut keywords = HashSet::new();
        
        // Core constructs
        keywords.insert("Процедура");
        keywords.insert("Функция");
        keywords.insert("КонецПроцедуры");
        keywords.insert("КонецФункции");
        keywords.insert("Перем");
        keywords.insert("Экспорт");
        keywords.insert("&НаСервере");
        keywords.insert("&НаКлиенте");
        keywords.insert("&НаСервереБезКонтекста");
        
        // Conditional constructs
        keywords.insert("Если");
        keywords.insert("Тогда");
        keywords.insert("Иначе");
        keywords.insert("ИначеЕсли");
        keywords.insert("КонецЕсли");
        
        // Loops
        keywords.insert("Для");
        keywords.insert("Каждого");
        keywords.insert("Из");
        keywords.insert("Цикл");
        keywords.insert("КонецЦикла");
        keywords.insert("Пока");
        keywords.insert("Индекс");
        keywords.insert("По");
        
        // Exception handling
        keywords.insert("Попытка");
        keywords.insert("Исключение");
        keywords.insert("КонецПопытки");
        
        // Flow control
        keywords.insert("Возврат");
        keywords.insert("Продолжить");
        keywords.insert("Прервать");
        
        // Object creation
        keywords.insert("Новый");
        
        // Constants
        keywords.insert("Неопределено");
        keywords.insert("Истина");
        keywords.insert("Ложь");
        keywords.insert("Null");
        
        // Logical operators
        keywords.insert("И");
        keywords.insert("Или");
        keywords.insert("НЕ");
        keywords.insert("Не");
        
        // Special directives
        keywords.insert("&УсловнаяКомпиляция");
        keywords.insert("&КонецЕсли");
        keywords.insert("&Область");
        keywords.insert("&КонецОбласти");
        
        keywords
    }
    
    /// Creates BSL operators set
    fn create_operators() -> HashSet<&'static str> {
        let mut operators = HashSet::new();
        
        // Arithmetic
        operators.insert("+");
        operators.insert("-");
        operators.insert("*");
        operators.insert("/");
        
        // Assignment and comparison
        operators.insert("=");
        operators.insert(":=");
        operators.insert("<");
        operators.insert(">");
        operators.insert("<=");
        operators.insert(">=");
        operators.insert("<>");
        
        // Punctuation
        operators.insert("(");
        operators.insert(")");
        operators.insert("[");
        operators.insert("]");
        operators.insert("{");
        operators.insert("}");
        operators.insert(".");
        operators.insert(",");
        operators.insert(";");
        operators.insert(":");
        operators.insert("?");
        
        operators
    }
    
    /// Creates compiled regex patterns for tokenization
    fn create_patterns() -> Vec<(LexicalTokenType, Regex)> {
        vec![
            // Comments (single-line)
            (
                LexicalTokenType::Comment,
                Regex::new(r"//.*").expect("Invalid comment regex")
            ),
            
            // Strings (double quotes)
            (
                LexicalTokenType::String,
                Regex::new(r#""[^"]*""#).expect("Invalid string regex")
            ),
            
            // Strings (single quotes) 
            (
                LexicalTokenType::String,
                Regex::new(r"'[^']*'").expect("Invalid string regex")
            ),
            
            // Numbers (integers and floats)
            (
                LexicalTokenType::Number,
                Regex::new(r"\b\d+(?:\.\d+)?\b").expect("Invalid number regex")
            ),
            
            // Identifiers and keywords (Cyrillic and Latin)
            (
                LexicalTokenType::Identifier,
                Regex::new(r"\b[a-zA-Zа-яА-Я_&][a-zA-Zа-яА-Я0-9_]*\b").expect("Invalid identifier regex")
            ),
            
            // Multi-character operators
            (
                LexicalTokenType::Operator,
                Regex::new(r"(:=)|(<>)|(<=)|(>=)").expect("Invalid multi-char operator regex")
            ),
            
            // Single-character operators and punctuation
            (
                LexicalTokenType::Operator,
                Regex::new(r"[+\-*/=<>()\[\]{}.,;:?]").expect("Invalid operator regex")
            ),
            
            // Newlines
            (
                LexicalTokenType::Newline,
                Regex::new(r"\r?\n").expect("Invalid newline regex")
            ),
            
            // Whitespace
            (
                LexicalTokenType::Whitespace,
                Regex::new(r"[ \t]+").expect("Invalid whitespace regex")
            )
        ]
    }
    
    /// Tokenizes BSL source code
    pub fn tokenize(&self, code: &str) -> Result<Vec<LexicalToken>> {
        if self.config.verbose {
            println!("🔍 Starting lexical analysis");
        }
        
        if code.trim().is_empty() {
            if self.config.verbose {
                println!("📝 Code is empty, returning empty token list");
            }
            return Ok(Vec::new());
        }

        let mut tokens = Vec::new();
        let mut pos = 0;
        let mut line = 1;
        let mut column = 1;
        
        // Convert to UTF-8 chars for proper iteration
        let chars: Vec<char> = code.chars().collect();
        let mut char_pos = 0;
        
        while pos < code.len() {
            let mut matched = false;
            
            // Try to match against each pattern
            for (token_type, pattern) in &self.patterns {
                if let Some(mat) = pattern.find_at(code, pos) {
                    if mat.start() == pos {
                        let value = mat.as_str().to_string();
                        let token_type = self.determine_token_type(&value, token_type.clone());
                        
                        let token = LexicalToken::new(token_type.clone(), value.clone(), line, column);
                        
                        // Only include token if configured to do so
                        let should_include = match token_type {
                            LexicalTokenType::Whitespace => self.config.include_whitespace,
                            LexicalTokenType::Comment => self.config.include_comments,
                            LexicalTokenType::Newline => self.config.include_whitespace, // Treat newlines as whitespace
                            _ => true,
                        };
                        
                        if should_include {
                            tokens.push(token);
                        }
                        
                        // Update position - count characters properly
                        let match_chars = value.chars().count();
                        pos = mat.end();
                        char_pos += match_chars;
                        
                        if token_type == LexicalTokenType::Newline {
                            line += 1;
                            column = 1;
                        } else {
                            column += match_chars;
                        }
                        
                        matched = true;
                        break;
                    }
                }
            }
            
            // Handle unknown characters
            if !matched {
                if char_pos < chars.len() {
                    let unknown_char = chars[char_pos];
                    let token = LexicalToken::new(
                        LexicalTokenType::Unknown,
                        unknown_char.to_string(),
                        line,
                        column
                    );
                    tokens.push(token);
                    
                    pos += unknown_char.len_utf8();
                    char_pos += 1;
                    column += 1;
                } else {
                    break; // Avoid infinite loop
                }
            }
        }
        
        if self.config.verbose {
            println!("🔍 Lexical analysis completed. Tokens: {}", tokens.len());
        }
        
        Ok(tokens)
    }    /// Determines the exact token type based on value and base type
    fn determine_token_type(&self, value: &str, base_type: LexicalTokenType) -> LexicalTokenType {
        match base_type {
            LexicalTokenType::Identifier => {
                if self.keywords.contains(value) {
                    LexicalTokenType::Keyword
                } else {
                    LexicalTokenType::Identifier
                }
            }
            LexicalTokenType::Operator => {
                if self.operators.contains(value) {
                    LexicalTokenType::Operator
                } else {
                    LexicalTokenType::Unknown
                }
            }
            _ => base_type,
        }
    }
    
    /// Validates a list of tokens
    pub fn validate_tokens(&self, tokens: &[LexicalToken]) -> Result<()> {
        for (i, token) in tokens.iter().enumerate() {
            if token.value.is_empty() {
                return Err(anyhow!("Token {} has empty value", i));
            }
            
            if token.line == 0 || token.column == 0 {
                return Err(anyhow!("Token {} has invalid position (line: {}, column: {})", 
                                 i, token.line, token.column));
            }
        }
        
        Ok(())
    }
}

impl Default for LexicalAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lexical_analyzer_creation() {
        let analyzer = LexicalAnalyzer::new();
        assert!(!analyzer.keywords.is_empty());
        assert!(!analyzer.operators.is_empty());
        assert!(!analyzer.patterns.is_empty());
    }
    
    #[test]
    fn test_tokenize_empty_code() {
        let analyzer = LexicalAnalyzer::new();
        let tokens = analyzer.tokenize("").unwrap();
        assert!(tokens.is_empty());
    }
    
    #[test]
    fn test_tokenize_simple_procedure() {
        let analyzer = LexicalAnalyzer::new();
        let code = "Процедура Тест() КонецПроцедуры";
        let tokens = analyzer.tokenize(code).unwrap();
        
        assert_eq!(tokens.len(), 5); // Процедура, Тест, (, ), КонецПроцедуры
        assert_eq!(tokens[0].token_type, LexicalTokenType::Keyword);
        assert_eq!(tokens[0].value, "Процедура");
        assert_eq!(tokens[1].token_type, LexicalTokenType::Identifier);
        assert_eq!(tokens[1].value, "Тест");
    }
    
    #[test]
    fn test_tokenize_keywords() {
        let analyzer = LexicalAnalyzer::new();
        let code = "Если Тогда Иначе КонецЕсли";
        let tokens = analyzer.tokenize(code).unwrap();
        
        for token in &tokens {
            assert_eq!(token.token_type, LexicalTokenType::Keyword);
        }
    }
    
    #[test]
    fn test_tokenize_numbers() {
        let analyzer = LexicalAnalyzer::new();
        let code = "123 45.67 0";
        let tokens = analyzer.tokenize(code).unwrap();
        
        assert_eq!(tokens.len(), 3);
        for token in &tokens {
            assert_eq!(token.token_type, LexicalTokenType::Number);
        }
    }
    
    #[test]
    fn test_tokenize_strings() {
        let analyzer = LexicalAnalyzer::new();
        let code = r#""Строка" 'Другая строка'"#;
        let tokens = analyzer.tokenize(code).unwrap();
        
        assert_eq!(tokens.len(), 2);
        for token in &tokens {
            assert_eq!(token.token_type, LexicalTokenType::String);
        }
    }
    
    #[test]
    fn test_tokenize_operators() {
        let analyzer = LexicalAnalyzer::new();
        let code = "+ - * / = := < > <= >= <>";
        let tokens = analyzer.tokenize(code).unwrap();
        
        for token in &tokens {
            assert_eq!(token.token_type, LexicalTokenType::Operator);
        }
    }
    
    #[test]
    fn test_tokenize_comments() {
        let analyzer = LexicalAnalyzer::with_config(LexicalAnalysisConfig {
            include_comments: true,
            ..Default::default()
        });
        let code = "// Это комментарий\nПроцедура";
        let tokens = analyzer.tokenize(code).unwrap();
        
        assert_eq!(tokens[0].token_type, LexicalTokenType::Comment);
        assert_eq!(tokens[0].value, "// Это комментарий");
    }
    
    #[test]
    fn test_position_tracking() {
        let analyzer = LexicalAnalyzer::new();
        let code = "Процедура\n  Тест";
        let tokens = analyzer.tokenize(code).unwrap();
        
        // Debug output
        for (i, token) in tokens.iter().enumerate() {
            println!("Token {}: {:?} at line {}, column {}", i, token.value, token.line, token.column);
        }
        
        assert_eq!(tokens[0].line, 1);
        assert_eq!(tokens[0].column, 1);
        assert_eq!(tokens[1].line, 2);
        assert_eq!(tokens[1].column, 3);
    }
    
    #[test]
    fn test_validate_tokens() {
        let analyzer = LexicalAnalyzer::new();
        let tokens = vec![
            LexicalToken::new(LexicalTokenType::Keyword, "Процедура".to_string(), 1, 1),
            LexicalToken::new(LexicalTokenType::Identifier, "Тест".to_string(), 1, 10),
        ];
        
        assert!(analyzer.validate_tokens(&tokens).is_ok());
    }
}
