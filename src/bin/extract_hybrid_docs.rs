use bsl_analyzer::docs_integration::bsl_syntax_extractor::BslSyntaxExtractor;
use anyhow::Result;
use tracing_subscriber;
use clap::Parser;
use std::path::Path;
use std::fs;

#[derive(Parser)]
#[command(name = "extract_hybrid_docs")]
#[command(about = "Extract BSL documentation from HBK archives to syntax database")]
struct Args {
    /// Path to HBK archive file (.hbk or .zip) (required)
    #[arg(long, short)]
    archive: String,
    
    /// Output directory for syntax database
    #[arg(long, short, default_value = "./output/bsl_syntax_database.json")]
    output: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Настраиваем логирование
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("=== BSL Syntax Database Extractor ===");
    println!("Extracting BSL syntax database from documentation...");

    // Проверяем существование архива
    if !Path::new(&args.archive).exists() {
        eprintln!("❌ Ошибка: Архив документации не найден: {}", args.archive);
        eprintln!("📝 Пример использования:");
        eprintln!("   cargo run --bin extract_hybrid_docs -- --archive \"C:\\путь\\к\\архиву.zip\" --output \"./database.json\"");
        std::process::exit(1);
    }

    let archive_path = &args.archive;
    let output_path = &args.output;

    // Создаем экстрактор синтаксиса
    let mut extractor = BslSyntaxExtractor::new(archive_path);

    println!("📁 Source: {}", archive_path);
    println!("📁 Output: {}", output_path);

    // Извлекаем синтаксисную базу данных
    let database = extractor.extract_syntax_database(None)?;

    // Создаем выходную директорию если нужно
    if let Some(parent) = Path::new(output_path).parent() {
        fs::create_dir_all(parent)?;
    }

    // Сохраняем в JSON
    let json_data = serde_json::to_string_pretty(&database)?;
    fs::write(output_path, json_data)?;

    println!("✅ Extraction completed successfully!");
    println!("📊 Statistics:");
    println!("   - Objects: {}", database.objects.len());
    println!("   - Methods: {}", database.methods.len());
    println!("   - Properties: {}", database.properties.len());
    println!("   - Functions: {}", database.functions.len());
    println!("   - Operators: {}", database.operators.len());
    println!("📂 Syntax database saved to: {}", output_path);

    Ok(())
}