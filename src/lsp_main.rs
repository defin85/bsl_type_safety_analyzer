use anyhow::Result;
/// LSP Server main entry point
///
/// 0?CA:05B BSL Language Server G5@57 stdio 8=B5@D59A
/// A?>;L7C5B ACI5AB2CNICN LSP 0@E8B5:BC@C 87 src/lsp/
///
/// ><0=40 70?CA:0: cargo run --bin lsp_server
use bsl_analyzer::lsp;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup logging 4;O LSP (B>;L:> 2 stderr, stdout 8A?>;L7C5BAO 4;O LSP ?@>B>:>;0)
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting BSL Language Server via stdio...");

    // 0?CA:05< LSP A5@25@ G5@57 stdio
    lsp::start_stdio_server().await?;

    Ok(())
}
