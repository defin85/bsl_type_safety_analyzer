/*!
# Module Generator

Генератор контрактов модулей 1С.
Портирован с Python проекта onec-contract-generator на Rust.

СТАТУС: ЗАГЛУШКА
Этот генератор зарезервирован для будущей реализации анализа исходного кода модулей.
Текущая функциональность (анализ метаданных) уже покрывается генераторами метаданных и форм.

Планы развития:
- Парсинг файлов модулей (.bsl, .os)
- Извлечение функций и процедур
- Анализ параметров и типов возвращаемых значений
- Создание API документации для модулей
- Анализ бизнес-логики и обработчиков событий
*/

use std::path::{Path, PathBuf};
use std::collections::HashMap;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;
use chrono::Utc;

/// Контракт модуля 1С (заглушка)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleContract {
    pub metadata_type: String, // Всегда "Module"
    pub name: String,
    pub module_type: String,
    pub status: String,
    pub description: String,
    pub generation_metadata: ModuleGenerationMetadata,
}

/// Метаданные генерации модульного контракта
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleGenerationMetadata {
    pub generated_at: String,
    pub generator_version: String,
}

/// Генератор контрактов модулей (заглушка)
pub struct ModuleGenerator {
    conf_dir: PathBuf,
    output_dir: PathBuf,
    logs: HashMap<String, Vec<String>>,
}

impl ModuleGenerator {
    /// Создает новый генератор контрактов модулей
    pub fn new<P: AsRef<Path>>(conf_dir: P, output_dir: P) -> Self {
        Self {
            conf_dir: conf_dir.as_ref().to_path_buf(),
            output_dir: output_dir.as_ref().to_path_buf(),
            logs: HashMap::new(),
        }
    }
    
    /// Добавляет сообщение в лог
    fn log(&mut self, category: &str, message: String) {
        self.logs.entry(category.to_string())
            .or_insert_with(Vec::new)
            .push(message);
    }
    
    /// Выводит сгруппированные логи
    pub fn print_logs(&self) {
        println!("\n📋 Сводка по генерации контрактов модулей:");
        println!("{}", "=".repeat(50));
        
        for (category, messages) in &self.logs {
            if !messages.is_empty() {
                println!("\n🔍 {} ({}):", category, messages.len());
                for msg in messages.iter().take(5) {
                    println!("  • {}", msg);
                }
                if messages.len() > 5 {
                    println!("  • ... и еще {} сообщений", messages.len() - 5);
                }
            }
        }
        
        println!("{}", "=".repeat(50));
    }
    
    /// Очищает целевую папку
    pub fn clean_output_directory(&mut self) -> Result<()> {
        if self.output_dir.exists() {
            self.log("info", format!("Очищаю целевую папку: {}", self.output_dir.display()));
            
            // Удаляем все JSON файлы
            let mut deleted_files = 0;
            for entry in WalkDir::new(&self.output_dir)
                .min_depth(1)
                .max_depth(1)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    std::fs::remove_file(path)?;
                    deleted_files += 1;
                }
            }
            
            self.log("success", format!("Целевая папка очищена: удалено {} файлов", deleted_files));
        } else {
            self.log("info", format!("Целевая папка не существует, будет создана: {}", self.output_dir.display()));
        }
        
        Ok(())
    }
    
    /// Генерирует контракты модулей (заглушка)
    pub fn generate(&mut self) -> Result<bool> {
        self.log("info", format!("Конфигурация: {}", self.conf_dir.display()));
        self.log("info", format!("Выходная директория: {}", self.output_dir.display()));
        
        // Очищаем целевую папку
        self.clean_output_directory()?;
        std::fs::create_dir_all(&self.output_dir)?;
        
        // Создаем информационный контракт о статусе
        let info_contract = ModuleContract {
            metadata_type: "ModuleGenerator".to_string(),
            status: "STUB".to_string(),
            name: "ModuleGenerator".to_string(),
            module_type: "Info".to_string(),
            description: "Генератор контрактов модулей - заглушка".to_string(),
            generation_metadata: ModuleGenerationMetadata {
                generated_at: Utc::now().to_rfc3339(),
                generator_version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };
        
        // Создаем информационный файл
        let info_path = self.output_dir.join("_module_generator_info.json");
        let json = serde_json::to_string_pretty(&info_contract)?;
        std::fs::write(info_path, json)?;
        
        self.log("info", "Генератор контрактов модулей - это заглушка".to_string());
        self.log("info", "Функциональность анализа метаданных покрывается другими генераторами".to_string());
        self.log("success", "Создан информационный файл о статусе генератора".to_string());
        
        Ok(true)
    }
    
    /// Генерирует все контракты (совместимость с другими генераторами)
    pub fn generate_all_contracts(&mut self) -> Result<Vec<ModuleContract>> {
        self.generate()?;
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_module_generator_creation() {
        let temp_dir = TempDir::new().unwrap();
        let generator = ModuleGenerator::new(temp_dir.path(), temp_dir.path());
        assert!(generator.logs.is_empty());
    }
    
    #[test]
    fn test_module_generator_stub() {
        let temp_dir = TempDir::new().unwrap();
        let mut generator = ModuleGenerator::new(temp_dir.path(), temp_dir.path());
        
        let result = generator.generate().unwrap();
        assert!(result);
        
        // Проверяем создание информационного файла
        let info_file = temp_dir.path().join("_module_generator_info.json");
        assert!(info_file.exists());
    }
}