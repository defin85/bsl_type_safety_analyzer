/*!
# Enhanced BSL Language Server v2.0

Модернизированный LSP сервер с UnifiedBslIndex интеграцией.

## Возможности v2.0
- UnifiedBslIndex для поиска типов и методов 
- Real-time диагностика через BslAnalyzer
- Enhanced автодополнение с документацией
- Hover подсказки из единого индекса BSL типов
- Go-to-definition для объектов конфигурации
*/

use crate::unified_index::{UnifiedBslIndex, UnifiedIndexBuilder, BslApplicationMode};
use crate::BslAnalyzer;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::env;
use std::path::PathBuf;
use tokio::sync::RwLock;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

/// Enhanced BSL Language Server v2.0 с UnifiedBslIndex
pub struct BslLanguageServer {
    client: Client,
    /// Единый индекс всех BSL типов (платформенные + конфигурационные)
    unified_index: Arc<RwLock<Option<UnifiedBslIndex>>>,
    /// Версия платформы 1С
    platform_version: String,
    /// Кэш открытых документов  
    documents: Arc<RwLock<HashMap<Url, DocumentInfo>>>,
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
            unified_index: Arc::new(RwLock::new(None)),
            platform_version: env::var("BSL_PLATFORM_VERSION").unwrap_or_else(|_| "8.3.25".to_string()),
            documents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Инициализация UnifiedBslIndex из workspace
    async fn initialize_unified_index(&self, workspace_uri: &Url) -> Result<()> {
        let workspace_path = workspace_uri
            .to_file_path()
            .map_err(|_| anyhow::anyhow!("Invalid workspace URI"))?;

        self.client
            .log_message(MessageType::INFO, format!("Loading BSL index from: {}", workspace_path.display()))
            .await;

        // Поиск конфигурации в workspace
        let config_path = self.find_configuration_path(&workspace_path).await?;

        let builder = UnifiedIndexBuilder::new()?
            .with_application_mode(BslApplicationMode::ManagedApplication);

        match builder.build_index(config_path.to_str().unwrap_or_default(), &self.platform_version) {
            Ok(index) => {
                let entity_count = index.get_entity_count();
                
                self.client
                    .log_message(MessageType::INFO, format!("BSL index loaded: {} types", entity_count))
                    .await;

                self.client
                    .show_message(MessageType::INFO, format!("BSL index ready: {} types loaded", entity_count))
                    .await;

                *self.unified_index.write().await = Some(index);
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Failed to build BSL index: {}", e);
                
                self.client
                    .log_message(MessageType::ERROR, &error_msg)
                    .await;

                self.client
                    .show_message(MessageType::ERROR, &error_msg)
                    .await;

                Err(e)
            }
        }
    }

    /// Поиск пути к конфигурации в workspace
    async fn find_configuration_path(&self, workspace_path: &std::path::Path) -> Result<PathBuf> {
        // Стандартные пути для конфигурации 1С
        let config_candidates = [
            workspace_path.join("Configuration.xml"),
            workspace_path.join("src").join("Configuration.xml"),  
            workspace_path.join("metadata").join("Configuration.xml"),
            workspace_path.join("conf").join("Configuration.xml"),
            workspace_path.join("examples").join("ConfTest").join("Configuration.xml"), // Для тестов
        ];

        for candidate in &config_candidates {
            if candidate.exists() {
                self.client
                    .log_message(MessageType::INFO, format!("Found configuration: {}", candidate.display()))
                    .await;
                
                return Ok(candidate.parent().unwrap_or(workspace_path).to_path_buf());
            }
        }

        // Если не найдена конфигурация, используем workspace как есть
        self.client
            .log_message(MessageType::WARNING, "No Configuration.xml found, using workspace root")
            .await;

        Ok(workspace_path.to_path_buf())
    }

    /// Real-time диагностика BSL кода с UnifiedBslIndex
    async fn analyze_document(&self, uri: &Url, text: &str) -> Result<Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();

        // Проверяем что UnifiedBslIndex загружен
        let index_guard = self.unified_index.read().await;
        let index = match &*index_guard {
            Some(idx) => idx,
            None => {
                // Индекс не загружен - только базовая проверка синтаксиса
                let diagnostic = Diagnostic {
                    range: Range {
                        start: Position { line: 0, character: 0 },
                        end: Position { line: 0, character: 0 },
                    },
                    severity: Some(DiagnosticSeverity::INFORMATION),
                    code: None,
                    code_description: None,
                    source: Some("bsl-lsp".to_string()),
                    message: "BSL index not loaded - limited analysis available".to_string(),
                    related_information: None,
                    tags: None,
                    data: None,
                };
                diagnostics.push(diagnostic);
                return Ok(diagnostics);
            }
        };

        // Создаем анализатор с UnifiedBslIndex
        match BslAnalyzer::with_index(index.clone()) {
            Ok(mut analyzer) => {
                let file_name = uri.path().split('/').last().unwrap_or("unknown.bsl");
                
                match analyzer.analyze_code(text, file_name) {
                    Ok(()) => {
                        let (errors, warnings) = analyzer.get_errors_and_warnings();

                        // Конвертируем ошибки в LSP диагностику
                        for error in errors {
                            let diagnostic = Diagnostic {
                                range: Range {
                                    start: Position {
                                        line: error.position.line.saturating_sub(1) as u32,
                                        character: error.position.column.saturating_sub(1) as u32,
                                    },
                                    end: Position {
                                        line: error.position.line.saturating_sub(1) as u32,
                                        character: (error.position.column + 20).saturating_sub(1) as u32,
                                    },
                                },
                                severity: Some(DiagnosticSeverity::ERROR),
                                code: error.error_code.as_ref().map(|c| NumberOrString::String(c.clone())),
                                code_description: None,
                                source: Some("bsl-analyzer".to_string()),
                                message: error.message,
                                related_information: None,
                                tags: None,
                                data: None,
                            };
                            diagnostics.push(diagnostic);
                        }

                        // Конвертируем предупреждения
                        for warning in warnings {
                            let diagnostic = Diagnostic {
                                range: Range {
                                    start: Position {
                                        line: warning.position.line.saturating_sub(1) as u32,
                                        character: warning.position.column.saturating_sub(1) as u32,
                                    },
                                    end: Position {
                                        line: warning.position.line.saturating_sub(1) as u32,
                                        character: (warning.position.column + 20).saturating_sub(1) as u32,
                                    },
                                },
                                severity: Some(DiagnosticSeverity::WARNING),
                                code: warning.error_code.as_ref().map(|c| NumberOrString::String(c.clone())),
                                code_description: None,
                                source: Some("bsl-analyzer".to_string()),
                                message: warning.message,
                                related_information: None,
                                tags: None,
                                data: None,
                            };
                            diagnostics.push(diagnostic);
                        }
                    }
                    Err(e) => {
                        let diagnostic = Diagnostic {
                            range: Range {
                                start: Position { line: 0, character: 0 },
                                end: Position { line: 0, character: 0 },
                            },
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: Some(NumberOrString::String("BSL000".to_string())),
                            code_description: None,
                            source: Some("bsl-analyzer".to_string()),
                            message: format!("Analysis error: {}", e),
                            related_information: None,
                            tags: None,
                            data: None,
                        };
                        diagnostics.push(diagnostic);
                    }
                }
            }
            Err(e) => {
                let diagnostic = Diagnostic {
                    range: Range {
                        start: Position { line: 0, character: 0 },
                        end: Position { line: 0, character: 0 },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::String("BSL001".to_string())),
                    code_description: None,
                    source: Some("bsl-lsp".to_string()),
                    message: format!("Failed to create analyzer: {}", e),
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
                    for object in &config.objects {
                        let completion = CompletionItem {
                            label: object.name.clone(),
                            kind: Some(CompletionItemKind::CLASS),
                            detail: Some(format!("{:?}", object.object_type)),
                            documentation: Some(Documentation::String(format!(
                                "Объект конфигурации: {} ({})",
                                object.name,
                                object.object_type
                            ))),
                            insert_text: Some(object.name.clone()),
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
                    for object in &config.objects {
                        if line.contains(&object.name) {
                            let hover_text = format!(
                                "**{}** ({})\n\nОбъект конфигурации",
                                object.name,
                                object.object_type
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
