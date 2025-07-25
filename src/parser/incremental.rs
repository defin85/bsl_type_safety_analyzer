/*!
# Incremental AST Parser for BSL

Implements incremental parsing to update only changed parts of the AST tree
instead of rebuilding the entire tree on each code change.
*/

use super::ast::{AstNode, AstNodeType, Position, Span};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

/// Represents a text change in the source code
#[derive(Debug, Clone)]
pub struct TextEdit {
    /// Start position of the edit
    pub start: Position,
    /// End position of the edit (before change)
    pub end: Position,
    /// New text to insert
    pub text: String,
}

impl TextEdit {
    pub fn new(start: Position, end: Position, text: String) -> Self {
        Self { start, end, text }
    }

    /// Calculate the new end position after applying this edit
    pub fn new_end(&self) -> Position {
        let lines_added = self.text.matches('\n').count();
        if lines_added == 0 {
            Position::new(
                self.start.line,
                self.start.column + self.text.len(),
                self.start.offset + self.text.len(),
            )
        } else {
            let last_line_length = self.text.split('\n').last().unwrap_or("").len();
            Position::new(
                self.start.line + lines_added,
                last_line_length,
                self.start.offset + self.text.len(),
            )
        }
    }
}

/// Cache entry for parsed AST nodes
#[derive(Debug, Clone)]
struct CacheEntry {
    /// The parsed AST node
    node: Arc<AstNode>,
    /// Source text hash for cache validation
    #[allow(dead_code)]
    source_hash: u64,
    /// Last modification timestamp
    #[allow(dead_code)]
    timestamp: std::time::SystemTime,
}

/// Incremental AST parser that maintains cache of parsed nodes
#[derive(Debug)]
pub struct IncrementalParser {
    /// Cache of parsed AST nodes by source position
    cache: HashMap<String, CacheEntry>,
    /// Current source code
    source: String,
    /// Current AST tree
    current_tree: Option<Arc<AstNode>>,
}

impl IncrementalParser {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            source: String::new(),
            current_tree: None,
        }
    }

    /// Parse initial source code
    pub fn parse_initial(&mut self, source: &str) -> Result<Arc<AstNode>> {
        info!("Parsing initial BSL source ({} chars)", source.len());
        
        self.source = source.to_string();
        
        // TODO: Replace with actual BSL grammar parser
        let tree = self.parse_full_tree(source)?;
        let tree_arc = Arc::new(tree);
        
        // Cache all nodes from the initial parse
        self.cache_tree_nodes(&tree_arc, source);
        self.current_tree = Some(tree_arc.clone());
        
        Ok(tree_arc)
    }

    /// Apply text edit and incrementally update AST
    pub fn apply_edit(&mut self, edit: TextEdit) -> Result<Arc<AstNode>> {
        info!("Applying incremental edit at {}:{}", edit.start.line, edit.start.column);
        
        // Apply text change to source
        let (new_source, affected_range) = self.apply_text_edit(&edit)?;
        self.source = new_source;
        
        // Find nodes that might be affected by this change
        let affected_nodes = self.find_affected_nodes(&affected_range);
        
        if affected_nodes.is_empty() {
            debug!("No nodes affected by edit, keeping current tree");
            return Ok(self.current_tree.as_ref().unwrap().clone());
        }

        // Incrementally rebuild only affected parts
        let updated_tree = self.rebuild_affected_nodes(affected_nodes, &affected_range)?;
        self.current_tree = Some(updated_tree.clone());
        
        Ok(updated_tree)
    }

    /// Get current AST tree
    pub fn current_tree(&self) -> Option<Arc<AstNode>> {
        self.current_tree.clone()
    }

    /// Clear cache (useful for memory management)
    pub fn clear_cache(&mut self) {
        self.cache.clear();
        info!("AST cache cleared");
    }

    /// Get cache size for statistics
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }

    // Private implementation methods

    fn apply_text_edit(&self, edit: &TextEdit) -> Result<(String, Span)> {
        let mut new_source = self.source.clone();
        
        // Calculate byte offsets for the edit
        let start_offset = self.position_to_offset(&edit.start)?;
        let end_offset = self.position_to_offset(&edit.end)?;
        
        // Apply the text change
        new_source.replace_range(start_offset..end_offset, &edit.text);
        
        // Calculate affected range
        let affected_range = Span::new(edit.start, edit.new_end());
        
        Ok((new_source, affected_range))
    }

    fn position_to_offset(&self, pos: &Position) -> Result<usize> {
        let mut offset = 0;
        let mut current_line = 0;
        
        for ch in self.source.chars() {
            if current_line == pos.line && offset == pos.column {
                return Ok(offset);
            }
            
            if ch == '\n' {
                current_line += 1;
            }
            
            offset += ch.len_utf8();
        }
        
        Ok(offset.min(self.source.len()))
    }

    fn find_affected_nodes(&self, range: &Span) -> Vec<String> {
        let mut affected = Vec::new();
        
        for (cache_key, entry) in &self.cache {
            if self.node_intersects_range(&entry.node, range) {
                affected.push(cache_key.clone());
            }
        }
        
        debug!("Found {} affected nodes", affected.len());
        affected
    }

    fn node_intersects_range(&self, node: &AstNode, range: &Span) -> bool {
        // Check if node's span intersects with the changed range
        let node_span = &node.span;
        
        !(node_span.end.offset < range.start.offset || 
          node_span.start.offset > range.end.offset)
    }

    fn rebuild_affected_nodes(&mut self, affected_keys: Vec<String>, range: &Span) -> Result<Arc<AstNode>> {
        // Remove affected nodes from cache
        for key in &affected_keys {
            self.cache.remove(key);
        }
        
        // Find the smallest containing node that needs to be reparsed
        let reparse_range = self.find_reparse_range(range);
        
        // Extract source text for reparsing
        let start_offset = self.position_to_offset(&reparse_range.start)?;
        let end_offset = self.position_to_offset(&reparse_range.end)?;
        let reparse_text = &self.source[start_offset..end_offset];
        
        // Parse the affected section
        let new_subtree = self.parse_subtree(reparse_text, &reparse_range)?;
        
        // Merge back into the main tree
        let updated_tree = self.merge_subtree(new_subtree, &reparse_range)?;
        
        // Cache the new nodes
        let source_clone = self.source.clone();
        self.cache_tree_nodes(&updated_tree, &source_clone);
        
        Ok(updated_tree)
    }

    fn find_reparse_range(&self, change_range: &Span) -> Span {
        // For BSL, we need to find statement or procedure boundaries
        // This is a simplified version - in practice, you'd look for:
        // - Procedure/Function boundaries
        // - Statement boundaries (semicolons)
        // - Block boundaries (КонецПроцедуры, КонецФункции)
        
        let expanded_start = Position::new(
            change_range.start.line,
            0, // Start of line
            change_range.start.offset - change_range.start.column,
        );
        
        // Find next statement/procedure end
        let expanded_end = self.find_next_safe_boundary(&change_range.end);
        
        Span::new(expanded_start, expanded_end)
    }

    fn find_next_safe_boundary(&self, pos: &Position) -> Position {
        // In BSL, safe boundaries are:
        // - End of procedure/function (КонецПроцедуры, КонецФункции)
        // - End of statement (semicolon + newline)
        // - End of file
        
        // Simplified: just go to end of next line
        Position::new(
            pos.line + 1,
            0,
            pos.offset + 50, // Approximate
        )
    }

    fn parse_subtree(&self, text: &str, range: &Span) -> Result<Arc<AstNode>> {
        debug!("Reparsing subtree: {} chars at {}:{}", 
               text.len(), range.start.line, range.start.column);
        
        // TODO: Use actual BSL grammar parser
        // For now, create a simple statement node
        let mut node = AstNode::new(AstNodeType::Block, *range);
        
        // Parse statements within the text
        for (i, line) in text.lines().enumerate() {
            if !line.trim().is_empty() {
                let line_span = Span::new(
                    Position::new(range.start.line + i, 0, range.start.offset + i * 50),
                    Position::new(range.start.line + i, line.len(), range.start.offset + i * 50 + line.len()),
                );
                
                let stmt_node = AstNode::with_value(
                    AstNodeType::Assignment, 
                    line_span, 
                    line.to_string()
                );
                node.add_child(stmt_node);
            }
        }
        
        Ok(Arc::new(node))
    }

    fn merge_subtree(&self, new_subtree: Arc<AstNode>, _range: &Span) -> Result<Arc<AstNode>> {
        // Clone current tree and replace the affected range
        let current = self.current_tree.as_ref().unwrap();
        let mut new_tree = (**current).clone();
        
        // Remove old nodes in range and insert new subtree
        // This is simplified - in practice you'd do proper tree surgery
        new_tree.children.clear();
        new_tree.add_child((*new_subtree).clone());
        
        Ok(Arc::new(new_tree))
    }

    fn cache_tree_nodes(&mut self, tree: &Arc<AstNode>, source: &str) {
        let source_hash = self.calculate_hash(source);
        self.cache_node_recursive(tree, source_hash, std::time::SystemTime::now());
    }

    fn cache_node_recursive(&mut self, node: &Arc<AstNode>, source_hash: u64, timestamp: std::time::SystemTime) {
        let cache_key = format!("{}:{}:{}", 
                               node.span.start.line, 
                               node.span.start.column,
                               node.node_type);
        
        self.cache.insert(cache_key, CacheEntry {
            node: node.clone(),
            source_hash,
            timestamp,
        });
        
        for child in &node.children {
            let child_arc = Arc::new(child.clone());
            self.cache_node_recursive(&child_arc, source_hash, timestamp);
        }
    }

    fn calculate_hash(&self, text: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        hasher.finish()
    }

    fn parse_full_tree(&self, source: &str) -> Result<AstNode> {
        // TODO: Replace with actual BSL grammar parser
        // This is a placeholder implementation
        
        let span = Span::new(
            Position::zero(),
            Position::new(
                source.lines().count().saturating_sub(1),
                source.lines().last().unwrap_or("").len(),
                source.len(),
            ),
        );
        
        let mut module = AstNode::new(AstNodeType::Module, span);
        
        // Parse each line as a statement (simplified)
        for (line_num, line) in source.lines().enumerate() {
            if !line.trim().is_empty() {
                let line_span = Span::new(
                    Position::new(line_num, 0, line_num * 50),
                    Position::new(line_num, line.len(), line_num * 50 + line.len()),
                );
                
                let stmt = AstNode::with_value(
                    AstNodeType::Assignment,
                    line_span,
                    line.to_string(),
                );
                module.add_child(stmt);
            }
        }
        
        Ok(module)
    }
}

impl Default for IncrementalParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_incremental_parsing() {
        let mut parser = IncrementalParser::new();
        
        // Initial parse
        let source = "Процедура Тест()\n  Переменная = 1;\nКонецПроцедуры";
        let tree = parser.parse_initial(source).unwrap();
        assert_eq!(tree.node_type, AstNodeType::Module);
        
        // Apply edit
        let edit = TextEdit::new(
            Position::new(1, 2, 20),
            Position::new(1, 13, 31),
            "НоваяПеременная".to_string(),
        );
        
        let updated_tree = parser.apply_edit(edit).unwrap();
        assert_eq!(updated_tree.node_type, AstNodeType::Module);
    }

    #[test]
    fn test_text_edit_calculation() {
        let edit = TextEdit::new(
            Position::new(0, 5, 5),
            Position::new(0, 10, 10),
            "hello\nworld".to_string(),
        );
        
        let new_end = edit.new_end();
        assert_eq!(new_end.line, 1);
        assert_eq!(new_end.column, 5);
    }
}
