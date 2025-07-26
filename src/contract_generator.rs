/*!
# Contract Generator Launcher

Единый запускатель для генерации контрактов метаданных 1С.
Портирован с Python проекта onec-contract-generator на Rust.

Поддерживает генерацию:
- Контракты метаданных объектов (справочники, документы и т.д.)
- Контракты форм
- Контракты модулей (заглушка)
*/

use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use serde_json;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

use crate::configuration::{
    MetadataReportParser, FormXmlParser, ModuleGenerator,
    MetadataContract, FormContract, ModuleContract
};

/// Компоненты для генерации
#[derive(Debug, Clone)]
pub struct GenerationComponents {
    pub metadata: bool,
    pub forms: bool,
    pub modules: bool,
}

impl Default for GenerationComponents {
    fn default() -> Self {
        Self {
            metadata: true,
            forms: true,
            modules: false, // По умолчанию выключено, так как это заглушка
        }
    }
}

/// Единый генератор контрактов
pub struct ContractGeneratorLauncher {
    conf_dir: PathBuf,
    report_path: Option<PathBuf>,
    output_dir: PathBuf,
    components: GenerationComponents,
}

impl ContractGeneratorLauncher {
    /// Создает новый генератор
    pub fn new<P: AsRef<Path>>(conf_dir: P, output_dir: P) -> Self {
        Self {
            conf_dir: conf_dir.as_ref().to_path_buf(),
            report_path: None,
            output_dir: output_dir.as_ref().to_path_buf(),
            components: GenerationComponents::default(),
        }
    }
    
    /// Устанавливает путь к отчету конфигурации
    pub fn with_report_path<P: AsRef<Path>>(mut self, report_path: P) -> Self {
        self.report_path = Some(report_path.as_ref().to_path_buf());
        self
    }
    
    /// Устанавливает компоненты для генерации
    pub fn with_components(mut self, components: GenerationComponents) -> Self {
        self.components = components;
        self
    }
    
    /// Запускает генерацию выбранных компонентов
    pub fn run_generation(&self) -> Result<GenerationResult> {
        println!("\n{}", style("=".repeat(60)).blue());
        println!("{}", style("🚀 ЗАПУСК ГЕНЕРАЦИИ КОНТРАКТОВ").bold().cyan());
        println!("{}", style("=".repeat(60)).blue());
        
        let mut result = GenerationResult::default();
        
        // Создаем выходную директорию
        std::fs::create_dir_all(&self.output_dir)
            .context("Failed to create output directory")?;
        
        // Генерация контрактов метаданных
        if self.components.metadata {
            println!("\n{}", style("📋 Генерация контрактов метаданных...").yellow());
            
            match self.generate_metadata_contracts() {
                Ok(contracts) => {
                    result.metadata_count = contracts.len();
                    result.metadata_success = true;
                    println!("{}", style(format!("✅ Сгенерировано {} контрактов метаданных", contracts.len())).green());
                }
                Err(e) => {
                    result.metadata_error = Some(e.to_string());
                    println!("{}", style(format!("❌ Ошибка генерации метаданных: {}", e)).red());
                }
            }
        }
        
        // Генерация контрактов форм
        if self.components.forms {
            println!("\n{}", style("📄 Генерация контрактов форм...").yellow());
            
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.cyan} {msg}")
                    .unwrap()
            );
            pb.set_message("Поиск форм...");
            
            match self.generate_form_contracts() {
                Ok(contracts) => {
                    pb.finish_and_clear();
                    result.forms_count = contracts.len();
                    result.forms_success = true;
                    println!("{}", style(format!("✅ Сгенерировано {} контрактов форм", contracts.len())).green());
                }
                Err(e) => {
                    pb.finish_and_clear();
                    result.forms_error = Some(e.to_string());
                    println!("{}", style(format!("❌ Ошибка генерации форм: {}", e)).red());
                }
            }
        }
        
        // Генерация контрактов модулей (заглушка)
        if self.components.modules {
            println!("\n{}", style("📦 Генерация контрактов модулей...").yellow());
            
            match self.generate_module_contracts() {
                Ok(_) => {
                    result.modules_success = true;
                    println!("{}", style("✅ Контракты модулей (заглушка) сгенерированы").green());
                }
                Err(e) => {
                    result.modules_error = Some(e.to_string());
                    println!("{}", style(format!("❌ Ошибка генерации модулей: {}", e)).red());
                }
            }
        }
        
        // Выводим итоговую статистику
        self.print_summary(&result);
        
        Ok(result)
    }
    
    /// Генерирует контракты метаданных
    fn generate_metadata_contracts(&self) -> Result<Vec<MetadataContract>> {
        let output_dir = self.output_dir.join("metadata");
        std::fs::create_dir_all(&output_dir)?;
        
        // Определяем путь к отчету
        let report_path = if let Some(ref path) = self.report_path {
            path.clone()
        } else {
            // Пытаемся найти отчет автоматически
            let possible_paths = vec![
                self.conf_dir.join("config_report.txt"),
                self.conf_dir.join("ОтчетПоКонфигурации.txt"),
                self.conf_dir.join("ConfigurationReport.txt"),
            ];
            
            possible_paths.into_iter()
                .find(|p| p.exists())
                .ok_or_else(|| anyhow::anyhow!("Configuration report not found"))?
        };
        
        // Парсим отчет
        let parser = MetadataReportParser::new()?;
        let contracts = parser.parse_report(report_path)?;
        
        // Сохраняем контракты
        for contract in &contracts {
            let filename = format!("{}.{}.json", contract.object_type, contract.name);
            let filepath = output_dir.join(filename);
            
            let json = serde_json::to_string_pretty(contract)?;
            std::fs::write(filepath, json)?;
        }
        
        Ok(contracts)
    }
    
    /// Генерирует контракты форм
    fn generate_form_contracts(&self) -> Result<Vec<FormContract>> {
        let output_dir = self.output_dir.join("forms");
        std::fs::create_dir_all(&output_dir)?;
        
        // Генерируем контракты форм
        let parser = FormXmlParser::new();
        let contracts = parser.generate_all_contracts(&self.conf_dir)?;
        
        // Сохраняем контракты
        for contract in &contracts {
            let filename = format!("{}.json", contract.name);
            let filepath = output_dir.join(filename);
            
            let json = serde_json::to_string_pretty(contract)?;
            std::fs::write(filepath, json)?;
        }
        
        Ok(contracts)
    }
    
    /// Генерирует контракты модулей (заглушка)
    fn generate_module_contracts(&self) -> Result<Vec<ModuleContract>> {
        let output_dir = self.output_dir.join("modules");
        
        let mut generator = ModuleGenerator::new(&self.conf_dir, &output_dir);
        generator.generate()?;
        generator.print_logs();
        
        Ok(vec![])
    }
    
    /// Выводит итоговую статистику
    fn print_summary(&self, result: &GenerationResult) {
        println!("\n{}", style("=".repeat(60)).blue());
        println!("{}", style("📊 ИТОГОВАЯ СТАТИСТИКА").bold().cyan());
        println!("{}", style("=".repeat(60)).blue());
        
        if self.components.metadata {
            let status = if result.metadata_success {
                style("✅ Успешно").green()
            } else {
                style("❌ Ошибка").red()
            };
            println!("Контракты метаданных: {} ({})", result.metadata_count, status);
        }
        
        if self.components.forms {
            let status = if result.forms_success {
                style("✅ Успешно").green()
            } else {
                style("❌ Ошибка").red()
            };
            println!("Контракты форм: {} ({})", result.forms_count, status);
        }
        
        if self.components.modules {
            let status = if result.modules_success {
                style("✅ Заглушка").green()
            } else {
                style("❌ Ошибка").red()
            };
            println!("Контракты модулей: {}", status);
        }
        
        println!("\nВыходная директория: {}", self.output_dir.display());
        println!("{}", style("=".repeat(60)).blue());
    }
}

/// Результат генерации
#[derive(Default)]
pub struct GenerationResult {
    pub metadata_count: usize,
    pub metadata_success: bool,
    pub metadata_error: Option<String>,
    
    pub forms_count: usize,
    pub forms_success: bool,
    pub forms_error: Option<String>,
    
    pub modules_count: usize,
    pub modules_success: bool,
    pub modules_error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_contract_generator_creation() {
        let temp_dir = TempDir::new().unwrap();
        let generator = ContractGeneratorLauncher::new(temp_dir.path(), temp_dir.path());
        
        assert_eq!(generator.conf_dir, temp_dir.path());
        assert_eq!(generator.output_dir, temp_dir.path());
    }
}