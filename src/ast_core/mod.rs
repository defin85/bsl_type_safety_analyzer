//! Experimental new AST core (Phase 1)
//!
//! Не интегрировано в основной пайплайн. Цель: ввести фундаментальные примитивы
//! (NodeId, Arena, AstKind, PackedSpan) для последующего постепенного переноса
//! анализа. API на данном этапе нестабилен.

use crate::core::position::PackedSpan;
use crate::ast_core::interner::{StringInterner, SymbolId};

/// Устойчивый идентификатор узла внутри одной арены.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub u32);

/// Вид AST узла (минимальный набор).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AstKind {
    Module,
    Procedure,
    Function,
    Param,
    Block,
    Identifier,
    Literal,
    Call,
    Member,
    Assignment,
    New,
    VarDecl,
    If,
    While,
    Return,
    Error,
    Binary,
    Unary,
}

/// Дополнительные данные для конкретных узлов.
#[derive(Debug, Clone, Default)]
pub enum AstPayload {
    #[default]
    None,
    /// Идентификатор (только интернированный символ).
    Ident { sym: SymbolId },
    /// Литерал (только интернированный символ).
    Literal { sym: SymbolId },
    /// Ошибка хранит индекс сообщения в отдельной таблице (payload split step 1).
    Error { msg: u32 },
}

/// Узел в аренe. Дети связаны через first_child / next_sibling (sibling chain).
#[derive(Debug, Clone)]
pub struct AstNode {
    pub kind: AstKind,
    pub first_child: Option<NodeId>,
    pub next_sibling: Option<NodeId>,
    pub span: PackedSpan,
    pub payload: AstPayload,
}

impl AstNode {
    fn new(kind: AstKind, span: PackedSpan, payload: AstPayload) -> Self {
        Self { kind, first_child: None, next_sibling: None, span, payload }
    }
}

/// Агрегатор всех узлов.
#[derive(Default, Debug)]
pub struct Arena {
    nodes: Vec<AstNode>,
}

impl Arena {
    pub fn new() -> Self { Self { nodes: Vec::new() } }
    fn alloc(&mut self, node: AstNode) -> NodeId {
        let id = NodeId(self.nodes.len() as u32);
        self.nodes.push(node);
        id
    }
    pub fn node(&self, id: NodeId) -> &AstNode { &self.nodes[id.0 as usize] }
    pub fn node_mut(&mut self, id: NodeId) -> &mut AstNode { &mut self.nodes[id.0 as usize] }
    pub fn len(&self) -> usize { self.nodes.len() }
    pub fn is_empty(&self) -> bool { self.nodes.is_empty() }
    pub fn iter(&self) -> impl Iterator<Item=(NodeId, &AstNode)> { self.nodes.iter().enumerate().map(|(i,n)|(NodeId(i as u32), n)) }
}

/// Построитель дерева (nested push/pop стеком).
pub struct AstBuilder {
    arena: Arena,
    stack: Vec<NodeId>,
    root: Option<NodeId>,
    interner: StringInterner,
    error_messages: Vec<String>,
}

impl AstBuilder {
    pub fn new() -> Self { Self { arena: Arena::new(), stack: Vec::new(), root: None, interner: StringInterner::new(), error_messages: Vec::new() } }

    pub fn start_node(&mut self, kind: AstKind, span: PackedSpan) {
        let id = self.arena.alloc(AstNode::new(kind, span, AstPayload::None));
        self.attach(id);
        self.stack.push(id);
        if self.root.is_none() { self.root = Some(id); }
    }

    pub fn start_node_with_payload(&mut self, kind: AstKind, span: PackedSpan, payload: AstPayload) {
        let id = self.arena.alloc(AstNode::new(kind, span, payload));
        self.attach(id);
        self.stack.push(id);
        if self.root.is_none() { self.root = Some(id); }
    }

    pub fn start_node_with_ident(&mut self, kind: AstKind, span: PackedSpan, name: String) {
        let sym = self.interner.intern(&name);
        self.start_node_with_payload(kind, span, AstPayload::Ident { sym });
    }

    pub fn finish_node(&mut self) {
        if let Some(&id) = self.stack.last() {
            // Автоматическая агрегация span для Block если изначально (0,0)
            let node = self.arena.node(id).clone();
            if node.kind == AstKind::Block && node.span.len == 0 {
                // вычисляем min start и max end из детей
                let mut min_start: Option<u32> = None;
                let mut max_end: u32 = 0;
                let mut cur = node.first_child;
                while let Some(c) = cur {
                    let ch = self.arena.node(c);
                    let st = ch.span.start;
                    let en = ch.span.start + ch.span.len;
                    min_start = Some(match min_start { Some(ms) => ms.min(st), None => st });
                    if en > max_end { max_end = en; }
                    cur = ch.next_sibling;
                }
                if let Some(ms) = min_start { if max_end >= ms { let span = PackedSpan::new(ms, max_end - ms); self.arena.node_mut(id).span = span; } }
            }
        }
        self.stack.pop();
    }

    pub fn leaf(&mut self, kind: AstKind, span: PackedSpan, payload: AstPayload) -> NodeId {
        let id = self.arena.alloc(AstNode::new(kind, span, payload));
        self.attach(id);
        id
    }

    pub fn leaf_ident(&mut self, span: PackedSpan, name: String) -> NodeId {
        let sym = self.interner.intern(&name);
        self.leaf(AstKind::Identifier, span, AstPayload::Ident { sym })
    }

    pub fn leaf_literal(&mut self, span: PackedSpan, text: String) -> NodeId {
        let sym = self.interner.intern(&text);
        self.leaf(AstKind::Literal, span, AstPayload::Literal { sym })
    }

    pub fn intern_symbol(&mut self, text: &str) -> SymbolId { self.interner.intern(text) }

    /// Создать Error-узел (leaf) с сообщением.
    pub fn error(&mut self, span: PackedSpan, message: impl Into<String>) -> NodeId {
        let msg = message.into();
        let idx = self.error_messages.len();
        self.error_messages.push(msg);
        self.leaf(AstKind::Error, span, AstPayload::Error { msg: idx as u32 })
    }

    fn attach(&mut self, id: NodeId) {
        if let Some(&parent) = self.stack.last() {
            let parent_node = self.arena.node_mut(parent);
            match parent_node.first_child {
                None => parent_node.first_child = Some(id),
                Some(first) => {
                    // Найти последний sibling
                    let mut cur = first;
                    loop {
                        let next = self.arena.node(cur).next_sibling;
                        if let Some(n) = next { cur = n; } else { break; }
                    }
                    self.arena.node_mut(cur).next_sibling = Some(id);
                }
            }
        }
    }

    pub fn build(self) -> BuiltAst { BuiltAst { arena: self.arena, root: self.root.expect("AST has no root"), interner: self.interner, error_messages: self.error_messages } }
}

impl Default for AstBuilder {
    fn default() -> Self { Self::new() }
}

#[derive(Debug)]
pub struct BuiltAst {
    pub arena: Arena,
    pub root: NodeId,
    pub interner: StringInterner,
    pub error_messages: Vec<String>,
}

impl BuiltAst {
    pub fn root(&self) -> NodeId { self.root }
    pub fn arena(&self) -> &Arena { &self.arena }
    pub fn resolve_symbol(&self, sym: SymbolId) -> &str { self.interner.resolve(sym) }
    pub fn interner_symbol_count(&self) -> usize { self.interner.symbol_count() }
    pub fn interner_total_bytes(&self) -> usize { self.interner.bytes() }
    /// Получить текст идентификатора по NodeId (если это Identifier/Param/Function/Procedure с payload Ident).
    pub fn node_ident_text(&self, id: NodeId) -> Option<&str> {
        let node = self.arena.node(id);
        match node.payload { AstPayload::Ident { sym } => Some(self.resolve_symbol(sym)), _ => None }
    }
    /// Получить текст литерала по NodeId (если узел Literal и имеет payload Literal).
    pub fn node_literal_text(&self, id: NodeId) -> Option<&str> {
        let node = self.arena.node(id);
        match node.payload { AstPayload::Literal { sym } => Some(self.resolve_symbol(sym)), _ => None }
    }
    /// Получить сообщение об ошибке для Error узла.
    pub fn node_error_message(&self, id: NodeId) -> Option<&str> {
        let node = self.arena.node(id);
        match node.payload { AstPayload::Error { msg } => self.error_messages.get(msg as usize).map(|s| s.as_str()), _ => None }
    }
}

/// Утилита обхода (предварительный проход).
pub fn preorder<'a>(arena: &'a Arena, root: NodeId) -> impl Iterator<Item=NodeId> + 'a {
    let mut stack = vec![root];
    let mut out = Vec::new();
    while let Some(id) = stack.pop() {
        out.push(id);
        // push children в обратном порядке для сохранения слева-направо
        let mut children = Vec::new();
        let mut cur = arena.node(id).first_child;
        while let Some(c) = cur { children.push(c); cur = arena.node(c).next_sibling; }
        for c in children.into_iter().rev() { stack.push(c); }
    }
    out.into_iter()
}

/// Итератор детей конкретного узла.
pub struct NodeChildren<'a> {
    arena: &'a Arena,
    next: Option<NodeId>,
}

impl<'a> Iterator for NodeChildren<'a> {
    type Item = NodeId;
    fn next(&mut self) -> Option<Self::Item> {
        let cur = self.next?;
        self.next = self.arena.node(cur).next_sibling;
        Some(cur)
    }
}

impl Arena {
    pub fn children(&self, id: NodeId) -> NodeChildren<'_> {
        NodeChildren { arena: self, next: self.node(id).first_child }
    }
    /// Дети заданного kind.
    pub fn children_of_kind(&self, id: NodeId, kind: AstKind) -> impl Iterator<Item=NodeId> + '_ {
        self.children(id).filter(move |c| self.node(*c).kind == kind)
    }
    /// Первый ребенок нужного kind.
    pub fn first_child_of_kind(&self, id: NodeId, kind: AstKind) -> Option<NodeId> {
        self.children(id).find(|c| self.node(*c).kind == kind)
    }
}

impl AstNode {
    pub fn ident_text<'a>(&self, interner: &'a StringInterner) -> Option<&'a str> { match &self.payload { AstPayload::Ident { sym } => Some(interner.resolve(*sym)), _ => None } }
    pub fn literal_text<'a>(&self, interner: &'a StringInterner) -> Option<&'a str> { match &self.payload { AstPayload::Literal { sym } => Some(interner.resolve(*sym)), _ => None } }
    pub fn ident_symbol(&self) -> Option<SymbolId> { match &self.payload { AstPayload::Ident { sym, .. } => Some(*sym), _ => None } }
    pub fn literal_symbol(&self) -> Option<SymbolId> { match &self.payload { AstPayload::Literal { sym, .. } => Some(*sym), _ => None } }
}

/// Контроль обхода.
pub enum VisitControl { Continue, SkipChildren, Stop }

/// Visitor API (enter/leave). Возвращаем VisitControl из enter.
pub trait Visitor {
    fn enter(&mut self, _id: NodeId, _node: &AstNode, _arena: &Arena) -> VisitControl { VisitControl::Continue }
    fn leave(&mut self, _id: NodeId, _node: &AstNode, _arena: &Arena) {}
}

/// Обход дерева с visitor (preorder + post события). Возвращает true если не прерван.
pub fn walk<V: Visitor>(arena: &Arena, root: NodeId, visitor: &mut V) -> bool {
    // Нестрогая рекурсия (глубина BSL обычно умеренная). Можно заменить на явный стек позже.
    fn inner<V: Visitor>(arena: &Arena, id: NodeId, vis: &mut V) -> Option<()> {
        let node = arena.node(id);
        match vis.enter(id, node, arena) {
            VisitControl::Continue => {
                // дети
                let mut child = node.first_child;
                while let Some(c) = child {
                    inner(arena, c, vis)?;
                    child = arena.node(c).next_sibling;
                }
            }
            VisitControl::SkipChildren => { /* пропуск */ }
            VisitControl::Stop => { return None; }
        }
        vis.leave(id, node, arena);
        Some(())
    }
    inner(arena, root, visitor).is_some()
}

pub mod interner;

#[cfg(test)]
mod tests {
    use super::*;

    fn ps(start: u32, len: u32) -> PackedSpan { PackedSpan::new(start, len) }

    #[test]
    fn build_simple_module() {
        let mut b = AstBuilder::new();
        b.start_node(AstKind::Module, ps(0, 10));
        b.start_node_with_ident(AstKind::Procedure, ps(0,5), "TestProc".into());
        b.leaf_ident(ps(0,4), "Var".into());
        b.finish_node(); // procedure
        b.finish_node(); // module
        let ast = b.build();
        assert_eq!(ast.arena.len(), 3);
        let order: Vec<_> = preorder(&ast.arena, ast.root).collect();
        assert_eq!(order.len(), 3);
        let proc_node = ast.arena.node(order[1]);
        matches!(proc_node.payload, AstPayload::Ident { .. });
    }

    struct CountingVisitor { enter_seq: Vec<AstKind>, leaves: usize, stop_after: usize }
    impl Visitor for CountingVisitor {
    fn enter(&mut self, _id: NodeId, node: &AstNode, _arena: &Arena) -> VisitControl {
            self.enter_seq.push(node.kind);
            if self.enter_seq.len() == self.stop_after { return VisitControl::Stop; }
            VisitControl::Continue
        }
        fn leave(&mut self, _id: NodeId, _node: &AstNode, _arena: &Arena) { self.leaves += 1; }
    }

    #[test]
    fn visitor_walk_and_error_node() {
        let mut b = AstBuilder::new();
        b.start_node(AstKind::Module, ps(0, 20));
    b.start_node_with_ident(AstKind::Function, ps(0,10), "F".into());
        b.error(ps(5,0), "unexpected token");
    b.leaf_literal(ps(6,1), "1".into());
        b.finish_node();
        b.finish_node();
        let ast = b.build();
        // Найти error узел
        let mut has_error = false;
    for (id, node) in ast.arena.iter() { if matches!(node.payload, AstPayload::Error { .. }) { has_error = true; assert_eq!(node.kind, AstKind::Error); assert_eq!(node.span.len, 0); assert!(matches!(ast.arena.node(id).payload, AstPayload::Error { .. })); } }
        assert!(has_error, "Error node not found");
        let mut v = CountingVisitor { enter_seq: Vec::new(), leaves: 0, stop_after: 10 };
        let completed = walk(&ast.arena, ast.root, &mut v);
        assert!(completed);
        assert!(v.enter_seq.len() >= 4); // Module, Function, Error, Literal (порядок pre-order)
    }
}
