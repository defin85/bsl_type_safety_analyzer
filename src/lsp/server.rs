/*!
# Enhanced BSL Language Server

LSP сервер с интеграцией парсеров конфигурации и документации.

## Возможности
- Автодополнение на основе метаданных конфигурации
- Валидация BSL кода в реальном времени
- Hover информация с документацией
- Диагностика и предложения исправлений
*/

use crate::bsl_parser::{BslAnalyzer, BslParser};
use crate::configuration::Configuration;
use crate::docs_integration::DocsIntegration;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

/// Enhanced BSL Language Server с интеграцией документации
pub struct BslLanguageServer {
    client: Client,
    /// Загруженная конфигурация проекта
    configuration: Arc<RwLock<Option<Configuration>>>,
    /// Интеграция с документацией 1С
    docs_integration: Arc<RwLock<DocsIntegration>>,
    /// Кэш открытых документов
    documents: Arc<RwLock<HashMap<Url, DocumentInfo>>>,
    /// BSL анализатор для проверки кода
    analyzer: Arc<RwLock<BslAnalyzer>>,
    /// BSL парсер
    parser: Arc<BslParser>,
}

/// Информация об открытом документе
#[derive(Debug, Clone)]
struct DocumentInfo {
    #[allow(dead_code)]
    uri: Url,
    version: i32,
    text: String,
    /// Результаты последнего анализа
    diagnostics: Vec<Diagnostic>,
}

impl BslLanguageServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            configuration: Arc::new(RwLock::new(None)),
            docs_integration: Arc::new(RwLock::new(DocsIntegration::new())),
            documents: Arc::new(RwLock::new(HashMap::new())),
            analyzer: Arc::new(RwLock::new(
                BslAnalyzer::new().expect("Failed to create BSL analyzer"),
            )),
            parser: Arc::new(BslParser::new().expect("Failed to create BSL parser")),
        }
    }

    /// Загружает конфигурацию из workspace
    async fn load_configuration(&self, workspace_uri: &Url) -> Result<()> {
        let workspace_path = workspace_uri
            .to_file_path()
            .map_err(|_| anyhow::anyhow!("Invalid workspace URI"))?;

        tracing::info!(
            "Loading BSL configuration from: {}",
            workspace_path.display()
        );

        match Configuration::load_from_directory(&workspace_path) {
            Ok(config) => {
                tracing::info!(
                    "Configuration loaded: {} modules, {} metadata contracts, {} forms",
                    config.get_modules().len(),
                    config.metadata_contracts.len(),
                    config.forms.len()
                );

                *self.configuration.write().await = Some(config);

                // Пытаемся загрузить документацию 1С
                self.try_load_documentation(&workspace_path).await;

                // Уведомляем клиента об успешной загрузке
                self.client
                    .show_message(MessageType::INFO, "BSL configuration loaded successfully")
                    .await;

                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to load configuration: {}", e);

                self.client
                    .show_message(
                        MessageType::ERROR,
                        format!("Failed to load BSL configuration: {}", e),
                    )
                    .await;

                Err(e)
            }
        }
    }

    /// Анализирует документ и отправляет диагностику
    async fn analyze_document(&self, _uri: &Url, text: &str) -> Result<Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();

        // Парсим BSL код
        let parse_result = self.parser.parse(text, "unknown");
        match parse_result.ast {
            Some(_ast) => {
                // Выполняем семантический анализ
                let mut analyzer = self.analyzer.write().await;
                if let Err(e) = analyzer.analyze_code(text, "unknown") {
                    // Создаем диагностику об ошибке анализа
                    let diagnostic = Diagnostic {
                        range: Range {
                            start: Position {
                                line: 0,
                                character: 0,
                            },
                            end: Position {
                                line: 0,
                                character: 0,
                            },
                        },
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: None,
                        code_description: None,
                        source: Some("bsl-analyzer".to_string()),
                        message: format!("Analysis error: {}", e),
                        related_information: None,
                        tags: None,
                        data: None,
                    };
                    diagnostics.push(diagnostic);
                } else {
                    // Получаем результаты анализа и конвертируем в LSP диагностику
                    let results = analyzer.get_results();

                    // Конвертируем ошибки анализатора в LSP диагностику
                    for error in results.get_errors() {
                        let diagnostic = Diagnostic {
                            range: Range {
                                start: Position {
                                    line: error.position.line.saturating_sub(1) as u32,
                                    character: error.position.column.saturating_sub(1) as u32,
                                },
                                end: Position {
                                    line: error.position.line.saturating_sub(1) as u32,
                                    character: (error.position.column + 10).saturating_sub(1)
                                        as u32, // Default length
                                },
                            },
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: error
                                .error_code
                                .as_ref()
                                .map(|c| NumberOrString::String(c.clone())),
                            code_description: None,
                            source: Some("bsl-analyzer".to_string()),
                            message: error.message.clone(),
                            related_information: None,
                            tags: None,
                            data: None,
                        };
                        diagnostics.push(diagnostic);
                    }

                    // Конвертируем предупреждения (используем тот же тип AnalysisError)
                    for warning in results.get_warnings() {
                        let diagnostic = Diagnostic {
                            range: Range {
                                start: Position {
                                    line: warning.position.line.saturating_sub(1) as u32,
                                    character: warning.position.column.saturating_sub(1) as u32,
                                },
                                end: Position {
                                    line: warning.position.line.saturating_sub(1) as u32,
                                    character: (warning.position.column + 10).saturating_sub(1)
                                        as u32, // Default length
                                },
                            },
                            severity: Some(DiagnosticSeverity::WARNING),
                            code: warning
                                .error_code
                                .as_ref()
                                .map(|c| NumberOrString::String(c.clone())),
                            code_description: None,
                            source: Some("bsl-analyzer".to_string()),
                            message: warning.message.clone(),
                            related_information: None,
                            tags: None,
                            data: None,
                        };
                        diagnostics.push(diagnostic);
                    }
                }
            }
            None => {
                // Ошибка парсинга
                let diagnostic = Diagnostic {
                    range: Range {
                        start: Position {
                            line: 0,
                            character: 0,
                        },
                        end: Position {
                            line: 0,
                            character: 0,
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: None,
                    code_description: None,
                    source: Some("bsl-parser".to_string()),
                    message: "Parse error: Failed to parse BSL code".to_string(),
                    related_information: None,
                    tags: None,
                    data: None,
                };
                diagnostics.push(diagnostic);
            }
        }

        Ok(diagnostics)
    }

    /// Генерирует автодополнение на основе конфигурации
    async fn provide_completion(
        &self,
        position: Position,
        text: &str,
    ) -> Result<Vec<CompletionItem>> {
        let mut completions = Vec::new();

        // Получаем текущую строку для анализа контекста
        let lines: Vec<&str> = text.lines().collect();
        if let Some(current_line) = lines.get(position.line as usize) {
            let current_char = position.character as usize;
            let line_prefix = if current_char <= current_line.len() {
                &current_line[..current_char]
            } else {
                current_line
            };

            // Анализируем контекст для определения типа автодополнения
            if let Some(config) = self.configuration.read().await.as_ref() {
                // Автодополнение объектов конфигурации
                if line_prefix.ends_with('.')
                    || line_prefix.contains("Справочники.")
                    || line_prefix.contains("Документы.")
                {
                    // Предлагаем объекты метаданных
                    for contract in &config.metadata_contracts {
                        let completion = CompletionItem {
                            label: contract.name.clone(),
                            kind: Some(CompletionItemKind::CLASS),
                            detail: Some(format!("{:?}", contract.object_type)),
                            documentation: Some(Documentation::String(format!(
                                "Объект конфигурации: {} ({})",
                                contract.name,
                                contract.object_type.to_string()
                            ))),
                            insert_text: Some(contract.name.clone()),
                            ..Default::default()
                        };
                        completions.push(completion);
                    }
                }

                // Автодополнение из документации
                let docs = self.docs_integration.read().await;
                if docs.is_loaded() {
                    let doc_completions = docs.get_completions(line_prefix.trim());
                    for doc_completion in doc_completions {
                        let completion = CompletionItem {
                            label: doc_completion.label,
                            kind: Some(CompletionItemKind::FUNCTION),
                            detail: doc_completion.detail,
                            documentation: doc_completion.documentation.map(Documentation::String),
                            insert_text: doc_completion.insert_text,
                            ..Default::default()
                        };
                        completions.push(completion);
                    }
                }
            }
        }

        Ok(completions)
    }

    /// Пытается загрузить документацию 1С из .hbk файлов
    async fn try_load_documentation(&self, workspace_path: &std::path::Path) {
        // Ищем .hbk файлы в workspace
        let hbk_patterns = [
            "help.hbk",
            "1cv8.hbk",
            "syntax.hbk",
            "docs/*.hbk",
            "documentation/*.hbk",
        ];

        for pattern in &hbk_patterns {
            let hbk_path = workspace_path.join(pattern);
            if hbk_path.exists() {
                tracing::info!("Found HBK documentation: {}", hbk_path.display());

                let mut docs = self.docs_integration.write().await;
                match docs.load_documentation(&hbk_path) {
                    Ok(()) => {
                        tracing::info!("HBK documentation loaded successfully");
                        self.client
                            .show_message(
                                MessageType::INFO,
                                "1C documentation loaded - enhanced autocompletion available",
                            )
                            .await;
                        return;
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load HBK documentation: {}", e);
                    }
                }
            }
        }

        // Пытаемся найти .hbk файлы рекурсивно
        if let Ok(entries) = std::fs::read_dir(workspace_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("hbk") {
                    tracing::info!("Found HBK file: {}", path.display());

                    let mut docs = self.docs_integration.write().await;
                    match docs.load_documentation(&path) {
                        Ok(()) => {
                            tracing::info!("HBK documentation loaded successfully");
                            self.client
                                .show_message(
                                    MessageType::INFO,
                                    "1C documentation loaded - enhanced autocompletion available",
                                )
                                .await;
                            return;
                        }
                        Err(e) => {
                            tracing::warn!("Failed to load HBK documentation: {}", e);
                        }
                    }
                }
            }
        }

        tracing::info!("No HBK documentation found in workspace");
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for BslLanguageServer {
    async fn initialize(
        &self,
        params: InitializeParams,
    ) -> tower_lsp::jsonrpc::Result<InitializeResult> {
        tracing::info!("Initializing BSL Language Server");

        // Загружаем конфигурацию из workspace
        if let Some(workspace_folders) = params.workspace_folders {
            if let Some(workspace) = workspace_folders.first() {
                if let Err(e) = self.load_configuration(&workspace.uri).await {
                    tracing::error!("Failed to load workspace configuration: {}", e);
                }
            }
        }

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string(), " ".to_string()]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        identifier: Some("bsl-analyzer".to_string()),
                        inter_file_dependencies: true,
                        workspace_diagnostics: false,
                        ..Default::default()
                    },
                )),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "BSL Language Server".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        tracing::info!("BSL LSP Server initialized successfully");

        self.client
            .log_message(MessageType::INFO, "BSL Language Server ready")
            .await;
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        tracing::debug!("Document opened: {}", params.text_document.uri);

        // Анализируем документ
        match self
            .analyze_document(&params.text_document.uri, &params.text_document.text)
            .await
        {
            Ok(diagnostics) => {
                // Сохраняем информацию о документе
                let doc_info = DocumentInfo {
                    uri: params.text_document.uri.clone(),
                    version: params.text_document.version,
                    text: params.text_document.text,
                    diagnostics: diagnostics.clone(),
                };

                self.documents
                    .write()
                    .await
                    .insert(params.text_document.uri.clone(), doc_info);

                // Отправляем диагностику клиенту
                self.client
                    .publish_diagnostics(
                        params.text_document.uri,
                        diagnostics,
                        Some(params.text_document.version),
                    )
                    .await;
            }
            Err(e) => {
                tracing::error!("Failed to analyze document: {}", e);
            }
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.first() {
            tracing::debug!("Document changed: {}", params.text_document.uri);

            // Анализируем обновленный документ
            match self
                .analyze_document(&params.text_document.uri, &change.text)
                .await
            {
                Ok(diagnostics) => {
                    // Обновляем информацию о документе
                    if let Some(doc_info) = self
                        .documents
                        .write()
                        .await
                        .get_mut(&params.text_document.uri)
                    {
                        doc_info.version = params.text_document.version;
                        doc_info.text = change.text.clone();
                        doc_info.diagnostics = diagnostics.clone();
                    }

                    // Отправляем обновленную диагностику
                    self.client
                        .publish_diagnostics(
                            params.text_document.uri,
                            diagnostics,
                            Some(params.text_document.version),
                        )
                        .await;
                }
                Err(e) => {
                    tracing::error!("Failed to analyze document: {}", e);
                }
            }
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        tracing::debug!("Document closed: {}", params.text_document.uri);

        // Удаляем документ из кэша
        self.documents
            .write()
            .await
            .remove(&params.text_document.uri);

        // Очищаем диагностику
        self.client
            .publish_diagnostics(params.text_document.uri, vec![], None)
            .await;
    }

    async fn completion(
        &self,
        params: CompletionParams,
    ) -> tower_lsp::jsonrpc::Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        if let Some(doc_info) = self.documents.read().await.get(uri) {
            match self.provide_completion(position, &doc_info.text).await {
                Ok(completions) => Ok(Some(CompletionResponse::Array(completions))),
                Err(e) => {
                    tracing::error!("Failed to provide completion: {}", e);
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    }

    async fn hover(&self, params: HoverParams) -> tower_lsp::jsonrpc::Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        if let Some(doc_info) = self.documents.read().await.get(uri) {
            let lines: Vec<&str> = doc_info.text.lines().collect();
            if let Some(line) = lines.get(position.line as usize) {
                // Простая реализация hover - показываем информацию о объектах конфигурации
                if let Some(config) = self.configuration.read().await.as_ref() {
                    for contract in &config.metadata_contracts {
                        if line.contains(&contract.name) {
                            let hover_text = format!(
                                "**{}** ({:?})\n\nРеквизиты: {}\nТабличные части: {}",
                                contract.name,
                                contract.object_type,
                                contract.structure.attributes.len(),
                                contract.structure.tabular_sections.len()
                            );

                            return Ok(Some(Hover {
                                contents: HoverContents::Markup(MarkupContent {
                                    kind: MarkupKind::Markdown,
                                    value: hover_text,
                                }),
                                range: None,
                            }));
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    async fn shutdown(&self) -> tower_lsp::jsonrpc::Result<()> {
        tracing::info!("BSL LSP Server shutting down");
        Ok(())
    }
}
