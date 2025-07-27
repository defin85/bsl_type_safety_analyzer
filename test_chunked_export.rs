use bsl_analyzer::docs_integration::BslSyntaxExtractor;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    
    let hbk_path = "C:/1CProject/1c-help-parser/data/rebuilt.shcntx_ru.zip";
    let output_dir = "test_output/docs_search";
    
    println!("Starting chunked export of BSL syntax documentation...");
    println!("Source: {}", hbk_path);
    println!("Output: {}", output_dir);
    
    // Создаем extractor
    let mut extractor = BslSyntaxExtractor::new(hbk_path);
    
    // Экспортируем все файлы в chunked формате
    // Для теста можно ограничить количество файлов
    let start_time = std::time::Instant::now();
    
    match extractor.extract_and_export_chunked(output_dir, Some(100)) {  // Ограничиваем 100 файлами для теста
        Ok(()) => {
            let elapsed = start_time.elapsed();
            println!("\nChunked export completed successfully!");
            println!("Time elapsed: {:.2} seconds", elapsed.as_secs_f64());
            
            // Проверяем структуру созданных файлов
            check_output_structure(output_dir)?;
        }
        Err(e) => {
            eprintln!("Failed to export: {}", e);
        }
    }
    
    Ok(())
}

fn check_output_structure(output_dir: &str) -> anyhow::Result<()> {
    use std::fs;
    use std::path::Path;
    
    println!("\n=== Checking output structure ===");
    
    // Проверяем main_index.json
    let main_index_path = Path::new(output_dir).join("main_index.json");
    if main_index_path.exists() {
        let content = fs::read_to_string(&main_index_path)?;
        let index: serde_json::Value = serde_json::from_str(&content)?;
        
        println!("Main index found:");
        println!("- Total items: {}", index["total_items"]);
        println!("- Categories:");
        
        if let Some(categories) = index["categories"].as_object() {
            for (category, info) in categories {
                println!("  - {}: {} items in {} files", 
                    category,
                    info["items_count"],
                    info["chunks_count"]
                );
            }
        }
    } else {
        println!("Main index not found!");
    }
    
    // Проверяем категории
    for category in ["objects", "methods", "functions", "properties", "operators"] {
        let category_dir = Path::new(output_dir).join(category);
        if category_dir.exists() {
            let files = fs::read_dir(&category_dir)?
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("json"))
                .count();
            println!("\n{} directory: {} JSON files", category, files);
            
            // Проверяем индекс категории
            let index_path = category_dir.join(format!("{}_index.json", category));
            if index_path.exists() {
                println!("- Category index found");
            }
        }
    }
    
    Ok(())
}