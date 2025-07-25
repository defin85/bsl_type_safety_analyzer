/*!
# File-based Cache Implementation

Файловый кэш для персистентного хранения данных анализатора BSL.
Поддерживает сжатие, инвалидацию и автоматическую очистку.
*/

use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};
use anyhow::{Context, Result};
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use crate::cache::{CacheKey, CacheValue};

/// Файловый кэш с поддержкой сжатия
pub struct FileCache {
    /// Директория для хранения кэша
    cache_dir: PathBuf,
    /// Включено ли сжатие
    compression_enabled: bool,
    /// Метаданные файлов кэша
    metadata: HashMap<String, FileCacheMetadata>,
    /// Максимальный возраст файла в кэше (в секундах)
    max_age_seconds: u64,
}

/// Запись файлового кэша
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCacheEntry {
    /// Ключ кэша
    pub key: CacheKey,
    /// Значение кэша
    pub value: CacheValue,
    /// Время создания записи
    pub created_at: SystemTime,
    /// Хэш содержимого для проверки целостности
    pub content_hash: String,
}

/// Метаданные файла кэша
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCacheMetadata {
    /// Имя файла
    pub filename: String,
    /// Размер файла в байтах
    pub file_size: u64,
    /// Время создания
    pub created_at: SystemTime,
    /// Время последнего доступа
    pub accessed_at: SystemTime,
    /// Сжат ли файл
    pub compressed: bool,
    /// Хэш содержимого
    pub content_hash: String,
}

impl FileCache {
    /// Создает новый файловый кэш
    pub fn new<P: AsRef<Path>>(cache_dir: P, compression_enabled: bool) -> Self {
        let cache_dir = cache_dir.as_ref().to_path_buf();
        
        // Создаем директорию кэша если её нет
        if !cache_dir.exists() {
            if let Err(e) = fs::create_dir_all(&cache_dir) {
                tracing::error!("Failed to create cache directory: {}", e);
            }
        }
        
        let mut file_cache = Self {
            cache_dir,
            compression_enabled,
            metadata: HashMap::new(),
            max_age_seconds: 24 * 3600, // 24 часа по умолчанию
        };
        
        // Загружаем существующие метаданные
        if let Err(e) = file_cache.load_metadata() {
            tracing::warn!("Failed to load cache metadata: {}", e);
        }
        
        file_cache
    }
    
    /// Получает значение из файлового кэша
    pub fn get(&mut self, key: &CacheKey) -> Result<Option<CacheValue>> {
        let filename = self.key_to_filename(key);
        let file_path = self.cache_dir.join(&filename);
        
        if !file_path.exists() {
            return Ok(None);
        }
        
        // Проверяем метаданные
        if let Some(metadata) = self.metadata.get(&filename) {
            // Проверяем возраст файла
            if let Ok(age) = SystemTime::now().duration_since(metadata.created_at) {
                if age.as_secs() > self.max_age_seconds {
                    // Файл устарел, удаляем его
                    self.remove_file(&filename)?;
                    return Ok(None);
                }
            }
            
            // Обновляем время доступа
            let mut updated_metadata = metadata.clone();
            updated_metadata.accessed_at = SystemTime::now();
            self.metadata.insert(filename.clone(), updated_metadata);
        }
        
        // Читаем и десериализуем файл
        let entry = self.read_cache_file(&file_path)?;
        
        tracing::debug!("File cache hit: {}", filename);
        Ok(Some(entry.value))
    }
    
    /// Сохраняет значение в файловый кэш
    pub fn set(&mut self, key: CacheKey, value: CacheValue) -> Result<()> {
        let filename = self.key_to_filename(&key);
        let file_path = self.cache_dir.join(&filename);
        
        let entry = FileCacheEntry {
            key: key.clone(),
            value: value.clone(),
            created_at: SystemTime::now(),
            content_hash: self.calculate_content_hash(&value)?,
        };
        
        // Записываем файл
        self.write_cache_file(&file_path, &entry)?;
        
        // Обновляем метаданные
        let file_size = file_path.metadata()?.len();
        let metadata = FileCacheMetadata {
            filename: filename.clone(),
            file_size,
            created_at: entry.created_at,
            accessed_at: entry.created_at,
            compressed: self.compression_enabled,
            content_hash: entry.content_hash,
        };
        
        self.metadata.insert(filename, metadata);
        
        // Сохраняем метаданные
        self.save_metadata()?;
        
        tracing::debug!("File cache set: {}", file_path.display());
        Ok(())
    }
    
    /// Инвалидирует записи для файла
    pub fn invalidate_file(&mut self, file_path: &Path) -> Result<()> {
        let file_str = file_path.to_string_lossy();
        let keys_to_remove: Vec<String> = self.metadata
            .keys()
            .filter(|filename| {
                // Извлекаем путь файла из имени кэша
                filename.contains(&file_str.replace(['/', '\\'], "_"))
            })
            .cloned()
            .collect();
        
        for filename in keys_to_remove {
            self.remove_file(&filename)?;
        }
        
        self.save_metadata()?;
        tracing::debug!("Invalidated file cache for: {}", file_path.display());
        Ok(())
    }
    
    /// Очищает весь файловый кэш
    pub fn clear(&mut self) -> Result<()> {
        for filename in self.metadata.keys() {
            let file_path = self.cache_dir.join(filename);
            if file_path.exists() {
                fs::remove_file(&file_path)
                    .with_context(|| format!("Failed to remove cache file: {}", file_path.display()))?;
            }
        }
        
        self.metadata.clear();
        self.save_metadata()?;
        
        tracing::info!("File cache cleared");
        Ok(())
    }
    
    /// Выполняет сборку мусора (удаляет устаревшие файлы)
    pub fn garbage_collect(&mut self) -> Result<()> {
        let now = SystemTime::now();
        let max_age = Duration::from_secs(self.max_age_seconds);
        
        let expired_files: Vec<String> = self.metadata
            .iter()
            .filter(|(_, metadata)| {
                now.duration_since(metadata.accessed_at)
                    .is_ok_and(|age| age > max_age)
            })
            .map(|(filename, _)| filename.clone())
            .collect();
        
        for filename in expired_files {
            self.remove_file(&filename)?;
        }
        
        self.save_metadata()?;
        tracing::debug!("File cache garbage collection completed");
        Ok(())
    }
    
    /// Получает статистику файлового кэша
    pub fn get_statistics(&self) -> FileCacheStatistics {
        let total_files = self.metadata.len();
        let total_size = self.metadata.values().map(|m| m.file_size).sum();
        let compressed_files = self.metadata.values().filter(|m| m.compressed).count();
        
        FileCacheStatistics {
            total_files,
            total_size_bytes: total_size,
            compressed_files,
            cache_directory: self.cache_dir.clone(),
        }
    }
    
    /// Преобразует ключ кэша в имя файла
    fn key_to_filename(&self, key: &CacheKey) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();
        
        format!("cache_{:x}.dat", hash)
    }
    
    /// Читает файл кэша
    fn read_cache_file(&self, file_path: &Path) -> Result<FileCacheEntry> {
        let mut file = fs::File::open(file_path)
            .with_context(|| format!("Failed to open cache file: {}", file_path.display()))?;
        
        if self.compression_enabled {
            let mut decoder = GzDecoder::new(file);
            let mut content = Vec::new();
            decoder.read_to_end(&mut content)
                .context("Failed to decompress cache file")?;
            
            bincode::deserialize(&content)
                .context("Failed to deserialize compressed cache entry")
        } else {
            let mut content = Vec::new();
            file.read_to_end(&mut content)
                .context("Failed to read cache file")?;
            
            bincode::deserialize(&content)
                .context("Failed to deserialize cache entry")
        }
    }
    
    /// Записывает файл кэша
    fn write_cache_file(&self, file_path: &Path, entry: &FileCacheEntry) -> Result<()> {
        let serialized = bincode::serialize(entry)
            .context("Failed to serialize cache entry")?;
        
        if self.compression_enabled {
            let file = fs::File::create(file_path)
                .with_context(|| format!("Failed to create cache file: {}", file_path.display()))?;
            
            let mut encoder = GzEncoder::new(file, Compression::default());
            encoder.write_all(&serialized)
                .context("Failed to write compressed data")?;
            encoder.finish()
                .context("Failed to finish compression")?;
        } else {
            fs::write(file_path, &serialized)
                .with_context(|| format!("Failed to write cache file: {}", file_path.display()))?;
        }
        
        Ok(())
    }
    
    /// Удаляет файл из кэша
    fn remove_file(&mut self, filename: &str) -> Result<()> {
        let file_path = self.cache_dir.join(filename);
        if file_path.exists() {
            fs::remove_file(&file_path)
                .with_context(|| format!("Failed to remove cache file: {}", file_path.display()))?;
        }
        
        self.metadata.remove(filename);
        Ok(())
    }
    
    /// Вычисляет хэш содержимого
    fn calculate_content_hash(&self, value: &CacheValue) -> Result<String> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        value.data.hash(&mut hasher);
        Ok(format!("{:x}", hasher.finish()))
    }
    
    /// Загружает метаданные из файла
    fn load_metadata(&mut self) -> Result<()> {
        let metadata_path = self.cache_dir.join("metadata.json");
        if !metadata_path.exists() {
            return Ok(());
        }
        
        let content = fs::read_to_string(&metadata_path)
            .context("Failed to read metadata file")?;
        
        self.metadata = serde_json::from_str(&content)
            .context("Failed to deserialize metadata")?;
        
        tracing::debug!("Loaded file cache metadata: {} entries", self.metadata.len());
        Ok(())
    }
    
    /// Сохраняет метаданные в файл
    fn save_metadata(&self) -> Result<()> {
        let metadata_path = self.cache_dir.join("metadata.json");
        let content = serde_json::to_string_pretty(&self.metadata)
            .context("Failed to serialize metadata")?;
        
        fs::write(&metadata_path, content)
            .context("Failed to write metadata file")?;
        
        Ok(())
    }
    
    /// Устанавливает максимальный возраст файлов в кэше
    pub fn set_max_age(&mut self, seconds: u64) {
        self.max_age_seconds = seconds;
    }
    
    /// Получает директорию кэша
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }
}

/// Статистика файлового кэша
#[derive(Debug, Clone)]
pub struct FileCacheStatistics {
    pub total_files: usize,
    pub total_size_bytes: u64,
    pub compressed_files: usize,
    pub cache_directory: PathBuf,
}

impl std::fmt::Display for FileCacheStatistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "File Cache Statistics:\n\
             Total files: {}\n\
             Total size: {:.2} MB\n\
             Compressed files: {}\n\
             Cache directory: {}",
            self.total_files,
            self.total_size_bytes as f64 / (1024.0 * 1024.0),
            self.compressed_files,
            self.cache_directory.display()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::cache::{CacheKey, CacheValue, CacheType};
    
    #[test]
    fn test_file_cache_creation() {
        let temp_dir = TempDir::new().unwrap();
        let cache = FileCache::new(temp_dir.path(), false);
        
        assert_eq!(cache.cache_dir(), temp_dir.path());
        assert_eq!(cache.metadata.len(), 0);
    }
    
    #[test]
    fn test_file_cache_set_get() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = FileCache::new(temp_dir.path(), false);
        
        let key = CacheKey {
            cache_type: CacheType::ParseResult,
            file_path: Some("test.bsl".into()),
            content_hash: "hash123".to_string(),
            params: HashMap::new(),
        };
        
        let data = vec![1, 2, 3, 4, 5];
        let value = CacheValue::new(&data, "test".to_string()).unwrap();
        
        cache.set(key.clone(), value.clone()).unwrap();
        let retrieved = cache.get(&key).unwrap().unwrap();
        
        let deserialized: Vec<i32> = retrieved.deserialize().unwrap();
        assert_eq!(deserialized, data);
    }
    
    #[test]
    fn test_file_cache_compression() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = FileCache::new(temp_dir.path(), true);
        
        let key = CacheKey {
            cache_type: CacheType::ParseResult,
            file_path: Some("test.bsl".into()),
            content_hash: "hash123".to_string(),
            params: HashMap::new(),
        };
        
        let data = vec![1; 1000]; // Большие данные для сжатия
        let value = CacheValue::new(&data, "test".to_string()).unwrap();
        
        cache.set(key.clone(), value).unwrap();
        let retrieved = cache.get(&key).unwrap().unwrap();
        
        let deserialized: Vec<i32> = retrieved.deserialize().unwrap();
        assert_eq!(deserialized, data);
    }
    
    #[test]
    fn test_file_cache_invalidation() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = FileCache::new(temp_dir.path(), false);
        
        let file_path = PathBuf::from("test.bsl");
        let key = CacheKey {
            cache_type: CacheType::ParseResult,
            file_path: Some(file_path.clone()),
            content_hash: "hash123".to_string(),
            params: HashMap::new(),
        };
        
        let value = CacheValue::new(&vec![1, 2, 3], "test".to_string()).unwrap();
        cache.set(key.clone(), value).unwrap();
        
        assert!(cache.get(&key).unwrap().is_some());
        
        cache.invalidate_file(&file_path).unwrap();
        assert!(cache.get(&key).unwrap().is_none());
    }
    
    #[test]
    fn test_file_cache_statistics() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = FileCache::new(temp_dir.path(), true);
        
        let key = CacheKey {
            cache_type: CacheType::ParseResult,
            file_path: Some("test.bsl".into()),
            content_hash: "hash123".to_string(),
            params: HashMap::new(),
        };
        
        let value = CacheValue::new(&vec![1, 2, 3], "test".to_string()).unwrap();
        cache.set(key, value).unwrap();
        
        let stats = cache.get_statistics();
        assert_eq!(stats.total_files, 1);
        assert_eq!(stats.compressed_files, 1);
        assert!(stats.total_size_bytes > 0);
    }
}