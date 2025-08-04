/*!
# Configuration File Change Tracking

Отслеживание изменений файлов конфигурации 1С для инкрементального обновления
UnifiedBslIndex без полной перестройки.
*/

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::fs;
use anyhow::Result;
use serde::{Serialize, Deserialize};

/// Отслеживание изменений конфигурации для инкрементального обновления
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationWatcher {
    /// Хеши файлов для отслеживания изменений
    file_hashes: HashMap<PathBuf, u64>,
    /// Время последнего сканирования
    last_scan: SystemTime,
    /// Корневая директория конфигурации
    config_root: PathBuf,
}

impl ConfigurationWatcher {
    /// Создает новый ConfigurationWatcher для указанной директории
    pub fn new(config_path: &Path) -> Result<Self> {
        let mut watcher = Self {
            file_hashes: HashMap::new(),
            last_scan: SystemTime::now(),
            config_root: config_path.to_path_buf(),
        };
        
        // Начальное сканирование
        watcher.scan_configuration_files()?;
        
        Ok(watcher)
    }
    
    /// Сканирование файлов конфигурации и вычисление хешей
    pub fn scan_configuration_files(&mut self) -> Result<()> {
        self.file_hashes.clear();
        
        // Основные файлы конфигурации для отслеживания
        let key_files = [
            "Configuration.xml",
            "ConfigDumpInfo.xml",
        ];
        
        // Сканируем ключевые файлы
        for file_name in &key_files {
            let file_path = self.config_root.join(file_name);
            if file_path.exists() {
                let hash = self.calculate_file_hash(&file_path)?;
                self.file_hashes.insert(file_path, hash);
            }
        }
        
        // Сканируем директории с метаданными объектов
        let metadata_dirs = [
            "Catalogs",
            "Documents", 
            "DataProcessors",
            "Reports",
            "InformationRegisters",
            "AccumulationRegisters",
            "ChartsOfAccounts",
            "ChartsOfCharacteristicTypes",
            "ChartsOfCalculationTypes",
            "BusinessProcesses",
            "Tasks",
            "ExchangePlans",
            "FilterCriteria",
            "SettingsStorages",
            "FunctionalOptions",
            "FunctionalOptionsParameters",
            "DefinedTypes",
            "CommonModules",
            "SessionParameters",
            "Roles",
            "CommonAttributes",
            "ExchangePlans",
            "EventSubscriptions",
            "ScheduledJobs",
            "FunctionalOptions",
            "FunctionalOptionsParameters",
            "DefinedTypes",
            "CommonPictures",
            "CommonTemplates",
            "CommonForms",
            "CommonCommands",
        ];
        
        for dir_name in &metadata_dirs {
            let dir_path = self.config_root.join(dir_name);
            if dir_path.exists() && dir_path.is_dir() {
                self.scan_directory_recursive(&dir_path)?;
            }
        }
        
        self.last_scan = SystemTime::now();
        
        tracing::debug!(
            "Scanned {} configuration files", 
            self.file_hashes.len()
        );
        
        Ok(())
    }
    
    /// Рекурсивное сканирование директории
    fn scan_directory_recursive(&mut self, dir: &Path) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                self.scan_directory_recursive(&path)?;
            } else if path.is_file() {
                // Отслеживаем только XML файлы (метаданные) и BSL модули
                if let Some(extension) = path.extension() {
                    if extension == "xml" || extension == "bsl" {
                        let hash = self.calculate_file_hash(&path)?;
                        self.file_hashes.insert(path, hash);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Вычисление хеша файла (быстрый алгоритм FNV-1a)
    fn calculate_file_hash(&self, file_path: &Path) -> Result<u64> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        // Получаем метаданные файла для быстрой проверки
        let metadata = fs::metadata(file_path)?;
        let modified_time = metadata.modified()?;
        let file_size = metadata.len();
        
        // Создаем хеш на основе времени модификации и размера
        // Это быстрее чтения всего файла для больших конфигураций
        let mut hasher = DefaultHasher::new();
        file_path.hash(&mut hasher);
        modified_time.hash(&mut hasher);
        file_size.hash(&mut hasher);
        
        Ok(hasher.finish())
    }
    
    /// Проверка изменений с момента последнего сканирования
    pub fn check_for_changes(&mut self) -> Result<Vec<PathBuf>> {
        let mut changed_files = Vec::new();
        let mut current_hashes = HashMap::new();
        
        // Пересканируем все отслеживаемые файлы
        for file_path in self.file_hashes.keys() {
            if file_path.exists() {
                let current_hash = self.calculate_file_hash(file_path)?;
                current_hashes.insert(file_path.clone(), current_hash);
                
                // Проверяем изменения
                if let Some(&old_hash) = self.file_hashes.get(file_path) {
                    if old_hash != current_hash {
                        changed_files.push(file_path.clone());
                        tracing::debug!("File changed: {:?}", file_path);
                    }
                } else {
                    // Новый файл
                    changed_files.push(file_path.clone());
                    tracing::debug!("New file detected: {:?}", file_path);
                }
            } else {
                // Файл удален
                changed_files.push(file_path.clone());
                tracing::debug!("File deleted: {:?}", file_path);
            }
        }
        
        // Проверяем новые файлы (полное пересканирование периодически)
        if self.should_full_rescan() {
            let old_count = self.file_hashes.len();
            self.scan_configuration_files()?;
            
            if self.file_hashes.len() != old_count {
                tracing::info!(
                    "Configuration structure changed: {} -> {} files",
                    old_count, 
                    self.file_hashes.len()
                );
                
                // При изменении структуры конфигурации требуется полная перестройка
                return Ok(self.file_hashes.keys().cloned().collect());
            }
        } else {
            // Обновляем сохраненные хеши
            self.file_hashes = current_hashes;
        }
        
        self.last_scan = SystemTime::now();
        
        Ok(changed_files)
    }
    
    /// Определяет нужно ли полное пересканирование
    fn should_full_rescan(&self) -> bool {
        // Полное пересканирование каждые 5 минут или при первом запуске
        match self.last_scan.elapsed() {
            Ok(elapsed) => elapsed.as_secs() > 300, // 5 минут
            Err(_) => true, // Ошибка времени - делаем полное сканирование
        }
    }
    
    /// Получает список всех отслеживаемых файлов
    pub fn get_tracked_files(&self) -> Vec<&PathBuf> {
        self.file_hashes.keys().collect()
    }
    
    /// Получает количество отслеживаемых файлов
    pub fn tracked_files_count(&self) -> usize {
        self.file_hashes.len()
    }
    
    /// Определяет тип изменения на основе измененных файлов
    pub fn analyze_change_impact(&self, changed_files: &[PathBuf]) -> ChangeImpact {
        if changed_files.is_empty() {
            return ChangeImpact::None;
        }
        
        let mut has_config_xml = false;
        let mut has_metadata_files = false;
        let mut has_module_files = false;
        
        for file in changed_files {
            if let Some(file_name) = file.file_name() {
                let file_name_str = file_name.to_string_lossy();
                
                if file_name_str == "Configuration.xml" {
                    has_config_xml = true;
                } else if file_name_str.ends_with(".xml") {
                    has_metadata_files = true;
                } else if file_name_str.ends_with(".bsl") {
                    has_module_files = true;
                }
            }
        }
        
        if has_config_xml {
            ChangeImpact::FullRebuild
        } else if has_metadata_files {
            ChangeImpact::MetadataUpdate 
        } else if has_module_files {
            ChangeImpact::ModuleUpdate
        } else {
            ChangeImpact::Minor
        }
    }
}

/// Типы изменений конфигурации и их влияние на индекс
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeImpact {
    /// Нет изменений
    None,
    /// Минорные изменения (например, комментарии)
    Minor,
    /// Изменения в BSL модулях - обновить анализ модулей
    ModuleUpdate,
    /// Изменения в метаданных объектов - инкрементальное обновление 
    MetadataUpdate,
    /// Изменения в Configuration.xml - полная перестройка индекса
    FullRebuild,
}

impl ChangeImpact {
    /// Требует ли данное изменение перестройки индекса
    pub fn requires_rebuild(&self) -> bool {
        matches!(self, ChangeImpact::FullRebuild)
    }
    
    /// Требует ли данное изменение инкрементального обновления
    pub fn requires_incremental_update(&self) -> bool {
        matches!(self, ChangeImpact::MetadataUpdate | ChangeImpact::ModuleUpdate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    
    #[test]
    fn test_configuration_watcher_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path();
        
        // Создаем Configuration.xml
        let config_xml = config_path.join("Configuration.xml");
        fs::write(&config_xml, "<Configuration />").unwrap();
        
        let watcher = ConfigurationWatcher::new(config_path).unwrap();
        assert!(watcher.tracked_files_count() > 0);
    }
    
    #[test]
    fn test_change_impact_analysis() {
        let temp_dir = TempDir::new().unwrap();
        let watcher = ConfigurationWatcher::new(temp_dir.path()).unwrap();
        
        // Configuration.xml изменение -> полная перестройка
        let config_change = vec![temp_dir.path().join("Configuration.xml")];
        assert_eq!(
            watcher.analyze_change_impact(&config_change),
            ChangeImpact::FullRebuild
        );
        
        // Метаданные изменение -> инкрементальное обновление
        let metadata_change = vec![temp_dir.path().join("Catalogs/Users.xml")];
        assert_eq!(
            watcher.analyze_change_impact(&metadata_change),
            ChangeImpact::MetadataUpdate
        );
        
        // BSL модуль изменение -> обновление модуля
        let module_change = vec![temp_dir.path().join("CommonModules/Utils.bsl")];
        assert_eq!(
            watcher.analyze_change_impact(&module_change),
            ChangeImpact::ModuleUpdate
        );
    }
}