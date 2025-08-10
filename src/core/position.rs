/*!
# Source position types (Position, Span)

Centralized location types used across the analyzer to decouple core from legacy parser module.
*/

use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;

/// Position in source code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

impl Position {
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Self { line, column, offset }
    }

    pub fn zero() -> Self {
        Self::new(0, 0, 0)
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// Span in source code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Span {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    pub fn zero() -> Self {
        Self::new(Position::zero(), Position::zero())
    }
}

/// Compact span representation (offset + length) within a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PackedSpan {
    pub start: u32,
    pub len: u32,
}

impl PackedSpan {
    pub fn new(start: u32, len: u32) -> Self { Self { start, len } }
    pub fn empty() -> Self { Self { start: 0, len: 0 } }
    pub fn end(&self) -> u32 { self.start + self.len }
}

/// File identifier (stable within one analysis session)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FileId(pub u32);

/// Line index for fast offset->(line,column) mapping.
#[derive(Debug, Clone)]
pub struct LineIndex {
    /// Byte offsets where each line starts.
    line_starts: Arc<Vec<u32>>, // Arc для дешёвого клонирования
}

impl LineIndex {
    pub fn new(text: &str) -> Self {
        let mut starts = Vec::with_capacity(text.len() / 32 + 1);
        starts.push(0u32);
        for (i, b) in text.bytes().enumerate() {
            if b == b'\n' { starts.push((i + 1) as u32); }
        }
        Self { line_starts: Arc::new(starts) }
    }

    pub fn line_count(&self) -> usize { self.line_starts.len() }

    pub fn to_position(&self, offset: u32) -> Position {
        // Бинарный поиск последнего line_start <= offset
        let starts = &self.line_starts;
        let mut lo = 0usize;
        let mut hi = starts.len();
        while lo + 1 < hi {
            let mid = (lo + hi) / 2;
            if starts[mid] <= offset { lo = mid; } else { hi = mid; }
        }
        let line_start = starts[lo];
        Position::new(lo, (offset - line_start) as usize, offset as usize)
    }

    pub fn position_range(&self, span: PackedSpan) -> Span {
        let start = self.to_position(span.start);
        let end = self.to_position(span.end());
        Span::new(start, end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_index_basic() {
        let text = "line1\nline2\nlast";
        let idx = LineIndex::new(text);
        assert_eq!(idx.line_count(), 3);
        let p = idx.to_position(7); // 'i' in line2
        assert_eq!(p.line, 1);
        assert_eq!(p.column, 1);
    }

    #[test]
    fn test_packed_span_to_span() {
        let text = "ab\ncd"; // offsets: a=0 b=1 \n=2 c=3 d=4
        let idx = LineIndex::new(text);
        let ps = PackedSpan::new(3, 2); // 'cd'
        let span = idx.position_range(ps);
        assert_eq!(span.start.line, 1);
        assert_eq!(span.start.column, 0);
        assert_eq!(span.end.column, 2);
    }
}
