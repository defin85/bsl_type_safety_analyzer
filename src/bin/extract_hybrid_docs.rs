//! CLI утилита для извлечения документации BSL из архивов HBK

use std::path::{Path, PathBuf};
use std::fs;
use anyhow::Result;
use clap::Parser as ClapParser;
use bsl_analyzer::docs_integration::bsl_syntax_extractor::{BslSyntaxExtractor, BslSyntaxDatabase};
use bsl_analyzer::cli_common::{self, OutputWriter, OutputFormat, CliCommand, ProgressReporter};
use serde::{Serialize, Deserialize};

#[derive(ClapParser, Debug)]
#[command(
    name = "extract_hybrid_docs",
    about = "Извлекает документацию BSL из архивов HBK",
    long_about = "Утилита для извлечения синтаксической базы данных BSL из архивов документации 1С (.hbk, .zip)"
)]
struct Args {
    /// Путь к архиву HBK (.hbk или .zip)
    #[arg(short, long, help = "Путь к архиву документации")]
    archive: PathBuf,
    
    /// Директория для сохранения результатов
    #[arg(short, long, default_value = "./output/bsl_syntax_database.json")]
    output: PathBuf,
    
    /// Формат вывода (text, json, table)
    #[arg(short, long, default_value = "text")]
    format: String,
    
    /// Максимальное количество файлов для обработки
    #[arg(short, long)]
    max_files: Option<usize>,
    
    /// Сохранить отдельные категории в разные файлы
    #[arg(short, long)]
    split_categories: bool,
    
    /// Подробный вывод
    #[arg(short, long)]
    verbose: bool,
    
    /// Тихий режим
    #[arg(short, long)]
    quiet: bool,
    
    /// Включить детальную статистику
    #[arg(short, long)]
    detailed_stats: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExtractionResult {
    archive_path: String,
    output_path: String,
    statistics: ExtractionStatistics,
    categories: Vec<CategoryInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExtractionStatistics {
    total_objects: usize,
    total_methods: usize,
    total_properties: usize,
    total_functions: usize,
    total_operators: usize,
    total_items: usize,
    extraction_time_ms: u128,
    archive_size_bytes: u64,
    output_size_bytes: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct CategoryInfo {
    name: String,
    count: usize,
    examples: Vec<String>,
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
    let command = ExtractHybridDocsCommand::new(args);
    cli_common::run_command(command)
}

struct ExtractHybridDocsCommand {
    args: Args,
}

impl ExtractHybridDocsCommand {
    fn new(args: Args) -> Self {
        Self { args }
    }
}

impl CliCommand for ExtractHybridDocsCommand {
    fn name(&self) -> &str {
        "extract_hybrid_docs"
    }
    
    fn description(&self) -> &str {
        "Extract BSL documentation from HBK archives"
    }
    
    fn execute(&self) -> Result<()> {
        self.run_extraction()
    }
}

impl ExtractHybridDocsCommand {
    fn run_extraction(&self) -> Result<()> {
        // Validate input path
        cli_common::validate_path(&self.args.archive, "Archive file")?;
        
        if !self.args.quiet {
            cli_common::print_info("Извлечение документации BSL из архива...");
        }
        
        let start_time = std::time::Instant::now();
        
        // Get archive size
        let archive_metadata = fs::metadata(&self.args.archive)?;
        let archive_size = archive_metadata.len();
        
        // Create extractor
        let mut extractor = BslSyntaxExtractor::new(self.args.archive.to_str().unwrap());
        
        // Set up progress reporting
        let progress = if !self.args.quiet {
            Some(ProgressReporter::new(100, "Извлечение документации"))
        } else {
            None
        };
        
        // Extract syntax database
        if !self.args.quiet {
            cli_common::print_info(&format!(
                "📁 Источник: {} ({:.2} MB)",
                self.args.archive.display(),
                archive_size as f64 / 1_048_576.0
            ));
        }
        
        let database = extractor.extract_syntax_database(self.args.max_files)?;
        
        if let Some(p) = progress {
            p.finish();
        }
        
        // Create output directory if needed
        if let Some(parent) = self.args.output.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Save results
        if self.args.split_categories {
            self.save_split_categories(&database)?;
        } else {
            self.save_single_file(&database)?;
        }
        
        // Calculate statistics
        let output_size = if self.args.output.exists() {
            fs::metadata(&self.args.output)?.len()
        } else {
            0
        };
        
        let elapsed = start_time.elapsed();
        
        let result = ExtractionResult {
            archive_path: self.args.archive.display().to_string(),
            output_path: self.args.output.display().to_string(),
            statistics: ExtractionStatistics {
                total_objects: database.objects.len(),
                total_methods: database.methods.len(),
                total_properties: database.properties.len(),
                total_functions: database.functions.len(),
                total_operators: database.operators.len(),
                total_items: database.objects.len() + database.methods.len() + 
                           database.properties.len() + database.functions.len() + 
                           database.operators.len(),
                extraction_time_ms: elapsed.as_millis(),
                archive_size_bytes: archive_size,
                output_size_bytes: output_size,
            },
            categories: self.collect_category_info(&database),
        };
        
        // Display results
        self.display_results(&result)?;
        
        Ok(())
    }
    
    fn save_single_file(&self, database: &BslSyntaxDatabase) -> Result<()> {
        let json_data = serde_json::to_string_pretty(database)?;
        fs::write(&self.args.output, json_data)?;
        
        if !self.args.quiet {
            cli_common::print_success(&format!(
                "База данных сохранена в {}", 
                self.args.output.display()
            ));
        }
        
        Ok(())
    }
    
    fn save_split_categories(&self, database: &BslSyntaxDatabase) -> Result<()> {
        let base_path = self.args.output.with_extension("");
        let base_name = base_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("bsl_syntax");
        
        let parent = self.args.output.parent().unwrap_or(Path::new("."));
        
        // Save each category separately
        // Objects
        if !database.objects.is_empty() {
            let file_path = parent.join(format!("{}_objects.json", base_name));
            let json_data = serde_json::to_string_pretty(&database.objects)?;
            fs::write(&file_path, json_data)?;
            if !self.args.quiet && self.args.verbose {
                println!("  📄 Сохранено: {}", file_path.display());
            }
        }
        
        // Methods
        if !database.methods.is_empty() {
            let file_path = parent.join(format!("{}_methods.json", base_name));
            let json_data = serde_json::to_string_pretty(&database.methods)?;
            fs::write(&file_path, json_data)?;
            if !self.args.quiet && self.args.verbose {
                println!("  📄 Сохранено: {}", file_path.display());
            }
        }
        
        // Properties
        if !database.properties.is_empty() {
            let file_path = parent.join(format!("{}_properties.json", base_name));
            let json_data = serde_json::to_string_pretty(&database.properties)?;
            fs::write(&file_path, json_data)?;
            if !self.args.quiet && self.args.verbose {
                println!("  📄 Сохранено: {}", file_path.display());
            }
        }
        
        // Functions
        if !database.functions.is_empty() {
            let file_path = parent.join(format!("{}_functions.json", base_name));
            let json_data = serde_json::to_string_pretty(&database.functions)?;
            fs::write(&file_path, json_data)?;
            if !self.args.quiet && self.args.verbose {
                println!("  📄 Сохранено: {}", file_path.display());
            }
        }
        
        // Operators
        if !database.operators.is_empty() {
            let file_path = parent.join(format!("{}_operators.json", base_name));
            let json_data = serde_json::to_string_pretty(&database.operators)?;
            fs::write(&file_path, json_data)?;
            if !self.args.quiet && self.args.verbose {
                println!("  📄 Сохранено: {}", file_path.display());
            }
        }
        
        if !self.args.quiet {
            cli_common::print_success(&format!(
                "Категории сохранены в {}/", 
                parent.display()
            ));
        }
        
        Ok(())
    }
    
    fn collect_category_info(&self, database: &BslSyntaxDatabase) -> Vec<CategoryInfo> {
        let mut categories = Vec::new();
        
        // Objects
        if !database.objects.is_empty() {
            let examples: Vec<String> = database.objects.keys()
                .take(3)
                .cloned()
                .collect();
            categories.push(CategoryInfo {
                name: "objects".to_string(),
                count: database.objects.len(),
                examples,
            });
        }
        
        // Methods
        if !database.methods.is_empty() {
            let examples: Vec<String> = database.methods.keys()
                .take(3)
                .cloned()
                .collect();
            categories.push(CategoryInfo {
                name: "methods".to_string(),
                count: database.methods.len(),
                examples,
            });
        }
        
        // Properties
        if !database.properties.is_empty() {
            let examples: Vec<String> = database.properties.keys()
                .take(3)
                .cloned()
                .collect();
            categories.push(CategoryInfo {
                name: "properties".to_string(),
                count: database.properties.len(),
                examples,
            });
        }
        
        // Functions
        if !database.functions.is_empty() {
            let examples: Vec<String> = database.functions.keys()
                .take(3)
                .cloned()
                .collect();
            categories.push(CategoryInfo {
                name: "functions".to_string(),
                count: database.functions.len(),
                examples,
            });
        }
        
        // Operators
        if !database.operators.is_empty() {
            let examples: Vec<String> = database.operators.keys()
                .take(3)
                .cloned()
                .collect();
            categories.push(CategoryInfo {
                name: "operators".to_string(),
                count: database.operators.len(),
                examples,
            });
        }
        
        categories.sort_by(|a, b| b.count.cmp(&a.count));
        categories
    }
    
    fn display_results(&self, result: &ExtractionResult) -> Result<()> {
        let format = OutputFormat::from_str(&self.args.format)?;
        let mut writer = OutputWriter::stdout(format);
        
        if !self.args.quiet {
            writer.write_header("Извлечение документации BSL")?;
            
            // Basic statistics
            let stats_rows = vec![
                vec!["Объектов".to_string(), result.statistics.total_objects.to_string()],
                vec!["Методов".to_string(), result.statistics.total_methods.to_string()],
                vec!["Свойств".to_string(), result.statistics.total_properties.to_string()],
                vec!["Функций".to_string(), result.statistics.total_functions.to_string()],
                vec!["Операторов".to_string(), result.statistics.total_operators.to_string()],
                vec!["ИТОГО элементов".to_string(), result.statistics.total_items.to_string()],
            ];
            
            writer.write_table(&["Категория", "Количество"], stats_rows)?;
            
            // Performance metrics
            if self.args.detailed_stats {
                writer.write_header("Метрики производительности")?;
                
                let perf_rows = vec![
                    vec![
                        "Время извлечения".to_string(), 
                        format!("{:.2} сек", result.statistics.extraction_time_ms as f64 / 1000.0)
                    ],
                    vec![
                        "Размер архива".to_string(),
                        format!("{:.2} MB", result.statistics.archive_size_bytes as f64 / 1_048_576.0)
                    ],
                    vec![
                        "Размер результата".to_string(),
                        format!("{:.2} MB", result.statistics.output_size_bytes as f64 / 1_048_576.0)
                    ],
                    vec![
                        "Коэффициент сжатия".to_string(),
                        format!("{:.2}x", 
                            result.statistics.archive_size_bytes as f64 / 
                            result.statistics.output_size_bytes.max(1) as f64)
                    ],
                    vec![
                        "Скорость обработки".to_string(),
                        format!("{:.0} элементов/сек",
                            result.statistics.total_items as f64 * 1000.0 / 
                            result.statistics.extraction_time_ms.max(1) as f64)
                    ],
                ];
                
                writer.write_table(&["Метрика", "Значение"], perf_rows)?;
            }
            
            // Categories breakdown
            if !result.categories.is_empty() && self.args.verbose {
                writer.write_header("Распределение по категориям")?;
                
                for category in &result.categories {
                    writer.write_line(&format!(
                        "📂 {} ({} элементов)",
                        category.name, category.count
                    ))?;
                    
                    if !category.examples.is_empty() {
                        writer.write_line("   Примеры:")?;
                        for example in &category.examples {
                            writer.write_line(&format!("   • {}", example))?;
                        }
                    }
                }
            }
            
            cli_common::print_success(&format!(
                "Извлечено {} элементов за {:.2} сек",
                result.statistics.total_items,
                result.statistics.extraction_time_ms as f64 / 1000.0
            ));
        }
        
        writer.flush()?;
        Ok(())
    }
}