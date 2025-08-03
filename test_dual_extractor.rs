// Быстрый тест для проверки двойного архива

use bsl_analyzer::docs_integration::bsl_syntax_extractor::BslSyntaxExtractor;
use anyhow::Result;

fn main() -> Result<()> {
    // Инициализируем экстрактор с контекстным архивом
    let mut extractor = BslSyntaxExtractor::new("examples/rebuilt.shcntx_ru.zip");
    
    // Добавляем языковой архив
    extractor.set_language_archive("examples/rebuilt.shlang_ru.zip");
    
    // Извлекаем базу данных
    let database = extractor.extract_syntax_database(Some(50))?;
    
    println!("Результаты извлечения:");
    println!("  Objects: {}", database.objects.len());
    println!("  Methods: {}", database.methods.len());
    println!("  Properties: {}", database.properties.len());
    println!("  Functions: {}", database.functions.len());
    println!("  Operators: {}", database.operators.len());
    println!("  Keywords: {}", database.keywords.len());
    
    // Проверяем примитивные типы
    println!("\nПримитивные типы:");
    if let Some(string_type) = database.objects.get("String") {
        println!("  - String: {}", string_type.description.as_ref().unwrap_or(&"No description".to_string()));
    }
    if let Some(number_type) = database.objects.get("Number") {
        println!("  - Number: {}", number_type.description.as_ref().unwrap_or(&"No description".to_string()));
    }
    
    // Проверяем директивы
    println!("\nДирективы компиляции:");
    for keyword in &database.keywords {
        if keyword.starts_with('&') {
            println!("  - {}", keyword);
        }
    }
    
    Ok(())
}