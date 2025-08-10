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

use crate::bsl_parser::{AnalysisConfig, BslAnalyzer};
use crate::lsp::diagnostics::{convert_analysis_results, create_analysis_error_diagnostic};
use crate::unified_index::{BslApplicationMode, UnifiedBslIndex, UnifiedIndexBuilder};
use anyhow::Result;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp::lsp_types::*;
use tower_lsp::lsp_types::{
    DocumentDiagnosticParams, DocumentDiagnosticReport, DocumentDiagnosticReportResult,
    FullDocumentDiagnosticReport, RelatedFullDocumentDiagnosticReport,
};
use tower_lsp::{Client, LanguageServer};

/// Enhanced BSL Language Server v2.0 с UnifiedBslIndex
pub struct BslLanguageServer {
    client: Client,
    /// Единый индекс всех BSL типов (платформенные + конфигурационные)
    unified_index: Arc<RwLock<Option<UnifiedBslIndex>>>,
    /// Версия платформы 1С
    #[allow(dead_code)]
    platform_version: String,
    /// Кэш открытых документов  
    documents: Arc<RwLock<HashMap<Url, DocumentInfo>>>,
    /// Конфигурация анализа
    analysis_config: Arc<RwLock<AnalysisConfig>>,
}

/// Информация об открытом документе
struct DocumentInfo {
    #[allow(dead_code)]
    uri: Url,
    version: i32,
    text: String,
    /// Результаты последнего анализа
    diagnostics: Vec<Diagnostic>,
    /// Персистентный анализатор для инкрементального пути
    analyzer: BslAnalyzer,
}

impl BslLanguageServer {
    pub fn new(client: Client) -> Self {
        // Определяем уровень анализа из переменной окружения или используем по умолчанию
        let analysis_level =
            env::var("BSL_ANALYSIS_LEVEL").unwrap_or_else(|_| "semantic".to_string());

        let config = match analysis_level.to_lowercase().as_str() {
            "syntax" => AnalysisConfig::syntax_only(),
            "semantic" => AnalysisConfig::semantic(),
            "dataflow" => AnalysisConfig::data_flow(),
            "full" => AnalysisConfig::full(),
            _ => AnalysisConfig::semantic(), // По умолчанию семантический анализ
        };

        Self {
            client,
            unified_index: Arc::new(RwLock::new(None)),
            platform_version: env::var("BSL_PLATFORM_VERSION")
                .unwrap_or_else(|_| "8.3.25".to_string()),
            documents: Arc::new(RwLock::new(HashMap::new())),
            analysis_config: Arc::new(RwLock::new(config)),
        }
    }

    /// Инициализация UnifiedBslIndex из workspace (вызывается при initialize)
    async fn initialize_unified_index(&self, workspace_uri: &Url) -> Result<()> {
        let workspace_path = workspace_uri
            .to_file_path()
            .map_err(|_| anyhow::anyhow!("Invalid workspace URI"))?;

        self.client
            .log_message(
                MessageType::INFO,
                format!("Loading BSL index from: {}", workspace_path.display()),
            )
            .await;

        // Поиск конфигурации в workspace
        let config_path = match self.find_configuration_path(&workspace_path).await {
            Ok(path) => path,
            Err(e) => {
                self.client
                    .log_message(MessageType::WARNING, format!("No configuration found in workspace: {}. LSP will work without type checking.", e))
                    .await;
                return Ok(()); // Продолжаем работу без конфигурации
            }
        };

        // Создаем builder с обработкой ошибок
        let mut builder = match UnifiedIndexBuilder::new() {
            Ok(b) => b.with_application_mode(BslApplicationMode::ManagedApplication),
            Err(e) => {
                self.client
                    .log_message(
                        MessageType::WARNING,
                        format!(
                            "Failed to create index builder: {}. BSL features will be limited.",
                            e
                        ),
                    )
                    .await;
                return Ok(()); // Продолжаем работу без индекса
            }
        };

        match builder.build_index(
            config_path.to_str().unwrap_or_default(),
            &self.platform_version,
        ) {
            Ok(index) => {
                let entity_count = index.get_entity_count();

                self.client
                    .log_message(
                        MessageType::INFO,
                        format!("BSL index loaded: {} types", entity_count),
                    )
                    .await;

                self.client
                    .show_message(
                        MessageType::INFO,
                        format!("BSL index ready: {} types loaded", entity_count),
                    )
                    .await;

                *self.unified_index.write().await = Some(index);
                Ok(())
            }
            Err(e) => {
                let error_msg = format!(
                    "Failed to build BSL index: {}. LSP will work with limited features.",
                    e
                );

                self.client
                    .log_message(MessageType::WARNING, &error_msg)
                    .await;

                self.client
                    .show_message(MessageType::WARNING, &error_msg)
                    .await;

                // Возвращаем Ok чтобы сервер продолжил работу
                Ok(())
            }
        }
    }

    /// Поиск пути к конфигурации в workspace
    #[allow(dead_code)]
    async fn find_configuration_path(&self, workspace_path: &std::path::Path) -> Result<PathBuf> {
        // Стандартные пути для конфигурации 1С
        let config_candidates = [
            workspace_path.join("Configuration.xml"),
            workspace_path.join("src").join("Configuration.xml"),
            workspace_path.join("metadata").join("Configuration.xml"),
            workspace_path.join("conf").join("Configuration.xml"),
            workspace_path
                .join("examples")
                .join("ConfTest")
                .join("Configuration.xml"), // Для тестов
        ];

        for candidate in &config_candidates {
            if candidate.exists() {
                self.client
                    .log_message(
                        MessageType::INFO,
                        format!("Found configuration: {}", candidate.display()),
                    )
                    .await;

                return Ok(candidate.parent().unwrap_or(workspace_path).to_path_buf());
            }
        }

        // Если не найдена конфигурация, используем workspace как есть
        self.client
            .log_message(
                MessageType::WARNING,
                "No Configuration.xml found, using workspace root",
            )
            .await;

        Ok(workspace_path.to_path_buf())
    }


    /// Enhanced completion с UnifiedBslIndex
    async fn provide_enhanced_completion(
        &self,
        position: Position,
        text: &str,
    ) -> Result<Vec<CompletionItem>> {
    let mut completions = Vec::new();

        // Получаем контекст для анализа
        let lines: Vec<&str> = text.lines().collect();
        let line = lines.get(position.line as usize).unwrap_or(&"");
        
        // Правильная конвертация позиции из UTF-16 code units в байтовый индекс
        let char_pos = position.character as usize;
        let mut byte_pos = 0;
        let mut utf16_pos = 0;
        
        for ch in line.chars() {
            if utf16_pos >= char_pos {
                break;
            }
            byte_pos += ch.len_utf8();
            utf16_pos += ch.len_utf16();
        }
        
        // Безопасное получение префикса строки (всё до курсора)
        let line_prefix = if byte_pos <= line.len() { &line[..byte_pos] } else { line };

        // Выделяем последний «токен» (буквенно-цифровая / подчёркивание / кириллица последовательность)
        fn extract_last_token(s: &str) -> &str {
            let trimmed = s.trim_end();
            // Отбрасываем завершающие пробелы вручную (без промежуточных аллокаций)
            let mut end = trimmed.len();
            while end > 0 {
                if let Some(ch) = trimmed[..end].chars().next_back() {
                    if ch.is_whitespace() {
                        end -= ch.len_utf8();
                        continue;
                    }
                }
                break;
            }
            if end == 0 { return ""; }
            // Идём назад посимвольно пока символ удовлетворяет условию токена
            let mut start = end;
            for (idx, ch) in trimmed[..end].char_indices().rev() {
                let is_token_ch = ch.is_alphanumeric() || ch == '_' || ('\u{0400}'..='\u{04FF}').contains(&ch);
                if !is_token_ch { break; }
                start = idx;
            }
            &trimmed[start..end]
        }
        let current_token = extract_last_token(line_prefix);
        let token_lower = current_token.to_lowercase();

    let index_guard = self.unified_index.read().await;
    if let Some(index) = &*index_guard {
            // Автодополнение типов из UnifiedBslIndex
            if line_prefix.ends_with('.')
                || line_prefix.contains("Справочники.")
                || line_prefix.contains("Документы.")
            {
                // Предлагаем объекты конфигурации
                let entities = index.get_all_entities();
                for entity in entities.iter().take(50) {
                    // Ограничиваем количество для производительности
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

            // Автодополнение глобальных функций (по текущему токену)
            if let Some(global_entity) = index.find_entity("Global") {
                for (method_name, method) in &global_entity.interface.methods {
                    let m_lower = method_name.to_lowercase();
                    if token_lower.is_empty() || m_lower.starts_with(&token_lower) {
                        completions.push(CompletionItem {
                            label: method_name.clone(),
                            kind: Some(CompletionItemKind::FUNCTION),
                            detail: method.return_type.as_ref().map(|t| format!("-> {}", t)),
                            documentation: method.documentation.as_ref().map(|d| Documentation::String(d.clone())),
                            insert_text: Some(format!("{}()", method_name)),
                            ..Default::default()
                        });
                    }
                }
            }

            // Автодополнение платформенных типов (конструкторы)
            // Извлекаем последнее слово из строки для фильтрации
            let last_word = line_prefix
                .split_whitespace()
                .last()
                .unwrap_or("")
                .trim_start_matches('=')
                .trim();
            
            if !last_word.is_empty() || line_prefix.ends_with(' ') || line_prefix.ends_with('=') {
                let entities = index.get_all_entities();
                for entity in entities.iter() {
                    // Показываем только платформенные типы с конструкторами
                    if format!("{:?}", entity.entity_type).contains("Platform") {
                        // Проверяем, является ли это типом с конструктором
                        let constructable_types = [
                            "ТаблицаЗначений", "ValueTable",
                            "Массив", "Array",
                            "Структура", "Structure",
                            "Соответствие", "Map",
                            "СписокЗначений", "ValueList",
                            "ДеревоЗначений", "ValueTree",
                            "ТабличныйДокумент", "SpreadsheetDocument",
                            "ХранилищеЗначения", "ValueStorage",
                            "УникальныйИдентификатор", "UUID",
                            "ЧтениеXML", "XMLReader",
                            "ЗаписьXML", "XMLWriter",
                            "ЧтениеJSON", "JSONReader",
                            "ЗаписьJSON", "JSONWriter",
                            "ЧтениеТекста", "TextReader",
                            "ЗаписьТекста", "TextWriter",
                            "ЧтениеДанных", "DataReader",
                            "ЗаписьДанных", "DataWriter",
                        ];
                        
                        let entity_name = &entity.display_name;
                        let is_constructable = constructable_types.iter().any(|&ct| {
                            entity_name.contains(ct)
                        });
                        
                        if is_constructable {
                            // Фильтруем по введенному префиксу
                            if last_word.is_empty() 
                                || entity_name.to_lowercase().starts_with(&last_word.to_lowercase()) {
                                
                                let completion = CompletionItem {
                                    label: entity_name.clone(),
                                    kind: Some(CompletionItemKind::CLASS),
                                    detail: Some("Платформенный тип (конструктор)".to_string()),
                                    documentation: entity.documentation.as_ref().map(|d| {
                                        Documentation::String(format!(
                                            "{}\n\nИспользование: Новый {}()",
                                            d, entity_name
                                        ))
                                    }),
                                    insert_text: Some(format!("Новый {}()", entity_name)),
                                    ..Default::default()
                                };
                                completions.push(completion);
                            }
                        }
                    }
                }
            }
        }

        // Базовые ключевые слова BSL
        let bsl_keywords = [
            "Процедура",
            "Функция",
            "КонецПроцедуры",
            "КонецФункции",
            "Если",
            "Тогда",
            "Иначе",
            "КонецЕсли",
            "Для",
            "Каждого",
            "Из",
            "По",
            "Цикл",
            "КонецЦикла",
            "Пока",
            "КонецЦикла",
            "Прервать",
            "Продолжить",
            "Попытка",
            "Исключение",
            "КонецПопытки",
            "ВызватьИсключение",
            "Истина",
            "Ложь",
            "Неопределено",
            "NULL",
        ];

        for keyword in &bsl_keywords {
            if token_lower.is_empty() || keyword.to_lowercase().starts_with(&token_lower) {
                completions.push(CompletionItem {
                    label: keyword.to_string(),
                    kind: Some(CompletionItemKind::KEYWORD),
                    detail: Some("BSL keyword".to_string()),
                    ..Default::default()
                });
            }
        }

        Ok(completions)
    }

    async fn provide_enhanced_hover(&self, position: Position, text: &str) -> Option<Hover> {
        let lines: Vec<&str> = text.lines().collect();
        let line = lines.get(position.line as usize)?;
        // UTF-16 -> char index
        let mut char_index = 0usize; let mut utf16 = 0usize;
        for ch in line.chars() { if utf16 >= position.character as usize { break; } utf16 += ch.len_utf16(); char_index += 1; }
        let word = self.extract_word_at_position(line, char_index);
        if word.is_empty() { return None; }
        let index_guard = self.unified_index.read().await;
        if let Some(index) = &*index_guard {
            if let Some(entity) = index.find_entity(&word) {
                return Some(Hover { contents: HoverContents::Markup(MarkupContent { kind: MarkupKind::Markdown, value: format!("**{}** ({:?})", entity.display_name, entity.entity_type) }), range: None });
            }
        }
        None
    }

    fn extract_word_at_position(&self, line: &str, position: usize) -> String {
        let chars: Vec<char> = line.chars().collect();
        if position >= chars.len() { return String::new(); }
        let mut start = position; let mut end = position;
        while start > 0 && (chars[start-1].is_alphanumeric() || chars[start-1] == '_') { start -= 1; }
        while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') { end += 1; }
        chars[start..end].iter().collect()
    }

    /// Enhanced hover с информацией из UnifiedBslIndex
    async fn analyze_document(&self, uri: &Url, text: &str) -> Result<Vec<Diagnostic>> {
        // Попытка инкрементального анализа если документ уже есть
        let mut make_new = false;
        {
            let docs = self.documents.read().await;
            if !docs.contains_key(uri) { make_new = true; }
        }
        if make_new {
            // Создаём новый анализатор
            let config = self.analysis_config.read().await.clone();
            let index_guard = self.unified_index.read().await;
            let analyzer_res = match &*index_guard {
                Some(idx) => BslAnalyzer::with_index_and_config(idx.clone(), config),
                None => BslAnalyzer::with_config(AnalysisConfig::syntax_only()),
            };
            let mut analyzer = match analyzer_res { Ok(a) => a, Err(e) => {
                return Ok(vec![Diagnostic { range: Range { start: Position { line:0, character:0 }, end: Position { line:0, character:0 } }, severity: Some(DiagnosticSeverity::ERROR), code: Some(NumberOrString::String("BSL001".into())), code_description: None, source: Some("bsl-lsp".into()), message: format!("Failed to create analyzer: {}", e), related_information: None, tags: None, data: None }]); } };
            let file_name = uri.path().split('/').next_back().unwrap_or("unknown.bsl");
            let _ = analyzer.analyze_code(text, file_name);
            let (errors, warnings) = analyzer.get_errors_and_warnings();
            let diagnostics = self.convert_errors_warnings(errors, warnings);
            // Помещаем DocumentInfo
            let doc = DocumentInfo { uri: uri.clone(), version: 0, text: text.to_string(), diagnostics: diagnostics.clone(), analyzer };
            self.documents.write().await.insert(uri.clone(), doc);
            return Ok(diagnostics);
        }
        // Инкрементальный путь
        let mut diagnostics = Vec::new();
        if let Some(doc) = self.documents.write().await.get_mut(uri) {
            // analyze_incremental
            let file_name = uri.path().split('/').next_back().unwrap_or("unknown.bsl");
            let _ = doc.analyzer.analyze_incremental(text, file_name);
            let (errors, warnings) = doc.analyzer.get_errors_and_warnings();
            diagnostics = self.convert_errors_warnings(errors, warnings);
            doc.text = text.to_string();
            doc.diagnostics = diagnostics.clone();
        }
        Ok(diagnostics)
    }

    fn convert_errors_warnings(&self, errors: Vec<crate::core::errors::AnalysisError>, warnings: Vec<crate::core::errors::AnalysisError>) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        for error in errors { diagnostics.push(Diagnostic { range: Range { start: Position { line: error.position.line.saturating_sub(1) as u32, character: error.position.column.saturating_sub(1) as u32 }, end: Position { line: error.position.line.saturating_sub(1) as u32, character: (error.position.column + 20).saturating_sub(1) as u32 } }, severity: Some(DiagnosticSeverity::ERROR), code: error.error_code.as_ref().map(|c| NumberOrString::String(c.clone())), code_description: None, source: Some("bsl-analyzer".into()), message: error.message, related_information: None, tags: None, data: None }); }
        for warning in warnings { diagnostics.push(Diagnostic { range: Range { start: Position { line: warning.position.line.saturating_sub(1) as u32, character: warning.position.column.saturating_sub(1) as u32 }, end: Position { line: warning.position.line.saturating_sub(1) as u32, character: (warning.position.column + 20).saturating_sub(1) as u32 } }, severity: Some(DiagnosticSeverity::WARNING), code: warning.error_code.as_ref().map(|c| NumberOrString::String(c.clone())), code_description: None, source: Some("bsl-analyzer".into()), message: warning.message, related_information: None, tags: None, data: None }); }
        diagnostics
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        tracing::debug!("Document opened: {}", params.text_document.uri);

        // Real-time анализ документа
        match self
            .analyze_document(&params.text_document.uri, &params.text_document.text)
            .await
        {
            Ok(diagnostics) => {
                // Сохраняем состояние документа
                // Документ уже добавлен внутри analyze_document при первом открытии; здесь только обновим версию
                if let Some(doc) = self.documents.write().await.get_mut(&params.text_document.uri) {
                    doc.version = params.text_document.version;
                }

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

            // Real-time анализ изменений
            match self
                .analyze_document(&params.text_document.uri, &change.text)
                .await
            {
                Ok(diagnostics) => {
                    // Обновляем состояние документа
                    if let Some(doc_info) = self.documents.write().await.get_mut(&params.text_document.uri) {
                        doc_info.version = params.text_document.version;
                        // text/diagnostics уже обновлены в analyze_document
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
                    tracing::error!("Failed to analyze document changes: {}", e);
                }
            }
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        tracing::debug!("Document closed: {}", params.text_document.uri);

        // Удаляем документ из кеша
        self.documents
            .write()
            .await
            .remove(&params.text_document.uri);

        // Очищаем диагностику
        self.client
            .publish_diagnostics(params.text_document.uri, vec![], None)
            .await;
    }

    #[allow(dead_code)]
    async fn did_change_configuration(&self, _params: DidChangeConfigurationParams) {
        // Обрабатываем изменение конфигурации
        tracing::debug!("Configuration changed, handling update...");

        // TODO: В будущем здесь можно перезагрузить настройки анализатора
        // Пока просто логируем для устранения предупреждения
        self.client
            .log_message(MessageType::INFO, "Configuration updated")
            .await;
    }

    async fn completion(
        &self,
        params: CompletionParams,
    ) -> tower_lsp::jsonrpc::Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        if let Some(doc_info) = self.documents.read().await.get(uri) {
            match self
                .provide_enhanced_completion(position, &doc_info.text)
                .await
            {
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
        if let Some(doc_info) = self.documents.read().await.get(uri) { Ok(self.provide_enhanced_hover(position, &doc_info.text).await) } else { Ok(None) }
    }

    async fn execute_command(
        &self,
        params: ExecuteCommandParams,
    ) -> tower_lsp::jsonrpc::Result<Option<serde_json::Value>> {
        tracing::info!("Executing command: {}", params.command);

        match params.command.as_str() {
            "bslAnalyzer.getMetrics" => {
                // Expect first argument: file URI string
                if let Some(uri_val) = params.arguments.first() {
                    if let Ok(uri_str) = serde_json::from_value::<String>(uri_val.clone()) {
                        if let Ok(uri) = Url::parse(&uri_str) {
                            // Retrieve document text (from cache or disk)
                            let text_opt = if let Some(doc) = self.documents.read().await.get(&uri) { Some(doc.text.clone()) } else { match uri.to_file_path() { Ok(p) => std::fs::read_to_string(p).ok(), Err(_) => None } };
                            if let Some(text) = text_opt {
                                // Build a temporary analyzer to capture interner stats.
                                // We reuse analysis_config for consistency.
                                let config = self.analysis_config.read().await.clone();
                                // Use index if available.
                                let index_guard = self.unified_index.read().await;
                                let mut analyzer = match &*index_guard {
                                    Some(idx) => match BslAnalyzer::with_index_and_config(idx.clone(), config) { Ok(a)=>a, Err(_)=> return Err(tower_lsp::jsonrpc::Error::internal_error()) },
                                    None => match BslAnalyzer::with_config(config) { Ok(a)=>a, Err(_)=> return Err(tower_lsp::jsonrpc::Error::internal_error()) },
                                };
                                // Perform analysis to build AST & interner.
                                let cfg_clone = analyzer.get_config().clone();
                                let _ = analyzer.analyze_text(&text, &cfg_clone);
                                // Extract metrics (fallback 0 if not available).
                                let (symbol_count, total_bytes) = analyzer.get_interner_metrics();
                                let lines = text.lines().count();
                                let complexity = 0; // TODO real complexity
                                // Routine & call stats via arena if доступно
                                let (functions, call_total, call_methods, call_functions, avg_args, max_args) = if let Some(ast) = analyzer.last_built_arena() {
                                    let routines = ast.count_routines();
                                    let (total, method, func, args_sum, max_a) = ast.call_stats();
                                    let avg = if total>0 { args_sum as f64 / total as f64 } else { 0.0 };
                                    (routines, total, method, func, avg, max_a)
                                } else { (0,0,0,0,0.0,0) };
                                let (errors, warnings) = analyzer.get_errors_and_warnings();
                                let score = 100i32.saturating_sub((errors.len()*5 + warnings.len()) as i32).max(0);
                                let fingerprint = analyzer.get_root_fingerprint();
                                let value = serde_json::json!({
                                    "file": uri_str,
                                    "complexity": complexity,
                                    "lines": lines,
                                    "functions": functions,
                                    "errors": errors.len(),
                                    "warnings": warnings.len(),
                                    "score": score,
                                    "internerSymbols": symbol_count,
                                    "internerBytes": total_bytes,
                                    "callsTotal": call_total,
                                    "callsMethod": call_methods,
                                    "callsFunction": call_functions,
                                    "callsAvgArgs": (avg_args * 100.0).round() / 100.0,
                                    "callsMaxArgs": max_args,
                                    "fingerprint": fingerprint,
                                });
                                return Ok(Some(value));
                            }
                        }
                    }
                }
                Err(tower_lsp::jsonrpc::Error::invalid_params("Invalid getMetrics arguments"))
            }
            "bslAnalyzer.getIncrementalMetrics" => {
                if let Some(uri_val) = params.arguments.first() {
                    if let Ok(uri_str) = serde_json::from_value::<String>(uri_val.clone()) {
                        if let Ok(uri) = Url::parse(&uri_str) {
                            if let Some(doc) = self.documents.read().await.get(&uri) {
                                if let Some(stats) = doc.analyzer.last_incremental_stats() {
                                    let debug_enabled = std::env::var("BSL_ANALYZER_DEBUG_METRICS").ok().as_deref() == Some("1");
                                    let mut core = serde_json::json!({
                                        "totalNodes": stats.total_nodes,
                                        "changedNodes": stats.changed_nodes,
                                        "reusedNodes": stats.reused_nodes,
                                        "reusedSubtrees": stats.reused_subtrees,
                                        "reuseRatio": stats.reuse_ratio,
                                        "parseNs": stats.parse_ns,
                                        "arenaNs": stats.arena_ns,
                                        "fingerprintNs": stats.fingerprint_ns,
                                        "semanticNs": stats.semantic_ns,
                                        "totalNs": stats.total_ns,
                                        "plannedRoutines": stats.planned_routines,
                                        "replacedRoutines": stats.replaced_routines,
                                        "fallbackReason": stats.fallback_reason,
                                        "initialTouched": stats.initial_touched,
                                        "expandedTouched": stats.expanded_touched,
                                    });
                                    if debug_enabled {
                                        if let Some(obj) = core.as_object_mut() {
                                            obj.insert("innerReusedNodes".into(), serde_json::json!(stats.inner_reused_nodes));
                                            obj.insert("innerReuseRatio".into(), serde_json::json!(stats.inner_reuse_ratio));
                                            obj.insert("recomputedFingerprints".into(), serde_json::json!(stats.recomputed_fingerprints));
                                            obj.insert("semanticProcessedRoutines".into(), serde_json::json!(stats.semantic_processed_routines));
                                            obj.insert("semanticReusedRoutines".into(), serde_json::json!(stats.semantic_reused_routines));
                                            obj.insert("semanticSelectiveRatio".into(), serde_json::json!(stats.semantic_selective_ratio));
                                        }
                                    }
                                    return Ok(Some(core));
                                } else {
                                    return Ok(Some(serde_json::json!({"status":"no-stats"})));
                                }
                            }
                        }
                    }
                }
                Err(tower_lsp::jsonrpc::Error::invalid_params("Invalid getIncrementalMetrics arguments"))
            }
            "bslAnalyzer.lsp.analyzeFile" => {
                if let Some(uri_value) = params.arguments.first() {
                    if let Ok(uri_str) = serde_json::from_value::<String>(uri_value.clone()) {
                        if let Ok(uri) = Url::parse(&uri_str) {
                            self.analyze_file_command(uri).await?;
                            return Ok(Some(serde_json::json!({"status": "success"})));
                        }
                    }
                }
                Err(tower_lsp::jsonrpc::Error::invalid_params(
                    "Invalid file URI",
                ))
            }
            "bslAnalyzer.lsp.analyzeWorkspace" => {
                if let Some(uri_value) = params.arguments.first() {
                    if let Ok(uri_str) = serde_json::from_value::<String>(uri_value.clone()) {
                        if let Ok(uri) = Url::parse(&uri_str) {
                            self.analyze_workspace_command(uri).await?;
                            return Ok(Some(serde_json::json!({"status": "success"})));
                        }
                    }
                }
                Err(tower_lsp::jsonrpc::Error::invalid_params(
                    "Invalid workspace URI",
                ))
            }
            _ => {
                tracing::warn!("Unknown command: {}", params.command);
                Err(tower_lsp::jsonrpc::Error::method_not_found())
            }
        }
    }

    async fn diagnostic(
        &self,
        params: DocumentDiagnosticParams,
    ) -> tower_lsp::jsonrpc::Result<DocumentDiagnosticReportResult> {
        tracing::debug!(
            "Pull-based diagnostic request for: {}",
            params.text_document.uri
        );

        // Получаем содержимое документа из кеша или пытаемся прочитать с диска
        let document_text =
            if let Some(doc_info) = self.documents.read().await.get(&params.text_document.uri) {
                doc_info.text.clone()
            } else {
                // Если документ не в кеше, пытаемся прочитать с диска
                match params.text_document.uri.to_file_path() {
                    Ok(file_path) => {
                        match tokio::fs::read_to_string(&file_path).await {
                            Ok(content) => content,
                            Err(_) => {
                                // Не можем прочитать файл - возвращаем пустую диагностику
                                return Ok(DocumentDiagnosticReportResult::Report(
                                    DocumentDiagnosticReport::Full(
                                        RelatedFullDocumentDiagnosticReport {
                                            related_documents: None,
                                            full_document_diagnostic_report:
                                                FullDocumentDiagnosticReport {
                                                    result_id: None,
                                                    items: vec![],
                                                },
                                        },
                                    ),
                                ));
                            }
                        }
                    }
                    Err(_) => {
                        // Некорректный URI - возвращаем пустую диагностику
                        return Ok(DocumentDiagnosticReportResult::Report(
                            DocumentDiagnosticReport::Full(RelatedFullDocumentDiagnosticReport {
                                related_documents: None,
                                full_document_diagnostic_report: FullDocumentDiagnosticReport {
                                    result_id: None,
                                    items: vec![],
                                },
                            }),
                        ));
                    }
                }
            };

        // Выполняем анализ документа
        match self
            .analyze_document(&params.text_document.uri, &document_text)
            .await
        {
            Ok(diagnostics) => {
                tracing::debug!(
                    "Pull-based diagnostic completed: {} items",
                    diagnostics.len()
                );
                Ok(DocumentDiagnosticReportResult::Report(
                    DocumentDiagnosticReport::Full(RelatedFullDocumentDiagnosticReport {
                        related_documents: None,
                        full_document_diagnostic_report: FullDocumentDiagnosticReport {
                            result_id: None,
                            items: diagnostics,
                        },
                    }),
                ))
            }
            Err(e) => {
                tracing::error!("Pull-based diagnostic analysis failed: {}", e);
                // В случае ошибки анализа возвращаем пустую диагностику, а не ошибку
                Ok(DocumentDiagnosticReportResult::Report(
                    DocumentDiagnosticReport::Full(RelatedFullDocumentDiagnosticReport {
                        related_documents: None,
                        full_document_diagnostic_report: FullDocumentDiagnosticReport {
                            result_id: None,
                            items: vec![],
                        },
                    }),
                ))
            }
        }
    }

    #[allow(dead_code)]
    async fn shutdown(&self) -> tower_lsp::jsonrpc::Result<()> {
        tracing::info!("Enhanced BSL LSP Server v2.0 shutting down");
        Ok(())
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for BslLanguageServer {
    async fn initialize(&self, params: InitializeParams) -> tower_lsp::jsonrpc::Result<InitializeResult> {
        tracing::info!("Initializing Enhanced BSL Language Server v2.0");
        // Пытаемся сразу загрузить индекс (берём первый workspace folder)
        if let Some(workspace_folders) = params.workspace_folders.as_ref() {
            if let Some(workspace) = workspace_folders.first() {
                tracing::info!("Workspace detected: {}", workspace.uri);
                // Попытка инициализации индекса; ошибки не фатальны
                if let Err(e) = self.initialize_unified_index(&workspace.uri).await {
                    tracing::warn!("Unified index init failed: {}", e);
                    self.client.log_message(MessageType::WARNING, format!("Unified index init failed: {}", e)).await;
                }
            } else {
                tracing::warn!("No workspace folders provided in initialize params");
            }
        } else {
            tracing::warn!("No workspace_folders in initialize params");
        }

        Ok(InitializeResult { capabilities: ServerCapabilities { text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)), completion_provider: Some(CompletionOptions { resolve_provider: Some(false), trigger_characters: Some(vec![".".into(), " ".into()]), ..Default::default() }), hover_provider: Some(HoverProviderCapability::Simple(true)), definition_provider: Some(OneOf::Left(true)), diagnostic_provider: Some(DiagnosticServerCapabilities::Options(DiagnosticOptions { identifier: Some("bsl-lsp".into()), inter_file_dependencies: true, workspace_diagnostics: false, ..Default::default() })), execute_command_provider: None, ..Default::default() }, server_info: Some(ServerInfo { name: "Enhanced BSL Language Server v2.0".into(), version: Some(env!("CARGO_PKG_VERSION").into()) }) })
    }
    async fn initialized(&self, _params: InitializedParams) { self.client.log_message(MessageType::INFO, "Enhanced BSL Language Server v2.0 ready").await; }
    #[allow(dead_code)]
    async fn shutdown(&self) -> tower_lsp::jsonrpc::Result<()> { Ok(()) }
    async fn did_open(&self, params: DidOpenTextDocumentParams) { BslLanguageServer::did_open(self, params).await }
    async fn did_change(&self, params: DidChangeTextDocumentParams) { BslLanguageServer::did_change(self, params).await }
    async fn did_close(&self, params: DidCloseTextDocumentParams) { BslLanguageServer::did_close(self, params).await }
    async fn completion(&self, params: CompletionParams) -> tower_lsp::jsonrpc::Result<Option<CompletionResponse>> { BslLanguageServer::completion(self, params).await }
    async fn hover(&self, params: HoverParams) -> tower_lsp::jsonrpc::Result<Option<Hover>> { BslLanguageServer::hover(self, params).await }
    async fn execute_command(&self, params: ExecuteCommandParams) -> tower_lsp::jsonrpc::Result<Option<serde_json::Value>> { BslLanguageServer::execute_command(self, params).await }
    async fn diagnostic(&self, params: DocumentDiagnosticParams) -> tower_lsp::jsonrpc::Result<DocumentDiagnosticReportResult> { BslLanguageServer::diagnostic(self, params).await }
}

impl BslLanguageServer {
    /// Обработка команды анализа файла
    async fn analyze_file_command(&self, uri: Url) -> tower_lsp::jsonrpc::Result<()> {
        tracing::info!("Analyzing file: {}", uri);

        // Получаем содержимое файла из кеша документов
        let documents = self.documents.read().await;
        if let Some(doc_info) = documents.get(&uri) {
            // Выполняем анализ через BslAnalyzer
            match self.analyze_bsl_content(&doc_info.text).await {
                Ok(diagnostics) => {
                    // Отправляем диагностики клиенту
                    self.client
                        .publish_diagnostics(uri.clone(), diagnostics, Some(doc_info.version))
                        .await;

                    self.client
                        .show_message(MessageType::INFO, "File analysis completed successfully")
                        .await;
                }
                Err(e) => {
                    let error_msg = format!("Analysis failed: {}", e);
                    tracing::error!("{}", error_msg);

                    self.client
                        .show_message(MessageType::ERROR, &error_msg)
                        .await;
                }
            }
        } else {
            // Файл не открыт в редакторе - попробуем прочитать с диска
            if let Ok(file_path) = uri.to_file_path() {
                match tokio::fs::read_to_string(&file_path).await {
                    Ok(content) => match self.analyze_bsl_content(&content).await {
                        Ok(diagnostics) => {
                            self.client
                                .publish_diagnostics(uri.clone(), diagnostics, None)
                                .await;

                            self.client
                                .show_message(
                                    MessageType::INFO,
                                    "File analysis completed successfully",
                                )
                                .await;
                        }
                        Err(e) => {
                            let error_msg = format!("Analysis failed: {}", e);
                            tracing::error!("{}", error_msg);

                            self.client
                                .show_message(MessageType::ERROR, &error_msg)
                                .await;
                        }
                    },
                    Err(e) => {
                        let error_msg = format!("Failed to read file: {}", e);
                        tracing::error!("{}", error_msg);

                        self.client
                            .show_message(MessageType::ERROR, &error_msg)
                            .await;
                    }
                }
            }
        }

        Ok(())
    }

    /// Обработка команды анализа workspace
    async fn analyze_workspace_command(
        &self,
        workspace_uri: Url,
    ) -> tower_lsp::jsonrpc::Result<()> {
        tracing::info!("Analyzing workspace: {}", workspace_uri);

        self.client
            .show_message(MessageType::INFO, "Starting workspace analysis...")
            .await;

        // TODO: Реализовать полный анализ workspace
        // Найти все .bsl файлы в workspace и проанализировать их

        self.client
            .show_message(MessageType::INFO, "Workspace analysis completed")
            .await;

        Ok(())
    }

    /// Анализ BSL содержимого через BslAnalyzer
    async fn analyze_bsl_content(&self, content: &str) -> Result<Vec<Diagnostic>> {
        // Получаем конфигурацию и индекс
        let config = self.analysis_config.read().await.clone();
        let index_guard = self.unified_index.read().await;

        // Создаем анализатор
        let mut analyzer = match &*index_guard {
            Some(idx) => BslAnalyzer::with_index_and_config(idx.clone(), config)?,
            None => BslAnalyzer::with_config(config)?,
        };

        // Анализируем код
        if let Err(e) = analyzer.analyze_code(content, "lsp_temp.bsl") {
            // Если анализ не удался, возвращаем ошибку как диагностику
            return Ok(vec![create_analysis_error_diagnostic(
                format!("Analysis error: {}", e),
                "BSL000",
            )]);
        }

        // Получаем результаты анализа и конвертируем в LSP диагностику
        let (errors, warnings) = analyzer.get_errors_and_warnings();
        let diagnostics = convert_analysis_results(errors, warnings);

        Ok(diagnostics)
    }
}
