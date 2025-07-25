/*!
# LSP Integration for Incremental Parsing

Demonstrates how incremental parsing can be integrated with LSP server
for real-time code analysis and diagnostics.
*/

#![allow(dead_code)]

use crate::parser::{IncrementalParser, TextEdit, Position, AstNode};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::{debug, info, warn};

/// Document state for LSP integration
#[derive(Debug)]
struct DocumentState {
    /// URI of the document
    uri: String,
    /// Current version number
    version: i32,
    /// Document text
    text: String,
    /// Incremental parser instance
    parser: IncrementalParser,
    /// Last parsed AST
    ast: Option<Arc<AstNode>>,
    /// Parsing errors
    errors: Vec<String>,
}

/// LSP document manager with incremental parsing
pub struct IncrementalLspManager {
    /// Open documents
    documents: Arc<RwLock<HashMap<String, DocumentState>>>,
}

impl IncrementalLspManager {
    pub fn new() -> Self {
        Self {
            documents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Open a new document
    pub fn open_document(&self, uri: String, version: i32, text: String) -> Result<()> {
        info!("Opening document: {} (version {})", uri, version);
        
        let mut parser = IncrementalParser::new();
        let ast = parser.parse_initial(&text)?;
        
        let document = DocumentState {
            uri: uri.clone(),
            version,
            text,
            parser,
            ast: Some(ast),
            errors: Vec::new(),
        };
        
        let mut documents = self.documents.write().unwrap();
        documents.insert(uri, document);
        
        Ok(())
    }

    /// Close a document
    pub fn close_document(&self, uri: &str) -> Result<()> {
        info!("Closing document: {}", uri);
        
        let mut documents = self.documents.write().unwrap();
        documents.remove(uri);
        
        Ok(())
    }

    /// Apply text change to document
    pub fn change_document(
        &self, 
        uri: &str, 
        version: i32, 
        changes: Vec<TextChangeEvent>
    ) -> Result<()> {
        debug!("Applying {} changes to document: {} (version {})", 
               changes.len(), uri, version);
        
        let mut documents = self.documents.write().unwrap();
        let document = documents.get_mut(uri)
            .ok_or_else(|| anyhow::anyhow!("Document not found: {}", uri))?;
        
        document.version = version;
        
        // Apply changes incrementally
        for change in changes {
            let edit = self.convert_change_to_edit(&change, &document.text)?;
            
            // Update document text
            document.text = self.apply_text_change(&document.text, &change)?;
            
            // Apply incremental parsing
            match document.parser.apply_edit(edit) {
                Ok(new_ast) => {
                    document.ast = Some(new_ast);
                    document.errors.clear();
                    debug!("Successfully applied incremental parsing");
                },
                Err(e) => {
                    warn!("Incremental parsing failed: {}, falling back to full parse", e);
                    
                    // Fallback to full reparse
                    match document.parser.parse_initial(&document.text) {
                        Ok(ast) => {
                            document.ast = Some(ast);
                            document.errors.clear();
                        },
                        Err(parse_err) => {
                            document.errors.push(format!("Parse error: {}", parse_err));
                            document.ast = None;
                        }
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Get current AST for document
    pub fn get_ast(&self, uri: &str) -> Option<Arc<AstNode>> {
        let documents = self.documents.read().unwrap();
        documents.get(uri)?.ast.clone()
    }

    /// Get parsing errors for document
    pub fn get_errors(&self, uri: &str) -> Vec<String> {
        let documents = self.documents.read().unwrap();
        documents.get(uri)
            .map(|doc| doc.errors.clone())
            .unwrap_or_default()
    }

    /// Get document statistics
    pub fn get_stats(&self) -> LspStats {
        let documents = self.documents.read().unwrap();
        
        LspStats {
            open_documents: documents.len(),
            total_cache_entries: documents.values()
                .map(|doc| doc.parser.cache_size())
                .sum(),
            memory_usage_mb: self.estimate_memory_usage(&documents),
        }
    }

    // Private helper methods

    fn convert_change_to_edit(&self, change: &TextChangeEvent, current_text: &str) -> Result<TextEdit> {
        let start_pos = self.offset_to_position(current_text, change.range.start)?;
        let end_pos = self.offset_to_position(current_text, change.range.end)?;
        
        Ok(TextEdit::new(start_pos, end_pos, change.text.clone()))
    }

    fn offset_to_position(&self, text: &str, offset: usize) -> Result<Position> {
        let mut line = 0;
        let mut column = 0;
        let mut current_offset = 0;
        
        for ch in text.chars() {
            if current_offset >= offset {
                break;
            }
            
            if ch == '\n' {
                line += 1;
                column = 0;
            } else {
                column += 1;
            }
            
            current_offset += ch.len_utf8();
        }
        
        Ok(Position::new(line, column, offset))
    }

    fn apply_text_change(&self, text: &str, change: &TextChangeEvent) -> Result<String> {
        let mut result = text.to_string();
        result.replace_range(change.range.start..change.range.end, &change.text);
        Ok(result)
    }

    fn estimate_memory_usage(&self, documents: &HashMap<String, DocumentState>) -> f64 {
        // Rough estimate in MB
        let text_size: usize = documents.values()
            .map(|doc| doc.text.len())
            .sum();
        
        let ast_size: usize = documents.values()
            .filter_map(|doc| doc.ast.as_ref())
            .map(|_| 1024) // Rough estimate per AST
            .sum();
        
        (text_size + ast_size) as f64 / (1024.0 * 1024.0)
    }
}

/// Text change event from LSP
#[derive(Debug, Clone)]
pub struct TextChangeEvent {
    /// Range of the change
    pub range: TextRange,
    /// New text
    pub text: String,
}

/// Text range
#[derive(Debug, Clone)]
pub struct TextRange {
    /// Start offset
    pub start: usize,
    /// End offset  
    pub end: usize,
}

/// LSP manager statistics
#[derive(Debug)]
pub struct LspStats {
    /// Number of open documents
    pub open_documents: usize,
    /// Total cache entries across all documents
    pub total_cache_entries: usize,
    /// Estimated memory usage in MB
    pub memory_usage_mb: f64,
}

impl Default for IncrementalLspManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lsp_integration() {
        let manager = IncrementalLspManager::new();
        
        // Open document
        let uri = "file:///test.bsl".to_string();
        let initial_text = "Процедура Тест()\nКонецПроцедуры".to_string();
        
        manager.open_document(uri.clone(), 1, initial_text).unwrap();
        
        // Verify AST was created
        let ast = manager.get_ast(&uri);
        assert!(ast.is_some());
        
        // Apply change
        let change = TextChangeEvent {
            range: TextRange { start: 14, end: 14 }, // After "Процедура Тест()"
            text: "\n  // Комментарий".to_string(),
        };
        
        manager.change_document(&uri, 2, vec![change]).unwrap();
        
        // Verify AST was updated
        let updated_ast = manager.get_ast(&uri);
        assert!(updated_ast.is_some());
        
        // Check stats
        let stats = manager.get_stats();
        assert_eq!(stats.open_documents, 1);
    }

    #[test]
    fn test_offset_to_position() {
        let manager = IncrementalLspManager::new();
        let text = "line1\nline2\nline3";
        
        let pos = manager.offset_to_position(text, 6).unwrap(); // Start of "line2"
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 0);
        
        let pos = manager.offset_to_position(text, 8).unwrap(); // "ne2" in "line2"
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 2);
    }
}
