//! Simple string interner for Phase 4 (prototype).
//! Provides stable SymbolId for deduplicated identifier / literal strings.
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(pub u32);

#[derive(Debug, Default, Clone)]
pub struct StringInterner {
    map: HashMap<String, SymbolId>,
    rev: Vec<String>,
    bytes: usize,
}

impl StringInterner {
    pub fn new() -> Self { Self::default() }
    pub fn intern<S: AsRef<str>>(&mut self, s: S) -> SymbolId {
        let st = s.as_ref();
        if let Some(id) = self.map.get(st) { return *id; }
        let id = SymbolId(self.rev.len() as u32);
        self.bytes += st.len();
        self.rev.push(st.to_string());
        self.map.insert(st.to_string(), id);
        id
    }
    pub fn resolve(&self, sym: SymbolId) -> &str { &self.rev[sym.0 as usize] }
    pub fn len(&self) -> usize { self.rev.len() }
    pub fn is_empty(&self) -> bool { self.rev.is_empty() }
    pub fn bytes(&self) -> usize { self.bytes }
    pub fn symbol_count(&self) -> usize { self.rev.len() }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn interning_deduplicates() {
        let mut i = StringInterner::new();
        let a = i.intern("Name");
        let b = i.intern("Name");
        assert_eq!(a, b);
        assert_eq!(i.len(), 1);
        assert_eq!(i.resolve(a), "Name");
    }
}
