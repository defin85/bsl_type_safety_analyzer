use std::collections::HashMap;
use std::fs;
use serde::{Deserialize, Serialize};
use serde_json;
use anyhow::Result;

#[derive(Debug, Deserialize)]
struct MainIndex {
    item_index: HashMap<String, ItemIndexEntry>,
}

#[derive(Debug, Deserialize)]
struct ItemIndexEntry {
    category: String,
    _file: String,
    title: String,
    object_name: String,
}

#[derive(Debug, Serialize)]
struct ObjectSummary {
    total_unique_objects: usize,
    objects_by_category: HashMap<String, Vec<String>>,
    objects_list: Vec<ObjectInfo>,
}

#[derive(Debug, Serialize, Clone)]
struct ObjectInfo {
    name: String,
    methods_count: usize,
    properties_count: usize,
    functions_count: usize,
    operators_count: usize,
    description: String,
    english_name: Option<String>,
}

fn main() -> Result<()> {
    println!("Анализ объектов BSL из main_index.json...");
    
    // Читаем main_index.json
    let content = fs::read_to_string("output/docs_search/main_index.json")?;
    let index: MainIndex = serde_json::from_str(&content)?;
    
    // Собираем статистику по объектам
    let mut object_stats: HashMap<String, ObjectStats> = HashMap::new();
    
    for (_, entry) in &index.item_index {
        let stats = object_stats.entry(entry.object_name.clone()).or_insert(ObjectStats::default());
        
        match entry.category.as_str() {
            "methods" => stats.methods_count += 1,
            "properties" => stats.properties_count += 1,
            "functions" => stats.functions_count += 1,
            "operators" => stats.operators_count += 1,
            _ => {}
        }
        
        // Извлекаем английское название из заголовка
        if let Some(eng_name) = extract_english_name(&entry.title) {
            stats.english_name = Some(eng_name);
        }
    }
    
    // Группируем объекты по категориям
    let categories = categorize_objects(&object_stats);
    
    // Создаем список объектов с описаниями
    let mut objects_list: Vec<ObjectInfo> = object_stats
        .into_iter()
        .map(|(name, stats)| ObjectInfo {
            name: name.clone(),
            methods_count: stats.methods_count,
            properties_count: stats.properties_count,
            functions_count: stats.functions_count,
            operators_count: stats.operators_count,
            description: get_object_description(&name),
            english_name: stats.english_name,
        })
        .collect();
    
    // Сортируем по имени
    objects_list.sort_by(|a, b| a.name.cmp(&b.name));
    
    let summary = ObjectSummary {
        total_unique_objects: objects_list.len(),
        objects_by_category: categories,
        objects_list,
    };
    
    // Сохраняем результат
    let json = serde_json::to_string_pretty(&summary)?;
    fs::write("output/docs_search/objects_summary.json", json)?;
    
    println!("✓ Найдено {} уникальных объектов", summary.total_unique_objects);
    println!("✓ Результат сохранен в objects_summary.json");
    
    // Выводим топ-10 объектов по количеству методов
    let mut top_objects = summary.objects_list.clone();
    top_objects.sort_by(|a, b| b.methods_count.cmp(&a.methods_count));
    
    println!("\nТоп-10 объектов по количеству методов:");
    for (i, obj) in top_objects.iter().take(10).enumerate() {
        println!("{}. {} - {} методов", i + 1, obj.name, obj.methods_count);
    }
    
    Ok(())
}

#[derive(Default)]
struct ObjectStats {
    methods_count: usize,
    properties_count: usize,
    functions_count: usize,
    operators_count: usize,
    english_name: Option<String>,
}

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

fn get_object_description(name: &str) -> String {
    match name {
        // Коллекции
        "Массив" => "Коллекция для хранения упорядоченных элементов",
        "Соответствие" => "Коллекция пар ключ-значение",
        "СписокЗначений" => "Список значений с возможностью пометки",
        "ТаблицаЗначений" => "Таблица с произвольным набором колонок",
        "ДеревоЗначений" => "Иерархическая коллекция данных",
        
        // Ввод/вывод
        "ЧтениеТекста" => "Чтение текстовых файлов",
        "ЗаписьТекста" => "Запись текстовых файлов",
        "ЧтениеXML" => "Парсинг XML документов",
        "ЗаписьXML" => "Создание XML документов",
        
        // Системные
        "СистемнаяИнформация" => "Информация о системе и платформе",
        "ИнформацияОПользователе" => "Данные текущего пользователя",
        
        // Запросы
        "Запрос" => "Выполнение запросов к базе данных",
        "ПостроительЗапроса" => "Конструктор запросов",
        
        // Формы
        "УправляемаяФорма" => "Управляемая форма приложения",
        "ЭлементыФормы" => "Коллекция элементов формы",
        
        _ => ""
    }.to_string()
}