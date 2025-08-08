use anyhow::{Context, Result};
use log::info;
use std::path::Path;

use super::entity::{BslApplicationMode, BslEntity, BslProperty};
use super::index::UnifiedBslIndex;
use super::platform_cache::PlatformDocsCache;
use super::project_cache::ProjectIndexCache;
use super::xml_parser::ConfigurationXmlParser;

pub struct UnifiedIndexBuilder {
    platform_cache: PlatformDocsCache,
    project_cache: ProjectIndexCache,
    application_mode: BslApplicationMode,
    platform_docs_archive: Option<std::path::PathBuf>,
}

impl UnifiedIndexBuilder {
    pub fn new() -> Result<Self> {
        Ok(Self {
            platform_cache: PlatformDocsCache::new()?,
            project_cache: ProjectIndexCache::new()?,
            application_mode: BslApplicationMode::ManagedApplication, // по умолчанию управляемый режим
            platform_docs_archive: None,
        })
    }

    pub fn with_application_mode(mut self, mode: BslApplicationMode) -> Self {
        self.application_mode = mode;
        self
    }

    pub fn with_platform_docs_archive(mut self, archive_path: Option<std::path::PathBuf>) -> Self {
        self.platform_docs_archive = archive_path;
        self
    }

    pub fn build_index(
        &mut self,
        config_path: impl AsRef<Path>,
        platform_version: &str,
    ) -> Result<UnifiedBslIndex> {
        let config_path = config_path.as_ref();

        info!("Building unified BSL index for: {}", config_path.display());
        info!("DEBUG: About to call project_cache.get_or_create");

        // Проверяем ConfigurationWatcher и кеш, затем строим индекс при необходимости
        let index = {
            let project_name = self.project_cache.generate_project_name(config_path);
            let project_dir = self
                .project_cache
                .get_project_versioned_dir(&project_name, platform_version);
            let manifest_file = project_dir.join("manifest.json");

            // Получаем информацию о необходимости перестройки
            let should_rebuild = self.check_configuration_changes(config_path)?;

            // Проверяем существующий кеш
            let use_cache = manifest_file.exists()
                && !should_rebuild
                && self.is_cache_valid(&manifest_file, config_path, platform_version)?;

            if use_cache {
                tracing::debug!("Loading index from cache");
                self.project_cache.load_from_cache(&project_dir)?
            } else {
                tracing::info!(
                    "Building new unified index{}",
                    if should_rebuild {
                        " (configuration changed)"
                    } else {
                        ""
                    }
                );
                let index = self.build_index_from_scratch(config_path, platform_version)?;
                self.project_cache.save_to_cache(
                    &project_name,
                    config_path,
                    platform_version,
                    &index,
                )?;
                index
            }
        };

        // Примитивные типы теперь добавляются в platform_cache автоматически

        Ok(index)
    }

    /// Проверяет изменения конфигурации через ConfigurationWatcher
    fn check_configuration_changes(&mut self, config_path: &Path) -> Result<bool> {
        // Инициализируем или обновляем ConfigurationWatcher
        if let Some(ref mut watcher) = self.project_cache.configuration_watcher {
            // Проверяем изменения через ConfigurationWatcher
            let changed_files = watcher.check_for_changes()?;
            let impact = watcher.analyze_change_impact(&changed_files);

            if !changed_files.is_empty() {
                tracing::info!(
                    "Configuration changes detected: {} files changed, impact: {:?}",
                    changed_files.len(),
                    impact
                );
            }

            Ok(impact.requires_rebuild())
        } else {
            // Первый запуск - создаем ConfigurationWatcher
            match super::configuration_watcher::ConfigurationWatcher::new(config_path) {
                Ok(watcher) => {
                    tracing::info!(
                        "Created ConfigurationWatcher for {} files",
                        watcher.tracked_files_count()
                    );
                    self.project_cache.configuration_watcher = Some(watcher);
                    Ok(false) // Не требуем перестройку при первом создании watcher
                }
                Err(e) => {
                    tracing::warn!("Failed to create ConfigurationWatcher: {}", e);
                    Ok(false)
                }
            }
        }
    }

    /// Проверяет валидность существующего кеша
    fn is_cache_valid(
        &self,
        manifest_file: &Path,
        config_path: &Path,
        platform_version: &str,
    ) -> Result<bool> {
        let manifest = self.project_cache.load_manifest(manifest_file)?;
        Ok(manifest.platform_version == platform_version
            && self.project_cache.is_cache_fresh(&manifest, config_path)?)
    }

    fn build_index_from_scratch(
        &mut self,
        config_path: &Path,
        platform_version: &str,
    ) -> Result<UnifiedBslIndex> {
        info!("Building index from scratch for: {}", config_path.display());
        info!("Application mode: {:?}", self.application_mode);

        let mut index = UnifiedBslIndex::with_application_mode(self.application_mode);

        // 1. Загружаем платформенные типы
        let start = std::time::Instant::now();
        let platform_entities = self
            .platform_cache
            .get_or_create_with_archive(platform_version, self.platform_docs_archive.as_deref())
            .context("Failed to load platform types")?;
        info!(
            "Platform types: {} (loaded in {:?})",
            platform_entities.len(),
            start.elapsed()
        );

        // Добавляем платформенные типы в индекс
        for entity in platform_entities {
            index.add_entity(entity)?;
        }

        // 2. Парсим конфигурацию
        let start = std::time::Instant::now();
        let xml_parser = ConfigurationXmlParser::new(config_path);
        let config_entities = xml_parser
            .parse_configuration()
            .context("Failed to parse configuration")?;
        info!(
            "Configuration objects: {} (parsed in {:?})",
            config_entities.len(),
            start.elapsed()
        );

        // Добавляем объекты конфигурации в индекс
        for entity in config_entities {
            index.add_entity(entity)?;
        }

        // 3. Загружаем данные из существующих парсеров (если есть)
        self.load_legacy_data(&mut index, config_path)?;

        // 4. Строим граф наследования
        let start = std::time::Instant::now();
        index.build_inheritance_relationships()?;

        // 5. Инициализируем глобальные алиасы 1С
        index.initialize_global_aliases()?;

        info!(
            "✅ Index built successfully: {} entities (total time: {:?})",
            index.get_entity_count(),
            start.elapsed()
        );

        Ok(index)
    }

    fn load_legacy_data(&self, index: &mut UnifiedBslIndex, _config_path: &Path) -> Result<()> {
        // Проверяем наличие данных от legacy парсеров
        let output_dir = std::path::PathBuf::from("output/hybrid_docs");

        if output_dir.exists() {
            // Загружаем metadata_types
            let metadata_types_dir = output_dir.join("configuration/metadata_types");
            if metadata_types_dir.exists() {
                let mut count = 0;
                for entry in std::fs::read_dir(&metadata_types_dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("json") {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if let Ok(legacy_data) =
                                serde_json::from_str::<serde_json::Value>(&content)
                            {
                                if let Ok(entity) = self.convert_legacy_metadata(&legacy_data) {
                                    index.add_entity(entity)?;
                                    count += 1;
                                }
                            }
                        }
                    }
                }
                if count > 0 {
                    info!("Legacy metadata: {} entities", count);
                }
            }

            // Загружаем формы
            let forms_dir = output_dir.join("configuration/forms");
            if forms_dir.exists() {
                let mut count = 0;
                for entry in walkdir::WalkDir::new(&forms_dir)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_type().is_file())
                    .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("json"))
                {
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        if let Ok(legacy_data) = serde_json::from_str::<serde_json::Value>(&content)
                        {
                            if let Ok(entity) = self.convert_legacy_form(&legacy_data) {
                                index.add_entity(entity)?;
                                count += 1;
                            }
                        }
                    }
                }
                if count > 0 {
                    info!("Legacy forms: {} entities", count);
                }
            }
        }

        Ok(())
    }

    fn convert_legacy_metadata(&self, legacy_data: &serde_json::Value) -> Result<BslEntity> {
        use super::entity::*;

        let name = legacy_data
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing name in legacy data"))?;

        let type_str = legacy_data
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let kind = match type_str {
            "catalog" => BslEntityKind::Catalog,
            "document" => BslEntityKind::Document,
            "information_register" => BslEntityKind::InformationRegister,
            "accumulation_register" => BslEntityKind::AccumulationRegister,
            _ => BslEntityKind::Other(type_str.to_string()),
        };

        let mut entity = BslEntity::new(
            name.to_string(),
            name.to_string(),
            BslEntityType::Configuration,
            kind,
        );

        // Конвертируем атрибуты в свойства
        if let Some(attributes) = legacy_data.get("attributes").and_then(|v| v.as_array()) {
            for attr in attributes {
                if let Ok(property) = self.convert_legacy_attribute(attr) {
                    entity
                        .interface
                        .properties
                        .insert(property.name.clone(), property);
                }
            }
        }

        // Конвертируем табличные части
        if let Some(tabular_sections) = legacy_data
            .get("tabular_sections")
            .and_then(|v| v.as_array())
        {
            for ts in tabular_sections {
                if let Some(ts_name) = ts.get("name").and_then(|v| v.as_str()) {
                    // Создаем упрощенную табличную часть без атрибутов для legacy данных
                    let tabular_section = super::entity::BslTabularSection {
                        name: ts_name.to_string(),
                        display_name: ts_name.to_string(),
                        attributes: Vec::new(),
                    };
                    entity.relationships.tabular_sections.push(tabular_section);
                }
            }
        }

        entity.source = BslEntitySource::TextReport {
            path: "legacy_import".to_string(),
        };

        Ok(entity)
    }

    fn convert_legacy_attribute(&self, attr_data: &serde_json::Value) -> Result<BslProperty> {
        use super::entity::*;

        let name = attr_data
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing attribute name"))?;

        let type_name = attr_data
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        Ok(BslProperty {
            name: name.to_string(),
            english_name: None,
            type_name,
            is_readonly: false,
            is_indexed: attr_data
                .get("indexed")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            documentation: attr_data
                .get("description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            availability: vec![BslContext::Server, BslContext::Client],
        })
    }

    fn convert_legacy_form(&self, form_data: &serde_json::Value) -> Result<BslEntity> {
        use super::entity::*;

        let name = form_data
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing form name"))?;

        let parent = form_data
            .get("parent")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");

        let form_type = form_data
            .get("form_type")
            .and_then(|v| v.as_str())
            .unwrap_or("ManagedForm");

        let kind = match form_type {
            "ManagedForm" => BslEntityKind::ManagedForm,
            "OrdinaryForm" => BslEntityKind::OrdinaryForm,
            _ => BslEntityKind::Form,
        };

        let mut entity = BslEntity::new(
            format!("{}.{}", parent, name),
            format!("{}.{}", parent, name),
            BslEntityType::Form,
            kind,
        );

        entity.relationships.owner = Some(parent.to_string());
        entity.source = BslEntitySource::FormXml {
            path: "legacy_import".to_string(),
        };

        // Конвертируем элементы формы в свойства
        if let Some(elements) = form_data.get("elements").and_then(|v| v.as_array()) {
            for element in elements {
                if let Some(elem_name) = element.get("name").and_then(|v| v.as_str()) {
                    let elem_type = element
                        .get("type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown");

                    let property = BslProperty {
                        name: elem_name.to_string(),
                        english_name: None,
                        type_name: elem_type.to_string(),
                        is_readonly: true,
                        is_indexed: false,
                        documentation: None,
                        availability: vec![BslContext::Client],
                    };

                    entity
                        .interface
                        .properties
                        .insert(elem_name.to_string(), property);
                }
            }
        }

        Ok(entity)
    }

    // ===== INCREMENTAL UPDATE METHODS =====

    /// Проверяет возможность инкрементального обновления
    pub fn check_incremental_update_feasibility(
        &mut self,
        config_path: &Path,
        _platform_version: &str,
    ) -> Result<(
        super::configuration_watcher::ChangeImpact,
        Vec<(
            std::path::PathBuf,
            super::configuration_watcher::ChangeImpact,
        )>,
    )> {
        use super::configuration_watcher::ConfigurationWatcher;

        // Инициализируем или получаем ConfigurationWatcher из project_cache
        if self.project_cache.configuration_watcher.is_none() {
            self.project_cache.configuration_watcher =
                Some(ConfigurationWatcher::new(config_path)?);
        }

        let watcher = self
            .project_cache
            .configuration_watcher
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("ConfigurationWatcher not initialized"))?;

        let changed_files_paths = watcher.check_for_changes()?;
        let impact = watcher.analyze_change_impact(&changed_files_paths);

        // Создаем детализированный список изменений
        let changed_files_with_impact: Vec<(
            std::path::PathBuf,
            super::configuration_watcher::ChangeImpact,
        )> = changed_files_paths
            .into_iter()
            .map(|path| {
                let file_impact =
                    if path.file_name().and_then(|n| n.to_str()) == Some("Configuration.xml") {
                        super::configuration_watcher::ChangeImpact::FullRebuild
                    } else if path.extension().and_then(|e| e.to_str()) == Some("xml") {
                        super::configuration_watcher::ChangeImpact::MetadataUpdate
                    } else if path.extension().and_then(|e| e.to_str()) == Some("bsl") {
                        super::configuration_watcher::ChangeImpact::ModuleUpdate
                    } else {
                        super::configuration_watcher::ChangeImpact::Minor
                    };
                (path, file_impact)
            })
            .collect();

        Ok((impact, changed_files_with_impact))
    }

    /// Выполняет инкрементальное обновление индекса
    pub fn perform_incremental_update(
        &mut self,
        config_path: &Path,
        platform_version: &str,
        changed_files: Vec<(
            std::path::PathBuf,
            super::configuration_watcher::ChangeImpact,
        )>,
    ) -> Result<super::index::IncrementalUpdateResult> {
        let project_name = self.project_cache.generate_project_name(config_path);
        let project_dir = self
            .project_cache
            .get_project_versioned_dir(&project_name, platform_version);

        // Загружаем существующий индекс
        let mut index = self
            .project_cache
            .load_from_cache(&project_dir)
            .context("Failed to load existing index for incremental update")?;

        let start = std::time::Instant::now();
        let mut changed_entities = Vec::new();

        // Анализируем изменения по типам файлов
        for (file_path, _impact) in changed_files {
            if let Some(file_name) = file_path.file_name().and_then(|n| n.to_str()) {
                match file_name {
                    "Configuration.xml" => {
                        // Перечитываем основные метаданные
                        let xml_parser = ConfigurationXmlParser::new(config_path);
                        let new_entities = xml_parser.parse_configuration()?;
                        changed_entities.extend(new_entities);
                    }
                    name if name.ends_with(".xml") => {
                        // Отдельные файлы объектов конфигурации
                        if let Ok(entity) = self.parse_individual_metadata_file(&file_path) {
                            changed_entities.push(entity);
                        }
                    }
                    name if name.ends_with(".bsl") => {
                        // BSL модули - пока не поддерживается детальный анализ
                        tracing::debug!("BSL module changed: {}, skipping detailed analysis", name);
                    }
                    _ => {
                        tracing::debug!("Unknown file type changed: {}", file_name);
                    }
                }
            }
        }

        // Выполняем инкрементальное обновление
        let result = index.update_entities(changed_entities)?;

        // Сохраняем обновленный индекс в кеш
        self.project_cache
            .save_to_cache(&project_name, config_path, platform_version, &index)?;

        tracing::info!(
            "Incremental update completed in {:.2?}: {} entities changed",
            start.elapsed(),
            result.total_changes()
        );

        Ok(result)
    }

    /// Парсит отдельный файл метаданных
    fn parse_individual_metadata_file(&self, file_path: &Path) -> Result<BslEntity> {
        // Для простоты пока используем базовую реализацию
        // В будущем можно добавить специализированные парсеры для разных типов объектов

        let file_content =
            std::fs::read_to_string(file_path).context("Failed to read metadata file")?;

        // Пытаемся извлечь основную информацию из XML
        if let Some(name_start) = file_content.find("<Properties>") {
            if let Some(_name_end) = file_content[name_start..].find("</Properties>") {
                // Простая эвристика для извлечения имени объекта
                if let Some(file_stem) = file_path.file_stem().and_then(|s| s.to_str()) {
                    let entity = BslEntity::new(
                        file_stem.to_string(),
                        file_stem.to_string(),
                        super::entity::BslEntityType::Configuration,
                        super::entity::BslEntityKind::CommonModule, // по умолчанию
                    );

                    return Ok(entity);
                }
            }
        }

        Err(anyhow::anyhow!(
            "Could not parse metadata file: {}",
            file_path.display()
        ))
    }
}
