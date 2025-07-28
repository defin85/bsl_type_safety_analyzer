/*!
Утилита для извлечения форм из конфигурации 1С в гибридное хранилище документации
*/

use std::path::PathBuf;
use clap::Parser;
use anyhow::Result;
use bsl_analyzer::configuration::form_parser::FormXmlParser;
use bsl_analyzer::docs_integration::hybrid_storage::HybridDocumentationStorage;

/// Извлечение форм из конфигурации в гибридное хранилище документации
#[derive(Parser, Debug)]
#[command(name = "extract-forms")]
#[command(about = "Извлекает формы из конфигурации 1С в гибридное хранилище")]
struct Args {
    /// Путь к директории конфигурации
    #[arg(short, long)]
    config: PathBuf,
    
    /// Путь к выходной директории для гибридного хранилища
    #[arg(short, long, default_value = "output/hybrid_docs_direct")]
    output: PathBuf,
}

fn main() -> Result<()> {
    // Инициализация логирования
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("bsl_analyzer=debug".parse()?)
        )
        .init();
    
    let args = Args::parse();
    
    println!("============================================================");
    println!("🚀 ИЗВЛЕЧЕНИЕ ФОРМ ИЗ КОНФИГУРАЦИИ");
    println!("============================================================");
    println!();
    println!("📁 Директория конфигурации: {}", args.config.display());
    println!("📁 Выходная папка: {}", args.output.display());
    
    // Проверяем существование директории конфигурации
    if !args.config.exists() {
        anyhow::bail!("Директория конфигурации не найдена: {}", args.config.display());
    }
    
    // Инициализируем хранилище
    let mut storage = HybridDocumentationStorage::new(args.output);
    
    // Создаем парсер форм
    let parser = FormXmlParser::new();
    
    println!();
    println!("🔍 Поиск XML файлов форм...");
    
    // Парсим все формы и записываем в хранилище
    parser.parse_to_hybrid_storage(&args.config, &mut storage)?;
    
    // Финализируем хранилище
    storage.finalize()?;
    
    println!();
    println!("============================================================");
    println!("✅ ИЗВЛЕЧЕНИЕ ЗАВЕРШЕНО УСПЕШНО");
    println!("============================================================");
    
    Ok(())
}