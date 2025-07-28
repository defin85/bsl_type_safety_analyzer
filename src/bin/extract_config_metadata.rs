/*!
Скрипт для извлечения метаданных конфигурации в гибридный формат
*/

use anyhow::Result;
use bsl_analyzer::{
    configuration::MetadataReportParser,
    docs_integration::hybrid_storage::HybridDocumentationStorage,
};
use clap::Parser;
use std::path::PathBuf;
use console::style;

#[derive(Parser, Debug)]
#[command(author, version, about = "Extract configuration metadata to hybrid format")]
struct Args {
    /// Path to configuration report file
    #[arg(short, long, default_value = "examples/sample_config_report.txt")]
    report: PathBuf,
    
    /// Output directory for hybrid documentation
    #[arg(short, long, default_value = "output/hybrid_docs_direct")]
    output: PathBuf,
}

fn main() -> Result<()> {
    // Инициализация логирования
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("info".parse().unwrap())
        )
        .init();
    
    let args = Args::parse();
    
    println!("\n{}", style("=".repeat(60)).blue());
    println!("{}", style("🚀 ИЗВЛЕЧЕНИЕ МЕТАДАННЫХ КОНФИГУРАЦИИ").bold().cyan());
    println!("{}", style("=".repeat(60)).blue());
    
    // Проверяем существование файла отчета
    if !args.report.exists() {
        anyhow::bail!("Configuration report not found: {}", args.report.display());
    }
    
    println!("\n📄 Файл отчета: {}", style(&args.report.display()).yellow());
    println!("📁 Выходная папка: {}", style(&args.output.display()).yellow());
    
    // Создаем парсер и хранилище
    let parser = MetadataReportParser::new()?;
    let mut storage = HybridDocumentationStorage::new(&args.output);
    
    // Инициализируем хранилище
    storage.initialize()?;
    
    println!("\n{}", style("📋 Парсинг отчета конфигурации...").green());
    
    // Парсим отчет и записываем в гибридное хранилище
    parser.parse_to_hybrid_storage(&args.report, &mut storage)?;
    
    // Завершаем обработку
    storage.finalize()?;
    
    println!("\n{}", style("=".repeat(60)).blue());
    println!("{}", style("✅ ИЗВЛЕЧЕНИЕ ЗАВЕРШЕНО УСПЕШНО").bold().green());
    println!("{}", style("=".repeat(60)).blue());
    
    Ok(())
}