use bsl_analyzer::ast_core::{AstBuilder, AstKind};
use bsl_analyzer::core::position::PackedSpan;

fn sample_module() -> bsl_analyzer::ast_core::BuiltAst {
    // Module
    let mut b = AstBuilder::new();
    b.start_node(AstKind::Module, PackedSpan::new(0, 50));
    // Procedure 1 span 0..20
    b.start_node(AstKind::Procedure, PackedSpan::new(0, 20));
    b.leaf_ident(PackedSpan::new(0, 4), "Proc1".into());
    b.start_node(AstKind::Block, PackedSpan::new(5, 10));
    b.finish_node(); // block
    b.finish_node(); // proc1
    // Procedure 2 span 21..50
    b.start_node(AstKind::Procedure, PackedSpan::new(21, 29));
    b.leaf_ident(PackedSpan::new(21, 5), "Proc2".into());
    b.start_node(AstKind::Block, PackedSpan::new(27, 15));
    b.finish_node();
    b.finish_node(); // proc2
    b.finish_node(); // module
    b.build()
}

#[test]
fn test_overlapping_nodes_insertion() {
    let ast = sample_module();
    // Insertion at offset 22 should hit Procedure 2 and Module
    let nodes = ast.overlapping_nodes(22, 22);
    assert!(nodes.len() >= 2, "Expected at least two nodes (module + proc2)");
    let roots = ast.overlapping_root_nodes(22,22);
    // Root overlapping nodes should contain module only (since module is ancestor of proc2)
    assert_eq!(roots.len(), 1, "Expected single top-level overlapping node (module)");
    let leaves = ast.overlapping_leaf_nodes(22,22);
    // Leaf overlapping should include the deepest nodes covering position 22 (likely identifier or block/proc)
    assert!(!leaves.is_empty(), "Expected some leaf overlaps");
}

#[test]
fn test_overlapping_range_selection() {
    let ast = sample_module();
    // Range 0..20 should mostly pick first procedure + module
    let roots = ast.overlapping_root_nodes(0,20);
    assert!(roots.len() >= 1, "At least module root");
    // Leaves for 0..20 should not include Procedure 2
    let leaves = ast.overlapping_leaf_nodes(0,20);
    for nid in &leaves { let span = ast.arena().node(*nid).span; assert!(span.start < 21, "Leaf from wrong region"); }
}

#[test]
fn test_dirty_rebuild_boundaries() {
    let ast = sample_module();
    // Изменение внутри второй процедуры (offset 25)
    let boundaries = ast.dirty_rebuild_boundaries(25,25);
    assert!(!boundaries.is_empty(), "Expect at least one boundary");
    // Должна быть найдена именно вторая процедура, не модуль
    assert!(boundaries.iter().any(|nid| ast.arena().node(*nid).kind == AstKind::Procedure), "Expected procedure boundary");
}
