/*!
# HBK Archive Parser (Full Port from Python)

Полный порт парсера архивов документации 1С (.hbk файлы) с Python на Rust.
Основан на src/parsers/hbk_parser.py из проекта 1c-help-parser.

Основные возможности:
- Парсинг .hbk архивов с HTML документацией
- Извлечение структурированного контента из HTML
- Анализ структуры архивов документации
- Кэширование содержимого файлов
- Полная совместимость с Python версией

## Использование

```rust,ignore
let mut parser = HbkArchiveParser::new("1C_Help.hbk");
parser.open_archive()?;
let analysis = parser.analyze_structure()?;
let samples = parser.extract_sample_files(5)?;
```
*/

use anyhow::{bail, Result};
use tracing::{error, warn};
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

/// Основной парсер HBK архивов (полный порт Python HBKParser)
pub struct HbkArchiveParser {
    hbk_file: PathBuf,
    zip_file: Option<ZipArchive<File>>,
    content_cache: HashMap<String, String>,
}

/// Структура для хранения распарсенного HTML контента
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HtmlContent {
    pub title: String,
    pub syntax: String,
    pub fields: Vec<FieldInfo>,
    pub description: String,
    pub example: String,
    pub links: Vec<LinkInfo>,
    pub filename: Option<String>,
}

/// Информация о поле объекта/метода
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldInfo {
    pub name: String,
    pub link: String,
}

/// Информация о ссылке
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkInfo {
    pub text: String,
    pub href: String,
}

/// Результат анализа архива
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveStructure {
    pub total_files: usize,
    pub html_files: usize,
    pub st_files: usize,
    pub categories: HashMap<String, usize>,
    pub file_types: HashMap<String, usize>,
    pub largest_files: Vec<FileInfo>,
}

/// Информация о файле
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub size: u64,
    pub compressed_size: u64,
}

impl HbkArchiveParser {
    /// Создает новый парсер для HBK файла (конструктор как в Python)
    pub fn new<P: AsRef<Path>>(hbk_file: P) -> Self {
        Self {
            hbk_file: hbk_file.as_ref().to_path_buf(),
            zip_file: None,
            content_cache: HashMap::new(),
        }
    }

    /// Открывает архив .hbk как ZIP (метод open_archive из Python)
    pub fn open_archive(&mut self) -> Result<bool> {
        match File::open(&self.hbk_file) {
            Ok(file) => match ZipArchive::new(file) {
                Ok(archive) => {
                    self.zip_file = Some(archive);
                    Ok(true)
                }
                Err(_) => {
                    error!(
                        "Ошибка: '{}' не является корректным ZIP-архивом",
                        self.hbk_file.display()
                    );
                    Ok(false)
                }
            },
            Err(e) => {
                error!("Ошибка при открытии архива: {}", e);
                Ok(false)
            }
        }
    }

    /// Возвращает список файлов в архиве (метод list_contents из Python)
    pub fn list_contents(&mut self) -> Vec<String> {
        match &mut self.zip_file {
            Some(archive) => {
                let mut contents = Vec::new();
                for i in 0..archive.len() {
                    if let Ok(file) = archive.by_index(i) {
                        contents.push(file.name().to_string());
                    }
                }
                contents
            }
            None => Vec::new(),
        }
    }

    /// Извлекает содержимое файла из архива (метод extract_file_content из Python)
    pub fn extract_file_content(&mut self, filename: &str) -> Option<String> {
        // Проверяем кэш
        if let Some(cached) = self.content_cache.get(filename) {
            return Some(cached.clone());
        }

        let content = match &mut self.zip_file {
            Some(archive) => {
                match archive.by_name(filename) {
                    Ok(mut file) => {
                        let mut content = Vec::new();
                        if file.read_to_end(&mut content).is_ok() {
                            // Пытаемся декодировать как UTF-8, если не получается - как cp1251
                            let decoded = if let Ok(utf8_str) = String::from_utf8(content.clone()) {
                                utf8_str
                            } else {
                                let (cow, _, _) = encoding_rs::WINDOWS_1251.decode(&content);
                                cow.into_owned()
                            };
                            Some(decoded)
                        } else {
                            None
                        }
                    }
                    Err(e) => {
                        error!("Ошибка при извлечении файла {}: {}", filename, e);
                        None
                    }
                }
            }
            None => None,
        };

        // Кэшируем результат
        if let Some(ref content_str) = content {
            self.content_cache
                .insert(filename.to_string(), content_str.clone());
        }

        content
    }

    /// Парсит HTML-контент и извлекает структурированную информацию (метод parse_html_content из Python)
    pub fn parse_html_content(&mut self, html_content: &str) -> Result<HtmlContent> {
        let document = Html::parse_document(html_content);

        let mut result = HtmlContent {
            title: String::new(),
            syntax: String::new(),
            fields: Vec::new(),
            description: String::new(),
            example: String::new(),
            links: Vec::new(),
            filename: None,
        };

        // Извлекаем заголовок
        let title_selector = Selector::parse("h1.V8SH_pagetitle").unwrap();
        if let Some(title_elem) = document.select(&title_selector).next() {
            result.title = title_elem.text().collect::<String>().trim().to_string();
        }

        // Извлекаем синтаксис
        let chapter_selector = Selector::parse("p.V8SH_chapter").unwrap();
        for chapter_elem in document.select(&chapter_selector) {
            let chapter_text = chapter_elem.text().collect::<String>();
            if chapter_text.contains("Синтаксис") {
                // Получаем следующий элемент после заголовка "Синтаксис"
                if let Some(next_sibling) = Self::get_next_sibling_element(&chapter_elem) {
                    result.syntax = next_sibling.text().collect::<String>().trim().to_string();
                }
            }

            // Извлекаем поля
            if chapter_text.contains("Поля") {
                // Собираем все ссылки после заголовка "Поля"
                let mut next = Self::get_next_sibling_element(&chapter_elem);
                while let Some(elem) = next {
                    if elem.value().name() == "a" {
                        if let Some(href) = elem.value().attr("href") {
                            let text = elem.text().collect::<String>().trim().to_string();
                            if !text.is_empty() && !href.is_empty() {
                                result.fields.push(FieldInfo {
                                    name: text,
                                    link: href.to_string(),
                                });
                            }
                        }
                    } else if elem.value().name() == "p"
                        && elem.value().attr("class") == Some("V8SH_chapter")
                    {
                        // Достигли следующей секции
                        break;
                    }
                    next = Self::get_next_sibling_element(&elem);
                }
            }

            // Извлекаем описание
            if chapter_text.contains("Описание") {
                if let Some(desc_elem) = Self::get_next_sibling_element(&chapter_elem) {
                    if desc_elem.value().name() == "p" {
                        result.description =
                            desc_elem.text().collect::<String>().trim().to_string();
                    }
                }
            }

            // Извлекаем пример
            if chapter_text.contains("Пример") {
                // Ищем таблицу после заголовка "Пример"
                let table_selector = Selector::parse("table").unwrap();
                if let Some(table) = document.select(&table_selector).next() {
                    result.example = table.text().collect::<String>().trim().to_string();
                }
            }
        }

        // Извлекаем ссылки
        let link_selector = Selector::parse("a").unwrap();
        for link in document.select(&link_selector) {
            if let Some(href) = link.value().attr("href") {
                if href.starts_with("v8help://") {
                    result.links.push(LinkInfo {
                        text: link.text().collect::<String>().trim().to_string(),
                        href: href.to_string(),
                    });
                }
            }
        }

        Ok(result)
    }

    /// Анализирует структуру архива (метод analyze_structure из Python)
    pub fn analyze_structure(&mut self) -> Result<ArchiveStructure> {
        if self.zip_file.is_none() {
            bail!("Архив не открыт");
        }

        let mut structure = ArchiveStructure {
            total_files: 0,
            html_files: 0,
            st_files: 0,
            categories: HashMap::new(),
            file_types: HashMap::new(),
            largest_files: Vec::new(),
        };

        if let Some(archive) = &mut self.zip_file {
            structure.total_files = archive.len();

            // Анализируем файлы
            for i in 0..archive.len() {
                if let Ok(file) = archive.by_index(i) {
                    let filename = file.name();
                    // Подсчитываем типы файлов
                    let ext = Path::new(filename)
                        .extension()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_lowercase();

                    *structure.file_types.entry(ext.clone()).or_insert(0) += 1;

                    if ext == "html" {
                        structure.html_files += 1;
                    } else if ext == "st" {
                        structure.st_files += 1;
                    }

                    // Анализируем категории
                    if let Some(category) = filename.split('/').next() {
                        *structure
                            .categories
                            .entry(category.to_string())
                            .or_insert(0) += 1;
                    }

                    // Сохраняем информацию о крупных файлах
                    if file.size() > 10000 {
                        // Больше 10KB
                        structure.largest_files.push(FileInfo {
                            name: filename.to_string(),
                            size: file.size(),
                            compressed_size: file.compressed_size(),
                        });
                    }
                }
            }

            // Сортируем крупные файлы по размеру
            structure.largest_files.sort_by(|a, b| b.size.cmp(&a.size));
            structure.largest_files.truncate(10); // Топ 10
        }

        Ok(structure)
    }

    /// Извлекает и анализирует несколько примеров файлов (метод extract_sample_files из Python)
    pub fn extract_sample_files(&mut self, count: usize) -> Result<Vec<HtmlContent>> {
        if self.zip_file.is_none() {
            bail!("Архив не открыт");
        }

        let mut samples = Vec::new();
        let html_files: Vec<String> = self
            .list_contents()
            .into_iter()
            .filter(|f| f.ends_with(".html"))
            .collect();

        for filename in html_files.iter().take(count) {
            match self.extract_file_content(filename) {
                Some(content) => match self.parse_html_content(&content) {
                    Ok(mut parsed) => {
                        parsed.filename = Some(filename.clone());
                        samples.push(parsed);
                    }
                    Err(e) => {
                        error!("Ошибка при обработке файла {}: {}", filename, e);
                    }
                },
                None => {
                    warn!("Не удалось извлечь содержимое файла: {}", filename);
                }
            }
        }

        Ok(samples)
    }

    /// Закрывает архив (метод close из Python)
    pub fn close(&mut self) {
        self.zip_file = None;
        self.content_cache.clear();
    }

    /// Вспомогательный метод для получения следующего элемента-соседа
    fn get_next_sibling_element<'a>(element: &ElementRef<'a>) -> Option<ElementRef<'a>> {
        let mut next_sibling = element.next_sibling();
        while let Some(node) = next_sibling {
            if let Some(elem) = ElementRef::wrap(node) {
                return Some(elem);
            }
            next_sibling = node.next_sibling();
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use tempfile::TempDir;

    #[test]
    fn test_parser_creation() {
        let parser = HbkArchiveParser::new("test.hbk");
        assert!(parser.zip_file.is_none());
    }

    #[test]
    fn test_list_contents_empty() {
        let mut parser = HbkArchiveParser::new("test.hbk");
        assert!(parser.list_contents().is_empty());
    }
}
