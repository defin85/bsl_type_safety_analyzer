/*!
Test for BOM handling in BSL files
*/

use bsl_analyzer::parser::{BslLexer, read_bsl_file};
use tempfile::NamedTempFile;
use std::io::Write;

#[test]
fn test_bom_handling_integration() {
    let lexer = BslLexer::new();
    
    // Test UTF-8 BOM with BSL code
    let bsl_code_with_bom = "\u{FEFF}Процедура ТестоваяПроцедура() Экспорт\n    Сообщить(\"Тест\");\nКонецПроцедуры";
    let bsl_code_without_bom = "Процедура ТестоваяПроцедура() Экспорт\n    Сообщить(\"Тест\");\nКонецПроцедуры";
    
    // Tokenize both versions
    let tokens_with_bom = lexer.tokenize(bsl_code_with_bom).unwrap();
    let tokens_without_bom = lexer.tokenize(bsl_code_without_bom).unwrap();
    
    // Should produce identical token streams
    assert_eq!(tokens_with_bom.len(), tokens_without_bom.len());
    assert_eq!(tokens_with_bom[0].token_type, tokens_without_bom[0].token_type);
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
    
    // Should be able to tokenize cleanly
    let lexer = BslLexer::new();
    let tokens = lexer.tokenize(&content).unwrap();
    assert!(!tokens.is_empty());
    
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