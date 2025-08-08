//! CLI утилита для анализа объектов BSL из индекса документации

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use clap::Parser as ClapParser;
use bsl_analyzer::cli_common::{self, OutputWriter, OutputFormat, CliCommand, ProgressReporter};

#[derive(ClapParser, Debug)]
#[command(
    name = "analyze_objects",
    about = "Анализирует объекты BSL из индекса документации",
    long_about = "Утилита для анализа объектов BSL, извлеченных из документации платформы"
)]
struct Args {
    /// Путь к файлу main_index.json
    #[arg(short, long, default_value = "output/docs_search/main_index.json")]
    input: PathBuf,
    
    /// Путь для сохранения результата
    #[arg(short, long, default_value = "output/docs_search/objects_summary.json")]
    output: PathBuf,
    
    /// Формат вывода (text, json, table, csv)
    #[arg(short, long, default_value = "text")]
    format: String,
    
    /// Показать топ N объектов по количеству методов
    #[arg(short, long, default_value = "10")]
    top: usize,
    
    /// Подробный вывод
    #[arg(short, long)]
    verbose: bool,
    
    /// Тихий режим (только результат)
    #[arg(short, long)]
    quiet: bool,
}

#[derive(Debug, Deserialize)]
struct MainIndex {
    item_index: HashMap<String, ItemIndexEntry>,
}

#[derive(Debug, Deserialize)]
struct ItemIndexEntry {
    category: String,
    #[serde(rename = "file")]
    _file: String,
    title: String,
    object_name: String,
}

#[derive(Debug, Serialize)]
struct ObjectSummary {
    total_unique_objects: usize,
    objects_by_category: HashMap<String, Vec<String>>,
    objects_list: Vec<ObjectInfo>,
    statistics: AnalysisStatistics,
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
    category: String,
}

#[derive(Debug, Serialize)]
struct AnalysisStatistics {
    total_methods: usize,
    total_properties: usize,
    total_functions: usize,
    total_operators: usize,
    categories_count: usize,
    avg_methods_per_object: f64,
}

#[derive(Default)]
struct ObjectStats {
    methods_count: usize,
    properties_count: usize,
    functions_count: usize,
    operators_count: usize,
    english_name: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Initialize logging
    if !args.quiet {
        cli_common::init_logging(args.verbose)?;
    } else {
        cli_common::init_minimal_logging()?;
    }
    
    // Create command and run
    let command = AnalyzeObjectsCommand::new(args);
    cli_common::run_command(command)
}

struct AnalyzeObjectsCommand {
    args: Args,
}

impl AnalyzeObjectsCommand {
    fn new(args: Args) -> Self {
        Self { args }
    }
}

impl CliCommand for AnalyzeObjectsCommand {
    fn name(&self) -> &str {
        "analyze_objects"
    }
    
    fn description(&self) -> &str {
        "Analyze BSL objects from documentation index"
    }
    
    fn execute(&self) -> Result<()> {
        self.run_analysis()
    }
}

impl AnalyzeObjectsCommand {
    fn run_analysis(&self) -> Result<()> {
        // Validate input path
        cli_common::validate_path(&self.args.input, "Input index file")?;
        
        if !self.args.quiet {
            cli_common::print_info("Анализ объектов BSL из индекса документации...");
        }
        
        // Read main_index.json
        let content = fs::read_to_string(&self.args.input)?;
        let index: MainIndex = serde_json::from_str(&content)?;
        
        // Create progress reporter
        let progress = if !self.args.quiet {
            Some(ProgressReporter::new(index.item_index.len(), "Обработка элементов"))
        } else {
            None
        };
        
        // Collect object statistics
        let mut object_stats: HashMap<String, ObjectStats> = HashMap::new();
        
        for (_, entry) in &index.item_index {
            let stats = object_stats
                .entry(entry.object_name.clone())
                .or_insert(ObjectStats::default());
            
            match entry.category.as_str() {
                "methods" => stats.methods_count += 1,
                "properties" => stats.properties_count += 1,
                "functions" => stats.functions_count += 1,
                "operators" => stats.operators_count += 1,
                _ => {}
            }
            
            // Extract English name from title
            if let Some(eng_name) = extract_english_name(&entry.title) {
                stats.english_name = Some(eng_name);
            }
            
            if let Some(p) = &progress {
                p.inc();
            }
        }
        
        if let Some(p) = progress {
            p.finish();
        }
        
        // Categorize objects
        let categories = categorize_objects(&object_stats);
        
        // Create objects list with descriptions
        let mut objects_list: Vec<ObjectInfo> = object_stats
            .iter()
            .map(|(name, stats)| {
                let category = determine_object_category(name);
                ObjectInfo {
                    name: name.clone(),
                    methods_count: stats.methods_count,
                    properties_count: stats.properties_count,
                    functions_count: stats.functions_count,
                    operators_count: stats.operators_count,
                    description: get_object_description(name),
                    english_name: stats.english_name.clone(),
                    category,
                }
            })
            .collect();
        
        // Sort by name
        objects_list.sort_by(|a, b| a.name.cmp(&b.name));
        
        // Calculate statistics
        let statistics = AnalysisStatistics {
            total_methods: objects_list.iter().map(|o| o.methods_count).sum(),
            total_properties: objects_list.iter().map(|o| o.properties_count).sum(),
            total_functions: objects_list.iter().map(|o| o.functions_count).sum(),
            total_operators: objects_list.iter().map(|o| o.operators_count).sum(),
            categories_count: categories.len(),
            avg_methods_per_object: if objects_list.is_empty() {
                0.0
            } else {
                objects_list.iter().map(|o| o.methods_count).sum::<usize>() as f64 
                    / objects_list.len() as f64
            },
        };
        
        let summary = ObjectSummary {
            total_unique_objects: objects_list.len(),
            objects_by_category: categories,
            objects_list,
            statistics,
        };
        
        // Save result
        let json = serde_json::to_string_pretty(&summary)?;
        fs::write(&self.args.output, json)?;
        
        // Display results
        self.display_results(&summary)?;
        
        Ok(())
    }
    
    fn display_results(&self, summary: &ObjectSummary) -> Result<()> {
        let format = OutputFormat::from_str(&self.args.format)?;
        let mut writer = OutputWriter::stdout(format);
        
        if !self.args.quiet {
            writer.write_header("Анализ объектов BSL")?;
            
            // Main statistics
            let stats_rows = vec![
                vec!["Уникальных объектов".to_string(), summary.total_unique_objects.to_string()],
                vec!["Всего методов".to_string(), summary.statistics.total_methods.to_string()],
                vec!["Всего свойств".to_string(), summary.statistics.total_properties.to_string()],
                vec!["Всего функций".to_string(), summary.statistics.total_functions.to_string()],
                vec!["Всего операторов".to_string(), summary.statistics.total_operators.to_string()],
                vec!["Категорий".to_string(), summary.statistics.categories_count.to_string()],
                vec!["Среднее методов на объект".to_string(), 
                     format!("{:.2}", summary.statistics.avg_methods_per_object)],
            ];
            
            writer.write_table(&["Параметр", "Значение"], stats_rows)?;
            
            // Categories breakdown
            writer.write_header("Распределение по категориям")?;
            
            let mut category_rows: Vec<Vec<String>> = summary.objects_by_category
                .iter()
                .map(|(cat, objects)| {
                    vec![
                        get_category_display_name(cat),
                        objects.len().to_string(),
                    ]
                })
                .collect();
            category_rows.sort_by(|a, b| b[1].parse::<usize>().unwrap_or(0)
                .cmp(&a[1].parse::<usize>().unwrap_or(0)));
            
            writer.write_table(&["Категория", "Объектов"], category_rows)?;
            
            // Top objects by methods
            writer.write_header(&format!("Топ-{} объектов по количеству методов", self.args.top))?;
            
            let mut top_objects = summary.objects_list.clone();
            top_objects.sort_by(|a, b| b.methods_count.cmp(&a.methods_count));
            
            let top_rows: Vec<Vec<String>> = top_objects
                .iter()
                .take(self.args.top)
                .enumerate()
                .map(|(i, obj)| {
                    vec![
                        (i + 1).to_string(),
                        obj.name.clone(),
                        obj.methods_count.to_string(),
                        obj.properties_count.to_string(),
                        obj.english_name.clone().unwrap_or_default(),
                    ]
                })
                .collect();
            
            writer.write_table(
                &["#", "Объект", "Методов", "Свойств", "English"], 
                top_rows
            )?;
            
            cli_common::print_success(&format!(
                "Результат сохранен в {}", 
                self.args.output.display()
            ));
        }
        
        writer.flush()?;
        Ok(())
    }
}

fn extract_english_name(title: &str) -> Option<String> {
    // Examples:
    // "Массив.Добавить (Array.Add)"
    // "СоединитьСтроки (StrConcat)"
    
    if let Some(start) = title.find(" (") {
        if let Some(end) = title.find(')') {
            let eng_part = &title[start + 2..end];
            // For methods take only object name
            if let Some(dot_pos) = eng_part.find('.') {
                return Some(eng_part[..dot_pos].to_string());
            } else {
                return Some(eng_part.to_string());
            }
        }
    }
    None
}

fn determine_object_category(name: &str) -> String {
    match name {
        // Collections
        "Массив" | "Соответствие" | "СписокЗначений" | "ТаблицаЗначений" | 
        "ДеревоЗначений" | "ФиксированныйМассив" | "ФиксированноеСоответствие" => "collections",
        
        // I/O
        "ЧтениеТекста" | "ЗаписьТекста" | "ЧтениеXML" | "ЗаписьXML" | 
        "ЧтениеJSON" | "ЗаписьJSON" | "ПотокВПамяти" | "ФайловыйПоток" => "io",
        
        // System
        "СистемнаяИнформация" | "ИнформацияОПользователе" | "НастройкиКлиента" |
        "РаботаСФайлами" | "СредстваКриптографии" => "system",
        
        // Date/Time
        "Дата" | "СтандартныйПериод" | "ОписаниеПериода" => "datetime",
        
        // Metadata
        "Метаданные" | "ОбъектМетаданных" | "КонфигурацияМетаданных" => "metadata",
        
        // Queries
        "Запрос" | "ПостроительЗапроса" | "РезультатЗапроса" | "ВыборкаИзРезультатаЗапроса" => "query",
        
        // Forms
        "УправляемаяФорма" | "ЭлементыФормы" | "ДанныеФормы" | "ПолеФормы" => "forms",
        
        // HTTP and Web
        "HTTPСоединение" | "HTTPЗапрос" | "HTTPОтвет" | "WSПрокси" | "WSСсылка" => "web",
        
        // COM and external components
        "COMОбъект" | "ВнешниеКомпоненты" => "com",
        
        // Other
        _ => "other"
    }.to_string()
}

fn categorize_objects(objects: &HashMap<String, ObjectStats>) -> HashMap<String, Vec<String>> {
    let mut categories: HashMap<String, Vec<String>> = HashMap::new();
    
    for name in objects.keys() {
        let category = determine_object_category(name);
        categories
            .entry(category)
            .or_insert_with(Vec::new)
            .push(name.clone());
    }
    
    // Sort objects in each category
    for objects in categories.values_mut() {
        objects.sort();
    }
    
    categories
}

fn get_category_display_name(category: &str) -> String {
    match category {
        "collections" => "Коллекции",
        "io" => "Ввод/Вывод",
        "system" => "Системные",
        "datetime" => "Дата и время",
        "metadata" => "Метаданные",
        "query" => "Запросы",
        "forms" => "Формы",
        "web" => "HTTP и Web",
        "com" => "COM объекты",
        "other" => "Прочие",
        _ => category,
    }.to_string()
}

fn get_object_description(name: &str) -> String {
    match name {
        // Collections
        "Массив" => "Коллекция для хранения упорядоченных элементов",
        "Соответствие" => "Коллекция пар ключ-значение",
        "СписокЗначений" => "Список значений с возможностью пометки",
        "ТаблицаЗначений" => "Таблица с произвольным набором колонок",
        "ДеревоЗначений" => "Иерархическая коллекция данных",
        
        // I/O
        "ЧтениеТекста" => "Чтение текстовых файлов",
        "ЗаписьТекста" => "Запись текстовых файлов",
        "ЧтениеXML" => "Парсинг XML документов",
        "ЗаписьXML" => "Создание XML документов",
        "ЧтениеJSON" => "Парсинг JSON документов",
        "ЗаписьJSON" => "Создание JSON документов",
        
        // System
        "СистемнаяИнформация" => "Информация о системе и платформе",
        "ИнформацияОПользователе" => "Данные текущего пользователя",
        "РаботаСФайлами" => "Операции с файловой системой",
        "СредстваКриптографии" => "Криптографические операции",
        
        // Queries
        "Запрос" => "Выполнение запросов к базе данных",
        "ПостроительЗапроса" => "Конструктор запросов",
        "РезультатЗапроса" => "Результат выполнения запроса",
        
        // Forms
        "УправляемаяФорма" => "Управляемая форма приложения",
        "ЭлементыФормы" => "Коллекция элементов формы",
        
        _ => ""
    }.to_string()
}