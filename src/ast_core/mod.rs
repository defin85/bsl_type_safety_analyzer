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
    /// Вызов функции/метода: индекс структуры CallData.
    Call { data: u32 },
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
#[derive(Default, Debug, Clone)]
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
    call_data: Vec<CallData>,
}

impl AstBuilder {
    pub fn new() -> Self { Self { arena: Arena::new(), stack: Vec::new(), root: None, interner: StringInterner::new(), error_messages: Vec::new(), call_data: Vec::new() } }

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

    /// Начать Call без немедленного вычисления arg_count.
    pub fn start_call(&mut self, span: PackedSpan) {
        self.start_node(AstKind::Call, span);
    }
    /// Завершить Call: вычислить аргументы и сохранить CallData.
    pub fn finish_call(&mut self) {
        if let Some(&id) = self.stack.last() { // текущий Call
            let node = self.arena.node(id).clone();
            if node.kind == AstKind::Call {
                // children chain: determine method vs function call
                let mut child_ids = Vec::new();
                let mut c = node.first_child;
                while let Some(cc) = c { child_ids.push(cc); c = self.arena.node(cc).next_sibling; }
                let mut is_method = false;
                if child_ids.len() >= 2 {
                    let first_kind = self.arena.node(child_ids[0]).kind;
                    let second_kind = self.arena.node(child_ids[1]).kind;
                    if second_kind == AstKind::Identifier && first_kind != AstKind::Identifier { is_method = true; }
                }
                let arg_start_index = if is_method { 2 } else { 1 };
                let arg_count = if child_ids.len() > arg_start_index { (child_ids.len() - arg_start_index) as u16 } else { 0 };
                let data_idx = self.call_data.len();
                self.call_data.push(CallData { arg_count, is_method });
                // обновляем payload
                if let Some(nm) = self.arena.nodes.get_mut(id.0 as usize) { nm.payload = AstPayload::Call { data: data_idx as u32 }; }
            }
        }
        self.finish_node();
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

    pub fn build(self) -> BuiltAst {
        let mut built = BuiltAst {
            arena: self.arena,
            root: self.root.expect("AST has no root"),
            interner: self.interner,
            error_messages: self.error_messages,
            call_data: self.call_data,
            fingerprints: Vec::new(),
            dirty_fp: Vec::new(),
            fp_generation: 0,
            fingerprint_time_ns: 0,
            last_partial_recomputed: 0,
        };
        let start = std::time::Instant::now();
        built.recompute_fingerprints();
        built.fingerprint_time_ns = start.elapsed().as_nanos();
        built
    }

    /// Построить AST без немедленного вычисления fingerprint'ов (для частичного восстановления).
    pub fn build_without_fingerprints(self) -> BuiltAst {
        BuiltAst {
            arena: self.arena,
            root: self.root.expect("AST has no root"),
            interner: self.interner,
            error_messages: self.error_messages,
            call_data: self.call_data,
            fingerprints: Vec::new(),
            dirty_fp: Vec::new(),
            fp_generation: 0,
            fingerprint_time_ns: 0,
            last_partial_recomputed: 0,
        }
    }
}

impl Default for AstBuilder {
    fn default() -> Self { Self::new() }
}

#[derive(Debug, Clone)]
pub struct BuiltAst {
    pub arena: Arena,
    pub root: NodeId,
    pub interner: StringInterner,
    pub error_messages: Vec<String>,
    pub call_data: Vec<CallData>,
    /// Кэш fingerprint'ов для каждого узла (индекс = NodeId.0). Заполняется при build().
    pub fingerprints: Vec<u64>,
    /// Маркеры грязных fingerprint'ов (true = нужно пересчитать)
    pub dirty_fp: Vec<bool>,
    /// Поколение инкрементального пересчёта
    pub fp_generation: u64,
    /// Время вычисления fingerprint'ов при build (наносекунды).
    pub fingerprint_time_ns: u128,
    /// Сколько fingerprint узлов было пересчитано в последнем partial проходе
    pub last_partial_recomputed: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct CallData { pub arg_count: u16, pub is_method: bool }

impl BuiltAst {
    pub fn root(&self) -> NodeId { self.root }
    pub fn arena(&self) -> &Arena { &self.arena }
    pub fn resolve_symbol(&self, sym: SymbolId) -> &str { self.interner.resolve(sym) }
    pub fn interner_symbol_count(&self) -> usize { self.interner.symbol_count() }
    pub fn interner_total_bytes(&self) -> usize { self.interner.bytes() }
    /// Прямой доступ к кэшированным fingerprint'ам.
    pub fn fingerprints(&self) -> &[u64] { &self.fingerprints }
    /// Полное пересчитывание fingerprint'ов (использовать при отладке / после мутаций).
    pub fn recompute_fingerprints(&mut self) {
        self.fingerprints = compute_fingerprints_internal(self);
    self.dirty_fp = vec![false; self.fingerprints.len()];
    }
    /// Частичное восстановление fingerprint'ов: копированные поддеревья уже имеют fingerprint;
    /// узлы с нулевым значением будут вычислены (пост-ордер), существующие значения не пересчитываются.
    pub fn recompute_fingerprints_partial(&mut self) {
        if self.fingerprints.is_empty() { self.fingerprints = vec![0; self.arena.len()]; }
    if self.dirty_fp.len() != self.arena.len() { self.dirty_fp = vec![true; self.arena.len()]; }
        let start = std::time::Instant::now();
    let mut recomputed = 0usize;
        if !self.arena.is_empty() {
            // Собираем стек для итеративного пост-ордера
            let root = self.root;
            // Post-order двухстековый подход
            let mut stack = vec![root];
            let mut order: Vec<NodeId> = Vec::new();
            while let Some(id) = stack.pop() {
                order.push(id);
                let mut c = self.arena.node(id).first_child;
                while let Some(ch) = c { stack.push(ch); c = self.arena.node(ch).next_sibling; }
            }
            // Теперь вычисляем в обратном порядке (дети раньше родителей)
            for id in order.into_iter().rev() {
                if self.fingerprints[id.0 as usize] != 0 && !self.dirty_fp[id.0 as usize] { continue; }
                // Вычислить fingerprint
                let node = self.arena.node(id);
                let mut h: u64 = 0xcbf29ce484222325;
                fn fnv64(acc: u64, byte: u64) -> u64 { let mut h = acc; h ^= byte.wrapping_mul(0x100000001b3); h = h.wrapping_mul(0x100000001b3); h }
                h = fnv64(h, node.kind as u64);
                h = fnv64(h, node.span.start as u64);
                h = fnv64(h, node.span.len as u64);
                match node.payload {
                    AstPayload::Ident { sym } => { h = fnv64(h, 1); h = fnv64(h, sym.0 as u64); },
                    AstPayload::Literal { sym } => { h = fnv64(h, 2); h = fnv64(h, sym.0 as u64); },
                    AstPayload::Error { msg } => { h = fnv64(h, 3); h = fnv64(h, msg as u64); },
                    AstPayload::Call { data } => { if let Some(cd) = self.call_data.get(data as usize) { h = fnv64(h, 4); h = fnv64(h, cd.arg_count as u64); h = fnv64(h, cd.is_method as u64); } },
                    AstPayload::None => { h = fnv64(h, 0); },
                }
                let mut c = node.first_child;
                while let Some(ch) = c { let cf = self.fingerprints[ch.0 as usize]; h = fnv64(h, cf); c = self.arena.node(ch).next_sibling; }
        self.fingerprints[id.0 as usize] = h;
        self.dirty_fp[id.0 as usize] = false;
        recomputed += 1;
            }
        }
        self.fingerprint_time_ns = start.elapsed().as_nanos();
    self.last_partial_recomputed = recomputed;
    }
    /// Пометить узел и предков грязными.
    pub fn mark_dirty_upwards(&mut self, id: NodeId) {
        if self.dirty_fp.len() != self.arena.len() { self.dirty_fp = vec![false; self.arena.len()]; }
        let parents = self.build_parent_map();
        let mut cur = Some(id);
        while let Some(n) = cur { self.dirty_fp[n.0 as usize] = true; cur = parents[n.0 as usize]; }
    }
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
    /// Получить данные Call (если имеются).
    pub fn node_call_data(&self, id: NodeId) -> Option<CallData> {
        let node = self.arena.node(id);
        match node.payload { AstPayload::Call { data } => self.call_data.get(data as usize).copied(), _ => None }
    }
    /// Подсчитать количество узлов указанного вида.
    pub fn count_kind(&self, kind: AstKind) -> usize {
        preorder(&self.arena, self.root).filter(|nid| self.arena.node(*nid).kind == kind).count()
    }
    /// Подсчитать процедуры + функции.
    pub fn count_routines(&self) -> usize { self.count_kind(AstKind::Procedure) + self.count_kind(AstKind::Function) }
    /// Статистика вызовов: (total_calls, method_calls, function_calls, total_args, max_args)
    pub fn call_stats(&self) -> (usize, usize, usize, usize, u16) {
        let mut total=0; let mut method=0; let mut func=0; let mut args=0; let mut max_args=0u16;
        for (i, cd) in self.call_data.iter().enumerate() { total+=1; if cd.is_method { method+=1; } else { func+=1; } args += cd.arg_count as usize; if cd.arg_count>max_args { max_args=cd.arg_count; }
            // sanity: ensure some node references this call data (debug mode could assert)
            let _ = i; // placeholder
        }
        (total, method, func, args, max_args)
    }
    /// Вычислить vector fingerprint'ов (post-order) для всех узлов. Fingerprint стабилен между запусками.
    /// Совместимость со старым API: возвращает клон кэшированного списка.
    pub fn compute_fingerprints(&self) -> Vec<u64> { self.fingerprints.clone() }
    /// Root fingerprint convenience (берёт из кэша).
    pub fn root_fingerprint(&self) -> u64 { self.fingerprints.get(self.root.0 as usize).copied().unwrap_or(0) }
    /// Сравнить с другим AST и вернуть список узлов (NodeId) с отличающимися fingerprint'ами.
    /// Возвращает None если размеры арен различаются (сравнение невалидно).
    pub fn diff_changed_nodes(&self, other: &BuiltAst) -> Option<Vec<NodeId>> {
        if self.arena.len() != other.arena.len() { return None; }
        let mut changed = Vec::new();
        for i in 0..self.arena.len() {
            if self.fingerprints.get(i) != other.fingerprints.get(i) { changed.push(NodeId(i as u32)); }
        }
        Some(changed)
    }
    /// Построить parent map (однократно дёшево) для операций инкрементальности.
    pub fn build_parent_map(&self) -> Vec<Option<NodeId>> {
        let mut parents = vec![None; self.arena.len()];
        for (nid, _) in self.arena.iter() {
            let mut child = self.arena.node(nid).first_child;
            while let Some(c) = child { parents[c.0 as usize] = Some(nid); child = self.arena.node(c).next_sibling; }
        }
        parents
    }
    /// Проверка пересечения span узла с диапазоном [start,end). Если end==start трактуется как вставка и попадает
    /// в узел, если позиция лежит внутри или на конце узла.
    fn span_overlaps(span: PackedSpan, start: u32, end: u32) -> bool {
        if end == start { // insertion
            start >= span.start && start <= span.end()
        } else {
            start < span.end() && end > span.start
        }
    }
    /// Все узлы, span которых пересекается с диапазоном.
    pub fn overlapping_nodes(&self, start: u32, end: u32) -> Vec<NodeId> {
        let mut out = Vec::new();
        for (nid, node) in self.arena.iter() { if Self::span_overlaps(node.span, start, end) { out.push(nid); } }
        out
    }
    /// Overlapping root nodes: пересекающиеся узлы без пересекающегося предка (наиболее "верхние").
    pub fn overlapping_root_nodes(&self, start: u32, end: u32) -> Vec<NodeId> {
        let overlaps = self.overlapping_nodes(start, end);
        if overlaps.is_empty() { return overlaps; }
        let parent_map = self.build_parent_map();
        overlaps.iter().copied().filter(|nid| {
            let mut p = parent_map[nid.0 as usize];
            while let Some(pp) = p { if overlaps.contains(&pp) { return false; } p = parent_map[pp.0 as usize]; }
            true
        }).collect()
    }
    /// Overlapping leaf nodes: пересекающиеся узлы без пересекающегося потомка (наиболее "глубокие").
    pub fn overlapping_leaf_nodes(&self, start: u32, end: u32) -> Vec<NodeId> {
        let overlaps = self.overlapping_nodes(start, end);
        if overlaps.is_empty() { return overlaps; }
        // Пометим пересекающиеся
        let mut is_overlap = vec![false; self.arena.len()];
        for nid in &overlaps { is_overlap[nid.0 as usize] = true; }
        // Узел лист пересекающихся если ни один его ребёнок не в overlaps
        overlaps.into_iter().filter(|nid| {
            let mut child = self.arena.node(*nid).first_child;
            while let Some(c) = child { if is_overlap[c.0 as usize] { return false; } child = self.arena.node(c).next_sibling; }
            true
        }).collect()
    }
    /// Определить минимальные «границы» для частичной пересборки по dirty range.
    /// Правила:
    /// 1. Начинаем с deepest overlapping (leaf) узлов.
    /// 2. Поднимаемся вверх до ближайшего узла вида Procedure | Function.
    /// 3. Если ни один boundary-kind не найден (например изменение на уровне модульных деклараций), возвращаем Module root.
    /// 4. Дедупликация.
    pub fn dirty_rebuild_boundaries(&self, start: u32, end: u32) -> Vec<NodeId> {
        use std::collections::HashSet;
        let leaf_overlaps = self.overlapping_leaf_nodes(start, end);
        if leaf_overlaps.is_empty() { return Vec::new(); }
        let parent_map = self.build_parent_map();
        let mut out: Vec<NodeId> = Vec::new();
        let mut seen: HashSet<u32> = HashSet::new();
        'each: for leaf in leaf_overlaps {
            let mut cur = leaf;
            loop {
                let kind = self.arena.node(cur).kind;
                if matches!(kind, AstKind::Procedure | AstKind::Function) {
                    if seen.insert(cur.0) { out.push(cur); }
                    continue 'each;
                }
                if let Some(p) = parent_map[cur.0 as usize] { cur = p; } else { break; }
                if cur == self.root { break; }
            }
            // boundary не найден — модуль
            if seen.insert(self.root.0) { out.push(self.root); }
        }
        // Детеминированный порядок
        out.sort_by_key(|n| n.0);
        out
    }
    /// Сформировать структурный diff по fingerprint'ам (прототип инкрементальности).
    pub fn fingerprint_diff(&self, other: &BuiltAst) -> Option<FingerprintDiff> {
        if self.arena.len() != other.arena.len() { return None; }
        let mut changed = Vec::new();
        let mut reused_nodes = 0usize;
        let mut reused_subtrees = 0usize;
        let parent_map = self.build_parent_map();
        for i in 0..self.arena.len() {
            let a = self.fingerprints[i];
            let b = other.fingerprints[i];
            if a == b { reused_nodes += 1; }
        }
        // reused_subtrees: узлы с совпадающим fingerprint и (root или родитель тоже совпал) учитываются только когда родитель отличается (или отсутствует), т.е. верхушки полностью совпавших поддеревьев.
        for (i, parent_opt) in parent_map.iter().enumerate() {
            if self.fingerprints[i] == other.fingerprints[i] {
                if let Some(p) = parent_opt {
                    if self.fingerprints[p.0 as usize] != other.fingerprints[p.0 as usize] { reused_subtrees += 1; }
                } else { reused_subtrees += 1; }
            } else { changed.push(NodeId(i as u32)); }
        }
        Some(FingerprintDiff { changed, reused_nodes, total_nodes: self.arena.len(), reused_subtrees })
    }
}

/// Результат сравнения двух AST по fingerprint'ам (прототип для incremental rebuild).
#[derive(Debug, Clone)]
pub struct FingerprintDiff {
    pub changed: Vec<NodeId>,
    pub reused_nodes: usize,
    pub reused_subtrees: usize,
    pub total_nodes: usize,
}

impl FingerprintDiff {
    pub fn to_stats(&self) -> IncrementalStats { self.to_stats_with_timing(None, None, None) }
    pub fn to_stats_with_timing(&self, parse_ns: Option<u128>, arena_ns: Option<u128>, fingerprint_ns: Option<u128>) -> IncrementalStats {
        let changed_nodes = self.changed.len();
        let reuse_ratio = if self.total_nodes > 0 { self.reused_nodes as f64 / self.total_nodes as f64 } else { 0.0 };
        let total_ns = match (parse_ns, arena_ns, fingerprint_ns) {
            (Some(p), Some(a), Some(f)) => Some(p + a + f),
            _ => None,
        };
        IncrementalStats {
            total_nodes: self.total_nodes,
            changed_nodes,
            reused_nodes: self.reused_nodes,
            reused_subtrees: self.reused_subtrees,
            reuse_ratio,
            parse_ns,
            arena_ns,
            fingerprint_ns,
            semantic_ns: None,
            total_ns,
            planned_routines: None,
            replaced_routines: None,
            fallback_reason: None,
            initial_touched: None,
            expanded_touched: None,
            inner_reused_nodes: None,
            inner_reuse_ratio: None,
            recomputed_fingerprints: None,
            semantic_processed_routines: None,
            semantic_reused_routines: None,
            semantic_selective_ratio: None,
        }
    }
}

/// Сводные метрики инкрементального сравнения (без таймингов).
#[derive(Debug, Clone, Copy, Default)]
pub struct IncrementalStats {
    pub total_nodes: usize,
    pub changed_nodes: usize,
    pub reused_nodes: usize,
    pub reused_subtrees: usize,
    pub reuse_ratio: f64,
    pub parse_ns: Option<u128>,
    pub arena_ns: Option<u128>,
    pub fingerprint_ns: Option<u128>,
    /// Время семантического анализа (selective или полного) в наносекундах
    pub semantic_ns: Option<u128>,
    pub total_ns: Option<u128>,
    /// Плановое количество рутин к перестройке (если применялся RebuildPlan)
    pub planned_routines: Option<usize>,
    /// Фактически заменённые рутины (пока = planned_routines в прототипе)
    pub replaced_routines: Option<usize>,
    /// Причина fallback (module|heur_fraction|heur_absolute|exp_fraction|exp_absolute)
    pub fallback_reason: Option<&'static str>,
    /// Кол-во рутин затронутых до расширения зависимостями
    pub initial_touched: Option<usize>,
    /// Кол-во рутин после расширения зависимостей
    pub expanded_touched: Option<usize>,
    /// Дополнительно переиспользовано внутренних узлов внутри заменённых рутин (глубокое переиспользование)
    pub inner_reused_nodes: Option<usize>,
    /// Процент внутренних узлов заменённых рутин, переиспользованных после расширенного анализа
    pub inner_reuse_ratio: Option<f64>,
    /// Число пересчитанных fingerprint узлов в частичном цикле
    pub recomputed_fingerprints: Option<usize>,
    /// Сколько рутин было реально пересемантизировано (при selective)
    pub semantic_processed_routines: Option<usize>,
    /// Сколько рутин семантики удалось переиспользовать без пересчёта
    pub semantic_reused_routines: Option<usize>,
    /// Доля переиспользованных (reused/total)
    pub semantic_selective_ratio: Option<f64>,
}

/// Внутренняя реализация вычисления fingerprint'ов (post-order, стабильный).
fn compute_fingerprints_internal(ast: &BuiltAst) -> Vec<u64> {
    let mut fp = vec![0u64; ast.arena.len()];
    fn fnv64(acc: u64, byte: u64) -> u64 { let mut h = acc; h ^= byte.wrapping_mul(0x100000001b3); h = h.wrapping_mul(0x100000001b3); h }
    fn node_fp(ast: &BuiltAst, id: NodeId, out: &mut [u64]) -> u64 {
        if out[id.0 as usize] != 0 { return out[id.0 as usize]; }
        // children first (post-order)
        let mut children_fps = Vec::new();
        let mut cur = ast.arena.node(id).first_child;
        while let Some(c) = cur { let cf = node_fp(ast, c, out); children_fps.push(cf); cur = ast.arena.node(c).next_sibling; }
        let node = ast.arena.node(id);
        let mut h: u64 = 0xcbf29ce484222325; // FNV offset
        h = fnv64(h, node.kind as u64);
        h = fnv64(h, node.span.start as u64);
        h = fnv64(h, node.span.len as u64);
        match node.payload {
            AstPayload::Ident { sym } => { h = fnv64(h, 1); h = fnv64(h, sym.0 as u64); },
            AstPayload::Literal { sym } => { h = fnv64(h, 2); h = fnv64(h, sym.0 as u64); },
            AstPayload::Error { msg } => { h = fnv64(h, 3); h = fnv64(h, msg as u64); },
            AstPayload::Call { data } => { if let Some(cd) = ast.call_data.get(data as usize) { h = fnv64(h, 4); h = fnv64(h, cd.arg_count as u64); h = fnv64(h, cd.is_method as u64); } },
            AstPayload::None => { h = fnv64(h, 0); },
        }
        for cf in children_fps { h = fnv64(h, cf); }
        out[id.0 as usize] = h; h
    }
    if !ast.arena.is_empty() { node_fp(ast, ast.root, &mut fp); }
    fp
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
pub mod incremental;

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
