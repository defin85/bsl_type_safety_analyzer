// Тестирование расширенного семантического анализа
use bsl_analyzer::{BslAnalyzer, BslParser};
use bsl_analyzer::bsl_parser::{SemanticAnalyzer, SemanticAnalysisConfig};

fn main() -> anyhow::Result<()> {
    println!("🧠 Тестирование расширенного семантического анализа");
    println!("===================================================");
    
    // Тест расширенных семантических проверок
    println!("\n📁 Анализ файла с дублированными параметрами и неинициализированными переменными");
    
    let content = std::fs::read_to_string("test_advanced_semantic.bsl")?;
    println!("📖 Файл загружен: {} символов", content.len());
    
    // Тест 1: Полный анализ через BslAnalyzer
    println!("\n🔧 Тест 1: Полный анализ через BslAnalyzer");
    let mut analyzer = BslAnalyzer::new()?;
    
    match analyzer.analyze_code(&content, "test_advanced_semantic.bsl") {
        Ok(()) => {
            let results = analyzer.get_results();
            println!("✅ Анализ выполнен:");
            println!("   - Ошибки: {}", results.error_count());
            println!("   - Предупреждения: {}", results.warning_count());
            
            if results.has_errors() || results.has_warnings() {
                println!("\n📋 Все найденные проблемы:");
                println!("{}", results);
            }
        }
        Err(e) => {
            println!("❌ Ошибка анализа: {}", e);
        }
    }
    
    // Тест 2: Прямое тестирование семантического анализатора
    println!("\n🔧 Тест 2: Детальный семантический анализ");
    
    let parser = BslParser::new()?;
    let parse_result = parser.parse(&content, "test_advanced_semantic.bsl");
    
    if let Some(ast) = parse_result.ast {
        println!("✅ AST получен: {} объявлений", ast.module.declarations.len());
        
        // Показываем структуру
        for (i, decl) in ast.module.declarations.iter().enumerate() {
            match decl {
                bsl_analyzer::bsl_parser::ast::Declaration::Procedure(proc) => {
                    println!("  {}. Процедура: {} ({} параметров)", i+1, proc.name, proc.params.len());
                }
                bsl_analyzer::bsl_parser::ast::Declaration::Function(func) => {
                    println!("  {}. Функция: {} ({} параметров)", i+1, func.name, func.params.len());
                }
                bsl_analyzer::bsl_parser::ast::Declaration::Variable(var) => {
                    println!("  {}. Переменная: {:?}", i+1, var.names);
                }
            }
        }
        
        let mut config = SemanticAnalysisConfig::default();
        config.check_duplicate_parameters = true;
        config.check_uninitialized_variables = true;
        config.check_undeclared_variables = true;
        
        let mut semantic = SemanticAnalyzer::new(config);
        
        match semantic.analyze(&ast) {
            Ok(()) => {
                let diagnostics = semantic.get_diagnostics();
                println!("\n📊 Результаты детального анализа:");
                println!("   - Всего диагностик: {}", diagnostics.len());
                
                let mut errors = 0;
                let mut warnings = 0;
                
                for diag in diagnostics {
                    match diag.severity {
                        bsl_analyzer::bsl_parser::DiagnosticSeverity::Error => errors += 1,
                        _ => warnings += 1,
                    }
                    
                    println!("  {:?} в {}:{}: {}", 
                        diag.severity, 
                        diag.location.line, 
                        diag.location.column,
                        diag.message
                    );
                }
                
                println!("\n📈 Сводка: {} ошибок, {} предупреждений", errors, warnings);
                
                // Проверяем, что нашли ожидаемые проблемы
                let messages: Vec<&str> = diagnostics.iter().map(|d| d.message.as_str()).collect();
                
                let has_duplicate_params = messages.iter().any(|m| m.contains("Дублированный параметр"));
                let has_undeclared_vars = messages.iter().any(|m| m.contains("не объявлена"));
                
                println!("\n🎯 Проверка ожидаемых проблем:");
                println!("   - Дублированные параметры: {}", if has_duplicate_params { "✅ найдены" } else { "❌ не найдены" });
                println!("   - Необъявленные переменные: {}", if has_undeclared_vars { "✅ найдены" } else { "❌ не найдены" });
                
            }
            Err(e) => {
                println!("❌ Ошибка семантического анализа: {}", e);
            }
        }
    } else {
        println!("❌ Не удалось получить AST");
    }
    
    println!("\n🎯 Тестирование расширенного семантического анализа завершено");
    Ok(())
}