# План реализации Workflow Analyzer

## Обзор

План поэтапной реализации модуля анализа workflow для BSL Type Safety Analyzer.

## Фазы разработки

### Фаза 1: Базовая инфраструктура (2 недели)

#### Задачи
1. **Создание основных структур данных**
   - [ ] Реализовать `WorkflowGraph`, `WorkflowNode`, `WorkflowEdge`
   - [ ] Создать типы для представления различных узлов
   - [ ] Реализовать индексы для быстрого поиска

2. **Интеграция с существующим парсером**
   - [ ] Расширить `BslParser` для извлечения вызовов
   - [ ] Интегрироваться с `UnifiedBslIndex`
   - [ ] Использовать `DataFlowAnalyzer` как основу

3. **Базовый экстрактор**
   - [ ] Реализовать `WorkflowExtractor`
   - [ ] Извлечение вызовов процедур/функций
   - [ ] Построение локального графа модуля

#### Файлы для создания
```
src/workflow_analyzer/
├── mod.rs
├── graph.rs          # Структуры графа
├── extractor.rs      # Извлечение workflow
├── types.rs          # Типы данных
└── builder.rs        # Построитель графа
```

### Фаза 2: Анализ вызовов (2 недели)

#### Задачи
1. **Межмодульный анализ**
   - [ ] Связывание вызовов между модулями
   - [ ] Разрешение экспортных функций
   - [ ] Обработка динамических вызовов

2. **Анализ зависимостей**
   - [ ] Построение графа зависимостей
   - [ ] Обнаружение циклических зависимостей
   - [ ] Расчёт метрик связности

3. **Оптимизация производительности**
   - [ ] Кеширование результатов анализа
   - [ ] Параллельная обработка модулей
   - [ ] Инкрементальный анализ

#### Файлы для создания
```
src/workflow_analyzer/
├── call_graph.rs     # Граф вызовов
├── dependency.rs     # Анализ зависимостей
└── cache.rs         # Кеширование
```

### Фаза 3: Анализ документооборота (3 недели)

#### Задачи
1. **Извлечение документооборота**
   - [ ] Анализ связей между документами
   - [ ] Обработка движений регистров
   - [ ] Отслеживание состояний документов

2. **Анализ бизнес-процессов**
   - [ ] Парсинг объектов БизнесПроцесс
   - [ ] Извлечение маршрутов
   - [ ] Анализ точек маршрута

3. **Обработка событий**
   - [ ] Извлечение подписок на события
   - [ ] Построение цепочек событий
   - [ ] Анализ обработчиков

#### Файлы для создания
```
src/workflow_analyzer/
├── document_flow.rs   # Документооборот
├── business_process.rs # Бизнес-процессы
└── events.rs          # События
```

### Фаза 4: Визуализация (2 недели)

#### Задачи
1. **Генерация диаграмм**
   - [ ] Экспорт в DOT формат (Graphviz)
   - [ ] Генерация Mermaid диаграмм
   - [ ] Создание PlantUML диаграмм

2. **HTML отчёты**
   - [ ] Шаблоны отчётов
   - [ ] Интерактивные графы (D3.js)
   - [ ] Таблицы с метриками

3. **Экспорт данных**
   - [ ] JSON экспорт
   - [ ] CSV для метрик
   - [ ] Markdown документация

#### Файлы для создания
```
src/workflow_analyzer/
├── visualizer/
│   ├── mod.rs
│   ├── dot.rs        # DOT формат
│   ├── mermaid.rs    # Mermaid диаграммы
│   └── html.rs       # HTML отчёты
└── export.rs         # Экспорт данных
```

### Фаза 5: Паттерны и анализ (3 недели)

#### Задачи
1. **Распознавание паттернов**
   - [ ] Библиотека типовых паттернов
   - [ ] Алгоритмы поиска паттернов
   - [ ] Оценка найденных паттернов

2. **Обнаружение проблем**
   - [ ] Поиск антипаттернов
   - [ ] Анализ узких мест
   - [ ] Обнаружение мёртвого кода

3. **Генерация рекомендаций**
   - [ ] База знаний рекомендаций
   - [ ] Приоритизация проблем
   - [ ] Предложения по рефакторингу

#### Файлы для создания
```
src/workflow_analyzer/
├── patterns/
│   ├── mod.rs
│   ├── detector.rs   # Детектор паттернов
│   ├── library.rs    # Библиотека паттернов
│   └── assessment.rs # Оценка паттернов
└── recommendations.rs # Рекомендации
```

### Фаза 6: Интеграция с MCP (1 неделя)

#### Задачи
1. **MCP инструменты**
   - [ ] Инструмент для получения workflow
   - [ ] Инструмент для анализа процессов
   - [ ] Инструмент для генерации диаграмм

2. **API для LLM**
   - [ ] Методы запроса паттернов
   - [ ] Получение примеров процессов
   - [ ] Валидация сгенерированных workflow

#### Файлы для создания
```
src/workflow_analyzer/
└── mcp/
    ├── mod.rs
    ├── tools.rs      # MCP инструменты
    └── api.rs        # API для LLM
```

## Пример кода для начала реализации

### Базовые структуры (graph.rs)

```rust
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct NodeId(pub String);

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct EdgeId(pub String);

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowGraph {
    pub nodes: HashMap<NodeId, WorkflowNode>,
    pub edges: Vec<WorkflowEdge>,
    pub metadata: GraphMetadata,
}

impl WorkflowGraph {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            metadata: GraphMetadata::new(name),
        }
    }
    
    pub fn add_node(&mut self, node: WorkflowNode) {
        self.nodes.insert(node.id.clone(), node);
    }
    
    pub fn add_edge(&mut self, edge: WorkflowEdge) {
        self.edges.push(edge);
    }
}
```

### Экстрактор (extractor.rs)

```rust
use crate::bsl_parser::ast::*;
use super::graph::*;

pub struct WorkflowExtractor {
    current_module: String,
    graph: WorkflowGraph,
}

impl WorkflowExtractor {
    pub fn new() -> Self {
        Self {
            current_module: String::new(),
            graph: WorkflowGraph::new("Extracted Workflow"),
        }
    }
    
    pub fn extract_from_ast(&mut self, ast: &BslAst) -> Result<WorkflowGraph> {
        // Обход AST и извлечение вызовов
        self.visit_module(&ast.module)?;
        Ok(self.graph.clone())
    }
    
    fn visit_module(&mut self, module: &Module) -> Result<()> {
        for declaration in &module.declarations {
            self.visit_declaration(declaration)?;
        }
        Ok(())
    }
    
    fn visit_declaration(&mut self, decl: &Declaration) -> Result<()> {
        match decl {
            Declaration::Procedure(proc) => {
                // Создаём узел для процедуры
                let node = self.create_procedure_node(proc);
                self.graph.add_node(node);
                
                // Анализируем тело процедуры
                self.analyze_procedure_body(proc)?;
            }
            Declaration::Function(func) => {
                // Аналогично для функций
            }
            _ => {}
        }
        Ok(())
    }
}
```

### CLI инструмент (bin/workflow_analyzer.rs)

```rust
use clap::Parser;
use bsl_type_safety_analyzer::workflow_analyzer::*;

#[derive(Parser)]
#[command(name = "workflow_analyzer")]
#[command(about = "Анализатор workflow для 1С:Предприятие")]
struct Args {
    /// Путь к конфигурации
    #[arg(short, long)]
    config: String,
    
    /// Режим анализа
    #[arg(short, long, default_value = "call-graph")]
    mode: String,
    
    /// Формат вывода
    #[arg(short, long, default_value = "dot")]
    output_format: String,
    
    /// Файл для сохранения результата
    #[arg(short = 'o', long)]
    output: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Инициализация анализатора
    let analyzer = WorkflowAnalyzer::new();
    
    // Загрузка конфигурации
    let config = load_configuration(&args.config)?;
    
    // Анализ в зависимости от режима
    let graph = match args.mode.as_str() {
        "call-graph" => analyzer.build_call_graph(&config)?,
        "document-flow" => analyzer.build_document_flow(&config)?,
        "business-process" => analyzer.build_business_process(&config)?,
        _ => return Err(anyhow!("Неизвестный режим: {}", args.mode)),
    };
    
    // Визуализация результата
    let output = match args.output_format.as_str() {
        "dot" => visualizer::to_dot(&graph)?,
        "mermaid" => visualizer::to_mermaid(&graph)?,
        "json" => serde_json::to_string_pretty(&graph)?,
        _ => return Err(anyhow!("Неизвестный формат: {}", args.output_format)),
    };
    
    // Сохранение или вывод
    if let Some(output_file) = args.output {
        std::fs::write(output_file, output)?;
    } else {
        println!("{}", output);
    }
    
    Ok(())
}
```

## Тестирование

### Модульные тесты

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_graph_creation() {
        let mut graph = WorkflowGraph::new("Test");
        assert_eq!(graph.nodes.len(), 0);
        assert_eq!(graph.edges.len(), 0);
    }
    
    #[test]
    fn test_node_addition() {
        let mut graph = WorkflowGraph::new("Test");
        let node = WorkflowNode::new_procedure("TestProc");
        graph.add_node(node.clone());
        assert_eq!(graph.nodes.len(), 1);
        assert!(graph.nodes.contains_key(&node.id));
    }
}
```

### Интеграционные тесты

```rust
#[test]
fn test_extract_workflow_from_bsl() {
    let bsl_code = r#"
        Процедура ОбработатьДокумент(Документ) Экспорт
            Если Документ.Проведен Тогда
                СоздатьДвижения(Документ);
            КонецЕсли;
        КонецПроцедуры
        
        Процедура СоздатьДвижения(Документ)
            // Создание движений
        КонецПроцедуры
    "#;
    
    let ast = parse_bsl(bsl_code).unwrap();
    let mut extractor = WorkflowExtractor::new();
    let graph = extractor.extract_from_ast(&ast).unwrap();
    
    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.edges.len(), 1);
}
```

## Метрики успеха

### Производительность
- Анализ модуля в 1000 строк: < 100ms
- Построение графа для конфигурации с 1000 объектов: < 10s
- Генерация визуализации: < 1s

### Качество
- Покрытие тестами: > 80%
- Корректное извлечение вызовов: > 95%
- Распознавание паттернов: > 80%

### Юзабилити
- Понятная визуализация
- Полезные рекомендации
- Простая интеграция с существующими инструментами

## Риски и митигации

### Риск: Сложность анализа динамических вызовов
**Митигация**: Начать с анализа статических вызовов, постепенно добавлять поддержку динамических

### Риск: Производительность на больших конфигурациях
**Митигация**: Инкрементальный анализ, кеширование, параллельная обработка

### Риск: Неполнота извлечения workflow
**Митигация**: Итеративное улучшение, сбор обратной связи от пользователей

## Ресурсы и зависимости

### Необходимые библиотеки
- `petgraph` - для работы с графами
- `serde` - для сериализации
- `rayon` - для параллельной обработки
- `handlebars` - для HTML шаблонов

### Внешние инструменты
- Graphviz - для визуализации DOT файлов
- D3.js - для интерактивных графов
- Mermaid - для диаграмм в документации