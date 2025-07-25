// LSP сервер
use tower_lsp::{Client, LanguageServer};

pub struct BslLanguageServer {
    client: Client,
}

impl BslLanguageServer {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for BslLanguageServer {
    async fn initialize(
        &self,
        _params: tower_lsp::lsp_types::InitializeParams,
    ) -> tower_lsp::jsonrpc::Result<tower_lsp::lsp_types::InitializeResult> {
        Ok(tower_lsp::lsp_types::InitializeResult::default())
    }
    
    async fn initialized(&self, _params: tower_lsp::lsp_types::InitializedParams) {
        tracing::info!("BSL LSP Server initialized");
    }
    
    async fn shutdown(&self) -> tower_lsp::jsonrpc::Result<()> {
        Ok(())
    }
}
