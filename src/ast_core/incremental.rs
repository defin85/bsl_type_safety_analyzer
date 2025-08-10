//! Incremental rebuild prototype utilities.
//! Phase 5: selective (partial) rebuild planning.
//!
//! This module defines a minimal planning layer that, given a BuiltAst and a dirty text range,
//! identifies routine-level (Procedure/Function) subtrees to rebuild, and structures a plan
//! separating reused vs replaced nodes. The actual re-construction of subtrees is left for
//! subsequent steps (integration with parser / converter pipeline). For now, we provide:
//! - Boundary detection wrapper (delegates to BuiltAst::dirty_rebuild_boundaries)
//! - RebuildPlan struct capturing target routine nodes and fallback mode
//! - Simple heuristics: if > threshold of routines touched, fallback to full rebuild
//!
//! Future extensions:
//! - Node remapping table (old NodeId -> new NodeId) for preserved subtrees
//! - Fingerprint invalidation only for affected ancestors
//! - Semantic layer reuse (symbol tables) per routine

use super::{BuiltAst, NodeId, AstKind};
use super::{AstBuilder, AstPayload};

/// Клонирование поддерева из `src_ast` в новый AstBuilder. Возвращает Err если встречены unsupported payload (Call/Error).
fn clone_subtree(src_ast: &BuiltAst, id: NodeId, builder: &mut AstBuilder) -> Result<(), ()> {
    let node = src_ast.arena().node(id).clone();
    // Определяем есть ли дети
    let has_children = node.first_child.is_some();
    // Маппинг payload
    let mapped_payload = match node.payload {
        AstPayload::None => AstPayload::None,
        AstPayload::Ident { sym } => {
            let text = src_ast.resolve_symbol(sym).to_string();
            AstPayload::Ident { sym: builder.intern_symbol(&text) }
        },
        AstPayload::Literal { sym } => {
            let text = src_ast.resolve_symbol(sym).to_string();
            AstPayload::Literal { sym: builder.intern_symbol(&text) }
        },
        AstPayload::Error { .. } => return Err(()),
        AstPayload::Call { .. } => return Err(()),
    };
    if has_children {
        // Ветка: внутренний узел (включая рутину / блок)
        match node.payload {
            AstPayload::Ident { sym } if matches!(node.kind, AstKind::Procedure | AstKind::Function) => {
                // Рутины с идентификатором: используем специализированный start_node_with_ident для стабильности символа
                let name = src_ast.resolve_symbol(sym).to_string();
                builder.start_node_with_ident(node.kind, node.span, name);
            }
            _ => {
                builder.start_node_with_payload(node.kind, node.span, mapped_payload.clone());
            }
        }
        // Дети
        let mut child = node.first_child;
        while let Some(c) = child { clone_subtree(src_ast, c, builder)?; child = src_ast.arena().node(c).next_sibling; }
        builder.finish_node();
    } else {
        // Лист
        match node.payload {
            AstPayload::Ident { sym } => {
                let name = src_ast.resolve_symbol(sym).to_string();
                builder.leaf_ident(node.span, name);
            }
            AstPayload::Literal { sym } => {
                let lit = src_ast.resolve_symbol(sym).to_string();
                builder.leaf_literal(node.span, lit);
            }
            AstPayload::None => { builder.leaf(node.kind, node.span, AstPayload::None); }
            _ => return Err(()),
        }
    }
    Ok(())
}

/// Построить гибридный AST: рутины из плана берём из `new_full`, остальные копируем из `old_ast`.
/// Возвращает (BuiltAst, replaced_count, fallback_used).
pub fn selective_rebuild(old_ast: &BuiltAst, new_full: &BuiltAst, plan: &RebuildPlan) -> (BuiltAst, usize, bool, usize, usize) {
    if plan.fallback_full { return (new_full.clone(), 0, true, 0, 0); }
    // Готовим новый билдера
    let mut b = AstBuilder::new();
    // Старт модуля основываясь на новом AST (актуальные границы)
    let module_span = new_full.arena().node(new_full.root()).span;
    b.start_node(AstKind::Module, module_span);
    let mut replaced = 0usize;
    let mut fallback = false;
    // Собираем множество NodeId рутины к замене
    use std::collections::HashSet;
    let target: HashSet<u32> = plan.routines_to_rebuild.iter().map(|n| n.0).collect();
    // Индекс новых рутин по имени
    let mut new_proc_map = std::collections::HashMap::new();
    let mut child = new_full.arena().node(new_full.root()).first_child;
    while let Some(c) = child { if matches!(new_full.arena().node(c).kind, AstKind::Procedure | AstKind::Function) { if let Some(name) = new_full.node_ident_text(c) { new_proc_map.insert((new_full.arena().node(c).kind, name.to_string()), c); } } child = new_full.arena().node(c).next_sibling; }
    // Итерируем детей старого модуля в порядке
    let mut old_child = old_ast.arena().node(old_ast.root()).first_child;
    while let Some(c) = old_child {
        let node = old_ast.arena().node(c);
        if matches!(node.kind, AstKind::Procedure | AstKind::Function) && target.contains(&c.0) {
            // Заменяем на новую версию по имени
            if let Some(name) = old_ast.node_ident_text(c) {
                if let Some(&new_id) = new_proc_map.get(&(node.kind, name.to_string())) {
                    if clone_subtree(new_full, new_id, &mut b).is_err() { fallback = true; break; }
                    replaced += 1;
                } else { fallback = true; break; }
            } else { fallback = true; break; }
        } else {
            // Клонируем старый поддерево
            if clone_subtree(old_ast, c, &mut b).is_err() { fallback = true; break; }
        }
        old_child = node.next_sibling;
    }
    b.finish_node();
    if fallback { return (new_full.clone(), 0, true, 0, 0); }
    // Строим без fingerprint'ов
    let mut hybrid = b.build_without_fingerprints();
    // Копируем fingerprint'ы из old_ast для поддеревьев рутин, которые не были заменены (по имени)
    // Формируем отображение: имя рутины -> fingerprint корня рутины в old_ast
    use std::collections::HashMap;
    let mut old_fp_by_name: HashMap<(AstKind,String), u64> = HashMap::new();
    let mut child_old = old_ast.arena().node(old_ast.root()).first_child;
    while let Some(c) = child_old { let node = old_ast.arena().node(c); if matches!(node.kind, AstKind::Procedure|AstKind::Function) { if let Some(name) = old_ast.node_ident_text(c) { old_fp_by_name.insert((node.kind, name.to_string()), old_ast.fingerprints()[c.0 as usize]); } } child_old = node.next_sibling; }
    // Отмечаем какие рутинные поддеревья были заменены (по имени)
    let replaced_names: HashSet<(AstKind,String)> = plan.routines_to_rebuild.iter().filter_map(|nid| {
        let node = old_ast.arena().node(*nid); old_ast.node_ident_text(*nid).map(|name| (node.kind, name.to_string())) }).collect();
    // Копируем fingerprint'ы для совпадающих по имени поддеревьев которые не в replaced_names; сначала найдём соответствующие узлы в новом гибриде
    let mut hybrid_child = hybrid.arena().node(hybrid.root()).first_child;
    if hybrid.fingerprints.is_empty() { hybrid.fingerprints = vec![0; hybrid.arena.len()]; }
    while let Some(c) = hybrid_child { let node = hybrid.arena().node(c); if matches!(node.kind, AstKind::Procedure|AstKind::Function) { if let Some(name) = hybrid.node_ident_text(c) { let key = (node.kind, name.to_string()); if !replaced_names.contains(&key) { if let Some(&fp) = old_fp_by_name.get(&key) { hybrid.fingerprints[c.0 as usize] = fp; } } } } hybrid_child = hybrid.arena().node(c).next_sibling; }
    // Помечаем заменённые рутины (и их предков) грязными для точного подсчёта пересчитанных fingerprint'ов
    for repl in &plan.routines_to_rebuild { hybrid.mark_dirty_upwards(*repl); }
    // Частичный пересчёт недостающих fingerprint'ов (и грязных)
    hybrid.recompute_fingerprints_partial();

    // Глубокое переиспользование: для каждой заменённой рутины сравним старое и новое поддерево и посчитаем совпадающие fingerprint дочерних узлов.
    let mut inner_reused = 0usize;
    let mut inner_total = 0usize;
    for repl in &plan.routines_to_rebuild {
        // Найти старый и новый id по имени
        if let Some(old_name) = old_ast.node_ident_text(*repl) {
            // Найти соответствующий узел в гибриде (он уже заменён новой версией) с тем же именем и kind
            let mut c = hybrid.arena().node(hybrid.root()).first_child;
            while let Some(hc) = c {
                let hnode = hybrid.arena().node(hc);
                if matches!(hnode.kind, AstKind::Procedure|AstKind::Function) {
                    if let Some(hname) = hybrid.node_ident_text(hc) { if hname == old_name { // сравниваем внутренние узлы
                        // Построим карту fingerprint->count в старой версии поддерева
                        use std::collections::HashSet;
                        let mut old_fps: HashSet<u64> = HashSet::new();
                        fn collect_fp(ast:&BuiltAst,id:NodeId,set:&mut std::collections::HashSet<u64>) { set.insert(ast.fingerprints()[id.0 as usize]); let mut ch = ast.arena().node(id).first_child; while let Some(cc)=ch { collect_fp(ast, cc, set); ch = ast.arena().node(cc).next_sibling; } }
                        collect_fp(old_ast, *repl, &mut old_fps);
                        // Теперь считаем сколько fingerprint новых внутренних узлов присутствует (кроме корня – он заведомо менялся)
                        fn count_reused(ast:&BuiltAst,id:NodeId,old_set:&std::collections::HashSet<u64>,acc:&mut usize,total:&mut usize) {
                            let mut ch = ast.arena().node(id).first_child;
                            while let Some(cc)=ch { // считаем только дочерние (не сам root рутины)
                                let fp = ast.fingerprints()[cc.0 as usize];
                                *total += 1; // учитываем каждый внутренний узел
                                if old_set.contains(&fp) { *acc += 1; }
                                count_reused(ast, cc, old_set, acc, total);
                                ch = ast.arena().node(cc).next_sibling;
                            }
                        }
                        count_reused(&hybrid, hc, &old_fps, &mut inner_reused, &mut inner_total);
                        break;
                    } }
                }
                c = hybrid.arena().node(hc).next_sibling;
            }
        }
    }
    // Можно отфильтровать слишком маленькие поддеревья (например inner_total>0 уже отражает объём)
    if inner_total == 0 { inner_reused = 0; }
    (hybrid, replaced, false, inner_reused, inner_total)
}

/// Plan describing how to perform a partial rebuild.
#[derive(Debug, Clone)]
pub struct RebuildPlan {
    /// If true, caller should do a full rebuild (ignore other fields).
    pub fallback_full: bool,
    /// Routine nodes (Procedure / Function) to rebuild fully.
    pub routines_to_rebuild: Vec<NodeId>,
    /// Count of total routines present in the module (for metrics / heuristics).
    pub total_routines: usize,
    /// Первоначально затронуто рутин (до dependency expansion)
    pub initial_touched: usize,
    /// После расширения зависимостей
    pub expanded_touched: usize,
    /// Причина fallback (module|heur_fraction|heur_absolute|exp_fraction|exp_absolute|none)
    pub fallback_reason: &'static str,
}

impl RebuildPlan {
    pub fn full(total_routines: usize) -> Self {
    Self { fallback_full: true, routines_to_rebuild: Vec::new(), total_routines, initial_touched: 0, expanded_touched: 0, fallback_reason: "module" }
    }
}

/// Heuristic thresholds for deciding fallback vs selective.
#[derive(Debug, Clone, Copy)]
pub struct RebuildHeuristics {
    /// If number of touched routines exceeds this fraction of total, fallback to full.
    pub max_touched_fraction: f64,
    /// Absolute cap: if touched routines > this count, fallback.
    pub max_touched_absolute: usize,
}

impl Default for RebuildHeuristics {
    fn default() -> Self { Self { max_touched_fraction: 0.5, max_touched_absolute: 25 } }
}

/// Build a rebuild plan for a dirty text range.
/// Returns a plan instructing selective routine rebuild or full fallback.
pub fn plan_partial_rebuild(ast: &BuiltAst, dirty_start: u32, dirty_end: u32, heur: RebuildHeuristics) -> RebuildPlan {
    let boundaries = ast.dirty_rebuild_boundaries(dirty_start, dirty_end);
    #[cfg(test)] {
        eprintln!("DEBUG boundaries raw ids: {:?}", boundaries.iter().map(|n| n.0).collect::<Vec<_>>());
        for b in &boundaries { let k = ast.arena().node(*b).kind; eprintln!("DEBUG boundary kind {:?}", k); }
    }
    if boundaries.is_empty() {
        // Nothing overlapped: no-op (treat as empty selective plan)
    return RebuildPlan { fallback_full: false, routines_to_rebuild: Vec::new(), total_routines: ast.count_routines(), initial_touched: 0, expanded_touched: 0, fallback_reason: "none" };
    }
    // If module root appears => structural / top-level change => full rebuild.
    let mut has_module = false;
    let mut routine_nodes = Vec::new();
    for nid in &boundaries {
        let kind = ast.arena().node(*nid).kind;
        match kind {
            AstKind::Module => { has_module = true; },
            AstKind::Procedure | AstKind::Function => routine_nodes.push(*nid),
            _ => { /* ignore others */ }
        }
    }
    let total_routines = ast.count_routines();
    if has_module {
        // Попытка уточнения: если dirty range лежит строго внутри ровно одной рутины — можно обойтись без полного фолбэка.
        let mut covering: Vec<NodeId> = Vec::new();
        for (nid, node) in ast.arena().iter() {
            if matches!(node.kind, AstKind::Procedure | AstKind::Function) {
                let span = node.span;
                if dirty_start >= span.start && dirty_end <= span.end() {
                    covering.push(nid);
                }
            }
        }
        if covering.len() == 1 {
            return RebuildPlan { fallback_full: false, routines_to_rebuild: covering, total_routines, initial_touched: 1, expanded_touched: 1, fallback_reason: "none" };
        }
        return RebuildPlan::full(total_routines);
    }
    // Heuristics: fraction or absolute (pre-expansion)
    let touched_initial = routine_nodes.len();
    if total_routines == 0 { return RebuildPlan { fallback_full: false, routines_to_rebuild: routine_nodes, total_routines, initial_touched: 0, expanded_touched: 0, fallback_reason: "none" }; }
    let fraction_initial = touched_initial as f64 / total_routines as f64;
    if fraction_initial > heur.max_touched_fraction {
        let mut p = RebuildPlan::full(total_routines); p.fallback_reason = "heur_fraction"; p.initial_touched = touched_initial; p.expanded_touched = touched_initial; return p;
    }
    if touched_initial > heur.max_touched_absolute {
        let mut p = RebuildPlan::full(total_routines); p.fallback_reason = "heur_absolute"; p.initial_touched = touched_initial; p.expanded_touched = touched_initial; return p;
    }

    // --- Dependency expansion (Step 2) ---
    // Построим обратное отображение: callee_routine_id -> множество caller_routine_id.
    // Используем старый AST (достаточно для приблизительной точности; если сигнатура изменилась, вызывающие всё равно требуют пересемантики).
    use std::collections::{HashMap, HashSet, VecDeque};

    // Соберём map имени рутины -> Vec<NodeId> (в теории и процедура и функция с одинаковым именем)
    let mut name_to_routines: HashMap<String, Vec<NodeId>> = HashMap::new();
    for (nid, node) in ast.arena().iter() {
        if matches!(node.kind, AstKind::Procedure | AstKind::Function) {
            if let Some(name) = ast.node_ident_text(nid) { name_to_routines.entry(name.to_string()).or_default().push(nid); }
        }
    }

    // Обход каждой рутины для извлечения вызовов (Call, где первый ребёнок Identifier => потенциальный вызов функции/процедуры)
    let mut callee_to_callers: HashMap<NodeId, HashSet<NodeId>> = HashMap::new();
    fn walk_collect(ast: &BuiltAst, root: NodeId, f: &mut impl FnMut(NodeId, &super::AstNode)) {
        let node = ast.arena().node(root);
        f(root, node);
        let mut c = node.first_child;
        while let Some(cc) = c { walk_collect(ast, cc, f); c = ast.arena().node(cc).next_sibling; }
    }
    for (caller_id, caller_node) in ast.arena().iter() {
        if matches!(caller_node.kind, AstKind::Procedure | AstKind::Function) {
            walk_collect(ast, caller_id, &mut |_nid, node| {
                if node.kind == AstKind::Call {
                    if let Some(first) = node.first_child {
                        let first_node = ast.arena().node(first);
                        if first_node.kind == AstKind::Identifier {
                            if let super::AstPayload::Ident { sym } = first_node.payload {
                                let call_name = ast.resolve_symbol(sym).to_string();
                                if let Some(callee_ids) = name_to_routines.get(&call_name) {
                                    for callee in callee_ids {
                                        if *callee != caller_id { // избегаем самовызов (не влияет на зависимость)
                                            callee_to_callers.entry(*callee).or_default().insert(caller_id);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            });
        }
    }

    // Расширяем множество изменённых рутин их вызывающими (транзитивно)
    let mut expanded: HashSet<NodeId> = routine_nodes.iter().copied().collect();
    let mut queue: VecDeque<NodeId> = routine_nodes.iter().copied().collect();
    while let Some(changed) = queue.pop_front() {
        if let Some(callers) = callee_to_callers.get(&changed) {
            for caller in callers {
                if expanded.insert(*caller) {
                    queue.push_back(*caller);
                }
            }
        }
    }

    let mut expanded_vec: Vec<NodeId> = expanded.into_iter().collect();
    // Стабильность: сортируем по raw id (детерминированность планов)
    expanded_vec.sort_by_key(|n| n.0);

    let expanded_fraction = expanded_vec.len() as f64 / total_routines as f64;
    if expanded_fraction > heur.max_touched_fraction {
        let mut p = RebuildPlan::full(total_routines); p.fallback_reason = "exp_fraction"; p.initial_touched = touched_initial; p.expanded_touched = expanded_vec.len(); return p;
    }
    if expanded_vec.len() > heur.max_touched_absolute {
        let mut p = RebuildPlan::full(total_routines); p.fallback_reason = "exp_absolute"; p.initial_touched = touched_initial; p.expanded_touched = expanded_vec.len(); return p;
    }
    let expanded_len = expanded_vec.len();
    RebuildPlan { fallback_full: false, routines_to_rebuild: expanded_vec, total_routines, initial_touched: touched_initial, expanded_touched: expanded_len, fallback_reason: "none" }
}

/// Прототип применителя частичной перестройки.
/// Пока НЕ модифицирует существующую арену реально, а просто возвращает ту же ссылку и количество «замен».
/// Дальнейшая реализация будет строить новый BuiltAst с копированием.
pub fn apply_rebuild_plan<'a>(ast: &'a BuiltAst, plan: &RebuildPlan) -> (&'a BuiltAst, usize) {
    if plan.fallback_full { return (ast, 0); }
    // Прототип: считаем что каждая рутина будет «заменена» (позже добавим анализ фактических изменений)
    (ast, plan.routines_to_rebuild.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast_core::{AstBuilder, AstKind};
    use crate::core::position::PackedSpan;

    fn ps(s: u32, l: u32) -> PackedSpan { PackedSpan::new(s,l) }

    #[test]
    fn empty_change_plan() {
        let mut b = AstBuilder::new();
        b.start_node(AstKind::Module, ps(0,100));
        b.start_node_with_ident(AstKind::Procedure, ps(0,20), "P1".into()); b.finish_node();
        b.start_node_with_ident(AstKind::Function, ps(20,20), "F1".into()); b.finish_node();
        b.finish_node();
        let ast = b.build();
        let plan = plan_partial_rebuild(&ast, 200, 210, RebuildHeuristics::default());
        assert!(!plan.fallback_full);
        assert_eq!(plan.routines_to_rebuild.len(), 0);
        assert_eq!(plan.total_routines, 2);
    }

    #[test]
    fn single_routine_plan() {
        let mut b = AstBuilder::new();
        b.start_node(AstKind::Module, ps(0,200));
        b.start_node_with_ident(AstKind::Procedure, ps(0,50), "P1".into()); b.finish_node();
        b.start_node_with_ident(AstKind::Procedure, ps(50,50), "P2".into()); b.finish_node();
        b.start_node_with_ident(AstKind::Function, ps(100,50), "F1".into()); b.finish_node();
        b.finish_node();
        let ast = b.build();
        // Touch inside P2
        let plan = plan_partial_rebuild(&ast, 60, 65, RebuildHeuristics::default());
        assert!(!plan.fallback_full);
        assert_eq!(plan.routines_to_rebuild.len(), 1);
        // Must target the second procedure node (id ordering depends on build sequence)
    }

    #[test]
    fn module_level_change_fallback() {
        let mut b = AstBuilder::new();
        b.start_node(AstKind::Module, ps(0,60));
        b.start_node_with_ident(AstKind::Procedure, ps(0,20), "P1".into()); b.finish_node();
        b.start_node_with_ident(AstKind::Procedure, ps(20,20), "P2".into()); b.finish_node();
        b.finish_node();
        let ast = b.build();
        // Change overlapping multiple routines so boundaries escalate to module (simulate by span covering whole)
        let plan = plan_partial_rebuild(&ast, 0, 60, RebuildHeuristics::default());
        assert!(plan.fallback_full);
    }
}
    #[test]
    fn dependency_expansion_adds_callers() {
        use crate::core::position::PackedSpan; fn ps(s:u32,l:u32)->PackedSpan{PackedSpan::new(s,l)}
        // Two routines: F1 (calls F2) and F2. Change inside F2 should schedule both.
        let mut b = AstBuilder::new();
        b.start_node(AstKind::Module, ps(0,400));
        // F1 span 0-150
        b.start_node_with_ident(AstKind::Function, ps(0,150), "F1".into());
        b.start_call(ps(20,10)); // call at 20..30
        b.leaf_ident(ps(20,2), "F2".into());
        b.finish_call();
        b.finish_node();
        // F2 span 200-150 (start 200 length 150 => 200..350)
    b.start_node_with_ident(AstKind::Function, ps(200,150), "F2".into());
    // Add a leaf inside F2 so dirty range hits a descendant, enabling climb to routine
    b.leaf_literal(ps(210,5), "1".into());
    b.finish_node();
        b.finish_node();
        let ast = b.build();
        // Dirty range inside F2 only (e.g., 210..215)
    let custom_heur = RebuildHeuristics { max_touched_fraction: 1.0, max_touched_absolute: 100 };
    let plan = plan_partial_rebuild(&ast, 210, 215, custom_heur);
    assert!(!plan.fallback_full, "should not fallback full for isolated routine change");
        assert_eq!(plan.total_routines, 2);
        assert_eq!(plan.routines_to_rebuild.len(), 2, "expansion should include caller F1 and callee F2");
    }
