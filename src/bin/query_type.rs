use std::path::PathBuf;
use anyhow::{Result, Context};
use clap::Parser;
use bsl_analyzer::unified_index::{UnifiedIndexBuilder};

#[derive(Parser, Debug)]
#[command(author, version, about = "Query unified BSL type index", long_about = None)]
struct Args {
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
    
    /// Configuration path (required if index not cached)
    #[arg(short, long)]
    config: Option<PathBuf>,
    
    /// Platform version (default: "8.3.25")
    #[arg(short, long, default_value = "8.3.25")]
    platform_version: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    // For now, we need to rebuild the index each time
    // TODO: Implement persistent index storage
    let config_path = args.config
        .or_else(|| std::env::current_dir().ok())
        .ok_or_else(|| anyhow::anyhow!("Please specify configuration path with --config"))?;
    
    // Suppress logging for query tool
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::WARN)
        .init();
    
    let builder = UnifiedIndexBuilder::new()
        .context("Failed to create index builder")?;
    
    let index = builder.build_index(&config_path, &args.platform_version)
        .context("Failed to build unified index")?;
    
    // Query the type
    if let Some(entity) = index.find_entity(&args.name) {
        println!("\n✅ Found type: {}", entity.qualified_name);
        println!("Display name: {}", entity.display_name);
        if let Some(eng_name) = &entity.english_name {
            println!("English name: {}", eng_name);
        }
        println!("Type: {:?}", entity.entity_type);
        println!("Kind: {:?}", entity.entity_kind);
        
        if let Some(doc) = &entity.documentation {
            println!("\nDocumentation:");
            println!("{}", doc);
        }
        
        // Show methods
        if args.show_methods || (!args.show_properties && !args.show_methods) {
            if args.show_all_methods {
                let all_methods = index.get_all_methods(&args.name);
                println!("\nAll methods ({} including inherited):", all_methods.len());
                for (name, method) in all_methods.iter().take(20) {
                    print_method(name, method);
                }
                if all_methods.len() > 20 {
                    println!("... and {} more methods", all_methods.len() - 20);
                }
            } else {
                println!("\nDirect methods ({}):", entity.interface.methods.len());
                for (name, method) in entity.interface.methods.iter().take(10) {
                    print_method(name, method);
                }
                if entity.interface.methods.len() > 10 {
                    println!("... and {} more methods", entity.interface.methods.len() - 10);
                }
            }
        }
        
        // Show properties
        if args.show_properties || (!args.show_properties && !args.show_methods) {
            if args.show_all_methods {
                let all_properties = index.get_all_properties(&args.name);
                println!("\nAll properties ({} including inherited):", all_properties.len());
                for (name, property) in all_properties.iter().take(20) {
                    print_property(name, property);
                }
                if all_properties.len() > 20 {
                    println!("... and {} more properties", all_properties.len() - 20);
                }
            } else {
                println!("\nDirect properties ({}):", entity.interface.properties.len());
                for (name, property) in entity.interface.properties.iter().take(10) {
                    print_property(name, property);
                }
                if entity.interface.properties.len() > 10 {
                    println!("... and {} more properties", entity.interface.properties.len() - 10);
                }
            }
        }
        
        // Show inheritance
        if !entity.constraints.parent_types.is_empty() {
            println!("\nInherits from:");
            for parent in &entity.constraints.parent_types {
                println!("  - {}", parent);
            }
        }
        
        // Show relationships
        if !entity.relationships.forms.is_empty() {
            println!("\nForms:");
            for form in &entity.relationships.forms {
                println!("  - {}", form);
            }
        }
        
        if !entity.relationships.tabular_sections.is_empty() {
            println!("\nTabular sections:");
            for ts in &entity.relationships.tabular_sections {
                println!("  - {}", ts);
            }
        }
        
    } else {
        println!("❌ Type '{}' not found in index", args.name);
        
        // Suggest similar types
        println!("\nDid you mean one of these?");
        // Simple suggestion - types that contain the search term
        let search_lower = args.name.to_lowercase();
        let mut suggestions = Vec::new();
        
        for entity in index.get_entities_by_type(&bsl_analyzer::unified_index::BslEntityType::Platform) {
            if entity.display_name.to_lowercase().contains(&search_lower) {
                suggestions.push(&entity.display_name);
            }
        }
        
        for entity in index.get_entities_by_type(&bsl_analyzer::unified_index::BslEntityType::Configuration) {
            if entity.qualified_name.to_lowercase().contains(&search_lower) {
                suggestions.push(&entity.qualified_name);
            }
        }
        
        for (i, suggestion) in suggestions.iter().take(10).enumerate() {
            println!("  {}. {}", i + 1, suggestion);
        }
    }
    
    Ok(())
}

fn print_method(name: &str, method: &bsl_analyzer::unified_index::BslMethod) {
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
    
    println!("  - {}({}){}{}", name, params, return_info, deprecated);
}

fn print_property(name: &str, property: &bsl_analyzer::unified_index::BslProperty) {
    let readonly = if property.is_readonly { " [readonly]" } else { "" };
    println!("  - {}: {}{}", name, property.type_name, readonly);
}