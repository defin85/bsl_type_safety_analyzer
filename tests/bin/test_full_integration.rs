// Полный тест интеграции BSL анализатора с UnifiedBslIndex
use bsl_analyzer::unified_index::UnifiedIndexBuilder;
use bsl_analyzer::BslAnalyzer;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    println!("🎯 Полный тест интеграции BSL Type Safety Analyzer v1.2.0");
    println!("===========================================================");

    // Создаем комплексный BSL код для тестирования
    let test_code = r#"
// Полный тест валидации типов и методов

#Область ПолныйТест

// 1. Функция с дублированными параметрами (должна быть ошибка BSL010)
Функция ТестДублированныеПараметры(Параметр1, Параметр2, Параметр1) Экспорт
    Возврат Параметр1 + Параметр2;
КонецФункции

// 2. Процедура с корректными вызовами методов платформенных типов
Процедура КорректныеВызовыМетодов() Экспорт
    
    // Массив - все методы существуют
    Массив = Новый Массив();
    Массив.Добавить("Элемент1");
    Массив.Добавить("Элемент2");
    Количество = Массив.Количество();
    Элемент = Массив.Получить(0);
    
    // Строка - все методы существуют  
    Строка = "Тестовая строка";
    ВерхняяСтрока = Строка.ВРег();
    Позиция = Строка.Найти("строка");
    
КонецПроцедуры

// 3. Процедура с некорректными вызовами методов (должны быть ошибки BSL003, BSL004)
Процедура НекорректныеВызовыМетодов() Экспорт
    
    // Вызов несуществующего метода на массиве
    Массив = Новый Массив();
    Массив.НесуществующийМетод(); // BSL003
    
    // Вызов метода с неверным количеством параметров
    Массив.Добавить("Элемент", "ЛишнийПараметр"); // BSL004 (если парсер поддерживает аргументы)
    
    // Вызов на неизвестном типе
    НеизвестныйТип.СделатьЧтоТо(); // BSL002 Warning
    
КонецПроцедуры

// 4. Процедура с неинициализированными переменными (должны быть предупреждения BSL009)
Процедура НеинициализированныеПеременные() Экспорт
    
    Перем НеинициализированнаяПеременная;
    Перем ИнициализированнаяПеременная;
    
    // Инициализируем только одну переменную
    ИнициализированнаяПеременная = "Значение";
    
    // Используем неинициализированную переменную (должно быть предупреждение)
    Если НеинициализированнаяПеременная <> Неопределено Тогда
        Сообщить("Проблема");
    КонецЕсли;
    
    // Используем инициализированную переменную (должно быть OK)
    Сообщить(ИнициализированнаяПеременная);
    
КонецПроцедуры

// 5. Процедура с необъявленными переменными (должны быть ошибки BSL007)
Процедура НеобъявленныеПеременные() Экспорт
    
    // Использование необъявленной переменной
    НеобъявленнаяПеременная = "Ошибка"; // BSL007
    Сообщить(НеобъявленнаяПеременная);
    
КонецПроцедуры

#КонецОбласти
"#;

    println!("📖 Тестовый код подготовлен: {} символов", test_code.len());

    // Шаг 1: Создаем UnifiedBslIndex
    println!("\n🔧 Создание UnifiedBslIndex...");
    let config_path = Path::new("examples/ConfTest");
    let platform_version = "8.3.25";

    let mut builder = UnifiedIndexBuilder::new()?;
    let index = builder.build_index(config_path, platform_version)?;

    println!("✅ UnifiedBslIndex создан:");
    println!("   - Всего сущностей: {}", index.get_all_entities().len());

    // Проверяем доступность ключевых типов
    let key_types = ["Массив", "Array", "Строка", "String"];
    for type_name in &key_types {
        if let Some(entity) = index.find_entity(type_name) {
            let methods = index.get_all_methods(&entity.qualified_name);
            println!("   - {}: {} методов", type_name, methods.len());
        }
    }

    // Шаг 2: Создаем BslAnalyzer с UnifiedBslIndex
    println!("\n🔧 Создание BslAnalyzer с интеграцией UnifiedBslIndex...");
    let mut analyzer = BslAnalyzer::with_index(index)?;

    // Шаг 3: Выполняем полный анализ
    println!("\n🔍 Выполняем полный анализ BSL кода...");
    match analyzer.analyze_code(test_code, "test_full_integration.bsl") {
        Ok(()) => {
            println!("✅ Анализ выполнен успешно");

            let results = analyzer.get_results();
            println!("\n📊 Результаты анализа:");
            println!("   - Ошибки: {}", results.error_count());
            println!("   - Предупреждения: {}", results.warning_count());

            if results.has_errors() || results.has_warnings() {
                println!("\n📋 Подробные результаты:");
                println!("{}", results);
            }

            // Подсчитываем типы ошибок
            let (errors, warnings) = analyzer.get_errors_and_warnings();

            let mut bsl003_count = 0; // Неизвестные методы
            let mut bsl004_count = 0; // Неверное количество параметров
            let mut bsl007_count = 0; // Необъявленные переменные
            let mut bsl009_count = 0; // Неинициализированные переменные
            let mut bsl010_count = 0; // Дублированные параметры

            for error in &errors {
                if let Some(code) = &error.error_code {
                    match code.as_str() {
                        "BSL003" => bsl003_count += 1,
                        "BSL004" => bsl004_count += 1,
                        "BSL007" => bsl007_count += 1,
                        "BSL009" => bsl009_count += 1,
                        "BSL010" => bsl010_count += 1,
                        _ => {}
                    }
                }
            }

            for warning in &warnings {
                if let Some(code) = &warning.error_code {
                    match code.as_str() {
                        "BSL007" => bsl007_count += 1,
                        "BSL009" => bsl009_count += 1,
                        _ => {}
                    }
                }
            }

            println!("\n🎯 Проверка типов ошибок:");
            println!(
                "   - BSL003 (Неизвестные методы): {} {}",
                bsl003_count,
                if bsl003_count > 0 { "✅" } else { "❌" }
            );
            println!(
                "   - BSL004 (Неверные параметры): {} {}",
                bsl004_count,
                if bsl004_count > 0 { "✅" } else { "❌" }
            );
            println!(
                "   - BSL007 (Необъявленные переменные): {} {}",
                bsl007_count,
                if bsl007_count > 0 { "✅" } else { "❌" }
            );
            println!(
                "   - BSL009 (Неинициализированные переменные): {} {}",
                bsl009_count,
                if bsl009_count > 0 { "✅" } else { "❌" }
            );
            println!(
                "   - BSL010 (Дублированные параметры): {} {}",
                bsl010_count,
                if bsl010_count > 0 { "✅" } else { "❌" }
            );

            // Шаг 4: Проверяем функциональность без индекса
            println!("\n🔧 Тест без UnifiedBslIndex (для сравнения)...");
            let mut simple_analyzer = BslAnalyzer::new()?;
            simple_analyzer.analyze_code(test_code, "test_simple.bsl")?;

            let simple_results = simple_analyzer.get_results();
            println!("📊 Результаты без UnifiedBslIndex:");
            println!("   - Ошибки: {}", simple_results.error_count());
            println!("   - Предупреждения: {}", simple_results.warning_count());

            println!("\n✅ Сравнение:");
            println!(
                "   - С UnifiedBslIndex: {} ошибок, {} предупреждений",
                results.error_count(),
                results.warning_count()
            );
            println!(
                "   - Без UnifiedBslIndex: {} ошибок, {} предупреждений",
                simple_results.error_count(),
                simple_results.warning_count()
            );

            let improvement = results.error_count() + results.warning_count()
                - simple_results.error_count()
                - simple_results.warning_count();

            if improvement > 0 {
                println!(
                    "   🎯 Улучшение: +{} дополнительных проблем обнаружено",
                    improvement
                );
            }
        }
        Err(e) => {
            println!("❌ Ошибка анализа: {}", e);
            return Err(e);
        }
    }

    println!("\n🎯 Полный тест интеграции завершен");
    println!("✅ BSL Type Safety Analyzer v1.2.0 - Расширенная семантика и валидация методов активированы!");

    Ok(())
}
