use std::path::Path;
use std::fs;
use bsl_analyzer::configuration::metadata_parser::MetadataReportParser;
use bsl_analyzer::docs_integration::hybrid_storage::HybridDocumentationStorage;
use clap::Parser;

#[derive(Parser)]
#[command(name = "parse_metadata_full")]
#[command(about = "Full 1C metadata parser with HybridDocumentationStorage")]
struct Args {
    /// Path to 1C configuration report file (required)
    #[arg(long, short)]
    report: String,
    
    /// Output directory for parsed metadata
    #[arg(long, short, default_value = "./output/parsed_metadata")]
    output: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    // Проверяем существование файла отчета
    if !Path::new(&args.report).exists() {
        eprintln!("❌ Ошибка: Файл отчета не найден: {}", args.report);
        eprintln!("📝 Пример использования:");
        eprintln!("   cargo run --bin parse_metadata_full -- --report \"C:\\путь\\к\\отчету.txt\" --output \"./output\"");
        std::process::exit(1);
    }
    
    let report_path = &args.report;
    let output_dir = &args.output;
    
    println!("🔍 Парсинг метаданных из: {}", report_path);
    println!("📁 Результат будет сохранен в: {}", output_dir);
    
    // Создаем выходную директорию
    fs::create_dir_all(output_dir)?;
    
    // Парсим отчет по конфигурации
    let parser = MetadataReportParser::new()?;
    let contracts = parser.parse_report(report_path)?;
    
    println!("✅ Успешно обработано {} объектов конфигурации", contracts.len());
    
    // Создаем HybridDocumentationStorage структуру
    let mut storage = HybridDocumentationStorage::new(Path::new(output_dir));
    storage.initialize()?;
    
    println!("🏗️ Инициализирована HybridDocumentationStorage структура");
    
    // Селективная очистка для MetadataReportParser - только metadata_types, НЕ затрагиваем формы
    storage.clear_metadata_types_only()?;
    println!("🧹 Очищены старые metadata_types (селективная очистка - FormXmlParser данные сохранены)");
    
    // Группируем контракты по типам объектов
    let mut contracts_by_type: std::collections::HashMap<String, Vec<&_>> = std::collections::HashMap::new();
    for contract in &contracts {
        contracts_by_type
            .entry(contract.object_type.to_string().to_lowercase())
            .or_insert_with(Vec::new)
            .push(contract);
    }
    
    // Получаем путь к metadata_types для сохранения
    let metadata_types_dir = Path::new(output_dir).join("configuration").join("metadata_types");
    
    // Сохраняем каждый тип в отдельный файл
    for (object_type, type_contracts) in &contracts_by_type {
        let file_name = format!("{}.json", object_type);
        let type_file = metadata_types_dir.join(&file_name);
        let json_content = serde_json::to_string_pretty(type_contracts)?;
        fs::write(&type_file, json_content)?;
        println!("📁 Сохранено {} объектов типа '{}' в {}", type_contracts.len(), object_type, file_name);
    }
    
    // Выводим статистику
    println!("\n📊 СТАТИСТИКА ПАРСИНГА:");
    let mut stats: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    
    for contract in &contracts {
        *stats.entry(contract.object_type.to_string()).or_insert(0) += 1;
    }
    
    for (object_type, count) in stats.iter() {
        println!("  {} объектов типа '{}'", count, object_type);
    }
    
    // Показываем детали некоторых объектов
    println!("\n🔍 ДЕТАЛИ КЛЮЧЕВЫХ ОБЪЕКТОВ:");
    
    // Документ ЗаказНаряды
    if let Some(contract) = contracts.iter().find(|c| c.name == "ЗаказНаряды") {
        println!("\n📄 Документ.ЗаказНаряды:");
        println!("  Атрибутов: {}", contract.structure.attributes.len());
        println!("  Табличных частей: {}", contract.structure.tabular_sections.len());
        
        // Показываем интересные атрибуты
        for attr in &contract.structure.attributes {
            if attr.name.contains("Строковый") || attr.name.contains("Числовой") {
                let constraints = if let Some(length) = attr.length {
                    if let Some(precision) = attr.precision {
                        format!(" [length={}, precision={}]", length, precision)
                    } else {
                        format!(" [length={}]", length)
                    }
                } else {
                    String::new()
                };
                println!("    {} ({}){}", attr.name, attr.data_type, constraints);
            }
        }
        
        // Показываем составной тип
        for ts in &contract.structure.tabular_sections {
            if ts.name == "Стороны" {
                for attr in &ts.attributes {
                    if attr.name == "Сторона" {
                        println!("    Составной тип: {} ({})", attr.name, attr.data_type);
                    }
                }
            }
        }
    }
    
    // РегистрСведений ТестовыйРегистрСведений
    if let Some(contract) = contracts.iter().find(|c| c.name == "ТестовыйРегистрСведений") {
        println!("\n📊 РегистрСведений.ТестовыйРегистрСведений:");
        
        // Измерения (определяют уникальность записи)
        if let Some(ref dimensions) = contract.structure.dimensions {
            println!("  📐 Измерения ({}): ", dimensions.len());
            for dim in dimensions {
                println!("    {} ({})", dim.name, dim.data_type);
            }
        }
        
        // Ресурсы (собственно данные регистра)
        if let Some(ref resources) = contract.structure.resources {
            println!("  📊 Ресурсы ({}): ", resources.len());
            for res in resources {
                println!("    {} ({})", res.name, res.data_type);
            }
        }
        
        // Реквизиты (дополнительные атрибуты)
        println!("  📝 Реквизиты ({}): ", contract.structure.attributes.len());
        for attr in &contract.structure.attributes {
            println!("    {} ({})", attr.name, attr.data_type);
        }
    }
    
    println!("\n🎯 Парсинг завершен успешно!");
    println!("📂 Проверьте результаты в директории: {}", output_dir);
    
    Ok(())
}