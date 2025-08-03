// Тестирование валидации вызовов методов с UnifiedBslIndex
use bsl_analyzer::BslParser;
use bsl_analyzer::bsl_parser::{SemanticAnalyzer, SemanticAnalysisConfig};
use bsl_analyzer::unified_index::UnifiedIndexBuilder;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    println!("🧠 Тестирование валидации вызовов методов с UnifiedBslIndex");
    println!("=============================================================");
    
    // Загружаем тестовый файл
    let content = std::fs::read_to_string("test_method_validation.bsl")?;
    println!("📖 Файл загружен: {} символов", content.len());
    
    // Создаем UnifiedBslIndex
    println!("\n🔧 Создание UnifiedBslIndex...");
    let config_path = Path::new("examples/ConfTest");
    let platform_version = "8.3.25";
    
    let builder = UnifiedIndexBuilder::new()?;
    let index = builder.build_index(config_path, platform_version)?;
    
    println!("✅ UnifiedBslIndex создан:");
    println!("   - Всего сущностей: {}", index.get_all_entities().len());
    
    // Показываем несколько доступных типов
    println!("\n📋 Доступные платформенные типы (примеры):");
    let platform_types = ["Массив", "Array", "Строка", "String", "ТаблицаЗначений", "ValueTable"];
    for type_name in &platform_types {
        if let Some(entity) = index.find_entity(type_name) {
            let methods = index.get_all_methods(&entity.qualified_name);
            println!("   - {}: {} методов", type_name, methods.len());
        }
    }
    
    // Тест 1: Парсинг BSL с расширенной структурой
    println!("\n🔧 Тест 1: Парсинг BSL кода");
    let parser = BslParser::new()?;
    let parse_result = parser.parse(&content, "test_method_validation.bsl");
    
    if let Some(ast) = parse_result.ast {
        println!("✅ AST получен: {} объявлений", ast.module.declarations.len());
        
        // Показываем найденные функции
        for (i, decl) in ast.module.declarations.iter().enumerate() {
            match decl {
                bsl_analyzer::bsl_parser::ast::Declaration::Function(func) => {
                    println!("  {}. Функция: {} ({} параметров)", i+1, func.name, func.params.len());
                    println!("     Операторов в теле: {}", func.body.len());
                }
                _ => {}
            }
        }
        
        // Извлекаем вызовы методов
        let method_calls = ast.extract_method_calls();
        println!("\n📞 Найдено вызовов методов: {}", method_calls.len());
        for (i, call) in method_calls.iter().enumerate() {
            if let bsl_analyzer::bsl_parser::ast::Expression::Identifier(obj_name) = &*call.object {
                println!("  {}. {}.{}({} аргументов)", i+1, obj_name, call.method, call.args.len());
            }
        }
        
        // Тест 2: Семантический анализ с валидацией методов
        println!("\n🔧 Тест 2: Семантический анализ с валидацией методов");
        
        let mut config = SemanticAnalysisConfig::default();
        config.check_method_calls = true;
        config.check_duplicate_parameters = true;
        config.check_undeclared_variables = true;
        
        let mut semantic = SemanticAnalyzer::with_index(config, index);
        
        match semantic.analyze(&ast) {
            Ok(()) => {
                let diagnostics = semantic.get_diagnostics();
                println!("✅ Семантический анализ выполнен");
                println!("   - Всего диагностик: {}", diagnostics.len());
                
                let mut errors = 0;
                let mut warnings = 0;
                let mut method_errors = 0;
                let mut param_errors = 0;
                
                for diag in diagnostics {
                    match diag.severity {
                        bsl_analyzer::bsl_parser::DiagnosticSeverity::Error => errors += 1,
                        _ => warnings += 1,
                    }
                    
                    // Подсчитываем типы ошибок
                    if diag.code == "BSL003" { method_errors += 1; }
                    if diag.code == "BSL004" { param_errors += 1; }
                    
                    println!("  {:?} [{}] в {}:{}: {}", 
                        diag.severity, 
                        diag.code,
                        diag.location.line, 
                        diag.location.column,
                        diag.message
                    );
                    
                    // Показываем дополнительную информацию
                    if let Some(found) = &diag.details.found {
                        println!("    Найдено: {}", found);
                    }
                    if let Some(expected) = &diag.details.expected {
                        println!("    Ожидалось: {}", expected);
                    }
                }
                
                println!("\n📈 Сводка: {} ошибок, {} предупреждений", errors, warnings);
                println!("   - Ошибки методов (BSL003): {}", method_errors);
                println!("   - Ошибки параметров (BSL004): {}", param_errors);
                
                // Проверяем, что валидация работает
                println!("\n🎯 Проверка валидации:");
                println!("   - Найдены ошибки вызовов методов: {}", if method_errors > 0 { "✅ да" } else { "❌ нет" });
                println!("   - Найдены ошибки параметров: {}", if param_errors > 0 { "✅ да" } else { "❌ нет" });
                
            }
            Err(e) => {
                println!("❌ Ошибка семантического анализа: {}", e);
            }
        }
        
    } else {
        println!("❌ Не удалось получить AST");
        if !parse_result.diagnostics.is_empty() {
            println!("Ошибки парсинга:");
            for diag in &parse_result.diagnostics {
                println!("  {}", diag.message);
            }
        }
    }
    
    println!("\n🎯 Тестирование валидации методов завершено");
    Ok(())
}