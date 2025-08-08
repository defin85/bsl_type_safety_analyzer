// Тест CLI интеграции с новым BslAnalyzer
use bsl_analyzer::unified_index::UnifiedIndexBuilder;
use bsl_analyzer::{analyze_file, BslAnalyzer};
use std::path::Path;

fn main() -> anyhow::Result<()> {
    println!("🧪 Тест CLI интеграции с новым BslAnalyzer");
    println!("=============================================");

    let test_file = "test_config/cli_test.bsl";

    if !Path::new(test_file).exists() {
        println!("❌ Тестовый файл не найден: {}", test_file);
        return Ok(());
    }

    // 1. Тест простого анализа файла (без UnifiedBslIndex)
    println!("\n🔧 Тест 1: Простой анализ файла");
    match analyze_file(test_file) {
        Ok(result) => {
            println!("✅ {}", result);
        }
        Err(e) => {
            println!("❌ Ошибка: {}", e);
        }
    }

    // 2. Тест расширенного анализа с UnifiedBslIndex
    println!("\n🔧 Тест 2: Расширенный анализ с UnifiedBslIndex");

    // Создаем UnifiedBslIndex (если возможно)
    let config_path = Path::new("examples/ConfTest");
    if config_path.exists() {
        println!("📚 Создание UnifiedBslIndex...");
        let mut builder = UnifiedIndexBuilder::new()?;
        let index = builder.build_index(config_path, "8.3.25")?;
        println!(
            "✅ UnifiedBslIndex создан: {} сущностей",
            index.get_all_entities().len()
        );

        // Создаем анализатор с индексом
        let mut analyzer = BslAnalyzer::with_index(index)?;

        // Читаем и анализируем файл
        let content = std::fs::read_to_string(test_file)?;
        match analyzer.analyze_code(&content, test_file) {
            Ok(()) => {
                let results = analyzer.get_results();
                println!("✅ Анализ выполнен:");
                println!("   - Ошибки: {}", results.error_count());
                println!("   - Предупреждения: {}", results.warning_count());

                if results.has_errors() || results.has_warnings() {
                    println!("\n📋 Найденные проблемы:");
                    println!("{}", results);
                }
            }
            Err(e) => {
                println!("❌ Ошибка анализа: {}", e);
            }
        }
    } else {
        println!(
            "⚠️  Конфигурация examples/ConfTest не найдена, пропускаем тест с UnifiedBslIndex"
        );

        // Простой тест без UnifiedBslIndex
        let mut analyzer = BslAnalyzer::new()?;
        let content = std::fs::read_to_string(test_file)?;

        match analyzer.analyze_code(&content, test_file) {
            Ok(()) => {
                let results = analyzer.get_results();
                println!("✅ Простой анализ выполнен:");
                println!("   - Ошибки: {}", results.error_count());
                println!("   - Предупреждения: {}", results.warning_count());

                if results.has_errors() || results.has_warnings() {
                    println!("\n📋 Найденные проблемы:");
                    println!("{}", results);
                }
            }
            Err(e) => {
                println!("❌ Ошибка анализа: {}", e);
            }
        }
    }

    // 3. Тест с подробными результатами
    println!("\n🔧 Тест 3: Подробный анализ с типами ошибок");
    let mut analyzer = BslAnalyzer::new()?;
    let content = std::fs::read_to_string(test_file)?;

    match analyzer.analyze_code(&content, test_file) {
        Ok(()) => {
            let (errors, warnings) = analyzer.get_errors_and_warnings();

            println!("📊 Детальная статистика:");
            println!("   - Всего ошибок: {}", errors.len());
            println!("   - Всего предупреждений: {}", warnings.len());

            // Подсчитываем типы ошибок
            let mut error_codes = std::collections::HashMap::new();
            for error in &errors {
                if let Some(code) = &error.error_code {
                    *error_codes.entry(code.clone()).or_insert(0) += 1;
                }
            }

            for warning in &warnings {
                if let Some(code) = &warning.error_code {
                    *error_codes.entry(code.clone()).or_insert(0) += 1;
                }
            }

            if !error_codes.is_empty() {
                println!("\n🏷️  Коды ошибок:");
                for (code, count) in &error_codes {
                    println!("   - {}: {} раз", code, count);
                }
            }

            // Показываем первые несколько ошибок
            if !errors.is_empty() {
                println!("\n🔴 Первые ошибки:");
                for (i, error) in errors.iter().take(3).enumerate() {
                    println!(
                        "   {}. {}:{} - {}",
                        i + 1,
                        error.position.line,
                        error.position.column,
                        error.message
                    );
                    if let Some(code) = &error.error_code {
                        println!("      Код: {}", code);
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Ошибка подробного анализа: {}", e);
        }
    }

    println!("\n🎯 Тест CLI интеграции завершен");
    println!(
        "💡 Для полной интеграции нужно обновить main.rs для использования нового BslAnalyzer"
    );

    Ok(())
}
