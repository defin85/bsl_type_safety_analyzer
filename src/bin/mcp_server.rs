/// MCP сервер для BSL Type Safety Analyzer
/// 
/// Реализован через простой JSON-RPC протокол без внешних MCP библиотек
/// из-за нестабильности API в существующих Rust MCP SDK.
/// 
/// Поддерживает:
/// - Инициализацию протокола
/// - Получение списка инструментов
/// - Поиск типов в UnifiedBslIndex
/// 
/// Для запуска: cargo run --bin bsl-mcp-server
/// Тестирование: node test_mcp_server.js

use bsl_analyzer::unified_index::{UnifiedBslIndex, UnifiedIndexBuilder, BslApplicationMode};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing_subscriber::EnvFilter;

#[derive(Debug)]
pub struct BslMcpServer {
    index: Arc<RwLock<Option<UnifiedBslIndex>>>,
    platform_version: String,
}

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Option<Value>,
    id: Value,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    result: Option<Value>,
    error: Option<JsonRpcError>,
    id: Value,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    data: Option<Value>,
}

impl BslMcpServer {
    pub fn new() -> Self {
        Self {
            index: Arc::new(RwLock::new(None)),
            platform_version: env::var("BSL_PLATFORM_VERSION").unwrap_or_else(|_| "8.3.25".to_string()),
        }
    }
    
    pub async fn initialize(&self) -> Result<()> {
        let config_path = env::var("BSL_CONFIG_PATH").ok().map(PathBuf::from);
        
        if let Some(ref path) = config_path {
            eprintln!("Loading BSL index from: {:?}", path);
            self.load_index(path.to_str().unwrap_or_default()).await?;
        } else {
            eprintln!("No BSL_CONFIG_PATH set, index will be loaded on demand");
        }
        
        Ok(())
    }
    
    async fn load_index(&self, config_path: &str) -> Result<()> {
        let builder = UnifiedIndexBuilder::new()?
            .with_application_mode(BslApplicationMode::ManagedApplication);
        
        let index = builder.build_index(config_path, &self.platform_version)?;
        
        eprintln!("Loaded BSL index with {} types", index.get_entity_count());
        
        let mut guard = self.index.write().await;
        *guard = Some(index);
        
        Ok(())
    }
    
    async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        match request.method.as_str() {
            "initialize" => {
                let result = json!({
                    "protocolVersion": "0.1.0",
                    "serverInfo": {
                        "name": "BSL Type Safety Analyzer",
                        "version": "1.0.0"
                    },
                    "capabilities": {
                        "tools": true
                    }
                });
                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: Some(result),
                    error: None,
                    id: request.id,
                }
            }
            "tools/list" => {
                let tools = json!({
                    "tools": [
                        {
                            "name": "find_type",
                            "description": "Поиск типа в индексе BSL",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "type_name": {
                                        "type": "string",
                                        "description": "Имя типа для поиска"
                                    }
                                },
                                "required": ["type_name"]
                            }
                        },
                        {
                            "name": "analyze_code",
                            "description": "Полный анализ BSL кода с диагностикой ошибок",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "bsl_code": {
                                        "type": "string",
                                        "description": "BSL код для анализа"
                                    },
                                    "file_name": {
                                        "type": "string",
                                        "description": "Имя файла (опционально)"
                                    }
                                },
                                "required": ["bsl_code"]
                            }
                        },
                        {
                            "name": "validate_syntax",
                            "description": "Проверка синтаксиса BSL кода",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "bsl_code": {
                                        "type": "string",
                                        "description": "BSL код для проверки синтаксиса"
                                    }
                                },
                                "required": ["bsl_code"]
                            }
                        },
                        {
                            "name": "get_suggestions",
                            "description": "Получение предложений для исправления кода",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "error_code": {
                                        "type": "string",
                                        "description": "Код ошибки BSL"
                                    },
                                    "context": {
                                        "type": "string",
                                        "description": "Контекст ошибки"
                                    }
                                },
                                "required": ["error_code"]
                            }
                        }
                    ]
                });
                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: Some(tools),
                    error: None,
                    id: request.id,
                }
            }
            "tools/call" => {
                let params = request.params.unwrap_or(json!({}));
                let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let empty_args = json!({});
                let args = params.get("arguments").unwrap_or(&empty_args);
                
                match tool_name {
                    "find_type" => {
                        let type_name = args.get("type_name").and_then(|v| v.as_str()).unwrap_or("");
                        let result = self.find_type(type_name).await;
                        JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: Some(json!({
                                "content": [{
                                    "type": "text",
                                    "text": result
                                }]
                            })),
                            error: None,
                            id: request.id,
                        }
                    }
                    "analyze_code" => {
                        let bsl_code = args.get("bsl_code").and_then(|v| v.as_str()).unwrap_or("");
                        let file_name = args.get("file_name").and_then(|v| v.as_str()).unwrap_or("code.bsl");
                        let result = self.analyze_code(bsl_code, file_name).await;
                        JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: Some(json!({
                                "content": [{
                                    "type": "text",
                                    "text": result
                                }]
                            })),
                            error: None,
                            id: request.id,
                        }
                    }
                    "validate_syntax" => {
                        let bsl_code = args.get("bsl_code").and_then(|v| v.as_str()).unwrap_or("");
                        let result = self.validate_syntax(bsl_code).await;
                        JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: Some(json!({
                                "content": [{
                                    "type": "text",
                                    "text": result
                                }]
                            })),
                            error: None,
                            id: request.id,
                        }
                    }
                    "get_suggestions" => {
                        let error_code = args.get("error_code").and_then(|v| v.as_str()).unwrap_or("");
                        let context = args.get("context").and_then(|v| v.as_str()).unwrap_or("");
                        let result = self.get_suggestions(error_code, context).await;
                        JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: Some(json!({
                                "content": [{
                                    "type": "text",
                                    "text": result
                                }]
                            })),
                            error: None,
                            id: request.id,
                        }
                    }
                    _ => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32601,
                            message: format!("Unknown tool: {}", tool_name),
                            data: None,
                        }),
                        id: request.id,
                    }
                }
            }
            _ => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(JsonRpcError {
                    code: -32601,
                    message: format!("Method not found: {}", request.method),
                    data: None,
                }),
                id: request.id,
            }
        }
    }
    
    async fn find_type(&self, type_name: &str) -> String {
        let guard = self.index.read().await;
        let index = match &*guard {
            Some(idx) => idx,
            None => return json!({
                "error": "Индекс не загружен"
            }).to_string(),
        };
        
        if let Some(entity) = index.find_entity(type_name) {
            json!({
                "found": true,
                "entity": {
                    "id": entity.id.0,
                    "display_name": entity.display_name,
                    "entity_type": format!("{:?}", entity.entity_type),
                    "entity_kind": format!("{:?}", entity.entity_kind),
                    "methods_count": entity.interface.methods.len(),
                    "properties_count": entity.interface.properties.len(),
                    "tabular_sections": entity.relationships.tabular_sections.iter().map(|ts| {
                        json!({
                            "name": ts.name,
                            "display_name": ts.display_name,
                            "attributes_count": ts.attributes.len()
                        })
                    }).collect::<Vec<_>>(),
                }
            }).to_string()
        } else {
            let suggestions = index.suggest_similar_names(type_name);
            json!({
                "found": false,
                "suggestions": suggestions.into_iter().take(5).collect::<Vec<_>>()
            }).to_string()
        }
    }
    
    async fn analyze_code(&self, bsl_code: &str, file_name: &str) -> String {
        use bsl_analyzer::BslAnalyzer;
        use std::fs;
        
        // Создаем временный файл для анализа
        let temp_file = format!("temp_{}", file_name);
        if let Err(e) = fs::write(&temp_file, bsl_code) {
            return json!({
                "error": format!("Не удалось создать временный файл: {}", e)
            }).to_string();
        }
        
        let guard = self.index.read().await;
        let index = match &*guard {
            Some(idx) => idx,
            None => {
                let _ = fs::remove_file(&temp_file);
                return json!({
                    "error": "Индекс не загружен"
                }).to_string();
            }
        };
        
        // Создаем анализатор с UnifiedBslIndex
        let result = match BslAnalyzer::with_index(index.clone()) {
            Ok(mut analyzer) => {
                match analyzer.analyze_code(bsl_code, file_name) {
                    Ok(()) => {
                        let (errors, warnings) = analyzer.get_errors_and_warnings();
                        
                        let errors_json: Vec<_> = errors.into_iter().map(|err| {
                            json!({
                                "line": err.position.line,
                                "column": err.position.column,
                                "code": err.error_code.unwrap_or_else(|| "BSL000".to_string()),
                                "message": err.message,
                                "severity": "error"
                            })
                        }).collect();
                        
                        let warnings_json: Vec<_> = warnings.into_iter().map(|warn| {
                            json!({
                                "line": warn.position.line,
                                "column": warn.position.column,
                                "code": warn.error_code.unwrap_or_else(|| "BSL000".to_string()),
                                "message": warn.message,
                                "severity": "warning"
                            })
                        }).collect();
                        
                        json!({
                            "success": true,
                            "file_name": file_name,
                            "analysis": {
                                "errors_count": errors_json.len(),
                                "warnings_count": warnings_json.len(),
                                "errors": errors_json,
                                "warnings": warnings_json,
                                "has_issues": !errors_json.is_empty() || !warnings_json.is_empty()
                            }
                        }).to_string()
                    }
                    Err(e) => {
                        json!({
                            "error": format!("Ошибка анализа: {}", e)
                        }).to_string()
                    }
                }
            }
            Err(e) => {
                json!({
                    "error": format!("Не удалось создать анализатор: {}", e)
                }).to_string()
            }
        };
        
        // Удаляем временный файл
        let _ = fs::remove_file(&temp_file);
        
        result
    }
    
    async fn validate_syntax(&self, bsl_code: &str) -> String {
        use bsl_analyzer::parser::BslLexer;
        
        let lexer = BslLexer::new();
        match lexer.tokenize(bsl_code) {
            Ok(tokens) => {
                json!({
                    "valid": true,
                    "tokens_count": tokens.len(),
                    "message": "Синтаксис корректен"
                }).to_string()
            }
            Err(e) => {
                json!({
                    "valid": false,
                    "error": format!("Синтаксическая ошибка: {}", e),
                    "message": "Обнаружены ошибки синтаксиса"
                }).to_string()
            }
        }
    }
    
    async fn get_suggestions(&self, error_code: &str, context: &str) -> String {
        let suggestions = match error_code {
            "BSL001" => vec![
                "Проверьте правильность написания имени переменной",
                "Убедитесь что переменная объявлена перед использованием"
            ],
            "BSL003" => vec![
                "Проверьте правильность написания имени функции",
                "Убедитесь что функция доступна в текущем контексте",
                "Проверьте подключение модулей с нужными функциями"
            ],
            "BSL007" => vec![
                "Объявите переменную с помощью ключевого слова 'Перем'",
                "Инициализируйте переменную перед использованием",
                "Проверьте область видимости переменной"
            ],
            _ => vec![
                "Проверьте документацию по ошибке",
                "Убедитесь в корректности синтаксиса BSL",
                "Обратитесь к справочной системе 1С"
            ]
        };
        
        json!({
            "error_code": error_code,
            "context": context,
            "suggestions": suggestions
        }).to_string()
    }
    
    pub async fn run(&self) -> Result<()> {
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();
        let mut reader = BufReader::new(stdin);
        let mut stdout = stdout;
        
        let mut line = String::new();
        
        loop {
            line.clear();
            let n = reader.read_line(&mut line).await?;
            if n == 0 {
                break; // EOF
            }
            
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            
            eprintln!("Received: {}", line);
            
            match serde_json::from_str::<JsonRpcRequest>(line) {
                Ok(request) => {
                    let response = self.handle_request(request).await;
                    let response_str = serde_json::to_string(&response)?;
                    stdout.write_all(response_str.as_bytes()).await?;
                    stdout.write_all(b"\n").await?;
                    stdout.flush().await?;
                    eprintln!("Sent: {}", response_str);
                }
                Err(e) => {
                    eprintln!("Failed to parse request: {}", e);
                    let error_response = JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32700,
                            message: "Parse error".to_string(),
                            data: None,
                        }),
                        id: json!(null),
                    };
                    let response_str = serde_json::to_string(&error_response)?;
                    stdout.write_all(response_str.as_bytes()).await?;
                    stdout.write_all(b"\n").await?;
                    stdout.flush().await?;
                }
            }
        }
        
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Setup logging to stderr
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();
    
    eprintln!("Starting BSL MCP Server (Simple JSON-RPC)...");
    
    let server = BslMcpServer::new();
    server.initialize().await?;
    
    eprintln!("BSL MCP Server initialized. Waiting for JSON-RPC requests...");
    
    server.run().await?;
    
    Ok(())
}