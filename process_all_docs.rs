use bsl_analyzer::docs_integration::BslSyntaxExtractor;
use std::time::Instant;

fn main() -> anyhow::Result<()> {
    // Настраиваем логирование
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();
    
    let hbk_path = "C:/1CProject/1c-help-parser/data/rebuilt.shcntx_ru.zip";
    let output_dir = "output/docs_search";
    
    println!("=== BSL Syntax Documentation Full Export ===");
    println!("Source: {}", hbk_path);
    println!("Output: {}", output_dir);
    println!("Processing all HTML files from the archive...\n");
    
    // Создаем extractor
    let mut extractor = BslSyntaxExtractor::new(hbk_path);
    
    // Запускаем полную обработку
    let start_time = Instant::now();
    
    match extractor.extract_and_export_chunked(output_dir, None) {  // None = обработать все файлы
        Ok(()) => {
            let elapsed = start_time.elapsed();
            println!("\n✅ Export completed successfully!");
            println!("⏱️  Total time: {:.2} seconds ({:.2} minutes)", 
                elapsed.as_secs_f64(), 
                elapsed.as_secs_f64() / 60.0
            );
            
            // Показываем статистику
            show_export_statistics(output_dir)?;
        }
        Err(e) => {
            eprintln!("\n❌ Export failed: {}", e);
            std::process::exit(1);
        }
    }
    
    Ok(())
}

fn show_export_statistics(output_dir: &str) -> anyhow::Result<()> {
    use std::fs;
    use std::path::Path;
    
    println!("\n=== Export Statistics ===");
    
    // Читаем main_index.json для получения статистики
    let main_index_path = Path::new(output_dir).join("main_index.json");
    if main_index_path.exists() {
        let content = fs::read_to_string(&main_index_path)?;
        let index: serde_json::Value = serde_json::from_str(&content)?;
        
        if let Some(stats) = index.get("statistics") {
            println!("📊 Total items: {}", index["total_items"]);
            println!("📁 Total files: {}", stats["total_files"]);
            println!("💾 Total size: {:.2} MB", stats["total_size_mb"]);
            println!("📈 Average items per file: {:.1}", stats["average_items_per_file"]);
            
            if let Some(coverage) = stats.get("coverage") {
                println!("\n📋 Coverage:");
                println!("   - Processed: {} files", coverage["html_files_processed"]);
                println!("   - Total: {} files", coverage["html_files_total"]);
                println!("   - Coverage: {:.2}%", coverage["coverage_percent"]);
            }
            
            if let Some(processing) = stats.get("processing_info") {
                println!("\n⚙️  Processing:");
                println!("   - Time: {:.2} seconds", processing["extraction_time_seconds"]);
                println!("   - Errors: {}", processing["errors_count"]);
                println!("   - Warnings: {}", processing["warnings_count"]);
            }
        }
        
        // Показываем разбивку по категориям
        if let Some(categories) = index.get("categories").and_then(|c| c.as_object()) {
            println!("\n📂 Categories:");
            let mut total_items = 0;
            for (category, info) in categories {
                let items = info["items_count"].as_u64().unwrap_or(0);
                let chunks = info["chunks_count"].as_u64().unwrap_or(0);
                println!("   - {}: {} items in {} files", category, items, chunks);
                total_items += items;
            }
            println!("   Total: {} items", total_items);
        }
    } else {
        println!("Main index not found!");
    }
    
    Ok(())
}