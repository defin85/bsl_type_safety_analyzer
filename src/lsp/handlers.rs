use tower_lsp::{
    jsonrpc::Result,
    lsp_types::*,
    Client,
};
use crate::parser::BslParser;

pub struct LspHandlers {
    client: Client,
    parser: BslParser,
}

impl LspHandlers {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            parser: BslParser::new(),
        }
    }

    pub async fn handle_document_changed(&self, params: DidChangeTextDocumentParams) -> Result<()> {
        let uri = params.text_document.uri;
        let content = params.content_changes.first()
            .map(|change| change.text.as_str())
            .unwrap_or("");

        // Запускаем анализ в фоновом режиме
        if let Err(e) = self.analyze_document(&uri, content).await {
            self.client.log_message(MessageType::ERROR, &format!("Analysis failed: {}", e)).await;
        }

        Ok(())
    }

    async fn analyze_document(&self, uri: &Url, content: &str) -> Result<()> {
        // Базовый анализ документа
        match self.parser.parse_text(content) {
            Ok(_ast) => {
                // Отправляем диагностику об успешном парсинге
                let diagnostics = vec![];
                self.client.publish_diagnostics(uri.clone(), diagnostics, None).await;
            }
            Err(e) => {
                // Отправляем ошибки парсинга как диагностику
                let diagnostic = Diagnostic {
                    range: Range {
                        start: Position { line: 0, character: 0 },
                        end: Position { line: 0, character: 100 },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: format!("Parse error: {}", e),
                    ..Default::default()
                };
                self.client.publish_diagnostics(uri.clone(), vec![diagnostic], None).await;
            }
        }

        Ok(())
    }
}
