/*!
# Chunked Documentation Loader

Модуль для загрузки документации из разбитых JSON файлов.
Поддерживает эффективную загрузку и поиск в больших объемах данных.
*/

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};

use super::chunked_writer::{SyntaxItem, ItemMetadata};

/// Загрузчик chunked документации
pub struct ChunkedDocsLoader {
    docs_dir: PathBuf,
    index_cache: Option<MainIndexCache>,
}

/// Кэш главного индекса
#[derive(Debug, Clone)]
struct MainIndexCache {
    total_items: usize,
    categories: HashMap<String, CategoryInfo>,
    item_index: HashMap<String, ItemLocation>,
}

/// Информация о категории
#[derive(Debug, Clone, Deserialize)]
struct CategoryInfo {
    items_count: usize,
    chunks_count: usize,
    files: Vec<String>,
}

/// Местоположение элемента
#[derive(Debug, Clone)]
struct ItemLocation {
    category: String,
    file: String,
    title: String,
    object_name: String,
}

/// Индекс главного файла
#[derive(Debug, Deserialize)]
struct MainIndex {
    total_items: usize,
    categories: HashMap<String, CategoryInfo>,
    #[serde(default)]
    item_index: HashMap<String, ItemIndexEntry>,
}

/// Запись в индексе элементов
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ItemIndexEntry {
    pub category: String,
    pub file: String,
    pub title: String,
    pub object_name: String,
}

impl ChunkedDocsLoader {
    /// Создает новый загрузчик
    pub fn new<P: AsRef<Path>>(docs_dir: P) -> Self {
        Self {
            docs_dir: docs_dir.as_ref().to_path_buf(),
            index_cache: None,
        }
    }
    
    /// Загружает индекс и создает кэш
    pub fn load_index(&mut self) -> Result<()> {
        let index_path = self.docs_dir.join("main_index.json");
        let content = fs::read_to_string(&index_path)
            .context("Failed to read main_index.json")?;
        
        let main_index: MainIndex = serde_json::from_str(&content)
            .context("Failed to parse main_index.json")?;
        
        // Если item_index пустой, строим его сами
        let item_index = if main_index.item_index.is_empty() {
            self.build_item_index(&main_index.categories)?
        } else {
            main_index.item_index.into_iter()
                .map(|(id, entry)| (id, ItemLocation {
                    category: entry.category,
                    file: entry.file,
                    title: entry.title,
                    object_name: entry.object_name,
                }))
                .collect()
        };
        
        self.index_cache = Some(MainIndexCache {
            total_items: main_index.total_items,
            categories: main_index.categories,
            item_index,
        });
        
        Ok(())
    }
    
    /// Строит индекс элементов, сканируя файлы
    fn build_item_index(&self, categories: &HashMap<String, CategoryInfo>) -> Result<HashMap<String, ItemLocation>> {
        let mut item_index = HashMap::new();
        
        for (category, info) in categories {
            for file_name in &info.files {
                let file_path = self.docs_dir.join(category).join(file_name);
                if let Ok(content) = fs::read_to_string(&file_path) {
                    if let Ok(chunk_data) = serde_json::from_str::<ChunkFile>(&content) {
                        for item in chunk_data.items {
                            // Извлекаем имя объекта из title
                            let object_name = extract_object_name(&item.title);
                            
                            item_index.insert(item.id.clone(), ItemLocation {
                                category: category.clone(),
                                file: file_name.clone(),
                                title: item.title.clone(),
                                object_name,
                            });
                        }
                    }
                }
            }
        }
        
        Ok(item_index)
    }
    
    /// Получает элемент по ID
    pub fn get_item(&self, item_id: &str) -> Result<Option<SyntaxItem>> {
        let cache = self.index_cache.as_ref()
            .context("Index not loaded")?;
        
        if let Some(location) = cache.item_index.get(item_id) {
            let file_path = self.docs_dir
                .join(&location.category)
                .join(&location.file);
            
            let content = fs::read_to_string(&file_path)?;
            let chunk_data: ChunkFile = serde_json::from_str(&content)?;
            
            Ok(chunk_data.items.into_iter()
                .find(|item| item.id == item_id))
        } else {
            Ok(None)
        }
    }
    
    /// Ищет элементы по имени объекта
    pub fn find_by_object(&self, object_name: &str) -> Result<Vec<String>> {
        let cache = self.index_cache.as_ref()
            .context("Index not loaded")?;
        
        let matches: Vec<String> = cache.item_index
            .iter()
            .filter(|(_, location)| location.object_name.contains(object_name))
            .map(|(id, _)| id.clone())
            .collect();
        
        Ok(matches)
    }
    
    /// Получает все элементы категории
    pub fn get_category_items(&self, category: &str) -> Result<Vec<SyntaxItem>> {
        let cache = self.index_cache.as_ref()
            .context("Index not loaded")?;
        
        let mut items = Vec::new();
        
        if let Some(cat_info) = cache.categories.get(category) {
            for file_name in &cat_info.files {
                let file_path = self.docs_dir.join(category).join(file_name);
                let content = fs::read_to_string(&file_path)?;
                let chunk_data: ChunkFile = serde_json::from_str(&content)?;
                items.extend(chunk_data.items);
            }
        }
        
        Ok(items)
    }
    
    /// Получает статистику
    pub fn get_statistics(&self) -> Result<DocumentationStats> {
        let cache = self.index_cache.as_ref()
            .context("Index not loaded")?;
        
        let mut stats = DocumentationStats {
            total_items: cache.total_items,
            categories: HashMap::new(),
        };
        
        for (category, info) in &cache.categories {
            stats.categories.insert(category.clone(), CategoryStats {
                items_count: info.items_count,
                files_count: info.files.len(),
            });
        }
        
        Ok(stats)
    }
}

/// Статистика документации
#[derive(Debug, Clone)]
pub struct DocumentationStats {
    pub total_items: usize,
    pub categories: HashMap<String, CategoryStats>,
}

/// Статистика категории
#[derive(Debug, Clone)]
pub struct CategoryStats {
    pub items_count: usize,
    pub files_count: usize,
}

/// Структура chunk файла
#[derive(Debug, Deserialize)]
struct ChunkFile {
    items: Vec<SyntaxItem>,
}

/// Извлекает имя объекта из заголовка
fn extract_object_name(title: &str) -> String {
    // Примеры:
    // "ДинамическийСписок.АвтоЗаполнениеДоступныхПолей (DynamicList.AutoFillAvailableFields)"
    // "СоединитьСтроки (StrConcat)"
    
    if let Some(dot_pos) = title.find('.') {
        // Это метод или свойство объекта
        title[..dot_pos].to_string()
    } else if let Some(paren_pos) = title.find(" (") {
        // Это функция
        title[..paren_pos].to_string()
    } else {
        // Это объект или что-то другое
        title.to_string()
    }
}

/// Генератор улучшенного индекса с объектным индексом
pub fn generate_enhanced_index<P: AsRef<Path>>(docs_dir: P) -> Result<()> {
    let docs_path = docs_dir.as_ref();
    let main_index_path = docs_path.join("main_index.json");
    
    // Читаем текущий индекс
    let content = fs::read_to_string(&main_index_path)?;
    let mut index_data: serde_json::Value = serde_json::from_str(&content)?;
    
    // Строим item_index
    let mut item_index = HashMap::new();
    
    if let Some(categories) = index_data["categories"].as_object() {
        for (category, cat_info) in categories {
            if let Some(files) = cat_info["files"].as_array() {
                for file_value in files {
                    if let Some(file_name) = file_value.as_str() {
                        let file_path = docs_path.join(category).join(file_name);
                        
                        if let Ok(chunk_content) = fs::read_to_string(&file_path) {
                            if let Ok(chunk_data) = serde_json::from_str::<ChunkFile>(&chunk_content) {
                                for item in chunk_data.items {
                                    let object_name = extract_object_name(&item.title);
                                    
                                    item_index.insert(item.id.clone(), ItemIndexEntry {
                                        category: category.clone(),
                                        file: file_name.to_string(),
                                        title: item.title.clone(),
                                        object_name,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Добавляем item_index в JSON
    index_data["item_index"] = serde_json::to_value(item_index)?;
    
    // Записываем обновленный индекс
    let pretty_json = serde_json::to_string_pretty(&index_data)?;
    fs::write(main_index_path, pretty_json)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_object_name() {
        assert_eq!(
            extract_object_name("ДинамическийСписок.АвтоЗаполнениеДоступныхПолей (DynamicList.AutoFillAvailableFields)"),
            "ДинамическийСписок"
        );
        
        assert_eq!(
            extract_object_name("СоединитьСтроки (StrConcat)"),
            "СоединитьСтроки"
        );
        
        assert_eq!(
            extract_object_name("Массив"),
            "Массив"
        );
    }
}