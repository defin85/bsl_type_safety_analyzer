/*!
# Chunked Writer for BSL Syntax Documentation

Модуль для записи больших объемов данных синтаксиса BSL в разбитые файлы.
Обеспечивает генерацию структуры файлов, аналогичной эталонной docs_search.

## Основные возможности
- Разбиение данных на файлы с ограничением по размеру и количеству элементов
- Генерация индексных файлов для каждой категории
- Создание главного индекса с полной статистикой
- Поддержка категорий: objects, methods, functions, properties, operators
*/

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use serde_json;
use anyhow::Result;
use chrono::Utc;

/// Настройки для разбиения файлов
#[derive(Debug, Clone)]
pub struct ChunkSettings {
    /// Максимальный размер файла в килобайтах
    pub max_file_size_kb: usize,
    /// Максимальное количество элементов в файле
    pub max_items_per_file: usize,
    /// Режим экспорта
    pub mode: String,
}

impl Default for ChunkSettings {
    fn default() -> Self {
        Self {
            max_file_size_kb: 50,
            max_items_per_file: 50,
            mode: "max_split".to_string(),
        }
    }
}

/// Элемент синтаксиса для записи
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxItem {
    pub id: String,
    pub title: String,
    pub category: String,
    pub content: String,
    pub metadata: ItemMetadata,
}

/// Метаданные элемента
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemMetadata {
    pub filename: String,
    pub syntax: String,
    pub syntax_variants: Vec<String>,
    pub parameters: Vec<String>,
    pub parameters_by_variant: HashMap<String, Vec<String>>,
    pub return_value: String,
    pub example: String,
    pub links: Vec<LinkInfo>,
    pub collection_elements: HashMap<String, String>,
    pub methods: Vec<String>,
    pub availability: Vec<String>,
    pub version: String,
}

/// Информация о ссылке
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkInfo {
    pub text: String,
    pub href: String,
}

/// Запись в индексе элементов
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemIndexEntry {
    pub category: String,
    pub file: String,
    pub title: String,
    pub object_name: String,
}

/// Chunk файла с элементами
#[derive(Debug, Serialize, Deserialize)]
struct ChunkFile {
    items: Vec<SyntaxItem>,
    metadata: ChunkMetadata,
}

/// Метаданные chunk'а
#[derive(Debug, Serialize, Deserialize)]
struct ChunkMetadata {
    category: String,
    chunk: usize,
    total_chunks: usize,
    items_count: usize,
    created_at: String,
}

/// Информация о категории в главном индексе
#[derive(Debug, Serialize, Deserialize)]
struct CategoryInfo {
    items_count: usize,
    chunks_count: usize,
    files: Vec<String>,
}

/// Главный индекс
#[derive(Debug, Serialize, Deserialize)]
struct MainIndex {
    total_items: usize,
    categories: HashMap<String, CategoryInfo>,
    created_at: String,
    mode: String,
    settings: ChunkSettings,
}

/// Индекс категории
#[derive(Debug, Serialize, Deserialize)]
struct CategoryIndex {
    category: String,
    total_items: usize,
    chunks: Vec<ChunkInfo>,
    created_at: String,
}

/// Информация о chunk'е в индексе
#[derive(Debug, Serialize, Deserialize)]
struct ChunkInfo {
    chunk_number: usize,
    filename: String,
    items_count: usize,
    first_item_id: String,
    last_item_id: String,
    size_kb: f64,
}

/// Writer для разбитых файлов синтаксиса
pub struct ChunkedSyntaxWriter {
    output_dir: PathBuf,
    settings: ChunkSettings,
    category_counters: HashMap<String, usize>,
    category_items: HashMap<String, Vec<SyntaxItem>>,
}

impl ChunkedSyntaxWriter {
    /// Создает новый writer
    pub fn new<P: AsRef<Path>>(output_dir: P, settings: ChunkSettings) -> Self {
        Self {
            output_dir: output_dir.as_ref().to_path_buf(),
            settings,
            category_counters: HashMap::new(),
            category_items: HashMap::new(),
        }
    }
    
    /// Добавляет элемент синтаксиса
    pub fn add_item(&mut self, item: SyntaxItem) {
        let category = item.category.clone();
        
        // Добавляем в буфер категории
        self.category_items
            .entry(category.clone())
            .or_insert_with(Vec::new)
            .push(item);
            
        // Увеличиваем счетчик
        *self.category_counters.entry(category).or_insert(0) += 1;
    }
    
    /// Записывает все накопленные данные
    pub fn write_all(&mut self) -> Result<()> {
        tracing::info!("Writing chunked syntax data to: {}", self.output_dir.display());
        
        // Создаем директории
        self.create_directories()?;
        
        // Пишем файлы по категориям
        let mut main_categories = HashMap::new();
        
        for (category, items) in &self.category_items {
            let category_info = self.write_category(category, items)?;
            main_categories.insert(category.clone(), category_info);
        }
        
        // Пишем главный индекс
        self.write_main_index(main_categories)?;
        
        Ok(())
    }
    
    /// Создает структуру директорий
    fn create_directories(&self) -> Result<()> {
        fs::create_dir_all(&self.output_dir)?;
        
        for category in ["objects", "methods", "functions", "properties", "operators"] {
            fs::create_dir_all(self.output_dir.join(category))?;
        }
        
        Ok(())
    }
    
    /// Записывает все элементы категории
    fn write_category(&self, category: &str, items: &[SyntaxItem]) -> Result<CategoryInfo> {
        let category_dir = self.output_dir.join(category);
        let mut chunks = Vec::new();
        let mut files = Vec::new();
        
        // Разбиваем на chunk'и
        let chunked_items = self.chunk_items(items);
        let total_chunks = chunked_items.len();
        
        for (chunk_idx, chunk_items) in chunked_items.into_iter().enumerate() {
            let chunk_num = chunk_idx + 1;
            let filename = format!("{}_{:03}.json", category, chunk_num);
            let filepath = category_dir.join(&filename);
            
            // Создаем chunk файл
            let chunk_file = ChunkFile {
                items: chunk_items.clone(),
                metadata: ChunkMetadata {
                    category: category.to_string(),
                    chunk: chunk_num,
                    total_chunks,
                    items_count: chunk_items.len(),
                    created_at: Utc::now().to_rfc3339(),
                },
            };
            
            // Записываем файл
            let json = serde_json::to_string_pretty(&chunk_file)?;
            fs::write(&filepath, json)?;
            
            // Добавляем информацию о chunk'е
            let size_kb = fs::metadata(&filepath)?.len() as f64 / 1024.0;
            chunks.push(ChunkInfo {
                chunk_number: chunk_num,
                filename: filename.clone(),
                items_count: chunk_items.len(),
                first_item_id: chunk_items.first().map(|i| i.id.clone()).unwrap_or_default(),
                last_item_id: chunk_items.last().map(|i| i.id.clone()).unwrap_or_default(),
                size_kb,
            });
            
            files.push(filename);
        }
        
        // Записываем индекс категории
        let category_index = CategoryIndex {
            category: category.to_string(),
            total_items: items.len(),
            chunks,
            created_at: Utc::now().to_rfc3339(),
        };
        
        let index_path = self.output_dir.join(category).join(format!("{}_index.json", category));
        let json = serde_json::to_string_pretty(&category_index)?;
        fs::write(index_path, json)?;
        
        Ok(CategoryInfo {
            items_count: items.len(),
            chunks_count: files.len(),
            files,
        })
    }
    
    /// Разбивает элементы на chunk'и
    fn chunk_items(&self, items: &[SyntaxItem]) -> Vec<Vec<SyntaxItem>> {
        let mut chunks = Vec::new();
        let mut current_chunk = Vec::new();
        let mut current_size = 0;
        
        for item in items {
            // Оцениваем размер элемента
            let item_size = serde_json::to_string(item).unwrap_or_default().len();
            
            // Проверяем условия для нового chunk'а
            if !current_chunk.is_empty() && 
               (current_chunk.len() >= self.settings.max_items_per_file ||
                current_size + item_size > self.settings.max_file_size_kb * 1024) {
                chunks.push(current_chunk);
                current_chunk = Vec::new();
                current_size = 0;
            }
            
            current_chunk.push(item.clone());
            current_size += item_size;
        }
        
        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }
        
        chunks
    }
    
    /// Записывает главный индекс
    fn write_main_index(&self, categories: HashMap<String, CategoryInfo>) -> Result<()> {
        let total_items: usize = categories.values().map(|c| c.items_count).sum();
        
        let main_index = MainIndex {
            total_items,
            categories,
            created_at: Utc::now().to_rfc3339(),
            mode: self.settings.mode.clone(),
            settings: ChunkSettings {
                max_file_size_kb: self.settings.max_file_size_kb,
                max_items_per_file: self.settings.max_items_per_file,
                mode: self.settings.mode.clone(),
            },
        };
        
        let index_path = self.output_dir.join("main_index.json");
        let json = serde_json::to_string_pretty(&main_index)?;
        fs::write(index_path, json)?;
        
        tracing::info!("Main index written with {} total items", total_items);
        
        Ok(())
    }
    
    /// Улучшенный main_index.json с дополнительной статистикой
    pub fn write_enhanced_main_index(&self, categories: HashMap<String, CategoryInfo>) -> Result<()> {
        #[derive(Debug, Serialize)]
        struct EnhancedMainIndex {
            total_items: usize,
            categories: HashMap<String, CategoryInfo>,
            created_at: String,
            mode: String,
            settings: ChunkSettings,
            statistics: Statistics,
            version_info: VersionInfo,
            item_index: HashMap<String, ItemIndexEntry>,
            objects_summary: ObjectsSummary,
        }
        
        #[derive(Debug, Serialize)]
        struct ObjectsSummary {
            total_unique_objects: usize,
            objects_by_category: HashMap<String, Vec<String>>,
            top_objects_by_methods: Vec<ObjectStat>,
        }
        
        #[derive(Debug, Serialize)]
        struct ObjectStat {
            name: String,
            methods_count: usize,
            properties_count: usize,
            english_name: Option<String>,
        }
        
        #[derive(Debug, Serialize)]
        struct Statistics {
            total_files: usize,
            total_size_mb: f64,
            average_items_per_file: f64,
            coverage: Coverage,
            processing_info: ProcessingInfo,
        }
        
        #[derive(Debug, Serialize)]
        struct Coverage {
            html_files_processed: usize,
            html_files_total: usize,
            coverage_percent: f64,
        }
        
        #[derive(Debug, Serialize)]
        struct ProcessingInfo {
            extraction_time_seconds: f64,
            errors_count: usize,
            warnings_count: usize,
        }
        
        #[derive(Debug, Serialize)]
        struct VersionInfo {
            generator_version: String,
            bsl_version: String,
            platform_version: String,
        }
        
        // Подсчитываем статистику
        let total_items: usize = categories.values().map(|c| c.items_count).sum();
        let total_files: usize = categories.values().map(|c| c.files.len()).sum();
        let average_items = if total_files > 0 {
            total_items as f64 / total_files as f64
        } else {
            0.0
        };
        
        // Считаем общий размер
        let mut total_size = 0u64;
        for (category, info) in &categories {
            for file in &info.files {
                let path = self.output_dir.join(category).join(file);
                if let Ok(metadata) = fs::metadata(path) {
                    total_size += metadata.len();
                }
            }
        }
        
        // Строим item_index
        let mut item_index = HashMap::new();
        for (category, info) in &categories {
            for file_name in &info.files {
                let file_path = self.output_dir.join(category).join(file_name);
                if let Ok(content) = fs::read_to_string(&file_path) {
                    if let Ok(chunk_data) = serde_json::from_str::<ChunkFile>(&content) {
                        for item in chunk_data.items {
                            let object_name = extract_object_name(&item.title);
                            item_index.insert(item.id.clone(), ItemIndexEntry {
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
        
        // Собираем статистику по объектам
        let mut object_stats: HashMap<String, ObjectStats> = HashMap::new();
        
        for entry in item_index.values() {
            let stats = object_stats.entry(entry.object_name.clone()).or_insert(ObjectStats::default());
            
            match entry.category.as_str() {
                "methods" => stats.methods_count += 1,
                "properties" => stats.properties_count += 1,
                "functions" => stats.functions_count += 1,
                "operators" => stats.operators_count += 1,
                _ => {}
            }
            
            // Извлекаем английское название
            if let Some(eng_name) = extract_english_name(&entry.title) {
                stats.english_name = Some(eng_name);
            }
        }
        
        // Группируем объекты по категориям
        let objects_by_category = categorize_objects(&object_stats);
        
        // Топ-10 объектов по методам
        let mut top_objects: Vec<ObjectStat> = object_stats.iter()
            .map(|(name, stats)| ObjectStat {
                name: name.clone(),
                methods_count: stats.methods_count,
                properties_count: stats.properties_count,
                english_name: stats.english_name.clone(),
            })
            .collect();
        top_objects.sort_by(|a, b| b.methods_count.cmp(&a.methods_count));
        top_objects.truncate(10);
        
        let enhanced_index = EnhancedMainIndex {
            total_items,
            categories,
            created_at: Utc::now().to_rfc3339(),
            mode: self.settings.mode.clone(),
            settings: ChunkSettings {
                max_file_size_kb: self.settings.max_file_size_kb,
                max_items_per_file: self.settings.max_items_per_file,
                mode: self.settings.mode.clone(),
            },
            statistics: Statistics {
                total_files,
                total_size_mb: total_size as f64 / (1024.0 * 1024.0),
                average_items_per_file: average_items,
                coverage: Coverage {
                    html_files_processed: 24979, // TODO: получать из контекста
                    html_files_total: 24979,
                    coverage_percent: 100.0,
                },
                processing_info: ProcessingInfo {
                    extraction_time_seconds: 0.0, // TODO: замерять время
                    errors_count: 0,
                    warnings_count: 0,
                },
            },
            version_info: VersionInfo {
                generator_version: env!("CARGO_PKG_VERSION").to_string(),
                bsl_version: "8.3.22".to_string(),
                platform_version: "8.3.22.1923".to_string(),
            },
            item_index,
            objects_summary: ObjectsSummary {
                total_unique_objects: object_stats.len(),
                objects_by_category,
                top_objects_by_methods: top_objects,
            },
        };
        
        let index_path = self.output_dir.join("main_index.json");
        let json = serde_json::to_string_pretty(&enhanced_index)?;
        fs::write(index_path, json)?;
        
        tracing::info!("Enhanced main index written with {} total items in {} files", total_items, total_files);
        
        Ok(())
    }
}

/// Streaming writer для больших объемов данных
pub struct StreamingChunkedWriter {
    output_dir: PathBuf,
    settings: ChunkSettings,
    current_chunks: HashMap<String, Vec<SyntaxItem>>,
    category_stats: HashMap<String, CategoryStats>,
    start_time: std::time::Instant,
}

struct CategoryStats {
    items_count: usize,
    chunks_count: usize,
    current_chunk_size: usize,
}

impl StreamingChunkedWriter {
    /// Создает новый streaming writer
    pub fn new<P: AsRef<Path>>(output_dir: P, settings: ChunkSettings) -> Self {
        Self {
            output_dir: output_dir.as_ref().to_path_buf(),
            settings,
            current_chunks: HashMap::new(),
            category_stats: HashMap::new(),
            start_time: std::time::Instant::now(),
        }
    }
    
    /// Добавляет элемент и сразу записывает при необходимости
    pub fn add_item(&mut self, item: SyntaxItem) {
        let category = item.category.clone();
        
        // Инициализируем статистику категории если нужно
        self.category_stats.entry(category.clone()).or_insert(CategoryStats {
            items_count: 0,
            chunks_count: 0,
            current_chunk_size: 0,
        });
        
        // Добавляем в текущий чанк
        let current_items = self.current_chunks.entry(category.clone()).or_insert_with(Vec::new);
        
        // Оцениваем размер элемента
        let item_size = serde_json::to_string(&item).unwrap_or_default().len();
        
        // Проверяем нужно ли записать чанк
        let stats = self.category_stats.get(&category).unwrap();
        let needs_new_chunk = !current_items.is_empty() && 
           (current_items.len() >= self.settings.max_items_per_file ||
            stats.current_chunk_size + item_size > self.settings.max_file_size_kb * 1024);
        
        if needs_new_chunk {
            // Записываем текущий чанк
            let items_to_write = current_items.clone();
            if let Err(e) = self.write_chunk(&category, &items_to_write) {
                tracing::error!("Failed to write chunk: {}", e);
            }
            
            // Обновляем статистику
            if let Some(stats) = self.category_stats.get_mut(&category) {
                stats.chunks_count += 1;
                stats.current_chunk_size = 0;
            }
            
            // Очищаем текущий чанк
            self.current_chunks.get_mut(&category).unwrap().clear();
        }
        
        // Добавляем элемент
        self.current_chunks.get_mut(&category).unwrap().push(item);
        if let Some(stats) = self.category_stats.get_mut(&category) {
            stats.items_count += 1;
            stats.current_chunk_size += item_size;
        }
    }
    
    /// Записывает чанк на диск
    fn write_chunk(&self, category: &str, items: &[SyntaxItem]) -> Result<()> {
        let category_dir = self.output_dir.join(category);
        fs::create_dir_all(&category_dir)?;
        
        let stats = self.category_stats.get(category).unwrap();
        let chunk_num = stats.chunks_count + 1;
        let filename = format!("{}_{:03}.json", category, chunk_num);
        let filepath = category_dir.join(&filename);
        
        let chunk_file = ChunkFile {
            items: items.to_vec(),
            metadata: ChunkMetadata {
                category: category.to_string(),
                chunk: chunk_num,
                total_chunks: 0, // Будет обновлено в finalize
                items_count: items.len(),
                created_at: Utc::now().to_rfc3339(),
            },
        };
        
        let json = serde_json::to_string_pretty(&chunk_file)?;
        fs::write(filepath, json)?;
        
        Ok(())
    }
    
    /// Завершает запись и создает индексные файлы
    pub fn finalize(&mut self) -> Result<()> {
        tracing::info!("Finalizing chunked export");
        
        // Создаем базовую структуру директорий
        self.create_directories()?;
        
        // Записываем последние чанки
        let categories: Vec<String> = self.current_chunks.keys().cloned().collect();
        for category in categories {
            if let Some(items) = self.current_chunks.get(&category) {
                if !items.is_empty() {
                    self.write_chunk(&category, items)?;
                    if let Some(stats) = self.category_stats.get_mut(&category) {
                        stats.chunks_count += 1;
                    }
                }
            }
        }
        self.current_chunks.clear();
        
        // Создаем индексы для категорий
        let mut categories = HashMap::new();
        for (category, stats) in &self.category_stats {
            let category_dir = self.output_dir.join(category);
            
            // Собираем информацию о файлах в категории
            let mut files = Vec::new();
            for i in 1..=stats.chunks_count {
                let filename = format!("{}_{:03}.json", category, i);
                if category_dir.join(&filename).exists() {
                    files.push(filename);
                }
            }
            
            categories.insert(category.clone(), CategoryInfo {
                items_count: stats.items_count,
                chunks_count: stats.chunks_count,
                files,
            });
            
            // Создаем индекс категории
            self.write_category_index(category, stats)?;
        }
        
        // Создаем улучшенный главный индекс
        self.write_enhanced_main_index(categories)?;
        
        Ok(())
    }
    
    fn create_directories(&self) -> Result<()> {
        fs::create_dir_all(&self.output_dir)?;
        
        for category in ["objects", "methods", "functions", "properties", "operators"] {
            fs::create_dir_all(self.output_dir.join(category))?;
        }
        
        Ok(())
    }
    
    fn write_category_index(&self, category: &str, stats: &CategoryStats) -> Result<()> {
        let category_dir = self.output_dir.join(category);
        let mut chunks = Vec::new();
        
        for i in 1..=stats.chunks_count {
            let filename = format!("{}_{:03}.json", category, i);
            let filepath = category_dir.join(&filename);
            
            if filepath.exists() {
                let size_kb = fs::metadata(&filepath)?.len() as f64 / 1024.0;
                
                // Читаем файл чтобы получить информацию о первом и последнем элементе
                let content = fs::read_to_string(&filepath)?;
                if let Ok(chunk_file) = serde_json::from_str::<ChunkFile>(&content) {
                    chunks.push(ChunkInfo {
                        chunk_number: i,
                        filename: filename.clone(),
                        items_count: chunk_file.items.len(),
                        first_item_id: chunk_file.items.first().map(|i| i.id.clone()).unwrap_or_default(),
                        last_item_id: chunk_file.items.last().map(|i| i.id.clone()).unwrap_or_default(),
                        size_kb,
                    });
                }
            }
        }
        
        let category_index = CategoryIndex {
            category: category.to_string(),
            total_items: stats.items_count,
            chunks,
            created_at: Utc::now().to_rfc3339(),
        };
        
        let index_path = category_dir.join(format!("{}_index.json", category));
        let json = serde_json::to_string_pretty(&category_index)?;
        fs::write(index_path, json)?;
        
        Ok(())
    }
    
    /// Улучшенный main_index.json с дополнительной статистикой
    pub fn write_enhanced_main_index(&self, categories: HashMap<String, CategoryInfo>) -> Result<()> {
        #[derive(Debug, Serialize)]
        struct EnhancedMainIndex {
            total_items: usize,
            categories: HashMap<String, CategoryInfo>,
            created_at: String,
            mode: String,
            settings: ChunkSettings,
            statistics: Statistics,
            version_info: VersionInfo,
            item_index: HashMap<String, ItemIndexEntry>,
            objects_summary: ObjectsSummary,
        }
        
        #[derive(Debug, Serialize)]
        struct ObjectsSummary {
            total_unique_objects: usize,
            objects_by_category: HashMap<String, Vec<String>>,
            top_objects_by_methods: Vec<ObjectStat>,
        }
        
        #[derive(Debug, Serialize)]
        struct ObjectStat {
            name: String,
            methods_count: usize,
            properties_count: usize,
            english_name: Option<String>,
        }
        
        #[derive(Debug, Serialize)]
        struct Statistics {
            total_files: usize,
            total_size_mb: f64,
            average_items_per_file: f64,
            coverage: Coverage,
            processing_info: ProcessingInfo,
        }
        
        #[derive(Debug, Serialize)]
        struct Coverage {
            html_files_processed: usize,
            html_files_total: usize,
            coverage_percent: f64,
        }
        
        #[derive(Debug, Serialize)]
        struct ProcessingInfo {
            extraction_time_seconds: f64,
            errors_count: usize,
            warnings_count: usize,
        }
        
        #[derive(Debug, Serialize)]
        struct VersionInfo {
            generator_version: String,
            bsl_version: String,
            platform_version: String,
        }
        
        // Подсчитываем статистику
        let total_items: usize = categories.values().map(|c| c.items_count).sum();
        let total_files: usize = categories.values().map(|c| c.files.len()).sum();
        let average_items = if total_files > 0 {
            total_items as f64 / total_files as f64
        } else {
            0.0
        };
        
        // Считаем общий размер
        let mut total_size = 0u64;
        for (category, info) in &categories {
            for file in &info.files {
                let path = self.output_dir.join(category).join(file);
                if let Ok(metadata) = fs::metadata(path) {
                    total_size += metadata.len();
                }
            }
        }
        
        let elapsed = self.start_time.elapsed();
        
        // Строим item_index
        let mut item_index = HashMap::new();
        for (category, info) in &categories {
            for file_name in &info.files {
                let file_path = self.output_dir.join(category).join(file_name);
                if let Ok(content) = fs::read_to_string(&file_path) {
                    if let Ok(chunk_data) = serde_json::from_str::<ChunkFile>(&content) {
                        for item in chunk_data.items {
                            let object_name = extract_object_name(&item.title);
                            item_index.insert(item.id.clone(), ItemIndexEntry {
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
        
        // Собираем статистику по объектам
        let mut object_stats: HashMap<String, ObjectStats> = HashMap::new();
        
        for entry in item_index.values() {
            let stats = object_stats.entry(entry.object_name.clone()).or_insert(ObjectStats::default());
            
            match entry.category.as_str() {
                "methods" => stats.methods_count += 1,
                "properties" => stats.properties_count += 1,
                "functions" => stats.functions_count += 1,
                "operators" => stats.operators_count += 1,
                _ => {}
            }
            
            // Извлекаем английское название
            if let Some(eng_name) = extract_english_name(&entry.title) {
                stats.english_name = Some(eng_name);
            }
        }
        
        // Группируем объекты по категориям
        let objects_by_category = categorize_objects(&object_stats);
        
        // Топ-10 объектов по методам
        let mut top_objects: Vec<ObjectStat> = object_stats.iter()
            .map(|(name, stats)| ObjectStat {
                name: name.clone(),
                methods_count: stats.methods_count,
                properties_count: stats.properties_count,
                english_name: stats.english_name.clone(),
            })
            .collect();
        top_objects.sort_by(|a, b| b.methods_count.cmp(&a.methods_count));
        top_objects.truncate(10);
        
        let enhanced_index = EnhancedMainIndex {
            total_items,
            categories,
            created_at: Utc::now().to_rfc3339(),
            mode: self.settings.mode.clone(),
            settings: ChunkSettings {
                max_file_size_kb: self.settings.max_file_size_kb,
                max_items_per_file: self.settings.max_items_per_file,
                mode: self.settings.mode.clone(),
            },
            statistics: Statistics {
                total_files,
                total_size_mb: total_size as f64 / (1024.0 * 1024.0),
                average_items_per_file: average_items,
                coverage: Coverage {
                    html_files_processed: total_items, // Using actual processed count
                    html_files_total: 24979,
                    coverage_percent: (total_items as f64 / 24979.0 * 100.0).min(100.0),
                },
                processing_info: ProcessingInfo {
                    extraction_time_seconds: elapsed.as_secs_f64(),
                    errors_count: 0,
                    warnings_count: 0,
                },
            },
            version_info: VersionInfo {
                generator_version: env!("CARGO_PKG_VERSION").to_string(),
                bsl_version: "8.3.22".to_string(),
                platform_version: "8.3.22.1923".to_string(),
            },
            item_index,
            objects_summary: ObjectsSummary {
                total_unique_objects: object_stats.len(),
                objects_by_category,
                top_objects_by_methods: top_objects,
            },
        };
        
        let index_path = self.output_dir.join("main_index.json");
        let json = serde_json::to_string_pretty(&enhanced_index)?;
        fs::write(index_path, json)?;
        
        tracing::info!("Enhanced main index written with {} total items in {} files", total_items, total_files);
        
        Ok(())
    }
}

// Для совместимости с serde
impl Serialize for ChunkSettings {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("ChunkSettings", 2)?;
        state.serialize_field("max_file_size_kb", &self.max_file_size_kb)?;
        state.serialize_field("max_items_per_file", &self.max_items_per_file)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for ChunkSettings {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(ChunkSettings::default())
    }
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

/// Извлекает английское название из заголовка
fn extract_english_name(title: &str) -> Option<String> {
    // Примеры:
    // "Массив.Добавить (Array.Add)"
    // "СоединитьСтроки (StrConcat)"
    
    if let Some(start) = title.find(" (") {
        if let Some(end) = title.find(")") {
            let eng_part = &title[start + 2..end];
            // Для методов берем только имя объекта
            if let Some(dot_pos) = eng_part.find('.') {
                return Some(eng_part[..dot_pos].to_string());
            } else {
                return Some(eng_part.to_string());
            }
        }
    }
    None
}

#[derive(Default)]
struct ObjectStats {
    methods_count: usize,
    properties_count: usize,
    functions_count: usize,
    operators_count: usize,
    english_name: Option<String>,
}

fn categorize_objects(objects: &HashMap<String, ObjectStats>) -> HashMap<String, Vec<String>> {
    let mut categories: HashMap<String, Vec<String>> = HashMap::new();
    
    for name in objects.keys() {
        let category = match name.as_str() {
            // Коллекции
            "Массив" | "Соответствие" | "СписокЗначений" | "ТаблицаЗначений" | 
            "ДеревоЗначений" | "ФиксированныйМассив" | "ФиксированноеСоответствие" => "collections",
            
            // Ввод/вывод
            "ЧтениеТекста" | "ЗаписьТекста" | "ЧтениеXML" | "ЗаписьXML" | 
            "ЧтениеJSON" | "ЗаписьJSON" | "ПотокВПамяти" | "ФайловыйПоток" => "io",
            
            // Системные
            "СистемнаяИнформация" | "ИнформацияОПользователе" | "НастройкиКлиента" |
            "РаботаСФайлами" | "СредстваКриптографии" => "system",
            
            // Даты и время
            "Дата" | "СтандартныйПериод" | "ОписаниеПериода" => "datetime",
            
            // Метаданные
            "Метаданные" | "ОбъектМетаданных" | "КонфигурацияМетаданных" => "metadata",
            
            // Запросы
            "Запрос" | "ПостроительЗапроса" | "РезультатЗапроса" | "ВыборкаИзРезультатаЗапроса" => "query",
            
            // Формы
            "УправляемаяФорма" | "ЭлементыФормы" | "ДанныеФормы" | "ПолеФормы" => "forms",
            
            // HTTP и веб
            "HTTPСоединение" | "HTTPЗапрос" | "HTTPОтвет" | "WSПрокси" | "WSСсылка" => "web",
            
            // COM и внешние компоненты
            "COMОбъект" | "ВнешниеКомпоненты" => "com",
            
            // Прочие
            _ => "other"
        };
        
        categories
            .entry(category.to_string())
            .or_insert_with(Vec::new)
            .push(name.clone());
    }
    
    // Сортируем объекты в каждой категории
    for objects in categories.values_mut() {
        objects.sort();
    }
    
    categories
}