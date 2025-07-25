/*!
# Error System for BSL Analyzer

Comprehensive error handling and reporting system.
Ported from Python implementation with enhanced type safety.
*/

use serde::{Deserialize, Serialize};
use std::fmt;
use crate::parser::ast::Position;

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorLevel {
    Error,
    Warning,  
    Info,
    Hint,
}

impl fmt::Display for ErrorLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorLevel::Error => write!(f, "ERROR"),
            ErrorLevel::Warning => write!(f, "WARNING"),
            ErrorLevel::Info => write!(f, "INFO"),
            ErrorLevel::Hint => write!(f, "HINT"),
        }
    }
}

/// Analysis error with position and context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisError {
    pub message: String,
    pub position: Position,
    pub level: ErrorLevel,
    pub error_code: Option<String>,
    pub suggestion: Option<String>,
    pub related_positions: Vec<Position>,
}

impl AnalysisError {
    pub fn new(message: String, position: Position, level: ErrorLevel) -> Self {
        Self {
            message,
            position,
            level,
            error_code: None,
            suggestion: None,
            related_positions: Vec::new(),
        }
    }
    
    pub fn with_code(mut self, code: String) -> Self {
        self.error_code = Some(code);
        self
    }
    
    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestion = Some(suggestion);
        self
    }
    
    pub fn with_related_position(mut self, position: Position) -> Self {
        self.related_positions.push(position);
        self
    }
}

impl fmt::Display for AnalysisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}: {}", 
               self.level, 
               self.position, 
               self.message)?;
               
        if let Some(code) = &self.error_code {
            write!(f, " ({})", code)?;
        }
        
        if let Some(suggestion) = &self.suggestion {
            write!(f, "\n  Suggestion: {}", suggestion)?;
        }
        
        Ok(())
    }
}

/// Error collection and reporting
#[derive(Debug, Default)]
pub struct ErrorCollector {
    pub errors: Vec<AnalysisError>,
}

impl ErrorCollector {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
        }
    }
    
    pub fn add_error(&mut self, error: AnalysisError) {
        self.errors.push(error);
    }
    
    pub fn add_simple_error(&mut self, message: String, position: Position, level: ErrorLevel) {
        self.add_error(AnalysisError::new(message, position, level));
    }
    
    pub fn has_errors(&self) -> bool {
        self.errors.iter().any(|e| e.level == ErrorLevel::Error)
    }
    
    pub fn has_warnings(&self) -> bool {
        self.errors.iter().any(|e| e.level == ErrorLevel::Warning)
    }
    
    pub fn error_count(&self) -> usize {
        self.errors.iter().filter(|e| e.level == ErrorLevel::Error).count()
    }
    
    pub fn warning_count(&self) -> usize {
        self.errors.iter().filter(|e| e.level == ErrorLevel::Warning).count()
    }
    
    pub fn get_errors(&self) -> Vec<&AnalysisError> {
        self.errors.iter().filter(|e| e.level == ErrorLevel::Error).collect()
    }
    
    pub fn get_warnings(&self) -> Vec<&AnalysisError> {
        self.errors.iter().filter(|e| e.level == ErrorLevel::Warning).collect()
    }
    
    pub fn clear(&mut self) {
        self.errors.clear();
    }
    
    pub fn merge(&mut self, other: ErrorCollector) {
        self.errors.extend(other.errors);
    }
}

impl fmt::Display for ErrorCollector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for error in &self.errors {
            writeln!(f, "{}", error)?;
        }
        Ok(())
    }
}
