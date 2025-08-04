/*!
# Test Method Verifier

Тестовая программа для проверки интегрированного MethodVerifier с UnifiedBslIndex.
Проверяет корректность работы всех методов верификации.
*/

use std::path::Path;
use anyhow::Result;
use bsl_analyzer::unified_index::{UnifiedIndexBuilder};
use bsl_analyzer::verifiers::method_verifier::{MethodVerifier, ArgumentInfo};

fn main() -> Result<()> {
    println!("🚀 ТЕСТИРОВАНИЕ METHOD VERIFIER С UNIFIEDBSLINDEX");
    println!("{}", "=".repeat(60));
    
    // Строим единый индекс для тестирования
    println!("📖 Строим UnifiedBslIndex...");
    let config_path = Path::new("examples/ConfTest");
    let platform_version = "8.3.25";
    
    let builder = UnifiedIndexBuilder::new()?;
    let index = builder.build_index(config_path, platform_version)?;
    
    println!("✅ Индекс построен: {} типов", index.get_entity_count());
    
    // Создаем верификатор методов
    let verifier = MethodVerifier::new(index);
    println!("✅ MethodVerifier создан");
    
    println!("\n{}", "=".repeat(60));
    println!("🔍 ТЕСТИРОВАНИЕ МЕТОДОВ ВЕРИФИКАЦИИ");
    println!("{}", "=".repeat(60));
    
    // Тест 1: Проверка существования типов
    test_object_type_verification(&verifier);
    
    // Тест 2: Проверка существования методов
    test_method_existence(&verifier);
    
    // Тест 3: Получение сигнатур методов
    test_method_signatures(&verifier);
    
    // Тест 4: Получение списка доступных методов
    test_available_methods(&verifier);
    
    // Тест 5: Проверка совместимости типов
    test_type_compatibility(&verifier);
    
    // Тест 6: Полная проверка вызова метода
    test_method_call_verification(&verifier);
    
    println!("\n{}", "=".repeat(60));
    println!("🎯 ТЕСТИРОВАНИЕ ЗАВЕРШЕНО");
    println!("{}", "=".repeat(60));
    
    Ok(())
}

fn test_object_type_verification(verifier: &MethodVerifier) {
    println!("\n📋 ТЕСТ 1: Проверка существования типов");
    println!("{}", "-".repeat(40));
    
    let test_types = vec![
        ("Строка", true),
        ("String", true),
        ("Массив", true), 
        ("Array", true),
        ("ТаблицаЗначений", true),
        ("ValueTable", true),
        ("ЧтениеXML", true),
        ("XMLReader", true),
        ("НесуществующийТип", false),
    ];
    
    for (type_name, expected) in test_types {
        let exists = verifier.verify_object_type(type_name);
        let status = if exists == expected { "✅" } else { "❌" };
        println!("{} Тип '{}': найден={}, ожидалось={}", 
            status, type_name, exists, expected);
    }
}

fn test_method_existence(verifier: &MethodVerifier) {
    println!("\n📋 ТЕСТ 2: Проверка существования методов");
    println!("{}", "-".repeat(40));
    
    let test_cases = vec![
        ("Строка", "Длина", true),
        ("Строка", "ВРег", true),
        ("Строка", "НРег", true),
        ("String", "Длина", true),
        ("Массив", "Добавить", true),
        ("Массив", "Количество", true),
        ("Array", "Добавить", true),
        ("ТаблицаЗначений", "Добавить", true),
        ("ТаблицаЗначений", "Очистить", true),
        ("ЧтениеXML", "Прочитать", true),
        ("ЧтениеXML", "УстановитьСтроку", true),
        ("Строка", "НесуществующийМетод", false),
        ("НесуществующийТип", "Метод", false),
    ];
    
    for (object_type, method_name, expected) in test_cases {
        let exists = verifier.verify_method_exists(object_type, method_name);
        let status = if exists == expected { "✅" } else { "❌" };
        println!("{} {}.{}: найден={}, ожидалось={}", 
            status, object_type, method_name, exists, expected);
    }
}

fn test_method_signatures(verifier: &MethodVerifier) {
    println!("\n📋 ТЕСТ 3: Получение сигнатур методов");
    println!("{}", "-".repeat(40));
    
    let test_methods = vec![
        ("Строка", "Длина"),
        ("Строка", "ВРег"),
        ("Массив", "Добавить"),
        ("Массив", "Количество"),
        ("ТаблицаЗначений", "Добавить"),
        ("ЧтениеXML", "Прочитать"),
    ];
    
    for (object_type, method_name) in test_methods {
        if let Some(signature) = verifier.get_method_signature(object_type, method_name) {
            println!("✅ {}.{}: {}", object_type, method_name, signature);
        } else {
            println!("❌ {}.{}: сигнатура не найдена", object_type, method_name);
        }
    }
}

fn test_available_methods(verifier: &MethodVerifier) {
    println!("\n📋 ТЕСТ 4: Получение списка доступных методов");
    println!("{}", "-".repeat(40));
    
    let test_types = vec!["Строка", "Массив", "ТаблицаЗначений"];
    
    for type_name in test_types {
        let methods = verifier.get_available_methods(type_name);
        println!("✅ {}: {} методов", type_name, methods.len());
        
        if methods.len() <= 10 {
            println!("   Методы: {}", methods.join(", "));
        } else {
            let first_methods: Vec<String> = methods.into_iter().take(10).collect();
            println!("   Первые 10 методов: {}", first_methods.join(", "));
        }
    }
}

fn test_type_compatibility(verifier: &MethodVerifier) {
    println!("\n📋 ТЕСТ 5: Проверка совместимости типов");
    println!("{}", "-".repeat(40));
    
    let test_cases = vec![
        ("Строка", "Строка", true),
        ("String", "Строка", true),
        ("Массив", "Array", true),
        ("Строка", "Число", false),
        ("Массив", "Строка", false),
    ];
    
    for (from_type, to_type, expected) in test_cases {
        let compatible = verifier.verify_type_compatibility(from_type, to_type);
        let status = if compatible == expected { "✅" } else { "❌" };
        println!("{} {} → {}: совместимо={}, ожидалось={}", 
            status, from_type, to_type, compatible, expected);
    }
}

fn test_method_call_verification(verifier: &MethodVerifier) {
    println!("\n📋 ТЕСТ 6: Полная проверка вызова метода");
    println!("{}", "-".repeat(40));
    
    // Тест успешного вызова без параметров
    test_method_call(verifier, "Строка", "Длина", &[], true);
    test_method_call(verifier, "Массив", "Количество", &[], true);
    
    // Тест успешного вызова с параметрами
    let add_args = vec![ArgumentInfo {
        arg_type: "Произвольный".to_string(),
        value: Some("\"элемент\"".to_string()),
        position: 0,
    }];
    test_method_call(verifier, "Массив", "Добавить", &add_args, true);
    
    // Тест несуществующего метода
    test_method_call(verifier, "Строка", "НесуществующийМетод", &[], false);
    
    // Тест несуществующего типа
    test_method_call(verifier, "НесуществующийТип", "Метод", &[], false);
}

fn test_method_call(verifier: &MethodVerifier, object_type: &str, method_name: &str, 
    arguments: &[ArgumentInfo], should_succeed: bool) {
    
    let mut verifier = verifier.clone(); // Нужна мутабельная ссылка для verify_call
    let result = verifier.verify_call(object_type, method_name, arguments, 1);
    
    let status = if result.is_valid == should_succeed { "✅" } else { "❌" };
    
    println!("{} {}.{}({}): успешно={}", 
        status, object_type, method_name, 
        arguments.len(), result.is_valid);
    
    if !result.is_valid && !result.suggestions.is_empty() {
        println!("   Предложения: {}", result.suggestions.join("; "));
    }
}

// Нужен Clone для MethodVerifier в тестах
#[allow(dead_code)]
trait MethodVerifierExt {
    fn clone(&self) -> Self;
}

// Простая реализация клонирования для тестов
impl MethodVerifierExt for MethodVerifier {
    fn clone(&self) -> Self {
        // Для тестов создаем новый экземпляр с тем же индексом
        // В реальности это не оптимально, но подходит для тестирования
        MethodVerifier::new(self.unified_index.clone())
    }
}