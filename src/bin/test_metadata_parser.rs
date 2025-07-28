/*!
Простой тест парсера метаданных
*/

use std::path::Path;
use std::fs;

fn main() {
    let report_path = std::env::args().nth(1).unwrap_or("examples/sample_config_report.txt".to_string());
    
    if !Path::new(&report_path).exists() {
        eprintln!("Файл {} не найден", report_path);
        return;
    }
    
    let content = fs::read_to_string(report_path).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    
    let mut current_object = String::new();
    let mut objects_found = 0;
    
    for line in lines {
        let trimmed = line.trim();
        
        if trimmed.is_empty() {
            continue;
        }
        
        // Проверяем объекты конфигурации
        if trimmed.contains(".") && !trimmed.starts_with(" ") {
            let parts: Vec<&str> = trimmed.split('.').collect();
            if parts.len() == 2 {
                let obj_type = parts[0];
                let obj_name = parts[1];
                
                // Проверяем известные типы
                let known_types = vec![
                    "Справочник", "Документ", "РегистрСведений", "РегистрНакопления",
                    "Отчет", "Обработка", "Перечисление", "ОбщийМодуль", "Константа", "Роль"
                ];
                
                if known_types.contains(&obj_type) {
                    current_object = trimmed.to_string();
                    objects_found += 1;
                    println!("Найден объект: {} (тип: {}, имя: {})", trimmed, obj_type, obj_name);
                }
            }
        }
        
        // Показываем структуру объекта
        if !current_object.is_empty() && line.starts_with(" ") {
            let indent_level = line.len() - line.trim_start().len();
            println!("  [indent={}] {}", indent_level, trimmed);
        }
    }
    
    println!("\nВсего найдено объектов: {}", objects_found);
}