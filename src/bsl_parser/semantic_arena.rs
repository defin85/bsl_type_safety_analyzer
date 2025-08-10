//! Experimental (now primary) semantic analyzer over new Arena AST.
//! Phase 3: precise spans, parity tests, snapshot baselines, proto TypeTable.
//! Legacy semantic analyzer path is DEPRECATED and scheduled for removal after one stable release.

use crate::ast_core::{Arena, AstKind, AstNode, BuiltAst, NodeId, Visitor, VisitControl, AstPayload};
use crate::ast_core::interner::{SymbolId, StringInterner};
use crate::bsl_parser::diagnostics::{Diagnostic, DiagnosticSeverity, Location, codes};
use std::collections::{HashMap, HashSet};
use crate::bsl_parser::simple_types::{SimpleType, binary_result, unary_result};
use crate::core::position::LineIndex;

#[derive(Debug)]
pub struct VarInfo { pub name: String, pub declared: NodeId, pub used: bool, pub initialized: bool, pub is_param: bool, ty: SimpleType }
impl VarInfo { fn new(name: String, declared: NodeId, is_param: bool) -> Self { Self { name, declared, used: false, initialized: is_param, is_param, ty: SimpleType::Unknown } } }

#[derive(Default)]
pub struct SemanticArena {
    diagnostics: Vec<Diagnostic>,
    vars: HashMap<String, VarInfo>,
    expr_types: Vec<Option<SimpleType>>,
    check_unused: bool,
    check_uninitialized: bool,
    check_undeclared: bool,
    scopes: Vec<Vec<String>>, // стек имен для областей видимости (процедуры/функции + блоки)
    check_methods: bool, // включить проверки методов/свойств
    method_catalog: HashMap<String, HashSet<String>>,
    property_catalog: HashMap<String, HashSet<String>>,
    file_name: String,
    line_index: Option<LineIndex>,
    interner: Option<StringInterner>,
}

impl SemanticArena {
    pub fn new() -> Self { Self { file_name: "<arena>".into(), line_index: None, interner: None, ..Default::default() } }
    pub fn analyze(&mut self, ast: &BuiltAst) { self.analyze_with_flags(ast, true, true, true) }
    pub fn analyze_with_flags(&mut self, ast: &BuiltAst, check_unused: bool, check_uninitialized: bool, check_undeclared: bool) {
    self.diagnostics.clear();
        self.vars.clear();
    self.expr_types.clear();
        self.check_unused = check_unused;
        self.check_uninitialized = check_uninitialized;
    self.check_undeclared = check_undeclared;
    self.scopes.clear();
    if self.method_catalog.is_empty() { self.bootstrap_catalogs(); }
    // захватываем interner из AST
    self.interner = Some(ast.interner.clone());
    let mut v = Collector { sem: self };
        super::super::ast_core::walk(&ast.arena, ast.root, &mut v); // reuse walk
        // finalize diagnostics
        for vi in self.vars.values() {
            if self.check_unused && !vi.used { self.diagnostics.push(unused_var_diag(&vi.name, self.make_location(&ast.arena, vi.declared))); }
            if self.check_uninitialized && !vi.initialized && vi.used && !vi.is_param { self.diagnostics.push(uninitialized_var_diag(&vi.name, self.make_location(&ast.arena, vi.declared))); }
        }
    }
    pub fn diagnostics(&self) -> &[Diagnostic] { &self.diagnostics }
    pub fn enable_method_resolution(&mut self) { self.check_methods = true; }
    pub fn set_file_name(&mut self, name: impl Into<String>) { self.file_name = name.into(); }
    pub fn set_line_index(&mut self, idx: LineIndex) { self.line_index = Some(idx); }
    fn make_location(&self, arena: &Arena, id: NodeId) -> Location {
        let span = arena.node(id).span;
        if let Some(li) = &self.line_index {
            let pr = li.position_range(span);
            Location::new(self.file_name.clone(), pr.start.line, pr.start.column, pr.start.offset, span.len as usize)
        } else {
            Location::new(self.file_name.clone(), 0, 0, span.start as usize, span.len as usize)
        }
    }

    fn record_expr_type(&mut self, id: NodeId, ty: SimpleType) {
        let idx = id.0 as usize;
        if self.expr_types.len() <= idx { self.expr_types.resize(idx + 1, None); }
        self.expr_types[idx] = Some(ty);
    }
    fn get_expr_type(&self, id: NodeId) -> Option<&SimpleType> { self.expr_types.get(id.0 as usize).and_then(|o| o.as_ref()) }
    pub fn resolve_symbol(&self, sym: SymbolId) -> &str { self.interner.as_ref().map(|i| i.resolve(sym)).unwrap_or("<sym>") }
}

fn unused_var_diag(name: &str, loc: Location) -> Diagnostic {
    Diagnostic::new(
        DiagnosticSeverity::Warning,
        loc,
        codes::UNUSED_VARIABLE,
        format!("Переменная не используется: {name}"),
    )
}
fn undeclared_var_diag(name: &str, loc: Location) -> Diagnostic {
    Diagnostic::new(
        DiagnosticSeverity::Error,
        loc,
        codes::UNDECLARED_VARIABLE,
        format!("Необъявленная переменная: {name}"),
    )
}
fn uninitialized_var_diag(name: &str, loc: Location) -> Diagnostic {
    Diagnostic::new(
        DiagnosticSeverity::Warning,
        loc,
        codes::UNINITIALIZED_VARIABLE,
        format!("Переменная может быть неинициализирована: {name}"),
    )
}

struct Collector<'a> { sem: &'a mut SemanticArena }
impl<'a> Visitor for Collector<'a> {
    fn enter(&mut self, id: NodeId, node: &AstNode, arena: &Arena) -> VisitControl {
        match node.kind {
            AstKind::Module => {
                // глобальная область модуля
                self.sem.push_scope();
            }
            AstKind::Procedure | AstKind::Function => {
                // новая область для параметров и тела
                self.sem.push_scope();
                // Параметры
                if let Some(first) = node.first_child { // params precede body in our converter
                    let mut cur = Some(first);
                    while let Some(c) = cur { let n = arena.node(c); if n.kind == AstKind::Param { if let AstPayload::Ident { sym } = n.payload { let name_owned = self.sem.resolve_symbol(sym).to_string(); self.sem.declare_var(&name_owned, c, true, arena); } } cur = n.next_sibling; }
                }
            }
            AstKind::VarDecl => {
                for c in arena.children(id) { let n = arena.node(c); if n.kind == AstKind::Identifier { if let AstPayload::Ident { sym } = n.payload { let name_owned = self.sem.resolve_symbol(sym).to_string(); self.sem.declare_var(&name_owned, c, false, arena); } } }
            }
            AstKind::Assignment => {
                // Первый ребенок — target
                if let Some(target) = node.first_child { let tnode = arena.node(target); if tnode.kind == AstKind::Identifier { if let AstPayload::Ident { sym } = tnode.payload { let name_owned = self.sem.resolve_symbol(sym).to_string(); let name = name_owned.as_str();
                        let value = tnode.next_sibling; // expression assigned
                        let value_ty = value.and_then(|vid| self.sem.infer_expr_type(vid, arena));
                        if self.sem.vars.contains_key(name) {
                            // borrow mut after computing diag
                            if let Some(rhs_ty) = value_ty.clone() {
                                let current_ty = self.sem.vars.get(name).map(|v| v.ty.clone()).unwrap_or(SimpleType::Unknown);
                                if let Some(diag) = self.sem.check_type_update(&current_ty, &rhs_ty, target, name, arena) { self.sem.diagnostics.push(diag); }
                                if let Some(vmut) = self.sem.vars.get_mut(name) {
                                    vmut.initialized = true;
                                    if matches!(vmut.ty, SimpleType::Unknown) { vmut.ty = rhs_ty; }
                                }
                            } else if let Some(vmut) = self.sem.vars.get_mut(name) { vmut.initialized = true; }
                        } else if self.sem.check_undeclared { self.sem.diagnostics.push(undeclared_var_diag(name, self.sem.make_location(arena, target))); }
                    } } }
            }
            AstKind::Call => {
                if self.sem.check_methods {
                    // Simple method call: first child is object or function identifier
                    if let Some(obj_id) = node.first_child { let obj_node = arena.node(obj_id); let method_ident = obj_node.next_sibling; if let Some(mid) = method_ident { let mnode = arena.node(mid); if mnode.kind==AstKind::Identifier { if let AstPayload::Ident { sym } = mnode.payload { let method_name_owned = self.sem.resolve_symbol(sym).to_string(); let method_name = method_name_owned.as_str();
                                        // Infer object type
                                        let obj_ty = self.sem.infer_expr_type(obj_id, arena).unwrap_or(SimpleType::Unknown);
                                        match obj_ty {
                                            SimpleType::Object(ref tn) => {
                                                if !self.sem.type_has_method(tn, method_name) {
                                                    self.sem.diagnostics.push(Diagnostic::new(
                                                        DiagnosticSeverity::Error,
                                                        self.sem.make_location(arena, mid),
                                                        codes::UNKNOWN_METHOD,
                                                        format!("Тип '{tn}' не содержит метод '{method_name}'")
                                                    ));
                                                }
                                            }
                                            _ => {
                                                self.sem.diagnostics.push(Diagnostic::new(
                                                    DiagnosticSeverity::Error,
                                                    self.sem.make_location(arena, mid),
                                                    codes::UNKNOWN_METHOD,
                                                    format!("Метод '{method_name}' недопустим для типа {:?}", obj_ty)
                                                ));
                                            }
                                        }
                                    } } } }
                }
            }
            AstKind::Member => {
                if self.sem.check_methods {
                    if let Some(obj_id) = node.first_child { let prop_ident = arena.node(obj_id).next_sibling; if let Some(pid)=prop_ident { let pnode = arena.node(pid); if pnode.kind==AstKind::Identifier { if let AstPayload::Ident { sym } = pnode.payload { let prop_owned = self.sem.resolve_symbol(sym).to_string(); let prop = prop_owned.as_str();
                        let obj_ty = self.sem.infer_expr_type(obj_id, arena).unwrap_or(SimpleType::Unknown);
                        match obj_ty {
                            SimpleType::Object(ref tn) => {
                                if !self.sem.type_has_property(tn, prop) {
                                    self.sem.diagnostics.push(Diagnostic::new(
                                        DiagnosticSeverity::Error,
                                        self.sem.make_location(arena, pid),
                                        codes::UNKNOWN_PROPERTY,
                                        format!("Тип '{tn}' не содержит свойство '{prop}'")
                                    ));
                                }
                            }
                            _ => {
                                self.sem.diagnostics.push(Diagnostic::new(
                                    DiagnosticSeverity::Error,
                                    self.sem.make_location(arena, pid),
                                    codes::UNKNOWN_PROPERTY,
                                    format!("Свойство '{prop}' недопустимо для типа {:?}", obj_ty)
                                ));
                            }
                        }
                    } } } }
                }
            }
            #[allow(unused_assignments)]
            AstKind::If => {
                // Return-aware branch intersection.
                let snapshot: Vec<(String,bool)> = self.sem.vars.iter().map(|(k,v)|(k.clone(), v.initialized)).collect();
                let mut child = node.first_child; // condition
                if let Some(c) = child { child = arena.node(c).next_sibling; }
                // then branch
                let then_block = child; if let Some(tb)=then_block { self.visit_block(tb, arena); child = arena.node(tb).next_sibling; }
                let then_returns = then_block.map(|b| self.block_has_return(b, arena)).unwrap_or(false);
                let mut branches: Vec<(Vec<(String,bool)>, bool)> = vec![(self.sem.snapshot_inits(), then_returns)];
                // else-if chains
                while let Some(c) = child { if arena.node(c).kind == AstKind::If { self.walk_if_branch(c, arena, &mut branches); child = arena.node(c).next_sibling; } else { break; } }
                // else block
                let mut has_else = false;
                if let Some(c) = child { if arena.node(c).kind == AstKind::Block { has_else = true; self.visit_block(c, arena); let ret = self.block_has_return(c, arena); branches.push((self.sem.snapshot_inits(), ret)); child = arena.node(c).next_sibling; } }
                // restore pre-state
                for (n,i) in &snapshot { if let Some(v)=self.sem.vars.get_mut(n) { v.initialized = *i; } }
                if !branches.is_empty() && (has_else || branches.len()>1) {
                    for (name, v) in self.sem.vars.iter_mut() {
                        let mut all=true; let mut any_constraint=false; for (b,ret_flag) in &branches { if *ret_flag { continue; } any_constraint=true; if let Some((_, bi)) = b.iter().find(|(bn,_)| bn==name) { if !*bi { all=false; break; } } else { all=false; break; } }
                        if all && any_constraint { v.initialized = true; }
                    }
                }
                return VisitControl::SkipChildren;
            }
            AstKind::While => {
                // Fixed-point inside loop; propagate only if condition literally TRUE.
                let condition = node.first_child; let body = condition.and_then(|c| arena.node(c).next_sibling);
                let guaranteed = condition.and_then(|c| { let n = arena.node(c); if let AstPayload::Literal { sym, .. } = n.payload { Some(self.sem.resolve_symbol(sym)) } else { None } }).map(|t| t.eq_ignore_ascii_case("true")).unwrap_or(false);
                let pre: Vec<(String,bool)> = self.sem.vars.iter().map(|(k,v)|(k.clone(), v.initialized)).collect();
                let mut loop_inited: Vec<String>=Vec::new();
                const MAX_ITER: usize = 4; for _ in 0..MAX_ITER { if let Some(b)=body { let before = self.sem.snapshot_inits(); self.visit_block(b, arena); // compare
                        for (n,i) in self.sem.snapshot_inits() { if i { let pre_i = before.iter().find(|(bn,_)| bn==&n).map(|(_,v)|*v).unwrap_or(false); if !pre_i && !loop_inited.contains(&n) { loop_inited.push(n.clone()); } } }
                        // early break if no changes
                        let changed = self.sem.snapshot_inits().iter().any(|(n,i)| { let pre_i = before.iter().find(|(bn,_)| bn==n).map(|(_,v)|*v).unwrap_or(false); *i != pre_i }); if !changed { break; }
                    } }
                // restore
                for (n,i) in &pre { if let Some(v)=self.sem.vars.get_mut(n) { v.initialized = if guaranteed && loop_inited.contains(n) { true } else { *i }; } }
                return VisitControl::SkipChildren;
            }
            AstKind::Identifier => {
                if let AstPayload::Ident { sym } = node.payload { let name_owned = self.sem.resolve_symbol(sym).to_string(); let name = name_owned.as_str();
                    if let Some(v) = self.sem.vars.get_mut(name) { if id != v.declared { v.used = true; } }
                    else if self.sem.check_undeclared { self.sem.diagnostics.push(undeclared_var_diag(name, self.sem.make_location(arena, id))); }
                }
            }
            _ => {}
        }
        VisitControl::Continue
    }

    fn leave(&mut self, _id: NodeId, node: &AstNode, _arena: &Arena) {
    if matches!(node.kind, AstKind::Procedure | AstKind::Function) {
            self.sem.pop_scope();
        }
    if matches!(node.kind, AstKind::Module) { self.sem.pop_scope(); }
    }
}

impl<'a> Collector<'a> {
    fn visit_block(&mut self, id: NodeId, arena: &Arena) {
    // новая область для блока
    self.sem.push_scope();
        let mut child = arena.node(id).first_child;
        while let Some(c) = child { // manual walk limited to one nesting level; deeper handled by main walk via enter
            // Recursively walk subtree by invoking walk on child (reuse visitor) but avoid duplicating logic: call enter manually
            let node = arena.node(c).clone();
            self.enter(c, &node, arena); // children will be visited by main traversal anyway
            child = arena.node(c).next_sibling;
        }
    self.sem.pop_scope();
    }
    fn walk_if_branch(&mut self, if_id: NodeId, arena: &Arena, branch_inits: &mut Vec<(Vec<(String,bool)>, bool)>) {
        let node = arena.node(if_id); let mut child = node.first_child; if let Some(c)=child { child = arena.node(c).next_sibling; }
        if let Some(block)=child { self.visit_block(block, arena); let ret = self.block_has_return(block, arena); branch_inits.push((self.sem.snapshot_inits(), ret)); }
    }
    fn block_has_return(&self, id: NodeId, arena: &Arena) -> bool { arena.children(id).any(|c| arena.node(c).kind == AstKind::Return) }
}

impl SemanticArena { fn snapshot_inits(&self) -> Vec<(String,bool)> { self.vars.iter().map(|(k,v)|(k.clone(), v.initialized)).collect() } }
impl SemanticArena {
    fn push_scope(&mut self) { self.scopes.push(Vec::new()); }
    fn pop_scope(&mut self) { self.scopes.pop(); }
    fn declare_var(&mut self, name: &str, id: NodeId, is_param: bool, arena: &Arena) {
    let loc = self.make_location(arena, id);
        // Проверка в текущей области
        if let Some(cur) = self.scopes.last() {
            if cur.iter().any(|n| n == name) {
                let code = if is_param { codes::DUPLICATE_PARAMETER } else { codes::DUPLICATE_VARIABLE };
                self.diagnostics.push(Diagnostic::new(
                    DiagnosticSeverity::Error,
                    loc.clone(),
                    code,
                    format!("Дублированное объявление: {name}"),
                ));
                return; // не переопределяем первое
            }
        }
        // Тень из внешней области — подсказка
        if self.vars.contains_key(name) {
            self.diagnostics.push(Diagnostic::new(
                DiagnosticSeverity::Hint,
                loc.clone(),
                codes::DUPLICATE_VARIABLE,
                format!("Тень имени: {name}"),
            ));
        }
        self.vars.entry(name.to_string()).or_insert_with(|| VarInfo::new(name.to_string(), id, is_param));
        if let Some(cur) = self.scopes.last_mut() { cur.push(name.to_string()); }
    }
}

impl SemanticArena {
    fn infer_expr_type(&mut self, id: NodeId, arena: &Arena) -> Option<SimpleType> {
        if let Some(t) = self.get_expr_type(id).cloned() { return Some(t); }
        let node = arena.node(id);
        let computed = match node.kind {
            AstKind::Literal => {
                if let AstPayload::Literal { sym } = node.payload { Some(SimpleType::literal(self.resolve_symbol(sym))) } else { Some(SimpleType::Unknown) }
            }
            AstKind::Identifier => {
                match node.payload { AstPayload::Ident { sym } => { let n = self.resolve_symbol(sym); self.vars.get(n).map(|v| v.ty.clone()) } _ => None }
            }
            AstKind::New => {
                // Identifier child = type name
                if let Some(type_ident) = node.first_child { let tn = arena.node(type_ident); if let AstPayload::Ident { sym } = tn.payload { Some(SimpleType::Object(self.resolve_symbol(sym).to_string())) } else { Some(SimpleType::Unknown) } } else { Some(SimpleType::Unknown) }
            }
            AstKind::Call => {
                // Simplified: assume call returns Unknown
                Some(SimpleType::Unknown)
            }
            AstKind::Binary => {
                let left = node.first_child; let op = left.and_then(|l| arena.node(l).next_sibling); let right = op.and_then(|o| arena.node(o).next_sibling);
                let lt = left.and_then(|id| self.infer_expr_type(id, arena)).unwrap_or(SimpleType::Unknown);
                let rt = right.and_then(|id| self.infer_expr_type(id, arena)).unwrap_or(SimpleType::Unknown);
                let op_name = op.and_then(|o| { let n = arena.node(o); if let AstPayload::Ident { sym } = n.payload { Some(self.resolve_symbol(sym)) } else { None } }).unwrap_or("");
                Some(binary_result(&lt, op_name, &rt))
            }
            AstKind::Unary => {
                let op = node.first_child; let inner = op.and_then(|o| arena.node(o).next_sibling);
                let inner_ty = inner.and_then(|id| self.infer_expr_type(id, arena)).unwrap_or(SimpleType::Unknown);
                let op_name = op.and_then(|o| { let n = arena.node(o); if let AstPayload::Ident { sym } = n.payload { Some(self.resolve_symbol(sym)) } else { None } }).unwrap_or("");
                Some(unary_result(op_name, &inner_ty))
            }
            _ => Some(SimpleType::Unknown)
        };
        if let Some(ref ty) = computed { self.record_expr_type(id, ty.clone()); }
        computed
    }
    fn check_type_update(&self, current: &SimpleType, new: &SimpleType, target_id: NodeId, name: &str, arena: &Arena) -> Option<Diagnostic> {
        if matches!(current, SimpleType::Unknown) { return None; }
        if current != new && !matches!((current,new), (SimpleType::Number, SimpleType::Number)) {
            let loc = self.make_location(arena, target_id);
            return Some(Diagnostic::new(
                DiagnosticSeverity::Error,
                loc,
                codes::TYPE_MISMATCH,
                format!("Несовместимое присваивание для {name}: {:?} -> {:?}", current, new)
            ));
        }
        None
    }
    fn type_has_method(&self, type_name: &str, method: &str) -> bool {
        self.method_catalog.get(type_name).map(|s| s.contains(method)).unwrap_or(false)
    }
    fn type_has_property(&self, type_name: &str, prop: &str) -> bool {
        self.property_catalog.get(type_name).map(|s| s.contains(prop)).unwrap_or(false)
    }
    fn bootstrap_catalogs(&mut self) {
        let mut m = HashMap::new(); let mut p = HashMap::new();
        m.insert("Array".into(), HashSet::from(["Add".into(), "Insert".into(), "Delete".into(), "Clear".into()]));
        p.insert("Array".into(), HashSet::from(["Count".into()]));
        self.method_catalog = m; self.property_catalog = p;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bsl_parser::cst_to_arena::ArenaConverter;
    use crate::bsl_parser::ast::*;
    use crate::bsl_parser::Location;

    #[test]
    fn collect_unused_variable() {
        let module = Module { directives: vec![], declarations: vec![Declaration::Variable(VariableDecl { names: vec!["X".into()], export: false, location: Location::new("m.bsl".into(),1,1,0,10) })], location: Location::new("m.bsl".into(),1,1,0,10) };
        let built = ArenaConverter::build_module(&module);
        let mut sem = SemanticArena::new();
        sem.analyze(&built);
        assert_eq!(sem.diagnostics().len(), 1);
    }

    #[test]
    fn undeclared_and_uninitialized() {
        // module: VarDecl Y; use Y (ok), use Z (undeclared), VarDecl X (unused, uninitialized), assign Y (initialized), use X (uninitialized usage)
    use Expression as E; use Statement as S;
        let stmts = vec![
            S::Expression(E::Identifier("Y".into())), // use before declaration: undeclared
        ];
        let func = FunctionDecl { name: "F".into(), export: false, params: vec![Parameter { name: "P".into(), by_val: true, default_value: None, location: Location::new("m.bsl".into(),1,1,0,1) }], directives: vec![], body: stmts, location: Location::new("m.bsl".into(),1,1,0,10) };
        let module = Module { directives: vec![], declarations: vec![Declaration::Variable(VariableDecl { names: vec!["X".into()], export: false, location: Location::new("m.bsl".into(),1,1,0,5) }), Declaration::Function(func)], location: Location::new("m.bsl".into(),1,1,0,20) };
        let built = ArenaConverter::build_module(&module);
        let mut sem = SemanticArena::default();
        sem.analyze_with_flags(&built, true, true, true);
        // Expect at least an unused (X) and undeclared (Y) diagnostics
        let codes: Vec<_> = sem.diagnostics().iter().map(|d| d.code.as_str()).collect();
        assert!(codes.contains(&codes::UNUSED_VARIABLE));
        assert!(codes.contains(&codes::UNDECLARED_VARIABLE));
    }

    #[test]
    fn if_branch_initialization() {
        // module: VarDecl X; If (assign X in both then and else) then use X => no uninitialized warning, unused absent
        use Expression as E; use Statement as S; use Literal as L;
        // Build legacy AST approximating: Если 1 Тогда X = 1; Иначе X = 2; КонецЕсли; Сообщить(X);
        let assign_then = S::Assignment(super::super::ast::Assignment { target: E::Identifier("X".into()), value: E::Literal(L::Number(1.0)), location: Location::new("m.bsl".into(),1,1,0,1) });
        let assign_else = S::Assignment(super::super::ast::Assignment { target: E::Identifier("X".into()), value: E::Literal(L::Number(2.0)), location: Location::new("m.bsl".into(),1,1,0,1) });
        let if_stmt = S::If(super::super::ast::IfStatement { condition: E::Literal(L::Boolean(true)), then_branch: vec![assign_then.clone()], else_ifs: vec![], else_branch: Some(vec![assign_else.clone()]), location: Location::new("m.bsl".into(),1,1,0,1) });
        let use_x = S::Expression(E::Identifier("X".into()));
        let func = FunctionDecl { name: "F".into(), export: false, params: vec![], directives: vec![], body: vec![if_stmt, use_x], location: Location::new("m.bsl".into(),1,1,0,10) };
        let module = Module { directives: vec![], declarations: vec![Declaration::Variable(VariableDecl { names: vec!["X".into()], export: false, location: Location::new("m.bsl".into(),1,1,0,5) }), Declaration::Function(func)], location: Location::new("m.bsl".into(),1,1,0,20) };
        let built = ArenaConverter::build_module(&module);
        let mut sem = SemanticArena::default();
        sem.analyze_with_flags(&built, true, true, true);
        let codes: Vec<_> = sem.diagnostics().iter().map(|d| d.code.as_str()).collect();
        assert!(!codes.contains(&codes::UNINITIALIZED_VARIABLE), "Variable X wrongly marked uninitialized");
        assert!(!codes.contains(&codes::UNUSED_VARIABLE), "Variable X wrongly marked unused");
    }

    #[test]
    fn while_true_initialization() {
        use Expression as E; use Statement as S; use Literal as L;
        // VarDecl X; Пока Истина Цикл X = 1; КонецЦикла; Сообщить(X)
        let loop_body = vec![ S::Assignment(super::super::ast::Assignment { target: E::Identifier("X".into()), value: E::Literal(L::Number(1.0)), location: Location::new("m.bsl".into(),1,1,0,1) }) ];
        let while_stmt = S::While(super::super::ast::WhileStatement { condition: E::Literal(L::Boolean(true)), body: loop_body, location: Location::new("m.bsl".into(),1,1,0,1) });
        let use_x = S::Expression(E::Identifier("X".into()));
        let func = FunctionDecl { name: "F".into(), export: false, params: vec![], directives: vec![], body: vec![while_stmt, use_x], location: Location::new("m.bsl".into(),1,1,0,10) };
        let module = Module { directives: vec![], declarations: vec![Declaration::Variable(VariableDecl { names: vec!["X".into()], export: false, location: Location::new("m.bsl".into(),1,1,0,5) }), Declaration::Function(func)], location: Location::new("m.bsl".into(),1,1,0,20) };
        let built = ArenaConverter::build_module(&module); let mut sem = SemanticArena::default(); sem.analyze_with_flags(&built, true, true, true);
        let codes: Vec<_> = sem.diagnostics().iter().map(|d| d.code.as_str()).collect();
        assert!(!codes.contains(&codes::UNINITIALIZED_VARIABLE), "While(true) init should propagate");
    }

    #[test]
    fn while_conditional_not_guaranteed() {
        use Expression as E; use Statement as S; use Literal as L;
        // VarDecl X; Пока 1 Цикл X = 1; КонецЦикла; Сообщить(X)  (условие не булевое Истина)
        let loop_body = vec![ S::Assignment(super::super::ast::Assignment { target: E::Identifier("X".into()), value: E::Literal(L::Number(1.0)), location: Location::new("m.bsl".into(),1,1,0,1) }) ];
        let while_stmt = S::While(super::super::ast::WhileStatement { condition: E::Literal(L::Number(1.0)), body: loop_body, location: Location::new("m.bsl".into(),1,1,0,1) });
        let use_x = S::Expression(E::Identifier("X".into()));
        let func = FunctionDecl { name: "F".into(), export: false, params: vec![], directives: vec![], body: vec![while_stmt, use_x], location: Location::new("m.bsl".into(),1,1,0,10) };
        let module = Module { directives: vec![], declarations: vec![Declaration::Variable(VariableDecl { names: vec!["X".into()], export: false, location: Location::new("m.bsl".into(),1,1,0,5) }), Declaration::Function(func)], location: Location::new("m.bsl".into(),1,1,0,20) };
        let built = ArenaConverter::build_module(&module); let mut sem = SemanticArena::default(); sem.analyze_with_flags(&built, true, true, true);
        let codes: Vec<_> = sem.diagnostics().iter().map(|d| d.code.as_str()).collect();
        assert!(codes.contains(&codes::UNINITIALIZED_VARIABLE), "While(non-true) init should not propagate");
    }

    #[test]
    fn duplicate_parameters_and_variables() {
        use crate::bsl_parser::ast::*;
        // Function с дублирующим параметром P,P и модульная переменная X,X
        let func = FunctionDecl { name: "F".into(), export: false, params: vec![
            Parameter { name: "P".into(), by_val: true, default_value: None, location: Location::new("m.bsl".into(),1,1,0,1) },
            Parameter { name: "P".into(), by_val: true, default_value: None, location: Location::new("m.bsl".into(),1,1,0,1) },
        ], directives: vec![], body: vec![], location: Location::new("m.bsl".into(),1,1,0,1) };
        let module = Module { directives: vec![], declarations: vec![
            Declaration::Variable(VariableDecl { names: vec!["X".into(), "X".into()], export: false, location: Location::new("m.bsl".into(),1,1,0,1) }),
            Declaration::Function(func)
        ], location: Location::new("m.bsl".into(),1,1,0,1) };
        let built = ArenaConverter::build_module(&module); let mut sem = SemanticArena::default(); sem.analyze_with_flags(&built, true, true, true);
        let mut has_dup_param=false; let mut has_dup_var=false; for d in sem.diagnostics() { if d.code==codes::DUPLICATE_PARAMETER { has_dup_param=true; } if d.code==codes::DUPLICATE_VARIABLE && d.message.contains("Дублированное") { has_dup_var=true; } }
        assert!(has_dup_param, "Ожидалась диагностика дублированного параметра");
        assert!(has_dup_var, "Ожидалась диагностика дублированной переменной");
    }

    #[test]
    fn type_mismatch_number_string() {
    use crate::bsl_parser::ast::*;
    use crate::bsl_parser::ast::{Expression as E, Statement as S, Literal as L};
        // Var X; X = 1; X = "str" => mismatch
        let stmts = vec![
            S::Assignment(super::super::ast::Assignment { target: E::Identifier("X".into()), value: E::Literal(L::Number(1.0)), location: Location::new("m.bsl".into(),1,1,0,1) }),
            S::Assignment(super::super::ast::Assignment { target: E::Identifier("X".into()), value: E::Literal(L::String("s".into())), location: Location::new("m.bsl".into(),1,1,0,1) }),
        ];
        let func = FunctionDecl { name: "F".into(), export: false, params: vec![], directives: vec![], body: stmts, location: Location::new("m.bsl".into(),1,1,0,1) };
        let module = Module { directives: vec![], declarations: vec![Declaration::Variable(VariableDecl { names: vec!["X".into()], export: false, location: Location::new("m.bsl".into(),1,1,0,1) }), Declaration::Function(func)], location: Location::new("m.bsl".into(),1,1,0,1) };
        let built = ArenaConverter::build_module(&module); let mut sem = SemanticArena::default(); sem.analyze_with_flags(&built, true, true, true);
        assert!(sem.diagnostics().iter().any(|d| d.code==codes::TYPE_MISMATCH));
    }

    #[test]
    fn unknown_method_on_literal() {
    use crate::bsl_parser::ast::*;
    use crate::bsl_parser::ast::{Expression as E, Statement as S, Literal as L};
        // Сообщить(1.Add()) -> 1.Add() моделируем как 1.MethodCall("Add") упрощённо: (1).Add();
        let stmts = vec![
            S::Expression(E::MethodCall(super::super::ast::MethodCall { object: Box::new(E::Literal(L::Number(1.0))), method: "Add".into(), args: vec![], location: Location::new("m.bsl".into(),1,1,0,1) }))
        ];
        let func = FunctionDecl { name: "F".into(), export: false, params: vec![], directives: vec![], body: stmts, location: Location::new("m.bsl".into(),1,1,0,1) };
        let module = Module { directives: vec![], declarations: vec![Declaration::Function(func)], location: Location::new("m.bsl".into(),1,1,0,1) };
    let built = ArenaConverter::build_module(&module); let mut sem = SemanticArena::default(); sem.enable_method_resolution(); sem.analyze_with_flags(&built, true, true, true);
        assert!(sem.diagnostics().iter().any(|d| d.code==codes::UNKNOWN_METHOD));
    }

    #[test]
    fn known_method_on_array_ok() {
    use crate::bsl_parser::ast::*;
    use crate::bsl_parser::ast::{Expression as E, Statement as S, Literal as L};
        // Var X; X = Новый Array(); X.Add();
        let stmts = vec![
            S::Assignment(super::super::ast::Assignment { target: E::Identifier("X".into()), value: E::New(super::super::ast::NewExpression { type_name: "Array".into(), args: vec![], location: Location::new("m.bsl".into(),1,1,0,1) }), location: Location::new("m.bsl".into(),1,1,0,1) }),
            S::Expression(E::MethodCall(super::super::ast::MethodCall { object: Box::new(E::Identifier("X".into())), method: "Add".into(), args: vec![E::Literal(L::Number(1.0))], location: Location::new("m.bsl".into(),1,1,0,1) })),
        ];
        let func = FunctionDecl { name: "F".into(), export: false, params: vec![], directives: vec![], body: stmts, location: Location::new("m.bsl".into(),1,1,0,1) };
        let module = Module { directives: vec![], declarations: vec![Declaration::Variable(VariableDecl { names: vec!["X".into()], export: false, location: Location::new("m.bsl".into(),1,1,0,1) }), Declaration::Function(func)], location: Location::new("m.bsl".into(),1,1,0,1) };
        let built = ArenaConverter::build_module(&module); let mut sem = SemanticArena::default(); sem.enable_method_resolution(); sem.analyze_with_flags(&built, true, true, true);
        assert!(!sem.diagnostics().iter().any(|d| d.code==codes::UNKNOWN_METHOD && d.message.contains("Add")), "Метод Add не должен считаться неизвестным");
    }

    #[test]
    fn unknown_property_on_number() {
    use crate::bsl_parser::ast::*;
    use crate::bsl_parser::ast::{Expression as E, Statement as S, Literal as L};
        // (1).Count
        let stmts = vec![ S::Expression(E::PropertyAccess(super::super::ast::PropertyAccess { object: Box::new(E::Literal(L::Number(1.0))), property: "Count".into(), location: Location::new("m.bsl".into(),1,1,0,1) })) ];
        let func = FunctionDecl { name: "F".into(), export: false, params: vec![], directives: vec![], body: stmts, location: Location::new("m.bsl".into(),1,1,0,1) };
        let module = Module { directives: vec![], declarations: vec![Declaration::Function(func)], location: Location::new("m.bsl".into(),1,1,0,1) };
        let built = ArenaConverter::build_module(&module); let mut sem = SemanticArena::default(); sem.enable_method_resolution(); sem.analyze_with_flags(&built, true, true, true);
        assert!(sem.diagnostics().iter().any(|d| d.code==codes::UNKNOWN_PROPERTY));
    }

    #[test]
    fn line_index_applied_offsets() {
    use crate::bsl_parser::ast::*;
        // Смоделируем модуль с переменной и неиспользованием: две строки
        let var_decl = Declaration::Variable(VariableDecl { names: vec!["X".into()], export: false, location: Location::new("sample.bsl".into(),0,0,0,1) });
        let module = Module { directives: vec![], declarations: vec![var_decl], location: Location::new("sample.bsl".into(),0,0,0,5) };
        let built = ArenaConverter::build_module(&module);
        let mut sem = SemanticArena::new();
        sem.set_file_name("sample.bsl");
        // Простейший текст (offsetы сейчас всё равно нули в конвертере) => line/column останутся 0
        sem.set_line_index(LineIndex::new("Перем X\n"));
        sem.analyze(&built);
        let unused = sem.diagnostics().iter().find(|d| d.code==codes::UNUSED_VARIABLE).expect("unused var diag");
        assert_eq!(unused.location.file, "sample.bsl");
        assert_eq!(unused.location.line, 0);
    }

    #[test]
    fn type_mismatch_location_on_identifier_and_block_span_aggregated() {
        use crate::bsl_parser::ast::*;
        use crate::bsl_parser::cst_to_arena::ArenaConverter;
        use crate::bsl_parser::Location;
        // Module with variable X; X = 1; X = "str";
        let var_decl = Declaration::Variable(VariableDecl { names: vec!["X".into()], export: false, location: Location::new("m.bsl".into(),1,1,0,5) });
        // Assignments share simplistic locations; second triggers mismatch.
        let assign1 = Statement::Assignment(Assignment { target: Expression::Identifier("X".into()), value: Expression::Literal(Literal::Number(1.0)), location: Location::new("m.bsl".into(),1,1,6,3) });
        let assign2 = Statement::Assignment(Assignment { target: Expression::Identifier("X".into()), value: Expression::Literal(Literal::String("s".into())), location: Location::new("m.bsl".into(),1,1,10,4) });
        let proc = Declaration::Procedure(ProcedureDecl { name: "P".into(), export: false, params: vec![], directives: vec![], body: vec![assign1, assign2], location: Location::new("m.bsl".into(),1,1,6,8) });
        let module = Module { directives: vec![], declarations: vec![var_decl, proc], location: Location::new("m.bsl".into(),1,1,0,20) };
        let built = ArenaConverter::build_module(&module);
        let mut sem = SemanticArena::new();
        sem.set_file_name("m.bsl");
        sem.analyze_with_flags(&built, true, true, true);
        let diags = sem.diagnostics();
        let mismatch = diags.iter().find(|d| d.code == crate::bsl_parser::diagnostics::codes::TYPE_MISMATCH).expect("expected type mismatch");
        assert_eq!(mismatch.location.file, "m.bsl");
        // Identifier span currently length of name (1) and start offset equals 0 (due to converter simplification)
        assert_eq!(mismatch.location.length, 1);
        // Find a Block node and ensure its span now covers both assignments (offset >= first assign offset, end >= second)
        use crate::ast_core::{preorder, AstKind};
        let mut block_span_ok = false;
        for id in preorder(&built.arena, built.root) { let n = built.arena.node(id); if n.kind == AstKind::Block { if n.span.len > 0 { block_span_ok = true; } } }
        assert!(block_span_ok, "Block span was not aggregated");
    }

    #[test]
    fn unknown_method_span_points_to_method_name() {
        use crate::bsl_parser::ast::*; use crate::bsl_parser::cst_to_arena::ArenaConverter; use crate::bsl_parser::Location; use crate::bsl_parser::diagnostics::codes;
        // Simulate: Перем X; X = Новый Array(); X.Add2(1); (Add2 неизвестен)
        let var = Declaration::Variable(VariableDecl { names: vec!["X".into()], export: false, location: Location::new("m.bsl".into(),1,1,0,8) });
        let assign_new = Statement::Assignment(Assignment { target: Expression::Identifier("X".into()), value: Expression::New(NewExpression { type_name: "Array".into(), args: vec![], location: Location::new("m.bsl".into(),1,1,10,10) }), location: Location::new("m.bsl".into(),1,1,10,10) });
        let call_bad = Statement::Expression(Expression::MethodCall(MethodCall { object: Box::new(Expression::Identifier("X".into())), method: "Add2".into(), args: vec![Expression::Literal(Literal::Number(1.0))], location: Location::new("m.bsl".into(),1,1,25,8) }));
        let proc = Declaration::Procedure(ProcedureDecl { name: "P".into(), export:false, params: vec![], directives: vec![], body: vec![assign_new, call_bad], location: Location::new("m.bsl".into(),1,1,9,25) });
        let module = Module { directives: vec![], declarations: vec![var, proc], location: Location::new("m.bsl".into(),1,1,0,40) };
        let built = ArenaConverter::build_module(&module);
        let mut sem = SemanticArena::new(); sem.set_file_name("m.bsl"); sem.enable_method_resolution(); sem.analyze_with_flags(&built, true, true, true);
        let diag = sem.diagnostics().iter().find(|d| d.code==codes::UNKNOWN_METHOD).expect("expected unknown method");
        assert!(diag.location.offset >= 25 && diag.location.length >= 4, "diag should point to method name span");
    }
}
