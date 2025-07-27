use bsl_analyzer::docs_integration::{HbkArchiveParser, BslSyntaxExtractor};

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    
    // Пути к потенциальным HBK файлам
    let hbk_paths = vec![
        "C:/1CProject/1c-help-parser/data/rebuilt.shcntx_ru.zip",  // Восстановленный файл
        "C:/1CProject/1c-help-parser/data/shcntx_ru.hbk",
        "C:/1CProject/1c-help-parser/examples/help.hbk",
        "C:/1CProject/1c-help-parser/help.hbk", 
        "C:/1CProject/help.hbk",
        "./help.hbk",
    ];
    
    // Ищем существующий HBK файл
    let mut hbk_path = None;
    for path in &hbk_paths {
        if std::path::Path::new(path).exists() {
            hbk_path = Some(path);
            println!("Found HBK file at: {}", path);
            break;
        }
    }
    
    if let Some(path) = hbk_path {
        // Тестируем HbkArchiveParser
        println!("\n=== Testing HbkArchiveParser ===");
        let mut parser = HbkArchiveParser::new(path);
        
        match parser.analyze_archive() {
            Ok(analysis) => {
                println!("Archive analysis successful!");
                println!("Total files: {}", analysis.total_files);
                println!("HTML files: {}", analysis.html_files_count);
                println!("File categories: {:?}", analysis.file_categories);
                
                // Показываем примеры контента
                if !analysis.sample_content.is_empty() {
                    println!("\nSample content from first file:");
                    if let Some(content) = analysis.sample_content.first() {
                        println!("Title: {:?}", content.title);
                        println!("Object type: {:?}", content.object_type);
                        println!("Syntax items: {}", content.syntax.len());
                    }
                }
            }
            Err(e) => {
                println!("Failed to analyze archive: {}", e);
            }
        }
        
        // Тестируем BslSyntaxExtractor
        println!("\n=== Testing BslSyntaxExtractor ===");
        let mut extractor = BslSyntaxExtractor::new(path);
        
        // Извлекаем больше файлов для теста
        match extractor.extract_syntax_database(Some(1000)) {
            Ok(db) => {
                println!("Syntax extraction successful!");
                println!("Methods: {}", db.methods.len());
                println!("Objects: {}", db.objects.len());
                println!("Functions: {}", db.functions.len());
                println!("Properties: {}", db.properties.len());
                
                // Показываем примеры методов
                if !db.methods.is_empty() {
                    println!("\nFirst 5 methods:");
                    for (name, _) in db.methods.iter().take(5) {
                        println!("  - {}", name);
                    }
                } else if !db.objects.is_empty() {
                    println!("\nFirst 5 objects:");
                    for (name, _) in db.objects.iter().take(5) {
                        println!("  - {}", name);
                    }
                }
                
                // Тестируем автодополнение
                let completions = db.get_completion_items("Сообщ");
                println!("\nCompletions for 'Сообщ': {}", completions.len());
                for item in completions.iter().take(3) {
                    println!("  - {} ({:?})", item.label, item.kind);
                }
                
                // Сохраняем результаты в JSON
                println!("\nSaving syntax database to test_output/bsl_syntax.json...");
                std::fs::create_dir_all("test_output")?;
                let json_data = serde_json::to_string_pretty(&db)?;
                std::fs::write("test_output/bsl_syntax.json", json_data)?;
                println!("Syntax database saved successfully!");
            }
            Err(e) => {
                println!("Failed to extract syntax: {}", e);
            }
        }
        
    } else {
        println!("No HBK file found in the following locations:");
        for path in &hbk_paths {
            println!("  - {}", path);
        }
        println!("\nPlease ensure a valid HBK file exists in one of these locations.");
        
        // Проверяем, есть ли файлы в директории 1c-help-parser
        let parser_dir = std::path::Path::new("C:/1CProject/1c-help-parser");
        if parser_dir.exists() {
            println!("\nFiles in 1c-help-parser directory:");
            if let Ok(entries) = std::fs::read_dir(parser_dir) {
                for entry in entries.flatten().take(10) {
                    if let Ok(name) = entry.file_name().into_string() {
                        println!("  - {}", name);
                    }
                }
            }
        }
    }
    
    Ok(())
}