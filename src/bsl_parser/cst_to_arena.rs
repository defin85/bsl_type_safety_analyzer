//! Временный конвертер упрощённого BslAst (текущая заглушка парсера) в новый ast_core Arena.
//! После внедрения полноценного tree-sitter обхода будет заменён прямым построением.

use crate::ast_core::{AstBuilder, AstKind, AstPayload, BuiltAst};
use crate::core::position::PackedSpan;
use crate::bsl_parser::ast::*;

fn ps(start: u32, len: u32) -> PackedSpan { PackedSpan::new(start, len) }
fn from_loc(offset: usize, length: usize) -> PackedSpan { PackedSpan::new(offset as u32, length as u32) }

pub struct ArenaConverter;

impl ArenaConverter {
    pub fn build_module(module: &Module) -> BuiltAst {
        let mut b = AstBuilder::new();
    b.start_node(AstKind::Module, from_loc(module.location.offset, module.location.length));
        for decl in &module.declarations {
            match decl {
                Declaration::Procedure(p) => convert_procedure(&mut b, p),
                Declaration::Function(f) => convert_function(&mut b, f),
                Declaration::Variable(v) => convert_variable(&mut b, v),
            }
        }
        b.finish_node();
        b.build()
    }
}

fn convert_params(b: &mut AstBuilder, params: &[Parameter]) {
    for p in params {
    // Используем реальный span параметра
    let span = from_loc(p.location.offset, p.location.length.max(p.name.len()));
    let sym = b.intern_symbol(&p.name);
    b.leaf(AstKind::Param, span, AstPayload::Ident { sym });
    }
}

fn convert_body_statements(b: &mut AstBuilder, stmts: &[Statement]) {
    if stmts.is_empty() { return; }
    b.start_node(AstKind::Block, ps(0,0));
    for s in stmts { convert_statement(b, s); }
    b.finish_node();
}

fn convert_statement(b: &mut AstBuilder, stmt: &Statement) {
    match stmt {
        Statement::Expression(expr) => convert_expression(b, expr),
        Statement::Assignment(a) => {
            b.start_node(AstKind::Assignment, from_loc(a.location.offset, a.location.length));
            convert_expression(b, &a.target);
            convert_expression(b, &a.value);
            b.finish_node();
        }
        Statement::If(i) => {
            b.start_node(AstKind::If, from_loc(i.location.offset, i.location.length));
            // condition
            convert_expression(b, &i.condition);
            // then branch as Block
            b.start_node(AstKind::Block, ps(0,0));
            for s in &i.then_branch { convert_statement(b, s); }
            b.finish_node();
            // else-if pairs: each as If node nested (simplified)
            for ei in &i.else_ifs {
                b.start_node(AstKind::If, from_loc(ei.location.offset, ei.location.length));
                convert_expression(b, &ei.condition);
                b.start_node(AstKind::Block, ps(0,0));
                for s in &ei.body { convert_statement(b, s); }
                b.finish_node();
                b.finish_node();
            }
            // else branch
            if let Some(else_b) = &i.else_branch {
                b.start_node(AstKind::Block, ps(0,0));
                for s in else_b { convert_statement(b, s); }
                b.finish_node();
            }
            b.finish_node();
        }
        Statement::Return(r) => {
            b.start_node(AstKind::Return, from_loc(r.location.offset, r.location.length));
            if let Some(e) = &r.value { convert_expression(b, e); }
            b.finish_node();
        }
        Statement::While(w) => {
            b.start_node(AstKind::While, from_loc(w.location.offset, w.location.length));
            convert_expression(b, &w.condition);
            b.start_node(AstKind::Block, ps(0,0));
            for s in &w.body { convert_statement(b, s); }
            b.finish_node();
            b.finish_node();
        }
        _ => { b.error(ps(0,0), "unsupported statement kind in transitional converter"); }
    }
}

fn convert_expression(b: &mut AstBuilder, expr: &Expression) {
    match expr {
        Expression::Literal(l) => {
            let lit_text = match l { Literal::Number(n)=>n.to_string(), Literal::String(s)=>s.clone(), Literal::Boolean(bv)=>bv.to_string(), Literal::Date(d)=>d.clone(), Literal::Undefined => "Undefined".into(), Literal::Null => "Null".into() };
            b.leaf_literal(ps(0, lit_text.len() as u32), lit_text);
        }
        Expression::Identifier(name) => {
            b.leaf_ident(ps(0, name.len() as u32), name.clone());
        }
        Expression::MethodCall(mc) => {
            b.start_node(AstKind::Call, from_loc(mc.location.offset, mc.location.length));
            // object
            convert_expression(b, &mc.object);
            // Эвристика: имя метода в конце диапазона
            let m_off = mc.location.offset + mc.location.length.saturating_sub(mc.method.len());
            b.leaf_ident(from_loc(m_off, mc.method.len()), mc.method.clone());
            for a in &mc.args { convert_expression(b, a); }
            b.finish_node();
        }
        Expression::FunctionCall(fc) => {
            b.start_node(AstKind::Call, from_loc(fc.location.offset, fc.location.length));
            // Предполагаем что имя функции в начале
            b.leaf_ident(from_loc(fc.location.offset, fc.name.len()), fc.name.clone());
            for a in &fc.args { convert_expression(b, a); }
            b.finish_node();
        }
        Expression::PropertyAccess(pa) => {
            b.start_node(AstKind::Member, from_loc(pa.location.offset, pa.location.length));
            convert_expression(b, &pa.object);
            let p_off = pa.location.offset + pa.location.length.saturating_sub(pa.property.len());
            b.leaf_ident(from_loc(p_off, pa.property.len()), pa.property.clone());
            b.finish_node();
        }
        Expression::New(ne) => {
            b.start_node(AstKind::New, from_loc(ne.location.offset, ne.location.length));
            // Смещаем имя типа после ключевого слова 'Новый ' или 'New ' если длина совпадает с эвристикой.
            // Формула: keyword_len = total_len - type_name.len() - 2 (скобки). Если keyword_len == 6 ("Новый ") или 4 ("New "), используем его.
            let mut ty_off = ne.location.offset;
            if ne.location.length >= ne.type_name.len() + 2 {
                let kw_len = ne.location.length.saturating_sub(ne.type_name.len() + 2);
                if kw_len == 6 || kw_len == 4 { ty_off = ne.location.offset + kw_len; }
            }
            b.leaf_ident(from_loc(ty_off, ne.type_name.len()), ne.type_name.clone());
            for a in &ne.args { convert_expression(b, a); }
            b.finish_node();
        }
        Expression::Binary(bin) => {
            b.start_node(AstKind::Binary, from_loc(bin.location.offset, bin.location.length));
            convert_expression(b, &bin.left);
            // Фиксируем оператор: размещаем символ(ы) непосредственно после левого операнда.
            let op_text = match bin.op {
                BinaryOperator::Add => "+",
                BinaryOperator::Subtract => "-",
                BinaryOperator::Multiply => "*",
                BinaryOperator::Divide => "/",
                BinaryOperator::Modulo => "%",
                BinaryOperator::Equal => "=",
                BinaryOperator::NotEqual => "<>",
                BinaryOperator::Less => "<",
                BinaryOperator::Greater => ">",
                BinaryOperator::LessOrEqual => "<=",
                BinaryOperator::GreaterOrEqual => ">=",
                BinaryOperator::And => "И",
                BinaryOperator::Or => "ИЛИ",
            };
            // Пока нет реальных offset'ов под-выражений, ставим оператор в начало span бинарного выражения.
            // При появлении реальных offset у left/right нужно будет вычислить: left_end_offset + пробелы.
            b.leaf_ident(from_loc(bin.location.offset, op_text.len()), op_text.into());
            convert_expression(b, &bin.right);
            b.finish_node();
        }
        Expression::Unary(un) => {
            b.start_node(AstKind::Unary, from_loc(un.location.offset, un.location.length));
            let op_text = match un.op { UnaryOperator::Not => "НЕ", UnaryOperator::Minus => "-" };
            b.leaf_ident(from_loc(un.location.offset, op_text.len()), op_text.into());
            convert_expression(b, &un.operand);
            b.finish_node();
        }
        _ => { b.error(ps(0,0), "unsupported expression kind in transitional converter"); }
    }
}

fn convert_procedure(b: &mut AstBuilder, p: &ProcedureDecl) {
    b.start_node_with_ident(AstKind::Procedure, from_loc(p.location.offset, p.location.length), p.name.clone());
    convert_params(b, &p.params);
    convert_body_statements(b, &p.body);
    b.finish_node();
}

fn convert_function(b: &mut AstBuilder, f: &FunctionDecl) {
    b.start_node_with_ident(AstKind::Function, from_loc(f.location.offset, f.location.length), f.name.clone());
    convert_params(b, &f.params);
    convert_body_statements(b, &f.body);
    b.finish_node();
}

fn convert_variable(b: &mut AstBuilder, v: &VariableDecl) {
    b.start_node(AstKind::VarDecl, from_loc(v.location.offset, v.location.length));
    // Эвристическое распределение имён внутри span объявления
    let mut cursor = v.location.offset;
    for (i, name) in v.names.iter().enumerate() {
        let start = cursor;
        let len = name.len();
    b.leaf_ident(from_loc(start, len), name.clone());
        // предполагаем разделитель (запятая/пробел) между именами ~1-2 символа
        cursor += len + if i + 1 < v.names.len() { 2 } else { 0 };
    }
    b.finish_node();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bsl_parser::Location;

    #[test]
    fn convert_simple_function() {
        let module = Module { directives: vec![], declarations: vec![Declaration::Function(FunctionDecl { name: "F".into(), export: false, params: vec![], directives: vec![], body: vec![], location: Location::new("f.bsl".into(),1,1,0,10) })], location: Location::new("f.bsl".into(),1,1,0,10) };
        let built = ArenaConverter::build_module(&module);
        assert!(built.arena.len() >= 2); // Module + Function
    }

    #[test]
    fn variable_identifier_has_nonzero_offset() {
        use crate::bsl_parser::ast::*;
        use crate::bsl_parser::Location;
        let var = Declaration::Variable(VariableDecl { names: vec!["A".into(), "B".into()], export: false, location: Location::new("f.bsl".into(),1,1,5,10) });
        let module = Module { directives: vec![], declarations: vec![var], location: Location::new("f.bsl".into(),1,1,0,20) };
        let built = ArenaConverter::build_module(&module);
        // Find Identifier nodes and ensure start != 0
        use crate::ast_core::{preorder, AstKind};
        let mut found_nonzero = false;
        for id in preorder(&built.arena, built.root) { let n = built.arena.node(id); if n.kind==AstKind::Identifier { if n.span.start != 0 { found_nonzero = true; } } }
        assert!(found_nonzero, "Expected at least one identifier with non-zero start offset");
    }

    #[test]
    fn new_expression_type_offset_after_keyword() {
        use crate::bsl_parser::ast::*;
        // Симулируем: Новый Array()
        // location.length = 6 (Новый ) + 5 (Array) + 2 (()) = 13
        let new_expr = Expression::New(NewExpression { type_name: "Array".into(), args: vec![], location: Location::new("f.bsl".into(),1,1,10,13) });
        let assign = Statement::Assignment(Assignment { target: Expression::Identifier("X".into()), value: new_expr, location: Location::new("f.bsl".into(),1,1,10,13) });
        let var = Declaration::Variable(VariableDecl { names: vec!["X".into()], export: false, location: Location::new("f.bsl".into(),1,1,0,5) });
        let module = Module { directives: vec![], declarations: vec![var, Declaration::Procedure(ProcedureDecl { name: "P".into(), export: false, params: vec![], directives: vec![], body: vec![assign], location: Location::new("f.bsl".into(),1,1,0,30) })], location: Location::new("f.bsl".into(),1,1,0,40) };
        let built = ArenaConverter::build_module(&module);
        use crate::ast_core::{preorder, AstKind};
        // Найдём Identifier внутри New узла и проверим что start == offset_new + 6
        let mut ok = false;
        for id in preorder(&built.arena, built.root) {
            let n = built.arena.node(id);
            if n.kind==AstKind::New {
                let new_start = n.span.start;
                // Итерация по детям через first_child / next_sibling
                let mut child_opt = n.first_child;
                while let Some(cid) = child_opt {
                    let cn = built.arena.node(cid);
                    if cn.kind==AstKind::Identifier && cn.span.start == new_start + 6 { ok = true; }
                    child_opt = cn.next_sibling;
                }
            }
        }
        assert!(ok, "Type name in New expression not offset after keyword");
    }

    #[test]
    fn binary_operator_identifier_span_fixed() {
        use crate::bsl_parser::ast::*;
        let bin = Expression::Binary(BinaryOp { left: Box::new(Expression::Identifier("A".into())), op: BinaryOperator::Add, right: Box::new(Expression::Identifier("B".into())), location: Location::new("f.bsl".into(),1,1,20,5) });
        let stmt = Statement::Expression(bin);
        let module = Module { directives: vec![], declarations: vec![Declaration::Procedure(ProcedureDecl { name: "P".into(), export: false, params: vec![], directives: vec![], body: vec![stmt], location: Location::new("f.bsl".into(),1,1,0,40) })], location: Location::new("f.bsl".into(),1,1,0,50) };
        let built = ArenaConverter::build_module(&module);
        use crate::ast_core::{preorder, AstKind};
        let mut found_plus = false;
    for id in preorder(&built.arena, built.root) { let n = built.arena.node(id); if n.kind==AstKind::Identifier { if let AstPayload::Ident { sym } = &n.payload { let s = built.interner.resolve(*sym); if s=="+" && n.span.start==20 { found_plus = true; } } } }
        assert!(found_plus, "'+' operator identifier not found at expected offset");
    }

    #[test]
    fn unary_operator_identifier_span_fixed() {
        use crate::bsl_parser::ast::*;
        let un = Expression::Unary(UnaryOp { op: UnaryOperator::Minus, operand: Box::new(Expression::Identifier("X".into())), location: Location::new("f.bsl".into(),1,1,30,3) });
        let stmt = Statement::Expression(un);
        let module = Module { directives: vec![], declarations: vec![Declaration::Procedure(ProcedureDecl { name: "P".into(), export: false, params: vec![], directives: vec![], body: vec![stmt], location: Location::new("f.bsl".into(),1,1,0,40) })], location: Location::new("f.bsl".into(),1,1,0,50) };
        let built = ArenaConverter::build_module(&module);
        use crate::ast_core::{preorder, AstKind};
        let mut found_minus = false;
    for id in preorder(&built.arena, built.root) { let n = built.arena.node(id); if n.kind==AstKind::Identifier { if let AstPayload::Ident { sym } = &n.payload { let s = built.interner.resolve(*sym); if s=="-" && n.span.start==30 { found_minus = true; } } } }
        assert!(found_minus, "'-' operator identifier not found at expected offset");
    }
}
