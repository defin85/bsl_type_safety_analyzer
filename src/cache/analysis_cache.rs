/*!
# Analysis Cache Implementation

Специализированный кэш для результатов анализа BSL кода.
Оптимизирован для кэширования AST, семантического анализа и зависимостей.
*/

use crate::cache::{CacheKey, CacheManager, CacheType, CacheValue};
use crate::parser::ast::AstNode;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Специализированный кэш для анализа BSL
pub struct AnalysisCache {
    /// Основной менеджер кэша
    cache_manager: CacheManager,
    /// Кэш результатов парсинга AST
    ast_cache: HashMap<String, AstCacheEntry>,
    /// Кэш результатов семантического анализа
    semantic_cache: HashMap<String, SemanticCacheEntry>,
    /// Кэш зависимостей между модулями
    dependency_cache: HashMap<String, DependencyCacheEntry>,
    /// Время последней очистки кэша
    last_cleanup: SystemTime,
}

/// Ключ для кэша анализа
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AnalysisCacheKey {
    /// Путь к файлу
    pub file_path: PathBuf,
    /// Хэш содержимого файла
    pub content_hash: String,
    /// Тип анализа
    pub analysis_type: AnalysisType,
    /// Версия анализатора
    pub analyzer_version: String,
}

/// Значение в кэше анализа
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisCacheValue {
    /// Тип результата анализа
    pub result_type: String,
    /// Сериализованные данные результата
    pub data: Vec<u8>,
    /// Время создания
    pub created_at: SystemTime,
    /// Время анализа в миллисекундах
    pub analysis_time_ms: u64,
    /// Размер исходного файла в байтах
    pub file_size_bytes: u64,
}

/// Тип анализа для кэширования
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnalysisType {
    /// Парсинг в AST
    Parsing,
    /// Семантический анализ
    Semantic,
    /// Анализ зависимостей
    Dependencies,
    /// Проверка типов
    TypeChecking,
    /// Анализ стиля кода
    StyleAnalysis,
}

/// Запись кэша AST
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AstCacheEntry {
    /// AST дерево
    pub ast: AstNode,
    /// Хэш содержимого
    pub content_hash: String,
    /// Время создания
    pub created_at: SystemTime,
    /// Время парсинга в микросекундах
    pub parse_time_us: u64,
}

/// Запись кэша семантического анализа
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SemanticCacheEntry {
    /// Результаты семантического анализа
    pub analysis_results: SemanticAnalysisResults,
    /// Хэш содержимого
    pub content_hash: String,
    /// Время создания
    pub created_at: SystemTime,
    /// Зависимые файлы
    pub dependencies: Vec<PathBuf>,
}

/// Запись кэша зависимостей
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DependencyCacheEntry {
    /// Граф зависимостей
    pub dependency_graph: DependencyGraphData,
    /// Время создания
    pub created_at: SystemTime,
    /// Список проанализированных файлов
    pub analyzed_files: Vec<PathBuf>,
}

/// Результаты семантического анализа (упрощенная версия)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticAnalysisResults {
    /// Найденные ошибки
    pub errors: Vec<AnalysisError>,
    /// Предупреждения
    pub warnings: Vec<AnalysisError>,
    /// Найденные символы
    pub symbols: Vec<SymbolInfo>,
    /// Информация о типах
    pub types: HashMap<String, TypeInfo>,
}

/// Ошибка анализа
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisError {
    /// Код ошибки
    pub code: String,
    /// Сообщение об ошибке
    pub message: String,
    /// Позиция в файле
    pub position: Position,
    /// Серьезность
    pub severity: Severity,
}

/// Информация о символе
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolInfo {
    /// Имя символа
    pub name: String,
    /// Тип символа
    pub symbol_type: SymbolType,
    /// Позиция определения
    pub definition_position: Position,
    /// Список использований
    pub usages: Vec<Position>,
}

/// Информация о типе
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeInfo {
    /// Имя типа
    pub name: String,
    /// Базовый тип
    pub base_type: Option<String>,
    /// Методы типа
    pub methods: Vec<String>,
    /// Свойства типа
    pub properties: Vec<String>,
}

/// Данные графа зависимостей
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraphData {
    /// Узлы графа
    pub nodes: HashMap<String, DependencyNodeData>,
    /// Рёбра графа
    pub edges: Vec<DependencyEdgeData>,
    /// Обнаруженные циклы
    pub cycles: Vec<Vec<String>>,
}

/// Данные узла зависимостей
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyNodeData {
    /// Имя модуля
    pub name: String,
    /// Тип модуля
    pub module_type: String,
    /// Экспортируемые символы
    pub exports: Vec<String>,
    /// Импортируемые символы
    pub imports: Vec<String>,
}

/// Данные ребра зависимостей
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEdgeData {
    /// Источник зависимости
    pub from: String,
    /// Цель зависимости
    pub to: String,
    /// Тип зависимости
    pub dependency_type: String,
}

/// Позиция в файле
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

/// Серьезность ошибки
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Hint,
}

/// Тип символа
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SymbolType {
    Function,
    Procedure,
    Variable,
    Constant,
    Class,
    Method,
    Property,
}

impl AnalysisCache {
    /// Создает новый кэш анализа
    pub fn new(max_entries: usize) -> Self {
        Self {
            cache_manager: CacheManager::new(max_entries),
            ast_cache: HashMap::new(),
            semantic_cache: HashMap::new(),
            dependency_cache: HashMap::new(),
            last_cleanup: SystemTime::now(),
        }
    }

    /// Кэширует результат парсинга AST
    pub fn cache_ast_result(
        &mut self,
        file_path: &Path,
        content_hash: String,
        ast: AstNode,
        parse_time_us: u64,
    ) -> Result<()> {
        let entry = AstCacheEntry {
            ast: ast.clone(),
            content_hash: content_hash.clone(),
            created_at: SystemTime::now(),
            parse_time_us,
        };

        let cache_key = AnalysisCacheKey {
            file_path: file_path.to_path_buf(),
            content_hash,
            analysis_type: AnalysisType::Parsing,
            analyzer_version: env!("CARGO_PKG_VERSION").to_string(),
        };

        // Сохраняем в специализированный кэш
        let file_key = file_path.to_string_lossy().to_string();
        self.ast_cache.insert(file_key, entry);

        // Также сохраняем в основной кэш для персистентности
        let cache_value = AnalysisCacheValue::new(&ast, "AST".to_string(), parse_time_us, 0)?;
        let main_cache_key = CacheKey {
            cache_type: CacheType::ParseResult,
            file_path: Some(file_path.to_path_buf()),
            content_hash: cache_key.content_hash,
            params: HashMap::new(),
        };

        let main_cache_value = CacheValue::new(&cache_value, "AnalysisResult".to_string())?;
        self.cache_manager.set(main_cache_key, main_cache_value);

        tracing::debug!("Cached AST result for: {}", file_path.display());
        Ok(())
    }

    /// Получает кэшированный результат парсинга AST
    pub fn get_ast_result(&mut self, file_path: &Path, content_hash: &str) -> Option<AstNode> {
        let file_key = file_path.to_string_lossy().to_string();

        if let Some(entry) = self.ast_cache.get(&file_key) {
            if entry.content_hash == content_hash {
                tracing::debug!("AST cache hit for: {}", file_path.display());
                return Some(entry.ast.clone());
            } else {
                // Хэш не совпадает, удаляем устаревшую запись
                self.ast_cache.remove(&file_key);
            }
        }

        // Пытаемся получить из основного кэша
        let main_cache_key = CacheKey {
            cache_type: CacheType::ParseResult,
            file_path: Some(file_path.to_path_buf()),
            content_hash: content_hash.to_string(),
            params: HashMap::new(),
        };

        if let Some(cache_value) = self.cache_manager.get(&main_cache_key) {
            if let Ok(analysis_value) = cache_value.deserialize::<AnalysisCacheValue>() {
                if let Ok(ast) = bincode::deserialize::<AstNode>(&analysis_value.data) {
                    // Восстанавливаем в специализированный кэш
                    let entry = AstCacheEntry {
                        ast: ast.clone(),
                        content_hash: content_hash.to_string(),
                        created_at: analysis_value.created_at,
                        parse_time_us: analysis_value.analysis_time_ms * 1000,
                    };

                    self.ast_cache.insert(file_key, entry);
                    return Some(ast);
                }
            }
        }

        tracing::debug!("AST cache miss for: {}", file_path.display());
        None
    }

    /// Кэширует результат семантического анализа
    pub fn cache_semantic_result(
        &mut self,
        file_path: &Path,
        content_hash: String,
        results: SemanticAnalysisResults,
        dependencies: Vec<PathBuf>,
    ) -> Result<()> {
        let entry = SemanticCacheEntry {
            analysis_results: results.clone(),
            content_hash: content_hash.clone(),
            created_at: SystemTime::now(),
            dependencies,
        };

        let file_key = file_path.to_string_lossy().to_string();
        self.semantic_cache.insert(file_key, entry);

        // Сохраняем в основной кэш
        let cache_value = AnalysisCacheValue::new(&results, "SemanticAnalysis".to_string(), 0, 0)?;
        let main_cache_key = CacheKey {
            cache_type: CacheType::SemanticAnalysis,
            file_path: Some(file_path.to_path_buf()),
            content_hash,
            params: HashMap::new(),
        };

        let main_cache_value = CacheValue::new(&cache_value, "SemanticAnalysisResult".to_string())?;
        self.cache_manager.set(main_cache_key, main_cache_value);

        tracing::debug!(
            "Cached semantic analysis result for: {}",
            file_path.display()
        );
        Ok(())
    }

    /// Получает кэшированный результат семантического анализа
    pub fn get_semantic_result(
        &mut self,
        file_path: &Path,
        content_hash: &str,
    ) -> Option<SemanticAnalysisResults> {
        let file_key = file_path.to_string_lossy().to_string();

        if let Some(entry) = self.semantic_cache.get(&file_key) {
            if entry.content_hash == content_hash {
                // Проверяем актуальность зависимостей
                if self.are_dependencies_valid(&entry.dependencies) {
                    tracing::debug!("Semantic cache hit for: {}", file_path.display());
                    return Some(entry.analysis_results.clone());
                } else {
                    // Зависимости изменились, удаляем запись
                    self.semantic_cache.remove(&file_key);
                }
            } else {
                self.semantic_cache.remove(&file_key);
            }
        }

        tracing::debug!("Semantic cache miss for: {}", file_path.display());
        None
    }

    /// Инвалидирует кэш для файла
    pub fn invalidate_file(&mut self, file_path: &Path) {
        let file_key = file_path.to_string_lossy().to_string();

        self.ast_cache.remove(&file_key);
        self.semantic_cache.remove(&file_key);

        // Также инвалидируем зависимые файлы
        let dependent_files: Vec<String> = self
            .semantic_cache
            .iter()
            .filter(|(_, entry)| entry.dependencies.iter().any(|dep| dep == file_path))
            .map(|(key, _)| key.clone())
            .collect();

        for dep_file in dependent_files {
            self.semantic_cache.remove(&dep_file);
        }

        self.cache_manager.invalidate_file(file_path);

        tracing::debug!("Invalidated analysis cache for: {}", file_path.display());
    }

    /// Очищает весь кэш анализа
    pub fn clear(&mut self) {
        self.ast_cache.clear();
        self.semantic_cache.clear();
        self.dependency_cache.clear();
        self.cache_manager.clear();

        tracing::info!("Analysis cache cleared");
    }

    /// Выполняет автоматическую очистку устаревших записей
    pub fn auto_cleanup(&mut self) {
        let now = SystemTime::now();

        // Очищаем каждые 10 минут
        if now
            .duration_since(self.last_cleanup)
            .unwrap_or(Duration::ZERO)
            .as_secs()
            < 600
        {
            return;
        }

        let max_age = Duration::from_secs(3600); // 1 час

        // Очищаем AST кэш
        self.ast_cache.retain(|_, entry| {
            now.duration_since(entry.created_at)
                .unwrap_or(Duration::ZERO)
                < max_age
        });

        // Очищаем семантический кэш
        self.semantic_cache.retain(|_, entry| {
            now.duration_since(entry.created_at)
                .unwrap_or(Duration::ZERO)
                < max_age
        });

        // Очищаем кэш зависимостей
        self.dependency_cache.retain(|_, entry| {
            now.duration_since(entry.created_at)
                .unwrap_or(Duration::ZERO)
                < max_age
        });

        self.cache_manager.garbage_collect();
        self.last_cleanup = now;

        tracing::debug!("Analysis cache auto-cleanup completed");
    }

    /// Проверяет актуальность зависимостей
    fn are_dependencies_valid(&self, dependencies: &[PathBuf]) -> bool {
        for dep_path in dependencies {
            if let Ok(metadata) = std::fs::metadata(dep_path) {
                if let Ok(modified) = metadata.modified() {
                    // Проверяем, изменился ли файл за последний час
                    if let Ok(duration) = SystemTime::now().duration_since(modified) {
                        if duration.as_secs() < 3600 {
                            return false; // Файл недавно изменился
                        }
                    }
                }
            }
        }
        true
    }

    /// Получает статистику кэша анализа
    pub fn get_statistics(&self) -> AnalysisCacheStatistics {
        AnalysisCacheStatistics {
            ast_cache_entries: self.ast_cache.len(),
            semantic_cache_entries: self.semantic_cache.len(),
            dependency_cache_entries: self.dependency_cache.len(),
            main_cache_statistics: self.cache_manager.get_statistics(),
        }
    }
}

impl AnalysisCacheValue {
    /// Создает новое значение кэша анализа
    pub fn new<T: Serialize>(
        data: &T,
        result_type: String,
        analysis_time_ms: u64,
        file_size_bytes: u64,
    ) -> Result<Self> {
        let serialized = bincode::serialize(data).context("Failed to serialize analysis result")?;

        Ok(Self {
            result_type,
            data: serialized,
            created_at: SystemTime::now(),
            analysis_time_ms,
            file_size_bytes,
        })
    }

    /// Десериализует данные анализа
    pub fn deserialize<T: for<'de> Deserialize<'de>>(&self) -> Result<T> {
        bincode::deserialize(&self.data).context("Failed to deserialize analysis result")
    }
}

/// Статистика кэша анализа
#[derive(Debug, Clone)]
pub struct AnalysisCacheStatistics {
    pub ast_cache_entries: usize,
    pub semantic_cache_entries: usize,
    pub dependency_cache_entries: usize,
    pub main_cache_statistics: crate::cache::CacheStatistics,
}

impl std::fmt::Display for AnalysisCacheStatistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Analysis Cache Statistics:\n\
             AST cache entries: {}\n\
             Semantic cache entries: {}\n\
             Dependency cache entries: {}\n\
             Cache hit rate: {:.1}%",
            self.ast_cache_entries,
            self.semantic_cache_entries,
            self.dependency_cache_entries,
            self.main_cache_statistics.hit_rate() * 100.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analysis_cache_creation() {
        let cache = AnalysisCache::new(100);
        assert_eq!(cache.ast_cache.len(), 0);
        assert_eq!(cache.semantic_cache.len(), 0);
    }

    #[test]
    fn test_analysis_cache_key_creation() {
        let key = AnalysisCacheKey {
            file_path: "test.bsl".into(),
            content_hash: "hash123".to_string(),
            analysis_type: AnalysisType::Parsing,
            analyzer_version: "1.0.0".to_string(),
        };

        assert_eq!(key.analysis_type, AnalysisType::Parsing);
        assert_eq!(key.content_hash, "hash123");
    }

    #[test]
    fn test_analysis_cache_value_creation() {
        let data = vec![1, 2, 3, 4, 5];
        let value = AnalysisCacheValue::new(&data, "test".to_string(), 100, 1024).unwrap();

        assert_eq!(value.result_type, "test");
        assert_eq!(value.analysis_time_ms, 100);
        assert_eq!(value.file_size_bytes, 1024);

        let deserialized: Vec<i32> = value.deserialize().unwrap();
        assert_eq!(deserialized, data);
    }
}
