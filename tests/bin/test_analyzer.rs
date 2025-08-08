// Тестовый скрипт для проверки анализатора
use bsl_analyzer::{analyze_file, BslAnalyzer, BslParser};

fn main() -> anyhow::Result<()> {
    println!("🔍 Тестирование BSL Type Safety Analyzer v1.2.0");
    println!("==================================================");

    // Тест 1: Анализ файла через библиотечную функцию
    println!("\n📁 Тест 1: Анализ файла через analyze_file()");
    match analyze_file("test_sample.bsl") {
        Ok(result) => {
            println!("✅ Результат: {}", result);
        }
        Err(e) => {
            println!("❌ Ошибка: {}", e);
        }
    }

    // Тест 2: Прямое использование анализатора
    println!("\n🔧 Тест 2: Прямое использование BslAnalyzer");

    let content = std::fs::read_to_string("test_sample.bsl")?;
    println!("📖 Файл загружен: {} символов", content.len());

    // Создаем анализатор
    let mut analyzer = BslAnalyzer::new()?;
    println!("🚀 BslAnalyzer создан");

    // Анализируем код
    match analyzer.analyze_code(&content, "test_sample.bsl") {
        Ok(()) => {
            println!("✅ Анализ выполнен успешно");

            // Получаем результаты
            let results = analyzer.get_results();
            println!("📊 Результаты анализа:");
            println!("   - Ошибки: {}", results.error_count());
            println!("   - Предупреждения: {}", results.warning_count());

            if results.has_errors() || results.has_warnings() {
                println!("\n📋 Детали:");
                println!("{}", results);
            } else {
                println!("✨ Анализ не выявил проблем");
            }
        }
        Err(e) => {
            println!("❌ Ошибка анализа: {}", e);
        }
    }

    // Тест 3: Парсер отдельно
    println!("\n⚙️ Тест 3: Тестирование парсера");

    let parser = BslParser::new()?;
    println!("🔧 BslParser создан");

    let parse_result = parser.parse(&content, "test_sample.bsl");
    match parse_result.ast {
        Some(ast) => {
            println!("✅ Парсинг выполнен успешно");
            println!(
                "🌳 AST содержит модуль с {} объявлениями",
                ast.module.declarations.len()
            );

            // Показываем структуру AST
            for (i, decl) in ast.module.declarations.iter().enumerate() {
                match decl {
                    bsl_analyzer::bsl_parser::ast::Declaration::Procedure(proc) => {
                        println!(
                            "  {}. Процедура: {} (экспорт: {})",
                            i + 1,
                            proc.name,
                            proc.export
                        );
                    }
                    bsl_analyzer::bsl_parser::ast::Declaration::Function(func) => {
                        println!(
                            "  {}. Функция: {} (экспорт: {})",
                            i + 1,
                            func.name,
                            func.export
                        );
                    }
                    bsl_analyzer::bsl_parser::ast::Declaration::Variable(var) => {
                        println!(
                            "  {}. Переменная: {:?} (экспорт: {})",
                            i + 1,
                            var.names,
                            var.export
                        );
                    }
                }
            }
        }
        None => {
            println!("❌ Ошибка парсинга");
        }
    }

    if !parse_result.diagnostics.is_empty() {
        println!("\n⚠️ Диагностика парсера:");
        for diag in &parse_result.diagnostics {
            println!("  - {:?}: {}", diag.severity, diag.message);
        }
    }

    println!("\n🎯 Тестирование завершено");
    Ok(())
}
