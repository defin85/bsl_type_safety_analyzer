/*!
# HBK Archive Parser

Парсер архивов документации 1С (.hbk файлы).
Портирован с Python проекта 1c-help-parser на Rust.

Основные возможности:
- Парсинг .hbk архивов с HTML документацией
- Извлечение структурированного контента из HTML
- Анализ структуры архивов документации
- Кэширование содержимого файлов

## Использование

```rust
let mut parser = HbkArchiveParser::new("1C_Help.hbk");
let analysis = parser.analyze_archive()?;
let sample_content = parser.extract_sample_content(100)?;
```
*/

use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use zip::ZipArchive;
use scraper::{Html, Selector};
use anyhow::{Context, Result};

/// Основной парсер HBK архивов (замена Python HBKParser)
pub struct HbkArchiveParser {
    archive_path: PathBuf,
    content_cache: HashMap<String, String>,
}

/// Структура для хранения распарсенного HTML контента
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HtmlContent {
    pub title: Option<String>,
    pub syntax: Vec<String>,
    pub fields: Vec<FieldInfo>,
    pub description: Option<String>,
    pub examples: Vec<String>,
    pub object_type: Option<String>,
    pub availability: Option<String>,
    pub version: Option<String>,
}

/// Информация о поле объекта/метода
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldInfo {
    pub name: String,
    pub field_type: String,
    pub description: Option<String>,
}

/// Результат анализа архива
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveAnalysis {
    pub total_files: usize,
    pub html_files_count: usize,
    pub file_categories: HashMap<String, usize>,
    pub largest_files: Vec<FileInfo>,
    pub sample_content: Vec<HtmlContent>,
}

/// Информация о файле в архиве
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub size: u64,
    pub file_type: String,
}

impl HbkArchiveParser {
    /// Создает новый парсер HBK архива
    pub fn new<P: AsRef<Path>>(archive_path: P) -> Self {
        Self {
            archive_path: archive_path.as_ref().to_path_buf(),
            content_cache: HashMap::new(),
        }
    }
    
    /// Анализирует структуру HBK архива (замена Python analyze_structure)
    pub fn analyze_archive(&mut self) -> Result<ArchiveAnalysis> {
        tracing::info!("Analyzing HBK archive: {}", self.archive_path.display());
        
        let file = std::fs::File::open(&self.archive_path)
            .with_context(|| format!("Failed to open archive: {}", self.archive_path.display()))?;
        let mut archive = ZipArchive::new(file)
            .context("Failed to read ZIP archive")?;
        
        let mut analysis = ArchiveAnalysis {
            total_files: archive.len(),
            html_files_count: 0,
            file_categories: HashMap::new(),
            largest_files: Vec::new(),
            sample_content: Vec::new(),
        };
        
        // Анализируем каждый файл в архиве
        for i in 0..archive.len() {
            let file = archive.by_index(i)
                .with_context(|| format!("Failed to access file at index {}", i))?;
            
            let file_info = FileInfo {
                name: file.name().to_string(),
                size: file.size(),
                file_type: self.detect_file_type(file.name()),
            };
            
            // Считаем категории файлов
            *analysis.file_categories.entry(file_info.file_type.clone()).or_insert(0) += 1;
            
            if file_info.file_type == "html" {
                analysis.html_files_count += 1;
            }
            
            // Отслеживаем самые большие файлы
            if analysis.largest_files.len() < 10 || 
               file_info.size > analysis.largest_files.last().map(|f| f.size).unwrap_or(0) {
                analysis.largest_files.push(file_info);
                analysis.largest_files.sort_by(|a, b| b.size.cmp(&a.size));
                analysis.largest_files.truncate(10);
            }
        }
        
        tracing::info!(
            "Archive analyzed: {} total files, {} HTML files",
            analysis.total_files,
            analysis.html_files_count
        );
        
        Ok(analysis)
    }
    
    /// Парсинг HTML контента из файла архива (замена Python parse_html_content)
    pub fn parse_html_content(&mut self, file_name: &str) -> Result<HtmlContent> {
        let html_content = self.extract_file_content(file_name)?;
        let document = Html::parse_document(&html_content);
        
        let mut content = HtmlContent {
            title: None,
            syntax: Vec::new(),
            fields: Vec::new(),
            description: None,
            examples: Vec::new(),
            object_type: None,
            availability: None,
            version: None,
        };
        
        // Извлекаем заголовок
        if let Ok(title_selector) = Selector::parse("title") {
            if let Some(title_element) = document.select(&title_selector).next() {
                content.title = Some(title_element.text().collect::<Vec<_>>().join(" ").trim().to_string());
            }
        }
        
        // Извлекаем основной заголовок страницы
        if content.title.is_none() {
            if let Ok(h1_selector) = Selector::parse("h1") {
                if let Some(h1_element) = document.select(&h1_selector).next() {
                    content.title = Some(h1_element.text().collect::<Vec<_>>().join(" ").trim().to_string());
                }
            }
        }
        
        // Извлекаем секции синтаксиса
        if let Ok(syntax_selector) = Selector::parse(".V8SH_chapter, .syntax, .code") {
            for syntax_element in document.select(&syntax_selector) {
                let syntax_text = syntax_element.text().collect::<Vec<_>>().join(" ").trim().to_string();
                if syntax_text.contains("Синтаксис:") || syntax_text.contains("Syntax:") {
                    content.syntax.push(syntax_text);
                }
            }
        }
        
        // Извлекаем описание
        if let Ok(desc_selector) = Selector::parse("p, .description, .V8SH_text") {
            let mut descriptions = Vec::new();
            for desc_element in document.select(&desc_selector).take(3) { // Берем первые 3 абзаца
                let desc_text = desc_element.text().collect::<Vec<_>>().join(" ").trim().to_string();
                if !desc_text.is_empty() && desc_text.len() > 10 {
                    descriptions.push(desc_text);
                }
            }
            if !descriptions.is_empty() {
                content.description = Some(descriptions.join(" "));
            }
        }
        
        // Извлекаем примеры кода
        if let Ok(example_selector) = Selector::parse("pre, .example, .code-example") {
            for example_element in document.select(&example_selector) {
                let example_text = example_element.text().collect::<Vec<_>>().join("\n").trim().to_string();
                if !example_text.is_empty() && example_text.len() > 10 {
                    content.examples.push(example_text);
                }
            }
        }
        
        // Извлекаем доступность (Клиент/Сервер)
        if let Some(desc) = &content.description {
            if desc.contains("Доступность:") {
                if let Some(start) = desc.find("Доступность:") {
                    let availability_part = &desc[start..];
                    if let Some(end) = availability_part.find('.') {
                        content.availability = Some(availability_part[..end].to_string());
                    }
                }
            }
        }
        
        // Пытаемся определить тип объекта по контенту
        if let Some(title) = &content.title {
            content.object_type = Some(self.detect_object_type(title));
        }
        
        Ok(content)
    }
    
    /// Извлекает образцы HTML файлов для анализа (замена Python extract_sample_files)
    pub fn extract_sample_content(&mut self, max_files: usize) -> Result<Vec<HtmlContent>> {
        tracing::info!("Extracting sample content from {} files", max_files);
        
        let file = std::fs::File::open(&self.archive_path)?;
        let mut archive = ZipArchive::new(file)?;
        
        let mut sample_content = Vec::new();
        let mut html_count = 0;
        
        for i in 0..archive.len() {
            if html_count >= max_files {
                break;
            }
            
            let file = archive.by_index(i)?;
            if file.name().ends_with(".html") || file.name().ends_with(".htm") {
                match self.parse_html_content(file.name()) {
                    Ok(content) => {
                        sample_content.push(content);
                        html_count += 1;
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse HTML file {}: {}", file.name(), e);
                    }
                }
            }
        }
        
        tracing::info!("Extracted content from {} HTML files", sample_content.len());
        Ok(sample_content)
    }
    
    /// Извлекает содержимое файла из архива с кэшированием
    fn extract_file_content(&mut self, file_name: &str) -> Result<String> {
        // Проверяем кэш
        if let Some(cached) = self.content_cache.get(file_name) {
            return Ok(cached.clone());
        }
        
        let file = std::fs::File::open(&self.archive_path)?;
        let mut archive = ZipArchive::new(file)?;
        let mut file = archive.by_name(file_name)
            .with_context(|| format!("File not found in archive: {}", file_name))?;
        
        let mut content = String::new();
        file.read_to_string(&mut content)
            .with_context(|| format!("Failed to read file content: {}", file_name))?;
        
        // Кэшируем содержимое
        self.content_cache.insert(file_name.to_string(), content.clone());
        Ok(content)
    }
    
    /// Определяет тип файла по расширению
    fn detect_file_type(&self, file_name: &str) -> String {
        match file_name.split('.').last() {
            Some("html") | Some("htm") => "html".to_string(),
            Some("css") => "css".to_string(),
            Some("js") => "javascript".to_string(),
            Some("png") | Some("jpg") | Some("gif") | Some("jpeg") => "image".to_string(),
            Some("xml") => "xml".to_string(),
            Some("json") => "json".to_string(),
            _ => "other".to_string(),
        }
    }
    
    /// Определяет тип BSL объекта по заголовку
    fn detect_object_type(&self, title: &str) -> String {
        let title_lower = title.to_lowercase();
        
        if title_lower.contains("объект") || title_lower.contains("object") {
            "object".to_string()
        } else if title_lower.contains("коллекция") || title_lower.contains("collection") {
            "collection".to_string()
        } else if title_lower.contains("менеджер") || title_lower.contains("manager") {
            "manager".to_string()
        } else if title_lower.contains("форма") || title_lower.contains("form") {
            "form".to_string()
        } else if title_lower.contains("отчет") || title_lower.contains("report") {
            "report".to_string()
        } else if title_lower.contains("обработка") || title_lower.contains("dataprocessor") {
            "dataprocessor".to_string()
        } else if title.contains("(") && title.contains(")") {
            "method".to_string()
        } else {
            "unknown".to_string()
        }
    }
    
    /// Получает путь к архиву
    pub fn archive_path(&self) -> &Path {
        &self.archive_path
    }
    
    /// Очищает кэш содержимого файлов
    pub fn clear_cache(&mut self) {
        self.content_cache.clear();
    }
    
    /// Получает размер кэша
    pub fn cache_size(&self) -> usize {
        self.content_cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use tempfile::TempDir; // Unused import
    // use std::fs; // Unused import
    
    #[test]
    fn test_hbk_parser_creation() {
        let parser = HbkArchiveParser::new("test.hbk");
        assert_eq!(parser.archive_path(), Path::new("test.hbk"));
        assert_eq!(parser.cache_size(), 0);
    }
    
    #[test]
    fn test_detect_file_type() {
        let parser = HbkArchiveParser::new("test.hbk");
        
        assert_eq!(parser.detect_file_type("test.html"), "html");
        assert_eq!(parser.detect_file_type("style.css"), "css");
        assert_eq!(parser.detect_file_type("script.js"), "javascript");
        assert_eq!(parser.detect_file_type("image.png"), "image");
        assert_eq!(parser.detect_file_type("unknown"), "other");
    }
    
    #[test]
    fn test_detect_object_type() {
        let parser = HbkArchiveParser::new("test.hbk");
        
        assert_eq!(parser.detect_object_type("Объект СправочникОбъект"), "object");
        assert_eq!(parser.detect_object_type("Коллекция значений"), "collection");
        assert_eq!(parser.detect_object_type("Сообщить()"), "method");
        assert_eq!(parser.detect_object_type("Неизвестный тип"), "unknown");
    }
    
    #[test]
    fn test_html_content_creation() {
        let content = HtmlContent {
            title: Some("Test Title".to_string()),
            syntax: vec!["Синтаксис: Тест()".to_string()],
            fields: vec![],
            description: Some("Test description".to_string()),
            examples: vec![],
            object_type: Some("method".to_string()),
            availability: None,
            version: None,
        };
        
        assert_eq!(content.title, Some("Test Title".to_string()));
        assert_eq!(content.syntax.len(), 1);
        assert_eq!(content.object_type, Some("method".to_string()));
    }
}