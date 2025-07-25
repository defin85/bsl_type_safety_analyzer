/*!
# BSL LSP Server

Language Server Protocol implementation for BSL (1C:Enterprise).
*/

use anyhow::Result;
use bsl_analyzer::lsp::start_stdio_server;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging for LSP server
    tracing_subscriber::fmt()
        .with_env_filter("bsl_analyzer=info")
        .with_writer(std::io::stderr) // LSP uses stdout for protocol
        .init();
    
    tracing::info!("Starting BSL LSP Server");
    
    // Start LSP server in stdio mode
    start_stdio_server().await?;
    
    Ok(())
}
