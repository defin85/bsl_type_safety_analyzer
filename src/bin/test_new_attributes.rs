use std::path::Path;
use bsl_analyzer::configuration::metadata_parser::MetadataReportParser;
use clap::Parser;

#[derive(Parser)]
#[command(name = "analyze_metadata_types")]
#[command(about = "Detailed analysis of 1C metadata types with constraints")]
struct Args {
    /// Path to 1C configuration report file (required)
    #[arg(long, short)]
    report: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    // Проверяем существование файла отчета
    if !Path::new(&args.report).exists() {
        eprintln!("❌ Ошибка: Файл отчета не найден: {}", args.report);
        eprintln!("📝 Пример использования:");
        eprintln!("   cargo run --bin analyze_metadata_types -- --report \"C:\\путь\\к\\отчету.txt\"");
        std::process::exit(1);
    }
    
    let parser = MetadataReportParser::new()?;
    let result = parser.parse_report(&args.report)?;
    
    println!("Parsed {} contracts total", result.len());
    
    // Найдем документ ЗаказНаряды
    if let Some(contract) = result.iter().find(|c| c.name == "ЗаказНаряды") {
        println!("\n=== Документ ЗаказНаряды ===");
        println!("Name: {}", contract.name);
        println!("Object type: {}", contract.object_type);
        
        println!("\nАтрибуты ({}):", contract.structure.attributes.len());
        for attr in &contract.structure.attributes {
            println!("  {} ({})", attr.name, attr.data_type);
            if let Some(length) = attr.length {
                println!("    Length: {}", length);
            }
            if let Some(precision) = attr.precision {
                println!("    Precision: {}", precision);
            }
        }
        
        println!("\nТабличные части ({}):", contract.structure.tabular_sections.len());
        for ts in &contract.structure.tabular_sections {
            println!("  {} ({} атрибутов)", ts.name, ts.attributes.len());
            for attr in &ts.attributes {
                println!("    {} ({})", attr.name, attr.data_type);
                if let Some(length) = attr.length {
                    println!("      Length: {}", length);
                }
                if let Some(precision) = attr.precision {
                    println!("      Precision: {}", precision);
                }
            }
        }
    } else {
        println!("❌ Документ ЗаказНаряды не найден!");
    }
    
    // Найдем регистр сведений для проверки секций
    if let Some(contract) = result.iter().find(|c| c.name == "ТестовыйРегистрСведений") {
        println!("\n=== РегистрСведений ТестовыйРегистрСведений ===");
        println!("Name: {}", contract.name);
        println!("Object type: {}", contract.object_type);
        
        println!("\nВсего атрибутов: {}", contract.structure.attributes.len());
        for attr in &contract.structure.attributes {
            println!("  {} ({})", attr.name, attr.data_type);
        }
    } else {
        println!("❌ РегистрСведений ТестовыйРегистрСведений не найден!");
    }
    
    Ok(())
}