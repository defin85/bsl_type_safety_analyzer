/// <module>
///   <name>analyzer</name>
///   <purpose>Основная реализация MCP сервера для BSL</purpose>
/// </module>

use crate::unified_index::{UnifiedBslIndex, UnifiedIndexBuilder, BslApplicationMode};
use crate::mcp_server::types::{McpError, McpResult};
// Временно отключено до миграции на новую версию MCP SDK
/*
use rmcp::{
    model::{ServerCapabilities, ServerInfo},
    handler::server::ServerHandler,
};
*/
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// <type>
///   <name>BslTypeAnalyzer</name>
///   <purpose>MCP сервер для анализа BSL типов</purpose>
///   <description>
///     Предоставляет инструменты для LLM для работы с типовой системой BSL.
///     Автоматически загружает индекс из переменных окружения или параметров.
///   </description>
/// </type>
#[derive(Debug, Clone)]
pub struct BslTypeAnalyzer {
    index: Arc<RwLock<Option<UnifiedBslIndex>>>,
    _config_path: Option<PathBuf>,
    platform_version: String,
}

impl BslTypeAnalyzer {
    /// <method>
    ///   <name>new</name>
    ///   <purpose>Создание нового экземпляра MCP сервера</purpose>
    /// </method>
    pub async fn new() -> McpResult<Self> {
        // Читаем параметры из окружения
        let _config_path = env::var("BSL_CONFIG_PATH").ok().map(PathBuf::from);
        let platform_version = env::var("BSL_PLATFORM_VERSION").unwrap_or_else(|_| "8.3.25".to_string());
        
        let analyzer = Self {
            index: Arc::new(RwLock::new(None)),
            _config_path: _config_path.clone(),
            platform_version: platform_version.clone(),
        };
        
        // Пытаемся загрузить индекс если есть конфигурация
        if let Some(ref path) = _config_path {
            eprintln!("Loading BSL index from: {:?}", path);
            analyzer.load_index(path.to_str().unwrap_or_default()).await?;
        } else {
            eprintln!("No BSL_CONFIG_PATH set, index will be loaded on demand");
        }
        
        Ok(analyzer)
    }
    
    async fn load_index(&self, config_path: &str) -> McpResult<()> {
        let builder = UnifiedIndexBuilder::new()
            .map_err(|e| McpError::Internal(e.to_string()))?
            .with_application_mode(BslApplicationMode::ManagedApplication);
        
        let index = builder.build_index(config_path, &self.platform_version)
            .map_err(|e| McpError::Internal(e.to_string()))?;
        
        info!("Loaded BSL index with {} types", index.get_entity_count());
        
        let mut guard = self.index.write().await;
        *guard = Some(index);
        
        Ok(())
    }
    
    pub async fn ensure_index(&self) -> McpResult<()> {
        let guard = self.index.read().await;
        if guard.is_none() {
            return Err(McpError::IndexNotLoaded);
        }
        Ok(())
    }
    
    pub fn get_index(&self) -> &Arc<RwLock<Option<UnifiedBslIndex>>> {
        &self.index
    }
}

// Методы для MCP интеграции (временно отключены)
impl BslTypeAnalyzer {
    /// <tool>
    ///   <name>load_configuration</name>
    ///   <description>Загружает индекс BSL типов из конфигурации</description>
    ///   <parameters>
    ///     <param name="config_path">Путь к конфигурации 1С</param>
    ///     <param name="platform_version">Версия платформы (по умолчанию 8.3.25)</param>
    ///   </parameters>
    /// </tool>
    // #[tool(description = "Загружает индекс BSL типов из конфигурации 1С")]
    pub async fn load_configuration(
        &self, 
        // #[tool(param)] 
        config_path: String,
        // #[tool(param)] 
        platform_version: Option<String>
    ) -> String {
        let version = platform_version.unwrap_or_else(|| self.platform_version.clone());
        
        // Версия обновится при загрузке индекса
        
        match self.load_index(&config_path).await {
            Ok(_) => {
                let guard = self.index.read().await;
                if let Some(ref index) = *guard {
                    format!(
                        "Конфигурация загружена успешно. Индекс содержит {} типов (платформа {}).",
                        index.get_entity_count(),
                        version
                    )
                } else {
                    "Ошибка: индекс не загружен".to_string()
                }
            }
            Err(e) => format!("Ошибка загрузки конфигурации: {}", e),
        }
    }
    
    /// <tool>
    ///   <name>find_type</name>
    ///   <description>Поиск типа в индексе BSL</description>
    ///   <parameters>
    ///     <param name="type_name">Имя типа для поиска (русское или английское)</param>
    ///     <param name="language_preference">Языковое предпочтение: russian, english, auto</param>
    ///   </parameters>
    /// </tool>
    // #[tool(description = "Поиск типа в индексе BSL. Поддерживает русские и английские названия.")]
    pub async fn find_type(
        &self,
        // #[tool(param)] 
        type_name: String,
        // #[tool(param)] 
        language_preference: Option<String>
    ) -> String {
        crate::mcp_server::tools::find_type_impl(self, type_name, language_preference).await
    }
    
    /// <tool>
    ///   <name>get_type_methods</name>
    ///   <description>Получить все методы типа включая унаследованные</description>
    ///   <parameters>
    ///     <param name="type_name">Имя типа</param>
    ///     <param name="include_inherited">Включить унаследованные методы (по умолчанию true)</param>
    ///     <param name="filter_context">Фильтр по контексту: Client, Server, All</param>
    ///   </parameters>
    /// </tool>
    // #[tool(description = "Получить все методы типа включая унаследованные")]
    pub async fn get_type_methods(
        &self,
        // #[tool(param)] 
        type_name: String,
        // #[tool(param)] 
        include_inherited: Option<bool>,
        // #[tool(param)] 
        filter_context: Option<String>
    ) -> String {
        crate::mcp_server::tools::get_type_methods_impl(self, type_name, include_inherited, filter_context).await
    }
    
    /// <tool>
    ///   <name>check_type_compatibility</name>
    ///   <description>Проверить совместимость типов для присваивания</description>
    ///   <parameters>
    ///     <param name="from_type">Исходный тип (что присваиваем)</param>
    ///     <param name="to_type">Целевой тип (куда присваиваем)</param>
    ///   </parameters>
    /// </tool>
    // #[tool(description = "Проверить совместимость типов для присваивания")]
    pub async fn check_type_compatibility(
        &self,
        // #[tool(param)] 
        from_type: String,
        // #[tool(param)] 
        to_type: String
    ) -> String {
        crate::mcp_server::tools::check_type_compatibility_impl(self, from_type, to_type).await
    }
    
    /// <tool>
    ///   <name>validate_method_call</name>
    ///   <description>Проверить корректность вызова метода</description>
    ///   <parameters>
    ///     <param name="object_type">Тип объекта, у которого вызывается метод</param>
    ///     <param name="method_name">Имя вызываемого метода</param>
    ///     <param name="context">Контекст выполнения: Client, Server (по умолчанию Server)</param>
    ///   </parameters>
    /// </tool>
    // #[tool(description = "Проверить корректность вызова метода")]
    pub async fn validate_method_call(
        &self,
        // #[tool(param)] 
        object_type: String,
        // #[tool(param)] 
        method_name: String,
        // #[tool(param)] 
        context: Option<String>
    ) -> String {
        crate::mcp_server::tools::validate_method_call_impl(self, object_type, method_name, context).await
    }
}

// Реализация ServerHandler временно отключена до миграции на новую версию MCP SDK
/*
impl ServerHandler for BslTypeAnalyzer {
    fn get_info(&self) -> ServerInfo {
        eprintln!("ServerHandler::get_info called");
        ServerInfo {
            instructions: Some(
                "Сервер для анализа типов BSL (1C:Enterprise). \
                Предоставляет информацию о типах, методах, совместимости типов и валидации кода. \
                Используйте BSL_CONFIG_PATH и BSL_PLATFORM_VERSION для автоматической загрузки."
                .to_string()
            ),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            ..Default::default()
        }
    }
}
*/