# Интеграция с парсингом метаданных

## Анализ существующего парсинга

### Что уже есть в проекте

#### 1. ConfigurationXmlParser
Парсит основные объекты метаданных из Configuration.xml:
- ✅ BusinessProcesses - бизнес-процессы
- ✅ Tasks - задачи  
- ✅ Documents - документы
- ✅ Registers - регистры
- ✅ Forms - формы объектов
- ✅ CommonModules - общие модули

#### 2. BslEntity структура
Содержит полезные для workflow поля:
- `relationships` - связи между объектами
- `lifecycle` - жизненный цикл объекта
- `interface` - методы и свойства
- `tabular_sections` - табличные части

### Чего не хватает для полноценного workflow анализа

#### 1. Подписки на события (EventSubscriptions)
```xml
<!-- EventSubscriptions/ПриЗаписиДокумента/EventSubscription.xml -->
<MetaDataObject>
    <EventSubscription>
        <Source>Document.РеализацияТоваров</Source>
        <Event>ПриЗаписи</Event>
        <Handler>CommonModule.УправлениеЗапасами.ПриЗаписиДокументаРеализации</Handler>
    </EventSubscription>
</MetaDataObject>
```

#### 2. Регламентные задания (ScheduledJobs)
```xml
<!-- ScheduledJobs/ОбновлениеОстатков/ScheduledJob.xml -->
<MetaDataObject>
    <ScheduledJob>
        <MethodName>РегламентныеЗадания.ОбновитьОстатки</MethodName>
        <Schedule>
            <RepeatPeriodInSeconds>3600</RepeatPeriodInSeconds>
        </Schedule>
    </ScheduledJob>
</MetaDataObject>
```

#### 3. Маршруты бизнес-процессов
```xml
<!-- BusinessProcesses/Согласование/Ext/RouteMap.xml -->
<RouteMap>
    <RoutePoints>
        <RoutePoint name="Старт" type="Start"/>
        <RoutePoint name="Согласование" type="Activity">
            <Handler>ПриСогласовании</Handler>
            <NextPoint condition="Результат = Утверждено">Исполнение</NextPoint>
            <NextPoint condition="Результат = Отклонено">Доработка</NextPoint>
        </RoutePoint>
        <RoutePoint name="Исполнение" type="Activity"/>
        <RoutePoint name="Завершение" type="End"/>
    </RoutePoints>
</RouteMap>
```

#### 4. Последовательности документов
```xml
<!-- Documents/РеализацияТоваров/Document.xml -->
<MetaDataObject>
    <Document>
        <BasedOn>Document.ЗаказПокупателя</BasedOn>
        <RegisterRecords>
            <Register>AccumulationRegister.ТоварыНаСкладах</Register>
            <Register>AccountingRegister.Хозрасчетный</Register>
        </RegisterRecords>
    </Document>
</MetaDataObject>
```

## Расширение парсера для workflow

### 1. Новые структуры данных

```rust
/// Подписка на событие
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSubscription {
    pub name: String,
    pub source: EventSource,
    pub event: EventType,
    pub handler: HandlerReference,
    pub condition: Option<String>,
}

/// Источник события
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventSource {
    Document(String),
    Catalog(String),
    Register(String),
    Form(String),
    AllDocuments,
    AllCatalogs,
}

/// Ссылка на обработчик
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandlerReference {
    pub module: String,
    pub procedure: String,
    pub is_export: bool,
}

/// Регламентное задание
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledJob {
    pub name: String,
    pub method: HandlerReference,
    pub schedule: Schedule,
    pub description: Option<String>,
    pub use_: bool,
}

/// Маршрут бизнес-процесса
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessProcessRoute {
    pub process_name: String,
    pub route_points: Vec<RoutePoint>,
    pub transitions: Vec<RouteTransition>,
}

/// Точка маршрута
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutePoint {
    pub name: String,
    pub point_type: RoutePointType,
    pub handler: Option<HandlerReference>,
    pub performer: Option<String>,
}

/// Переход между точками
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteTransition {
    pub from: String,
    pub to: String,
    pub condition: Option<String>,
    pub probability: Option<f32>,
}

/// Цепочка документов
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChain {
    pub document: String,
    pub based_on: Vec<String>,
    pub creates: Vec<String>,
    pub affects_registers: Vec<String>,
    pub posting_mode: PostingMode,
}
```

### 2. Расширение ConfigurationXmlParser

```rust
impl ConfigurationXmlParser {
    /// Парсинг подписок на события
    pub fn parse_event_subscriptions(&self) -> Result<Vec<EventSubscription>> {
        let mut subscriptions = Vec::new();
        let subs_path = self.config_path.join("EventSubscriptions");
        
        if !subs_path.exists() {
            return Ok(subscriptions);
        }
        
        for entry in fs::read_dir(&subs_path)? {
            let entry = entry?;
            let sub_path = entry.path();
            
            if sub_path.is_dir() {
                let xml_path = sub_path.join("EventSubscription.xml");
                if xml_path.exists() {
                    let subscription = self.parse_event_subscription(&xml_path)?;
                    subscriptions.push(subscription);
                }
            }
        }
        
        Ok(subscriptions)
    }
    
    /// Парсинг регламентных заданий
    pub fn parse_scheduled_jobs(&self) -> Result<Vec<ScheduledJob>> {
        let mut jobs = Vec::new();
        let jobs_path = self.config_path.join("ScheduledJobs");
        
        if !jobs_path.exists() {
            return Ok(jobs);
        }
        
        for entry in fs::read_dir(&jobs_path)? {
            let entry = entry?;
            let job_path = entry.path();
            
            if job_path.is_dir() {
                let xml_path = job_path.join("ScheduledJob.xml");
                if xml_path.exists() {
                    let job = self.parse_scheduled_job(&xml_path)?;
                    jobs.push(job);
                }
            }
        }
        
        Ok(jobs)
    }
    
    /// Парсинг маршрутов бизнес-процессов
    pub fn parse_business_process_routes(&self) -> Result<Vec<BusinessProcessRoute>> {
        let mut routes = Vec::new();
        let bp_path = self.config_path.join("BusinessProcesses");
        
        if !bp_path.exists() {
            return Ok(routes);
        }
        
        for entry in fs::read_dir(&bp_path)? {
            let entry = entry?;
            let process_path = entry.path();
            
            if process_path.is_dir() {
                let route_map = process_path.join("Ext").join("RouteMap.xml");
                if route_map.exists() {
                    let route = self.parse_route_map(&route_map)?;
                    routes.push(route);
                }
            }
        }
        
        Ok(routes)
    }
    
    /// Построение цепочек документов
    pub fn build_document_chains(&self, entities: &[BslEntity]) -> Vec<DocumentChain> {
        let mut chains = Vec::new();
        
        for entity in entities {
            if entity.entity_kind == BslEntityKind::Document {
                let chain = DocumentChain {
                    document: entity.qualified_name.clone(),
                    based_on: self.extract_based_on(entity),
                    creates: self.extract_creates(entity),
                    affects_registers: self.extract_registers(entity),
                    posting_mode: self.extract_posting_mode(entity),
                };
                chains.push(chain);
            }
        }
        
        chains
    }
}
```

## Интеграция с Workflow Analyzer

### 1. Обогащение графа метаданными

```rust
pub struct MetadataEnrichedWorkflow {
    /// Базовый граф из анализа кода
    pub code_graph: WorkflowGraph,
    
    /// Подписки на события
    pub event_subscriptions: Vec<EventSubscription>,
    
    /// Регламентные задания
    pub scheduled_jobs: Vec<ScheduledJob>,
    
    /// Маршруты бизнес-процессов
    pub business_routes: Vec<BusinessProcessRoute>,
    
    /// Цепочки документов
    pub document_chains: Vec<DocumentChain>,
}

impl MetadataEnrichedWorkflow {
    /// Объединение данных из кода и метаданных
    pub fn merge_metadata(&mut self) {
        // Добавляем узлы для подписок на события
        for subscription in &self.event_subscriptions {
            self.add_event_subscription_nodes(subscription);
        }
        
        // Добавляем узлы для регламентных заданий
        for job in &self.scheduled_jobs {
            self.add_scheduled_job_nodes(job);
        }
        
        // Добавляем маршруты бизнес-процессов
        for route in &self.business_routes {
            self.add_business_route_nodes(route);
        }
        
        // Связываем документы в цепочки
        for chain in &self.document_chains {
            self.add_document_chain_edges(chain);
        }
    }
    
    fn add_event_subscription_nodes(&mut self, subscription: &EventSubscription) {
        // Создаём узел события
        let event_node = WorkflowNode {
            id: NodeId(format!("Event:{}:{}", subscription.source, subscription.event)),
            node_type: NodeType::Event,
            data: NodeData::Event {
                object: format!("{:?}", subscription.source),
                event_type: subscription.event.clone(),
                handler_module: subscription.handler.module.clone(),
                async_: false,
            },
            metadata: Default::default(),
            position: None,
        };
        
        // Создаём узел обработчика
        let handler_node = WorkflowNode {
            id: NodeId(format!("Handler:{}.{}", 
                subscription.handler.module, 
                subscription.handler.procedure)),
            node_type: NodeType::Procedure,
            data: NodeData::Procedure {
                module_path: subscription.handler.module.clone(),
                name: subscription.handler.procedure.clone(),
                export: subscription.handler.is_export,
                params: vec![],
                return_type: None,
                complexity: 0,
            },
            metadata: Default::default(),
            position: None,
        };
        
        // Добавляем узлы в граф
        self.code_graph.add_node(event_node.clone());
        self.code_graph.add_node(handler_node.clone());
        
        // Создаём связь событие -> обработчик
        let edge = WorkflowEdge {
            id: EdgeId(format!("{}->{}",
                event_node.id.0,
                handler_node.id.0)),
            from: event_node.id,
            to: handler_node.id,
            edge_type: EdgeType::EventTrigger,
            data: EdgeData {
                condition: subscription.condition.clone(),
                data_passed: vec![],
                probability: None,
                is_async: false,
                in_transaction: true,
            },
            metadata: EdgeMetadata {
                label: Some("Подписка на событие".to_string()),
                style: EdgeStyle::Dashed,
                weight: 1.0,
                is_critical_path: false,
            },
        };
        
        self.code_graph.add_edge(edge);
    }
}
```

### 2. Анализ специфичных для 1С паттернов

```rust
pub struct OneCSpecificPatterns {
    /// Документооборот: Заказ -> Резерв -> Реализация -> Оплата
    pub document_flow_patterns: Vec<DocumentFlowPattern>,
    
    /// Согласование: Создание -> Рассмотрение -> Утверждение/Отклонение
    pub approval_patterns: Vec<ApprovalPattern>,
    
    /// Обмен данными: ПланОбмена -> Регистрация -> Выгрузка -> Загрузка
    pub data_exchange_patterns: Vec<DataExchangePattern>,
    
    /// Закрытие периода: Проверка -> Закрытие -> Перепроведение
    pub period_closing_patterns: Vec<PeriodClosingPattern>,
}

impl OneCSpecificPatterns {
    pub fn detect_from_metadata(workflow: &MetadataEnrichedWorkflow) -> Self {
        let mut patterns = Self::default();
        
        // Поиск паттернов документооборота
        patterns.document_flow_patterns = Self::find_document_flows(&workflow.document_chains);
        
        // Поиск паттернов согласования
        patterns.approval_patterns = Self::find_approval_processes(&workflow.business_routes);
        
        // И так далее...
        
        patterns
    }
}
```

## Практические примеры использования

### Пример 1: Анализ документооборота продаж

```rust
// Загружаем метаданные
let parser = ConfigurationXmlParser::new("path/to/config");
let entities = parser.parse_configuration()?;
let chains = parser.build_document_chains(&entities);

// Находим цепочку продаж
let sales_chain = chains.iter()
    .filter(|c| c.document.contains("Заказ") || 
                c.document.contains("Реализация"))
    .collect::<Vec<_>>();

// Строим граф документооборота
let mut workflow = WorkflowGraph::new("Процесс продаж");
for chain in sales_chain {
    workflow.add_document_chain(chain);
}

// Визуализируем
let dot = workflow.to_dot();
```

### Пример 2: Анализ подписок на события

```rust
// Парсим подписки
let subscriptions = parser.parse_event_subscriptions()?;

// Группируем по объектам
let mut by_object: HashMap<String, Vec<EventSubscription>> = HashMap::new();
for sub in subscriptions {
    by_object.entry(sub.source.to_string())
        .or_default()
        .push(sub);
}

// Находим "горячие точки" - объекты с множеством подписок
let hot_spots: Vec<_> = by_object.iter()
    .filter(|(_, subs)| subs.len() > 5)
    .collect();

println!("Объекты с большим количеством подписок:");
for (object, subs) in hot_spots {
    println!("  {} - {} подписок", object, subs.len());
}
```

## Выводы и рекомендации

### Преимущества интеграции с метаданными:

1. **Полнота картины** - видим не только код, но и декларативные связи
2. **Понимание бизнес-логики** - маршруты процессов, цепочки документов
3. **Обнаружение скрытых связей** - через подписки и регламентные задания
4. **Специфика 1С** - учитываем особенности платформы

### План интеграции:

1. **Фаза 1**: Расширить парсер для EventSubscriptions и ScheduledJobs
2. **Фаза 2**: Добавить парсинг маршрутов бизнес-процессов
3. **Фаза 3**: Построить цепочки документов из метаданных
4. **Фаза 4**: Объединить с графом вызовов из кода
5. **Фаза 5**: Реализовать детекторы 1С-специфичных паттернов

### Критические замечания:

1. **Всё ещё неполная картина** - динамические вызовы остаются проблемой
2. **Сложность визуализации** - граф может стать огромным
3. **Производительность** - парсинг всех метаданных может быть медленным
4. **Поддержка версий** - разные версии платформы имеют разную структуру XML