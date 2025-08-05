//! Отладка парсинга локальных функций

use bsl_analyzer::bsl_parser::{BslAnalyzer, SemanticAnalysisConfig};

fn main() -> anyhow::Result<()> {
    println!("🔍 Отладка парсинга локальных функций BSL");
    println!("==========================================");
    
    let file_path = "C:\\1CProject\\Unicom\\src\\task_00004\\CreateUsers_v2.bsl";
    
    // Создаем анализатор
    let mut analyzer = BslAnalyzer::new()?;
    
    // Читаем файл
    let content = std::fs::read_to_string(file_path)?;
    
    // Анализируем код
    println!("\n📂 Анализируем файл: {}", file_path);
    println!("📏 Размер файла: {} символов", content.len());
    
    match analyzer.analyze_code(&content, file_path) {
        Ok(_) => {
            println!("\n✅ Парсинг завершен:");
            let (errors, warnings) = analyzer.get_errors_and_warnings();
            println!("   🔍 Ошибок найдено: {}", errors.len());
            println!("   ⚠️  Предупреждений: {}", warnings.len());
            
            // Выводим первые 10 ошибок для анализа
            println!("\n🔍 Первые 10 ошибок:");
            for (i, error) in errors.iter().take(10).enumerate() {
                println!("   {}: {} ({}:{})", i+1, error.message, error.position.line, error.position.column);
            }
            
            // Проверим, видит ли парсер функции как локальные
            let local_function_names = [
                "ЗаписатьВЛог",
                "РазобратьJSONИзСтроки",
                "ВалидироватьСтруктуруДанных",
                "СоздатьПользователя",
                "ПолучитьИлиСоздатьГруппуПользователей",
            ];
            
            println!("\n🔍 Проверка локальных функций:");
            for func_name in &local_function_names {
                let found_error = errors.iter().any(|e| 
                    e.message.contains(func_name) && e.message.contains("не найдена")
                );
                let status = if found_error { "❌ НЕ НАЙДЕНА" } else { "✅ НАЙДЕНА" };
                println!("   {} {}", status, func_name);
            }
        }
        Err(e) => {
            println!("❌ Ошибка анализа: {}", e);
        }
    }
    
    // Дополнительно - попробуем разобрать файл напрямую парсером
    println!("\n🧪 Прямое тестирование парсера BSL:");
    test_bsl_parser_directly(&content)?;
    
    Ok(())
}

fn test_bsl_parser_directly(content: &str) -> anyhow::Result<()> {
    use bsl_analyzer::bsl_parser::{BslParser, semantic::SemanticAnalyzer};
    
    let parser = BslParser::new()?;
    
    let parse_result = parser.parse(content, "test.bsl");
    
    if let Some(ast) = parse_result.ast {
        println!("   ✅ AST создан успешно");
        println!("   📊 Объявлений в модуле: {}", ast.module.declarations.len());
        
        // Выводим типы объявлений
        for (i, decl) in ast.module.declarations.iter().enumerate() {
            let decl_type = match decl {
                bsl_analyzer::bsl_parser::ast::Declaration::Function(f) => format!("Функция '{}'", f.name),
                bsl_analyzer::bsl_parser::ast::Declaration::Procedure(p) => format!("Процедура '{}'", p.name),
                bsl_analyzer::bsl_parser::ast::Declaration::Variable(v) => format!("Переменные: {:?}", v.names),
            };
            println!("   {}: {}", i+1, decl_type);
        }
        
        // Тестируем семантический анализ
        println!("\n🧪 Тестирование семантического анализа:");
        let config = SemanticAnalysisConfig::default();
        let mut semantic_analyzer = SemanticAnalyzer::new(config);
        
        match semantic_analyzer.analyze(&ast) {
            Ok(_) => {
                let diagnostics = semantic_analyzer.get_diagnostics();
                println!("   ✅ Семантический анализ завершен");
                println!("   📊 Диагностик найдено: {}", diagnostics.len());
                
                // Ищем ошибки с локальными функциями
                let local_func_errors: Vec<_> = diagnostics.iter()
                    .filter(|d| d.message.contains("не найдена") && 
                               (d.message.contains("ЗаписатьВЛог") || 
                                d.message.contains("МояФункция")))
                    .collect();
                
                println!("   🔍 Ошибок с локальными функциями: {}", local_func_errors.len());
                for err in local_func_errors.iter().take(3) {
                    println!("      ❌ {}", err.message);
                }
            }
            Err(e) => {
                println!("   ❌ Ошибка семантического анализа: {}", e);
            }
        }
    } else {
        println!("   ❌ Не удалось создать AST");
        for diagnostic in parse_result.diagnostics {
            println!("      ❌ {}", diagnostic.message);
        }
    }
    
    Ok(())
}