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

        let mut builder = UnifiedIndexBuilder::new()?
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

    /// Enhanced completion с UnifiedBslIndex
    async fn provide_enhanced_completion(&self, position: Position, text: &str) -> Result<Vec<CompletionItem>> {
        let mut completions = Vec::new();

        // Получаем контекст для анализа
        let lines: Vec<&str> = text.lines().collect();
        let line = lines.get(position.line as usize).unwrap_or(&"");
        let line_prefix = &line[..position.character.min(line.len() as u32) as usize];

        let index_guard = self.unified_index.read().await;
        if let Some(index) = &*index_guard {
            // Автодополнение типов из UnifiedBslIndex
            if line_prefix.ends_with('.') || line_prefix.contains("Справочники.") || line_prefix.contains("Документы.") {
                // Предлагаем объекты конфигурации
                let entities = index.get_all_entities();
                for entity in entities.iter().take(50) { // Ограничиваем количество для производительности
                    if format!("{:?}", entity.entity_type).contains("Configuration") {
                        let completion = CompletionItem {
                            label: entity.display_name.clone(),
                            kind: Some(CompletionItemKind::CLASS),
                            detail: Some(format!("{:?}", entity.entity_type)),
                            documentation: Some(Documentation::String(format!(
                                "Объект конфигурации: {} ({:?})\n\nМетодов: {}, Свойств: {}",
                                entity.display_name,
                                entity.entity_type,
                                entity.interface.methods.len(),
                                entity.interface.properties.len()
                            ))),
                            insert_text: Some(entity.display_name.clone()),
                            ..Default::default()
                        };
                        completions.push(completion);
                    }
                }
            }

            // Автодополнение глобальных функций
            if let Some(global_entity) = index.find_entity("Global") {
                for (method_name, method) in &global_entity.interface.methods {
                    if method_name.to_lowercase().contains(&line_prefix.to_lowercase()) 
                        || line_prefix.is_empty() {
                        let completion = CompletionItem {
                            label: method_name.clone(),
                            kind: Some(CompletionItemKind::FUNCTION),
                            detail: method.return_type.as_ref().map(|t| format!("-> {}", t)),
                            documentation: method.documentation.as_ref().map(|d| Documentation::String(d.clone())),
                            insert_text: Some(format!("{}()", method_name)),
                            ..Default::default()
                        };
                        completions.push(completion);
                    }
                }
            }
        }

        // Базовые ключевые слова BSL
        let bsl_keywords = [
            "Процедура", "Функция", "КонецПроцедуры", "КонецФункции",
            "Если", "Тогда", "Иначе", "КонецЕсли", 
            "Для", "Каждого", "Из", "По", "Цикл", "КонецЦикла",
            "Пока", "КонецЦикла", "Прервать", "Продолжить",
            "Попытка", "Исключение", "КонецПопытки", "ВызватьИсключение",
            "Истина", "Ложь", "Неопределено", "NULL",
        ];

        for keyword in &bsl_keywords {
            if keyword.to_lowercase().starts_with(&line_prefix.to_lowercase()) {
                let completion = CompletionItem {
                    label: keyword.to_string(),
                    kind: Some(CompletionItemKind::KEYWORD),
                    detail: Some("BSL keyword".to_string()),
                    ..Default::default()
                };
                completions.push(completion);
            }
        }

        Ok(completions)
    }

    /// Enhanced hover с информацией из UnifiedBslIndex
    async fn provide_enhanced_hover(&self, position: Position, text: &str) -> Option<Hover> {
        let lines: Vec<&str> = text.lines().collect();
        let line = lines.get(position.line as usize)?;
        
        // Извлекаем слово под курсором
        let char_pos = position.character as usize;
        let word = self.extract_word_at_position(line, char_pos);

        if word.is_empty() {
            return None;
        }

        let index_guard = self.unified_index.read().await;
        if let Some(index) = &*index_guard {
            // Ищем тип в UnifiedBslIndex
            if let Some(entity) = index.find_entity(&word) {
                let hover_content = format!(
                    "**{}** ({:?})\n\n{:?}\n\n**Методов:** {}\n**Свойств:** {}",
                    entity.display_name,
                    entity.entity_type,
                    entity.entity_kind,
                    entity.interface.methods.len(),
                    entity.interface.properties.len()
                );

                return Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: hover_content,
                    }),
                    range: None,
                });
            }

            // Ищем в методах глобального контекста
            if let Some(global_entity) = index.find_entity("Global") {
                if let Some(method) = global_entity.interface.methods.get(&word) {
                    let hover_content = format!(
                        "**{}** (функция)\n\n{}\n\n**Возвращает:** {}",
                        word,
                        method.documentation.as_deref().unwrap_or("Глобальная функция 1С"),
                        method.return_type.as_deref().unwrap_or("Произвольный")
                    );

                    return Some(Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: hover_content,
                        }),
                        range: None,
                    });
                }
            }
        }

        None
    }

    /// Извлечение слова в позиции курсора
    fn extract_word_at_position(&self, line: &str, position: usize) -> String {
        let chars: Vec<char> = line.chars().collect();
        if position >= chars.len() {
            return String::new();
        }

        // Находим границы слова
        let mut start = position;
        let mut end = position;

        // Идем назад до начала слова
        while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
            start -= 1;
        }

        // Идем вперед до конца слова
        while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
            end += 1;
        }

        chars[start..end].iter().collect()
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for BslLanguageServer {
    async fn initialize(&self, params: InitializeParams) -> tower_lsp::jsonrpc::Result<InitializeResult> {
        tracing::info!("Initializing Enhanced BSL Language Server v2.0");

        // Инициализируем UnifiedBslIndex из workspace
        if let Some(workspace_folders) = params.workspace_folders {
            if let Some(workspace) = workspace_folders.first() {
                if let Err(e) = self.initialize_unified_index(&workspace.uri).await {
                    tracing::error!("Failed to initialize BSL index: {}", e);
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
                definition_provider: Some(OneOf::Left(true)),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        identifier: Some("bsl-lsp".to_string()),
                        inter_file_dependencies: true,
                        workspace_diagnostics: false,
                        ..Default::default()
                    },
                )),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "Enhanced BSL Language Server v2.0".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        tracing::info!("Enhanced BSL LSP Server v2.0 initialized successfully");

        self.client
            .log_message(MessageType::INFO, "Enhanced BSL Language Server v2.0 ready with UnifiedBslIndex")
            .await;
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        tracing::debug!("Document opened: {}", params.text_document.uri);

        // Real-time анализ документа
        match self.analyze_document(&params.text_document.uri, &params.text_document.text).await {
            Ok(diagnostics) => {
                // Сохраняем состояние документа
                let doc_info = DocumentInfo {
                    uri: params.text_document.uri.clone(),
                    version: params.text_document.version,
                    text: params.text_document.text,
                    diagnostics: diagnostics.clone(),
                };

                self.documents.write().await.insert(params.text_document.uri.clone(), doc_info);

                // Отправляем диагностику клиенту
                self.client
                    .publish_diagnostics(params.text_document.uri, diagnostics, Some(params.text_document.version))
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

            // Real-time анализ изменений
            match self.analyze_document(&params.text_document.uri, &change.text).await {
                Ok(diagnostics) => {
                    // Обновляем состояние документа
                    if let Some(doc_info) = self.documents.write().await.get_mut(&params.text_document.uri) {
                        doc_info.version = params.text_document.version;
                        doc_info.text = change.text.clone();
                        doc_info.diagnostics = diagnostics.clone();
                    }

                    // Отправляем обновленную диагностику
                    self.client
                        .publish_diagnostics(params.text_document.uri, diagnostics, Some(params.text_document.version))
                        .await;
                }
                Err(e) => {
                    tracing::error!("Failed to analyze document changes: {}", e);
                }
            }
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        tracing::debug!("Document closed: {}", params.text_document.uri);

        // Удаляем документ из кеша
        self.documents.write().await.remove(&params.text_document.uri);

        // Очищаем диагностику
        self.client
            .publish_diagnostics(params.text_document.uri, vec![], None)
            .await;
    }

    async fn completion(&self, params: CompletionParams) -> tower_lsp::jsonrpc::Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        if let Some(doc_info) = self.documents.read().await.get(uri) {
            match self.provide_enhanced_completion(position, &doc_info.text).await {
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
            Ok(self.provide_enhanced_hover(position, &doc_info.text).await)
        } else {
            Ok(None)
        }
    }

    async fn shutdown(&self) -> tower_lsp::jsonrpc::Result<()> {
        tracing::info!("Enhanced BSL LSP Server v2.0 shutting down");
        Ok(())
    }
}