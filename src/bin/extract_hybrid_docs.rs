use bsl_analyzer::docs_integration::bsl_syntax_extractor::BslSyntaxExtractor;
use anyhow::Result;
use tracing_subscriber;
use clap::Parser;
use std::path::Path;

#[derive(Parser)]
#[command(name = "extract_hybrid_docs")]
#[command(about = "Extract BSL documentation from HBK archives to hybrid format")]
struct Args {
    /// Path to HBK archive file (.hbk or .zip) (required)
    #[arg(long, short)]
    archive: String,
    
    /// Output directory for hybrid documentation
    #[arg(long, short, default_value = "./output/hybrid_docs")]
    output: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Настраиваем логирование
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("=== BSL Documentation Hybrid Extractor ===");
    println!("Extracting BSL syntax directly to hybrid format...");

    // Проверяем существование архива
    if !Path::new(&args.archive).exists() {
        eprintln!("❌ Ошибка: Архив документации не найден: {}", args.archive);
        eprintln!("📝 Пример использования:");
        eprintln!("   cargo run --bin extract_hybrid_docs -- --archive \"C:\\путь\\к\\архиву.zip\" --output \"./output\"");
        std::process::exit(1);
    }

    let archive_path = &args.archive;
    let output_path = &args.output;

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