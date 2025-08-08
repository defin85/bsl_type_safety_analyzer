//! CLI утилита для анализа реального формата отчета конфигурации

use anyhow::Result;
use bsl_analyzer::cli_common::{self, CliCommand, OutputFormat, OutputWriter, ProgressReporter};
use clap::Parser as ClapParser;
use encoding_rs::{UTF_16LE, UTF_8, WINDOWS_1251};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(ClapParser, Debug)]
#[command(
    name = "analyze_real_report",
    about = "Анализирует структуру реального отчета конфигурации 1С",
    long_about = "Утилита для анализа текстового отчета конфигурации, извлечения объектов и их метаданных"
)]
struct Args {
    /// Путь к файлу отчета
    #[arg(short, long, help = "Путь к текстовому отчету конфигурации")]
    report: PathBuf,

    /// Формат вывода (text, json, table)
    #[arg(short, long, default_value = "text")]
    format: String,

    /// Максимальное количество объектов для анализа
    #[arg(short, long)]
    limit: Option<usize>,

    /// Показать только статистику
    #[arg(short, long)]
    stats_only: bool,

    /// Вывести список объектов в файл
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Подробный вывод
    #[arg(short, long)]
    verbose: bool,

    /// Тихий режим
    #[arg(short, long)]
    quiet: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct ReportAnalysis {
    encoding: String,
    total_lines: usize,
    objects: Vec<ConfigurationObject>,
    statistics: ReportStatistics,
}

#[derive(Debug, Serialize, Deserialize)]
struct ConfigurationObject {
    name: String,
    object_type: String,
    line_number: usize,
    attributes: Vec<ObjectAttribute>,
    properties: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ObjectAttribute {
    name: String,
    attr_type: String,
    line_number: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct ReportStatistics {
    total_objects: usize,
    total_attributes: usize,
    objects_by_type: HashMap<String, usize>,
    avg_attributes_per_object: f64,
    encoding_used: String,
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
    let command = AnalyzeReportCommand::new(args);
    cli_common::run_command(command)
}

struct AnalyzeReportCommand {
    args: Args,
}

impl AnalyzeReportCommand {
    fn new(args: Args) -> Self {
        Self { args }
    }
}

impl CliCommand for AnalyzeReportCommand {
    fn name(&self) -> &str {
        "analyze_real_report"
    }

    fn description(&self) -> &str {
        "Analyze 1C configuration text report structure"
    }

    fn execute(&self) -> Result<()> {
        self.run_analysis()
    }
}

impl AnalyzeReportCommand {
    fn run_analysis(&self) -> Result<()> {
        // Validate input path
        cli_common::validate_path(&self.args.report, "Report file")?;

        if !self.args.quiet {
            cli_common::print_info(&format!("Анализ отчета: {}", self.args.report.display()));
        }

        // Read file with encoding detection
        let file_bytes = fs::read(&self.args.report)?;
        let (content, encoding) = self.detect_and_decode(&file_bytes);

        if !self.args.quiet {
            cli_common::print_info(&format!("Обнаружена кодировка: {}", encoding));
        }

        // Parse report
        let analysis = self.parse_report(&content, &encoding)?;

        // Save to file if requested
        if let Some(output_path) = &self.args.output {
            let json = serde_json::to_string_pretty(&analysis)?;
            fs::write(output_path, json)?;
            if !self.args.quiet {
                cli_common::print_success(&format!(
                    "Результат сохранен в {}",
                    output_path.display()
                ));
            }
        }

        // Display results
        self.display_results(&analysis)?;

        Ok(())
    }

    fn detect_and_decode(&self, file_bytes: &[u8]) -> (String, String) {
        // Try different encodings
        if let (decoded, _, false) = UTF_16LE.decode(file_bytes) {
            (decoded.into_owned(), "UTF-16LE".to_string())
        } else if let (decoded, _, false) = UTF_8.decode(file_bytes) {
            (decoded.into_owned(), "UTF-8".to_string())
        } else if let (decoded, _, false) = WINDOWS_1251.decode(file_bytes) {
            (decoded.into_owned(), "Windows-1251".to_string())
        } else {
            // Fallback to UTF-8 with replacements
            let (decoded, _, _) = UTF_8.decode(file_bytes);
            (decoded.into_owned(), "UTF-8 (с заменами)".to_string())
        }
    }

    fn parse_report(&self, content: &str, encoding: &str) -> Result<ReportAnalysis> {
        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();

        let mut objects = Vec::new();
        let mut in_object = false;
        let mut current_object: Option<ConfigurationObject> = None;

        // Progress reporting
        let progress = if !self.args.quiet && !self.args.stats_only {
            Some(ProgressReporter::new(total_lines, "Анализ строк"))
        } else {
            None
        };

        for (i, line) in lines.iter().enumerate() {
            if let Some(p) = &progress {
                if i % 1000 == 0 {
                    p.update(i);
                }
            }

            // Check limit
            if let Some(limit) = self.args.limit {
                if objects.len() >= limit {
                    break;
                }
            }

            let trimmed = line.trim();

            // Look for configuration objects (start with "-")
            if trimmed.starts_with('-') && trimmed.contains('.') {
                let object_line = trimmed.trim_start_matches('-').trim();

                // Check if it's a main object, not nested
                if !object_line.contains(".Реквизиты.") && !object_line.contains(".ТабличныеЧасти.")
                {
                    if let Some(object_type) = self.determine_object_type(object_line) {
                        // Save previous object
                        if let Some(obj) = current_object.take() {
                            objects.push(obj);
                        }

                        // Start new object
                        current_object = Some(ConfigurationObject {
                            name: object_line.to_string(),
                            object_type,
                            line_number: i + 1,
                            attributes: Vec::new(),
                            properties: HashMap::new(),
                        });
                        in_object = true;
                    }
                } else if in_object && object_line.contains(".Реквизиты.") {
                    // This is an attribute of current object
                    if let Some(ref mut obj) = current_object {
                        if let Some(attr_name) = self.extract_attribute_name(object_line) {
                            // Look for type in next lines
                            let attr_type = self.find_attribute_type(&lines, i);
                            obj.attributes.push(ObjectAttribute {
                                name: attr_name,
                                attr_type,
                                line_number: i + 1,
                            });
                        }
                    }
                }
            } else if !trimmed.starts_with('-') && in_object && trimmed.contains(':') {
                // Object properties
                if let Some(ref mut obj) = current_object {
                    let parts: Vec<&str> = trimmed.splitn(2, ':').collect();
                    if parts.len() == 2 {
                        let key = parts[0].trim();
                        let value = parts[1].trim().trim_matches('"');
                        if !value.is_empty() && key != "Тип" {
                            obj.properties.insert(key.to_string(), value.to_string());
                        }
                    }
                }
            }
        }

        // Save last object
        if let Some(obj) = current_object {
            objects.push(obj);
        }

        if let Some(p) = progress {
            p.finish();
        }

        // Calculate statistics
        let statistics = self.calculate_statistics(&objects, encoding);

        Ok(ReportAnalysis {
            encoding: encoding.to_string(),
            total_lines,
            objects,
            statistics,
        })
    }

    fn determine_object_type(&self, object_line: &str) -> Option<String> {
        if object_line.contains("Справочники.") {
            Some("Справочник".to_string())
        } else if object_line.contains("Документы.") {
            Some("Документ".to_string())
        } else if object_line.contains("Константы.") {
            Some("Константа".to_string())
        } else if object_line.contains("РегистрыСведений.") {
            Some("РегистрСведений".to_string())
        } else if object_line.contains("РегистрыНакопления.") {
            Some("РегистрНакопления".to_string())
        } else if object_line.contains("Перечисления.") {
            Some("Перечисление".to_string())
        } else if object_line.contains("Отчеты.") {
            Some("Отчет".to_string())
        } else if object_line.contains("Обработки.") {
            Some("Обработка".to_string())
        } else if object_line.contains("Языки.") {
            Some("Язык".to_string())
        } else {
            None
        }
    }

    fn extract_attribute_name(&self, object_line: &str) -> Option<String> {
        let parts: Vec<&str> = object_line.split('.').collect();
        if parts.len() >= 4 {
            Some(parts[3].to_string())
        } else {
            None
        }
    }

    fn find_attribute_type(&self, lines: &[&str], start_index: usize) -> String {
        for next_line in lines
            .iter()
            .skip(start_index + 1)
            .take(lines.len().saturating_sub(start_index + 1).min(9))
            .map(|s| s.trim())
        {
            if next_line.starts_with("Тип:") {
                return next_line
                    .strip_prefix("Тип:")
                    .unwrap_or("")
                    .trim()
                    .to_string();
            }
            if next_line.starts_with('-') {
                break;
            }
        }
        "Неизвестный".to_string()
    }

    fn calculate_statistics(
        &self,
        objects: &[ConfigurationObject],
        encoding: &str,
    ) -> ReportStatistics {
        let mut objects_by_type = HashMap::new();
        let total_attributes: usize = objects.iter().map(|o| o.attributes.len()).sum();

        for obj in objects {
            *objects_by_type.entry(obj.object_type.clone()).or_insert(0) += 1;
        }

        let avg_attributes_per_object = if objects.is_empty() {
            0.0
        } else {
            total_attributes as f64 / objects.len() as f64
        };

        ReportStatistics {
            total_objects: objects.len(),
            total_attributes,
            objects_by_type,
            avg_attributes_per_object,
            encoding_used: encoding.to_string(),
        }
    }

    fn display_results(&self, analysis: &ReportAnalysis) -> Result<()> {
        let format = OutputFormat::parse_output_format(&self.args.format)?;
        let mut writer = OutputWriter::stdout(format);

        if self.args.stats_only {
            // Show only statistics
            writer.write_header("Статистика отчета конфигурации")?;

            let stats_rows = vec![
                vec![
                    "Кодировка".to_string(),
                    analysis.statistics.encoding_used.clone(),
                ],
                vec!["Всего строк".to_string(), analysis.total_lines.to_string()],
                vec![
                    "Найдено объектов".to_string(),
                    analysis.statistics.total_objects.to_string(),
                ],
                vec![
                    "Найдено реквизитов".to_string(),
                    analysis.statistics.total_attributes.to_string(),
                ],
                vec![
                    "Среднее реквизитов на объект".to_string(),
                    format!("{:.2}", analysis.statistics.avg_attributes_per_object),
                ],
            ];

            writer.write_table(&["Параметр", "Значение"], stats_rows)?;

            // Objects by type
            if !analysis.statistics.objects_by_type.is_empty() {
                writer.write_header("Распределение по типам")?;

                let mut type_rows: Vec<Vec<String>> = analysis
                    .statistics
                    .objects_by_type
                    .iter()
                    .map(|(t, count)| vec![t.clone(), count.to_string()])
                    .collect();
                type_rows.sort_by(|a, b| {
                    b[1].parse::<usize>()
                        .unwrap_or(0)
                        .cmp(&a[1].parse::<usize>().unwrap_or(0))
                });

                writer.write_table(&["Тип объекта", "Количество"], type_rows)?;
            }
        } else {
            // Show full results
            writer.write_header("Анализ отчета конфигурации")?;

            // Statistics first
            writer.write_line(&format!("📊 Кодировка: {}", analysis.encoding))?;
            writer.write_line(&format!("📊 Всего строк: {}", analysis.total_lines))?;
            writer.write_line(&format!(
                "📊 Найдено объектов: {}",
                analysis.statistics.total_objects
            ))?;
            writer.write_line(&format!(
                "📊 Найдено реквизитов: {}",
                analysis.statistics.total_attributes
            ))?;
            writer.write_line("")?;

            // Objects details
            if !analysis.objects.is_empty() && self.args.verbose {
                writer.write_header("Найденные объекты")?;

                for (i, obj) in analysis.objects.iter().enumerate() {
                    if i >= 10 && !self.args.verbose {
                        writer.write_line(&format!(
                            "... и еще {} объектов",
                            analysis.objects.len() - 10
                        ))?;
                        break;
                    }

                    writer.write_line(&format!(
                        "🔷 #{} {} [{}] (строка {})",
                        i + 1,
                        obj.name,
                        obj.object_type,
                        obj.line_number
                    ))?;

                    // Show properties
                    for (key, value) in &obj.properties {
                        writer.write_line(&format!("  🔸 {}: {}", key, value))?;
                    }

                    // Show attributes
                    if !obj.attributes.is_empty() {
                        writer
                            .write_line(&format!("  📌 Реквизиты ({}):", obj.attributes.len()))?;
                        for attr in &obj.attributes {
                            writer
                                .write_line(&format!("    - {} : {}", attr.name, attr.attr_type))?;
                        }
                    }

                    writer.write_line("")?;
                }
            }
        }

        writer.flush()?;
        Ok(())
    }
}
