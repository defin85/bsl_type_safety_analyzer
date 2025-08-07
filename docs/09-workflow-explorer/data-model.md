# Модель данных Workflow Analyzer

## Основные структуры данных

### 1. Граф Workflow

```rust
use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};

/// Уникальный идентификатор узла в графе
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct NodeId(String);

/// Основная структура графа workflow
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowGraph {
    /// Узлы графа
    pub nodes: HashMap<NodeId, WorkflowNode>,
    
    /// Рёбра графа
    pub edges: Vec<WorkflowEdge>,
    
    /// Метаданные графа
    pub metadata: GraphMetadata,
    
    /// Индекс для быстрого поиска
    pub index: GraphIndex,
}

/// Метаданные графа
#[derive(Debug, Serialize, Deserialize)]
pub struct GraphMetadata {
    /// Название workflow
    pub name: String,
    
    /// Тип workflow
    pub workflow_type: WorkflowType,
    
    /// Дата создания анализа
    pub created_at: chrono::DateTime<chrono::Utc>,
    
    /// Версия конфигурации
    pub config_version: String,
    
    /// Статистика
    pub stats: GraphStatistics,
}

/// Индексы для быстрого поиска
#[derive(Debug, Default)]
pub struct GraphIndex {
    /// Индекс по типу узла
    pub by_type: HashMap<NodeType, HashSet<NodeId>>,
    
    /// Индекс входящих рёбер
    pub incoming: HashMap<NodeId, Vec<EdgeId>>,
    
    /// Индекс исходящих рёбер
    pub outgoing: HashMap<NodeId, Vec<EdgeId>>,
    
    /// Индекс по модулю
    pub by_module: HashMap<String, HashSet<NodeId>>,
}
```

### 2. Узлы графа (Nodes)

```rust
/// Узел в графе workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNode {
    /// Идентификатор узла
    pub id: NodeId,
    
    /// Тип узла
    pub node_type: NodeType,
    
    /// Данные узла
    pub data: NodeData,
    
    /// Метаданные узла
    pub metadata: NodeMetadata,
    
    /// Позиция для визуализации (опционально)
    pub position: Option<Position>,
}

/// Типы узлов
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    Procedure,
    Function,
    Document,
    Register,
    Report,
    Processing,
    BusinessProcess,
    Task,
    Event,
    Decision,
    DataStore,
    ExternalSystem,
}

/// Данные узла в зависимости от типа
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NodeData {
    /// Процедура или функция
    Procedure {
        module_path: String,
        name: String,
        export: bool,
        params: Vec<Parameter>,
        return_type: Option<String>,
        complexity: u32,
    },
    
    /// Документ
    Document {
        name: String,
        type_path: String,
        state: DocumentState,
        registers_affected: Vec<String>,
    },
    
    /// Бизнес-процесс
    BusinessProcess {
        name: String,
        current_stage: String,
        stages: Vec<ProcessStage>,
        route_points: Vec<RoutePoint>,
    },
    
    /// Событие
    Event {
        object: String,
        event_type: EventType,
        handler_module: String,
        async: bool,
    },
    
    /// Точка принятия решения
    Decision {
        condition: Condition,
        branches: Vec<Branch>,
        default_branch: Option<NodeId>,
    },
    
    /// Регистр
    Register {
        name: String,
        register_type: RegisterType,
        dimensions: Vec<String>,
        resources: Vec<String>,
    },
    
    /// Внешняя система
    ExternalSystem {
        name: String,
        system_type: String,
        connection_params: HashMap<String, String>,
    },
}

/// Метаданные узла
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetadata {
    /// Описание узла
    pub description: Option<String>,
    
    /// Метки/теги
    pub tags: HashSet<String>,
    
    /// Метрики узла
    pub metrics: NodeMetrics,
    
    /// Исходный код (для процедур/функций)
    pub source_location: Option<SourceLocation>,
}

/// Метрики узла
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetrics {
    /// Частота использования
    pub usage_frequency: u32,
    
    /// Среднее время выполнения (мс)
    pub avg_execution_time: Option<f64>,
    
    /// Количество ошибок
    pub error_count: u32,
    
    /// Важность узла (0-100)
    pub importance: f32,
}
```

### 3. Рёбра графа (Edges)

```rust
/// Идентификатор ребра
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct EdgeId(String);

/// Ребро графа - связь между узлами
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEdge {
    /// Идентификатор ребра
    pub id: EdgeId,
    
    /// Источник
    pub from: NodeId,
    
    /// Назначение
    pub to: NodeId,
    
    /// Тип связи
    pub edge_type: EdgeType,
    
    /// Данные ребра
    pub data: EdgeData,
    
    /// Метаданные ребра
    pub metadata: EdgeMetadata,
}

/// Типы связей
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeType {
    /// Вызов метода
    MethodCall,
    
    /// Поток документов
    DocumentFlow,
    
    /// Срабатывание события
    EventTrigger,
    
    /// Изменение состояния
    StateTransition,
    
    /// Поток данных
    DataFlow,
    
    /// Поток управления
    ControlFlow,
    
    /// Зависимость
    Dependency,
    
    /// Асинхронное взаимодействие
    AsyncMessage,
}

/// Данные ребра
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeData {
    /// Условие перехода
    pub condition: Option<String>,
    
    /// Передаваемые данные
    pub data_passed: Vec<DataItem>,
    
    /// Вероятность перехода (0-1)
    pub probability: Option<f32>,
    
    /// Синхронность
    pub is_async: bool,
    
    /// Транзакционность
    pub in_transaction: bool,
}

/// Метаданные ребра
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeMetadata {
    /// Метка для визуализации
    pub label: Option<String>,
    
    /// Стиль линии
    pub style: EdgeStyle,
    
    /// Вес (для алгоритмов)
    pub weight: f32,
    
    /// Критический путь
    pub is_critical_path: bool,
}
```

### 4. Вспомогательные типы

```rust
/// Параметр процедуры/функции
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub param_type: String,
    pub by_ref: bool,
    pub optional: bool,
    pub default_value: Option<String>,
}

/// Состояние документа
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentState {
    Created,
    Filled,
    Posted,
    MarkedForDeletion,
    Custom(String),
}

/// Тип события
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    BeforeWrite,
    OnWrite,
    AfterWrite,
    BeforeDelete,
    AfterDelete,
    Posting,
    UndoPosting,
    Custom(String),
}

/// Условие
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub expression: String,
    pub variables: Vec<String>,
}

/// Ветка условия
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub condition: String,
    pub target: NodeId,
    pub probability: f32,
}

/// Тип workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowType {
    BusinessProcess,
    DocumentFlow,
    CallGraph,
    EventFlow,
    DataFlow,
    Mixed,
}

/// Позиция для визуализации
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub layer: u32,
}

/// Расположение в исходном коде
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    pub file: String,
    pub line_start: usize,
    pub line_end: usize,
    pub column_start: usize,
    pub column_end: usize,
}
```

### 5. Паттерны и анализ

```rust
/// Паттерн workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowPattern {
    /// Название паттерна
    pub name: String,
    
    /// Тип паттерна
    pub pattern_type: PatternType,
    
    /// Узлы, участвующие в паттерне
    pub nodes: Vec<NodeId>,
    
    /// Рёбра паттерна
    pub edges: Vec<EdgeId>,
    
    /// Оценка паттерна
    pub assessment: PatternAssessment,
}

/// Типы паттернов
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    // Хорошие паттерны
    Pipeline,
    Saga,
    EventSourcing,
    CQRS,
    
    // Антипаттерны
    CircularDependency,
    GodObject,
    SpaghettiCode,
    BottleNeck,
    DeadCode,
}

/// Оценка паттерна
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternAssessment {
    /// Тип оценки
    pub severity: Severity,
    
    /// Описание
    pub description: String,
    
    /// Рекомендации
    pub recommendations: Vec<String>,
    
    /// Влияние на производительность
    pub performance_impact: Impact,
    
    /// Влияние на поддерживаемость
    pub maintainability_impact: Impact,
}

/// Уровень серьёзности
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Влияние
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Impact {
    None,
    Low,
    Medium,
    High,
    Critical,
}
```

### 6. Результаты анализа

```rust
/// Результат анализа workflow
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowAnalysisResult {
    /// Граф workflow
    pub graph: WorkflowGraph,
    
    /// Найденные паттерны
    pub patterns: Vec<WorkflowPattern>,
    
    /// Метрики
    pub metrics: WorkflowMetrics,
    
    /// Проблемы
    pub issues: Vec<WorkflowIssue>,
    
    /// Рекомендации
    pub recommendations: Vec<Recommendation>,
}

/// Метрики workflow
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowMetrics {
    /// Общее количество узлов
    pub total_nodes: usize,
    
    /// Общее количество рёбер
    pub total_edges: usize,
    
    /// Цикломатическая сложность
    pub cyclomatic_complexity: u32,
    
    /// Глубина графа
    pub depth: u32,
    
    /// Ширина графа
    pub breadth: u32,
    
    /// Плотность связей
    pub density: f32,
    
    /// Количество компонент связности
    pub connected_components: u32,
    
    /// Критический путь
    pub critical_path_length: u32,
    
    /// Среднее количество связей на узел
    pub avg_connections: f32,
}

/// Проблема в workflow
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowIssue {
    /// Идентификатор проблемы
    pub id: String,
    
    /// Тип проблемы
    pub issue_type: IssueType,
    
    /// Серьёзность
    pub severity: Severity,
    
    /// Затронутые узлы
    pub affected_nodes: Vec<NodeId>,
    
    /// Описание
    pub description: String,
    
    /// Предлагаемое решение
    pub solution: Option<String>,
}

/// Рекомендация
#[derive(Debug, Serialize, Deserialize)]
pub struct Recommendation {
    /// Заголовок
    pub title: String,
    
    /// Описание
    pub description: String,
    
    /// Приоритет
    pub priority: Priority,
    
    /// Ожидаемый эффект
    pub expected_impact: String,
    
    /// Шаги реализации
    pub implementation_steps: Vec<String>,
}
```

## Примеры использования

### Создание графа workflow

```rust
use workflow_analyzer::*;

fn create_document_flow() -> WorkflowGraph {
    let mut graph = WorkflowGraph::new("Процесс продажи");
    
    // Добавляем узлы
    let order_node = WorkflowNode::document(
        "ЗаказПокупателя",
        DocumentState::Created
    );
    let shipment_node = WorkflowNode::document(
        "РеализацияТоваров",
        DocumentState::Posted
    );
    
    graph.add_node(order_node);
    graph.add_node(shipment_node);
    
    // Добавляем связь
    let edge = WorkflowEdge::document_flow(
        order_node.id.clone(),
        shipment_node.id.clone(),
        "На основании"
    );
    
    graph.add_edge(edge);
    
    graph
}
```

### Анализ графа

```rust
fn analyze_workflow(graph: &WorkflowGraph) -> WorkflowAnalysisResult {
    let analyzer = WorkflowAnalyzer::new();
    
    // Поиск паттернов
    let patterns = analyzer.find_patterns(graph);
    
    // Расчёт метрик
    let metrics = analyzer.calculate_metrics(graph);
    
    // Поиск проблем
    let issues = analyzer.find_issues(graph);
    
    WorkflowAnalysisResult {
        graph: graph.clone(),
        patterns,
        metrics,
        issues,
        recommendations: analyzer.generate_recommendations(&issues),
    }
}
```