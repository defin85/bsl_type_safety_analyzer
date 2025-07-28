/*! 
# Hybrid Storage for BSL Documentation and Metadata

Реализует архитектуру хранения документации в гибридном формате:
- Компактные группированные файлы для встроенных типов
- Разделение ядра и конфигурации
- Оптимизированные индексы для быстрого поиска
- Runtime кэширование
*/

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use chrono::{DateTime, Utc};
use super::chunked_writer::SyntaxItem;

/// Манифест системы документации
#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentationManifest {
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub bsl_version: String,
    pub platform_version: String,
    pub statistics: ManifestStatistics,
    pub components: Vec<ComponentInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ManifestStatistics {
    pub total_types: usize,
    pub builtin_types: usize,
    pub config_types: usize,
    pub total_methods: usize,
    pub total_properties: usize,
    pub total_size_mb: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentInfo {
    pub name: String,
    pub path: String,
    pub types_count: usize,
    pub size_kb: f64,
    pub checksum: String,
}

/// Определение типа в системе типов BSL
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TypeDefinition {
    pub id: String,
    pub name: String,
    pub english_name: Option<String>,
    pub category: TypeCategory,
    pub description: String,
    pub methods: HashMap<String, MethodDefinition>,
    pub properties: HashMap<String, PropertyDefinition>,
    pub constructors: Vec<ConstructorDefinition>,
    pub parent_types: Vec<String>,
    pub interfaces: Vec<String>,
    pub availability: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, Hash, PartialEq)]
pub enum TypeCategory {
    Primitive,
    Collection,
    System,
    Form,
    Database,
    IO,
    Web,
    Configuration,
    Reference,
    Object,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MethodDefinition {
    pub name: String,
    pub english_name: Option<String>,
    pub description: String,
    pub parameters: Vec<ParameterDefinition>,
    pub return_type: Option<String>,
    pub is_function: bool,
    pub availability: Vec<String>,
    pub examples: Vec<String>,
    pub deprecated: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PropertyDefinition {
    pub name: String,
    pub english_name: Option<String>,
    pub description: String,
    pub property_type: String,
    pub readonly: bool,
    pub availability: Vec<String>,
    pub deprecated: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ParameterDefinition {
    pub name: String,
    pub parameter_type: String,
    pub required: bool,
    pub description: String,
    pub default_value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConstructorDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ParameterDefinition>,
    pub availability: Vec<String>,
}

/// Менеджер гибридного хранилища документации
pub struct HybridDocumentationStorage {
    base_path: PathBuf,
    manifest: Option<DocumentationManifest>,
    type_cache: HashMap<String, TypeDefinition>,
    method_index: HashMap<String, Vec<String>>, // method_name -> [type_ids]
}

impl HybridDocumentationStorage {
    /// Создает новый менеджер хранилища
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
            manifest: None,
            type_cache: HashMap::new(),
            method_index: HashMap::new(),
        }
    }

    /// Загружает манифест
    pub fn load_manifest(&mut self) -> Result<()> {
        let manifest_path = self.base_path.join("manifest.json");
        if manifest_path.exists() {
            let content = fs::read_to_string(manifest_path)?;
            self.manifest = Some(serde_json::from_str(&content)?);
        }
        Ok(())
    }

    /// Инициализирует новую структуру документации
    pub fn initialize(&mut self) -> Result<()> {
        tracing::info!("Initializing hybrid documentation storage at: {}", self.base_path.display());
        
        // Создаем базовую структуру директорий
        self.create_directory_structure()?;
        
        // Создаем базовый манифест
        self.create_initial_manifest()?;
        
        Ok(())
    }

    /// Добавляет элемент синтаксиса в соответствующую категорию
    pub fn add_syntax_item(&mut self, item: SyntaxItem) -> Result<()> {
        // Определяем категорию и тип
        let type_id = self.determine_type_id(&item);
        
        // Создаем элементы заранее, чтобы избежать проблем с borrow checker
        let method_def_opt = if item.category == "methods" || item.category == "operators" {
            Some(self.create_method_definition(&item)?)
        } else {
            None
        };
        
        let prop_def_opt = if item.category == "properties" {
            Some(self.create_property_definition(&item)?)
        } else {
            None
        };
        
        let ctor_def_opt = if item.category == "functions" && self.is_constructor(&item) {
            Some(self.create_constructor_definition(&item)?)
        } else {
            None
        };
        
        // Получаем или создаем определение типа
        let type_def = self.get_or_create_type(&type_id, &item)?;
        
        // Добавляем элемент в соответствующую коллекцию
        match item.category.as_str() {
            "methods" => {
                if let Some(method_def) = method_def_opt {
                    type_def.methods.insert(method_def.name.clone(), method_def);
                }
            },
            "properties" => {
                if let Some(prop_def) = prop_def_opt {
                    type_def.properties.insert(prop_def.name.clone(), prop_def);
                }
            },
            "functions" => {
                // Функции могут быть конструкторами или глобальными
                if let Some(ctor_def) = ctor_def_opt {
                    type_def.constructors.push(ctor_def);
                } else {
                    // Глобальная функция - добавляем в глобальный контекст
                    self.add_global_function(&item)?;
                }
            },
            "operators" => {
                // Операторы добавляем как методы
                if let Some(method_def) = method_def_opt {
                    type_def.methods.insert(method_def.name.clone(), method_def);
                }
            },
            _ => {
                // Объекты - создаем определение типа
                self.create_object_type(&item)?;
            }
        }
        
        // Обновляем индекс методов
        self.update_method_index(&item);
        
        Ok(())
    }
    
    /// Завершает обработку и записывает все файлы
    pub fn finalize(&mut self) -> Result<()> {
        tracing::info!("Finalizing hybrid documentation storage");
        
        // Записываем сгруппированные типы
        self.write_builtin_types()?;
        self.write_global_context()?;
        
        // Создаем индекс форм
        self.create_forms_index()?;
        
        // Создаем оптимизированные индексы
        self.build_indices()?;
        
        // Обновляем манифест
        self.update_manifest()?;
        
        tracing::info!("Hybrid documentation storage finalized successfully");
        Ok(())
    }

    /// Получает определение типа по ID
    pub fn get_type(&mut self, type_id: &str) -> Result<Option<TypeDefinition>> {
        if let Some(cached) = self.type_cache.get(type_id) {
            return Ok(Some(cached.clone()));
        }

        // Пытаемся загрузить из файлов
        if let Some(type_def) = self.load_type_from_storage(type_id)? {
            self.type_cache.insert(type_id.to_string(), type_def.clone());
            return Ok(Some(type_def));
        }

        Ok(None)
    }

    /// Ищет методы по имени
    pub fn find_methods(&self, method_name: &str) -> Vec<String> {
        self.method_index.get(method_name).cloned().unwrap_or_default()
    }

    /// Создает структуру директорий
    fn create_directory_structure(&self) -> Result<()> {
        let dirs = [
            "core",
            "core/builtin_types", 
            "configuration",
            "configuration/metadata_types",
            "configuration/forms",
            "indices",
            "runtime",
        ];

        for dir in dirs {
            fs::create_dir_all(self.base_path.join(dir))?;
        }

        Ok(())
    }

    /// Создает начальный манифест
    fn create_initial_manifest(&mut self) -> Result<()> {
        let manifest = DocumentationManifest {
            version: "1.0.0".to_string(),
            created_at: Utc::now(),
            bsl_version: "8.3.22".to_string(),
            platform_version: "8.3.22.1923".to_string(),
            statistics: ManifestStatistics {
                total_types: 0,
                builtin_types: 0,
                config_types: 0,
                total_methods: 0,
                total_properties: 0,
                total_size_mb: 0.0,
            },
            components: Vec::new(),
        };

        let manifest_path = self.base_path.join("manifest.json");
        let json = serde_json::to_string_pretty(&manifest)?;
        fs::write(manifest_path, json)?;

        self.manifest = Some(manifest);
        Ok(())
    }

    /// Конвертирует встроенные типы из chunked формата
    #[allow(dead_code)]
    fn convert_builtin_types<P: AsRef<Path>>(&self, _chunked_path: P) -> Result<()> {
        // TODO: Реализовать конвертацию
        tracing::info!("Converting builtin types...");
        Ok(())
    }

    /// Конвертирует глобальный контекст
    #[allow(dead_code)]
    fn convert_global_context<P: AsRef<Path>>(&self, _chunked_path: P) -> Result<()> {
        // TODO: Реализовать конвертацию
        tracing::info!("Converting global context...");
        Ok(())
    }

    /// Строит оптимизированные индексы
    fn build_indices(&mut self) -> Result<()> {
        // TODO: Реализовать построение индексов
        tracing::info!("Building optimized indices...");
        Ok(())
    }

    /// Обновляет манифест статистикой
    fn update_manifest(&mut self) -> Result<()> {
        // TODO: Реализовать обновление манифеста
        tracing::info!("Updating manifest...");
        Ok(())
    }

    /// Загружает тип из хранилища
    fn load_type_from_storage(&self, _type_id: &str) -> Result<Option<TypeDefinition>> {
        // TODO: Реализовать загрузку из файлов
        Ok(None)
    }

    // ========== Вспомогательные методы для обработки элементов ==========

    /// Определяет ID типа по элементу синтаксиса
    fn determine_type_id(&self, item: &SyntaxItem) -> String {
        if let Some(dot_pos) = item.title.find('.') {
            // Это метод или свойство объекта - извлекаем имя объекта
            item.title[..dot_pos].to_string()
        } else if item.category == "functions" && item.title.contains("(") {
            // Это функция - может быть конструктором или глобальной
            if let Some(paren_pos) = item.title.find(" (") {
                item.title[..paren_pos].to_string()
            } else {
                item.title.clone()
            }
        } else {
            // Это объект или другой элемент
            item.title.clone()
        }
    }

    /// Определяет категорию типа
    fn determine_category(&self, item: &SyntaxItem) -> TypeCategory {
        let type_name = self.determine_type_id(item);
        
        match type_name.as_str() {
            // Примитивы
            "Число" | "Строка" | "Булево" | "Дата" | "Неопределено" | "Null" => TypeCategory::Primitive,
            
            // Коллекции
            "Массив" | "Соответствие" | "СписокЗначений" | "ФиксированныйМассив" | "ФиксированноеСоответствие" => TypeCategory::Collection,
            
            // Таблицы
            "ТаблицаЗначений" | "ДеревоЗначений" => TypeCategory::Collection,
            
            // Системные объекты
            "СистемнаяИнформация" | "ИнформацияОПользователе" | "Метаданные" => TypeCategory::System,
            
            // Ввод/вывод
            "ЧтениеТекста" | "ЗаписьТекста" | "ЧтениеXML" | "ЗаписьXML" | "ЧтениеJSON" | "ЗаписьJSON" => TypeCategory::IO,
            
            // Формы
            "УправляемаяФорма" | "ЭлементыФормы" | "ТаблицаФормы" | "ПолеФормы" => TypeCategory::Form,
            
            // Web
            "HTTPСоединение" | "HTTPЗапрос" | "HTTPОтвет" => TypeCategory::Web,
            
            // База данных
            "Запрос" | "РезультатЗапроса" | "ВыборкаИзРезультатаЗапроса" => TypeCategory::Database,
            
            _ => TypeCategory::System, // По умолчанию
        }
    }

    /// Получает или создает определение типа
    fn get_or_create_type(&mut self, type_id: &str, item: &SyntaxItem) -> Result<&mut TypeDefinition> {
        if !self.type_cache.contains_key(type_id) {
            let type_def = TypeDefinition {
                id: type_id.to_string(),
                name: type_id.to_string(),
                english_name: self.extract_english_name(&item.title),
                category: self.determine_category(item),
                description: item.content.clone(),
                methods: HashMap::new(),
                properties: HashMap::new(),
                constructors: Vec::new(),
                parent_types: Vec::new(),
                interfaces: Vec::new(),
                availability: item.metadata.availability.clone(),
            };
            self.type_cache.insert(type_id.to_string(), type_def);
        }
        
        Ok(self.type_cache.get_mut(type_id).unwrap())
    }

    /// Извлекает английское название из заголовка
    fn extract_english_name(&self, title: &str) -> Option<String> {
        if let Some(start) = title.find(" (") {
            if let Some(end) = title.find(")") {
                let eng_part = &title[start + 2..end];
                if let Some(dot_pos) = eng_part.find('.') {
                    return Some(eng_part[..dot_pos].to_string());
                } else {
                    return Some(eng_part.to_string());
                }
            }
        }
        None
    }

    /// Создает определение метода
    fn create_method_definition(&self, item: &SyntaxItem) -> Result<MethodDefinition> {
        let method_name = if let Some(dot_pos) = item.title.find('.') {
            let full_method = &item.title[dot_pos + 1..];
            if let Some(paren_pos) = full_method.find(" (") {
                full_method[..paren_pos].to_string()
            } else {
                full_method.to_string()
            }
        } else {
            item.title.clone()
        };

        Ok(MethodDefinition {
            name: method_name.clone(),
            english_name: self.extract_english_name(&item.title),
            description: item.content.clone(),
            parameters: item.metadata.parameters.iter().map(|p| ParameterDefinition {
                name: p.clone(),
                parameter_type: "any".to_string(), // TODO: парсить тип из синтаксиса
                required: true, // TODO: определять из синтаксиса
                description: "".to_string(),
                default_value: None,
            }).collect(),
            return_type: if item.metadata.return_value.is_empty() { 
                None 
            } else { 
                Some(item.metadata.return_value.clone()) 
            },
            is_function: !item.metadata.return_value.is_empty(),
            availability: item.metadata.availability.clone(),
            examples: if item.metadata.example.is_empty() { 
                Vec::new() 
            } else { 
                vec![item.metadata.example.clone()] 
            },
            deprecated: false,
        })
    }

    /// Создает определение свойства
    fn create_property_definition(&self, item: &SyntaxItem) -> Result<PropertyDefinition> {
        let prop_name = if let Some(dot_pos) = item.title.find('.') {
            item.title[dot_pos + 1..].to_string()
        } else {
            item.title.clone()
        };

        Ok(PropertyDefinition {
            name: prop_name,
            english_name: self.extract_english_name(&item.title),
            description: item.content.clone(),
            property_type: "any".to_string(), // TODO: определять тип
            readonly: false, // TODO: определять из документации
            availability: item.metadata.availability.clone(),
            deprecated: false,
        })
    }

    /// Создает определение конструктора
    fn create_constructor_definition(&self, item: &SyntaxItem) -> Result<ConstructorDefinition> {
        Ok(ConstructorDefinition {
            name: item.title.clone(),
            description: item.content.clone(),
            parameters: item.metadata.parameters.iter().map(|p| ParameterDefinition {
                name: p.clone(),
                parameter_type: "any".to_string(),
                required: true,
                description: "".to_string(),
                default_value: None,
            }).collect(),
            availability: item.metadata.availability.clone(),
        })
    }

    /// Проверяет, является ли элемент конструктором
    fn is_constructor(&self, item: &SyntaxItem) -> bool {
        item.title.contains("Новый ") || item.title.contains("new ") || 
        item.title.contains("Формирование неинициализированного объекта")
    }

    /// Добавляет глобальную функцию
    fn add_global_function(&mut self, _item: &SyntaxItem) -> Result<()> {
        // TODO: Реализовать добавление глобальных функций
        Ok(())
    }

    /// Создает определение типа объекта
    fn create_object_type(&mut self, _item: &SyntaxItem) -> Result<()> {
        // TODO: Реализовать создание объектного типа
        Ok(())
    }

    /// Обновляет индекс методов
    fn update_method_index(&mut self, item: &SyntaxItem) {
        if item.category == "methods" || item.category == "operators" {
            let method_name = if let Some(dot_pos) = item.title.find('.') {
                let full_method = &item.title[dot_pos + 1..];
                if let Some(paren_pos) = full_method.find(" (") {
                    full_method[..paren_pos].to_string()
                } else {
                    full_method.to_string()
                }
            } else {
                item.title.clone()
            };

            let type_id = self.determine_type_id(item);
            self.method_index
                .entry(method_name)
                .or_insert_with(Vec::new)
                .push(type_id);
        }
    }

    /// Записывает встроенные типы
    fn write_builtin_types(&self) -> Result<()> {
        tracing::info!("Writing builtin types...");
        
        let mut builtin_groups: HashMap<TypeCategory, Vec<&TypeDefinition>> = HashMap::new();
        let mut config_groups: HashMap<String, Vec<&TypeDefinition>> = HashMap::new();
        
        for type_def in self.type_cache.values() {
            if type_def.category == TypeCategory::Configuration {
                // Группируем конфигурационные типы по типу объекта
                let object_type = type_def.id.split('.').next().unwrap_or("unknown");
                config_groups.entry(object_type.to_string())
                    .or_insert_with(Vec::new)
                    .push(type_def);
            } else {
                // Группируем встроенные типы по категории
                builtin_groups.entry(type_def.category.clone())
                    .or_insert_with(Vec::new)
                    .push(type_def);
            }
        }
        
        // Записываем встроенные типы
        for (category, types) in builtin_groups {
            let filename = format!("{}.json", self.category_to_filename(&category));
            let filepath = self.base_path.join("core/builtin_types").join(filename);
            
            let json = serde_json::to_string_pretty(&types)?;
            fs::write(filepath, json)?;
            
            tracing::info!("Written {} builtin types for category {:?}", types.len(), category);
        }
        
        // Записываем конфигурационные типы
        for (object_type, types) in config_groups {
            let filename = format!("{}.json", object_type.to_lowercase());
            let filepath = self.base_path.join("configuration/metadata_types").join(filename);
            
            // Создаем директорию если не существует
            fs::create_dir_all(self.base_path.join("configuration/metadata_types"))?;
            
            let json = serde_json::to_string_pretty(&types)?;
            fs::write(filepath, json)?;
            
            tracing::info!("Written {} configuration types for {}", types.len(), object_type);
        }
        
        Ok(())
    }

    /// Записывает глобальный контекст
    fn write_global_context(&self) -> Result<()> {
        tracing::info!("Writing global context...");
        
        // Создаем структуру глобального контекста
        let global_context = serde_json::json!({
            "version": "1.0.0",
            "platform": "8.3.22",
            "total_types": self.type_cache.len(),
            "method_index": self.method_index,
            "categories": self.get_category_statistics()
        });
        
        let filepath = self.base_path.join("core/global_context.json");
        let json = serde_json::to_string_pretty(&global_context)?;
        fs::write(filepath, json)?;
        
        tracing::info!("Written global context with {} types", self.type_cache.len());
        
        Ok(())
    }

    /// Конвертирует категорию типа в имя файла
    fn category_to_filename(&self, category: &TypeCategory) -> String {
        match category {
            TypeCategory::Primitive => "primitives",
            TypeCategory::Collection => "collections", 
            TypeCategory::System => "system",
            TypeCategory::Form => "forms",
            TypeCategory::Database => "database",
            TypeCategory::IO => "io",
            TypeCategory::Web => "web",
            TypeCategory::Configuration => "configuration",
            TypeCategory::Reference => "references",
            TypeCategory::Object => "objects",
        }.to_string()
    }

    /// Получает статистику по категориям
    fn get_category_statistics(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        
        for type_def in self.type_cache.values() {
            let category_name = self.category_to_filename(&type_def.category);
            *stats.entry(category_name).or_insert(0) += 1;
        }
        
        stats
    }
    
    /// Добавляет тип из конфигурации
    pub fn add_configuration_type(&mut self, type_def: TypeDefinition) -> Result<()> {
        let type_id = type_def.id.clone();
        
        // Добавляем в кэш
        self.type_cache.insert(type_id.clone(), type_def);
        
        // Обновляем индекс методов для типа конфигурации
        if let Some(type_def) = self.type_cache.get(&type_id) {
            for method_name in type_def.methods.keys() {
                self.method_index
                    .entry(method_name.clone())
                    .or_insert_with(Vec::new)
                    .push(type_id.clone());
            }
        }
        
        Ok(())
    }

    /// Добавляет форму в оптимизированное хранилище
    pub fn add_form_optimized(&mut self, form_contract: &super::super::configuration::form_parser::FormContract) -> Result<()> {
        // Определяем тип объекта и имя на основе типа формы и object_name
        let (object_type, object_name) = if let Some(ref obj_name) = form_contract.object_name {
            // Если есть object_name, используем его для определения типа
            ("objects", obj_name.as_str())
        } else {
            // Иначе используем тип формы
            match form_contract.form_type {
                super::super::configuration::form_parser::FormType::CommonForm => ("common", "forms"),
                super::super::configuration::form_parser::FormType::ReportForm => ("reports", "forms"),
                super::super::configuration::form_parser::FormType::DataProcessorForm => ("dataprocessors", "forms"),
                _ => ("unknown", "forms")
            }
        };
        
        // Создаем директорию
        let form_dir = self.base_path
            .join("configuration")
            .join("forms")
            .join(object_type)
            .join(object_name);
        
        fs::create_dir_all(&form_dir)?;
        
        // Создаем файл формы
        let form_file = form_dir.join(format!("{}.json", form_contract.name));
        let json = serde_json::to_string_pretty(form_contract)?;
        fs::write(form_file, json)?;
        
        tracing::debug!("Saved form {} to optimized storage", form_contract.name);
        Ok(())
    }

    /// Очищает хранилище перед новым парсингом
    pub fn clear_storage(&self) -> Result<()> {
        tracing::info!("Clearing old storage data");
        
        let paths_to_clear = [
            "configuration/metadata_types",
            "configuration/forms", 
            "indices",
            "runtime"
        ];
        
        for path in paths_to_clear {
            let full_path = self.base_path.join(path);
            if full_path.exists() {
                fs::remove_dir_all(&full_path)?;
                tracing::debug!("Cleared directory: {}", path);
            }
        }
        
        // Пересоздаем структуру директорий
        self.create_directory_structure()?;
        
        tracing::info!("Storage cleared successfully");
        Ok(())
    }

    /// Очищает только формы, сохраняя метаданные
    pub fn clear_forms_only(&self) -> Result<()> {
        tracing::info!("Clearing only forms data");
        
        let paths_to_clear = [
            "configuration/forms"
        ];
        
        for path in paths_to_clear {
            let full_path = self.base_path.join(path);
            if full_path.exists() {
                fs::remove_dir_all(&full_path)?;
                tracing::debug!("Cleared directory: {}", path);
            }
        }
        
        // Пересоздаем только папку forms
        fs::create_dir_all(self.base_path.join("configuration").join("forms"))?;
        
        tracing::info!("Forms storage cleared successfully");
        Ok(())
    }

    /// Очищает только metadata_types, сохраняя формы (для MetadataReportParser)
    pub fn clear_metadata_types_only(&self) -> Result<()> {
        tracing::info!("Clearing only metadata_types data");
        
        let metadata_types_path = self.base_path.join("configuration").join("metadata_types");
        if metadata_types_path.exists() {
            fs::remove_dir_all(&metadata_types_path)?;
            tracing::debug!("Cleared directory: configuration/metadata_types");
        }
        
        // Пересоздаем папку metadata_types
        fs::create_dir_all(&metadata_types_path)?;
        
        tracing::info!("Metadata types storage cleared successfully (forms preserved)");
        Ok(())
    }

    /// Создает индекс форм для быстрого поиска
    pub fn create_forms_index(&self) -> Result<()> {
        let forms_dir = self.base_path.join("configuration").join("forms");
        let index_file = forms_dir.join("index.json");
        
        if !forms_dir.exists() {
            return Ok(()); // Нет форм для индексирования
        }
        
        let mut forms_index = std::collections::HashMap::new();
        
        // Рекурсивно обходим все файлы форм
        self.collect_forms_for_index(&forms_dir, &mut forms_index)?;
        
        // Записываем индекс
        let json = serde_json::to_string_pretty(&forms_index)?;
        fs::write(index_file, json)?;
        
        tracing::info!("Created forms index with {} entries", forms_index.len());
        Ok(())
    }

    /// Собирает информацию о формах для индекса
    fn collect_forms_for_index(
        &self, 
        dir: &Path, 
        index: &mut std::collections::HashMap<String, serde_json::Value>
    ) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                self.collect_forms_for_index(&path, index)?;
            } else if path.extension().map_or(false, |ext| ext == "json") && path.file_name() != Some(std::ffi::OsStr::new("index.json")) {
                // Читаем файл формы и добавляем в индекс
                let content = fs::read_to_string(&path)?;
                if let Ok(form_data) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(form_name) = form_data.get("name").and_then(|v| v.as_str()) {
                        let relative_path = path.strip_prefix(&self.base_path.join("configuration").join("forms"))
                            .unwrap_or(&path)
                            .to_string_lossy()
                            .to_string();
                        
                        // Создаем составной ключ для уникальности
                        let index_key = if let Some(obj_name) = form_data.get("object_name").and_then(|v| v.as_str()) {
                            format!("{}.{}", obj_name, form_name)
                        } else {
                            form_name.to_string()
                        };
                        
                        index.insert(index_key, serde_json::json!({
                            "path": relative_path,
                            "name": form_name,
                            "form_type": form_data.get("form_type").unwrap_or(&serde_json::Value::Null),
                            "object_name": form_data.get("object_name").unwrap_or(&serde_json::Value::Null),
                            "metadata_type": form_data.get("metadata_type").unwrap_or(&serde_json::Value::Null)
                        }));
                    }
                }
            }
        }
        
        Ok(())
    }
}