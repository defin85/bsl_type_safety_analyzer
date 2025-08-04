use std::path::PathBuf;
use anyhow::{Result, Context};
use clap::Parser;
use bsl_analyzer::unified_index::PlatformDocsCache;
use bsl_analyzer::docs_integration::BslSyntaxExtractor;

#[derive(Parser, Debug)]
#[command(author, about = "Extract platform types from 1C documentation archive", long_about = None)]
#[command(disable_version_flag = true)]
struct Args {
    /// Path to 1C documentation archive (e.g., rebuilt.shcntx_ru.zip)
    #[arg(short, long)]
    archive: PathBuf,
    
    /// Platform version (e.g., "8.3.25")
    #[arg(short = 'p', long = "platform-version")]
    version: String,
    
    /// Force re-extraction even if cache exists
    #[arg(short, long)]
    force: bool,
    
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Initialize logging - keep INFO level even for verbose to avoid spam
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    // Validate archive path
    if !args.archive.exists() {
        return Err(anyhow::anyhow!(
            "Archive file does not exist: {}",
            args.archive.display()
        ));
    }
    
    println!("Extracting platform documentation...");
    println!("Archive: {}", args.archive.display());
    println!("Version: {}", args.version);
    
    // Check if already cached
    let cache = PlatformDocsCache::new()?;
    let cache_file = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
        .join(".bsl_analyzer")
        .join("platform_cache")
        .join(format!("{}.jsonl", args.version));
    
    if cache_file.exists() && !args.force {
        println!("\n⚠️  Platform types for version {} already cached at:", args.version);
        println!("   {}", cache_file.display());
        println!("\n   Use --force to re-extract");
        return Ok(());
    }
    
    // Extract documentation
    println!("\nExtracting BSL syntax from documentation...");
    let start = std::time::Instant::now();
    
    let mut extractor = BslSyntaxExtractor::new(&args.archive);
    let syntax_db = extractor.extract_syntax_database(None)
        .context("Failed to extract BSL syntax")?;
    
    println!("Extracted in {:.2?}", start.elapsed());
    println!("\nExtracted types:");
    println!("  Objects: {}", syntax_db.objects.len());
    println!("  Methods: {}", syntax_db.methods.len());
    println!("  Properties: {}", syntax_db.properties.len());
    println!("  Functions: {}", syntax_db.functions.len());
    println!("  Operators: {}", syntax_db.operators.len());
    
    // Convert to BslEntity format and save to cache
    println!("\nConverting to unified format...");
    // Нормализуем версию - убираем префикс "v" если есть для консистентности
    let normalized_version = args.version.strip_prefix("v").unwrap_or(&args.version);
    let entities = convert_syntax_db_to_entities(&syntax_db, normalized_version)?;
    
    println!("Converted {} entities", entities.len());
    
    // Save to cache
    println!("\nSaving to platform cache...");
    cache.save_to_cache(normalized_version, &entities)
        .context("Failed to save to cache")?;
    
    println!("✅ Platform types cached at: {}", cache_file.display());
    
    // Show summary for verbose mode
    if args.verbose {
        println!("\n=== Type categories ===");
        let mut categories: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for entity in &entities {
            *categories.entry(format!("{:?}", entity.entity_kind)).or_insert(0) += 1;
        }
        for (category, count) in categories {
            println!("  {}: {}", category, count);
        }
    }
    
    Ok(())
}

fn convert_syntax_db_to_entities(
    syntax_db: &bsl_analyzer::docs_integration::BslSyntaxDatabase,
    version: &str
) -> Result<Vec<bsl_analyzer::unified_index::BslEntity>> {
    use bsl_analyzer::unified_index::*;
    use std::collections::HashMap;
    
    let mut entities = Vec::new();
    let mut entity_map: HashMap<String, BslEntity> = HashMap::new();
    // Дополнительный маппинг русских имен к полным именам для связывания методов
    let mut russian_name_map: HashMap<String, String> = HashMap::new();
    
    // First, create entities for all objects
    for (name, obj) in &syntax_db.objects {
        let mut entity = BslEntity::new(
            name.clone(),
            name.clone(),
            BslEntityType::Platform,
            determine_entity_kind(name)
        );
        
        entity.source = BslEntitySource::HBK { version: version.to_string() };
        entity.documentation = obj.description.clone();
        
        // Convert availability
        if let Some(availability_str) = &obj.availability {
            entity.availability = availability_str
                .split(',')
                .filter_map(|ctx| parse_context(ctx.trim()))
                .collect();
        }
        
        // Add properties from object
        for prop_name in &obj.properties {
            // Пытаемся определить тип свойства по имени
            let type_name = match prop_name.as_str() {
                "Колонки" | "Columns" => Some("КоллекцияКолонокТаблицыЗначений"),
                "Индексы" | "Indexes" => Some("ИндексыКоллекция"),
                _ => None,
            };
            
            let bsl_property = BslProperty {
                name: prop_name.clone(),
                english_name: None,
                type_name: type_name.map(String::from).unwrap_or_default(),
                is_readonly: false, // По умолчанию, можно уточнить
                is_indexed: false,
                documentation: None,
                availability: entity.availability.clone(),
            };
            entity.interface.properties.insert(prop_name.clone(), bsl_property);
        }
        
        entity_map.insert(name.clone(), entity);
        
        // Добавляем маппинг русского имени к полному имени
        if let Some(pos) = name.find(" (") {
            let russian_name = name[..pos].to_string();
            russian_name_map.insert(russian_name, name.clone());
        }
    }
    
    // Add methods to their respective objects
    for (method_name, method_info) in &syntax_db.methods {
        // Find which object this method belongs to
        if let Some(object_name) = &method_info.object_context {
            // Сначала пытаемся найти по точному имени
            let entity_key = if entity_map.contains_key(object_name) {
                Some(object_name.clone())
            } else if let Some(full_name) = russian_name_map.get(object_name) {
                // Если не нашли, пробуем через маппинг русских имен
                Some(full_name.clone())
            } else {
                // В крайнем случае ищем по префиксу
                entity_map.keys()
                    .find(|key| key.starts_with(object_name))
                    .cloned()
            };
            
            if let Some(key) = entity_key {
                if let Some(entity) = entity_map.get_mut(&key) {
                    let bsl_method = convert_method_info_to_bsl_method(method_info);
                    entity.interface.methods.insert(method_name.clone(), bsl_method);
                }
            }
        } else {
            // Global method - create a special "Global" entity if needed
            let global_entity = entity_map.entry("Global".to_string()).or_insert_with(|| {
                let mut entity = BslEntity::new(
                    "Global".to_string(),
                    "Global".to_string(),
                    BslEntityType::Platform,
                    BslEntityKind::Global
                );
                entity.source = BslEntitySource::HBK { version: version.to_string() };
                entity
            });
            
            let bsl_method = convert_method_info_to_bsl_method(method_info);
            global_entity.interface.methods.insert(method_name.clone(), bsl_method);
        }
    }
    
    // Add properties to their respective objects
    for (prop_name, prop_info) in &syntax_db.properties {
        if let Some(object_name) = &prop_info.object_context {
            // Используем ту же логику поиска, что и для методов
            let entity_key = if entity_map.contains_key(object_name) {
                Some(object_name.clone())
            } else if let Some(full_name) = russian_name_map.get(object_name) {
                Some(full_name.clone())
            } else {
                entity_map.keys()
                    .find(|key| key.starts_with(object_name))
                    .cloned()
            };
            
            if let Some(key) = entity_key {
                if let Some(entity) = entity_map.get_mut(&key) {
                    let bsl_property = convert_property_info_to_bsl_property(prop_info);
                    entity.interface.properties.insert(prop_name.clone(), bsl_property);
                }
            }
        }
    }
    
    // Convert functions to methods of Global entity
    for (func_name, func_info) in &syntax_db.functions {
        let global_entity = entity_map.entry("Global".to_string()).or_insert_with(|| {
            let mut entity = BslEntity::new(
                "Global".to_string(),
                "Global".to_string(),
                BslEntityType::Platform,
                BslEntityKind::Global
            );
            entity.source = BslEntitySource::HBK { version: version.to_string() };
            entity
        });
        
        let bsl_method = convert_function_info_to_bsl_method(func_info);
        global_entity.interface.methods.insert(func_name.clone(), bsl_method);
    }
    
    // Create primitive types from keywords
    create_primitive_types_from_keywords(&syntax_db.keywords, &mut entity_map, version);
    
    // Convert all entities to vector
    entities.extend(entity_map.into_values());
    
    Ok(entities)
}

fn convert_method_info_to_bsl_method(method_info: &bsl_analyzer::docs_integration::BslMethodInfo) -> bsl_analyzer::unified_index::BslMethod {
    use bsl_analyzer::unified_index::*;
    
    BslMethod {
        name: method_info.name.clone(),
        english_name: method_info.english_name.clone(),
        parameters: method_info.parameters.iter()
            .map(|p| BslParameter {
                name: p.name.clone(),
                type_name: p.param_type.clone(),
                is_optional: p.is_optional,
                default_value: p.default_value.clone(),
                description: p.description.clone(),
            })
            .collect(),
        return_type: method_info.return_type.clone(),
        documentation: method_info.description.clone(),
        availability: method_info.availability.iter()
            .filter_map(|ctx| parse_context(ctx))
            .collect(),
        is_function: method_info.return_type.is_some(),
        is_export: false,
        is_deprecated: false,
        deprecation_info: None,
    }
}

fn convert_property_info_to_bsl_property(prop_info: &bsl_analyzer::docs_integration::BslPropertyInfo) -> bsl_analyzer::unified_index::BslProperty {
    use bsl_analyzer::unified_index::*;
    
    BslProperty {
        name: prop_info.name.clone(),
        english_name: None,
        type_name: prop_info.property_type.clone(),
        is_readonly: matches!(prop_info.access_mode, bsl_analyzer::docs_integration::AccessMode::Read),
        is_indexed: false,
        documentation: prop_info.description.clone(),
        availability: if let Some(availability_str) = &prop_info.availability {
            availability_str
                .split(',')
                .filter_map(|ctx| parse_context(ctx.trim()))
                .collect()
        } else {
            vec![]
        },
    }
}

fn convert_function_info_to_bsl_method(func_info: &bsl_analyzer::docs_integration::BslFunctionInfo) -> bsl_analyzer::unified_index::BslMethod {
    use bsl_analyzer::unified_index::*;
    
    BslMethod {
        name: func_info.name.clone(),
        english_name: None,
        parameters: func_info.parameters.iter()
            .map(|p| BslParameter {
                name: p.name.clone(),
                type_name: p.param_type.clone(),
                is_optional: p.is_optional,
                default_value: p.default_value.clone(),
                description: p.description.clone(),
            })
            .collect(),
        return_type: func_info.return_type.clone(),
        documentation: func_info.description.clone(),
        availability: if let Some(availability_str) = &func_info.availability {
            availability_str
                .split(',')
                .filter_map(|ctx| parse_context(ctx.trim()))
                .collect()
        } else {
            vec![]
        },
        is_function: true,
        is_export: false,
        is_deprecated: false,
        deprecation_info: None,
    }
}

fn determine_entity_kind(name: &str) -> bsl_analyzer::unified_index::BslEntityKind {
    use bsl_analyzer::unified_index::BslEntityKind;
    
    match name {
        "Массив" | "Array" => BslEntityKind::Array,
        "Структура" | "Structure" => BslEntityKind::Structure,
        "Соответствие" | "Map" => BslEntityKind::Map,
        "СписокЗначений" | "ValueList" => BslEntityKind::ValueList,
        "ТаблицаЗначений" | "ValueTable" => BslEntityKind::ValueTable,
        "ДеревоЗначений" | "ValueTree" => BslEntityKind::ValueTree,
        "Число" | "Number" | "Строка" | "String" | "Булево" | "Boolean" | "Дата" | "Date" => BslEntityKind::Primitive,
        _ => BslEntityKind::System,
    }
}

fn parse_context(context_str: &str) -> Option<bsl_analyzer::unified_index::BslContext> {
    use bsl_analyzer::unified_index::BslContext;
    
    match context_str {
        "Client" | "Клиент" => Some(BslContext::Client),
        "Server" | "Сервер" => Some(BslContext::Server),
        "ExternalConnection" | "ВнешнееСоединение" => Some(BslContext::ExternalConnection),
        "MobileApp" | "МобильноеПриложение" => Some(BslContext::MobileApp),
        "MobileClient" | "МобильныйКлиент" => Some(BslContext::MobileClient),
        "MobileServer" | "МобильныйСервер" => Some(BslContext::MobileServer),
        "ThickClient" | "ТолстыйКлиент" => Some(BslContext::ThickClient),
        "ThinClient" | "ТонкийКлиент" => Some(BslContext::ThinClient),
        "WebClient" | "ВебКлиент" => Some(BslContext::WebClient),
        _ => None,
    }
}

/// Создает примитивные типы BSL из keywords
fn create_primitive_types_from_keywords(
    keywords: &[String],
    entity_map: &mut std::collections::HashMap<String, bsl_analyzer::unified_index::BslEntity>,
    version: &str
) {
    use bsl_analyzer::unified_index::*;
    
    // Определяем какие keywords являются примитивными типами
    let primitive_types = [
        ("Число", "Number", "Числовой примитивный тип"),
        ("Строка", "String", "Строковый примитивный тип"),
        ("Дата", "Date", "Примитивный тип даты"),
        ("Булево", "Boolean", "Логический примитивный тип"),
        ("Неопределено", "Undefined", "Неопределенное значение"),
        ("NULL", "NULL", "Значение NULL"),
        ("Тип", "Type", "Тип данных"),
        ("Истина", "True", "Логическое значение Истина"),
        ("Ложь", "False", "Логическое значение Ложь"),
    ];
    
    for (russian_name, english_name, documentation) in &primitive_types {
        // Проверяем, есть ли это ключевое слово в списке keywords
        if keywords.contains(&russian_name.to_string()) {
            let display_name = format!("{} ({})", russian_name, english_name);
            let mut entity = BslEntity::new(
                display_name.clone(),
                display_name.clone(),
                BslEntityType::Platform,
                BslEntityKind::Primitive
            );
            
            entity.english_name = Some(english_name.to_string());
            entity.documentation = Some(documentation.to_string());
            entity.availability = vec![BslContext::Server, BslContext::Client];
            entity.source = BslEntitySource::HBK { version: version.to_string() };
            
            // Добавляем методы для строкового типа
            if *russian_name == "Строка" {
                add_string_methods(&mut entity);
            }
            
            // Также создаем версии только с русским и английским именами для удобного поиска
            entity_map.insert(display_name.clone(), entity.clone());
            entity_map.insert(russian_name.to_string(), entity.clone());
            entity_map.insert(english_name.to_string(), entity);
        }
    }
}

/// Добавляет основные методы для строкового типа
fn add_string_methods(entity: &mut bsl_analyzer::unified_index::BslEntity) {
    use bsl_analyzer::unified_index::*;
    
    let string_methods = [
        ("Длина", "Length", "Number", "Возвращает длину строки"),
        ("ВРег", "Upper", "String", "Преобразует строку в верхний регистр"),
        ("НРег", "Lower", "String", "Преобразует строку в нижний регистр"),
        ("Лев", "Left", "String", "Возвращает левую часть строки"),
        ("Прав", "Right", "String", "Возвращает правую часть строки"),
        ("Сред", "Mid", "String", "Возвращает подстроку"),
        ("СокрЛП", "TrimAll", "String", "Удаляет пробелы слева и справа"),
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
        
        entity.interface.methods.insert(method_name.to_string(), method);
    }
}