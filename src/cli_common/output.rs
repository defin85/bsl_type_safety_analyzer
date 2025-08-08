//! Модуль для форматирования и вывода результатов

use anyhow::Result;
use colored::Colorize;
use serde::Serialize;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::str::FromStr;

/// Формат вывода результатов
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputFormat {
    Text,
    Json,
    Table,
    Csv,
}

impl FromStr for OutputFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "text" | "txt" => Ok(OutputFormat::Text),
            "json" => Ok(OutputFormat::Json),
            "table" => Ok(OutputFormat::Table),
            "csv" => Ok(OutputFormat::Csv),
            _ => Err(anyhow::anyhow!("Unknown output format: {}", s)),
        }
    }
}

impl OutputFormat {
    /// Совместимая обёртка
    pub fn parse_output_format(s: &str) -> Result<Self> {
        <Self as FromStr>::from_str(s)
    }
}

/// Writer для вывода результатов
pub struct OutputWriter {
    writer: Box<dyn Write>,
    format: OutputFormat,
    pretty: bool,
}

impl OutputWriter {
    /// Создает writer для stdout
    pub fn stdout(format: OutputFormat) -> Self {
        Self {
            writer: Box::new(io::stdout()),
            format,
            pretty: false,
        }
    }

    /// Создает writer для файла
    pub fn file(path: &Path, format: OutputFormat) -> Result<Self> {
        let file = File::create(path)?;
        Ok(Self {
            writer: Box::new(file),
            format,
            pretty: false,
        })
    }

    /// Включает pretty-печать для JSON
    pub fn with_pretty(mut self, pretty: bool) -> Self {
        self.pretty = pretty;
        self
    }

    /// Записывает сериализуемый объект
    pub fn write_object<T: Serialize>(&mut self, obj: &T) -> Result<()> {
        match self.format {
            OutputFormat::Json => {
                let json = if self.pretty {
                    serde_json::to_string_pretty(obj)?
                } else {
                    serde_json::to_string(obj)?
                };
                writeln!(self.writer, "{}", json)?;
            }
            OutputFormat::Text => {
                // Для текстового формата используем JSON с pretty print
                let json = serde_json::to_string_pretty(obj)?;
                writeln!(self.writer, "{}", json)?;
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "Format {:?} not supported for objects",
                    self.format
                ));
            }
        }
        Ok(())
    }

    /// Записывает строку
    pub fn write_line(&mut self, line: &str) -> Result<()> {
        writeln!(self.writer, "{}", line)?;
        Ok(())
    }

    /// Записывает заголовок
    pub fn write_header(&mut self, header: &str) -> Result<()> {
        match self.format {
            OutputFormat::Text => {
                writeln!(self.writer, "\n{}", header.bold().blue())?;
                writeln!(self.writer, "{}", "=".repeat(header.len()).blue())?;
            }
            _ => {
                writeln!(self.writer, "{}", header)?;
            }
        }
        Ok(())
    }

    /// Записывает элемент списка
    pub fn write_list_item(&mut self, item: &str) -> Result<()> {
        match self.format {
            OutputFormat::Text => {
                writeln!(self.writer, "  • {}", item)?;
            }
            _ => {
                writeln!(self.writer, "- {}", item)?;
            }
        }
        Ok(())
    }

    /// Записывает таблицу
    pub fn write_table(&mut self, headers: &[&str], rows: Vec<Vec<String>>) -> Result<()> {
        match self.format {
            OutputFormat::Table | OutputFormat::Text => {
                // Вычисляем ширину колонок
                let mut widths = headers.iter().map(|h| h.len()).collect::<Vec<_>>();
                for row in &rows {
                    for (i, cell) in row.iter().enumerate() {
                        if i < widths.len() {
                            widths[i] = widths[i].max(cell.len());
                        }
                    }
                }

                // Печатаем заголовки
                for (i, header) in headers.iter().enumerate() {
                    if i > 0 {
                        write!(self.writer, " │ ")?;
                    }
                    write!(self.writer, "{:width$}", header.bold(), width = widths[i])?;
                }
                writeln!(self.writer)?;

                // Печатаем разделитель
                for (i, width) in widths.iter().enumerate() {
                    if i > 0 {
                        write!(self.writer, "─┼─")?;
                    }
                    write!(self.writer, "{}", "─".repeat(*width))?;
                }
                writeln!(self.writer)?;

                // Печатаем строки
                for row in rows {
                    for (i, cell) in row.iter().enumerate() {
                        if i > 0 {
                            write!(self.writer, " │ ")?;
                        }
                        if i < widths.len() {
                            write!(self.writer, "{:width$}", cell, width = widths[i])?;
                        } else {
                            write!(self.writer, "{}", cell)?;
                        }
                    }
                    writeln!(self.writer)?;
                }
            }
            OutputFormat::Csv => {
                // Печатаем заголовки CSV
                writeln!(self.writer, "{}", headers.join(","))?;
                // Печатаем строки
                for row in rows {
                    writeln!(self.writer, "{}", row.join(","))?;
                }
            }
            OutputFormat::Json => {
                // Конвертируем в JSON объекты
                let mut objects = Vec::new();
                for row in rows {
                    let mut obj = serde_json::Map::new();
                    for (i, header) in headers.iter().enumerate() {
                        if i < row.len() {
                            obj.insert(
                                header.to_string(),
                                serde_json::Value::String(row[i].clone()),
                            );
                        }
                    }
                    objects.push(serde_json::Value::Object(obj));
                }
                let json = if self.pretty {
                    serde_json::to_string_pretty(&objects)?
                } else {
                    serde_json::to_string(&objects)?
                };
                writeln!(self.writer, "{}", json)?;
            }
        }
        Ok(())
    }

    /// Завершает запись и сбрасывает буфер
    pub fn flush(&mut self) -> Result<()> {
        self.writer.flush()?;
        Ok(())
    }
}

/// Структура для вывода статистики
#[derive(Debug, Serialize)]
pub struct Statistics {
    pub total: usize,
    pub categories: Vec<(String, usize)>,
    pub duration: Option<String>,
}

impl Statistics {
    /// Выводит статистику в указанный writer
    pub fn write(&self, writer: &mut OutputWriter) -> Result<()> {
        match writer.format {
            OutputFormat::Json => {
                writer.write_object(self)?;
            }
            OutputFormat::Text | OutputFormat::Table => {
                writer.write_header("Statistics")?;
                writer.write_line(&format!("Total items: {}", self.total))?;

                if !self.categories.is_empty() {
                    writer.write_line("\nBy category:")?;
                    for (category, count) in &self.categories {
                        writer.write_line(&format!("  {}: {}", category, count))?;
                    }
                }

                if let Some(duration) = &self.duration {
                    writer.write_line(&format!("\nDuration: {}", duration))?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}
