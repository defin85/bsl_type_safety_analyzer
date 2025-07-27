use bsl_analyzer::docs_integration::{hbk_parser_full::HbkArchiveParser, bsl_syntax_extractor::BslSyntaxExtractor};
use anyhow::Result;
use tracing_subscriber;

fn main() -> Result<()> {
    // Настраиваем логирование
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("=== BSL Documentation Hybrid Extractor ===");
    println!("Extracting BSL syntax directly to hybrid format...");

    let archive_path = "C:/1CProject/1c-help-parser/data/rebuilt.shcntx_ru.zip";
    let output_path = "output/hybrid_docs_direct";

    // Создаем экстрактор синтаксиса напрямую с путем к архиву
    let mut extractor = BslSyntaxExtractor::new(archive_path);

    println!("📁 Source: {}", archive_path);
    println!("📁 Output: {}", output_path);

    // Извлекаем напрямую в гибридный формат
    extractor.extract_to_hybrid_storage(output_path, None)?;

    println!("✅ Hybrid extraction completed successfully!");
    println!("📂 Hybrid documentation saved to: {}", output_path);

    Ok(())
}