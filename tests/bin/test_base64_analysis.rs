/*!
# Test Base64 Analysis

Демонстрационная программа для полного анализа BSL файла с Base64 функциями.
Показывает работу интегрированного MethodVerifier на конкретном примере.
*/

use std::path::Path;
use anyhow::Result;
use bsl_analyzer::unified_index::{UnifiedIndexBuilder};
use bsl_analyzer::verifiers::method_verifier::{MethodVerifier, ArgumentInfo};

fn main() -> Result<()> {
    println!("🎯 ПОЛНЫЙ АНАЛИЗ BSL ФАЙЛА С BASE64 ФУНКЦИЯМИ");
    println!("{}",  "=".repeat(80));
    
    // Строим единый индекс
    println!("📖 Инициализация анализатора...");
    let config_path = Path::new("examples/ConfTest");
    let platform_version = "8.3.25";
    
    let mut builder = UnifiedIndexBuilder::new()?;
    let index = builder.build_index(config_path, platform_version)?;
    println!("✅ Загружено {} типов BSL в индекс", index.get_entity_count());
    
    // Создаем верификатор методов
    let verifier = MethodVerifier::new(index);
    println!("✅ MethodVerifier инициализирован");
    
    println!("\n{}", "=".repeat(80));
    println!("🔍 АНАЛИЗ BASE64 ФУНКЦИЙ В BSL КОДЕ");
    println!("{}", "=".repeat(80));
    
    // Тестируем Base64 функции
    analyze_base64_functions(&verifier);
    
    println!("\n{}", "=".repeat(80));
    println!("🔍 АНАЛИЗ МЕТОДОВ ОБЪЕКТОВ");
    println!("{}", "=".repeat(80));
    
    // Тестируем методы объектов из примера
    analyze_object_methods(&verifier);
    
    println!("\n{}", "=".repeat(80));
    println!("❌ АНАЛИЗ ОШИБОК В КОДЕ");
    println!("{}", "=".repeat(80));
    
    // Тестируем ошибочные вызовы
    analyze_error_cases(&verifier);
    
    println!("\n{}", "=".repeat(80));
    println!("📊 ИТОГОВАЯ СТАТИСТИКА АНАЛИЗА");
    println!("{}", "=".repeat(80));
    
    print_analysis_summary(&verifier);
    
    Ok(())
}

fn analyze_base64_functions(verifier: &MethodVerifier) {
    println!("📋 Проверка Base64 функций из test_base64_example.bsl:");
    println!("{}", "-".repeat(60));
    
    let base64_functions = vec![
        ("Base64Значение", vec!["Данные"]),
        ("Base64Строка", vec!["Данные"], ),
        ("ПолучитьДвоичныеДанныеИзBase64Строки", vec!["КодированнаяСтрока"]),
    ];
    
    for (func_name, params) in base64_functions {
        // Проверяем как глобальную функцию через Global контекст
        let exists = verifier.verify_method_exists("Глобальный контекст", func_name);
        
        if exists {
            if let Some(signature) = verifier.get_method_signature("Глобальный контекст", func_name) {
                println!("✅ {}: {}", func_name, signature);
                
                // Тестируем вызов с параметрами
                let arguments: Vec<ArgumentInfo> = params.iter().enumerate().map(|(i, param)| {
                    ArgumentInfo {
                        arg_type: "Строка".to_string(),
                        value: Some(format!("\"{}\"", param)),
                        position: i,
                    }
                }).collect();
                
                let mut verifier_mut = create_verifier_copy(verifier);
                let result = verifier_mut.verify_call("Глобальный контекст", func_name, &arguments, 1);
                
                if result.is_valid {
                    println!("   ✅ Вызов с параметрами: КОРРЕКТЕН");
                } else {
                    println!("   ❌ Вызов с параметрами: ОШИБКА - {}", 
                        result.error_message.unwrap_or("Неизвестная ошибка".to_string()));
                }
            } else {
                println!("⚠️  {}: найдена, но сигнатура недоступна", func_name);
            }
        } else {
            println!("❌ {}: НЕ НАЙДЕНА", func_name);
        }
    }
}

fn analyze_object_methods(verifier: &MethodVerifier) {
    println!("📋 Проверка методов объектов из примера:");
    println!("{}", "-".repeat(60));
    
    let object_methods = vec![
        ("Строка", "Длина", vec![]),
        ("Строка", "ВРег", vec![]),
        ("Массив", "Добавить", vec!["элемент"]),
        ("Массив", "Количество", vec![]),
    ];
    
    for (object_type, method_name, params) in object_methods {
        let exists = verifier.verify_method_exists(object_type, method_name);
        
        if exists {
            if let Some(signature) = verifier.get_method_signature(object_type, method_name) {
                println!("✅ {}.{}: {}", object_type, method_name, signature);
                
                if !params.is_empty() {
                    let arguments: Vec<ArgumentInfo> = params.iter().enumerate().map(|(i, param)| {
                        ArgumentInfo {
                            arg_type: "Произвольный".to_string(),
                            value: Some(format!("\"{}\"", param)),
                            position: i,
                        }
                    }).collect();
                    
                    let mut verifier_mut = create_verifier_copy(verifier); 
                    let result = verifier_mut.verify_call(object_type, method_name, &arguments, 1);
                    
                    if result.is_valid {
                        println!("   ✅ Вызов: {}.{}({})", object_type, method_name, 
                            params.join(", "));
                    } else {
                        println!("   ❌ Вызов: ОШИБКА - {}", 
                            result.error_message.unwrap_or("Неизвестная ошибка".to_string()));
                    }
                } else {
                    println!("   ✅ Вызов: {}.{}() - без параметров", object_type, method_name);
                }
            } else {
                println!("⚠️  {}.{}: найден, но сигнатура недоступна", object_type, method_name);
            }
        } else {
            println!("❌ {}.{}: НЕ НАЙДЕН", object_type, method_name);
        }
    }
}

fn analyze_error_cases(verifier: &MethodVerifier) {
    println!("📋 Проверка ошибочных вызовов из примера:");
    println!("{}", "-".repeat(60));
    
    // Тестируем несуществующие функции
    test_error_case(verifier, "Глобальный контекст", "Base64НесуществующаяФункция", 
        vec!["Данные"], "Несуществующая Base64 функция");
    
    test_error_case(verifier, "Глобальный контекст", "НесуществующаяФункция", 
        vec![], "Несуществующая глобальная функция");
    
    // Тестируем неправильное количество параметров для Base64Значение
    test_error_case(verifier, "Глобальный контекст", "Base64Значение", 
        vec![], "Base64Значение без параметров");
    
    test_error_case(verifier, "Глобальный контекст", "Base64Значение", 
        vec!["Данные", "лишний параметр"], "Base64Значение с лишним параметром");
    
    // Тестируем несуществующие методы объектов
    test_error_case(verifier, "Строка", "НесуществующийМетод", 
        vec![], "Несуществующий метод строки");
}

fn test_error_case(verifier: &MethodVerifier, object_type: &str, method_name: &str, 
    params: Vec<&str>, description: &str) {
    
    let arguments: Vec<ArgumentInfo> = params.iter().enumerate().map(|(i, param)| {
        ArgumentInfo {
            arg_type: "Произвольный".to_string(),
            value: Some(format!("\"{}\"", param)),
            position: i,
        }
    }).collect();
    
    let mut verifier_mut = create_verifier_copy(verifier);
    let result = verifier_mut.verify_call(object_type, method_name, &arguments, 1);
    
    if !result.is_valid {
        println!("✅ {}: КОРРЕКТНО ОБНАРУЖЕНА ОШИБКА", description);
        if let Some(error) = &result.error_message {
            println!("   📝 Сообщение: {}", error);
        }
        if !result.suggestions.is_empty() {
            println!("   💡 Предложения: {}", result.suggestions.join("; "));
        }
    } else {
        println!("❌ {}: ОШИБКА НЕ ОБНАРУЖЕНА (это проблема!)", description);
    }
}

fn print_analysis_summary(verifier: &MethodVerifier) {
    println!("📊 Статистика доступных типов и методов:");
    println!("{}", "-".repeat(60));
    
    // Показываем статистику по ключевым типам
    let key_types = vec!["Строка", "Массив", "ТаблицаЗначений", "ЧтениеXML"];
    
    for type_name in key_types {
        if verifier.verify_object_type(type_name) {
            let methods = verifier.get_available_methods(type_name);
            println!("✅ {}: {} методов доступно", type_name, methods.len());
            
            if methods.len() <= 5 {
                println!("   Методы: {}", methods.join(", "));
            } else {
                let sample: Vec<String> = methods.into_iter().take(3).collect();
                println!("   Примеры методов: {}, ... (и другие)", sample.join(", "));
            }
        } else {
            println!("❌ {}: тип не найден", type_name);
        }
    }
    
    // Информация о Global контексте
    if verifier.verify_object_type("Глобальный контекст") {
        let global_methods = verifier.get_available_methods("Глобальный контекст");
        println!("✅ Глобальный контекст: {} функций доступно", global_methods.len());
        
        // Подсчитываем Base64 функции
        let base64_count = global_methods.iter()
            .filter(|method| method.to_lowercase().contains("base64"))
            .count();
        println!("   📊 Из них Base64 функций: {}", base64_count);
    }
}

// Вспомогательная функция для создания мутабельной копии верификатора
fn create_verifier_copy(verifier: &MethodVerifier) -> MethodVerifier {
    MethodVerifier::new(verifier.unified_index.clone())
}