use std::path::PathBuf;
use anyhow::{Result, Context};
use clap::{Parser, ValueEnum};
use bsl_analyzer::unified_index::{UnifiedIndexBuilder, BslLanguagePreference};
use bsl_analyzer::cli_common::{self, OutputWriter, OutputFormat, CliCommand};

#[derive(ValueEnum, Debug, Clone)]
enum LanguagePreference {
    /// Приоритет русским именам (по умолчанию)
    Russian,
    /// Приоритет английским именам
    English,
    /// Автоматическое определение по языку запроса
    Auto,
}

impl From<LanguagePreference> for BslLanguagePreference {
    fn from(pref: LanguagePreference) -> Self {
        match pref {
            LanguagePreference::Russian => BslLanguagePreference::Russian,
            LanguagePreference::English => BslLanguagePreference::English,
            LanguagePreference::Auto => BslLanguagePreference::Auto,
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about = "Query unified BSL type index", long_about = None)]
struct Args {
    /// Enable verbose output
    #[arg(short = 'v', long)]
    verbose: bool,
    
    /// Output format (text, json)
    #[arg(long, default_value = "text")]
    format: String,
    /// Type name to query (e.g., "Массив", "Справочники.Номенклатура")
    #[arg(short, long)]
    name: String,
    
    /// Show all methods (including inherited)
    #[arg(long)]
    show_all_methods: bool,
    
    /// Show only methods
    #[arg(long)]
    show_methods: bool,
    
    /// Show only properties
    #[arg(long)]
    show_properties: bool,
    
    /// Language preference for type search optimization
    #[arg(short, long, value_enum, default_value = "auto")]
    language: LanguagePreference,
    
    /// Configuration path (required if index not cached)
    #[arg(short, long)]
    config: Option<PathBuf>,
    
    /// Platform version (default: "8.3.25")
    #[arg(short, long, default_value = "8.3.25")]
    platform_version: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Инициализируем логирование через общий модуль
    cli_common::init_logging(args.verbose)?;
    
    // Создаем команду и запускаем
    let command = QueryTypeCommand::new(args);
    cli_common::run_command(command)
}

struct QueryTypeCommand {
    args: Args,
}

impl QueryTypeCommand {
    fn new(args: Args) -> Self {
        Self { args }
    }
}

impl CliCommand for QueryTypeCommand {
    fn name(&self) -> &str {
        "query-type"
    }
    
    fn description(&self) -> &str {
        "Query BSL type information from unified index"
    }
    
    fn execute(&self) -> Result<()> {
        self.run_query()
    }
}

impl QueryTypeCommand {
    fn run_query(&self) -> Result<()> {
        // For now, we need to rebuild the index each time
        // TODO: Implement persistent index storage
        let config_path = self.args.config.clone()
            .or_else(|| std::env::current_dir().ok())
            .ok_or_else(|| anyhow::anyhow!("Please specify configuration path with --config"))?;
    
        let mut builder = UnifiedIndexBuilder::new()
            .context("Failed to create index builder")?;
        
        let index = builder.build_index(&config_path, &self.args.platform_version)
            .context("Failed to build unified index")?;
        
        // Create output writer
        let format = OutputFormat::from_str(&self.args.format)?;
        let mut writer = OutputWriter::stdout(format);
        
        // Show application mode if verbose
        if self.args.verbose {
            writer.write_line(&format!("Application mode: {:?}", index.get_application_mode()))?;
        }
    
        // Query the type with language preference
        let language_pref: BslLanguagePreference = self.args.language.clone().into();
        if let Some(entity) = index.find_entity_with_preference(&self.args.name, language_pref) {
            writer.write_header(&format!("Found type: {}", entity.qualified_name))?;
            writer.write_line(&format!("Display name: {}", entity.display_name))?;
            if let Some(eng_name) = &entity.english_name {
                writer.write_line(&format!("English name: {}", eng_name))?;
            }
            writer.write_line(&format!("Type: {:?}", entity.entity_type))?;
            writer.write_line(&format!("Kind: {:?}", entity.entity_kind))?;
            
            if let Some(doc) = &entity.documentation {
                writer.write_header("Documentation")?;
                writer.write_line(doc)?;
            }
        
            // Show methods
            if self.args.show_methods || (!self.args.show_properties && !self.args.show_methods) {
                if self.args.show_all_methods {
                    let all_methods = index.get_all_methods(&self.args.name);
                    writer.write_header(&format!("All methods ({} including inherited)", all_methods.len()))?;
                    for (name, method) in all_methods.iter().take(20) {
                        writer.write_list_item(&format_method(name, method))?;
                    }
                    if all_methods.len() > 20 {
                        writer.write_line(&format!("... and {} more methods", all_methods.len() - 20))?;
                    }
                } else {
                    writer.write_header(&format!("Direct methods ({})", entity.interface.methods.len()))?;
                    for (name, method) in entity.interface.methods.iter().take(10) {
                        writer.write_list_item(&format_method(name, method))?;
                    }
                    if entity.interface.methods.len() > 10 {
                        writer.write_line(&format!("... and {} more methods", entity.interface.methods.len() - 10))?;
                    }
                }
        }
        
            // Show properties
            if self.args.show_properties || (!self.args.show_properties && !self.args.show_methods) {
                if self.args.show_all_methods {
                    let all_properties = index.get_all_properties(&self.args.name);
                    writer.write_header(&format!("All properties ({} including inherited)", all_properties.len()))?;
                    for (name, property) in all_properties.iter().take(20) {
                        writer.write_list_item(&format_property(name, property))?;
                    }
                    if all_properties.len() > 20 {
                        writer.write_line(&format!("... and {} more properties", all_properties.len() - 20))?;
                    }
                } else {
                    writer.write_header(&format!("Direct properties ({})", entity.interface.properties.len()))?;
                    for (name, property) in entity.interface.properties.iter().take(10) {
                        writer.write_list_item(&format_property(name, property))?;
                    }
                    if entity.interface.properties.len() > 10 {
                        writer.write_line(&format!("... and {} more properties", entity.interface.properties.len() - 10))?;
                    }
                }
        }
        
            // Show inheritance
            if !entity.constraints.parent_types.is_empty() {
                writer.write_header("Inherits from")?;
                for parent in &entity.constraints.parent_types {
                    writer.write_list_item(parent)?;
                }
            }
        
            // Show relationships
            if !entity.relationships.forms.is_empty() {
                writer.write_header("Forms")?;
                for form in &entity.relationships.forms {
                    writer.write_list_item(form)?;
                }
            }
            
            if !entity.relationships.tabular_sections.is_empty() {
                writer.write_header("Tabular sections")?;
                for ts in &entity.relationships.tabular_sections {
                    writer.write_list_item(&format!("{} ({})", ts.name, ts.display_name))?;
                    if self.args.show_all_methods {
                        for attr in &ts.attributes {
                            writer.write_line(&format!("      {}: {}", attr.name, attr.type_name))?;
                        }
                    }
                }
            }
        
        } else {
            cli_common::print_error(&format!("Type '{}' not found in index", self.args.name));
            
            // Suggest similar types using the improved suggestion engine
            let suggestions = index.suggest_similar_names(&self.args.name);
            if !suggestions.is_empty() {
                writer.write_header("Did you mean one of these?")?;
                for (i, suggestion) in suggestions.iter().enumerate() {
                    writer.write_line(&format!("  {}. {}", i + 1, suggestion))?;
                }
            } else {
                writer.write_line("No similar types found. Try a different search term.")?;
            }
        }
        
        writer.flush()?;
        Ok(())
    }
}

fn format_method(name: &str, method: &bsl_analyzer::unified_index::BslMethod) -> String {
    let params = method.parameters.iter()
        .map(|p| {
            let mut param_str = p.name.clone();
            if let Some(type_name) = &p.type_name {
                param_str.push_str(&format!(": {}", type_name));
            }
            if p.is_optional {
                param_str = format!("[{}]", param_str);
            }
            param_str
        })
        .collect::<Vec<_>>()
        .join(", ");
    
    let return_info = if method.is_function {
        method.return_type.as_ref()
            .map(|t| format!(" -> {}", t))
            .unwrap_or_else(|| " -> ?".to_string())
    } else {
        String::new()
    };
    
    let deprecated = if method.is_deprecated { " [DEPRECATED]" } else { "" };
    
    format!("{}({}){}{}", name, params, return_info, deprecated)
}

fn format_property(name: &str, property: &bsl_analyzer::unified_index::BslProperty) -> String {
    let readonly = if property.is_readonly { " [readonly]" } else { "" };
    format!("{}: {}{}", name, property.type_name, readonly)
}