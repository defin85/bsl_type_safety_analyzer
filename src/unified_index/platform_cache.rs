use anyhow::{Context, Result};
use serde_json;
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use super::entity::{BslContext, BslEntity, BslMethod, BslParameter, BslProperty};

pub struct PlatformDocsCache {
    cache_dir: PathBuf,
}

#[allow(dead_code)]
impl PlatformDocsCache {
    pub fn new() -> Result<Self> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
        let cache_dir = home_dir.join(".bsl_analyzer").join("platform_cache");

        fs::create_dir_all(&cache_dir).context("Failed to create platform cache directory")?;

        Ok(Self { cache_dir })
    }

    pub fn get_or_create(&self, version: &str) -> Result<Vec<BslEntity>> {
        let cache_file = self.get_cache_file_path(version);

        if cache_file.exists() {
            self.load_from_cache(&cache_file)
        } else {
            // Если кеша нет, пытаемся извлечь из существующего hybrid storage
            self.extract_from_hybrid_storage(version)
        }
    }

    pub fn get_or_create_with_archive(
        &self,
        version: &str,
        archive_path: Option<&Path>,
    ) -> Result<Vec<BslEntity>> {
        let cache_file = self.get_cache_file_path(version);

        if cache_file.exists() {
            self.load_from_cache(&cache_file)
        } else {
            // Если кеша нет, пытаемся извлечь из указанного архива или fallback
            if let Some(archive) = archive_path {
                self.extract_from_custom_archive(archive, version)
            } else {
                self.extract_from_hybrid_storage(version)
            }
        }
    }

    pub fn save_to_cache(&self, version: &str, entities: &[BslEntity]) -> Result<()> {
        let cache_file = self.get_cache_file_path(version);
        let file = fs::File::create(&cache_file).context("Failed to create cache file")?;
        let mut writer = BufWriter::new(file);

        for entity in entities {
            let json = serde_json::to_string(entity)?;
            writeln!(writer, "{}", json)?;
        }

        writer.flush()?;
        Ok(())
    }

    fn get_cache_file_path(&self, version: &str) -> PathBuf {
        // Нормализуем версию - убираем префикс "v" если есть
        let normalized_version = version.strip_prefix("v").unwrap_or(version);
        self.cache_dir.join(format!("{}.jsonl", normalized_version))
    }

    fn load_from_cache(&self, cache_file: &Path) -> Result<Vec<BslEntity>> {
        let file = fs::File::open(cache_file).context("Failed to open cache file")?;
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

    fn extract_from_custom_archive(
        &self,
        archive_path: &Path,
        version: &str,
    ) -> Result<Vec<BslEntity>> {
        use crate::docs_integration::BslSyntaxExtractor;

        log::info!(
            "Extracting platform types from custom archive: {}",
            archive_path.display()
        );

        if !archive_path.exists() {
            return Err(anyhow::anyhow!(
                "Archive file does not exist: {}",
                archive_path.display()
            ));
        }

        // Извлекаем типы из архива
        let mut extractor = BslSyntaxExtractor::new(archive_path);
        let syntax_db = extractor
            .extract_syntax_database(None)
            .context("Failed to extract BSL syntax from custom archive")?;

        // Конвертируем в BslEntity
        let entities = self
            .convert_syntax_db_to_entities(&syntax_db, version)
            .context("Failed to convert syntax database to entities")?;

        log::info!(
            "Extracted {} platform types from custom archive",
            entities.len()
        );

        // Сохраняем в кеш для будущего использования
        if !entities.is_empty() {
            self.save_to_cache(version, &entities)?;
        }

        Ok(entities)
    }

    fn extract_from_hybrid_storage(&self, version: &str) -> Result<Vec<BslEntity>> {
        // NO FALLBACK - если архив не найден, возвращаем ошибку
        log::error!(
            "Platform documentation archive not provided for version {}. User must specify archive path.",
            version
        );
        
        Err(anyhow::anyhow!(
            "Platform documentation archive is required but not provided. \
             Please specify path to rebuilt.shcntx_ru.zip or rebuilt.shlang_ru.zip archive."
        ))
    }

    fn convert_method(&self, method_data: &serde_json::Value) -> Result<BslMethod> {
        let name = method_data
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing method name"))?;

        let mut method = BslMethod {
            name: name.to_string(),
            english_name: method_data
                .get("english_name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            parameters: Vec::new(),
            return_type: method_data
                .get("return_type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            documentation: method_data
                .get("description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            availability: Vec::new(),
            is_function: method_data
                .get("is_function")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            is_export: false,
            is_deprecated: method_data
                .get("is_deprecated")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            deprecation_info: method_data
                .get("deprecation_info")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        };

        // Конвертируем параметры
        if let Some(params) = method_data.get("parameters").and_then(|v| v.as_array()) {
            for param_data in params {
                if let Ok(param) = self.convert_parameter(param_data) {
                    method.parameters.push(param);
                }
            }
        }

        // Устанавливаем доступность
        if let Some(contexts) = method_data.get("availability").and_then(|v| v.as_array()) {
            method.availability = contexts
                .iter()
                .filter_map(|v| v.as_str())
                .filter_map(|s| self.parse_context(s))
                .collect();
        }

        Ok(method)
    }

    fn convert_property(&self, prop_data: &serde_json::Value) -> Result<BslProperty> {
        let name = prop_data
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing property name"))?;

        let mut property = BslProperty {
            name: name.to_string(),
            english_name: prop_data
                .get("english_name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            type_name: prop_data
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string(),
            is_readonly: prop_data
                .get("is_readonly")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            is_indexed: false,
            documentation: prop_data
                .get("description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            availability: Vec::new(),
        };

        // Устанавливаем доступность
        if let Some(contexts) = prop_data.get("availability").and_then(|v| v.as_array()) {
            property.availability = contexts
                .iter()
                .filter_map(|v| v.as_str())
                .filter_map(|s| self.parse_context(s))
                .collect();
        }

        Ok(property)
    }

    fn convert_parameter(&self, param_data: &serde_json::Value) -> Result<BslParameter> {
        let name = param_data
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing parameter name"))?;

        Ok(BslParameter {
            name: name.to_string(),
            type_name: param_data
                .get("type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            is_optional: param_data
                .get("is_optional")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            default_value: param_data
                .get("default_value")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            description: param_data
                .get("description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        })
    }

    fn parse_context(&self, context_str: &str) -> Option<BslContext> {
        match context_str {
            "Client" | "Клиент" => Some(BslContext::Client),
            "Server" | "Сервер" => Some(BslContext::Server),
            "ExternalConnection" | "ВнешнееСоединение" => {
                Some(BslContext::ExternalConnection)
            }
            "MobileApp" | "МобильноеПриложение" => Some(BslContext::MobileApp),
            "MobileClient" | "МобильныйКлиент" => Some(BslContext::MobileClient),
            "MobileServer" | "МобильныйСервер" => Some(BslContext::MobileServer),
            "ThickClient" | "ТолстыйКлиент" => Some(BslContext::ThickClient),
            "ThinClient" | "ТонкийКлиент" => Some(BslContext::ThinClient),
            "WebClient" | "ВебКлиент" => Some(BslContext::WebClient),
            _ => None,
        }
    }

    /// Добавляет базовые примитивные типы BSL, парся их из синтаксис-помощника
    fn add_primitive_types(&self, entities: &mut Vec<BslEntity>, version: &str) -> Result<()> {
        // Путь к архиву синтаксис-помощника
        let syntax_helper_path = PathBuf::from("examples/rebuilt.shcntx_ru.zip");
        if !syntax_helper_path.exists() {
            log::warn!(
                "Syntax helper archive not found at {}, using fallback primitive types",
                syntax_helper_path.display()
            );
            return self.add_fallback_primitive_types(entities, version);
        }

        // Парсим примитивные типы из архива
        match self.parse_primitive_types_from_archive(&syntax_helper_path, version) {
            Ok(primitive_entities) => {
                entities.extend(primitive_entities);
                log::info!(
                    "Successfully parsed {} primitive types from syntax helper",
                    entities.len()
                );
            }
            Err(e) => {
                log::warn!(
                    "Failed to parse primitive types from syntax helper: {}, using fallback",
                    e
                );
                self.add_fallback_primitive_types(entities, version)?;
            }
        }

        Ok(())
    }

    /// Парсит примитивные типы из архива синтаксис-помощника
    fn parse_primitive_types_from_archive(
        &self,
        archive_path: &Path,
        version: &str,
    ) -> Result<Vec<BslEntity>> {
        use std::io::Read;
        use zip::ZipArchive;

        let file = std::fs::File::open(archive_path)?;
        let mut archive = ZipArchive::new(file)?;

        let mut entities = Vec::new();

        // Список конструкторов примитивных типов в архиве
        let constructor_files = [
            (
                "objects/Global context/methods/catalog4841/String960.html",
                "Строка",
                "String",
            ),
            (
                "objects/Global context/methods/catalog4841/Boolean958.html",
                "Булево",
                "Boolean",
            ),
            (
                "objects/Global context/methods/catalog4841/Date961.html",
                "Дата",
                "Date",
            ),
            // Другие типы могут быть в других каталогах
        ];

        for (file_path, russian_name, english_name) in &constructor_files {
            match archive.by_name(file_path) {
                Ok(mut file) => {
                    let mut content = String::new();
                    file.read_to_string(&mut content)?;

                    if let Ok(entity) =
                        self.parse_constructor_html(&content, russian_name, english_name, version)
                    {
                        entities.push(entity);
                        log::debug!("Parsed primitive type: {}", russian_name);
                    }
                }
                Err(_) => {
                    log::debug!("Constructor file not found: {}", file_path);
                }
            }
        }

        // Добавляем типы, которые могут отсутствовать в архиве
        self.add_missing_primitive_types(&mut entities, version)?;

        Ok(entities)
    }

    /// Парсит HTML файл конструктора типа
    fn parse_constructor_html(
        &self,
        html_content: &str,
        russian_name: &str,
        english_name: &str,
        version: &str,
    ) -> Result<BslEntity> {
        use super::entity::*;
        use scraper::{Html, Selector};

        let document = Html::parse_document(html_content);

        // Извлекаем информацию из HTML
        let description_selector = Selector::parse("p").unwrap();

        let display_name = format!("{} ({})", russian_name, english_name);
        let mut entity = BslEntity::new(
            display_name.clone(),
            display_name.clone(),
            BslEntityType::Platform,
            BslEntityKind::Primitive,
        );

        entity.english_name = Some(english_name.to_string());
        entity.source = BslEntitySource::HBK {
            version: version.to_string(),
        };

        // Извлекаем описание
        for element in document.select(&description_selector) {
            let text = element.text().collect::<String>();
            if text.contains("Преобразует") && text.len() > 50 {
                entity.documentation = Some(text.trim().to_string());
                break;
            }
        }

        // Устанавливаем доступность (все контексты для примитивных типов)
        entity.availability = vec![
            BslContext::Server,
            BslContext::Client,
            BslContext::ThinClient,
            BslContext::ThickClient,
            BslContext::WebClient,
            BslContext::MobileClient,
            BslContext::ExternalConnection,
        ];

        // Добавляем основные методы для строк
        if russian_name == "Строка" {
            self.add_string_methods(&mut entity);
        }

        Ok(entity)
    }

    /// Добавляет отсутствующие примитивные типы
    fn add_missing_primitive_types(
        &self,
        entities: &mut Vec<BslEntity>,
        version: &str,
    ) -> Result<()> {
        use super::entity::*;

        // Типы, которые могут отсутствовать в архиве
        let missing_types = [
            ("Число", "Number", "Числовой примитивный тип"),
            ("Неопределено", "Undefined", "Неопределенное значение"),
            ("NULL", "NULL", "Значение NULL"),
            ("Тип", "Type", "Тип данных"),
            ("Произвольный", "Arbitrary", "Произвольный тип данных"),
        ];

        let existing_names: std::collections::HashSet<String> = entities
            .iter()
            .filter_map(|e| e.english_name.clone())
            .collect();

        for (russian_name, english_name, documentation) in &missing_types {
            if !existing_names.contains(*english_name) {
                let display_name = format!("{} ({})", russian_name, english_name);
                let mut entity = BslEntity::new(
                    display_name.clone(),
                    display_name.clone(),
                    BslEntityType::Platform,
                    BslEntityKind::Primitive,
                );

                entity.english_name = Some(english_name.to_string());
                entity.documentation = Some(documentation.to_string());
                entity.availability = vec![BslContext::Server, BslContext::Client];
                entity.source = BslEntitySource::HBK {
                    version: version.to_string(),
                };

                entities.push(entity);
            }
        }

        Ok(())
    }

    /// Fallback метод для добавления примитивных типов, если парсинг не удался
    fn add_fallback_primitive_types(
        &self,
        entities: &mut Vec<BslEntity>,
        version: &str,
    ) -> Result<()> {
        use super::entity::*;

        let primitive_types = [
            ("Строка", "String", "Строковый примитивный тип"),
            ("Число", "Number", "Числовой примитивный тип"),
            ("Булево", "Boolean", "Логический примитивный тип"),
            ("Дата", "Date", "Примитивный тип даты"),
            ("Неопределено", "Undefined", "Неопределенное значение"),
            ("NULL", "NULL", "Значение NULL"),
            ("Тип", "Type", "Тип данных"),
            ("Произвольный", "Arbitrary", "Произвольный тип данных"),
        ];

        for (russian_name, english_name, documentation) in &primitive_types {
            let display_name = format!("{} ({})", russian_name, english_name);
            let mut entity = BslEntity::new(
                display_name.clone(),
                display_name.clone(),
                BslEntityType::Platform,
                BslEntityKind::Primitive,
            );

            entity.english_name = Some(english_name.to_string());
            entity.documentation = Some(documentation.to_string());
            entity.availability = vec![BslContext::Server, BslContext::Client];
            entity.source = BslEntitySource::HBK {
                version: version.to_string(),
            };

            if *russian_name == "Строка" {
                self.add_string_methods(&mut entity);
            }

            entities.push(entity);
        }

        Ok(())
    }

    /// Добавляет основные методы для строкового типа
    fn add_string_methods(&self, entity: &mut BslEntity) {
        use super::entity::*;

        let string_methods = [
            ("Длина", "Length", "Number", "Возвращает длину строки"),
            (
                "ВРег",
                "Upper",
                "String",
                "Преобразует строку в верхний регистр",
            ),
            (
                "НРег",
                "Lower",
                "String",
                "Преобразует строку в нижний регистр",
            ),
            ("Лев", "Left", "String", "Возвращает левую часть строки"),
            ("Прав", "Right", "String", "Возвращает правую часть строки"),
            ("Сред", "Mid", "String", "Возвращает подстроку"),
            (
                "СокрЛП",
                "TrimAll",
                "String",
                "Удаляет пробелы слева и справа",
            ),
            ("Найти", "Find", "Number", "Поиск подстроки в строке"),
        ];

        for (method_name, english_name, return_type, doc) in &string_methods {
            let method = BslMethod {
                name: method_name.to_string(),
                english_name: Some(english_name.to_string()),
                parameters: vec![], // Упрощенно, без параметров
                return_type: Some(return_type.to_string()),
                documentation: Some(doc.to_string()),
                availability: vec![BslContext::Server, BslContext::Client],
                is_function: true,
                is_export: false,
                is_deprecated: false,
                deprecation_info: None,
            };
            entity
                .interface
                .methods
                .insert(method_name.to_string(), method);
        }
    }

    /// Конвертирует BslSyntaxDatabase в BslEntity типы
    fn convert_syntax_db_to_entities(
        &self,
        syntax_db: &crate::docs_integration::BslSyntaxDatabase,
        version: &str,
    ) -> Result<Vec<BslEntity>> {
        use super::converters::SyntaxDbConverter;

        let converter = SyntaxDbConverter::new(version);
        converter.convert(syntax_db)
    }
}
