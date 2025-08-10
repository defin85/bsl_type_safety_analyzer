//! File system utility helpers (BOM-aware readers, etc.)
use std::fs;
use std::path::Path;

/// Read a BSL file as UTF-8 text, stripping UTF-8 BOM if present.
pub fn read_bsl_file(path: &Path) -> std::io::Result<String> {
    let mut content = fs::read_to_string(path)?;
    if content.starts_with('\u{FEFF}') {
        content = content.trim_start_matches('\u{FEFF}').to_string();
    }
    Ok(content)
}
