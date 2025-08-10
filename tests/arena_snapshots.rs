//! Snapshot tests for arena semantic diagnostics.
//! Set env UPDATE_SNAPSHOTS=1 to regenerate.
use std::{fs, path::Path};
use bsl_analyzer::bsl_parser::{parser::BslParser, semantic_arena::SemanticArena};
use bsl_analyzer::core::position::LineIndex;
use bsl_analyzer::bsl_parser::diagnostics::Diagnostic;

// Removed unused FileDiagnostics placeholder to keep tests warning-free.

fn analyze_file(path: &Path) -> Vec<Diagnostic> {
    let src = fs::read_to_string(path).expect("read");
    let parser = BslParser::new().unwrap();
    let pr = parser.parse(&src, path.file_name().unwrap().to_string_lossy().as_ref());
    let mut out = Vec::new();
    if let Some(built) = &pr.arena {
        let mut sem = SemanticArena::new();
        sem.set_file_name(path.file_name().unwrap().to_string_lossy());
        sem.set_line_index(LineIndex::new(&src));
        sem.analyze_with_flags(built, true, true, true);
        out.extend_from_slice(sem.diagnostics());
    }
    out
}

fn snapshot_dir() -> &'static str { "tests/fixtures/arena" }
fn snapshot_store() -> &'static str { "tests/fixtures/arena/_snapshots" }

#[test]
fn arena_diagnostics_snapshots() {
    let update = std::env::var("UPDATE_SNAPSHOTS").ok().map(|v| v=="1" || v.to_lowercase()=="true").unwrap_or(false);
    let dir = Path::new(snapshot_dir());
    let store = Path::new(snapshot_store());
    if update && !store.exists() { fs::create_dir_all(store).unwrap(); }

    let mut failures = Vec::new();
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap(); let path = entry.path(); if path.is_dir() { continue; }
        if path.extension().and_then(|e| e.to_str()) != Some("bsl") { continue; }
        let diags = analyze_file(&path);
        let json = serde_json::to_string_pretty(&diags).unwrap();
        let snap_name = path.file_stem().unwrap().to_string_lossy().to_string() + ".json";
        let snap_path = store.join(&snap_name);
        if update {
            fs::write(&snap_path, &json).unwrap();
        } else {
            if !snap_path.exists() { failures.push(format!("missing snapshot: {}", snap_name)); continue; }
            let expected = fs::read_to_string(&snap_path).unwrap();
            if expected != json { failures.push(format!("diff in {}\n--- expected\n{}\n--- actual\n{}", snap_name, expected, json)); }
        }
    }
    if !failures.is_empty() { panic!("Snapshot mismatches:\n{}", failures.join("\n")); }
}
