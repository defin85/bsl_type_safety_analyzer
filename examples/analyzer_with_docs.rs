/*!
# Пример использования анализатора с документацией

Показывает, как использовать BSL анализатор вместе с загруженной документацией
для улучшенной верификации методов и автодополнения.
*/

use bsl_analyzer::BslAnalyzer;
use bsl_analyzer::Configuration;
use bsl_analyzer::DocsIntegration;
use anyhow::Result;

fn main() -> Result<()> {
    // Настраиваем логирование
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    println!("=== BSL Analyzer with Documentation Integration ===\n");
    
    // 1. Загружаем документацию
    println!("1. Loading documentation...");
    let mut docs = DocsIntegration::new();
    
    // Загружаем из chunked формата (быстро)
    docs.load_chunked_documentation("output/docs_search")?;
    println!("   ✓ Documentation loaded from chunked format");
    
    // 2. Загружаем конфигурацию
    println!("\n2. Loading configuration...");
    let config = Configuration::load_from_directory("test_config")?;
    println!("   ✓ Configuration loaded: {} modules", config.get_modules().len());
    
    // 3. Создаем анализатор
    println!("\n3. Creating analyzer...");
    let mut analyzer = BslAnalyzer::new()?;
    
    // 4. Анализируем конфигурацию
    println!("\n4. Analyzing configuration...");
    let results = analyzer.analyze_configuration(&config)?;
    
    println!("\n=== Analysis Results ===");
    let total_issues: usize = results.iter().map(|r| r.total_issues()).sum();
    println!("Total issues: {}", total_issues);
    
    // 5. Пример использования документации для верификации
    println!("\n5. Method verification with documentation:");
    
    // Пытаемся найти информацию о методе
    if let Some(method_info) = docs.get_method_info("Сообщить") {
        println!("   Found method 'Сообщить':");
        println!("   - English name: {:?}", method_info.english_name);
        println!("   - Parameters: {} total", method_info.parameters.len());
        
        // Можно использовать для проверки правильности вызовов
        let call_params = 2; // Пример: вызов с 2 параметрами
        let param_count = method_info.parameters.len();
        if call_params > param_count {
            println!("   ⚠️  Warning: Too many parameters!");
        } else {
            println!("   ✓ Parameter count seems valid");
        }
    }
    
    // 6. Пример автодополнения
    println!("\n6. Autocomplete example:");
    let completions = docs.get_completions("Масс");
    println!("   Completions for 'Масс':");
    for (i, item) in completions.iter().take(5).enumerate() {
        println!("   {}. {} - {:?}", i + 1, item.label, item.detail);
    }
    
    // 7. Поиск методов
    println!("\n7. Method search example:");
    let methods = docs.search_methods("Добавить");
    println!("   Found {} methods containing 'Добавить'", methods.len());
    for (i, method) in methods.iter().take(3).enumerate() {
        println!("   {}. {} ({:?})", i + 1, method.name, method.english_name);
    }
    
    println!("\n✅ Analysis complete!");
    
    Ok(())
}

// Пример интеграции с LSP сервером
#[allow(dead_code)]
fn use_in_lsp_server(_docs: &DocsIntegration) {
    // При инициализации LSP сервера загружаем документацию
    // docs.load_chunked_documentation("docs_search")?;
    
    // При запросе автодополнения
    fn on_completion_request(docs: &DocsIntegration, prefix: &str) {
        let _items = docs.get_completions(prefix);
        // Конвертируем в LSP CompletionItem и отправляем клиенту
    }
    
    // При наведении на метод
    fn on_hover_request(docs: &DocsIntegration, method_name: &str) {
        if let Some(info) = docs.get_method_info(method_name) {
            // Показываем документацию метода
            let _hover_text = format!(
                "**{}** ({:?})\n\n{:?}\n\nПараметров: {}",
                info.name,
                info.english_name,
                info.description,
                info.parameters.len()
            );
        }
    }
}

// Пример использования с chunked loader напрямую
#[allow(dead_code)]
fn use_chunked_loader_directly() -> Result<()> {
    use bsl_analyzer::docs_integration::chunked_loader::ChunkedDocsLoader;
    
    // Создаем загрузчик
    let mut loader = ChunkedDocsLoader::new("output/docs_search");
    loader.load_index()?;
    
    // Получаем конкретный элемент по ID
    if let Some(item) = loader.get_item("methods_42")? {
        println!("Found item: {}", item.title);
    }
    
    // Ищем по имени объекта
    let items = loader.find_by_object("ДинамическийСписок")?;
    println!("Found {} items for ДинамическийСписок", items.len());
    
    // Получаем все методы
    let methods = loader.get_category_items("methods")?;
    println!("Total methods: {}", methods.len());
    
    Ok(())
}