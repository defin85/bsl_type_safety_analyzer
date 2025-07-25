// Диагностика
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Info,
    Hint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiagnosticCode {
    SyntaxError,
    TypeError,
    UndefinedVariable,
    UnusedVariable,
    MissingExport,
    CircularDependency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub code: Option<DiagnosticCode>,
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub source: Option<String>,
}

impl Diagnostic {
    pub fn error(message: String, line: usize, column: usize) -> Self {
        Self {
            level: DiagnosticLevel::Error,
            code: None,
            message,
            line,
            column,
            source: None,
        }
    }
    
    pub fn warning(message: String, line: usize, column: usize) -> Self {
        Self {
            level: DiagnosticLevel::Warning,
            code: None,
            message,
            line,
            column,
            source: None,
        }
    }
    
    pub fn info(message: String, line: usize, column: usize) -> Self {
        Self {
            level: DiagnosticLevel::Info,
            code: None,
            message,
            line,
            column,
            source: None,
        }
    }
}
