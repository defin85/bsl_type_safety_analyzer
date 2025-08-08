// Тестирование семантического анализа
use bsl_analyzer::bsl_parser::{SemanticAnalysisConfig, SemanticAnalyzer};
use bsl_analyzer::{BslAnalyzer, BslParser};

fn main() -> anyhow::Result<()> {
    println!("🧠 Тестирование семантического анализа BSL");
    println!("==========================================");

    // Тест 1: Семантический анализ файла с проблемами
    println!("\n📁 Тест 1: Анализ файла с семантическими проблемами");

    let content = std::fs::read_to_string("test_semantic.bsl")?;
    println!("📖 Файл загружен: {} символов", content.len());

    // Создаем анализатор
    let mut analyzer = BslAnalyzer::new()?;
    println!("🚀 BslAnalyzer создан");

    // Анализируем код
    match analyzer.analyze_code(&content, "test_semantic.bsl") {
        Ok(()) => {
            println!("✅ Анализ выполнен успешно");

            let results = analyzer.get_results();
            println!("📊 Результаты анализа:");
            println!("   - Ошибки: {}", results.error_count());
            println!("   - Предупреждения: {}", results.warning_count());

            if results.has_errors() || results.has_warnings() {
                println!("\n📋 Детали диагностики:");
                println!("{}", results);
            } else {
                println!(
                    "⚠️ Анализ не выявил ожидаемых проблем - проверьте семантический анализатор"
                );
            }
        }
        Err(e) => {
            println!("❌ Ошибка анализа: {}", e);
        }
    }

    // Тест 2: Прямое тестирование семантического анализатора
    println!("\n🔧 Тест 2: Прямое тестирование SemanticAnalyzer");

    let parser = BslParser::new()?;
    let parse_result = parser.parse(&content, "test_semantic.bsl");

    if let Some(ast) = parse_result.ast {
        println!("✅ AST получен");

        let config = SemanticAnalysisConfig::default();
        let mut semantic = SemanticAnalyzer::new(config);

        match semantic.analyze(&ast) {
            Ok(()) => {
                let diagnostics = semantic.get_diagnostics();
                println!("📊 Семантический анализ:");
                println!("   - Диагностика: {} элементов", diagnostics.len());

                if !diagnostics.is_empty() {
                    println!("\n📋 Детали семантического анализа:");
                    for diag in diagnostics {
                        println!(
                            "  - {:?} в {}:{}: {}",
                            diag.severity, diag.location.line, diag.location.column, diag.message
                        );
                    }
                } else {
                    println!("⚠️ Семантический анализ не выявил проблем");
                }
            }
            Err(e) => {
                println!("❌ Ошибка семантического анализа: {}", e);
            }
        }
    } else {
        println!("❌ Не удалось получить AST");
    }

    // Тест 3: Тестирование на корректном коде
    println!("\n✨ Тест 3: Анализ корректного кода");

    let correct_code = r#"
        Процедура КорректнаяПроцедура(Параметр1, Параметр2) Экспорт
            Перем ЛокальнаяПеременная;
            ЛокальнаяПеременная = Параметр1 + Параметр2;
            Сообщить(ЛокальнаяПеременная);
        КонецПроцедуры
    "#;

    let mut analyzer2 = BslAnalyzer::new()?;
    match analyzer2.analyze_code(correct_code, "correct.bsl") {
        Ok(()) => {
            let results = analyzer2.get_results();
            println!(
                "✅ Корректный код: {} ошибок, {} предупреждений",
                results.error_count(),
                results.warning_count()
            );

            if results.has_errors() || results.has_warnings() {
                println!("📋 Неожиданная диагностика:");
                println!("{}", results);
            }
        }
        Err(e) => {
            println!("❌ Ошибка анализа корректного кода: {}", e);
        }
    }

    println!("\n🎯 Тестирование семантического анализа завершено");
    Ok(())
}
