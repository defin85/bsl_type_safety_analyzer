//! Модуль для конвертации диагностик между форматами

use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Range, Position, NumberOrString};
use crate::core::errors::{AnalysisError, ErrorLevel};

/// Конвертирует AnalysisError в LSP Diagnostic
pub fn convert_to_lsp_diagnostic(error: &AnalysisError) -> Diagnostic {
    Diagnostic {
        range: Range {
            start: Position {
                line: error.position.line.saturating_sub(1) as u32,
                character: error.position.column.saturating_sub(1) as u32,
            },
            end: Position {
                line: error.position.line.saturating_sub(1) as u32,
                character: (error.position.column + 20).saturating_sub(1) as u32,
            },
        },
        severity: Some(convert_severity(&error.level)),
        code: error.error_code.as_ref().map(|c| NumberOrString::String(c.clone())),
        code_description: None,
        source: Some("bsl-analyzer".to_string()),
        message: error.message.clone(),
        related_information: None,
        tags: None,
        data: None,
    }
}

/// Конвертирует ErrorLevel в DiagnosticSeverity
fn convert_severity(level: &ErrorLevel) -> DiagnosticSeverity {
    match level {
        ErrorLevel::Error => DiagnosticSeverity::ERROR,
        ErrorLevel::Warning => DiagnosticSeverity::WARNING,
        ErrorLevel::Info => DiagnosticSeverity::INFORMATION,
        ErrorLevel::Hint => DiagnosticSeverity::HINT,
    }
}

/// Конвертирует список ошибок и предупреждений в LSP диагностики
pub fn convert_analysis_results(
    errors: Vec<AnalysisError>,
    warnings: Vec<AnalysisError>,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    
    // Конвертируем ошибки
    for error in errors {
        diagnostics.push(convert_to_lsp_diagnostic(&error));
    }
    
    // Конвертируем предупреждения
    for warning in warnings {
        diagnostics.push(convert_to_lsp_diagnostic(&warning));
    }
    
    diagnostics
}

/// Создает диагностику для ошибки анализа
pub fn create_analysis_error_diagnostic(message: String, code: &str) -> Diagnostic {
    Diagnostic {
        range: Range {
            start: Position { line: 0, character: 0 },
            end: Position { line: 0, character: 0 },
        },
        severity: Some(DiagnosticSeverity::ERROR),
        code: Some(NumberOrString::String(code.to_string())),
        code_description: None,
        source: Some("bsl-analyzer".to_string()),
        message,
        related_information: None,
        tags: None,
        data: None,
    }
}

/// Создает информационную диагностику
pub fn create_info_diagnostic(message: &str) -> Diagnostic {
    Diagnostic {
        range: Range {
            start: Position { line: 0, character: 0 },
            end: Position { line: 0, character: 0 },
        },
        severity: Some(DiagnosticSeverity::INFORMATION),
        code: None,
        code_description: None,
        source: Some("bsl-lsp".to_string()),
        message: message.to_string(),
        related_information: None,
        tags: None,
        data: None,
    }
}