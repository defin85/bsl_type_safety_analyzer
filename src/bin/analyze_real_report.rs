/*!
Анализатор реального формата отчета конфигурации
*/

use std::path::PathBuf;
use clap::Parser;
use anyhow::Result;
use std::fs;
use encoding_rs::{UTF_16LE, UTF_8, WINDOWS_1251};

/// Анализ структуры реального отчета конфигурации
#[derive(Parser, Debug)]
#[command(name = "analyze-real-report")]
#[command(about = "Анализирует структуру реального отчета конфигурации 1С")]
struct Args {
    /// Путь к файлу отчета
    #[arg(short, long)]
    report: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    println!("Анализ отчета: {}", args.report.display());
    println!("{}", "=".repeat(80));
    
    // Читаем файл с правильной кодировкой
    let file_bytes = fs::read(&args.report)?;
    
    // Пробуем разные кодировки
    let content = if let (decoded, _, false) = UTF_16LE.decode(&file_bytes) {
        println!("Кодировка: UTF-16LE");
        decoded.into_owned()
    } else if let (decoded, _, false) = UTF_8.decode(&file_bytes) {
        println!("Кодировка: UTF-8");
        decoded.into_owned()
    } else if let (decoded, _, false) = WINDOWS_1251.decode(&file_bytes) {
        println!("Кодировка: Windows-1251");
        decoded.into_owned()
    } else {
        // Fallback to UTF-8 with replacements
        let (decoded, _, _) = UTF_8.decode(&file_bytes);
        println!("Кодировка: UTF-8 (с заменами)");
        decoded.into_owned()
    };
    
    let lines: Vec<&str> = content.lines().collect();
    
    let mut in_object = false;
    let mut current_object = String::new();
    let mut found_objects = 0;
    let mut found_attributes = 0;
    
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        
        // Ищем объекты конфигурации (начинаются с "-")
        if trimmed.starts_with("-") && trimmed.contains(".") {
            let object_line = trimmed.trim_start_matches("-").trim();
            
            // Проверяем, что это не вложенный элемент
            if !object_line.contains(".Реквизиты.") && !object_line.contains(".ТабличныеЧасти.") {
                if object_line.contains("Справочники.") || 
                   object_line.contains("Документы.") ||
                   object_line.contains("Константы.") ||
                   object_line.contains("Языки.") {
                    found_objects += 1;
                    current_object = object_line.to_string();
                    in_object = true;
                    println!("\n🔷 Объект #{}: {}", found_objects, object_line);
                    println!("  Строка: {}", i + 1);
                }
            } else if in_object && object_line.starts_with(&current_object) {
                // Это вложенный элемент текущего объекта
                if object_line.contains(".Реквизиты.") {
                    let parts: Vec<&str> = object_line.split('.').collect();
                    if parts.len() >= 4 {
                        found_attributes += 1;
                        println!("  📌 Реквизит: {}", parts[3]);
                        
                        // Ищем тип в следующих строках
                        for j in (i+1)..lines.len() {
                            let next_line = lines[j].trim();
                            if next_line.starts_with("Тип:") {
                                println!("     Тип: {}", next_line.strip_prefix("Тип:").unwrap().trim());
                                break;
                            }
                            if next_line.starts_with("-") || j > i + 10 {
                                break;
                            }
                        }
                    }
                }
            }
        } else if !trimmed.starts_with("-") && in_object {
            // Свойства объекта
            if trimmed.contains(":") && !trimmed.is_empty() {
                let parts: Vec<&str> = trimmed.splitn(2, ':').collect();
                if parts.len() == 2 {
                    let key = parts[0].trim();
                    let value = parts[1].trim().trim_matches('"');
                    if !value.is_empty() && key != "Тип" {
                        println!("  🔸 {}: {}", key, value);
                    }
                }
            }
        }
    }
    
    println!("\n{}", "=".repeat(80));
    println!("📊 Статистика:");
    println!("  - Найдено объектов: {}", found_objects);
    println!("  - Найдено реквизитов: {}", found_attributes);
    println!("  - Всего строк: {}", lines.len());
    
    Ok(())
}