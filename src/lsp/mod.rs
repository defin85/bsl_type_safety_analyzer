/*!
# Language Server Protocol (LSP) implementation

Provides real-time BSL analysis through LSP for IDE integration.
*/

use anyhow::Result;
use tower_lsp::{LspService, Server};

mod diagnostics;
mod server;

pub use diagnostics::{convert_analysis_results, convert_to_lsp_diagnostic};
pub use server::BslLanguageServer;

/// Starts LSP server in stdio mode
pub async fn start_stdio_server() -> Result<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(BslLanguageServer::new);

    tracing::info!("BSL LSP Server starting...");

    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}

/// Starts LSP server on TCP port
pub async fn start_tcp_server(_port: u16) -> Result<()> {
    // TODO: Implement TCP server
    tracing::warn!("TCP LSP server not yet implemented");
    anyhow::bail!("TCP server not implemented");
}
