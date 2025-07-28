/*!
# Documentation Integration Module

Интеграция парсера документации 1С из проекта 1c-help-parser.
Портировано с Python на Rust для работы с .hbk архивами документации.

## Основные компоненты

- `HbkArchiveParser` - Парсинг .hbk архивов документации 1С
- `BslSyntaxExtractor` - Извлечение синтаксиса BSL из HTML документации  
- `BslSyntaxDatabase` - База знаний методов, функций, объектов BSL

## Использование

```rust
use crate::docs_integration::{HbkArchiveParser, BslSyntaxExtractor};

// Создаем парсер архива документации
let mut parser = HbkArchiveParser::new("1C_Help.hbk");

// Анализируем структуру архива
let analysis = parser.analyze_archive()?;
println!("HTML files: {}", analysis.html_files_count);

// Извлекаем базу знаний BSL
let mut extractor = BslSyntaxExtractor::new("1C_Help.hbk");
let syntax_db = extractor.extract_syntax_database(Some(1000))?;

// Используем для автодополнения
let completions = syntax_db.get_completion_items("Сообщ");
```
*/

pub mod hbk_parser;
pub mod hbk_parser_full;
pub mod bsl_syntax_extractor;
pub mod chunked_writer;
pub mod chunked_loader;
pub mod hybrid_storage;

pub use hbk_parser::{HbkArchiveParser, HtmlContent, ArchiveAnalysis, FileInfo};
pub use bsl_syntax_extractor::{
    BslSyntaxExtractor, BslSyntaxDatabase, BslMethodInfo, BslObjectInfo,
    BslPropertyInfo, BslFunctionInfo, CompletionItem, ParameterInfo
};

use anyhow::Result;
use std::path::Path;

use chunked_loader::ChunkedDocsLoader;

/// Основной фасад для работы с документацией 1С
pub struct DocsIntegration {
    syntax_database: Option<BslSyntaxDatabase>,
    chunked_loader: Option<ChunkedDocsLoader>,
}

impl DocsIntegration {
    /// Создает новый экземпляр интеграции документации
    pub fn new() -> Self {
        Self {
            syntax_database: None,
            chunked_loader: None,
        }
    }
    
    /// Загружает документацию BSL из .hbk архива
    pub fn load_documentation<P: AsRef<Path>>(&mut self, hbk_path: P) -> Result<()> {
        tracing::info!("Loading BSL documentation from: {}", hbk_path.as_ref().display());
        
        let mut extractor = BslSyntaxExtractor::new(hbk_path);
        self.syntax_database = Some(extractor.extract_syntax_database(None)?);
        
        if let Some(db) = &self.syntax_database {
            tracing::info!(
                "Documentation loaded: {} methods, {} objects, {} functions",
                db.methods.len(),
                db.objects.len(), 
                db.functions.len()
            );
        }
        
        Ok(())
    }
    
    /// Загружает предварительно проиндексированную документацию из JSON
    pub fn load_indexed_documentation<P: AsRef<Path>>(&mut self, json_path: P) -> Result<()> {
        tracing::info!("Loading indexed documentation from: {}", json_path.as_ref().display());
        
        let content = std::fs::read_to_string(json_path)?;
        self.syntax_database = Some(serde_json::from_str(&content)?);
        
        if let Some(db) = &self.syntax_database {
            tracing::info!(
                "Indexed documentation loaded: {} methods, {} objects, {} functions",
                db.methods.len(),
                db.objects.len(),
                db.functions.len()
            );
        }
        
        Ok(())
    }
    
    /// Загружает документацию из chunked формата
    pub fn load_chunked_documentation<P: AsRef<Path>>(&mut self, docs_dir: P) -> Result<()> {
        tracing::info!("Loading chunked documentation from: {}", docs_dir.as_ref().display());
        
        let mut loader = ChunkedDocsLoader::new(docs_dir);
        loader.load_index()?;
        
        let stats = loader.get_statistics()?;
        tracing::info!(
            "Chunked documentation loaded: {} total items in {} categories",
            stats.total_items,
            stats.categories.len()
        );
        
        self.chunked_loader = Some(loader);
        
        Ok(())
    }
    
    /// Сохраняет проиндексированную документацию в JSON файл
    pub fn save_indexed_documentation<P: AsRef<Path>>(&self, json_path: P) -> Result<()> {
        if let Some(db) = &self.syntax_database {
            let json = serde_json::to_string_pretty(db)?;
            std::fs::write(json_path, json)?;
            tracing::info!("Documentation index saved successfully");
        } else {
            anyhow::bail!("No documentation loaded to save");
        }
        
        Ok(())
    }
    
    /// Получает информацию о методе для верификации
    pub fn get_method_info(&self, method_name: &str) -> Option<&BslMethodInfo> {
        self.syntax_database.as_ref()?.methods.get(method_name)
    }
    
    /// Получает информацию об объекте BSL
    pub fn get_object_info(&self, object_name: &str) -> Option<&BslObjectInfo> {
        self.syntax_database.as_ref()?.objects.get(object_name)
    }
    
    /// Получает список автодополнения для LSP
    pub fn get_completions(&self, prefix: &str) -> Vec<CompletionItem> {
        self.syntax_database.as_ref()
            .map(|db| db.get_completion_items(prefix))
            .unwrap_or_default()
    }
    
    /// Поиск методов по паттерну
    pub fn search_methods(&self, query: &str) -> Vec<&BslMethodInfo> {
        self.syntax_database.as_ref()
            .map(|db| db.search_methods(query))
            .unwrap_or_default()
    }
    
    /// Проверяет, загружена ли документация
    pub fn is_loaded(&self) -> bool {
        self.syntax_database.is_some()
    }
    
    /// Получает статистику загруженной документации
    pub fn get_statistics(&self) -> Option<DocumentationStatistics> {
        let db = self.syntax_database.as_ref()?;
        
        Some(DocumentationStatistics {
            methods_count: db.methods.len(),
            objects_count: db.objects.len(),
            properties_count: db.properties.len(),
            functions_count: db.functions.len(),
            operators_count: db.operators.len(),
            keywords_count: db.keywords.len(),
        })
    }
}

impl Default for DocsIntegration {
    fn default() -> Self {
        Self::new()
    }
}

/// Статистика документации BSL
#[derive(Debug, Clone)]
pub struct DocumentationStatistics {
    pub methods_count: usize,
    pub objects_count: usize,
    pub properties_count: usize,
    pub functions_count: usize,
    pub operators_count: usize,
    pub keywords_count: usize,
}

impl std::fmt::Display for DocumentationStatistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, 
            "BSL Documentation Statistics:\n\
             Methods: {}\n\
             Objects: {}\n\
             Properties: {}\n\
             Functions: {}\n\
             Operators: {}\n\
             Keywords: {}",
            self.methods_count,
            self.objects_count,
            self.properties_count,
            self.functions_count,
            self.operators_count,
            self.keywords_count
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use tempfile::TempDir; // Unused import
    
    #[test]
    fn test_docs_integration_creation() {
        let integration = DocsIntegration::new();
        assert!(!integration.is_loaded());
        assert!(integration.get_statistics().is_none());
    }
    
    #[test]
    fn test_get_completions_empty() {
        let integration = DocsIntegration::new();
        let completions = integration.get_completions("test");
        assert!(completions.is_empty());
    }
    
    #[test]
    fn test_search_methods_empty() {
        let integration = DocsIntegration::new();
        let methods = integration.search_methods("test");
        assert!(methods.is_empty());
    }
}