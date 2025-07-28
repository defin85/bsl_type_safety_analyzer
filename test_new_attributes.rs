use std::path::Path;
use bsl_analyzer::configuration::metadata_parser::MetadataReportParser;

fn main() {
    let parser = MetadataReportParser::new().expect("Failed to create parser");
    let result = parser.parse_report("C:\\Users\\Egor\\Downloads\\ОтчетПоКонфигурации888.txt")
        .expect("Failed to parse report");
    
    println!("Parsed {} contracts total", result.len());
    
    // Найдем документ ЗаказНаряды
    if let Some(contract) = result.iter().find(|c| c.name == "ЗаказНаряды") {
        println!("\n=== Документ ЗаказНаряды ===");
        println!("ID: {}", contract.id);
        println!("Name: {}", contract.name);
        println!("Object type: {}", contract.object_type);
        
        println!("\nАтрибуты ({}):", contract.attributes.len());
        for attr in &contract.attributes {
            println!("  {} ({})", attr.name, attr.data_type);
            if let Some(length) = attr.length {
                println!("    Length: {}", length);
            }
            if let Some(precision) = attr.precision {
                println!("    Precision: {}", precision);
            }
        }
        
        println!("\nТабличные части ({}):", contract.tabular_sections.len());
        for ts in &contract.tabular_sections {
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
    }
    
    // Найдем регистр сведений для проверки секций
    if let Some(contract) = result.iter().find(|c| c.name == "ТестовыйРегистрСведений") {
        println!("\n=== РегистрСведений ТестовыйРегистрСведений ===");
        println!("ID: {}", contract.id);
        println!("Name: {}", contract.name);
        println!("Object type: {}", contract.object_type);
        
        println!("\nВсего атрибутов: {}", contract.attributes.len());
        for attr in &contract.attributes {
            println!("  {} ({})", attr.name, attr.data_type);
        }
    }
}