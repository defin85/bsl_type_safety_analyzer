/*!
# Direct Base64 Test

Прямая проверка Base64 функций в UnifiedBslIndex
*/

use std::path::Path;
use anyhow::Result;
use bsl_analyzer::unified_index::{UnifiedIndexBuilder};

fn main() -> Result<()> {
    println!("🔍 ПРЯМАЯ ПРОВЕРКА BASE64 ФУНКЦИЙ");
    println!("{}", "=".repeat(60));
    
    // Строим индекс
    let config_path = Path::new("examples/ConfTest");
    let platform_version = "8.3.25";
    
    let mut builder = UnifiedIndexBuilder::new()?;
    let index = builder.build_index(config_path, platform_version)?;
    
    // Проверяем Глобальный контекст
    if let Some(global_entity) = index.find_entity("Глобальный контекст") {
        println!("✅ Найден Глобальный контекст с {} методами", 
            global_entity.interface.methods.len());
        
        // Ищем Base64 функции
        let mut base64_functions = Vec::new();
        for (method_name, method) in &global_entity.interface.methods {
            if method_name.contains("Base64") || method_name.contains("base64") {
                base64_functions.push((method_name.clone(), method.clone()));
            }
        }
        
        println!("\n📋 Найдено Base64 функций: {}", base64_functions.len());
        println!("{}", "-".repeat(60));
        
        for (name, method) in base64_functions {
            let params: Vec<String> = method.parameters.iter().map(|p| {
                format!("{}: {}", p.name, p.type_name.as_deref().unwrap_or("Произвольный"))
            }).collect();
            
            let return_type = method.return_type.as_deref().unwrap_or("void");
            println!("✅ {}({}): {}", 
                name.replace("Глобальный контекст.", ""),
                params.join(", "), return_type);
                
            if let Some(doc) = &method.documentation {
                println!("   📝 {}", doc);
            }
            println!();
        }
        
    } else {
        println!("❌ Глобальный контекст не найден!");
    }
    
    Ok(())
}