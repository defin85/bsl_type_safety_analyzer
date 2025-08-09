/*!
Test for BOM handling in BSL files
*/

use bsl_analyzer::core::read_bsl_file;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_bom_handling_integration() {
    // Just ensure stripping BOM results in identical text starts
    let with_bom = "\u{FEFF}Процедура ТестоваяПроцедура() Экспорт";
    let without_bom = "Процедура ТестоваяПроцедура() Экспорт";
    assert_eq!(with_bom.trim_start_matches('\u{FEFF}'), without_bom);
}

#[test]
fn test_file_reading_with_bom() -> std::io::Result<()> {
    // Create temporary file with UTF-8 BOM
    let mut temp_file = NamedTempFile::new()?;
    let bsl_content = "\u{FEFF}Процедура Тест()\n    Возврат Истина;\nКонецПроцедуры";
    temp_file.write_all(bsl_content.as_bytes())?;

    // Read file using our BOM-aware function
    let content = read_bsl_file(temp_file.path()).unwrap();

    // BOM should be stripped
    assert!(!content.starts_with('\u{FEFF}'));
    assert!(content.starts_with("Процедура"));

    // Basic sanity: content contains procedure keyword
    assert!(content.contains("Процедура"));

    Ok(())
}

#[test]
fn test_utf8_bom_bytes() -> std::io::Result<()> {
    // Create file with UTF-8 BOM bytes (EF BB BF)
    let mut temp_file = NamedTempFile::new()?;
    let mut content = vec![0xEF, 0xBB, 0xBF]; // UTF-8 BOM
    content.extend_from_slice("Функция Тест()\n    Возврат 42;\nКонецФункции".as_bytes());
    temp_file.write_all(&content)?;

    // Read and verify BOM is stripped
    let content = read_bsl_file(temp_file.path()).unwrap();
    assert!(!content.starts_with('\u{FEFF}'));
    assert!(content.starts_with("Функция"));

    Ok(())
}
