use bsl_analyzer::ast_core::{AstBuilder, AstKind};
use bsl_analyzer::core::position::PackedSpan;

fn build_simple_proc(name: &str) -> bsl_analyzer::ast_core::BuiltAst {
    let mut b = AstBuilder::new();
    // Module
    b.start_node(AstKind::Module, PackedSpan::new(0, 0));
    // Procedure
    b.start_node(AstKind::Procedure, PackedSpan::new(0, 0));
    // Identifier (procedure name)
    b.leaf_ident(PackedSpan::new(0, name.len() as u32), name.to_string());
    // Body block
    b.start_node(AstKind::Block, PackedSpan::new(0, 0));
    b.finish_node(); // Block
    b.finish_node(); // Procedure
    b.finish_node(); // Module
    b.build()
}

#[test]
fn fingerprint_stable_for_identical_ast() {
    let a1 = build_simple_proc("Test");
    let a2 = build_simple_proc("Test");
    assert_eq!(a1.root_fingerprint(), a2.root_fingerprint(), "Root fingerprints must match for identical ASTs");
    let diff = a1.diff_changed_nodes(&a2).expect("same size");
    assert!(diff.is_empty(), "No nodes should differ for identical ASTs, got {:?}", diff);
}

#[test]
fn fingerprint_diff_detects_identifier_change() {
    let a1 = build_simple_proc("Test");
    let a2 = build_simple_proc("Test2"); // changed name → different interned symbol
    assert_ne!(a1.root_fingerprint(), a2.root_fingerprint(), "Root fingerprint must change when identifier changes");
    let diff = a1.diff_changed_nodes(&a2).expect("same size");
    assert!(!diff.is_empty(), "Changed identifier should produce non-empty diff");
    // Корень должен войти из-за каскада изменения вверх по дереву
    assert!(diff.contains(&a1.root()), "Root id should be in diff due to propagated fingerprint change");
}
