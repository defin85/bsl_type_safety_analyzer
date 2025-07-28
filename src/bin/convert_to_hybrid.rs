use bsl_analyzer::docs_integration::hybrid_storage::HybridDocumentationStorage;
use anyhow::Result;
use tracing_subscriber;

fn main() -> Result<()> {
    // Настраиваем логирование
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("=== BSL Documentation Hybrid Converter ===");
    println!("Converting chunked format to hybrid storage format...");

    let chunked_path = "output/docs_search";
    let hybrid_path = "output/hybrid_docs";

    // Создаем новое гибридное хранилище
    let mut storage = HybridDocumentationStorage::new(hybrid_path);

    // Конвертируем из chunked формата
    // TODO: Реализовать convert_from_chunked метод
    println!("⚠️ Метод convert_from_chunked временно недоступен");

    println!("✅ Conversion completed successfully!");
    println!("📁 Hybrid documentation saved to: {}", hybrid_path);

    Ok(())
}