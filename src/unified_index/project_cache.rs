use anyhow::{Context, Result};
use chrono;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use super::configuration_watcher::ConfigurationWatcher;
use super::entity::BslEntity;
use super::index::UnifiedBslIndex;

/// Сериализуемый граф наследования для кеширования
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InheritanceGraph {
    /// Ребра графа: (parent_id, child_id)
    pub edges: Vec<(String, String)>,
    /// Версия формата графа (для миграций)
    pub version: u32,
}

impl InheritanceGraph {
    pub fn new() -> Self {
        Self {
            edges: Vec::new(),
            version: 1,
        }
    }
}

impl Default for InheritanceGraph {
    fn default() -> Self {
        Self::new()
    }
}
impl InheritanceGraph {
    /// Создает граф из UnifiedBslIndex
    pub fn from_index(index: &UnifiedBslIndex) -> Self {
        let edges = index.serialize_inheritance_graph();
        Self { edges, version: 1 }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ProjectManifest {
    pub project_name: String,
    pub config_path: String,
    pub platform_version: String,
    pub created_at: String,
    pub updated_at: String,
    pub entities_count: usize,
    pub platform_entities_count: usize,
    pub config_entities_count: usize,
}

pub struct ProjectIndexCache {
    cache_dir: PathBuf,
    /// Кеш графа наследования в памяти для текущей сессии
    #[allow(dead_code)]
    inheritance_cache: Option<InheritanceGraph>,
    /// Отслеживание изменений конфигурации для инкрементального обновления
    pub configuration_watcher: Option<ConfigurationWatcher>,
}

impl ProjectIndexCache {
    pub fn new() -> Result<Self> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
        let cache_dir = home_dir.join(".bsl_analyzer").join("project_indices");

        fs::create_dir_all(&cache_dir).context("Failed to create project indices directory")?;

        Ok(Self {
            cache_dir,
            inheritance_cache: None,
            configuration_watcher: None,
        })
    }

    /// Gets or creates project index from cache with advanced change tracking
    pub fn get_or_create(
        &mut self,
        config_path: &Path,
        platform_version: &str,
        builder_fn: &dyn Fn() -> Result<UnifiedBslIndex>,
    ) -> Result<UnifiedBslIndex> {
        let project_name = self.generate_project_name(config_path);
        let project_dir = self.get_project_versioned_dir(&project_name, platform_version);
        let manifest_file = project_dir.join("manifest.json");

        // Инициализируем или обновляем ConfigurationWatcher
        let should_rebuild = if let Some(ref mut watcher) = self.configuration_watcher {
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

            impact.requires_rebuild()
        } else {
            // Первый запуск - создаем ConfigurationWatcher
            match ConfigurationWatcher::new(config_path) {
                Ok(watcher) => {
                    tracing::info!(
                        "Created ConfigurationWatcher for {} files",
                        watcher.tracked_files_count()
                    );
                    self.configuration_watcher = Some(watcher);
                    false // Не требуем перестройку при первом создании watcher
                }
                Err(e) => {
                    tracing::warn!("Failed to create ConfigurationWatcher: {}", e);
                    false
                }
            }
        };

        // Check if cached version exists and is valid
        if manifest_file.exists() && !should_rebuild {
            if let Ok(manifest) = self.load_manifest(&manifest_file) {
                // Check if cache is still valid
                if manifest.platform_version == platform_version
                    && self.is_cache_fresh(&manifest, config_path)?
                {
                    tracing::debug!("Loading index from cache");
                    return self.load_from_cache(&project_dir);
                }
            }
        }

        // Build new index and save to cache
        tracing::info!(
            "Building new unified index{}",
            if should_rebuild {
                " (configuration changed)"
            } else {
                ""
            }
        );
        let index = builder_fn()?;
        self.save_to_cache(&project_name, config_path, platform_version, &index)?;

        Ok(index)
    }

    /// Saves unified index to project cache
    pub fn save_to_cache(
        &self,
        project_name: &str,
        config_path: &Path,
        platform_version: &str,
        index: &UnifiedBslIndex,
    ) -> Result<()> {
        let project_dir = self.get_project_versioned_dir(project_name, platform_version);
        fs::create_dir_all(&project_dir).context("Failed to create project directory")?;

        // Separate entities by type
        let (platform_entities, config_entities): (Vec<_>, Vec<_>) =
            index.get_all_entities().into_iter().partition(|entity| {
                matches!(entity.entity_type, super::entity::BslEntityType::Platform)
            });

        // Save config entities to JSONL
        let config_entities_file = project_dir.join("config_entities.jsonl");
        self.save_entities_to_jsonl(&config_entities_file, &config_entities)?;

        // Save only configuration-specific index metadata
        // Platform types will be loaded separately from platform_cache
        let index_metadata = IndexMetadata {
            by_name: config_entities
                .iter()
                .map(|entity| (entity.display_name.clone(), entity.id.0.clone()))
                .collect(),
            by_qualified_name: config_entities
                .iter()
                .map(|entity| (entity.qualified_name.clone(), entity.id.0.clone()))
                .collect(),
            // Note: We only save config entity indices, not platform indices
            // Platform indices are rebuilt from platform_cache on load
        };

        let unified_index_file = project_dir.join("unified_index.json");
        let index_json = serde_json::to_string_pretty(&index_metadata)?;
        fs::write(&unified_index_file, index_json).context("Failed to write unified index file")?;

        // Save inheritance graph
        let inheritance_graph = InheritanceGraph::from_index(index);
        let inheritance_file = project_dir.join("inheritance_graph.json");
        let inheritance_json = serde_json::to_string_pretty(&inheritance_graph)?;
        fs::write(&inheritance_file, inheritance_json)
            .context("Failed to write inheritance graph file")?;

        // Save manifest
        let manifest = ProjectManifest {
            project_name: project_name.to_string(),
            config_path: config_path.to_string_lossy().to_string(),
            platform_version: platform_version.to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            entities_count: index.get_entity_count(),
            platform_entities_count: platform_entities.len(),
            config_entities_count: config_entities.len(),
        };

        let manifest_file = project_dir.join("manifest.json");
        let manifest_json = serde_json::to_string_pretty(&manifest)?;
        fs::write(&manifest_file, manifest_json).context("Failed to write manifest file")?;

        Ok(())
    }

    /// Loads unified index from project cache
    pub fn load_from_cache(&self, project_dir: &Path) -> Result<UnifiedBslIndex> {
        // Read manifest to get platform version
        let manifest_file = project_dir.join("manifest.json");
        let manifest = self.load_manifest(&manifest_file)?;

        let mut index = UnifiedBslIndex::new();

        // 1. Load platform entities from platform cache
        let platform_cache = super::platform_cache::PlatformDocsCache::new()?;
        let platform_entities = platform_cache
            .get_or_create(&manifest.platform_version)
            .context("Failed to load platform types from cache")?;

        for entity in platform_entities {
            index.add_entity(entity)?;
        }

        // 2. Load config entities from project cache
        let config_entities_file = project_dir.join("config_entities.jsonl");
        let config_entities = self.load_entities_from_jsonl(&config_entities_file)?;

        for entity in config_entities {
            index.add_entity(entity)?;
        }

        // 3. Load cached inheritance graph
        let inheritance_file = project_dir.join("inheritance_graph.json");
        if inheritance_file.exists() {
            let inheritance_json = fs::read_to_string(&inheritance_file)
                .context("Failed to read inheritance graph file")?;
            let inheritance_graph: InheritanceGraph = serde_json::from_str(&inheritance_json)
                .context("Failed to deserialize inheritance graph")?;

            // Load inheritance relationships from cache
            index.load_inheritance_graph(inheritance_graph)?;
        } else {
            // Fallback: build inheritance relationships (legacy)
            index.build_inheritance_relationships()?;
        }

        Ok(index)
    }

    fn save_entities_to_jsonl(&self, file_path: &Path, entities: &[&BslEntity]) -> Result<()> {
        let file = fs::File::create(file_path).context("Failed to create entities file")?;
        let mut writer = BufWriter::new(file);

        for entity in entities {
            let json = serde_json::to_string(entity)?;
            writeln!(writer, "{}", json)?;
        }

        writer.flush()?;
        Ok(())
    }

    fn load_entities_from_jsonl(&self, file_path: &Path) -> Result<Vec<BslEntity>> {
        if !file_path.exists() {
            return Ok(Vec::new());
        }

        let file = fs::File::open(file_path).context("Failed to open entities file")?;
        let reader = BufReader::new(file);
        let mut entities = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if !line.trim().is_empty() {
                let entity: BslEntity =
                    serde_json::from_str(&line).context("Failed to deserialize entity")?;
                entities.push(entity);
            }
        }

        Ok(entities)
    }

    pub fn load_manifest(&self, manifest_file: &Path) -> Result<ProjectManifest> {
        let content = fs::read_to_string(manifest_file)?;
        let manifest: ProjectManifest = serde_json::from_str(&content)?;
        Ok(manifest)
    }

    pub fn is_cache_fresh(&self, manifest: &ProjectManifest, config_path: &Path) -> Result<bool> {
        // Check if Configuration.xml has been modified since cache creation
        let config_xml = config_path.join("Configuration.xml");
        if config_xml.exists() {
            let config_modified = fs::metadata(&config_xml)?.modified()?;
            let cache_created = chrono::DateTime::parse_from_rfc3339(&manifest.created_at)?;
            let cache_created_system: std::time::SystemTime = cache_created.into();

            if config_modified > cache_created_system {
                return Ok(false);
            }
        }

        // TODO: Check other files for modifications
        // For now, assume cache is fresh if Configuration.xml hasn't changed
        Ok(true)
    }

    pub fn generate_project_name(&self, config_path: &Path) -> String {
        // Try to extract UUID from Configuration.xml for stable project ID
        if let Ok(uuid) = self.extract_config_uuid(config_path) {
            let project_name = config_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            // Format: projectname_uuid (e.g., "ConfTest_787997b1dd2a4b98a8ccc38eb2830949")
            return format!("{}_{}", project_name, uuid.replace("-", ""));
        }

        // Fallback to path-based hash for non-1C configurations
        self.generate_project_name_fallback(config_path)
    }

    fn extract_config_uuid(&self, config_path: &Path) -> Result<String> {
        let config_xml = config_path.join("Configuration.xml");
        let content =
            fs::read_to_string(&config_xml).context("Failed to read Configuration.xml")?;

        // Extract UUID from <Configuration uuid="...">
        if let Some(start) = content.find(r#"<Configuration uuid=""#) {
            let start_pos = start + r#"<Configuration uuid=""#.len();
            if let Some(end_pos) = content[start_pos..].find('"') {
                let uuid = &content[start_pos..start_pos + end_pos];
                return Ok(uuid.to_string());
            }
        }

        anyhow::bail!("UUID not found in Configuration.xml")
    }

    fn generate_project_name_fallback(&self, config_path: &Path) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Use canonical path to ensure uniqueness
        let canonical_path = config_path
            .canonicalize()
            .unwrap_or_else(|_| config_path.to_path_buf());

        // Create a unique identifier based on full path
        let mut hasher = DefaultHasher::new();
        canonical_path.hash(&mut hasher);
        let hash = hasher.finish();

        // Include readable project name if possible
        let project_name = canonical_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        // Format: projectname_hash (e.g., "ConfTest_a1b2c3d4")
        format!("{}_{:x}", project_name, hash)
    }

    #[allow(dead_code)]
    fn get_project_dir(&self, project_name: &str) -> PathBuf {
        self.cache_dir.join(project_name)
    }

    pub fn get_project_versioned_dir(&self, project_name: &str, platform_version: &str) -> PathBuf {
        // Structure: project_indices/ProjectName_hash/8.3.25/
        self.cache_dir
            .join(project_name)
            .join(platform_version)
    }
}

#[derive(Serialize, Deserialize, Default)]
struct IndexMetadata {
    pub by_name: HashMap<String, String>, // name -> entity_id
    pub by_qualified_name: HashMap<String, String>, // qualified_name -> entity_id
                                          // TODO: Add other index metadata as needed
}
