/*!
# Integration Tests for BSL Type Safety Analyzer v2.0

Tests modern UnifiedBslIndex architecture.
Legacy parsers have been removed.
*/

use bsl_analyzer::{Configuration, bsl_parser::analyzer::BslAnalyzer};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_configuration_loading() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path();

    // Create minimal Configuration.xml
    let metadata_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
    <MetaDataObject xmlns="http://v8.1c.ru/8.3/MDClasses" version="2.19">
      <Configuration uuid="12345678-1234-1234-1234-123456789012">
        <Name>TestConfiguration</Name>
        <Version>1.0.0</Version>
        <DefaultRunMode>ManagedApplication</DefaultRunMode>
        <ScriptVariant>Russian</ScriptVariant>
      </Configuration>
    </MetaDataObject>"#;

    fs::write(config_path.join("Configuration.xml"), metadata_xml).unwrap();

    // Test modern configuration loading
    let config = Configuration::load_from_directory(config_path).unwrap();

    assert_eq!(config.metadata.name, "TestConfiguration");
    assert_eq!(config.metadata.version, "1.0.0");
    assert!(config.modules.is_empty()); // No BSL modules in this test
    assert!(config.objects.is_empty());
}

#[test]
fn incremental_metrics_core_snapshot() {
  let mut analyzer = BslAnalyzer::new().expect("analyzer");
  let original = "Процедура P1()\n КонецПроцедуры\n\nПроцедура P2()\n КонецПроцедуры";
  analyzer.analyze_code(original, "file.bsl").unwrap();
  // small edit inside second routine
  let modified = "Процедура P1()\n КонецПроцедуры\n\nПроцедура P2()\n // edit\n КонецПроцедуры";
  analyzer.analyze_incremental(modified, "file.bsl").unwrap();
  let stats = analyzer.last_incremental_stats().expect("stats present");
  // Build core JSON like LSP without debug flag
  let core = serde_json::json!({
    "totalNodes": stats.total_nodes,
    "changedNodes": stats.changed_nodes,
    "reusedNodes": stats.reused_nodes,
    "reusedSubtrees": stats.reused_subtrees,
    "reuseRatio": stats.reuse_ratio,
    "parseNs": stats.parse_ns,
    "arenaNs": stats.arena_ns,
    "fingerprintNs": stats.fingerprint_ns,
    "semanticNs": stats.semantic_ns,
    "totalNs": stats.total_ns,
    "plannedRoutines": stats.planned_routines,
    "replacedRoutines": stats.replaced_routines,
    "fallbackReason": stats.fallback_reason,
    "initialTouched": stats.initial_touched,
    "expandedTouched": stats.expanded_touched,
  });
  // Stable shape assertions (not exact numbers)
  assert!(core.get("totalNodes").is_some());
  assert!(core.get("reuseRatio").is_some());
  assert!(core.get("innerReusedNodes").is_none(), "debug field must be absent in core reconstruction test" );
}
#[ignore = "Legacy test - parsers removed"]
#[test]
fn test_legacy_metadata_parser() {
    // This test is disabled because MetadataReportParser has been removed
    // in favor of direct XML parsing in UnifiedBslIndex
}

#[ignore = "Legacy test - parsers removed"]
#[test]
fn test_legacy_form_parser() {
    // This test is disabled because FormXmlParser has been removed
    // in favor of UnifiedBslIndex architecture
}
