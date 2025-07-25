/*!
# Dependency Analyzer

Анализатор зависимостей между BSL модулями для выявления 
циклических зависимостей и построения графа связей.

## Возможности
- Построение графа зависимостей между модулями
- Обнаружение циклических зависимостей
- Анализ экспорта/импорта между модулями
- Поиск неиспользуемых экспортных процедур
- Валидация зависимостей с учетом контекста (клиент/сервер)

## Использование

```rust
let analyzer = DependencyAnalyzer::new();
let dependencies = analyzer.analyze_configuration(&config)?;

// Проверяем циклические зависимости
if let Some(cycles) = dependencies.find_cycles() {
    println!("Found circular dependencies: {:?}", cycles);
}
```
*/

use std::collections::{HashMap, HashSet, VecDeque};
use serde::{Deserialize, Serialize};
use anyhow::{Context, Result};
use crate::configuration::{Configuration, BslModule};
use crate::parser::{BslParser, ast::AstNode};

/// Анализатор зависимостей между модулями
pub struct DependencyAnalyzer {
    parser: BslParser,
}

/// Граф зависимостей конфигурации
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    /// Узлы графа (модули)
    pub nodes: HashMap<String, DependencyNode>, 
    /// Рёбра графа (зависимости)
    pub edges: Vec<DependencyEdge>,
    /// Обнаруженные циклические зависимости
    pub cycles: Vec<DependencyCycle>,
    /// Статистика анализа
    pub statistics: DependencyStatistics,
}

/// Узел графа зависимостей (модуль)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyNode {
    /// Имя модуля
    pub module_name: String,
    /// Тип модуля
    pub module_type: ModuleType,
    /// Экспортируемые символы
    pub exports: Vec<ExportedSymbol>,
    /// Импортируемые символы
    pub imports: Vec<ImportedSymbol>,
    /// Контекст выполнения модуля
    pub execution_context: ExecutionContext,
}

/// Ребро графа зависимостей
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEdge {
    /// Модуль-источник
    pub from: String,
    /// Модуль-назначение
    pub to: String,
    /// Тип зависимости
    pub dependency_type: DependencyType,
    /// Символы, которые используются
    pub used_symbols: Vec<String>,
    /// Позиции в коде, где происходит использование
    pub usage_positions: Vec<UsagePosition>,
}

/// Циклическая зависимость
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyCycle {
    /// Модули, участвующие в цикле
    pub modules: Vec<String>,
    /// Рёбра, образующие цикл
    pub edges: Vec<DependencyEdge>,
    /// Серьёзность проблемы
    pub severity: CycleSeverity,
}

/// Экспортируемый символ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedSymbol {
    /// Имя символа
    pub name: String,
    /// Тип символа
    pub symbol_type: SymbolType,
    /// Контекст, в котором доступен символ
    pub context: ExecutionContext,
    /// Описание символа (из комментариев)
    pub description: Option<String>,
    /// Позиция в коде
    pub position: UsagePosition,
}

/// Импортируемый символ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportedSymbol {
    /// Имя символа
    pub name: String,
    /// Модуль, из которого импортируется
    pub source_module: String,
    /// Тип символа
    pub symbol_type: SymbolType,
    /// Позиции использования в коде
    pub usage_positions: Vec<UsagePosition>,
}

/// Позиция использования в коде
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsagePosition {
    pub line: usize,
    pub column: usize,
    pub length: usize,
}

/// Тип модуля 1С
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModuleType {
    CommonModule,          // ОбщийМодуль
    ObjectModule,          // МодульОбъекта
    ManagerModule,         // МодульМенеджера
    FormModule,            // МодульФормы
    CommandModule,         // МодульКоманды
    ApplicationModule,     // МодульПриложения
    SessionModule,         // МодульСеанса
    ExternalConnectionModule, // МодульВнешнегоСоединения
}

/// Тип зависимости
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DependencyType {
    DirectCall,            // Прямой вызов процедуры/функции
    ObjectAccess,          // Доступ к объекту конфигурации
    EventSubscription,     // Подписка на событие
    DataExchange,          // Обмен данными
    ConfigurationReference, // Ссылка на элемент конфигурации
}

/// Тип символа
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SymbolType {
    Procedure,             // Процедура
    Function,              // Функция
    Variable,              // Переменная
    Constant,              // Константа
    Type,                  // Тип данных
}

/// Контекст выполнения
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecutionContext {
    Server,                // Сервер
    Client,                // Клиент
    ClientServer,          // Клиент-Сервер
    External,              // Внешнее соединение
    Unknown,               // Неизвестный контекст
}

/// Серьёзность циклической зависимости
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CycleSeverity {
    Critical,              // Критическая (может вызвать ошибки)
    Warning,               // Предупреждение (потенциальная проблема)
    Info,                  // Информация (архитектурная проблема)
}

/// Статистика зависимостей
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyStatistics {
    /// Общее количество модулей
    pub total_modules: usize,
    /// Количество зависимостей
    pub total_dependencies: usize,
    /// Количество циклических зависимостей
    pub circular_dependencies: usize,
    /// Количество неиспользуемых экспортов
    pub unused_exports: usize,
    /// Максимальная глубина зависимостей
    pub max_depth: usize,
    /// Среднее количество зависимостей на модуль
    pub avg_dependencies_per_module: f64,
}

impl DependencyAnalyzer {
    /// Создает новый анализатор зависимостей
    pub fn new() -> Self {
        Self {
            parser: BslParser::new(),
        }
    }
    
    /// Анализирует зависимости в конфигурации
    pub fn analyze_configuration(&self, config: &Configuration) -> Result<DependencyGraph> {
        tracing::info!("Starting dependency analysis for {} modules", config.get_modules().len());
        
        let mut nodes = HashMap::new();
        let mut edges = Vec::new();
        
        // Фаза 1: Анализируем каждый модуль для построения узлов
        for module in config.get_modules() {
            let node = self.analyze_module(module).context("Failed to analyze module")?;
            nodes.insert(module.name.clone(), node);
        }
        
        // Фаза 2: Анализируем зависимости между модулями
        for module in config.get_modules() {
            let module_edges = self.find_module_dependencies(module, &nodes)
                .context("Failed to find module dependencies")?;
            edges.extend(module_edges);
        }
        
        // Фаза 3: Ищем циклические зависимости
        let cycles = self.find_cycles(&nodes, &edges);
        
        // Фаза 4: Собираем статистику
        let statistics = self.calculate_statistics(&nodes, &edges, &cycles);
        
        tracing::info!("Dependency analysis completed: {} nodes, {} edges, {} cycles", 
            nodes.len(), edges.len(), cycles.len());
        
        Ok(DependencyGraph {
            nodes,
            edges,
            cycles,
            statistics,
        })
    }
    
    /// Анализирует отдельный модуль
    fn analyze_module(&self, module: &BslModule) -> Result<DependencyNode> {
        let content = std::fs::read_to_string(&module.path)
            .with_context(|| format!("Failed to read module: {}", module.path.display()))?;
        
        // Парсим модуль
        let ast = self.parser.parse_text(&content)
            .with_context(|| format!("Failed to parse module: {}", module.name))?;
        
        // Извлекаем экспорты
        let exports = self.extract_exports(&ast, &module.name)?;
        
        // Извлекаем импорты  
        let imports = self.extract_imports(&ast, &module.name)?;
        
        // Определяем тип модуля
        let module_type = self.determine_module_type(&module.path, &content);
        
        // Определяем контекст выполнения
        let execution_context = self.determine_execution_context(&content, &module_type);
        
        Ok(DependencyNode {
            module_name: module.name.clone(),
            module_type,
            exports,
            imports,
            execution_context,
        })
    }
    
    /// Извлекает экспортируемые символы из AST
    fn extract_exports(&self, ast: &AstNode, module_name: &str) -> Result<Vec<ExportedSymbol>> {
        let mut exports = Vec::new();
        
        // Рекурсивно обходим AST в поисках экспортных процедур/функций
        self.visit_ast_for_exports(ast, &mut exports, module_name)?;
        
        Ok(exports)
    }
    
    /// Рекурсивно обходит AST для поиска экспортов
    fn visit_ast_for_exports(&self, node: &AstNode, exports: &mut Vec<ExportedSymbol>, _module_name: &str) -> Result<()> {
        // Ищем узлы процедур и функций с модификатором "Экспорт"
        if matches!(node.node_type, crate::parser::ast::AstNodeType::Procedure | crate::parser::ast::AstNodeType::Function) {
            if let Some(export_info) = self.check_export_node(node)? {
                exports.push(export_info);
            }
        }
        
        // Рекурсивно обрабатываем дочерние узлы
        for child in &node.children {
            self.visit_ast_for_exports(child, exports, _module_name)?;
        }
        
        Ok(())
    }
    
    /// Проверяет узел на экспорт и извлекает информацию
    fn check_export_node(&self, node: &AstNode) -> Result<Option<ExportedSymbol>> {
        // Простая реализация - ищем "Экспорт" в значении узла или атрибутах
        let node_text = node.value.as_deref().unwrap_or("");
        let has_export = node_text.contains("Экспорт") || 
                        node.attributes.values().any(|v| v.contains("Экспорт"));
        
        if has_export {
            let symbol_type = match node.node_type {
                crate::parser::ast::AstNodeType::Procedure => SymbolType::Procedure,
                crate::parser::ast::AstNodeType::Function => SymbolType::Function,
                _ => SymbolType::Function, // Default
            };
            
            // Извлекаем имя процедуры/функции
            let name = self.extract_symbol_name(node_text)?;
            
            return Ok(Some(ExportedSymbol {
                name,
                symbol_type,
                context: ExecutionContext::ClientServer, // Default
                description: None,
                position: UsagePosition {
                    line: node.span.start.line,
                    column: node.span.start.column,
                    length: node_text.len(),
                },
            }));
        }
        
        Ok(None)
    }
    
    /// Извлекает имя символа из текста
    fn extract_symbol_name(&self, text: &str) -> Result<String> {
        // Простой парсинг имени процедуры/функции
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.len() >= 2 {
            let name = words[1].split('(').next().unwrap_or("Unknown");
            Ok(name.to_string())
        } else {
            Ok("Unknown".to_string())
        }
    }
    
    /// Извлекает импортируемые символы из AST
    fn extract_imports(&self, ast: &AstNode, _module_name: &str) -> Result<Vec<ImportedSymbol>> {
        let mut imports = Vec::new();
        
        // Ищем вызовы внешних процедур/функций
        self.visit_ast_for_imports(ast, &mut imports)?;
        
        Ok(imports)
    }
    
    /// Рекурсивно обходит AST для поиска импортов
    fn visit_ast_for_imports(&self, node: &AstNode, imports: &mut Vec<ImportedSymbol>) -> Result<()> {
        // Ищем узлы вызовов методов
        if matches!(node.node_type, crate::parser::ast::AstNodeType::CallExpression | crate::parser::ast::AstNodeType::MemberExpression) {
            if let Some(import_info) = self.check_import_node(node)? {
                imports.push(import_info);
            }
        }
        
        // Рекурсивно обрабатываем дочерние узлы
        for child in &node.children {
            self.visit_ast_for_imports(child, imports)?;
        }
        
        Ok(())
    }
    
    /// Проверяет узел на импорт и извлекает информацию
    fn check_import_node(&self, node: &AstNode) -> Result<Option<ImportedSymbol>> {
        // Простая эвристика - если вызов содержит точку, то это может быть внешний вызов
        let node_text = node.value.as_deref().unwrap_or("");
        if node_text.contains('.') && !node_text.contains("\"") {
            let parts: Vec<&str> = node_text.split('.').collect();
            if parts.len() >= 2 {
                let source_module = parts[0].trim().to_string();
                let symbol_name = parts[1].split('(').next().unwrap_or("Unknown").trim().to_string();
                
                return Ok(Some(ImportedSymbol {
                    name: symbol_name,
                    source_module,
                    symbol_type: SymbolType::Function, // Default assumption
                    usage_positions: vec![UsagePosition {
                        line: node.span.start.line,
                        column: node.span.start.column,
                        length: node_text.len(),
                    }],
                }));
            }
        }
        
        Ok(None)
    }
    
    /// Определяет тип модуля по пути и содержимому
    fn determine_module_type(&self, path: &std::path::Path, _content: &str) -> ModuleType {
        let path_str = path.to_string_lossy().to_lowercase();
        
        if path_str.contains("commonmodules") {
            ModuleType::CommonModule
        } else if path_str.contains("forms") {
            ModuleType::FormModule
        } else if path_str.contains("commands") {
            ModuleType::CommandModule
        } else if path_str.contains("objects") {
            ModuleType::ObjectModule
        } else {
            ModuleType::CommonModule // Default
        }
    }
    
    /// Определяет контекст выполнения модуля
    fn determine_execution_context(&self, content: &str, module_type: &ModuleType) -> ExecutionContext {
        match module_type {
            ModuleType::CommonModule => {
                // Анализируем директивы компиляции
                if content.contains("&НаСервере") || content.contains("&AtServer") {
                    ExecutionContext::Server
                } else if content.contains("&НаКлиенте") || content.contains("&AtClient") {
                    ExecutionContext::Client
                } else if content.contains("&НаСервереБезКонтекста") || content.contains("&AtServerNoContext") {
                    ExecutionContext::Server
                } else {
                    ExecutionContext::ClientServer // Default for common modules
                }
            },
            ModuleType::FormModule => ExecutionContext::Client,
            ModuleType::ObjectModule => ExecutionContext::Server,
            ModuleType::ManagerModule => ExecutionContext::Server,
            _ => ExecutionContext::Unknown,
        }
    }
    
    /// Находит зависимости модуля
    fn find_module_dependencies(&self, module: &BslModule, nodes: &HashMap<String, DependencyNode>) -> Result<Vec<DependencyEdge>> {
        let mut edges = Vec::new();
        
        if let Some(current_node) = nodes.get(&module.name) {
            // Для каждого импорта создаем ребро зависимости
            for import in &current_node.imports {
                // Проверяем, существует ли модуль-источник
                if nodes.contains_key(&import.source_module) {
                    let edge = DependencyEdge {
                        from: module.name.clone(),
                        to: import.source_module.clone(),
                        dependency_type: DependencyType::DirectCall,
                        used_symbols: vec![import.name.clone()],
                        usage_positions: import.usage_positions.clone(),
                    };
                    edges.push(edge);
                }
            }
        }
        
        Ok(edges)
    }
    
    /// Ищет циклические зависимости в графе
    fn find_cycles(&self, nodes: &HashMap<String, DependencyNode>, edges: &[DependencyEdge]) -> Vec<DependencyCycle> {
        let mut cycles = Vec::new();
        
        // Строим граф смежности
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        for edge in edges {
            graph.entry(edge.from.clone())
                .or_default()
                .push(edge.to.clone());
        }
        
        // Используем DFS для поиска циклов
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        
        for node_name in nodes.keys() {
            if !visited.contains(node_name) {
                if let Some(cycle) = self.dfs_find_cycle(node_name, &graph, &mut visited, &mut rec_stack, edges) {
                    cycles.push(cycle);
                }
            }
        }
        
        cycles
    }
    
    /// DFS поиск циклов
    fn dfs_find_cycle(
        &self,
        node: &str,
        graph: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        edges: &[DependencyEdge],
    ) -> Option<DependencyCycle> {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        
        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    if let Some(cycle) = self.dfs_find_cycle(neighbor, graph, visited, rec_stack, edges) {
                        return Some(cycle);
                    }
                } else if rec_stack.contains(neighbor) {
                    // Найден цикл
                    let cycle_edges: Vec<DependencyEdge> = edges.iter()
                        .filter(|e| e.from == *node && e.to == *neighbor)
                        .cloned()
                        .collect();
                    
                    return Some(DependencyCycle {
                        modules: vec![node.to_string(), neighbor.to_string()],
                        edges: cycle_edges,
                        severity: CycleSeverity::Warning,
                    });
                }
            }
        }
        
        rec_stack.remove(node);
        None
    }
    
    /// Вычисляет статистику зависимостей
    fn calculate_statistics(
        &self,
        nodes: &HashMap<String, DependencyNode>,
        edges: &[DependencyEdge],
        cycles: &[DependencyCycle],
    ) -> DependencyStatistics {
        let total_modules = nodes.len();
        let total_dependencies = edges.len();
        let circular_dependencies = cycles.len();
        
        // Подсчитываем неиспользуемые экспорты
        let mut used_symbols = HashSet::new();
        for edge in edges {
            for symbol in &edge.used_symbols {
                used_symbols.insert(symbol.clone());
            }
        }
        
        let mut total_exports = 0;
        for node in nodes.values() {
            total_exports += node.exports.len();
        }
        
        let unused_exports = total_exports.saturating_sub(used_symbols.len());
        
        // Вычисляем среднее количество зависимостей на модуль
        let avg_dependencies_per_module = if total_modules > 0 {
            total_dependencies as f64 / total_modules as f64
        } else {
            0.0
        };
        
        DependencyStatistics {
            total_modules,
            total_dependencies,
            circular_dependencies,
            unused_exports,
            max_depth: self.calculate_max_depth(nodes, edges),
            avg_dependencies_per_module,
        }
    }
    
    /// Вычисляет максимальную глубину зависимостей
    fn calculate_max_depth(&self, nodes: &HashMap<String, DependencyNode>, edges: &[DependencyEdge]) -> usize {
        // Строим граф и используем BFS для поиска максимальной глубины
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        for edge in edges {
            graph.entry(edge.from.clone())
                .or_default()
                .push(edge.to.clone());
        }
        
        let mut max_depth = 0;
        
        for start_node in nodes.keys() {
            let depth = self.bfs_max_depth(start_node, &graph);
            max_depth = max_depth.max(depth);
        }
        
        max_depth
    }
    
    /// BFS для поиска максимальной глубины от узла
    fn bfs_max_depth(&self, start: &str, graph: &HashMap<String, Vec<String>>) -> usize {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut max_depth = 0;
        
        queue.push_back((start.to_string(), 0));
        visited.insert(start.to_string());
        
        while let Some((node, depth)) = queue.pop_front() {
            max_depth = max_depth.max(depth);
            
            if let Some(neighbors) = graph.get(&node) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        visited.insert(neighbor.clone());
                        queue.push_back((neighbor.clone(), depth + 1));
                    }
                }
            }
        }
        
        max_depth
    }
}

impl Default for DependencyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyGraph {
    /// Возвращает модули с наибольшим количеством зависимостей
    pub fn get_most_dependent_modules(&self, limit: usize) -> Vec<(&String, usize)> {
        let mut dependencies_count: HashMap<&String, usize> = HashMap::new();
        
        for edge in &self.edges {
            *dependencies_count.entry(&edge.from).or_insert(0) += 1;
        }
        
        let mut sorted: Vec<_> = dependencies_count.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.truncate(limit);
        
        sorted
    }
    
    /// Возвращает модули, от которых больше всего зависят
    pub fn get_most_depended_on_modules(&self, limit: usize) -> Vec<(&String, usize)> {
        let mut dependents_count: HashMap<&String, usize> = HashMap::new();
        
        for edge in &self.edges {
            *dependents_count.entry(&edge.to).or_insert(0) += 1;
        }
        
        let mut sorted: Vec<_> = dependents_count.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.truncate(limit);
        
        sorted
    }
    
    /// Возвращает неиспользуемые экспорты
    pub fn get_unused_exports(&self) -> Vec<&ExportedSymbol> {
        let mut used_symbols = HashSet::new();
        
        // Собираем все используемые символы
        for edge in &self.edges {
            for symbol in &edge.used_symbols {
                used_symbols.insert(symbol);
            }
        }
        
        // Находим неиспользуемые экспорты
        let mut unused = Vec::new();
        for node in self.nodes.values() {
            for export in &node.exports {
                if !used_symbols.contains(&export.name) {
                    unused.push(export);
                }
            }
        }
        
        unused
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    
    fn create_test_module(dir: &std::path::Path, name: &str, content: &str) -> BslModule {
        let module_path = dir.join(format!("{}.bsl", name));
        fs::write(&module_path, content).unwrap();
        
        BslModule {
            name: name.to_string(),
            path: module_path,
            exports: Vec::new(),
            imports: Vec::new(),
            module_type: crate::configuration::ModuleType::CommonModule,
        }
    }
    
    #[test]
    fn test_dependency_analyzer_creation() {
        let analyzer = DependencyAnalyzer::new();
        assert!(!analyzer.parser.parse_text("").is_err());
    }
    
    #[test]
    fn test_module_analysis() {
        let temp_dir = TempDir::new().unwrap();
        let analyzer = DependencyAnalyzer::new();
        
        let module_content = r#"
        Процедура ТестоваяПроцедура() Экспорт
            Сообщить("Тест");
        КонецПроцедуры
        
        Процедура ВызовВнешнегоМодуля()
            ДругойМодуль.ВнешняяПроцедура();
        КонецПроцедуры
        "#;
        
        let module = create_test_module(temp_dir.path(), "ТестовыйМодуль", module_content);
        let node = analyzer.analyze_module(&module).unwrap();
        
        assert_eq!(node.module_name, "ТестовыйМодуль");
        assert_eq!(node.module_type, ModuleType::CommonModule);
        assert!(!node.exports.is_empty());
        assert!(!node.imports.is_empty());
    }
    
    #[test]
    fn test_export_extraction() {
        let analyzer = DependencyAnalyzer::new();
        let parser = BslParser::new();
        
        let content = r#"
        Процедура ЭкспортнаяПроцедура() Экспорт
            // Экспортная процедура
        КонецПроцедуры
        
        Функция ЭкспортнаяФункция() Экспорт
            Возврат "тест";
        КонецФункции
        "#;
        
        let ast = parser.parse_text(content).unwrap();
        let exports = analyzer.extract_exports(&ast, "ТестМодуль").unwrap();
        
        assert_eq!(exports.len(), 2);
        assert!(exports.iter().any(|e| e.name.contains("ЭкспортнаяПроцедура")));
        assert!(exports.iter().any(|e| e.name.contains("ЭкспортнаяФункция")));
    }
    
    #[test]
    fn test_import_extraction() {
        let analyzer = DependencyAnalyzer::new();
        let parser = BslParser::new();
        
        let content = r#"
        Процедура ТестоваяПроцедура()
            ВнешнийМодуль.ВнешняяПроцедура();
            АльтернативныйМодуль.АльтернативнаяФункция();
        КонецПроцедуры
        "#;
        
        let ast = parser.parse_text(content).unwrap();
        let imports = analyzer.extract_imports(&ast, "ТестМодуль").unwrap();
        
        assert!(!imports.is_empty());
        assert!(imports.iter().any(|i| i.source_module == "ВнешнийМодуль"));
        assert!(imports.iter().any(|i| i.source_module == "АльтернативныйМодуль"));
    }
    
    #[test]
    fn test_module_type_determination() {
        let analyzer = DependencyAnalyzer::new();
        
        let common_path = std::path::Path::new("CommonModules/ОбщийМодуль/Module.bsl");
        let form_path = std::path::Path::new("DataProcessors/Обработка/Forms/Форма/Form.bsl");
        
        assert_eq!(analyzer.determine_module_type(common_path, ""), ModuleType::CommonModule);
        assert_eq!(analyzer.determine_module_type(form_path, ""), ModuleType::FormModule);
    }
    
    #[test]
    fn test_execution_context_determination() {
        let analyzer = DependencyAnalyzer::new();
        
        let server_content = "&НаСервере\nПроцедура СерверПроцедура() КонецПроцедуры";
        let client_content = "&НаКлиенте\nПроцедура КлиентПроцедура() КонецПроцедуры";
        let client_server_content = "Процедура ОбщаяПроцедура() КонецПроцедуры";
        
        assert_eq!(analyzer.determine_execution_context(server_content, &ModuleType::CommonModule), ExecutionContext::Server);
        assert_eq!(analyzer.determine_execution_context(client_content, &ModuleType::CommonModule), ExecutionContext::Client);
        assert_eq!(analyzer.determine_execution_context(client_server_content, &ModuleType::CommonModule), ExecutionContext::ClientServer);
    }
    
    #[test]
    fn test_dependency_statistics() {
        let analyzer = DependencyAnalyzer::new();
        
        let nodes = HashMap::new();
        let edges = Vec::new();
        let cycles = Vec::new();
        
        let stats = analyzer.calculate_statistics(&nodes, &edges, &cycles);
        
        assert_eq!(stats.total_modules, 0);
        assert_eq!(stats.total_dependencies, 0);
        assert_eq!(stats.circular_dependencies, 0);
        assert_eq!(stats.avg_dependencies_per_module, 0.0);
    }
}