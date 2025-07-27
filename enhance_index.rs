use bsl_analyzer::docs_integration::chunked_loader::generate_enhanced_index;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    let docs_dir = if args.len() > 1 {
        &args[1]
    } else {
        "output/docs_search"
    };
    
    println!("Enhancing index for: {}", docs_dir);
    println!("This will add item_index mapping to main_index.json...");
    
    match generate_enhanced_index(docs_dir) {
        Ok(()) => {
            println!("✅ Index enhanced successfully!");
            println!("main_index.json now contains item_index with object mapping");
        }
        Err(e) => {
            eprintln!("❌ Failed to enhance index: {}", e);
            std::process::exit(1);
        }
    }
    
    Ok(())
}